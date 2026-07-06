# Summary

This PR adds an optional Hyperdimensional Computing (HDC) integration for IronClaw workspace memory. HDC provides deterministic 10,240-bit binary fingerprints for documents, plus local similarity operations that can support memory deduplication, heartbeat novelty checks, and search-ranking experiments.

The integration is opt-in:

- Compile-time gate: `--features hdc`
- Runtime defaults: off unless the relevant HDC env vars are enabled
- First production posture: shadow/observe before warning, blocking, or suppressing

# Context

IronClaw workspace memory stores durable notes, decisions, daily logs, identity files, and project context. Over time, agents can write the same observation or decision multiple times with small formatting or wording changes. That makes search noisier and wastes context on redundant memory.

HDC is used here as a local fingerprinting signal, not as a replacement for full-text search or neural embeddings. Each fingerprint is a fixed 1,280-byte value. Similarity is normalized Hamming similarity over binary vectors. The core operations are CPU-local integer operations and do not call an embedding provider.

The intended use is conservative:

- Store fingerprints in shadow mode first.
- Use `memory_write` dedup warnings or duplicate blocks only when explicitly configured.
- Keep search HDC in comparison/shadow mode until retrieval quality is measured.
- Keep heartbeat repeated-notification suppression default-off.

# What Changed

## Core HDC crate

- Adds `crates/ironclaw_hdc`, a standalone crate with no dependency on the main IronClaw crate.
- Implements:
  - `HdcVector`: fixed 10,240-bit vector stored as `[u64; 160]`.
  - XOR binding, cyclic permutation, majority bundling, decaying bundling, serialization, Hamming distance, and normalized similarity.
  - Deterministic seeded vectors using stable hashing/PRNG construction.
  - `Codebook` and `ItemMemory` for symbol vectors and nearest-neighbor lookup.
  - Document/text encoders.
  - Dedup decisions: `Unique`, `Similar`, `Duplicate`.
  - Linear top-k scan helpers.
- Document encoding currently combines:
  - content byte-trigram signal, weight 5
  - metadata tags, weight 3
  - path segments, weight 2

## Workspace storage

- Adds nullable HDC fingerprint storage to `memory_documents`:
  - PostgreSQL: `hdc_fingerprint BYTEA NULL` via `migrations/V33__memory_hdc_fingerprint.sql`
  - libSQL: `hdc_fingerprint BLOB NULL` via incremental migration
- Adds DB trait methods to update and list document fingerprints.
- Enforces fingerprint length at 1,280 bytes.
- Preserves existing documents with `NULL` fingerprints until they are rewritten or backfilled.
- Lists fingerprints using user/agent scoping that mirrors normal workspace reads.

## Shadow fingerprinting

- Adds `Workspace::with_hdc_fingerprint_shadow(true)`.
- When shadow mode is enabled and the `hdc` feature is compiled in, workspace writes compute and store fingerprints after successful writes.
- Covered paths include normal writes, appends, patches, layer writes/appends, and memory append paths.
- Live workspace fingerprinting currently encodes `DocumentEncodingInput { content, tags: &[], path }`; backfill/tag-aware behavior must be normalized before tag-sensitive promotion.
- HDC fingerprint failures are fail-open: they are logged at debug level and do not block the write.
- Engine runtime paths are excluded from fingerprinting.

## Write-time dedup preflight

- Adds `Workspace::check_dedup(&self, path: &str, content: &str, config: &DedupConfig) -> Result<DedupDecision<Uuid>, WorkspaceError>` as the reusable preflight primitive.
- Wires active caller behavior through `MemoryWriteTool` only.
- Active dedup currently applies only to configured non-append `memory_write`
  calls; `append=true`, daily-log append behavior, and direct
  `Workspace::write/append/patch` calls remain behavior-preserving.
- Active dedup is controlled by:
  - `IRONCLAW_HDC_DEDUP_MODE=off|warn|block`
  - `IRONCLAW_HDC_SIMILAR_THRESHOLD`, default `0.78`
  - `IRONCLAW_HDC_DUPLICATE_THRESHOLD`, default `0.92`
- `warn` mode writes the document and attaches a `dedup` warning to tool output.
- `block` mode rejects duplicate writes and returns the existing path/id/similarity.
- `force=true` bypasses active dedup; a fingerprint is recorded only when
  workspace HDC shadow fingerprinting is also enabled.
- Direct `Workspace::write/append/patch` calls remain behavior-preserving and do not block or warn by themselves.

## Search support

- Extends `SearchConfig` and `SearchResult` with HDC fields:
  - `use_hdc`
  - `hdc_weight`
  - `hdc_shadow_mode`
  - `hdc_rank`
  - `hdc_score`
- When enabled, HDC scores are computed for documents with fingerprints and attached to existing FTS/vector search results.
- Current HDC scoring loads fingerprints for `self.user_id` and nullable
  `agent_id`; additional read scopes are not included in HDC scoring yet.
- In shadow mode, HDC does not change final ordering.
- In live mode, HDC contributes a weighted score boost and results are re-sorted.
- Adds a CLI comparison path for viewing FTS, vector, and HDC rank differences.
- Rollout rule: search is off by default; the first enabled mode should be
  shadow annotation. Live reranking needs an explicit config value and
  fixture-backed retrieval-quality gate.

## Heartbeat novelty

- Adds optional heartbeat HDC novelty detection behind `HEARTBEAT_HDC_*` env vars.
- Encodes `NeedsAttention` heartbeat text and compares it to an in-memory decaying HDC accumulator.
- Classifies observations as `Novel`, `Uncertain`, or `Repeated`.
- `HEARTBEAT_HDC_SUPPRESS_REPEATED=false` by default, so observe-only behavior is the default.
- `HEARTBEAT_OK` bypasses HDC processing.

## CLI support

- Adds HDC memory subcommands behind the `hdc` feature:
  - backfill fingerprints for existing documents
  - compare search rankings with HDC in shadow mode via
    `ironclaw memory hdc search-compare "<query>" [--limit N]`
- Backfill supports `--user`, `--limit`, `--dry-run`, and `--batch-size`.

## Fixtures and tests

- Adds `tests/fixtures/hdc_memory/` with labeled dedup, heartbeat, and search cases.
- Adds unit, property, golden, integration, caller-level, and performance-smoke coverage for the HDC paths.
- Adds tests that verify shadow fingerprints are not exposed in default `memory_write` output.

# Real Examples / Use Cases

## Repeated memory write

An agent writes `notes/decision.md` with a project decision. Later it writes the same decision to `notes/decision-v2.md` with minor formatting changes.

With HDC shadow mode only:

- Both writes still succeed.
- Fingerprints are stored for later evaluation.
- Tool output remains the same as before.

With `IRONCLAW_HDC_DEDUP_MODE=warn`:

- The call must be a non-append write, e.g. `append=false`; current append and
  daily-log paths skip active dedup.
- The second write still succeeds.
- The `memory_write` result includes `dedup.decision="similar"` or `"duplicate"`, plus the existing path/id/similarity.

With `IRONCLAW_HDC_DEDUP_MODE=block`:

- The call must be a non-append write, e.g. `append=false`.
- A duplicate second write returns `status="blocked"`.
- The caller can retry with `force=true` if the duplicate is intentional.

## Near-duplicate project notes

Two notes both discuss a Rust async design, but one is only a formatting or wording variant of the other. HDC should not be treated as the source of truth, but it can flag likely near-duplicates so the agent can update the existing note instead of creating another almost-identical one. A substantially expanded note should remain unique unless its fingerprint crosses the configured threshold.

## Search comparison

A query such as `project alpha status` may have useful path/structure signal that FTS/vector ranking does not expose directly. HDC search comparison can show:

- final rank
- FTS rank
- vector rank
- HDC rank
- HDC similarity
- rank delta

This is for evaluation. HDC-only retrieval and default ranking promotion are not part of this PR.

## Heartbeat repeated observation

Heartbeat might report `disk space low on /data` repeatedly. With HDC enabled and suppression disabled, the runner can classify repeat observations without changing notification behavior. If later canaries show acceptable false-suppression risk, `HEARTBEAT_HDC_SUPPRESS_REPEATED=true` can suppress observations classified as repeated.

## Backfilling existing memory

Existing memory rows have `NULL` fingerprints until rewritten or backfilled. Operators can dry-run backfill for a user, then backfill a limited batch before enabling dedup warning mode.

## Roko-derived cognitive architecture examples

The Roko-derived material includes several HDC/memory patterns that are useful
as follow-up examples, but they are not dependencies of this PR and are not
required context for reviewers. A fuller standalone writeup lives in
`tmp/hdc-roko-cognitive-usecases.md`, including Rust-shaped implementation
sketches, fixture names, benchmark hooks, and diagnostic command examples. The
IronClaw-specific advanced integration map is in
`tmp/hdc-ironclaw-innovation-integrations.md`, and concrete fixture rows,
telemetry shapes, benchmark commands, and promotion gates are in
`tmp/hdc-innovation-examples-and-benchmarks.md`.

- **Context assembly dedup:** gather knowledge, episodes, files, and recent
  signals, then remove HDC-similar chunks before prompt budgeting. This can
  quantify token savings and prevent daily logs from crowding out durable
  runbooks. The proposed implementation keeps the higher-authority chunk and
  records dropped chunk ids, similarity, and saved tokens.
- **Knowledge admission:** score candidate memories by confidence, source trust,
  and novelty (`1.0 - max_similarity`) before promoting them to durable memory.
  Similar anti-knowledge should surface as a warning instead of becoming
  positive advice. This should be driven through workspace/memory-tool callers,
  not a helper-only path.
- **Dream consolidation:** cluster completed turns by task shape/outcome, distill
  repeated successes into playbook candidates, and reject redundant dream output
  when HDC similarity to existing durable knowledge is too high. This remains an
  offline job until evidence and review gates exist.
- **Code structural fingerprints:** fingerprint symbols with role vectors,
  name trigrams, and surrounding context to support parity review, clone
  detection, and "find similar implementation" workflows. HDC should propose
  candidates only; AST/parser facts and caller-level tests decide correctness.
- **Heartbeat as clock:** treat repeated/novel classifications as input to
  adaptive cadence later. The first PR should only observe/log classification;
  notification metadata does not include novelty fields yet, and any suppression
  or cadence changes require false-suppression fixtures.
- **Weighted HDC search:** combine raw similarity with document weight
  (recency, confidence, pinning, or citation history) only after raw HDC search
  improves measured retrieval quality. Diagnostic output should show raw and
  weighted scores side by side.

Repo-contract guardrails applied to all of the above:

- DB behavior must go through the shared workspace store trait and maintain
  PostgreSQL/libSQL parity.
- Side-effecting behavior needs caller-level tests, especially
  `MemoryWriteTool.execute()`, workspace search, and heartbeat runner paths.
- Workspace memory semantics, user/agent scoping, identity/system-prompt loading,
  protected paths, and prompt-injection checks must remain intact.
- HDC must not create a second memory system, transcript-backed store, or
  side-channel path; fingerprints are derived metadata inside workspace memory.
- Backfill and live-write fingerprinting must use the same metadata/tag inputs
  before tag-sensitive behavior is promoted.
- `FEATURE_PARITY.md`, subsystem docs, and changelog expectations need review
  before behavior is promoted.

# Benchmark Commands and Expected Interpretation

Run the core correctness suite:

```bash
cargo test -p ironclaw_hdc
```

Expected interpretation: validates algebra, deterministic encoding, serialization, dedup decisions, and doctests. This is a correctness gate, not a performance signal.

Run core crate linting:

```bash
cargo clippy -p ironclaw_hdc --all-targets -- -D warnings
```

Expected interpretation: must remain warning-free before promotion.

Run the main crate compile check with HDC enabled:

```bash
cargo check --features hdc
```

Expected interpretation: verifies optional feature wiring compiles without requiring HDC in default builds.

Run Criterion benchmarks:

```bash
cargo bench -p ironclaw_hdc
```

Expected interpretation:

- Treat results as trend data against the prior local baseline, not as absolute pass/fail numbers.
- `bind`, `hamming_distance`, and `similarity` should remain very small single-vector operations.
- `encode_text_1kb` and `encode_document_1kb_5tags` are the write-path cost to watch.
- `dedup_scan_1k`, `dedup_scan_10k`, and `dedup_scan_100k` should scale roughly linearly with candidate count.
- Large regressions in encoding or scan cost should block enabling dedup warning/block modes by default.

Run HDC search performance smoke tests:

```bash
cargo test --features hdc hdc_search_perf -- --nocapture
```

Expected interpretation:

- These are generous wall-clock smoke checks for debug builds, not precise benchmarks.
- Current local assertions (debug-build generous thresholds):
  - 10K dedup scan under 50 ms
  - 100K dedup scan under 500 ms
  - top-k scan over 10K under 50 ms
  - top-k scan over 100K under 500 ms
- Release builds should be significantly faster (roughly 10x); use Criterion for regression analysis.
- Failures should be investigated, but Criterion is the better tool for regression analysis.

Run workspace dedup integration:

```bash
cargo test --features hdc,libsql --test workspace_hdc_dedup --no-fail-fast
```

Expected interpretation: validates `Workspace::check_dedup`, scope isolation, self-overwrite handling, default-off `MemoryWriteTool` output shape, append/daily-log exclusions, and null-fingerprint handling.

# Testing Matrix / Checklist

- [ ] `cargo test -p ironclaw_hdc`
- [ ] `cargo clippy -p ironclaw_hdc --all-targets -- -D warnings`
- [ ] `cargo check --features hdc`
- [x] `CARGO_INCREMENTAL=0 cargo test --features hdc,libsql --test workspace_hdc_dedup --no-fail-fast` — 28 passed locally
- [x] `CARGO_INCREMENTAL=0 cargo test --features hdc,libsql,integration --test workspace_hdc --no-fail-fast` — 25 passed locally
- [ ] `cargo test --features hdc,integration --test workspace_hdc_search --no-fail-fast`
- [ ] `cargo test --features hdc --test heartbeat_hdc --no-fail-fast`
- [ ] `cargo test --features hdc,integration,libsql --test heartbeat_hdc --no-fail-fast`
- [ ] `cargo test --features hdc hdc_search_perf -- --nocapture`
- [ ] `cargo bench -p ironclaw_hdc`
- [ ] PostgreSQL HDC storage contract command, once available in local CI
- [ ] Configured `MemoryWriteTool.execute()` tests for `off`, `warn`, `block`,
  and `force=true`, including output shape and no ghost document creation

Manual review checklist:

- [ ] Confirm HDC remains feature-gated and runtime-default-off.
- [ ] Confirm `memory_write` output does not include `hdc_fingerprint` in default/shadow mode.
- [ ] Confirm dedup block mode can be bypassed with `force=true`.
- [ ] Confirm configured caller tests capture `user_id`, nullable `agent_id`,
  layer/scope, path, final content, append mode, force flag, and dedup config.
- [ ] Confirm PostgreSQL and libSQL both cover migration, nullable rows,
  1,280-byte validation, scoped listing, missing-document errors, overwrite,
  and null exclusion.
- [ ] Confirm HDC search is off by default and first enabled rollout is shadow.
- [ ] Confirm prompt-injection checks still run on writes.
- [ ] Confirm multi-user and agent-scoped fingerprint listing does not cross scopes.
- [ ] Confirm existing documents with `NULL` fingerprints do not break dedup checks.
- [ ] Confirm `FEATURE_PARITY.md` was checked and either updated or recorded as
  not applicable.
- [ ] Confirm relevant docs/changelog were updated if behavior changed:
  `src/workspace/README.md`, `src/db/CLAUDE.md`, `src/tools/README.md`, setup
  docs for config/onboarding changes, API docs for web-visible behavior, and
  `CHANGELOG.md` for user-visible rollout changes.

# Known Limitations / Not In This PR

- PostgreSQL HDC storage still needs explicit integration-test coverage.
- Normal workspace shadow fingerprinting currently passes `tags: &[]`; the backfill command reads metadata tags, but the live write path does not yet thread document metadata tags into fingerprinting.
- Documents with metadata tags can therefore get different fingerprints from backfill versus later live recomputation until tag handling is normalized.
- Exact duplicate content at different paths can produce the same fingerprint because content intentionally dominates path-only differences.
- Path-sensitive search quality needs fixture-backed tuning before HDC is promoted into default ranking.
- Active dedup is not a `Workspace::write/append/patch` behavior. It is `Workspace::check_dedup` plus `MemoryWriteTool` preflight.
- Active dedup skips append-mode writes, including current daily-log append behavior.
- Layer-aware active dedup is incomplete; current tool preflight checks the primary workspace before layer routing.
- Heartbeat HDC state is in-memory only. Persistence, corrupt-state recovery, notification metadata, and trace/log assertions are pending.
- Multi-tenant heartbeat currently does not use the HDC novelty path.
- Search HDC annotates/reranks existing FTS/vector results. HDC-only retrieval is not implemented.
- Search HDC quality is not promoted by this PR; benchmark speed does not prove retrieval quality.
- `src/tools/hdc_index.rs` provides a capped skill/tool similarity index module with unit tests, but live skill/tool selection is not wired to it here.
- Benchmarks are not CI quality gates beyond existing smoke thresholds.

# Security / Privacy

- HDC encoding is local. It does not call an external model, embedding API, or network service.
- Fingerprints are content-derived workspace metadata. They should be treated as sensitive metadata and scoped like the documents they describe.
- This PR does not add encryption at rest for fingerprints.
- Fingerprints are stored in `memory_documents` and are not exposed in default `memory_write` output.
- HDC fingerprint listing is scoped by `user_id` and nullable `agent_id`, matching normal workspace read semantics.
- Dedup failures are fail-open and logged at debug level; they do not bypass normal workspace write validation.
- Prompt-injection scanning and protected path rules remain on the normal write path.
- Identity files are skipped by dedup preflight, while engine runtime paths are skipped by shadow fingerprinting.
- No bearer-token auth, CORS/origin checks, listener behavior, secret handling, or outbound HTTP policy is weakened by this integration.
- No webhook auth, body limits, rate limits, allowlists, sandboxing, or listener
  behavior is changed by this integration.
- HDC paths do not mint `TrustedInboundTurnRequest` or call trusted trigger
  submitter factories.
- Raw fingerprints should not be logged or exposed outside already-authorized
  workspace or bounded diagnostic paths.

# Rollout / Rollback

Suggested rollout:

1. Build with `--features hdc` while leaving all HDC env vars unset.
2. Enable `IRONCLAW_HDC_FINGERPRINT_SHADOW=true` for a limited environment.
3. Run backfill in dry-run mode for one user, then backfill a small limited batch.
4. Use HDC search comparison in shadow mode to inspect rank deltas without changing ranking.
5. Enable `IRONCLAW_HDC_DEDUP_MODE=warn` after reviewing false positives on local fixtures and sampled real memory.
6. Consider `IRONCLAW_HDC_DEDUP_MODE=block` only after warning-mode results are acceptable. Keep `force=true` as the documented escape hatch.
7. Enable heartbeat HDC with `HEARTBEAT_HDC_SUPPRESS_REPEATED=false` first. Only enable suppression after canary review.

Rollback:

- Set `IRONCLAW_HDC_DEDUP_MODE=off`.
- Set `IRONCLAW_HDC_FINGERPRINT_SHADOW=false`.
- Set `HEARTBEAT_HDC_ENABLED=false` or `HEARTBEAT_HDC_SUPPRESS_REPEATED=false`.
- Restart affected processes so env changes take effect.
- If needed, ship without the `hdc` feature.
- Leave the nullable `hdc_fingerprint` column and existing fingerprint bytes in place; they are inert when the feature/runtime flags are disabled.
- For an individual false duplicate block, retry the write with `force=true` or temporarily switch dedup mode to `warn`/`off`.

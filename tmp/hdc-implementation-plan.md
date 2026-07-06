# HDC Implementation Plan

Add Hyperdimensional Computing (HDC) as an optional, local similarity primitive
for IronClaw workspace memory. The first useful outcomes are memory
fingerprinting, write-time near-duplicate detection, and heartbeat novelty
detection. Search fusion is later and must earn its way in with measured
retrieval quality.

HDC does not replace FTS, neural embeddings, or RRF. It adds a deterministic
binary fingerprint that is cheap to compute, cheap to compare, offline, and easy
to feature-gate.

## Design Summary

| Decision | Choice |
|---|---|
| Core representation | 10,240-bit binary vector stored as `[u64; 160]` |
| Similarity | Normalized Hamming similarity: `1.0 - popcount(a XOR b) / 10_240` |
| Algebra | XOR bind, majority bundle, cyclic permute |
| First integration | Workspace memory fingerprinting in shadow mode |
| First behavior change | `memory_write` duplicate/similar warning path |
| Rollout | `hdc` feature flag, default off; shadow before enforcement |
| Persistence | Nullable 1,280-byte fingerprint column, PostgreSQL and libSQL |
| Required testing posture | Unit/property tests for algebra; caller-level tests for memory tool behavior |

## Non-Goals

- Do not replace existing FTS/vector search.
- Do not make HDC the only dedup mechanism. Exact hashes still matter.
- Do not block writes until false-positive risk is quantified.
- Do not add a second memory system or bypass workspace semantics.
- Do not add HDC to skill/tool routing until memory search evidence supports it.

## Current IronClaw Touchpoints

Use these as the implementation map:

| Area | Current owner |
|---|---|
| Workspace document model | `src/workspace/document.rs` |
| Workspace write/search path | `src/workspace/mod.rs` |
| Search config and fusion structs | `src/workspace/search.rs` |
| Workspace DB trait | `src/db/mod.rs` |
| PostgreSQL workspace store | `src/db/postgres.rs`, `src/workspace/repository.rs` |
| libSQL workspace store | `src/db/libsql/workspace.rs`, `src/db/libsql_migrations.rs` |
| Memory tool caller | `src/tools/builtin/memory.rs` |
| Heartbeat runner | `src/agent/heartbeat.rs` |
| CLI memory commands | `src/cli/memory.rs` |

The implementation must preserve PostgreSQL/libSQL parity, existing workspace
path rules, system-prompt injection checks, multi-user scoping, read-scope
identity isolation, and feature-default behavior.

Keep `src/main.rs` and `src/app.rs` orchestration-only. HDC config parsing,
workspace fingerprinting, dedup decisions, heartbeat novelty, search fusion, and
DB operations must live in their owning modules behind public helpers/factories.
`AppBuilder` may wire those modules together, but it must not own HDC behavior.

## Rollout Shape

1. Baseline and fixtures.
2. Core `crates/ironclaw_hdc` library.
3. Persist fingerprints in shadow mode.
4. Add write-time dedup as a caller-visible memory feature.
5. Add heartbeat novelty as observe-only, then optional suppression.
6. Evaluate HDC search fusion in shadow mode.
7. Consider skill/tool matching only if retrieval metrics justify it.

Each phase should be independently mergeable and reversible.

## Roko-Derived Architecture Inputs

The Roko-derived material includes several HDC/memory patterns that are useful
design inputs. They are not dependencies of this IronClaw work, and the
reviewer does not need that source available. The standalone source map,
implementation sketches, fixtures, benchmarks, and examples live in
`tmp/hdc-roko-cognitive-usecases.md`. The more ambitious IronClaw integration
tracks, including episode pattern memory, anti-knowledge immunity, replay
utility, cross-domain resonance, heartbeat attention, and subagent/routine
coordination, are broken out in
`tmp/hdc-ironclaw-innovation-integrations.md`. The companion benchmark and
scenario specs for those advanced tracks live in
`tmp/hdc-innovation-examples-and-benchmarks.md`.

| Roko pattern | What it suggests for IronClaw | Phase fit |
|---|---|---|
| HDC substrate with `text_query` and post-filters | Keep HDC search scoped by normal workspace filters; never expose raw cross-user scans. | Phase 5 |
| Weighted top-k HDC index | Track raw similarity first; add recency/confidence/citation weights only after quality evidence. | Later |
| Context assembly dedup | Remove near-duplicate prompt chunks before token budgeting; measure token savings and QA regressions. | Later |
| Knowledge admission gate | Use novelty, confidence, and source trust before durable memory promotion. | Later |
| Anti-knowledge | Treat repeated failed approaches as suppressive memory, not positive advice. | Later |
| Dream consolidation | Batch completed turns, cluster by task shape/outcome, and promote verified playbook candidates. | Later |
| Counterfactual strategy transfer | Compare failure clusters to successful clusters and emit draft remediation strategies. | Later |
| Code structural fingerprints | Use role vectors, name trigrams, and context for code parity/navigation. | Later |
| Heartbeat hot graph | Treat novelty as cadence/attention input after observe-only metrics, not immediate suppression. | Phase 4+ |
| Cognitive loop graph | Keep HDC integrated into sense/score/compose/persist/react boundaries instead of a side channel. | All phases |

Near-term rule: only Phase 1-5 items are part of the current HDC implementation.
Roko-inspired admission, dreams, code fingerprints, and adaptive cadence should
be tracked as future work until the memory dedup/search evidence is strong.

### Repository Contract Applied to HDC

The repo-level agent rules materially constrain this implementation:

| Rule | HDC application |
|---|---|
| Channels and tools should drive existing agent/workspace paths. | HDC must extend `Workspace`, `WorkspaceStore`, `MemoryWriteTool`, search, and heartbeat; it must not create a second memory pipeline. |
| Add DB operations to the shared trait first. | `update_document_hdc_fingerprint` and `list_document_hdc_fingerprints` need PostgreSQL and libSQL parity before workspace code depends on them. |
| Test through callers when helpers gate side effects. | A `check_dedup` unit test is not enough for blocking writes; `MemoryWriteTool.execute()` must cover warn/block/force output. |
| Keep feature-flag branching in owning modules. | Workspace fingerprinting lives in workspace code, search fusion in search/workspace code, and heartbeat novelty in heartbeat code. |
| Persistent memory is workspace memory. | Fingerprints are document metadata and must preserve file-like path semantics, user/agent scoping, chunk/search behavior, and identity/system-prompt loading. |
| Security-sensitive paths need explicit review. | HDC must not weaken protected-path checks, prompt-injection validation, bearer-token auth, CORS/origin checks, body limits, rate limits, allowlists, secrets handling, sandboxing, or outbound network policy. |
| Behavior changes require docs/status review. | Before merge, check `FEATURE_PARITY.md`, relevant subsystem docs, API docs, and changelog expectations. |

HDC must not create a transcript-backed side store or second memory system.
Backfill and live-write fingerprinting must use the same metadata/tag inputs
before tag-sensitive behavior is promoted. Product adapters, product workflow,
first-party capabilities, and host-runtime handlers must not mint
`TrustedInboundTurnRequest` or call trusted trigger submitter factories for HDC
paths.

### Roko-Inspired Follow-Up Tracks

These are the concrete future tracks from the companion doc. They should stay
separate from the first PR unless explicitly promoted and tested:

| Track | Minimal implementation | Required tests/metrics |
|---|---|---|
| Context assembly dedup | Fingerprint candidate prompt chunks, keep higher-authority duplicate, report token savings. | Recorded QA traces, dropped-cited-chunk rate, p95 prompt assembly overhead. |
| Durable admission | Combine `Workspace::check_dedup` novelty with confidence/source-trust gates. | No false suppressions, caller-level memory promotion tests, anti-knowledge fixtures. |
| Dream consolidation | Offline cluster completed turns and screen playbook candidates for HDC redundancy. | Offline fixtures, evidence-count gate, no live loop side effect. |
| Code fingerprints | Parser-backed symbol fingerprints for parity/navigation candidates. | Parser confirmation, caller-level DB parity tests, no correctness claim from HDC alone. |
| Weighted search diagnostics | Rank by raw similarity and `similarity * weight`, show both scores. | Search fixtures with NDCG/MRR, rank-delta review, raw score visibility. |
| Adaptive heartbeat | Record proposed interval from novelty/severity without changing cadence. | False-suppression fixtures, delayed-novel-finding bound, canary audit logs. |

## Current Audit Status (2026-07-04)

This section supersedes any stale checked boxes below. The detailed phase
sections remain as the implementation map, but the checklist state is:

- [x] Core `ironclaw_hdc` crate builds and passes unit, golden, property, and
  doctests with `cargo test -p ironclaw_hdc`.
- [x] Core crate clippy is clean with
  `cargo clippy -p ironclaw_hdc --all-targets -- -D warnings`.
- [x] Main crate compiles with `cargo check --features hdc`.
- [x] `DedupConfig` now rejects non-finite thresholds.
- [x] `DecayingBundleAccumulator` deserialization now revalidates decay factor.
- [x] HDC workspace env names now match `.env.example`
  (`IRONCLAW_HDC_*`) while retaining legacy `HDC_DEDUP_*` fallbacks.
- [x] HDC workspace booleans now use strict bool parsing instead of silently
  treating invalid values as false.
- [x] HDC fingerprint listing now uses the same nullable exact `agent_id`
  matching as normal workspace reads.
- [x] HDC fingerprint updates now error when the target document ID does not
  exist.
- [x] Shadow fingerprinting now recomputes after `write`, `append`, `patch`,
  layer writes/appends, and `append_memory`.
- [x] Phase 2 libSQL storage tests exist and now exercise real shadow
  fingerprint writes for workspace write/append/patch/delete paths.
- [x] Phase 2 caller tests now exercise `MemoryWriteTool.execute()` and verify
  shadow fingerprinting does not leak `hdc_fingerprint` or `dedup` fields into
  default tool output.
- [ ] PostgreSQL HDC storage contract tests still need explicit coverage.
- [ ] Workspace fingerprinting passes content and path with `tags: &[]`;
  metadata tags are not threaded into `Workspace::compute_and_store_hdc_fingerprint`.
- [ ] With no tags, the current weighted-majority encoder lets content dominate
  path-only differences, so exact duplicate content at different paths can share
  one fingerprint. That is useful for dedup, but path-sensitive search should
  not be promoted until path weighting is measured and tuned with fixtures.
- [ ] Active dedup is not a `Workspace::write/append/patch` behavior. It exists
  as `Workspace::check_dedup` plus `MemoryWriteTool` preflight for non-append
  writes.
- [ ] Configured `MemoryWriteTool` block/warn/force output assertions are
  pending; current caller tests cover default-off output shape.
- [ ] Default `memory_write` append behavior, `append=true`, and daily-log
  writes are not actively deduped.
- [ ] Layer-aware dedup is incomplete; current tool preflight checks the primary
  workspace before layer routing.
- [ ] Heartbeat HDC novelty exists in the runner loop, but most tests call
  `check_heartbeat()` and bypass `apply_hdc_novelty`.
- [ ] Heartbeat HDC state is in-memory only; persistence, corrupt-state
  recovery, notification metadata, and log-capture tests are pending.
- [ ] Multi-tenant heartbeat currently bypasses HDC novelty.
- [ ] Search HDC annotates/reranks only existing FTS/vector results. HDC-only
  search and multi-scope HDC scoring are pending.
- [ ] Benchmark numbers are measured manually/with Criterion, not enforced as
  CI assertions; documented targets need retuning against observed baselines.

---

## Phase 0: Baseline, Fixtures, and Gates

Do this before writing production integration code. HDC thresholds are easy to
overstate; the first step is a repeatable corpus and measurement harness.

### 0.1 Fixture Corpus

Create `tests/fixtures/hdc_memory/` with JSONL or Markdown plus a manifest.

**Dedup fixtures** (`dedup/`):

- [x] `exact-duplicate-different-path.jsonl` — identical content at `projects/a.md` vs `projects/b.md` (expected: duplicate)
- [x] `reformatted-duplicate.jsonl` — same prose with changed line breaks, bullet styles, header levels (expected: duplicate)
- [x] `paraphrase-duplicate.jsonl` — same information rewritten in different words, ~80% semantic overlap (expected: similar or duplicate, tune threshold)
- [x] `added-paragraph.jsonl` — original document plus one new paragraph appended (expected: similar)
- [x] `same-topic-different-facts.jsonl` — two Rust async notes that discuss different crates/patterns (expected: unique)
- [x] `same-tags-different-content.jsonl` — both tagged `["rust", "database"]` but covering unrelated topics (expected: unique, verifies tags don't dominate)
- [x] `same-path-pattern-different-content.jsonl` — `projects/alpha/status.md` vs `projects/beta/status.md` with unrelated bodies (expected: unique, verifies path doesn't dominate)
- [x] `content-subset.jsonl` — short document whose entire content appears inside a longer document (expected: similar)
- [x] `translated-content.jsonl` — same technical content in English vs Spanish (expected: similar at byte-trigram level, lower than paraphrase)

**Heartbeat fixtures** (`heartbeat/`):

- [x] `repeated-observation.jsonl` — same "disk space low on /data" finding across 10 cycles (expected: novel on cycle 1, repeated by cycle 3-4)
- [x] `recurring-after-gap.jsonl` — "disk space low" appears, then 20 cycles of different findings, then "disk space low" again (expected: novel again after decay)
- [x] `similar-but-different-metric.jsonl` — "disk at 85%" then "disk at 92%" (expected: repeated structure but novel detail — threshold sensitivity test)
- [x] `completely-novel.jsonl` — sequence of genuinely unrelated observations (expected: always novel)

**Search fixtures** (`search/`):

- [x] `structural-query.jsonl` — query "rust async database connection pooling", with labeled relevant documents (tests HDC structural match vs FTS keyword match)
- [x] `tag-heavy-query.jsonl` — query that matches document tags better than content (tests whether HDC tag encoding helps)
- [x] `path-oriented-query.jsonl` — query "project alpha status" where path structure matters (tests path encoding contribution)
- [x] `semantic-only-query.jsonl` — query "how to handle errors gracefully" where only neural embeddings should help (baseline — HDC should not regress this)

**Manifest** (`manifest.json`):

- [x] Each fixture has `expected_decision` (unique/similar/duplicate), `expected_similarity_range` (min, max), and `rationale`
- [x] Heartbeat fixtures have `expected_novelty_at_cycle` array
- [x] Search fixtures have `relevant_document_ids` and `irrelevant_document_ids`

Keep the total fixture set under 50 documents and 10 search queries. Expand
only when a regression is missed.

### 0.2 Metrics and Promotion Gates

| Metric | How to measure | Phase gate | Promotion threshold |
|---|---|---|---|
| Duplicate precision | Labeled duplicate fixtures, `check_dedup` results | Phase 3 | 1.00 — no false blocks on unique fixtures |
| Duplicate recall | Labeled exact/near duplicate fixtures | Phase 3 | Report in PR, no hard gate initially |
| Similar-warning precision | Labeled near-duplicate fixtures | Phase 3 | >= 0.90 before showing warnings to agent |
| False block rate on unique docs | Run `check_dedup` against all unique fixtures at `duplicate_threshold` | Phase 3 | 0 — any false block is a blocking regression |
| Write p95 overhead | Criterion bench: workspace write with HDC vs without, fixture corpus, 100 iterations | Phase 2 | < 2 ms delta |
| Fingerprint scan cost | Criterion bench: `check_dedup` over synthetic 10K/100K vector sets | Phase 1 | 10K < 200 μs, 100K < 2 ms |
| Encoding throughput | Criterion bench: `encode_document` over fixture corpus | Phase 1 | 1 KB document < 1 ms |
| Search NDCG@10 | Search fixtures with labeled relevant docs, compare FTS+vector vs FTS+vector+HDC | Phase 5 | No regression on baseline queries |
| Search MRR | Same fixture set | Phase 5 | No regression |
| Search overlap@5 | Compare top-5 HDC-only vs top-5 FTS+vector | Phase 5 | Report disagreement rate, no hard gate |
| Heartbeat false suppression | Labeled novel heartbeat fixtures with suppression enabled | Phase 4 | 0 — any suppressed novel observation is blocking |
| Heartbeat decay accuracy | Labeled repeated fixtures, verify cycle count to cross repeated threshold | Phase 4 | Within 2 cycles of theoretical half-life |

Each metric must be automated and runnable via `cargo test` or `cargo bench`.
Manual evaluation is acceptable for search quality in Phase 5 but must be
backed by fixture-based regression tests in CI.

### 0.3 Commands to Add Early

- `cargo test -p ironclaw_hdc`
- `cargo test -p ironclaw_hdc --no-default-features` if the crate supports
  `no_std` core mode.
- `cargo clippy -p ironclaw_hdc --all-targets -- -D warnings`
- `cargo bench -p ironclaw_hdc`
- `cargo test --features hdc,libsql,integration --test workspace_hdc --no-fail-fast`
- `cargo test --features hdc,libsql --test workspace_hdc_dedup --no-fail-fast`
- PostgreSQL equivalent integration tests where local CI already supports it.

---

## Phase 1: Core Crate

Create `crates/ironclaw_hdc/` as a focused library with no dependency on the
main IronClaw crate. The crate should be useful on its own and straightforward
to review.

### 1.1 Crate Structure

```text
crates/ironclaw_hdc/
  Cargo.toml
  CLAUDE.md
  src/
    lib.rs
    vector.rs
    bundle.rs
    codebook.rs
    encoder.rs
    dedup.rs
    scan.rs
    error.rs
  benches/
    operations.rs
    scan.rs
```

Suggested dependencies:

- Runtime: `serde`, `thiserror`.
- Optional/runtime gated: `rkyv` only if binary archived state is needed.
- Dev: `criterion`, `proptest`, `rand`, `serde_json`.

Keep hot vector operations allocation-free. `HdcVector::from_seed` may allocate
temporary seed material; types that need `Vec`, `HashMap`, or `Box` can require
`std` or `alloc`.

### 1.2 Public API

```rust
pub const DIMENSION_BITS: usize = 10_240;
pub const WORDS: usize = 160;
pub const BYTE_LEN: usize = 1_280;

pub struct HdcVector([u64; WORDS]);

impl HdcVector {
    pub const fn zero() -> Self;
    pub fn random(rng: &mut impl rand::Rng) -> Self;
    pub fn from_seed(domain: &str, seed: &[u8]) -> Self;
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, HdcError>;
    pub fn to_bytes(self) -> [u8; BYTE_LEN];
    pub fn bind(self, other: Self) -> Self;
    pub fn permute(self, shift: usize) -> Self;
    pub fn hamming_distance(self, other: Self) -> u32;
    pub fn similarity(self, other: Self) -> f64;
    pub fn words(&self) -> &[u64; WORDS];
    pub fn words_mut(&mut self) -> &mut [u64; WORDS];
}
```

Implementation notes:

- Use little-endian byte order for `to_bytes`/`from_bytes`.
- Domain-separate seeded vectors, e.g. `from_seed("tag", b"rust")` differs from
  `from_seed("path", b"rust")`.
- Use a stable local PRNG such as SplitMix64 seeded from a stable hash. Do not
  use a platform-dependent RNG for deterministic construction.
- `permute` must shift across the whole 10,240-bit vector, not per word.
- `Debug` should avoid dumping all 160 words.

### 1.3 Bundle Types

```rust
pub struct BundleAccumulator {
    votes: Box<[i32; DIMENSION_BITS]>,
    count: u32,
}

pub struct DecayingBundleAccumulator {
    votes: Box<[f32; DIMENSION_BITS]>,
    decay_factor: f32,
    count: u64,
}
```

Rules:

- Empty bundle finalizes to `HdcVector::zero()`.
- Ties resolve to `0` unless a deterministic tie-break vector is explicitly
  provided later.
- `DecayingBundleAccumulator::new(decay_factor)` accepts only finite values in
  `(0.0, 1.0)`.
- `effective_half_life_cycles()` returns `ln(0.5) / ln(decay_factor)`.

### 1.4 Codebook and Item Memory

```rust
pub struct Codebook { /* symbol -> HdcVector */ }
pub struct ItemMemory { /* Vec<(String, HdcVector)> */ }
```

Rules:

- `Codebook::get_or_create("role:path")` must be deterministic across runs.
- `ItemMemory::insert` should replace by name, not append duplicates. That keeps
  lookup output stable and avoids accidental cache growth.
- `lookup(query, top_k)` returns results sorted by descending similarity, then
  name for deterministic ties.

### 1.5 Encoders

Expose two stable encoders:

```rust
pub struct DocumentEncodingInput<'a> {
    pub content: &'a str,
    pub tags: &'a [&'a str],
    pub path: &'a str,
}

pub fn encode_document(input: DocumentEncodingInput<'_>, codebook: &mut Codebook) -> HdcVector;
pub fn encode_text(text: &str, codebook: &mut Codebook) -> HdcVector;
```

Default document encoding:

```text
content_hv = bundle(byte_trigram(content))
tag_hv     = bundle(bind(role:tag,  atom(tag)) for tag)
path_hv    = bundle(bind(role:path, atom(segment)).permute(segment_index) for path segment)
doc_hv     = weighted_bundle(content=5, tags=3, path=2)
```

Edge cases:

- Empty content uses tags/path.
- No tags uses content/path.
- Empty path uses content/tags.
- Fully empty input returns a deterministic `empty-document` vector.
- Unicode is handled as UTF-8 bytes for trigram stability.

### 1.6 Dedup and Scan Helpers

Keep pure scan logic in the crate, but keep IronClaw-specific storage outside
the crate.

```rust
pub struct DedupConfig {
    pub similar_threshold: f64,
    pub duplicate_threshold: f64,
}

pub enum DedupDecision<Id> {
    Unique,
    Similar { id: Id, path: String, similarity: f64 },
    Duplicate { id: Id, path: String, similarity: f64 },
}

pub struct FingerprintCandidate<Id> {
    pub id: Id,
    pub path: String,
    pub fingerprint: HdcVector,
}

pub fn check_dedup<Id: Clone>(
    query: HdcVector,
    candidates: &[FingerprintCandidate<Id>],
    config: &DedupConfig,
) -> DedupDecision<Id>;
```

Default thresholds should be conservative:

- `duplicate_threshold = 0.92`
- `similar_threshold = 0.78`

Treat these as starting values. Phase 0 fixtures decide whether they survive.

### 1.7 Core Test Suite

Each test below is a separate `#[test]` function. Group by module.

**`vector.rs` unit tests:**

- [x] `bind_involution` — `bind(bind(a, b), b) == a` for 50 seeded random pairs
- [x] `bind_commutative` — `bind(a, b) == bind(b, a)`
- [x] `bind_self_is_zero` — `bind(a, a) == zero()`
- [x] `bind_quasi_orthogonal` — `similarity(a, bind(a, b))` in `0.48..0.52` for random `a, b`
- [x] `similarity_identity` — `similarity(a, a) == 1.0` for 10 random vectors
- [x] `similarity_zero_zero` — `similarity(zero(), zero()) == 1.0`
- [x] `similarity_bounds` — for 1000 random pairs, `0.0 <= similarity(a, b) <= 1.0`
- [x] `similarity_random_concentration` — mean of 1000 random-pair similarities in `0.495..0.505`, stddev < 0.008
- [x] `similarity_statistical_threshold` — document that `0.526` is the p < 10^-7 threshold after Bonferroni correction for 100K comparisons (assert similarity of 1000 random pairs never exceeds `0.53`)
- [x] `permute_identity` — `a.permute(0) == a`
- [x] `permute_full_cycle` — `a.permute(DIMENSION_BITS) == a`
- [x] `permute_inverse` — `a.permute(k).permute(DIMENSION_BITS - k) == a` for k in `{1, 7, 63, 64, 160, 5000, 10239}`
- [x] `permute_cross_word_boundary` — verify `permute(63)` and `permute(65)` produce different results (catches per-word-only shift bugs)
- [x] `permute_produces_different_vector` — `a.permute(1) != a` for non-zero `a`
- [x] `from_seed_deterministic` — `from_seed("tag", b"rust")` called 10 times returns identical vectors
- [x] `from_seed_domain_separation` — `from_seed("tag", b"rust") != from_seed("path", b"rust")`
- [x] `from_seed_different_seeds_orthogonal` — `similarity(from_seed("d", b"a"), from_seed("d", b"b"))` in `0.48..0.52`
- [x] `from_seed_empty` — `from_seed("d", b"")` does not panic
- [x] `bytes_roundtrip` — `from_bytes(&v.to_bytes())` for 10 random vectors
- [x] `from_bytes_rejects_short` — `from_bytes(&[0u8; 100])` returns `Err(InvalidBytes)`
- [x] `from_bytes_rejects_long` — `from_bytes(&[0u8; 2000])` returns `Err(InvalidBytes)`
- [x] `from_bytes_accepts_exact` — `from_bytes(&[0u8; 1280])` succeeds
- [x] `hamming_distance_self` — `hamming_distance(a, a) == 0`
- [x] `hamming_distance_complement` — `hamming_distance(a, !a) == DIMENSION_BITS` (where `!a` flips all bits)

**`bundle.rs` unit tests:**

- [x] `bundle_single_vector` — accumulate one vector, finalize returns that vector
- [x] `bundle_majority_3v1` — add A three times, B once → `similarity(result, A) > similarity(result, B)`
- [x] `bundle_weighted` — add A with weight 5, B with weight 1 → result closer to A
- [x] `bundle_tie_resolves_to_zero` — add A once, then `bind(A, ones())` once (all bits flip) → each bit has vote 0, finalize returns zero
- [x] `bundle_empty_finalize` — `BundleAccumulator::new().finalize() == zero()`
- [x] `bundle_count` — after 7 adds, `count() == 7`
- [x] `bundle_capacity_gradient` — for K in `{2, 5, 10, 50, 100, 200}`, bundle K random vectors, verify component recovery degrades: similarity decreases as K increases
- [x] `decay_rejects_zero` — `DecayingBundleAccumulator::new(0.0)` returns Err
- [x] `decay_rejects_one` — `new(1.0)` returns Err
- [x] `decay_rejects_negative` — `new(-0.5)` returns Err
- [x] `decay_rejects_nan` — `new(f32::NAN)` returns Err
- [x] `decay_rejects_infinity` — `new(f32::INFINITY)` returns Err
- [x] `decay_recency_bias` — add A ten times, then B once; with decay 0.5, `similarity(result, B)` is higher than with a non-decaying accumulator
- [x] `decay_half_life` — with factor 0.95, `effective_half_life_cycles()` ≈ 13.5 (within 0.1)
- [x] `decay_full_forget` — add A, then add 200 random vectors with factor 0.9 → `similarity(result, A)` converges to ~0.5
- [x] `decay_reset` — after `reset()`, `finalize()` returns zero, `count() == 0`
- [x] `decay_serde_roundtrip` — JSON serialize → deserialize, finalize produces same result

**`codebook.rs` unit tests:**

- [x] `codebook_idempotent` — `get_or_create("x")` returns same vector on repeated calls
- [x] `codebook_orthogonal_symbols` — 50 distinct symbols, all pairwise similarities in `0.47..0.53`
- [x] `codebook_with_defaults` — `with_defaults(&["role:tag", "role:path"])` contains both, `len() == 2`
- [x] `codebook_get_absent` — `get("nonexistent")` returns None without inserting
- [x] `codebook_serde_roundtrip` — JSON round-trip preserves all entries and vectors
- [x] `item_memory_exact_match` — insert v as "x", lookup v returns ("x", 1.0)
- [x] `item_memory_ordering` — insert 100 items, lookup returns correct top-1
- [x] `item_memory_replacement` — insert "x" twice with different vectors, lookup sees only the second
- [x] `item_memory_top_k_clamp` — `lookup(query, 1000)` on 5 items returns 5 results
- [x] `item_memory_deterministic_ties` — ties sorted by name

**`encoder.rs` unit tests:**

- [x] `encode_document_deterministic` — same inputs, 10 calls → identical fingerprints
- [x] `encode_document_content_dominates` — same tags/path, different content → similarity < 0.85 in the current test; tighten only after fixture tuning
- [x] `encode_document_tags_contribute` — same content/path, different tags → similarity < 0.9 (tags shift result)
- [x] `encode_document_path_contributes` — same content/tags, different path → similarity < 0.95 (path shifts result)
- [x] `encode_document_empty_content` — doesn't panic, returns valid vector
- [x] `encode_document_empty_tags` — doesn't panic, same content still matches
- [x] `encode_document_empty_path` — doesn't panic
- [x] `encode_document_all_empty` — returns deterministic `empty-document` vector
- [x] `encode_document_unicode` — content with emoji, CJK, Arabic script produces valid fingerprint
- [x] `encode_document_long_content` — long-content regression test covers about 10 KB; precise throughput belongs to Criterion
- [x] `encode_document_single_char` — very short content doesn't panic (fewer than 3 bytes = no trigrams, fallback)
- [x] `encode_text_matches_content_component` — `encode_text(content)` ≈ content-only document encoding
- [x] `encode_text_similar_text` — "rust async tokio" vs "rust async runtime" → similarity > 0.6
- [x] `encode_text_dissimilar_text` — "rust async tokio" vs "french cooking recipes" → similarity in `0.45..0.55`
- [x] `encode_text_empty` — returns valid vector, not zero

**`dedup.rs` unit tests:**

- [x] `dedup_unique` — no candidates above threshold → `Unique`
- [x] `dedup_similar` — candidate at 0.80 with thresholds (0.78, 0.92) → `Similar`
- [x] `dedup_duplicate` — candidate at 0.95 with thresholds (0.78, 0.92) → `Duplicate`
- [x] `dedup_exact_match` — similarity 1.0 → `Duplicate`
- [x] `dedup_empty_candidates` — empty slice → `Unique`
- [x] `dedup_highest_wins` — candidates at 0.79 and 0.93, returns the 0.93 one (Duplicate), not the 0.79
- [x] `dedup_just_below_similar` — candidate at 0.77 with threshold 0.78 → `Unique`
- [x] `dedup_just_above_similar` — candidate at 0.78 → `Similar`
- [x] `dedup_config_validation` — `similar >= duplicate`, `similar <= 0.5`, `duplicate >= 1.0`, and non-finite thresholds are rejected

**Property tests (proptest):**

- [x] `prop_bind_involution` — `∀ a, b: bind(bind(a, b), b) == a`
- [x] `prop_similarity_in_unit` — `∀ a, b: 0.0 <= similarity(a, b) <= 1.0`
- [x] `prop_permute_inverse` — `∀ a, k in 0..DIMENSION_BITS: permute(k).permute(DIMENSION_BITS - k) == a`
- [x] `prop_bytes_roundtrip` — `∀ v: from_bytes(to_bytes(v)) == v`
- [ ] `prop_dedup_max_similarity` — planned; current proptests cover self-duplicate and random-unique behavior, not arbitrary max-selection
- [x] `prop_bundle_similarity_monotonic` — adding more copies of A to a bundle never decreases similarity to A

**Golden tests:**

- [x] `crates/ironclaw_hdc/tests/golden_vectors.rs` — stores deterministic seeded-vector expectations in Rust tests
- [ ] Full byte-for-byte JSON fixture such as `tests/golden/seed_vectors.json` is not present
- [x] Any change to seeding algorithm breaks this test — forces explicit version bump

**Benchmarks (criterion, `benches/operations.rs`):**

| Benchmark | What it measures | Target |
|---|---|---|
| `bench_bind` | `HdcVector::bind(a, b)` | track baseline; observed sub-microsecond |
| `bench_similarity` | `HdcVector::similarity(a, b)` | track baseline; popcount over 160 words |
| `bench_hamming` | `HdcVector::hamming_distance(a, b)` | track baseline; popcount over 160 words |
| `bench_permute_1` | `a.permute(1)` | track baseline |
| `bench_permute_5000` | `a.permute(5000)` | track baseline |
| `bench_from_seed` | `HdcVector::from_seed("d", b"test")` | track baseline |
| `bench_bundle_10` | Add 10 vectors to accumulator | track baseline |
| `bench_bundle_100` | Add 100 vectors | observed low milliseconds, not microseconds |
| `bench_finalize` | `BundleAccumulator::finalize()` | track baseline |
| `bench_decay_add` | `DecayingBundleAccumulator::add()` | track baseline |
| `bench_encode_text_1kb` | `encode_text` on 1 KB string | observed low tens of milliseconds |
| `bench_encode_document_1kb_5tags` | `encode_document` with 1 KB + 5 tags | observed low tens of milliseconds |

**Benchmarks (`benches/scan.rs`):**

| Benchmark | What it measures | Target |
|---|---|---|
| `bench_dedup_scan_1k` | `check_dedup` over 1K candidates | < 20 μs |
| `bench_dedup_scan_10k` | `check_dedup` over 10K candidates | observed around low milliseconds |
| `bench_dedup_scan_100k` | `check_dedup` over 100K candidates | observed single-digit milliseconds |
| `bench_item_memory_lookup_10k` | `ItemMemory::lookup` over 10K entries | track baseline |

Record first measured values in the PR description. Benchmarks are guardrails,
not correctness tests.

### Phase 1 Gate

- `cargo test -p ironclaw_hdc` passes.
- `cargo clippy -p ironclaw_hdc --all-targets -- -D warnings` passes.
- Benchmarks run and measured values are documented.
- Integration has since landed behind the `hdc` feature; Phase 1 itself remains independently testable.

---

## Phase 2: Persist Fingerprints in Shadow Mode

Compute and store HDC fingerprints for workspace documents without changing
write, search, or heartbeat behavior.

### 2.1 Schema

Latest PostgreSQL migration is currently `V32`, so use:

```sql
-- migrations/V33__memory_hdc_fingerprint.sql
ALTER TABLE memory_documents
  ADD COLUMN hdc_fingerprint BYTEA;
```

For libSQL, add the equivalent migration in `src/db/libsql_migrations.rs`:

```sql
ALTER TABLE memory_documents ADD COLUMN hdc_fingerprint BLOB;
```

Prefer Rust-side length validation for portability. A DB `CHECK` constraint is
nice but not required if it complicates libSQL migration behavior.

### 2.2 Document and Store Changes

Add:

```rust
pub hdc_fingerprint: Option<Vec<u8>>
```

to `MemoryDocument`.

Extend `WorkspaceStore` with scoped fingerprint operations:

```rust
async fn update_document_hdc_fingerprint(
    &self,
    id: Uuid,
    fingerprint: &[u8],
) -> Result<(), WorkspaceError>;

async fn list_document_hdc_fingerprints(
    &self,
    user_id: &str,
    agent_id: Option<Uuid>,
) -> Result<Vec<DocumentHdcFingerprint>, WorkspaceError>;
```

`DocumentHdcFingerprint` should include `id`, `path`, and `fingerprint` so
dedup does not need a second lookup.

Important:

- Reject non-1,280-byte fingerprints at the store boundary.
- Return only non-null fingerprints.
- Scope by `user_id` and `agent_id`.
- Implement both PostgreSQL and libSQL before using the trait from workspace
  code.

### 2.3 Workspace Integration

Add an optional dependency and feature:

```toml
ironclaw_hdc = { path = "crates/ironclaw_hdc", optional = true }

[features]
hdc = ["dep:ironclaw_hdc"]
```

Integrate after successful writes and reindexing:

1. Encode `(content, tags: &[], path)` in the live workspace write path.
2. Store bytes through `update_document_hdc_fingerprint`.
3. Keep the backfill path's metadata/tag behavior explicit.
4. Normalize live-write and backfill tag inputs before promoting tag-sensitive
   behavior.

Rules:

- Skip engine runtime state paths that already skip semantic indexing.
- Do not change return values or tool output in Phase 2.
- If fingerprinting fails, fail open and log at `debug`; ordinary memory writes
  must remain available.
- Unchanged-content writes may compute a missing fingerprint, but should not
  rewrite an identical existing fingerprint.
- Existing documents are allowed to have `NULL` until backfill runs.

### 2.4 Backfill

Add a CLI path, preferably:

```text
ironclaw memory hdc backfill [--user <id>] [--limit <n>] [--dry-run]
```

Behavior:

- List documents in scope.
- Filter documents with missing fingerprints.
- Process in batches.
- Print counts: scanned, skipped, fingerprinted, failed.
- Dry-run computes but does not update.
- Re-running is idempotent.

### 2.5 Phase 2 Tests

**Storage tests:**

- [x] libSQL `migration_fresh_db` — migration applies on fresh database, column exists and is nullable
- [x] libSQL `migration_existing_data` — migration on database with 100 existing documents, all get NULL fingerprint, no data loss
- [x] libSQL `roundtrip_none` — insert document with `hdc_fingerprint: None`, read back, field is None
- [x] libSQL `roundtrip_valid_fingerprint` — insert document with 1,280-byte fingerprint, read back through fingerprint listing, bytes match exactly
- [x] libSQL `store_rejects_wrong_length` — `update_document_hdc_fingerprint` with 500-byte input returns error
- [x] libSQL `store_rejects_empty` — `update_document_hdc_fingerprint` with 0-byte input returns error
- [x] libSQL `list_fingerprints_scoped` — user A has 3 docs with fingerprints, user B has 2 → listing for user A returns 3
- [x] libSQL `list_fingerprints_excludes_null` — user has 5 docs, 2 without fingerprints → listing returns 3
- [x] libSQL `list_fingerprints_includes_path` — returned `DocumentHdcFingerprint` structs have correct paths
- [x] libSQL `list_fingerprints_agent_scoped` — exact nullable `agent_id` matching isolates scoped docs from unscoped docs
- [x] libSQL `overwrite_fingerprint` — calling `update_document_hdc_fingerprint` twice on same document keeps only the latest
- [ ] PostgreSQL contract coverage for the same cases.

**Workspace integration tests (`--features hdc,integration`):**

- [x] `write_stores_fingerprint` — `workspace.write("test.md", "hello world")` with shadow mode → fingerprint listing has non-NULL 1,280-byte fingerprint
- [x] `write_deterministic_fingerprint` — write same content/path twice → fingerprints are identical bytes
- [x] `write_different_content_different_fingerprint` — write "hello" then "goodbye" to different paths → fingerprints differ
- [ ] `write_metadata_tags_affect_fingerprint` — pending; workspace currently passes `tags: &[]`
- [x] `write_same_content_different_paths_same_fingerprint` — same content, different paths, no metadata tags → fingerprints are identical under current weighted-majority encoder
- [x] `write_unchanged_content_preserves_fingerprint` — rewrite same content to same path → fingerprint unchanged (not recomputed needlessly)
- [x] `write_engine_runtime_path_no_fingerprint` — engine state paths (e.g., `is_engine_runtime_path()` returns true) → fingerprint stays NULL
- [x] `write_identity_file_gets_fingerprint` — `IDENTITY.md`, `SOUL.md` etc. get fingerprints (they are user-authored, not engine state)
- [x] `append_updates_fingerprint` — appending to existing document recomputes fingerprint from final content
- [x] `patch_updates_fingerprint` — patching via `old_string`/`new_string` recomputes fingerprint from final content
- [x] `delete_removes_fingerprint` — deleting a document removes it from fingerprint listings
- [x] `multi_user_isolation` — user A cannot see user B's fingerprints via `list_document_hdc_fingerprints`
- [ ] `fingerprint_failure_fails_open` — pending mocked failure coverage; current test verifies shadow-disabled writes remain unchanged

**Caller-level tests:**

- [x] `memory_write_tool_succeeds_with_hdc` — true `MemoryWriteTool.execute()` caller coverage; verifies successful write plus persisted shadow fingerprint
- [x] `memory_write_tool_output_unchanged` — true `MemoryWriteTool.execute()` JSON-shape coverage; verifies no `hdc_fingerprint` or default-off `dedup` field leaks into output

**Feature matrix tests:**

- [ ] `cargo test` (default features, no `hdc`) — not run in this audit
- [x] `cargo check --features hdc` — main crate compiles with HDC
- [x] `cargo test -p ironclaw_hdc` — HDC crate tests pass
- [ ] `cargo test --features hdc,integration` — full integration matrix across both backends pending

### Phase 2 Gate

- Both DB backends pass the new contract tests.
- Write p95 overhead is measured under 2 ms on the fixture corpus.
- Backfill dry-run and real-run are idempotent on a test database.
- No user-facing behavior changes.

---

## Phase 3: Write-Time Deduplication

This is the first user-visible feature. It must be conservative and test through
the actual memory tool, not only through helper functions.

### 3.1 Placement in Write Flow

Do not run dedup after `get_or_create_document_by_path` for new paths, because a
blocked duplicate would leave a ghost empty document. The preflight should run
before document creation when the target path does not already exist.

Recommended flow:

1. Normalize target path and validate protected paths.
2. Compute final content for the non-append write path. Append/patch active
   dedup is future work unless explicitly added.
3. Determine whether the target document already exists.
4. Resolve metadata needed for tags only after live-write/backfill tag inputs
   are normalized.
5. Compute HDC fingerprint. Current tool preflight uses `tags: &[]`.
6. Load scoped existing fingerprints.
7. Exclude the current document ID if this is an overwrite/append to an existing
   path.
8. Make dedup decision.
9. Only create/update document if the decision allows it.

If current APIs make step 3 awkward, add a scoped optional lookup helper instead
of abusing `get_or_create`.

### 3.2 Behavior

| Decision | Default behavior | Tool output |
|---|---|---|
| Unique | Write proceeds | Normal success |
| Similar | Write proceeds | Include warning with path, id, similarity |
| Duplicate | Write is blocked unless `force=true` | Include existing path, id, similarity, retry hint |

Suggested JSON fields:

```json
{
  "status": "blocked",
  "path": "projects/new.md",
  "dedup": {
    "decision": "duplicate",
    "existing_path": "projects/old.md",
    "existing_id": "...",
    "similarity": 0.94,
    "retry": "Pass force=true to write anyway."
  }
}
```

Rules:

- `force=true` bypasses duplicate blocking. It records a fingerprint only when
  workspace HDC shadow fingerprinting is also enabled.
- Same-path overwrite/append is never blocked by its own old fingerprint.
- Current active dedup is scoped by the primary workspace `user_id` and
  nullable `agent_id`; layer-aware active dedup is pending because tool preflight
  runs before layer routing.
- Dedup must not bypass prompt-injection checks for protected workspace files.
- Dedup currently skips only `is_identity_path` files: `IDENTITY.md`, `SOUL.md`,
  `AGENTS.md`, `USER.md`, `TOOLS.md`, and `BOOTSTRAP.md`. It does not skip every
  system-prompt-like file such as `MEMORY.md` or `HEARTBEAT.md`.

### 3.3 Cache

A cache is useful but should not be in the first behavior-changing patch unless
benchmarks require it.

If added:

- Cache key: `(user_id, agent_id, layer/scope)`.
- Value: `HashMap<Uuid, (path, HdcVector)>`.
- Populate lazily from `list_document_hdc_fingerprints`.
- Update on successful write; remove on delete.
- Bound by count and approximate bytes.
- Invalidate on backfill completion.

### 3.4 Phase 3 Tests

Current coherent coverage is preflight/default-off coverage. Active block,
warn, and force-bypass behavior is still pending.

**Workspace preflight and shadow tests:**

- [x] `check_dedup_detects_exact_duplicate` — preflight returns `Duplicate` for exact same content at a different path.
- [x] `check_dedup_flags_near_duplicate` — preflight flags a reformatted near duplicate as non-unique.
- [x] `check_dedup_allows_related_content_with_substantial_addition` — preflight stays `Unique` when related content has enough new material.
- [x] `direct_workspace_write_does_not_enforce_dedup` — direct `Workspace::write` still allows duplicate content.
- [x] `dedup_allows_unique` — distinct writes remain readable.
- [x] `dedup_same_path_overwrite` — self-overwrite succeeds; active self-exclusion blocking is not asserted.
- [x] `dedup_same_path_append` — append succeeds and the stored fingerprint matches final content.
- [x] `dedup_patch_uses_final_content` — patch succeeds and the stored fingerprint matches final content.
- [x] `dedup_append_uses_final_content` — append recomputes the fingerprint from full final content, not only the appended fragment.
- [x] `dedup_no_ghost_document` — preflight does not create the candidate path.
- [x] `dedup_scoped_by_user` — preflight does not cross user scopes.
- [x] `dedup_scoped_by_agent` — preflight does not cross agent scopes.
- [x] `layer_writes_store_fingerprints_in_target_scope` — layer writes persist fingerprints under the target layer scope.
- [ ] Layer-aware active dedup — pending; current tool preflight runs before layer routing.
- [x] `dedup_skips_identity_files` — identity/system paths remain writable; caller-level active behavior pending.
- [x] `dedup_does_not_bypass_injection_check` — protected path injection rejection still fires.
- [x] `shadow_writes_store_fingerprints_for_duplicate_content` — shadow mode stores fingerprints for duplicate direct writes.
- [x] `dedup_handles_no_fingerprints` — fresh workspace writes succeed.
- [x] `dedup_handles_null_fingerprints` — preflight skips legacy documents with null fingerprints.
- [ ] Active workspace block/warn/force behavior — not a direct `Workspace::write/append/patch` contract.

**Memory tool caller tests (through `MemoryWriteTool.execute()`):**

- [x] `tool_default_off_allows_duplicate_without_dedup_field` — default-off duplicate writes succeed without `dedup` output.
- [x] `tool_duplicate_force_retry` — `force=true` preserves baseline success while dedup mode is off; force-as-dedup-bypass is pending.
- [x] `tool_default_off_writes_similar_without_warning` — default-off similar write succeeds without warning.
- [x] `tool_unique_no_dedup_field` — unique write output has no `dedup` field.
- [x] `tool_default_off_preserves_baseline_write_output` — compatibility shape is strictly checked with no `dedup` output.
- [x] `tool_daily_log_append_currently_skips_dedup` — append/daily-log path currently skips dedup output.
- [x] `tool_bootstrap_exempt` — bootstrap clear path remains exempt.
- [ ] Active tool duplicate block output — pending configured block-mode assertion.
- [ ] Active tool similar warning output — pending configured warn-mode assertion.
- [ ] Active tool force retry/bypass output — pending configured block-mode force assertion.
- [ ] Active append/daily-log dedup — pending; append writes currently skip preflight.

**Benchmark smoke tests:**

- [x] `dedup_preflight_smoke_over_seeded_fingerprints` — seeded shadow fingerprints are usable by `Workspace::check_dedup`; real scan thresholds live in Criterion.
- [x] `bench_write_with_dedup_cold` — smoke test only; no wall-clock CI gate.
- [x] `bench_write_with_dedup_warm` — smoke test over 100 docs only; no wall-clock CI gate.

### Phase 3 Gate

- [ ] False block-rate gate is automated against unique fixtures.
- [ ] Configured `MemoryWriteTool` duplicate block, similar warn, and force bypass are asserted.
- [x] Rollback remains one feature flag or config flag.

---

## Phase 4: Heartbeat Novelty Detection

Use a decaying HDC summary to detect repeated heartbeat findings. Start
observe-only. Suppression comes later and must be configurable.

### 4.1 State Model

Current heartbeat HDC state is in-memory only. If it becomes persisted, do not
store it in a global path such as `~/.ironclaw` for multi-user deployments.
Persist it in a user-scoped place:

- preferred: DB-backed heartbeat state keyed by user,
- acceptable: workspace-internal state document that is excluded from search,
- fallback: in-memory only for the first observe-only patch.

State should include:

- serialized accumulator,
- codebook version,
- decay factor,
- last update timestamp,
- encoder version.

### 4.2 Runner Integration

Add optional HDC state to `HeartbeatRunner` behind `hdc`.

After the LLM response is produced:

1. Encode response text with `encode_text`.
2. Compare against the finalized decaying accumulator if non-empty.
3. Classify as `novel`, `repeated`, or `uncertain`.
4. Add the response vector to the accumulator.
5. Persist state only after a user-scoped persistence path is implemented;
   current behavior keeps state in memory.
6. Emit debug/metric fields.

Suggested defaults:

| Setting | Default | Meaning |
|---|---:|---|
| `heartbeat_hdc_enabled` | false | Observe-only feature gate |
| `heartbeat_hdc_decay_factor` | 0.93 | About 9.5 cycles half-life |
| `heartbeat_hdc_repeated_threshold` | 0.80 | Likely repeated |
| `heartbeat_hdc_novel_threshold` | 0.60 | Likely novel |
| `heartbeat_hdc_suppress_repeated` | false | No suppression until proven |

### 4.3 Tests

**Unit tests (accumulator behavior with heartbeat fixtures):**

- [x] `novelty_first_observation` — empty accumulator, encode observation → similarity to finalize() is ~0.5 → classified as novel
- [x] `novelty_identical_converges` — add same observation 10 times → similarity crosses repeated threshold by cycle 3-4
- [x] `novelty_different_stays_novel` — add genuinely different observation each cycle → similarity stays below repeated threshold
- [x] `novelty_decay_allows_recurrence` — add observation A 5 times, then 20 different observations, then A again → A is novel again after decay window
- [x] `novelty_uncertain_zone` — observation with similarity between novel and repeated thresholds → classified as uncertain
- [x] `novelty_decay_factor_sensitivity` — with factor 0.5 (fast decay), observations become novel again after ~2 cycles; with 0.99 (slow decay), they stay repeated for 50+ cycles
- [x] `novelty_similar_but_different_metric` — "disk at 85%" then "disk at 92%" → tests whether trigram-level encoding distinguishes these (document expected behavior)

**Heartbeat runner caller tests (`--features hdc,integration`):**

- [x] `heartbeat_hdc_disabled_unchanged` — with `heartbeat_hdc_enabled: false`, `check_heartbeat()` returns same `HeartbeatResult` variants as without HDC feature
- [ ] `heartbeat_observe_only_no_suppress` — test currently calls `check_heartbeat()` and bypasses `apply_hdc_novelty`; needs runner-loop coverage
- [ ] `heartbeat_observe_only_logs_novelty` — pending tracing capture; implementation logs classification/count but not score
- [ ] `heartbeat_suppression_enabled` — pending runner-loop test; `check_heartbeat()` alone does not suppress
- [ ] `heartbeat_suppression_never_hides_novel` — pending runner-loop false-suppression test
- [ ] `heartbeat_multi_user_isolation` — current test uses separate runner instances but does not apply HDC state; multi-tenant runner bypasses HDC
- [x] `heartbeat_hdc_failure_fails_open` — invalid decay factor disables HDC and raw result passes through
- [ ] `heartbeat_state_persists_across_restart` — pending persisted state
- [ ] `heartbeat_corrupt_state_resets` — pending persisted state and corrupt-state recovery
- [ ] `heartbeat_notification_includes_novelty` — pending `OutgoingResponse` novelty metadata

### Phase 4 Gate

- Observe-only logs run for at least one representative workload.
- No novel fixture observation is suppressed.
- Suppression remains default off.

---

## Phase 5: Search Fusion in Shadow Mode

HDC search is conditional. It should not affect ranking until it demonstrates a
retrieval lift on labeled fixtures and sampled real workspace queries.

Rollout rule: search is off by default. The first enabled mode must be shadow
mode, where HDC rank and score are attached without changing order. Live
reranking requires a separate explicit config value and a fixture-backed quality
gate. If the current boolean env shape proves ambiguous, replace it with
`IRONCLAW_HDC_SEARCH_MODE=off|shadow|live`.

### 5.1 Search Types

Extend `SearchConfig`:

```rust
pub use_hdc: bool,
pub hdc_weight: f32,
pub hdc_shadow_mode: bool,
```

Extend `SearchResult`:

```rust
pub hdc_rank: Option<u32>,
pub hdc_score: Option<f64>,
```

Defaults:

- `use_hdc = false`
- `hdc_weight = 0.2`
- `hdc_shadow_mode = true`

Existing constructors must keep compiling.

### 5.2 Ranking Flow

In `Workspace::search_with_config`:

1. Keep current FTS/vector behavior unchanged.
2. If `use_hdc`, encode query locally.
3. Load scoped fingerprints.
4. Compute similarity scan.
5. Rank by HDC similarity.
6. In shadow mode, attach HDC rank/score but do not affect final ordering.
7. In live mode, add HDC to RRF/weighted fusion.

### 5.3 Evaluation

Add a local comparison command:

```text
ironclaw memory hdc search-compare "<query>" [--limit N]
```

Output:

- path,
- FTS rank,
- vector rank,
- HDC rank,
- HDC similarity,
- final rank,
- rank delta,
- whether the document is labeled relevant when using fixtures.

Metrics:

- NDCG@10,
- MRR,
- relevant@10,
- overlap@5,
- Kendall tau between HDC and current fused rank,
- p50/p95 latency delta.

### Phase 5 Tests

- [x] `search_hdc_disabled_identical` — `SearchConfig { use_hdc: false, .. }` keeps HDC fields unset in synthetic fusion tests
- [x] `search_shadow_no_reorder` — synthetic fusion test verifies shadow config does not alter baseline fusion output
- [x] `search_shadow_populates_fields` — synthetic test manually attaches `hdc_rank`/`hdc_score` and verifies field contract
- [ ] `search_live_mode_changes_ranking` — config-only coverage exists; real workspace search assertion pending
- [ ] `search_hdc_boosts_structural_match` — config-only coverage exists; fixture-backed workspace search pending
- [ ] `search_hdc_does_not_regress_semantic` — shadow-mode config assertion only; needs labeled fixture metric
- [ ] `search_multi_scope_identity_exclusion` — pending HDC-specific assertion; HDC currently loads primary-scope fingerprints only
- [x] `search_fts_plus_hdc_no_embeddings` — synthetic FTS-only fusion returns FTS results
- [ ] `search_hdc_only` — pending; current orchestrator cannot create results from HDC alone when FTS/vector are disabled
- [x] `search_weight_normalization` — synthetic weighted-score math covered
- [ ] `search_scan_10k_benchmark` — Criterion exists, but no CI assertion and observed values exceed original target
- [ ] `search_scan_100k_benchmark` — Criterion exists, but no CI assertion and observed values exceed original target
- [ ] `search_shadow_log_format` — implementation logs count/top-5 only, not query, overlap@5, or Kendall tau

### Phase 5 Gate

- No ordinary-query regression on fixture set.
- Measurable lift on compositional/structural queries.
- Shadow-mode production logs show useful disagreement, not random noise.

---

## Phase 6: Optional Skill and Tool Matching

This is explicitly conditional. Do not start it until Phase 5 proves HDC is a
useful retrieval signal.

Candidate work:

- Build `SkillHdcIndex` from skill name, description, tags, and prompt summary.
- Build `ToolHdcIndex` from tool name, description, and parameter names.
- Use HDC as a small tie-breaker, not as the primary selector.

Rules:

- Exact keyword/regex/tool policy matches must still dominate.
- HDC boost must be capped.
- Hot reload must rebuild indexes.
- Feature remains off by default.

Tests:

- Exact matches still win over HDC-only matches.
- Irrelevant text produces no meaningful boost.
- Index rebuilds after skill/tool registration changes.

---

## Feature Flags and Configuration

| Flag/config | Default | Purpose |
|---|---:|---|
| Cargo feature `hdc` | off | Compile integration code |
| `IRONCLAW_HDC_FINGERPRINT_SHADOW` | false | Store fingerprints without behavior change |
| `IRONCLAW_HDC_DEDUP_MODE` | `off` | `off`, `warn`, `block` |
| `IRONCLAW_HDC_SIMILAR_THRESHOLD` | `0.78` | Similar warning threshold |
| `IRONCLAW_HDC_DUPLICATE_THRESHOLD` | `0.92` | Blocking threshold |
| `IRONCLAW_HDC_SEARCH_SHADOW` | true when search is enabled | Intended env-level shadow setting; current workspace search HDC is enabled per call through `SearchConfig` |
| Future `IRONCLAW_HDC_SEARCH_MODE` | `off` | Less ambiguous replacement for search booleans: `off`, `shadow`, `live` |
| `HEARTBEAT_HDC_ENABLED` | false | Enable heartbeat novelty classification |
| `HEARTBEAT_HDC_DECAY_FACTOR` | `0.93` | Decaying accumulator factor, finite value in `(0.0, 1.0)` |
| `HEARTBEAT_HDC_REPEATED_THRESHOLD` | `0.80` | Similarity at/above this is repeated |
| `HEARTBEAT_HDC_NOVEL_THRESHOLD` | `0.60` | Similarity below this is novel; must be below repeated threshold |
| `HEARTBEAT_HDC_SUPPRESS_REPEATED` | false | Suppress repeated findings; do not enable until runner-loop tests are in place |

Prefer DB-backed settings if the codebase already has a suitable settings path;
env vars are acceptable for early development and CI fixtures.

## Cross-Cutting Test Matrix

| Tier | Required coverage |
|---|---|
| Crate unit | Algebra, encoding, errors, serialization |
| Crate property | Involution, bounds, roundtrips, ranking invariants |
| DB contract | PostgreSQL and libSQL migration, row mapping, scoped fingerprint queries |
| Workspace integration | Write, append, patch, delete/cache invalidation, search |
| Tool caller | `memory_write` duplicate block, force retry, similar warning |
| Heartbeat caller | observe-only, suppression enabled, corrupt state, multi-user |
| Feature matrix | default features, `--features hdc`, PostgreSQL, libSQL |
| Benchmark | operation latency, scan latency, write/search overhead |

## Rollback Plan

- Disable behavior via config: `IRONCLAW_HDC_DEDUP_MODE=off`,
  HDC search disabled or shadow-only per `SearchConfig`, heartbeat suppression
  off.
- Leave nullable fingerprints in place; they are inert metadata.
- Revert search fusion without removing stored fingerprints.
- If a migration must be rolled back manually, dropping `hdc_fingerprint` is
  safe because fingerprints are derived data.

## Documentation Updates

- `crates/ironclaw_hdc/CLAUDE.md`: crate invariants and public API.
- Rustdoc on public HDC APIs.
- `src/workspace/README.md`: optional HDC fingerprinting, dedup, and search.
- `src/db/CLAUDE.md`: `hdc_fingerprint` schema and backend parity.
- `FEATURE_PARITY.md`: only if behavior status changes from planned to shipped.

## Open Questions

1. Should the first encoder be byte trigram only, or byte trigram plus word
   tokens? Decide with Phase 0 fixtures.
2. Should fingerprints be per document only, or also per chunk? Start with
   document-level; per-chunk belongs to search if Phase 5 needs it.
3. Should dedup skip only current `is_identity_path` files permanently, or also
   broaden to files such as `MEMORY.md` and `HEARTBEAT.md`? Decide with fixtures
   and protected-path review.
4. Should heartbeat HDC state live in `heartbeat_state` or a hidden workspace
   document? Prefer existing DB state if it keeps user scoping simple.
5. What is the smallest useful HDC search fixture set for CI? Start small and
   expand only when it catches real regressions.

## Total Checklist Count

Phase 0: ~25 fixture and metric items
Phase 1: ~85 tests + ~16 benchmarks + ~8 scaffold items
Phase 2: ~30 tests + ~3 benchmark items
Phase 3: ~25 tests + ~3 benchmark items
Phase 4: ~17 tests
Phase 5: ~13 tests
Phase 6: ~3 tests

**Approximate total: ~225 individually trackable items.**

## Practical Build Order for Follow-Up Work

This section lists independently shippable implementation slices ordered by
value, risk, and dependency. Each slice has a clear scope, file targets, and
test requirements. All slices are behind the `hdc` feature flag and
default-off.

### Phase A: Validate existing work (ship first)

1. **Fixture corpus test harness** — Create `tests/workspace_hdc_fixtures.rs`.
   Read the existing 17 fixture scenarios from `tests/fixtures/hdc_memory/`,
   encode documents, assert similarity ranges match manifest expectations.
   Pure crate test, no DB dependency. ~200 lines.
   - Why first: validates encoder quality with labeled data before building more.
   - Blocked by: nothing.
   - Ship independently: yes.

2. **Active dedup caller tests** — Add 4 tests to
   `tests/workspace_hdc_dedup.rs` for warn mode, block mode, force bypass,
   and unique-in-warn-mode. ~150 lines.
   - Why second: proves the user-visible dedup behavior works before promotion.
   - Blocked by: nothing.
   - Ship independently: yes.

3. **PostgreSQL HDC storage contract** — Add 5 tests to
   `tests/workspace_hdc.rs` using PostgreSQL backend. ~100 lines.
   - Why third: satisfies dual-backend parity rule.
   - Blocked by: testcontainers setup.
   - Ship independently: yes.

4. **Search config wiring** — Wire `HdcConfig::search_shadow` into
   `SearchConfig` in `src/app.rs`. Surface `hdc_rank`/`hdc_score` in
   `MemorySearchTool` output JSON. ~30 lines code + ~50 lines tests.
   - Why fourth: completes the existing search HDC path end-to-end.
   - Blocked by: nothing.
   - Ship independently: yes.

### Phase B: HDC crate extensions (new primitives)

5. **K-medoids clustering module** — Create
   `crates/ironclaw_hdc/src/cluster.rs`. PAM algorithm with Hamming distance.
   ~250 lines + tests.
   - Blocked by: nothing (pure crate addition).
   - Ship independently: yes.

6. **Structured role-filler encoding** — Add `encode_structured` and
   `encode_causal_link` to `crates/ironclaw_hdc/src/encoder.rs`. ~100 lines
   + tests.
   - Blocked by: nothing.
   - Ship independently: yes.

### Phase C: Integration extensions

7. **Heartbeat HDC state persistence** — Serialize `HeartbeatHdcState` to
   workspace document after each novelty check. Load on construction. ~60
   lines + tests.
   - Blocked by: nothing.
   - Ship independently: yes.

8. **Native memory HDC write-time fingerprinting** — Add HDC fingerprint
   computation to `ChunkingMemoryDocumentIndexer::reindex_document_with_audit_context()`.
   ~80 lines + tests.
   - Blocked by: nothing.
   - Ship independently: yes.

9. **Native memory context dedup** — Add inter-snippet HDC dedup in
   `NativeMemoryService::retrieve_context()`. ~120 lines + tests.
   - Blocked by: slice 8 (native fingerprints must exist first).
   - Ship independently: yes, after slice 8.

10. **Subagent goal dedup** — Add HDC fingerprint to
    `SubagentGoalStore::put_goal()`, compare against in-flight goals. ~100
    lines + tests.
    - Blocked by: nothing.
    - Ship independently: yes.

### Phase D: Advanced features (offline/batch)

11. **Episode fingerprinting** — Post-turn hook in agent loop or
    `SubagentCompletionObserver` that computes prompt+outcome HDC fingerprint.
    ~80 lines + tests.
    - Blocked by: nothing.
    - Ship independently: yes.

12. **Memory audit clustering CLI** — `ironclaw memory hdc audit --user X --k N`.
    Uses k-medoids from slice 5. ~150 lines + tests.
    - Blocked by: slice 5 (k-medoids module).
    - Ship independently: yes, after slice 5.

13. **Dream consolidation batch job** — Offline job that clusters completed
    threads, distills lessons, screens for redundancy, proposes playbook
    candidates. ~300 lines + tests.
    - Blocked by: slices 5 (clustering), 6 (structured encoding), 11 (episode fingerprints).
    - Ship independently: yes, after prerequisites.

14. **Anti-knowledge admission gate** — Workspace path convention for
    `anti-knowledge/` documents. Admission layer that warns when candidate
    matches anti-knowledge. ~150 lines + tests.
    - Blocked by: nothing (uses existing `check_dedup`).
    - Ship independently: yes.

### Dependency graph

```
1 (fixtures) ──────────────────────────────────────────────────
2 (active dedup tests) ────────────────────────────────────────
3 (PG contract) ───────────────────────────────────────────────
4 (search wiring) ─────────────────────────────────────────────
5 (k-medoids) ──────────> 12 (audit CLI) ──> 13 (dream)
6 (structured encoding) ───────────────────> 13 (dream)
7 (heartbeat persistence) ─────────────────────────────────────
8 (native fingerprints) ──> 9 (native context dedup)
10 (subagent goal dedup) ──────────────────────────────────────
11 (episode fingerprints) ─────────────────> 13 (dream)
14 (anti-knowledge) ───────────────────────────────────────────
```

### Total estimated scope

| Phase | Slices | New lines (code+tests) | Independently shippable |
|-------|--------|----------------------|------------------------|
| A | 1-4 | ~530 | Each slice is independent |
| B | 5-6 | ~350 | Each slice is independent |
| C | 7-10 | ~360 | 9 depends on 8; rest independent |
| D | 11-14 | ~680 | 12-13 have prerequisites |
| **Total** | **14** | **~1,920** | |

---

## Follow-Up Implementation Slices

These slices match the open audit items and extend the concrete work listed in
the build order above. Each entry specifies exact file paths, feature gates,
test function names, and line-count estimates. All slices are independently
shippable unless a dependency is noted.

---

### Slice A: Fixture Corpus Test Harness (Priority 1)

**Audit item:** fixture files exist in `tests/fixtures/hdc_memory/` but no
automated test reads them.

**File to create:** `tests/workspace_hdc_fixtures.rs`

**Feature gate:** `#[cfg(feature = "hdc")]` — pure crate test, no DB
dependency.

**What it does:**

- Reads `tests/fixtures/hdc_memory/manifest.json` once at test startup.
- For each dedup scenario: loads the JSONL pair, calls `encode_document` on
  both documents with `DocumentEncodingInput { content, tags: &[], path }`,
  computes `HdcVector::similarity`, and asserts the result falls within
  the manifest's `expected_similarity_range [min, max]`.
- For each heartbeat scenario: loads the observation sequence, runs each
  through `DecayingBundleAccumulator::add` (default decay factor), and at
  each cycle index asserts the classification (novel/repeated/uncertain)
  matches `expected_novelty_at_cycle[i]`.
- For each search scenario: encodes the query string and all labeled
  documents, runs `top_k_scan` with `k = len(documents)`, and asserts every
  `relevant_document_id` ranks strictly above every `irrelevant_document_id`
  in the returned order.

**Tests required (one `#[test]` per fixture):**

```
fixture_dedup_exact_duplicate_different_path
fixture_dedup_reformatted_duplicate
fixture_dedup_paraphrase_duplicate
fixture_dedup_added_paragraph
fixture_dedup_same_topic_different_facts
fixture_dedup_same_tags_different_content
fixture_dedup_same_path_pattern_different_content
fixture_dedup_content_subset
fixture_dedup_translated_content
fixture_heartbeat_repeated_observation
fixture_heartbeat_recurring_after_gap
fixture_heartbeat_similar_but_different_metric
fixture_heartbeat_completely_novel
fixture_search_structural_query
fixture_search_tag_heavy_query
fixture_search_path_oriented_query
fixture_search_semantic_only_query
```

**Estimated scope:** ~200 lines

**Independently shippable:** yes — no file conflicts with any other slice.

---

### Slice B: Active Dedup Caller Tests (Priority 1)

**Audit items:** "Configured `MemoryWriteTool` block/warn/force output
assertions are pending; current caller tests cover default-off output shape."

**File to modify:** `tests/workspace_hdc_dedup.rs`

**Module to extend:** `mod memory_tool_dedup` (already exists in that file)

**Feature gate:** `#[cfg(all(feature = "hdc", feature = "libsql"))]`

**Tests to add:**

- `tool_warn_mode_writes_with_dedup_warning` — set env var
  `IRONCLAW_HDC_DEDUP_MODE=warn`, write a near-duplicate document through
  `MemoryWriteTool::execute()`, assert the JSON output contains
  `dedup.decision="similar"` and the document is written (status is not
  `"blocked"`).
- `tool_block_mode_rejects_duplicate` — set `IRONCLAW_HDC_DEDUP_MODE=block`,
  write a document whose similarity to an existing fingerprint exceeds
  `duplicate_threshold`, assert output contains `status="blocked"` and a
  `dedup` object, assert the candidate path does not exist in the workspace
  (no ghost document).
- `tool_block_mode_force_bypasses` — set `IRONCLAW_HDC_DEDUP_MODE=block`,
  pass `force=true` in tool params, assert write succeeds and fingerprint is
  stored.
- `tool_warn_mode_unique_no_dedup_field` — set `IRONCLAW_HDC_DEDUP_MODE=warn`,
  write a document with similarity below `similar_threshold`, assert output
  JSON has no `dedup` field.

**Estimated scope:** ~150 lines

**Independently shippable:** yes — depends only on existing libSQL test
infrastructure and the current `MemoryWriteTool` implementation.

---

### Slice C: PostgreSQL HDC Storage Contract (Priority 1)

**Audit item:** `[ ] PostgreSQL contract coverage for the same cases` (under
Phase 2 storage tests).

**File to modify:** `tests/workspace_hdc.rs`

**Feature gate:** `#[cfg(all(feature = "hdc", feature = "integration"))]`

**What it does:** Add `_pg` variants of the libSQL contract tests. Test body
is identical except for DB setup — spin up a PostgreSQL connection via the
existing `integration` test harness, apply the V33 migration, then exercise
the `WorkspaceStore` trait method.

**Tests to add:**

```
migration_fresh_db_pg
roundtrip_valid_fingerprint_pg
store_rejects_wrong_length_pg
store_rejects_empty_pg
list_fingerprints_scoped_pg
list_fingerprints_excludes_null_pg
list_fingerprints_includes_path_pg
list_fingerprints_agent_scoped_pg
overwrite_fingerprint_pg
```

**Estimated scope:** ~100 lines

**Independently shippable:** yes — requires a running PostgreSQL instance
(gated by `--features integration`, which already implies that).

---

### Slice D: K-Medoids Clustering Module (Priority 2)

**File to create:** `crates/ironclaw_hdc/src/cluster.rs`

**File to modify:** `crates/ironclaw_hdc/src/lib.rs` — add `pub mod cluster;`

**Feature gate:** none (pure crate, no new external dependencies)

**Public API:**

```rust
pub struct KMedoidsConfig {
    pub k: usize,
    pub max_iterations: usize,
}

pub struct HdcCluster {
    pub medoid_index: usize,        // index into input slice
    pub medoid: HdcVector,
    pub members: Vec<usize>,        // indices into input slice
    pub intra_similarity: f64,      // mean pairwise similarity within cluster
}

pub struct KMedoidsResult {
    pub clusters: Vec<HdcCluster>,
    pub iterations: usize,
    pub converged: bool,
}

pub fn k_medoids(vectors: &[HdcVector], config: &KMedoidsConfig) -> KMedoidsResult;
```

**Implementation notes:**

- Distance: `1.0 - similarity(a, b)` (fractional Hamming).
- Seeding: farthest-first from the global centroid — deterministic, no RNG.
- Assignment: each vector goes to the nearest medoid.
- Update: reselect medoid as the member that minimizes total intra-cluster
  distance.
- Repeat until stable or `max_iterations` is reached.

**Tests required:**

```
k_medoids_three_well_separated_groups   // 3 groups of 10; recovery >= 27/30
k_medoids_empty_input                   // returns empty clusters, 0 iterations, converged=true
k_medoids_k_greater_than_n              // k=15 on 5 vectors; returns 5 singletons, no panic
k_medoids_k_equals_one                  // single cluster contains all input vectors
k_medoids_deterministic                 // same input produces identical output on two calls
```

**Estimated scope:** ~250 lines

**Independently shippable:** yes

---

### Slice E: Structured Role-Filler Encoding (Priority 2)

**File to modify:** `crates/ironclaw_hdc/src/encoder.rs`

**Feature gate:** none (pure crate extension)

**Functions to add:**

```rust
/// Encode a set of (role, filler) field pairs.
/// Each pair: bind(codebook.get_or_create(role), encode_text(value, codebook)).
/// Bundle all bound pairs with equal weight.
pub fn encode_structured(
    fields: &[(&str, &str)],
    codebook: &mut Codebook,
) -> HdcVector;

/// Encode a directed causal link (cause → effect).
/// Cause component uses permute(1); effect component uses permute(2).
/// Result is asymmetric: encode_causal_link("A","B") != encode_causal_link("B","A").
pub fn encode_causal_link(
    cause: &str,
    effect: &str,
    codebook: &mut Codebook,
) -> HdcVector;
```

**Tests required:**

```
encode_structured_differs_from_flat_text
    // same content as field pairs vs concatenated string → similarity < 0.9
encode_structured_field_role_matters
    // [("subject","rust"),("object","tokio")] vs [("subject","tokio"),("object","rust")]
    // → different vectors (role binding breaks symmetry)
encode_structured_empty_fields
    // empty slice does not panic; returns deterministic vector
encode_causal_direction_matters
    // encode_causal_link("A","B") != encode_causal_link("B","A")
encode_causal_unbinding_recovers_direction
    // bind(result, permuted_cause_role) is closer to encode_text("A") than "B"
```

**Estimated scope:** ~100 lines

**Independently shippable:** yes

---

### Slice F: Native Memory HDC Write-Time Fingerprinting (Priority 2)

**Audit item:** workspace fingerprinting currently passes `tags: &[]` and
`ironclaw_memory_native` indexer does not yet call HDC after chunk replacement.

**File to modify:** `crates/ironclaw_memory_native/src/indexer.rs`

**Feature gate:** `#[cfg(feature = "hdc")]` block inside the owning module

**What it does:**

After the existing `replace_document_chunks_if_current` call succeeds and `hdc`
is enabled:

1. Call `encode_document(DocumentEncodingInput { content, tags: &[], path })`
   using a workspace-scoped `Codebook`.
2. Call `upsert_document_fingerprint(path, &fingerprint.to_bytes())` on the
   repository.

**New trait method** (confirm exact trait name from the file; add a no-op
default so existing implementors do not break):

```rust
async fn upsert_document_fingerprint(
    &self,
    path: &str,
    fingerprint: &[u8],
) -> Result<(), MemoryError>;
```

**Tests required:**

```
native_write_stores_hdc_fingerprint
    // write through indexer with hdc enabled; assert stored fingerprint is 1,280 bytes
native_write_fingerprint_changes_on_content_update
    // write once, update content, write again; assert fingerprint bytes differ
```

**Estimated scope:** ~80 lines (implementation) + ~60 lines (tests)

**Independently shippable:** yes (does not require Slice C unless the tests
target the PostgreSQL backend).

---

### Slice G: Heartbeat HDC State Persistence (Priority 2)

**Audit items:** `heartbeat_state_persists_across_restart`,
`heartbeat_corrupt_state_resets`, and `heartbeat_multi_user_isolation` are all
open.

**File to modify:** `src/agent/heartbeat.rs`

**Feature gate:** `#[cfg(feature = "hdc")]`

**What it does:**

After each `apply_hdc_novelty` call:

1. Serialize `HeartbeatHdcState` (already `Serialize`/`Deserialize`) to JSON.
2. Write to a user-scoped workspace path:
   `engine/.hdc/heartbeat_state/{user_id}.json`.
   This path must satisfy `is_engine_runtime_path()` so it is excluded from
   FTS, vector search, HDC search, and identity-file injection.

On `HeartbeatRunner` construction:

1. Attempt to load `HeartbeatHdcState` from that path.
2. Missing: start with empty state (current behavior, unchanged).
3. Present but deserialization fails (corrupt): log at `debug!`, start with
   empty state, overwrite the corrupt file on the next write.

**Tests required:**

```
heartbeat_state_persists_across_restart
    // run N cycles with a repeated observation; serialize; reconstruct runner from
    // same workspace; assert the observation is still classified as repeated
heartbeat_corrupt_state_resets
    // write corrupt JSON to the state path; construct runner; assert clean startup,
    // no panic, state is empty
heartbeat_state_isolated_by_user
    // two users each run separate runners; assert their state documents are at
    // different workspace paths and do not share accumulator state
```

**Estimated scope:** ~60 lines

**Independently shippable:** yes

---

### Slice H: Search Config Wiring (Priority 1)

**Audit item:** `IRONCLAW_HDC_SEARCH_SHADOW` is not wired into `SearchConfig`;
`hdc_rank`/`hdc_score` fields on `SearchResult` are not surfaced in
`MemorySearchTool` output JSON.

**Files to modify:**

- `src/app.rs` (or whichever call site constructs `SearchConfig` from
  `HdcConfig`) — set `.with_hdc(config.search_enabled)` and
  `.with_hdc_shadow_mode(config.search_shadow)`.
- `src/tools/builtin/memory.rs` — in `MemorySearchTool::execute`, if the
  result has `hdc_rank: Some(_)` or `hdc_score: Some(_)`, include them in
  the serialized output JSON under keys `"hdc_rank"` and `"hdc_score"`.

**Feature gate:** `#[cfg(feature = "hdc")]` guard around the HDC field
injection in the tool output.

**Tests required:**

```
search_tool_output_includes_hdc_fields_when_shadow_enabled
    // run MemorySearchTool::execute with use_hdc=true, hdc_shadow_mode=true,
    // at least one fingerprinted document in workspace;
    // assert output JSON contains "hdc_rank" and "hdc_score" keys
search_tool_output_no_hdc_fields_when_disabled
    // same tool, use_hdc=false; assert neither key appears in any result object
```

**Estimated scope:** ~30 lines (wiring) + ~50 lines (tests)

**Independently shippable:** yes — underlying machinery already exists in
`SearchResult` and `SearchConfig`.

---

### Slice Summary Table

| Slice | Priority | Files | Feature gate | Est. lines | Shippable alone |
|---|---|---|---|---|---|
| A: Fixture corpus harness | 1 | `tests/workspace_hdc_fixtures.rs` (new) | `hdc` | ~200 | yes |
| B: Active dedup caller tests | 1 | `tests/workspace_hdc_dedup.rs` | `hdc,libsql` | ~150 | yes |
| C: PostgreSQL HDC storage | 1 | `tests/workspace_hdc.rs` | `hdc,integration` | ~100 | yes |
| H: Search config wiring | 1 | `src/tools/builtin/memory.rs`, `src/app.rs` | `hdc` | ~80 | yes |
| D: K-medoids clustering | 2 | `crates/ironclaw_hdc/src/cluster.rs` (new), `lib.rs` | none | ~250 | yes |
| E: Structured role-filler encoding | 2 | `crates/ironclaw_hdc/src/encoder.rs` | none | ~100 | yes |
| F: Native memory fingerprinting | 2 | `crates/ironclaw_memory_native/src/indexer.rs` | `hdc` | ~140 | yes (C not required unless PG tests) |
| G: Heartbeat state persistence | 2 | `src/agent/heartbeat.rs` | `hdc` | ~60 | yes |

Start with A, B, C, and H — they share no file-level conflicts and close the
four most significant open items in the current audit checklist.

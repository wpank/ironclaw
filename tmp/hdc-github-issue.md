# Issue Draft: Add Optional HDC Fingerprints for Workspace Memory Deduplication and Novelty Detection

Suggested labels: `area:workspace`, `area:memory`, `enhancement`, `needs-design-review`

## Summary

Introduce an optional Hyperdimensional Computing (HDC) layer for IronClaw
workspace memory. The goal is to give memory documents a cheap local fingerprint
that can support near-duplicate detection, heartbeat novelty checks, and later
search-ranking experiments without replacing the existing FTS/vector search
pipeline.

This issue is about building the capability behind feature flags and proving it
with tests and benchmarks before enabling user-facing behavior by default.

## Current Implementation Status

An initial implementation exists, but it should be reviewed as staged work, not
as a finished rollout:

- Core `crates/ironclaw_hdc` is implemented and passes crate tests/clippy.
- The main crate compiles with `--features hdc`.
- The nullable `memory_documents.hdc_fingerprint` column exists for PostgreSQL
  and libSQL.
- Shadow fingerprinting is wired for workspace write/append/patch paths.
- Workspace HDC config is default-off and uses `IRONCLAW_HDC_*` env names,
  with legacy `HDC_DEDUP_*` fallback support for dedup thresholds.
- Heartbeat novelty uses separate `HEARTBEAT_HDC_*` env names and is also
  default-off.
- Active dedup is implemented for configured `MemoryWriteTool` non-append
  writes; direct `Workspace::write/append/patch` calls remain behavior-preserving.
  Caller-level block/warn/force tests still need to be added for configured modes.
- Live workspace fingerprinting currently encodes `DocumentEncodingInput {
  content, tags: &[], path }`; the backfill command can read metadata tags, so
  tag inputs need normalization before tag-sensitive promotion.
- Heartbeat novelty is in-memory runner state only; persistence and notification
  metadata are pending.
- Search HDC only annotates/reranks existing FTS/vector results; HDC-only search
  is pending. HDC scoring currently loads fingerprints for the primary
  `user_id`/nullable `agent_id`, not additional read scopes.
- Exact duplicate content at different paths can share a fingerprint in the
  current no-tag workspace path because content dominates path-only differences;
  path-sensitive search still needs fixture-backed tuning before promotion.

## Why This Is Worth Doing

The practical win is memory hygiene. Agents write a lot of durable notes:
decisions, runbooks, observations, summaries, and heartbeat findings. Without a
cheap local similarity check, repeated notes keep getting stored and searched as
if they were independent facts. HDC gives IronClaw a derived signal that is:

- **local**: no provider call, no network dependency, no API key required,
- **deterministic**: same input produces the same fingerprint under a fixed
  encoder/version,
- **cheap**: comparisons are XOR plus popcount and can be scanned locally,
- **private**: content never leaves the process,
- **reversible**: fingerprints are derived metadata — disabling HDC leaves
  inert bytes, not broken state.

That makes it a good candidate for conservative deduplication and novelty
detection before considering more ambitious search-ranking changes.

## Background

IronClaw already has workspace memory: agents can write documents, read them
later, and search them with hybrid retrieval. Today that search path is built
around full-text search and optional neural embeddings. That is the right base,
but it leaves a few practical gaps:

- Memory can accumulate repeated or near-repeated notes because write-time
  deduplication is limited.
- Heartbeat checks can repeat the same observation because the runner has no
  cheap local notion of "I have already said this recently."
- Search quality depends on the current FTS/vector signals. There is no local
  structural-similarity signal that works offline and can be scanned quickly.

Hyperdimensional Computing is a good fit for the local-signal part of this
problem. In this proposal, every document can be encoded into a fixed
10,240-bit vector. Comparing two vectors is just XOR plus popcount. The result
is not a semantic embedding, but it is deterministic, cheap, private, and useful
for structure-sensitive comparisons such as similar content, similar tags, and
similar path patterns.

## What HDC Would and Would Not Do

HDC would:

- Compute a 1,280-byte fingerprint for workspace memory documents.
- Compare fingerprints locally without calling an embedding provider.
- Detect exact or near-exact repeated memory writes.
- Track whether heartbeat findings look new or repeated.
- Provide a candidate third search signal in shadow mode.

HDC would not:

- Replace FTS.
- Replace neural embeddings.
- Create a second memory system, transcript-backed store, or side channel.
- Become the only deduplication mechanism.
- Change search ranking until shadow-mode metrics show value.
- Bypass workspace authorization, path validation, system-prompt injection
  checks, or DB backend parity requirements.

## Workspace Memory Invariants

HDC fingerprints are derived metadata inside the existing workspace memory
system. They must preserve:

- file-like path semantics,
- layer routing,
- chunking and search behavior,
- identity and system-prompt loading,
- protected path checks,
- prompt-injection validation,
- user, agent, and layer scoping.

Backfill and live write fingerprinting must use the same metadata/tag inputs
before tag-sensitive behavior is promoted. Search must be off by default; the
first enabled mode should be shadow annotation. Live reranking should require a
separate explicit setting, preferably `IRONCLAW_HDC_SEARCH_MODE=off|shadow|live`,
plus fixture-backed retrieval quality gates.

## Proposed Architecture

Add a new crate:

```text
crates/ironclaw_hdc/
  src/vector.rs     # HdcVector: bind, permute, similarity
  src/bundle.rs     # majority and decaying accumulators
  src/codebook.rs   # deterministic symbol vectors and item memory
  src/encoder.rs    # document and text encoders
  src/dedup.rs      # threshold-based duplicate decisions
  src/scan.rs       # linear scan helpers
  src/error.rs
```

The main representation is:

```rust
pub struct HdcVector([u64; 160]);
```

That is 10,240 bits per vector. The core operations are:

- bind: word-wise XOR,
- bundle: per-bit majority vote,
- permute: cyclic bit shift,
- similarity: normalized Hamming similarity.

Workspace integration would be behind an `hdc` feature flag and default off.

## Implementation Phases

### Phase 0: Build Fixtures and Baselines

Before changing behavior, create a small labeled fixture set:

- exact duplicate memory documents,
- near duplicates,
- same topic but not duplicates,
- same tags with different content,
- repeated heartbeat observations,
- recurring heartbeat observations after a delay,
- search queries with labeled relevant documents.

Measure:

- duplicate precision and false block rate,
- write-path latency overhead,
- fingerprint scan cost,
- heartbeat false suppression risk,
- search relevant@10, MRR, and NDCG@10.

### Phase 1: Core HDC Library

Build `crates/ironclaw_hdc` with no dependency on the main IronClaw crate.

Required coverage:

- algebraic unit tests,
- property tests for bind/permute/serialization invariants,
- deterministic golden tests for seeded vectors,
- encoder tests for empty, unicode, long, and tagged inputs,
- criterion benchmarks for vector ops, encoding, and scans.

This phase should not touch workspace memory behavior.

### Phase 2: Store Fingerprints in Shadow Mode

Add nullable fingerprint storage to `memory_documents` for both backends:

- PostgreSQL migration, likely `V33__memory_hdc_fingerprint.sql`,
- libSQL migration in `src/db/libsql_migrations.rs`,
- `MemoryDocument.hdc_fingerprint: Option<Vec<u8>>`,
- scoped store APIs to update and list document fingerprints.

Then compute fingerprints after successful workspace writes. Behavior should not
change yet. Existing documents can remain `NULL` until a backfill command runs.

Important implementation detail: this phase must preserve PostgreSQL/libSQL
parity and should skip internal runtime-state documents that already skip
semantic indexing.

### Phase 3: Add Conservative Write-Time Deduplication

When a user or agent calls `memory_write`, compute the would-be document
fingerprint before creating/updating the document, compare it to existing scoped
fingerprints, and return one of:

- `unique`: write normally,
- `similar`: write normally but include a warning,
- `duplicate`: block by default and report the existing path/id/similarity,
  unless `force=true`.

This must be tested through the actual `MemoryWriteTool`, not only through HDC
helper functions. A duplicate block should produce a tool output that an agent
can understand and retry with `force=true` if appropriate.

Important implementation detail: do the duplicate preflight before creating a
new workspace document. A blocked duplicate should not leave behind an empty
document at the requested path.

### Phase 4: Heartbeat Novelty Detection

Use a decaying HDC accumulator to track recent heartbeat findings. Start in
observe-only mode:

- encode the heartbeat response,
- compare it to the decaying summary of recent responses,
- record whether it appears novel, repeated, or uncertain,
- do not suppress notifications by default.

Only enable suppression after tests and real traces show it does not hide novel
findings.

Heartbeat state must be scoped per user. It should not be stored in a global
host path that would break multi-user behavior.

### Phase 5: Search Fusion in Shadow Mode

Optionally compute HDC ranks during workspace search and attach them to results
without changing final ordering. Compare HDC ranking against current FTS/vector
fusion on fixture queries and sampled real queries.

Only use HDC in live fusion if it improves measured retrieval quality without
regressing ordinary searches.

The first enabled search mode must be shadow mode. Live reranking should not be
promoted by benchmark speed alone; it needs labeled search fixtures, NDCG/MRR
checks, and reviewer-visible rank deltas.

### Phase 6: Skill/Tool Matching, If Justified

If HDC proves useful for memory retrieval, consider a small capped boost for
skill or tool selection. This should remain a later experiment; exact keyword,
regex, policy, and permission logic must continue to dominate.

## Concrete Use Cases and Examples

### Immediate value (Phases 1–4)

#### 1. Agent rewrites the same decision multiple times

An agent summarizes a design decision after a conversation. Two days later,
a similar conversation happens. The agent writes another summary that says
the same thing in different words.

**Today:** Both notes are stored. `memory_search` returns both, consuming
context tokens with redundant information. Over weeks, the agent accumulates
5-10 versions of the same decision.

**With HDC dedup:** The second write detects similarity ≥ 0.78 to the first
note and warns the agent: "Similar document exists at
`decisions/api-auth-approach.md` (similarity: 0.84)." The agent can choose
to update the existing note instead of creating a new one. At ≥ 0.92, the
non-append write is blocked outright with a pointer to the existing document.
Current append and daily-log paths skip active dedup.

### 2. Heartbeat reports the same SSL certificate expiry for days

The heartbeat checks `HEARTBEAT.md` every 30 minutes. The checklist includes
"check certificate expiry dates." The LLM keeps reporting "SSL cert for
api.example.com expires in 12 days" every cycle for two weeks.

**Today:** The user receives the same notification 672 times (48 cycles/day ×
14 days). Notification fatigue causes them to ignore all heartbeat output.

**With HDC novelty:** After several identical reports, the decaying accumulator
can classify the observation as repeated. In the first rollout this is logged
for observe-only review; notification metadata is still pending. Suppression
stays disabled until runner-loop tests and canaries show it does not hide novel
findings. If the cert changes from
"12 days" to "2 days", the changed trigram signal may lower similarity enough
to classify the observation as novel depending on threshold tuning.

### Later value (Phases 5–6 and beyond)

#### 3. Offline search without an embedding provider

A user runs IronClaw without configuring an embedding API (no OpenAI key, no
NEAR AI, no Ollama). Today, `memory_search` falls back to FTS only.

**With HDC (Phase 5):** Search can compare the existing FTS/vector result set
against stored fingerprints and report HDC rank/score deltas without any API
call. HDC-only retrieval and tag-aware live-write fingerprints are future work;
the current live workspace write path stores fingerprints with `tags: &[]`.

### 4. Memory audit and cleanup tooling

An operator wants to find redundant documents across a user's workspace.

**With HDC:** A CLI command or diagnostic tool can scan all fingerprints
and report clusters of similar documents:

```text
$ ironclaw memory hdc audit --user default

Cluster 1 (3 documents, avg similarity 0.91):
  decisions/api-auth-v1.md
  decisions/api-auth-updated.md
  daily/2026-06-15.md (section: auth decision)

Cluster 2 (2 documents, avg similarity 0.85):
  runbooks/deploy-staging.md
  runbooks/deploy-production.md
```

This is a natural extension of dedup — same scan, but applied retroactively
across the entire workspace rather than at write time.

### 5. Cross-session context continuity

When an agent starts a new session, it searches memory for relevant context.
Two documents about the same topic written in different sessions might use
different vocabulary but share structural patterns (same tags, same path
conventions, same role-filler relationships).

**With HDC (Phase 5):** The structural similarity signal helps surface both
documents even when their prose is different enough that vector embeddings
rank them separately. The RRF fusion promotes documents that score well on
multiple signals.

#### 6. NEAR agent workspace hygiene at scale

In a multi-agent NEAR deployment, multiple agents write observations about
on-chain state — transaction volumes, gas prices, validator performance.
These observations are structurally repetitive: same path patterns
(`near/observations/YYYY-MM-DD.md`), same tags (`["near", "metrics"]`),
similar numeric content with different values.

**With HDC:** Dedup prevents the workspace from accumulating hundreds of
near-identical daily snapshots. The structural fingerprint catches "same
template, different numbers" patterns that exact-hash dedup would miss.
Heartbeat novelty can first label repeated "gas price normal" observations as
stale without changing notification behavior; later, after false-suppression
tests, it could suppress stale observations and surface genuine anomalies.

### Roko-derived cognitive architecture use cases

The Roko-derived material has several concrete HDC/memory patterns that are
portable to IronClaw and are summarized here without requiring source access.
They are documented fully in
`tmp/hdc-roko-cognitive-usecases.md`; the issue keeps the reviewer-facing
summary here. The IronClaw-specific advanced integration plan lives in
`tmp/hdc-ironclaw-innovation-integrations.md`, with concrete examples,
fixture rows, telemetry shapes, benchmark commands, and promotion gates in
`tmp/hdc-innovation-examples-and-benchmarks.md`. These are follow-up directions
unless explicitly called out in the current PR scope.

#### 7. Context assembly dedup and attention budgeting

Roko's context assembler gathers durable knowledge, recent episodes, inlined
files, and recent signals, scores each chunk, removes HDC-similar chunks, then
fits the result under a token budget. When two chunks are near-duplicates, the
higher-priority or higher-relevance source wins.

**IronClaw use:** before prompt assembly, fingerprint candidate memory/search
sections and drop redundant chunks. Measure prompt-token savings, dropped-chunk
citation rate, and recorded QA regressions before making this default.

#### 8. Knowledge admission with novelty and anti-knowledge

Roko's knowledge admission path uses confidence, source trust, and novelty
(`1.0 - max_similarity`) before promoting a candidate into durable memory. It
also models anti-knowledge: verified negative memory that suppresses harmful or
invalid actions.

**IronClaw use:** store repeated agent claims as candidates first. Promote only
when the claim is novel enough and backed by trusted evidence. If a candidate is
similar to admitted anti-knowledge, surface a warning instead of injecting it as
positive context.

#### 9. Dream consolidation dedup and playbook promotion

Roko's dream cycle clusters completed episodes by task shape, outcome, and
model; distills cluster lessons; promotes repeated successes into playbook
candidates; and screens staged knowledge for HDC redundancy before promotion.

**IronClaw use:** batch completed threads or routines into clusters such as
"DB parity fixes" or "gateway auth regressions." Use HDC to reject redundant
distilled lessons and require verification evidence before a playbook becomes
durable memory.

#### 10. Counterfactual strategy transfer

Roko computes cluster-structure fingerprints from task type, model, outcome,
success/failure balance, gate signals, and failure reason. It compares failure
clusters with successful clusters and drafts strategy-transfer hypotheses.

**IronClaw use:** if webhook auth tests repeatedly fail, compare that failure
cluster with successful route-test clusters. A high HDC match can suggest
"add caller-level route fixtures that drive auth, body limits, and persistence
together" as a draft remediation.

#### 11. Code structural fingerprinting

Roko's code-intelligence design fingerprints symbols with role vectors, name
trigrams, and surrounding context:

```text
symbol_hv = bind(role_vector(kind), bundle(name_trigrams, context_vector))
file_hv   = bundle(symbol_hvs for all symbols in file)
```

**IronClaw use:** later code-search work can fingerprint Rust symbols to find
similar handlers, DB backend implementations, and parity-test gaps. HDC should
produce candidate links only; AST facts and caller-level tests still decide
correctness.

#### 12. HDC substrate and weighted top-k retrieval

Roko's Mirage HDC substrate exposes store-compatible retrieval: put an engram,
store a vector, query with `text_query`, rank by HDC score, then apply normal
kind/author/session/tag/time filters. Its flat index ranks by
`similarity * weight` and supports insert, remove, and weight updates.

**IronClaw use:** an HDC diagnostic search can show both raw similarity and a
weighted score using recency, confidence, pinning, or successful citation
history. Weighting should stay off until raw HDC quality is measured.

#### 13. Heartbeat as a cognitive clock

Roko frames heartbeat as observable gamma/theta/delta hot graphs: fast action
cycles, periodic reflection cycles, and idle/offline consolidation cycles.

**IronClaw use:** current heartbeat HDC should stay observe-only. Later,
repeated/novel classifications could tune cadence: repeated stable findings
lengthen intervals, materially novel findings shorten intervals or trigger
reflection. Suppression still requires false-suppression fixtures and canaries.

#### 14. Multi-agent shared workspace dedup

When multiple agents work on related subproblems in a swarm, they
independently discover and write similar knowledge to a shared
workspace. Agent A writes "the API uses JWT authentication," agent B
writes "authentication is handled by JWT tokens," agent C writes
"auth flow: JWT-based."

**Without HDC:** Three near-identical documents accumulate. Memory
search returns all three, consuming 3× the context tokens for the same
fact.

**With HDC:** The second and third writes detect similarity to the first.
In warn mode, agents B and C see "similar document exists at
`findings/auth-mechanism.md`" and can choose to add their evidence to
the existing document rather than creating a new one. In block mode,
only the first write succeeds and subsequent agents are pointed to it.

This is especially valuable in orchestrated workflows where a
coordinator dispatches parallel research tasks. The first implementation should
stay scoped by user and agent; cross-agent dedup belongs behind an explicit
shared-scope policy.

#### 15. Bounty submission similarity (NEAR marketplace)

In a NEAR-based agent bounty marketplace, multiple agents submit work
for the same bounty specification. Exact content hashes catch identical
submissions, but HDC fingerprints catch structurally similar submissions
— two agents who solved the same problem with minor code style
differences, or who plagiarized each other's approach with superficial
changes.

**With HDC:** Each submission is fingerprinted. Before accepting a
submission, the marketplace compares it against all prior submissions
for the same bounty. High similarity flags potential plagiarism for
reviewer attention. This complements the content hash
(`spec_hash`/`result_hash`) that catches only identical artifacts.

## User Experience

The first visible behavior should be memory deduplication:

```json
{
  "status": "blocked",
  "append": false,
  "path": "projects/new-note.md",
  "dedup": {
    "decision": "duplicate",
    "existing_path": "projects/original-note.md",
    "existing_id": "...",
    "similarity": 0.94,
    "retry": "Pass force=true to write anyway."
  }
}
```

For similar but non-duplicate content:

```json
{
  "status": "written",
  "append": false,
  "path": "projects/new-note.md",
  "dedup": {
    "decision": "similar",
    "existing_path": "projects/related-note.md",
    "similarity": 0.81
  }
}
```

The default rollout should be:

1. store fingerprints only,
2. warn only,
3. block only after the duplicate threshold has zero false blocks on fixtures.

## Testing Requirements

This feature touches a side-effecting path, so helper-only tests are not enough.

Current coverage:

- HDC crate unit, property, doctest, golden, and benchmark targets exist.
- libSQL storage, migration, scoped fingerprint reads/writes, shadow
  fingerprinting, null-fingerprint handling, user/agent scoping, and default-off
  `MemoryWriteTool` output shape are covered.
- Workspace preflight tests cover exact duplicates, near duplicates, unique
  related content, no-ghost behavior, protected-path injection rejection, and
  final-content fingerprints for append/patch paths.
- Heartbeat accumulator and raw runner checks exist for the default-off and
  fail-open paths.
- Search HDC helper/config tests exist for existing-result annotation/fusion.

Remaining test gaps:

- PostgreSQL HDC storage contract tests.
- Configured `MemoryWriteTool.execute()` block/warn/force caller tests for
  `off`, `warn`, `block`, and `force=true`, including output shape and no ghost
  document creation.
- Mocks for caller tests must capture all production arguments: `user_id`,
  nullable `agent_id`, layer/scope, path, final content, append mode, force flag,
  and dedup config.
- Layer-aware active dedup through `MemoryWriteTool`.
- Heartbeat runner-loop suppression, notification metadata, persistence, and
  corrupt-state recovery tests.
- Fixture-backed production search quality tests and HDC-only retrieval tests.
- Full feature-matrix coverage with HDC disabled and enabled.
- PostgreSQL and libSQL contract tests should both cover migration, nullable
  rows, 1,280-byte validation, scoped listing, missing-document errors,
  overwrite, and null exclusion.

Benchmark work:

- vector operations,
- document encoding,
- dedup scan at 10K and 100K fingerprints,
- write-path p95 overhead measurement,
- search-path overhead measurement in shadow mode.

## Acceptance Criteria

- `crates/ironclaw_hdc` builds, tests, and benchmarks independently.
- HDC integration is behind an `hdc` feature flag and default off.
- Fingerprints are persisted for both PostgreSQL and libSQL.
- Shared DB trait changes land before backend-specific workspace use.
- PostgreSQL and libSQL contract tests cover the same fingerprint behavior.
- Existing behavior is unchanged when HDC is disabled.
- Shadow fingerprinting overhead is measured and reported before warning/block
  mode is promoted.
- Duplicate blocking is tested through `memory_write` before block mode is
  promoted.
- No false duplicate blocks occur on the unique fixture set at the chosen block
  threshold.
- Heartbeat novelty starts observe-only and remains user-scoped.
- Search fusion remains shadow-only until retrieval metrics justify live ranking.
- `FEATURE_PARITY.md` is checked and either updated or recorded as not
  applicable.
- Relevant docs are updated in the same branch when behavior changes:
  `src/workspace/README.md`, `src/db/CLAUDE.md`, `src/tools/README.md`,
  setup docs for config/onboarding changes, API docs for web-visible behavior,
  and `CHANGELOG.md` for user-visible rollout changes.

## Security and Auth Boundaries

- No changes to listeners, web routes, bearer-token auth, webhook auth,
  CORS/origin checks, body limits, rate limits, allowlists, secrets handling,
  sandboxing, or outbound HTTP policy.
- HDC/product/host-runtime paths must not mint `TrustedInboundTurnRequest` or
  call trusted trigger submitter factories.
- Treat fingerprints as sensitive content-derived metadata.
- Do not log raw fingerprints.
- Do not expose fingerprints except through already-authorized workspace or
  bounded diagnostic paths.

## Risks

- False positives could block legitimate memory writes.
- Tags or paths could dominate fingerprints if weights are poorly chosen.
- Search fusion could add complexity without quality lift.
- Heartbeat suppression could hide important repeated-but-still-relevant issues.
- A cache could introduce stale fingerprint behavior if delete/update paths do
  not invalidate it.

Mitigations:

- feature flag default off,
- warn mode before block mode,
- conservative thresholds,
- caller-level tests,
- fixture-based evaluation,
- easy rollback by disabling HDC behavior while leaving fingerprints as inert
  derived data.

## What This Unlocks Later

These are not in scope for the first PR, but HDC fingerprints could become a
foundation for them if the measured quality is good:

| Future capability | What HDC enables | Depends on |
|---|---|---|
| **Memory audit CLI** | Cluster similar documents, find redundancy, suggest merges | Phase 2 (stored fingerprints) |
| **Workspace health score** | Ratio of unique vs near-duplicate content as a quality metric | Phase 3 (dedup decisions) |
| **Adaptive heartbeat intervals** | If observations are consistently stale, increase interval; if novel, decrease | Phase 4 (novelty scores) |
| **Smart memory consolidation** | Automatically merge near-duplicate documents into a single authoritative version | Phase 3 + LLM merge |
| **Offline-first search** | FTS + HDC search with no embedding provider configured | Phase 5 |
| **Multi-agent dedup** | Agents sharing a workspace avoid writing the same knowledge independently | Phase 3 + cross-agent scope |
| **Content drift detection** | Track how a document's fingerprint changes across versions — structural refactors vs cosmetic edits | Phase 2 (fingerprint per version) |
| **Context assembly dedup** | Remove near-duplicate sections from prompt composition before token budget allocation | Phase 1 (crate only, no DB) |
| **Code structural fingerprinting** | Symbol-level fingerprints for structural code search, refactoring candidates, structural diff | Phase 1 + code intelligence integration |

## Out of Scope for the First PR

- Live search reranking.
- Skill or tool selection changes.
- Per-chunk fingerprints.
- Default-on duplicate blocking.
- Any replacement of existing FTS/vector search.
- Memory audit CLI (depends on Phase 2 being stable first).
- Adaptive heartbeat intervals (depends on Phase 4 being proven).

## Scale

- ~1,500–2,000 lines of new library code in `crates/ironclaw_hdc/`
- ~200 lines of integration code across 8 existing files (all `#[cfg(feature = "hdc")]`)
- ~225 individually tracked test and benchmark items across 6 phases
- 1 nullable column added to `memory_documents` (both backends)
- Public structs are extended with optional fields (`MemoryDocument`,
  `SearchResult`); check source compatibility for struct literals.

## References

HDC foundations:

- Kanerva, P. (2009). "Hyperdimensional Computing: An Introduction to
  Computing in Distributed Representation with High-Dimensional Random
  Vectors." *Cognitive Computation*, 1(2), 139–159.
- Kleyko, D. et al. (2022). "A Survey on Hyperdimensional Computing:
  Theory, Algorithms, and Applications." *ACM Computing Surveys*.
- Thomas, A. et al. (2021). "A Theoretical Perspective on
  Hyperdimensional Computing." *JAIR*, 72, 215–249.

Similarity and dedup:

- Charikar, M. (2002). "Similarity Estimation Techniques from Rounding
  Algorithms." *STOC '02*. — SimHash as a candidate dedupe/search index
  technique.

Memory and consolidation:

- McClelland, J. et al. (1995). "Why There Are Complementary Learning
  Systems." *Psychological Review*. — Supports session-log vs
  durable-summary separation where HDC dedup keeps summaries clean.
- Mattar, M. and Daw, N. (2018). "Prioritized Memory Access Explains
  Planning and Hippocampal Replay." *Nature Neuroscience*. — Scoring
  idea for consolidation candidate selection by prediction error.
- Richards, B. and Frankland, P. (2017). "The Persistence and
  Transience of Memory." *Neuron*. — Forgetting as regularization,
  aligned with HDC's decaying accumulator.
- Park, J. et al. (2023). "Generative Agents: Interactive Simulacra of
  Human Behavior." — Memory and reflection in language-agent
  simulations; reflection dedup is a natural HDC application.

Context and attention:

- Liu, N. et al. (2024). "Lost in the Middle: How Language Models Use
  Long Contexts." — Motivates position-aware dedup: redundant sections
  in mid-context degrade attention.
- Itti, L. and Baldi, P. (2005). "Bayesian Surprise Attracts Human
  Attention." *NeurIPS*. — HDC similarity as a cheap proxy for Bayesian
  surprise in context selection.

Affect and somatic markers:

- Mehrabian, A. (1996). "Pleasure-Arousal-Dominance: A General
  Framework for Describing and Measuring Individual Differences in
  Temperament." — PAD model maps to decaying accumulator for session
  tone.
- Damasio, A. (1994). *Descartes' Error*. — Somatic markers as
  `bind(approach, outcome)` HDC vectors.

## Companion Plan

Detailed implementation checklist, test matrix, and phase gate criteria:

- [`tmp/hdc-implementation-plan.md`](./hdc-implementation-plan.md)

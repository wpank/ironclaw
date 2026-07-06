# Roko-Derived HDC, Memory, and Cognitive Architecture Use Cases

This note extracts portable design patterns from the Roko-derived material and
rewrites them as self-contained IronClaw design inputs. Treat these as design
inputs and future use cases, not as current IronClaw behavior or a runtime
dependency.

## Source Map

| Roko area | Provenance label | Portable idea | IronClaw anchor |
|---|---|---|---|
| Signal kernel | `README.md`, `CLAUDE.md` | One content-addressed unit plus verbs: store/query, score, gate, route, compose, policy/react. | Workspace documents, search, tools, and agent loop boundaries. |
| HDC primitive | `roko-primitives` | 10,240-bit vectors, XOR bind, majority bundle, permute, Hamming similarity, item memory, tier routing. | `crates/ironclaw_hdc::{HdcVector, DocumentEncodingInput, encode_document}`. |
| Knowledge memory | `roko-neuro` | Durable knowledge entries, tiers, decay, admission gates, anti-knowledge, context assembly. | Workspace memory plus `Workspace::check_dedup`. |
| Context assembly | `roko-neuro/context` | Gather knowledge, episodes, files, recent signals; score; dedup similar chunks; fit under token budget. | Future prompt assembly dedup using `DocumentEncodingInput`. |
| Dream cycle | `roko-dreams` | Offline replay, cluster distillation, playbook promotion, counterfactual hypotheses, redundancy screening. | Future offline consolidation job, not live agent-loop behavior. |
| Code intelligence | `code-intelligence/hdc-fingerprints` | Role-vector + trigram + context fingerprints for symbols and files. | Future parser-backed code navigation fixtures. |
| Search/context | `code-intelligence/search-context` | Keyword, structural, HDC, embedding, and RRF hybrid search. | `SearchConfig`, `SearchResult.hdc_rank`, `SearchResult.hdc_score`. |
| HDC substrate | `hdc-substrate` | Store-compatible HDC retrieval with text query, weights, filters, and pruning. | Scoped workspace diagnostic search only. |
| HDC chain index | `hdc-index` | Flat weighted top-k Hamming index with explicit insert/remove/weight update. | Future cache/index with update/delete invalidation. |
| HDC precompile | `hdc-precompile` | Bounded project, bind, bundle, similarity, search, insert, remove surface. | Future bounded CLI/tool diagnostic, no raw cross-user scan. |
| Cognitive loop | `cognitive-loop` | Sense -> assess -> compose -> act -> verify -> persist -> react. | Existing agent/tool/workspace boundaries. |
| Heartbeat graph | `heartbeat-hot-graph` | Gamma/theta/delta ticks as observable hot graphs, not a hidden scheduler. | `HeartbeatHdcConfig` observe-only novelty first. |

## Captured Pattern Contracts

These contracts are the self-contained Roko-derived ideas, rewritten without
depending on the original source paths:

- **Derived fingerprints, not primary memory:** HDC vectors are metadata on
  existing workspace documents or candidate chunks. They do not create another
  memory store.
- **Scope before score:** user, agent, layer, path, and permission boundaries are
  applied before HDC results are trusted or shown.
- **Measure before promotion:** thresholds such as `0.85` for context dedup or
  `0.90` for redundancy are fixture-tuned starting points, not inherited
  constants.
- **Advisory before automatic:** context dedup, admission, dream consolidation,
  counterfactual transfer, routing, and heartbeat cadence changes start as
  diagnostics or draft recommendations.
- **Raw score stays visible:** any weighted rank must show raw HDC similarity
  beside the weighted score.
- **Parser/test facts beat similarity:** code HDC can suggest related symbols,
  but AST facts and caller-level tests decide equivalence.
- **Caller tests protect side effects:** if HDC changes a write, notification,
  search result, or tool selection, tests must drive the production caller.

## Current IronClaw Status

- HDC is behind the `hdc` feature flag and runtime-default-off settings.
- Current document fingerprints are stored as nullable `memory_documents`
  metadata in shadow mode.
- Active dedup is a `Workspace::check_dedup` primitive plus configured
  `MemoryWriteTool` non-append preflight; it is not a direct
  `Workspace::write/append/patch` behavior.
- Live workspace fingerprinting currently uses `tags: &[]`; backfill/tag-aware
  paths must be normalized before tag-sensitive behavior is promoted.
- Search HDC annotates or optionally reranks existing FTS/vector result sets; it
  does not provide HDC-only retrieval today.
- Heartbeat HDC novelty is in-memory runner state. Runner-loop coverage,
  persistence, notification metadata, and the multi-tenant path are pending.
- Code-symbol HDC and skill/tool selection are future work, not live selection
  behavior.
- Native memory path (Reborn) has zero HDC integration today. All HDC is in
  the legacy `src/workspace/` path.
- Subagent coordination has no HDC fingerprint dedup; each subagent result is
  stored independently with no similarity check against prior results.
- Existing test fixtures in `tests/fixtures/hdc_memory/` are read by the
  harness in `tests/workspace_hdc_fixtures.rs`, which covers all dedup,
  heartbeat, and search scenarios declared in `manifest.json`.
- Active dedup warn/block modes have no caller tests; only the helper
  `check_dedup` is unit-tested.
- PostgreSQL HDC methods (`update_document_hdc_fingerprint`, bulk fingerprint
  queries) are untested; only the libSQL path has integration coverage.
- Search HDC is wired via `SearchConfig.use_hdc` (env: `IRONCLAW_HDC_SEARCH_SHADOW`)
  and invokes `Workspace::apply_hdc_to_results` when enabled. HDC-only retrieval
  and multi-scope HDC scoring are not yet implemented.

## Feature and Use-Case Inventory

### 1. Store-native HDC retrieval

Roko's `HdcSubstrate` stores raw engrams keyed by content hash and stores an
HDC vector beside each engram. A query with tag `text_query=<query>` projects
the text into HDC, scans the index, then applies normal filters such as kind,
author, session, time range, tags, and minimum weight.

IronClaw adaptation:

- Keep document fingerprints in `memory_documents`.
- Add an HDC diagnostic/search path that can annotate or compare scoped
  workspace results by HDC score without replacing FTS/vector search.
- Preserve normal user, agent, layer, and path filtering before trusting or
  returning HDC-ranked results.
- Keep HDC-only retrieval as future work until quality and scope behavior are
  fixture-backed.

Implementation flow:

1. Resolve user/agent/layer scope using existing workspace semantics.
2. Load only scoped fingerprints.
3. Encode the query with `encode_text`.
4. Compute HDC similarity and rank.
5. Intersect with or annotate current search results.
6. Return `hdc_rank` and `hdc_score` as diagnostic fields.

Example:

```text
Query: "rollback failed migration after schema drift"
HDC candidates:
  runbooks/db-rollback.md        score 0.87
  incidents/schema-drift.md      score 0.82
  daily/2026-06-18.md section    score 0.79
Then apply user/agent scope and existing workspace permissions.
```

### 2. Context assembly dedup before prompt composition

Roko's context assembler gathers knowledge entries, recent episodes, inline
files, and recent signals; scores them; removes near-duplicate chunks using a
fixture-tuned threshold that might start around `0.85`; and keeps the
higher-priority, higher-score, larger chunk when two chunks overlap.

IronClaw adaptation:

- Use HDC fingerprints to dedup candidate context sections before allocating
  prompt budget.
- Prefer the higher-authority source when two chunks are near-identical:
  user-authored memory > verified runbook > recent daily log > transient note.
- Track token savings and answer-quality deltas before enabling by default.

Benchmark/metric hooks:

- duplicate chunk rate per turn,
- prompt-token savings,
- rate of dropped chunks later cited by the model,
- answer regression rate on recorded QA traces.

Selection sketch:

```text
candidate = {
  id,
  source_authority,
  token_count,
  baseline_rank,
  hdc_fingerprint
}

sort by source_authority desc, baseline_rank asc, token_count desc
for each candidate:
  if max_similarity(candidate, selected) >= threshold:
    drop candidate and record kept_id, similarity, saved_tokens
  else:
    keep candidate
```

Fixture shape:

```yaml
name: daily_log_vs_runbook
threshold_start: 0.85
candidates:
  - id: runbook/deploy.md
    source: verified_runbook
    expected: keep
  - id: daily/2026-06-18.md#deploy
    source: daily_log
    expected: drop
assertions:
  min_saved_tokens: 120
  dropped_cited_by_recorded_answer: false
```

### 3. Four-factor memory retrieval

Roko's `FourFactorScorer` combines recency, importance, relevance, and emotional
congruence. Relevance can be keyword or HDC similarity; importance derives from
confidence and knowledge tier; emotional congruence uses PAD-state similarity.

IronClaw adaptation:

- Keep current retrieval ranking as the baseline.
- Use HDC only as a measured relevance feature first.
- Later, combine recency, durable-memory confidence, HDC similarity, and user
  task state into an explicit scoring model.

Non-binding scoring sketch:

```text
score =
  w_recency    * recency_score +
  w_confidence * durable_confidence +
  w_hdc        * hdc_similarity +
  w_task       * task_state_match
```

This should stay out of the first HDC PR. It is a useful direction for a later
memory-ranker experiment because it gives a concrete scoring decomposition
instead of a single opaque similarity score.

### 4. Knowledge admission using novelty, confidence, and source trust

Roko's light admission gate evaluates a candidate by confidence, novelty
(`1.0 - max_similarity`), and source trust. It also models anti-knowledge: durable
negative memory that suppresses actions known to be harmful or invalid.

IronClaw adaptation:

- Store repeated agent claims as candidates first, not durable memory.
- Use HDC max similarity to quantify novelty against existing durable memory.
- Promote only candidates that are novel enough, trusted enough, and backed by
  verification or user evidence.
- Preserve "anti-memory" entries for warnings such as "do not use this migration
  approach on libSQL because it failed gate X."

Example:

```text
Candidate: "Use SQL feature F for both Postgres and libSQL migrations"
Similarity to anti-memory: 0.91
Source trust: gate failure
Decision: suppress/promote as anti-knowledge, do not inject as positive advice.
```

Fixture shape:

```json
{
  "candidate": {
    "path": "candidates/migrations/sql-feature-f.md",
    "confidence": 0.72,
    "source_trust": 0.95,
    "content": "Use SQL feature F for both Postgres and libSQL migrations."
  },
  "nearest_existing": {
    "path": "anti-knowledge/libsql/sql-feature-f.md",
    "kind": "anti_knowledge",
    "similarity": 0.91
  },
  "expected_decision": "suppress",
  "required_evidence": ["libsql parity gate failure"]
}
```

### 5. Dream consolidation and playbook promotion

Roko's dream cycle clusters episodes by plan, task type, outcome, and model,
distills the cluster into knowledge, promotes repeated successful clusters into
playbooks, and emits regression or mistake entries for repeated failures. It
uses HDC structure vectors for cross-domain strategy transfer and screens staged
knowledge for redundancy using a fixture-tuned threshold that might start
around `0.90`.

IronClaw adaptation:

- Batch completed threads or routines into clusters such as "schema migration
  tasks", "gateway auth changes", "tool-caller fixes".
- Use HDC to detect whether a distilled lesson is new or redundant before
  writing it to durable memory.
- Promote repeated successful clusters into operator-visible playbooks only
  after they have passed caller-level tests or user confirmation.

Example:

```text
Cluster: 7 libSQL migration fixes
Observed pattern: write shared DB trait first, then backend parity tests
Dream output: playbook candidate
HDC redundancy check: 0.64 vs existing playbooks -> novel enough
Promotion gate: requires passing integration tests before durable memory write
```

Fixture shape:

```yaml
cluster:
  task_type: db_parity_fix
  outcome: success
  evidence_threads: [t-101, t-118, t-141]
  gates: [libsql_integration, postgres_contract]
nearest_existing_playbook:
  path: playbooks/db/backend-parity.md
  hdc_similarity: 0.64
expected:
  action: propose_playbook
  advisory_only: true
  promotion_requires: [passing_gates, maintainer_confirmation]
```

### 6. Counterfactual strategy transfer

Roko compares failure clusters with successful clusters using HDC structure
vectors and synthesizes hypotheses such as "apply the successful approach from
task type A to failure mode B." Its cluster vector binds task type, model,
outcome, success/failure balance, gate signals, and failure reason with
different permutations.

IronClaw adaptation:

- When a class of tasks repeatedly fails, compare its fingerprint against
  successful clusters.
- Propose a strategy transfer as a draft recommendation, not automatic action.
- Record whether the transferred strategy helped, then tune thresholds.

Example:

```text
Failure cluster: webhook auth tests repeatedly fail after route refactors
Similar success cluster: gateway rate-limit tests stabilized by caller-level fixtures
Suggested transfer: add a caller-level route fixture that drives auth + body limit together
```

Fixture shape:

```json
{
  "failure_cluster": {
    "task_type": "webhook_auth_route",
    "failure_reason": "route wrapper skipped body-limit assertion",
    "gate_result": "failed"
  },
  "nearest_success_cluster": {
    "task_type": "gateway_rate_limit_route",
    "strategy": "caller-level route fixture covering auth, body limit, persistence",
    "hdc_similarity": 0.78
  },
  "expected_output": "draft_recommendation",
  "auto_apply": false
}
```

### 7. Code structural fingerprints

Roko's code-intelligence design fingerprints symbols with:

- a role vector for symbol kind,
- character trigram vectors for names,
- a context vector from surrounding source,
- bind/bundle composition.

This supports "find similar" queries, clone detection, file-level similarity,
and hybrid search with keyword, structural, HDC, and embedding signals.

IronClaw adaptation:

- Keep this out of the memory PR unless code search is already in scope.
- Later, fingerprint Rust symbols and use HDC to find related handlers, DB
  backends, tool implementations, and tests.
- Use HDC for structural candidates, then verify with AST/parser facts.

Minimal symbol schema:

```json
{
  "file": "src/db/libsql/workspace.rs",
  "symbol": "update_document_hdc_fingerprint",
  "kind": "function",
  "name_trigrams": ["upd", "pda", "dat"],
  "context_hash": "stable-parser-context-hash",
  "ast_facts": {
    "async": true,
    "arguments": ["id", "fingerprint"],
    "returns_result": true
  }
}
```

Example:

```text
Anchor: LibSql workspace fingerprint update implementation
HDC similar candidates:
  Postgres workspace fingerprint update
  workspace repository trait mapping
  migration row-mapping tests
Use result as navigation aid, not semantic proof.
```

### 8. Heartbeat as cognitive clock

Roko frames heartbeat as gamma/theta/delta hot graphs:

- gamma: fast sense/assess/compose/act/verify/persist/react cycles,
- theta: periodic summarize/reflect/intervene cycles,
- delta: idle/offline consolidation cycles.

IronClaw adaptation:

- Current HDC heartbeat novelty should remain observe-only.
- Later, repeated/novel classifications can tune cadence:
  repeated stable findings lengthen intervals; novel or high-risk findings
  shorten intervals or trigger reflection.
- Suppression must be guarded by false-suppression fixtures and canaries.
- Current config fields are `HEARTBEAT_HDC_ENABLED`,
  `HEARTBEAT_HDC_DECAY_FACTOR`, `HEARTBEAT_HDC_REPEATED_THRESHOLD`,
  `HEARTBEAT_HDC_NOVEL_THRESHOLD`, and
  `HEARTBEAT_HDC_SUPPRESS_REPEATED`.
- Current limitations: runner-loop coverage, persistence, notification
  metadata, and multi-tenant heartbeat novelty are pending.

Example:

```text
Repeated low disk warning with same path and same free-space band:
  classify repeated; record metadata; leave notification behavior unchanged until suppression is explicitly enabled.
Different path or much lower free-space band:
  classify novel; preserve notification.
```

False-suppression fixture:

```json
[
  {"cycle": 1, "text": "disk /data is 85% full", "expected": "novel"},
  {"cycle": 2, "text": "disk /data is 85% full", "expected": "repeated_or_uncertain"},
  {"cycle": 3, "text": "disk /data is 92% full", "expected": "novel"}
]
```

### 9. Weighted HDC top-k index

Roko's Mirage HDC index ranks `score = similarity * weight`, supports insert,
remove, and weight updates, and skips zero-weight entries.

IronClaw adaptation:

- Use a weight term only after the raw HDC score is proven useful.
- Candidate weights could come from document recency, user pinning, memory tier,
  or successful citation history.
- Keep update/delete invalidation explicit if an in-memory cache is added.
- Show raw similarity beside weighted score in diagnostics.
- Enforce max `k`, max candidate count, and explicit cache invalidation for
  write/update/delete.

### 10. HDC operations as a tool or precompile-like surface

Roko's HDC precompile exposes bounded operations: project bytes/tokens, bind,
bundle, similarity, search, insert, and remove. It caps `k` and bundle size to
avoid unbounded work.

IronClaw adaptation:

- If HDC becomes a diagnostic tool, expose only bounded operations.
- Enforce vector length, max result count, max bundle inputs, and no raw
  cross-user scan.
- Prefer higher-level workspace commands over a generic vector mutation API.
- Do not expose raw insert/remove vector mutation to normal users unless it is
  scoped, audited, and invalidates search/index caches.

### 11. Cross-domain resonance and pattern metabolism

Roko's roadmap treats HDC fingerprints as a shared pattern language across
knowledge, code, coordination events, and runtime observations. Similarity can
identify structural analogies across domains, while decay/demurrage prevents
unused patterns from persisting indefinitely without evidence.

IronClaw adaptation:

- Keep first use domain-local: workspace memory dedup.
- Later compare fingerprints across memory documents, tool outcomes, heartbeat
  findings, and code symbols only after scope and privacy rules are explicit.
- Quantify whether cross-domain hits lead to better actions; otherwise keep
  them as diagnostics.

### 12. Cognitive tier routing and energy budget

Roko pairs HDC/tier primitives with cheap routing decisions: T0 suppresses an
LLM call, T1 uses a cheap model, and T2 uses a stronger model when vitality or
risk justifies it.

IronClaw adaptation:

- HDC novelty could become one input to model/tool routing after memory and
  heartbeat evidence exists.
- Candidate policy: repeated low-risk context could lower routing cost, while
  novel high-risk context could request stronger verification.
- This belongs after the memory and heartbeat evidence exists.

## Practical IronClaw Examples to Add to Fixtures

Existing anchors:

- `tests/fixtures/hdc_memory/manifest.json` for fixture manifest structure.
- `crates/ironclaw_hdc/examples/dedup_demo.rs` for dedup API shape.
- `crates/ironclaw_hdc/examples/heartbeat_novelty.rs` for novelty API shape.

| Fixture | Input | Expected use | Artifact |
|---|---|---|---|
| Context chunk dedup | Two daily-log chunks and one durable runbook repeat the same deploy rule. | Keep runbook; drop repeated daily chunks; record saved tokens and dropped-citation status. | `tests/fixtures/hdc_memory/roko_inspired/context_dedup/daily_log_vs_runbook.jsonl` |
| Admission novelty | New lesson is 0.93 similar to existing durable memory. | Defer or merge; do not create a second durable note. | `tests/fixtures/hdc_memory/roko_inspired/admission/near_duplicate_lesson.jsonl` |
| Anti-knowledge | Failed migration approach is similar to a proposed plan. | Surface warning before write/execution; do not inject as positive memory. | `tests/fixtures/hdc_memory/roko_inspired/admission/anti_knowledge_migration.jsonl` |
| Dream playbook | Multiple successful DB parity fixes share task shape. | Produce advisory playbook candidate with evidence count and gate history. | `tests/fixtures/hdc_memory/roko_inspired/dream/db_parity_playbook.jsonl` |
| Failure transfer | Repeated auth route failures resemble successful rate-limit route fixes. | Suggest caller-level route fixture pattern; do not auto-apply. | `tests/fixtures/hdc_memory/roko_inspired/dream/webhook_auth_transfer.jsonl` |
| Code HDC | Similar workspace storage functions across libSQL/Postgres. | Aid parity review; require AST/test confirmation. | `tests/fixtures/hdc_memory/roko_inspired/code/libsql_postgres_parity_symbols.jsonl` |
| Adaptive heartbeat | Same warning repeats for many cycles, then changes materially. | Classify repeat then novel; never suppress the materially changed finding. | `tests/fixtures/hdc_memory/roko_inspired/heartbeat/adaptive_cadence.jsonl` |
| Weighted HDC search | Exact duplicate has low weight, verified runbook has high weight. | Diagnostic ranking shows raw similarity and weighted score. | `tests/fixtures/hdc_memory/roko_inspired/search/weighted_runbook_vs_daily_log.jsonl` |

Reviewer commands:

```bash
cargo run -p ironclaw_hdc --example dedup_demo
cargo run -p ironclaw_hdc --example heartbeat_novelty
cargo test --features hdc,libsql --test workspace_hdc_dedup --no-fail-fast
```

Future fixture command once the Roko-inspired cases are added:

```bash
cargo test --features hdc,libsql --test workspace_hdc_roko_inspired --no-fail-fast
```

## What Fits This HDC PR

In scope now:

- document fingerprints,
- default-off shadow storage,
- `Workspace::check_dedup`,
- conservative `MemoryWriteTool` preflight,
- heartbeat novelty in observe-only mode,
- search annotation and explicitly configured reranking for existing result
  sets; the safe rollout path is shadow annotation first.

Useful follow-up PRs:

- context assembly dedup and token-savings metrics,
- durable-memory admission with novelty/source-trust gates,
- dream/offline consolidation of completed turns into playbook candidates,
- code-symbol HDC fingerprints for parity/navigation,
- adaptive heartbeat cadence based on novelty,
- weighted HDC index/cache with explicit invalidation,
- bounded HDC diagnostic tool surface.

## Implementation Sketches for Follow-Up Work

These sketches are intentionally concrete, but they are not claims that the
Roko-inspired systems already exist in IronClaw. They show how to extend the
current IronClaw HDC primitives without bypassing workspace storage, user/agent
scoping, caller-level tests, or the default-off rollout posture.

### A. Current HDC primitives to build on

The current crate already provides document encoding, dedup decisions, and
linear top-k scan. This is enough for first-pass fixtures and diagnostics:

```rust
use ironclaw_hdc::codebook::Codebook;
use ironclaw_hdc::dedup::{check_dedup, DedupConfig, DedupDecision, FingerprintCandidate};
use ironclaw_hdc::scan::top_k_scan;
use ironclaw_hdc::{encode_document, encode_text, DocumentEncodingInput, HdcVector};

fn fingerprint_doc(path: &str, tags: &[&str], content: &str) -> HdcVector {
    let mut codebook = Codebook::new();
    encode_document(
        DocumentEncodingInput {
            content,
            tags,
            path,
        },
        &mut codebook,
    )
}

fn dedup_against_existing(
    path: &str,
    content: &str,
    candidates: &[FingerprintCandidate<String>],
) -> DedupDecision<String> {
    let mut codebook = Codebook::new();
    let query = encode_document(
        DocumentEncodingInput {
            content,
            tags: &[],
            path,
        },
        &mut codebook,
    );
    let cfg = DedupConfig::default_config();
    check_dedup(query, candidates, &cfg)
}

fn diagnostic_hdc_top_k(query: &str, docs: &[(String, HdcVector)]) -> Vec<(String, f64)> {
    let mut codebook = Codebook::new();
    let query_hv = encode_text(query, &mut codebook);

    top_k_scan(query_hv, docs, 10)
        .into_iter()
        .map(|hit| (hit.id, hit.similarity))
        .collect()
}
```

Implementation rule: if this logic is promoted from diagnostics into a side
effecting path, the test must drive the real caller such as `MemoryWriteTool`,
workspace search, or the heartbeat runner loop. Helper-only tests are not
enough for a behavior change.

### B. Memory-write preflight shape

Write-time dedup should stay in the caller that owns user-visible behavior. The
workspace method supplies a reusable primitive; `MemoryWriteTool` decides
whether to warn, block, or continue.

```rust
use ironclaw_hdc::dedup::{DedupConfig, DedupDecision};

async fn preflight_memory_write(
    workspace: &crate::workspace::Workspace,
    path: &str,
    final_content: &str,
    force: bool,
) -> Result<Option<serde_json::Value>, crate::workspace::WorkspaceError> {
    if force {
        return Ok(None);
    }

    let cfg = DedupConfig::default_config();
    match workspace.check_dedup(path, final_content, &cfg).await? {
        DedupDecision::Unique => Ok(None),
        DedupDecision::Similar {
            id,
            path,
            similarity,
        } => Ok(Some(serde_json::json!({
            "decision": "similar",
            "existing_id": id,
            "existing_path": path,
            "similarity": similarity
        }))),
        DedupDecision::Duplicate {
            id,
            path,
            similarity,
        } => Ok(Some(serde_json::json!({
            "decision": "duplicate",
            "existing_id": id,
            "existing_path": path,
            "similarity": similarity,
            "retry": "Pass force=true to write anyway."
        }))),
    }
}
```

Required caller-level tests:

- default-off `memory_write` output contains no HDC fields,
- active dedup is only expected on configured non-append writes unless append
  preflight is explicitly added,
- warn mode writes and returns a `dedup.decision="similar"` payload,
- block mode returns `status="blocked"` before creating a new document,
- `force=true` bypasses block mode and stores a fingerprint only when workspace
  HDC shadow fingerprinting is enabled,
- protected-path and prompt-injection validation still run on the normal write
  path.

### C. Context assembly dedup before prompt budgeting

The Roko context assembler pattern becomes useful when IronClaw assembles a
prompt from durable memory, daily logs, files, and recent turn summaries. HDC
should not pick the final answer context by itself; it should remove redundant
chunks before token budgeting.

```rust
use ironclaw_hdc::codebook::Codebook;
use ironclaw_hdc::{encode_document, DocumentEncodingInput, HdcVector};

#[derive(Clone)]
enum ContextSource {
    UserAuthoredMemory,
    VerifiedRunbook,
    ProjectFile,
    DailyLog,
    RecentTurn,
}

#[derive(Clone)]
struct ContextCandidate {
    id: String,
    source: ContextSource,
    path: String,
    text: String,
    relevance: f64,
}

struct DroppedContext {
    dropped_id: String,
    kept_id: String,
    similarity: f64,
    reason: &'static str,
}

struct ContextDedupResult {
    selected: Vec<ContextCandidate>,
    dropped: Vec<DroppedContext>,
    prompt_tokens_saved: usize,
}

fn authority(source: &ContextSource) -> u8 {
    match source {
        ContextSource::UserAuthoredMemory => 5,
        ContextSource::VerifiedRunbook => 4,
        ContextSource::ProjectFile => 3,
        ContextSource::DailyLog => 2,
        ContextSource::RecentTurn => 1,
    }
}

fn rough_token_count(text: &str) -> usize {
    text.split_whitespace().count()
}

fn fingerprint_candidate(candidate: &ContextCandidate, codebook: &mut Codebook) -> HdcVector {
    encode_document(
        DocumentEncodingInput {
            content: &candidate.text,
            tags: &[],
            path: &candidate.path,
        },
        codebook,
    )
}

fn dedup_context_candidates(
    mut candidates: Vec<ContextCandidate>,
    threshold: f64,
) -> ContextDedupResult {
    candidates.sort_by(|a, b| {
        authority(&b.source)
            .cmp(&authority(&a.source))
            .then_with(|| b.relevance.total_cmp(&a.relevance))
            .then_with(|| b.text.len().cmp(&a.text.len()))
    });

    let mut codebook = Codebook::new();
    let mut selected: Vec<ContextCandidate> = Vec::new();
    let mut fingerprints: Vec<HdcVector> = Vec::new();
    let mut dropped = Vec::new();
    let mut prompt_tokens_saved = 0;

    for candidate in candidates {
        let fp = fingerprint_candidate(&candidate, &mut codebook);
        let duplicate_of = fingerprints
            .iter()
            .enumerate()
            .map(|(idx, existing)| (idx, fp.similarity(*existing)))
            .max_by(|a, b| a.1.total_cmp(&b.1));

        if let Some((idx, similarity)) = duplicate_of
            && similarity >= threshold
        {
            prompt_tokens_saved += rough_token_count(&candidate.text);
            dropped.push(DroppedContext {
                dropped_id: candidate.id,
                kept_id: selected[idx].id.clone(),
                similarity,
                reason: "hdc_near_duplicate_context",
            });
            continue;
        }

        fingerprints.push(fp);
        selected.push(candidate);
    }

    ContextDedupResult {
        selected,
        dropped,
        prompt_tokens_saved,
    }
}
```

Evaluation gates before enabling this in prompt composition:

- zero dropped chunks that are later cited by recorded QA traces,
- no answer-quality regression on Reborn QA fixtures,
- prompt-token savings reported per turn and per task family,
- manual review of all high-authority chunks dropped in the first canary.

### D. Durable memory admission with anti-knowledge

Roko's admission gate maps cleanly to an IronClaw follow-up: treat the HDC score
as a novelty signal, then combine it with confidence and source trust. Positive
memory and anti-knowledge need separate outcomes.

```rust
use ironclaw_hdc::dedup::{DedupConfig, DedupDecision};

struct MemoryAdmissionCandidate<'a> {
    path: &'a str,
    content: &'a str,
    confidence: f64,
    source_trust: f64,
}

struct AdmissionConfig {
    min_confidence: f64,
    min_source_trust: f64,
    min_novelty: f64,
    anti_match_threshold: f64,
    dedup: DedupConfig,
}

enum AdmissionDecision {
    Promote,
    Merge {
        existing_path: String,
        similarity: f64,
    },
    Defer {
        reason: &'static str,
    },
    Suppress {
        anti_memory_path: String,
        similarity: f64,
    },
}

async fn evaluate_memory_admission(
    workspace: &crate::workspace::Workspace,
    candidate: &MemoryAdmissionCandidate<'_>,
    cfg: &AdmissionConfig,
) -> Result<AdmissionDecision, crate::workspace::WorkspaceError> {
    if candidate.confidence < cfg.min_confidence {
        return Ok(AdmissionDecision::Defer {
            reason: "low_confidence",
        });
    }
    if candidate.source_trust < cfg.min_source_trust {
        return Ok(AdmissionDecision::Defer {
            reason: "low_source_trust",
        });
    }

    match workspace
        .check_dedup(candidate.path, candidate.content, &cfg.dedup)
        .await?
    {
        DedupDecision::Unique => Ok(AdmissionDecision::Promote),
        DedupDecision::Similar {
            path, similarity, ..
        }
        | DedupDecision::Duplicate {
            path, similarity, ..
        } => {
            if path.starts_with("anti-knowledge/") && similarity >= cfg.anti_match_threshold {
                return Ok(AdmissionDecision::Suppress {
                    anti_memory_path: path,
                    similarity,
                });
            }

            let novelty = 1.0 - similarity;
            if novelty < cfg.min_novelty {
                Ok(AdmissionDecision::Merge {
                    existing_path: path,
                    similarity,
                })
            } else {
                Ok(AdmissionDecision::Promote)
            }
        }
    }
}
```

Use case:

```text
Candidate: "Use SQL feature F in both Postgres and libSQL migrations."
Existing anti-knowledge: "SQL feature F failed libSQL parity gate."
HDC similarity: 0.91
Decision: Suppress; surface the anti-knowledge path before execution.
```

### E. Dream consolidation and playbook promotion

The dream-cycle pattern should be an offline consolidation job, not a live
agent-loop shortcut. HDC is useful for rejecting redundant distilled lessons and
for grouping similar task shapes before an operator or test gate promotes a
playbook.

```rust
use std::collections::BTreeMap;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
struct DreamClusterKey {
    task_type: String,
    outcome: String,
    failure_reason: Option<String>,
}

struct CompletedTurnSummary {
    thread_id: String,
    task_type: String,
    outcome: String,
    model: String,
    gates: Vec<String>,
    lesson: String,
    failure_reason: Option<String>,
}

struct PlaybookCandidate {
    title: String,
    evidence_threads: Vec<String>,
    required_gates: Vec<String>,
    lesson: String,
}

fn cluster_turns(turns: Vec<CompletedTurnSummary>) -> BTreeMap<DreamClusterKey, Vec<CompletedTurnSummary>> {
    let mut clusters: BTreeMap<DreamClusterKey, Vec<CompletedTurnSummary>> = BTreeMap::new();
    for turn in turns {
        let key = DreamClusterKey {
            task_type: turn.task_type.clone(),
            outcome: turn.outcome.clone(),
            failure_reason: turn.failure_reason.clone(),
        };
        clusters.entry(key).or_default().push(turn);
    }
    clusters
}

fn propose_playbook_candidates(
    clusters: BTreeMap<DreamClusterKey, Vec<CompletedTurnSummary>>,
    min_evidence: usize,
) -> Vec<PlaybookCandidate> {
    clusters
        .into_iter()
        .filter_map(|(key, turns)| {
            if key.outcome != "success" || turns.len() < min_evidence {
                return None;
            }

            let evidence_threads = turns.iter().map(|t| t.thread_id.clone()).collect();
            let required_gates = turns.iter().flat_map(|t| t.gates.clone()).collect();
            let lesson = turns
                .iter()
                .map(|t| t.lesson.as_str())
                .collect::<Vec<_>>()
                .join("\n");

            Some(PlaybookCandidate {
                title: format!("{} success playbook", key.task_type),
                evidence_threads,
                required_gates,
                lesson,
            })
        })
        .collect()
}
```

Promotion gate:

- candidate has at least `min_evidence` successful turns,
- referenced gates are reproducible, not only textual,
- HDC similarity against existing playbooks is below the redundancy threshold,
- user or maintainer approves promotion to durable memory.

### F. Code structural fingerprints

Code fingerprints belong in a later code-intelligence slice. Keep them parser
backed: HDC can suggest related symbols, but AST facts and tests decide
correctness.

```rust
use ironclaw_hdc::bundle::BundleAccumulator;
use ironclaw_hdc::codebook::Codebook;
use ironclaw_hdc::HdcVector;

struct SymbolFingerprintInput<'a> {
    kind: &'a str,
    name: &'a str,
    signature: &'a str,
    file_path: &'a str,
    surrounding_context: &'a str,
}

fn bind_text(role: &str, text: &str, codebook: &mut Codebook) -> HdcVector {
    let role_hv = codebook.get_or_create(role);
    let text_hv = codebook.get_or_create(text);
    role_hv.bind(text_hv)
}

fn fingerprint_symbol(input: &SymbolFingerprintInput<'_>, codebook: &mut Codebook) -> HdcVector {
    let mut acc = BundleAccumulator::new();
    acc.add(&bind_text("role:symbol_kind", input.kind, codebook));
    acc.add(&bind_text("role:symbol_name", input.name, codebook));
    acc.add(&bind_text("role:signature", input.signature, codebook));
    acc.add(&bind_text("role:file_path", input.file_path, codebook));
    acc.add(&bind_text("role:context", input.surrounding_context, codebook));
    acc.finalize()
}
```

Example query:

```text
Anchor symbol: libSQL workspace fingerprint update
Candidate hits:
  PostgreSQL workspace fingerprint update
  WorkspaceStore trait method mapping
  migration row-mapping tests
Required follow-up: parser confirmation plus caller-level DB parity tests.
```

### G. Weighted HDC search diagnostics

Weighted HDC ranking should remain diagnostic until raw HDC quality is proven.
Expose raw similarity and weighted score side by side so operators can see
whether a boost changes ranking for the right reason.

```rust
use ironclaw_hdc::HdcVector;

struct WeightedHdcCandidate<Id> {
    id: Id,
    fingerprint: HdcVector,
    weight: f64,
}

struct WeightedHdcHit<Id> {
    id: Id,
    raw_similarity: f64,
    weight: f64,
    score: f64,
}

fn weighted_hdc_top_k<Id: Clone>(
    query: HdcVector,
    candidates: &[WeightedHdcCandidate<Id>],
    top_k: usize,
) -> Vec<WeightedHdcHit<Id>> {
    let mut hits: Vec<_> = candidates
        .iter()
        .filter(|candidate| candidate.weight > 0.0)
        .map(|candidate| {
            let raw_similarity = query.similarity(candidate.fingerprint);
            let weight = candidate.weight.clamp(0.0, 2.0);
            WeightedHdcHit {
                id: candidate.id.clone(),
                raw_similarity,
                weight,
                score: raw_similarity * weight,
            }
        })
        .collect();

    hits.sort_by(|a, b| b.score.total_cmp(&a.score));
    hits.truncate(top_k);
    hits
}
```

Candidate weights:

- `1.25` for verified runbooks or user-pinned memory,
- `1.10` for frequently cited documents that led to successful outcomes,
- `0.80` for transient daily logs,
- `0.0` for quarantined or explicitly suppressed documents.

### H. Adaptive heartbeat cadence

HDC novelty can eventually tune heartbeat cadence, but the current safe posture
is observe-only classification. Any cadence change needs false-suppression
fixtures and user-visible audit logs.

```rust
use std::time::Duration;

enum NoveltyClass {
    Novel,
    Uncertain,
    Repeated,
}

enum FindingSeverity {
    Low,
    Medium,
    High,
}

fn next_heartbeat_interval(
    base: Duration,
    class: NoveltyClass,
    severity: FindingSeverity,
) -> Duration {
    match (class, severity) {
        (NoveltyClass::Novel, FindingSeverity::High) => base / 2,
        (NoveltyClass::Novel, _) => base,
        (NoveltyClass::Uncertain, _) => base,
        (NoveltyClass::Repeated, FindingSeverity::High) => base,
        (NoveltyClass::Repeated, _) => base * 2,
    }
}
```

Rollout rule: first record the proposed interval beside the actual interval.
Only switch to live cadence after fixture and canary data show no novel finding
would have been delayed beyond the configured safety bound.

### I. Bounded diagnostic command shape

If HDC becomes user-visible outside `memory_write`, prefer a bounded workspace
diagnostic over a generic vector mutation API:

```bash
ironclaw memory hdc query \
  --user default \
  --text "rollback failed migration after schema drift" \
  --limit 10 \
  --show-scores
```

Example output:

```json
{
  "query": "rollback failed migration after schema drift",
  "limit": 10,
  "results": [
    {
      "path": "runbooks/db-rollback.md",
      "raw_similarity": 0.87,
      "weighted_score": 0.96,
      "weight": 1.1,
      "reason": "verified_runbook"
    },
    {
      "path": "daily/2026-06-18.md",
      "raw_similarity": 0.79,
      "weighted_score": 0.63,
      "weight": 0.8,
      "reason": "transient_daily_log"
    }
  ]
}
```

Bounds:

- no cross-user scan,
- caller must provide user/agent scope,
- max `--limit` and max candidate count are enforced,
- output includes raw and weighted scores,
- no raw vector insert/remove command is exposed to normal users.

### 13. K-medoids HDC clustering

PAM (Partitioning Around Medoids) clustering using `1.0 - similarity` as the
distance metric. Seeding is farthest-first from a fixed starting point, so
the result is deterministic with no RNG dependency. Groups memory entries into
behavioral or topic clusters for audit, offline consolidation, and pattern
discovery.

IronClaw adaptation:

- New `cluster` module in `ironclaw_hdc` crate.
- Used by the memory audit CLI subcommand to produce a visual topic map of
  stored documents.
- Used as the clustering input stage in dream consolidation before distillation.
- Used in episode pattern discovery to group completed turns by structural
  similarity before a playbook is proposed.

Implementation sketch:

```rust
use ironclaw_hdc::HdcVector;

pub struct KMedoidsConfig {
    pub k: usize,
    pub max_iterations: usize,
}

pub struct HdcCluster {
    pub medoid_index: usize,
    pub medoid: HdcVector,
    pub members: Vec<usize>,
    pub intra_similarity: f64,
}

/// Farthest-first seeding: start from index 0, then repeatedly pick the
/// entry most dissimilar to all currently chosen medoids.
fn farthest_first_seeds(vectors: &[HdcVector], k: usize) -> Vec<usize> {
    let mut seeds = vec![0usize];
    while seeds.len() < k {
        let next = (0..vectors.len())
            .filter(|i| !seeds.contains(i))
            .max_by(|&a, &b| {
                let da = seeds
                    .iter()
                    .map(|&s| 1.0 - vectors[a].similarity(vectors[s]))
                    .fold(f64::INFINITY, f64::min);
                let db = seeds
                    .iter()
                    .map(|&s| 1.0 - vectors[b].similarity(vectors[s]))
                    .fold(f64::INFINITY, f64::min);
                da.total_cmp(&db)
            })
            .expect("at least one non-seed entry");
        seeds.push(next);
    }
    seeds
}

pub fn k_medoids(vectors: &[HdcVector], cfg: &KMedoidsConfig) -> Vec<HdcCluster> {
    let mut medoid_indices = farthest_first_seeds(vectors, cfg.k);

    for _ in 0..cfg.max_iterations {
        // Assignment step
        let assignments: Vec<usize> = vectors
            .iter()
            .map(|v| {
                medoid_indices
                    .iter()
                    .copied()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| {
                        v.similarity(vectors[*a]).total_cmp(&v.similarity(vectors[*b]))
                    })
                    .map(|(cluster_idx, _)| cluster_idx)
                    .unwrap_or(0)
            })
            .collect();

        // Update step: for each cluster, find the member with the minimum
        // total distance to all other members.
        let mut changed = false;
        for cluster_idx in 0..cfg.k {
            let members: Vec<usize> = assignments
                .iter()
                .enumerate()
                .filter(|(_, &c)| c == cluster_idx)
                .map(|(i, _)| i)
                .collect();

            if members.is_empty() {
                continue;
            }

            let best = members
                .iter()
                .copied()
                .min_by(|&a, &b| {
                    let da: f64 = members
                        .iter()
                        .map(|&m| 1.0 - vectors[a].similarity(vectors[m]))
                        .sum();
                    let db: f64 = members
                        .iter()
                        .map(|&m| 1.0 - vectors[b].similarity(vectors[m]))
                        .sum();
                    da.total_cmp(&db)
                })
                .unwrap_or(members[0]);

            if best != medoid_indices[cluster_idx] {
                medoid_indices[cluster_idx] = best;
                changed = true;
            }
        }

        if !changed {
            break;
        }
    }

    // Build final clusters
    let assignments: Vec<usize> = vectors
        .iter()
        .map(|v| {
            medoid_indices
                .iter()
                .copied()
                .enumerate()
                .max_by(|(_, a), (_, b)| {
                    v.similarity(vectors[*a]).total_cmp(&v.similarity(vectors[*b]))
                })
                .map(|(cluster_idx, _)| cluster_idx)
                .unwrap_or(0)
        })
        .collect();

    (0..cfg.k)
        .map(|cluster_idx| {
            let members: Vec<usize> = assignments
                .iter()
                .enumerate()
                .filter(|(_, &c)| c == cluster_idx)
                .map(|(i, _)| i)
                .collect();
            let medoid_index = medoid_indices[cluster_idx];
            let intra_similarity = if members.len() < 2 {
                1.0
            } else {
                let total: f64 = members
                    .iter()
                    .map(|&m| vectors[medoid_index].similarity(vectors[m]))
                    .sum();
                total / members.len() as f64
            };
            HdcCluster {
                medoid_index,
                medoid: vectors[medoid_index],
                members,
                intra_similarity,
            }
        })
        .collect()
}
```

Fixture shape for clustering 30 memory entries into 3 topic clusters:

```jsonl
{"entry_index": 0, "path": "memory/deployments/rollback-postgres.md", "expected_cluster": "database_ops"}
{"entry_index": 1, "path": "memory/deployments/rollback-libsql.md", "expected_cluster": "database_ops"}
...
{"entry_index": 10, "path": "memory/auth/oauth-setup.md", "expected_cluster": "auth_flows"}
...
{"entry_index": 20, "path": "memory/testing/integration-harness.md", "expected_cluster": "testing_patterns"}
...
```

```json
{
  "scenario": "memory_topic_clusters",
  "input_count": 30,
  "k": 3,
  "config": {"max_iterations": 20},
  "expected_clusters": [
    {"label": "database_ops", "min_members": 8, "min_intra_similarity": 0.70},
    {"label": "auth_flows", "min_members": 8, "min_intra_similarity": 0.70},
    {"label": "testing_patterns", "min_members": 8, "min_intra_similarity": 0.70}
  ],
  "stability_check": "re-run with same input must produce identical cluster assignment"
}
```

Caller-level test requirement: drive through the memory audit CLI subcommand
or a `cluster_documents` workspace method, not through the `k_medoids` helper
alone. Verify that cluster labels survive a round-trip through the audit report
format.

### 14. Structured role-filler HDC encoding

`bind(role_vec, filler_vec)` encodes structured knowledge beyond flat text by
pairing a role token with its filler. A composite knowledge frame is a bundle
of bound role-filler pairs. Unbinding recovers any filler given the role:
`unbind(composite, role) = composite XOR role`.

Directional permutation shifts add asymmetric positional semantics. For a
causal link:

```
role("cause").permute(1) XOR text("cause_text")
role("effect").permute(2) XOR text("effect_text")
```

Permuting by different amounts prevents roles and their reverses from
collapsing into identical vectors. The composite is the bitwise majority bundle
of these two bound pairs.

Unbinding recovers the filler:

```
composite XOR role("cause").permute(1) ≈ text("cause_text")
```

IronClaw adaptation: new `encode_structured` function in
`ironclaw_hdc::encoder`. Used for encoding causal memory entries, relationship
facts, and multi-slot knowledge structures that flat text cannot preserve.

Implementation sketch:

```rust
use ironclaw_hdc::bundle::BundleAccumulator;
use ironclaw_hdc::codebook::Codebook;
use ironclaw_hdc::HdcVector;

pub struct RoleFiller<'a> {
    pub role: &'a str,
    /// Number of left-rotation positions applied to the role vector before
    /// binding. Use distinct values for each role to preserve directionality.
    pub permute_shift: usize,
    pub filler_text: &'a str,
}

pub struct StructuredEncoding {
    pub composite: HdcVector,
    /// Encoded role vectors (after permutation) kept for unbinding.
    pub role_vectors: Vec<(String, HdcVector)>,
}

pub fn encode_structured(slots: &[RoleFiller<'_>], codebook: &mut Codebook) -> StructuredEncoding {
    let mut acc = BundleAccumulator::new();
    let mut role_vectors = Vec::with_capacity(slots.len());

    for slot in slots {
        let role_hv = codebook.get_or_create(slot.role).permute(slot.permute_shift);
        let filler_hv = codebook.get_or_create(slot.filler_text);
        acc.add(&role_hv.bind(filler_hv));
        role_vectors.push((slot.role.to_owned(), role_hv));
    }

    StructuredEncoding {
        composite: acc.finalize(),
        role_vectors,
    }
}

/// Recover a filler approximation from a composite by XOR-unbinding a role.
pub fn unbind_filler(composite: HdcVector, role_hv: HdcVector) -> HdcVector {
    composite.bind(role_hv) // XOR is its own inverse
}
```

CausalLink encoding example:

```rust
let encoding = encode_structured(
    &[
        RoleFiller { role: "cause", permute_shift: 1, filler_text: "schema drift on libSQL" },
        RoleFiller { role: "effect", permute_shift: 2, filler_text: "migration rollback required" },
    ],
    &mut codebook,
);

// Query: recover the effect given the composite and the cause role vector
let recovered_effect = unbind_filler(
    encoding.composite,
    encoding.role_vectors.iter().find(|(r, _)| r == "cause").unwrap().1,
);
// recovered_effect should have high similarity to codebook.get_or_create("migration rollback required")
```

Fixture shape for 5 cause-effect pairs with directional encoding:

```jsonl
{"id": "causal_1", "cause": "schema drift on libSQL", "effect": "migration rollback required", "expected_unbind_similarity_gte": 0.60}
{"id": "causal_2", "cause": "OAuth token expired", "effect": "re-auth gate triggered", "expected_unbind_similarity_gte": 0.60}
{"id": "causal_3", "cause": "rate limit exceeded on API", "effect": "exponential backoff applied", "expected_unbind_similarity_gte": 0.60}
{"id": "causal_4", "cause": "sandbox container OOM", "effect": "job marked failed and evicted", "expected_unbind_similarity_gte": 0.60}
{"id": "causal_5", "cause": "heartbeat finding repeats 5 cycles", "effect": "novelty class set to repeated", "expected_unbind_similarity_gte": 0.60}
```

Caller-level test requirement: drive through a workspace write path or a
dedicated `memory_write_structured` tool call. Verify that the composite can
be stored in `memory_documents.metadata` and that unbinding from a stored
fingerprint returns a vector similar to the original filler.

### 15. Episode fingerprinting per agent turn

After each completed agent turn, compute an HDC fingerprint from the
prompt input and the observed outcome. Store the fingerprint in an episode
log for later retrieval. This enables template-suggestion queries such as
"find past episodes similar to the current turn input" and pattern detection
across many turns.

Template suggestion: find past episodes with HDC similarity >= 0.7 within a
30-day window. Surface the top matching episodes as context before the agent
selects a plan.

IronClaw adaptation:

- Post-turn hook in the main agent loop after a turn commits, or a
  `TurnCommittedObserver` that fires when turn state transitions to
  `Completed`.
- For subagent results: integration seam at
  `crates/ironclaw_reborn/src/subagent/completion_observer.rs`. After a
  subagent resolves, fingerprint its prompt context and outcome summary and
  write to the episode log.
- Episode fingerprints are advisory metadata, never primary memory. They do
  not replace workspace documents.

Episode log entry shape:

```rust
use ironclaw_hdc::HdcVector;
use std::time::SystemTime;

pub struct EpisodeFingerprint {
    pub turn_id: String,
    pub agent_id: String,
    pub user_id: String,
    pub prompt_summary: String,
    pub outcome_summary: String,
    pub fingerprint: HdcVector,
    pub completed_at: SystemTime,
}

pub fn fingerprint_episode(
    prompt_summary: &str,
    outcome_summary: &str,
    codebook: &mut ironclaw_hdc::codebook::Codebook,
) -> HdcVector {
    use ironclaw_hdc::bundle::BundleAccumulator;
    let prompt_hv = codebook.get_or_create(prompt_summary);
    let outcome_hv = codebook.get_or_create(outcome_summary);
    let role_prompt = codebook.get_or_create("role:episode_prompt");
    let role_outcome = codebook.get_or_create("role:episode_outcome");
    let mut acc = BundleAccumulator::new();
    acc.add(&role_prompt.bind(prompt_hv));
    acc.add(&role_outcome.bind(outcome_hv));
    acc.finalize()
}
```

Retrieval example:

```rust
fn find_similar_episodes(
    current_fingerprint: HdcVector,
    episode_log: &[EpisodeFingerprint],
    min_similarity: f64,
    window_days: u64,
) -> Vec<&EpisodeFingerprint> {
    let cutoff = SystemTime::now()
        .checked_sub(std::time::Duration::from_secs(window_days * 86_400))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let mut hits: Vec<_> = episode_log
        .iter()
        .filter(|ep| ep.completed_at >= cutoff)
        .filter_map(|ep| {
            let sim = current_fingerprint.similarity(ep.fingerprint);
            if sim >= min_similarity {
                Some((sim, ep))
            } else {
                None
            }
        })
        .collect();

    hits.sort_by(|a, b| b.0.total_cmp(&a.0));
    hits.into_iter().map(|(_, ep)| ep).collect()
}
```

Fixture shape for 10 completed turns with prompt and outcome fingerprints:

```jsonl
{"turn_id": "t-001", "prompt_summary": "migrate postgres schema add column", "outcome_summary": "migration applied successfully on both backends", "expected_similar_to": ["t-003", "t-007"], "similarity_threshold": 0.70, "window_days": 30}
{"turn_id": "t-002", "prompt_summary": "fix oauth token refresh for gmail", "outcome_summary": "token refreshed and stored in secrets", "expected_similar_to": ["t-009"], "similarity_threshold": 0.70, "window_days": 30}
...
{"turn_id": "t-010", "prompt_summary": "migrate postgres schema add column nullable", "outcome_summary": "migration applied with nullable constraint on both backends", "expected_similar_to": ["t-001", "t-003"], "similarity_threshold": 0.70, "window_days": 30}
```

Caller-level test requirement: drive through the agent loop's post-turn
commit path or the subagent `completion_observer` integration seam, not
through the `fingerprint_episode` helper alone. The test must verify that
the episode fingerprint is written to persistent storage and that a subsequent
similarity query returns the expected matching episode IDs within the window.

## Fixture and Benchmark Additions

Use the existing `tests/fixtures/hdc_memory/` structure and add a
`roko_inspired/` subfolder for follow-up experiments. The goal is not to
replicate Roko; it is to preserve the portable behavior as IronClaw tests.

| Fixture file | Purpose | Expected assertion |
|---|---|---|
| `roko_inspired/context_dedup/daily_log_vs_runbook.jsonl` | Daily logs and a durable runbook repeat the same deployment rule. | Keep the runbook, drop daily-log duplicates, report saved tokens. |
| `roko_inspired/admission/anti_knowledge_migration.jsonl` | Candidate advice resembles a known failed libSQL migration approach. | Return suppress/warn outcome, not positive memory promotion. |
| `roko_inspired/dream/db_parity_playbook.jsonl` | Several successful DB parity fixes share task shape and gates. | Produce one playbook candidate with evidence count and gate list. |
| `roko_inspired/heartbeat/adaptive_cadence.jsonl` | Stable repeated warning later changes materially. | Classify repeat first, classify changed warning as novel. |
| `roko_inspired/code/libsql_postgres_parity_symbols.jsonl` | libSQL and PostgreSQL workspace methods are structurally related. | HDC suggests candidates; parser/test evidence remains required. |
| `roko_inspired/search/weighted_runbook_vs_daily_log.jsonl` | A daily log is more similar but a runbook is more authoritative. | Diagnostic output shows both raw and weighted rank. |
| `roko_inspired/clustering/memory_topic_clusters.jsonl` | 30 memory entries spanning three topic areas (database ops, auth flows, testing patterns). | PAM clustering into 3 clusters with deterministic farthest-first seeding; each cluster has >= 8 members and intra-similarity >= 0.70; re-running produces identical assignment. |
| `roko_inspired/episodes/prompt_outcome_fingerprints.jsonl` | 10 completed turns with prompt and outcome summaries. | Episode fingerprints written to persistent log; similarity query with threshold 0.70 and 30-day window returns expected matching turn IDs. |
| `roko_inspired/structured/causal_link_binding.jsonl` | 5 cause-effect pairs encoded with directional permutation shifts. | Unbinding the composite with the cause role vector recovers a vector with similarity >= 0.60 to the original effect filler; cause and effect vectors are not interchangeable (permutation asymmetry). |

Benchmark plan:

| Benchmark | What it quantifies | Promotion gate |
|---|---|---|
| Context dedup over 100 candidates | Prompt assembly overhead and saved tokens | Report p50/p95; no QA fixture regression. |
| Admission novelty scan over durable memory | Write/admission overhead | No false suppressions on unique/valid fixtures. |
| Dream redundancy screen | Offline consolidation cost per completed turn batch | Offline-only; cost must not affect live agent loop. |
| Code structural top-k over symbol fingerprints | Navigation latency and candidate quality | No correctness claim without parser/test confirmation. |
| Weighted search diagnostics | Ranking delta from raw HDC to weighted HDC | Improve labeled runbook ranking without hiding raw score. |
| Heartbeat novelty cadence simulation | Delay introduced by repeated classification | Zero delayed high-severity novel findings in fixtures. |

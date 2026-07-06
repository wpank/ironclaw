# HDC Innovation Examples and Benchmarks

This companion note turns the advanced HDC integration tracks into concrete
examples, fixture shapes, metrics, and rollout gates. It is scoped to IronClaw.
The goal is to make each idea testable before it affects live memory, prompts,
heartbeat notifications, subagents, routines, routing, or search.

Read this with:

- `tmp/hdc-implementation-plan.md` for the current HDC PR plan.
- `tmp/hdc-ironclaw-innovation-integrations.md` for the architecture and
  integration tracks.
- `tmp/hdc-roko-cognitive-usecases.md` for the larger source-map and design
  sketches.

## Benchmark Philosophy

HDC integrations should advance through five stages:

1. **Fixture-only helper tests:** pure encoding, similarity, clustering, and
   threshold behavior.
2. **Caller-level shadow tests:** drive the production caller while returning
   current behavior.
3. **Shadow telemetry:** record raw HDC scores, proposed decisions, and baseline
   decisions.
4. **Advisory canary:** show recommendations or warnings without automatic
   side effects.
5. **Live behavior:** enable only after scope, safety, quality, latency, docs,
   and rollback gates pass.

Hard rule: speed does not prove quality. Any feature that changes a prompt,
write, notification, subagent handoff, routine output, tool choice, model route,
or search ranking needs caller-level tests and quality fixtures.

## Proposed Config Surface

These names are intentionally explicit. They should live in the owning modules
and remain default-off.

| Setting | Default | Owner | Purpose |
|---|---:|---|---|
| `IRONCLAW_HDC_EPISODE_PATTERNS` | `false` | agent loop or host runtime | Record episode fingerprints after completed turns. |
| `IRONCLAW_HDC_EPISODE_CONTEXT_MODE` | `off` | memory context host | `off`, `shadow`, `advisory`, `live`. |
| `IRONCLAW_HDC_ANTI_KNOWLEDGE_MODE` | `off` | memory/workspace/native provider | `off`, `shadow`, `warn`, `block`. |
| `IRONCLAW_HDC_CONTEXT_DEDUP_MODE` | `off` | `ProductionMemoryPromptContextService` | `off`, `shadow`, `live`. |
| `IRONCLAW_HDC_RESONANCE_MODE` | `off` | CLI/tool diagnostic | `off`, `diagnostic`. |
| `IRONCLAW_HDC_REPLAY_MODE` | `off` | offline job | `off`, `shadow`, `stage_candidates`. |
| `IRONCLAW_HDC_COORDINATION_MODE` | `off` | subagent/routine owner | `off`, `shadow`, `advisory`. |
| `IRONCLAW_HDC_NATIVE_INDEX` | `false` | `ironclaw_memory_native` | Build provider-owned derived HDC index. |
| `IRONCLAW_HDC_MAX_SCAN` | `10000` | each caller | Bound flat scans before requiring an index/cache. |
| `IRONCLAW_HDC_LOG_RAW_VECTORS` | `false` | diagnostics only | Normally disabled; raw vectors should not be logged. |

Avoid a global "smart HDC" toggle. Each behavior has different risk and should
roll out separately.

## Telemetry Event Shape

Use one small event schema across experiments so shadow data can be compared.

```json
{
  "event": "hdc_decision",
  "feature": "context_dedup",
  "mode": "shadow",
  "scope_hash": "tenant:user:project hash, not raw ids",
  "candidate_count": 12,
  "selected_count": 9,
  "max_similarity": 0.88,
  "threshold": 0.85,
  "baseline_decision": "keep_all",
  "hdc_decision": "drop_duplicate",
  "latency_ms": 1.7,
  "saved_tokens": 420,
  "evidence_refs": ["fixture:advanced/context/runbook-vs-daily-log"],
  "safe_summary": "Dropped daily-log deploy reminder because verified runbook matched."
}
```

Rules:

- Hash or omit sensitive scope values.
- Do not include raw document bodies, raw prompts, secrets, or full transcripts.
- Do not include raw HDC vectors unless an explicit local diagnostic flag is
  enabled.
- Keep enough fields to compare baseline behavior against proposed HDC behavior.

## Example 1: DB Parity Episode Recall

### Scenario

The user asks IronClaw to add HDC fingerprint persistence to a new memory path.
The task resembles prior work where the correct plan was:

1. add a shared DB trait method,
2. implement PostgreSQL and libSQL,
3. test through the caller,
4. avoid helper-only coverage.

### Current Behavior

Memory search may find a few notes by text. The agent can still miss the
structural lesson and write only helper tests.

### HDC Behavior

Before planning, the agent encodes the task and retrieves similar successful
episodes plus known failed episodes:

```text
similar successes:
  0.84 db parity fingerprint update: trait first, both backends, caller tests
  0.79 memory write dedup: MemoryWriteTool coverage caught output-shape issue

similar failures:
  0.81 helper-only test regression: side effect lived in caller
```

The prompt context receives only bounded safe summaries:

```text
Untrusted memory content:
Prior similar work required DB trait + both backend implementations + caller-level tests.
```

### Fixture Row

```json
{
  "id": "episode-db-parity-001",
  "scope": {"tenant": "t1", "user": "u1", "project": "ironclaw"},
  "kind": "episode",
  "task_type": "db_parity",
  "task_summary": "Add HDC fingerprint storage to memory document backend.",
  "tools": ["apply_patch", "cargo_test"],
  "gates": ["libsql_integration", "caller_memory_tool"],
  "outcome": "succeeded",
  "expected_cluster_id": "db-parity-shared-trait-first",
  "expected_retrieval_for": ["new-db-backed-memory-feature"]
}
```

### Metrics

| Metric | Gate |
|---|---|
| Episode precision@5 | `>= 0.80` before advisory prompt inclusion |
| Cluster purity | `>= 0.85` on fixture clusters |
| False cross-cluster merge rate | `<= 2%` |
| Prompt regression on recorded traces | `0` known regressions |
| p95 encode + scoped top-k query | `< 5 ms` for 10K records |

## Example 2: Anti-Knowledge From Gate Failures

### Scenario

A proposed implementation says: "Use the same migration SQL for PostgreSQL and
libSQL." Prior gate failures show that this exact pattern broke libSQL.

### HDC Behavior

The candidate advice is fingerprinted and compared against scoped
anti-knowledge. A high match surfaces a warning:

```text
Warning: similar approach previously failed libSQL migration tests.
Evidence: migration gate failure, caller-level storage contract.
Suggested action: split backend-specific SQL and test both backends.
```

### Fixture Row

```json
{
  "id": "anti-libsql-postgres-migration-001",
  "scope": {"tenant": "t1", "user": "u1", "project": "ironclaw"},
  "kind": "anti_knowledge",
  "warning": "Do not reuse PostgreSQL-only migration SQL for libSQL.",
  "refutes": "Use one SQL migration body for both PostgreSQL and libSQL.",
  "evidence_refs": ["test:workspace_hdc_dedup", "gate:libsql_migration"],
  "candidate": "Use the same ALTER TABLE statement for both backends.",
  "expected_action": "warn",
  "max_false_suppression": 0
}
```

### Promotion Gate

Anti-knowledge can warn after one strong receipt, but should not block until:

- at least two independent receipts support the warning, or one receipt is a
  high-confidence security/data-loss failure,
- a positive newer success does not refute the warning,
- the candidate and warning share tenant/user/project scope,
- warning-mode fixtures have zero false suppressions on valid plans.

## Example 3: Context Dedup Without Dropping Evidence

### Scenario

Prompt assembly returns:

- a verified runbook explaining the DB parity rule,
- a daily log repeating the same rule,
- a recent turn summary repeating the same rule,
- an anti-knowledge warning about helper-only tests.

### HDC Behavior

In shadow mode, HDC proposes dropping the repeated daily log and turn summary,
keeping the verified runbook and the warning.

### Fixture Row

```json
{
  "id": "context-runbook-vs-daily-log-001",
  "query": "add db-backed memory hdc pattern storage",
  "candidates": [
    {
      "id": "verified-runbook-db-parity",
      "authority": "verified_runbook",
      "content": "Add the DB trait method first, then implement both backends.",
      "expected_action": "keep"
    },
    {
      "id": "daily-log-db-parity",
      "authority": "daily_log",
      "content": "Remember to update the DB trait before both backends.",
      "expected_action": "drop"
    },
    {
      "id": "anti-helper-only-tests",
      "authority": "anti_knowledge",
      "content": "Helper-only tests missed the memory_write side effect.",
      "expected_action": "keep"
    }
  ],
  "must_keep_ids": ["verified-runbook-db-parity", "anti-helper-only-tests"],
  "min_saved_tokens": 80
}
```

### Metrics

| Metric | Gate |
|---|---|
| Token savings | `>= 15%` on duplicate-heavy fixtures before live mode |
| Dropped required citation count | `0` |
| Reborn QA regression | `0` |
| p95 prompt assembly overhead | `< 2 ms` or `< 10%` over baseline |
| Stable ordering | deterministic for identical backend results |

## Example 4: Cross-Domain Resonance For Route Tests

### Scenario

A webhook auth route fix keeps failing. A different subsystem previously had a
rate-limit route bug where the fix was adding a route-level fixture instead of
testing only the helper.

### HDC Behavior

The resonance command excludes the current domain and returns a structurally
similar successful episode:

```text
match: rate_limit_routes
similarity: 0.548
suggestion: add caller-level route fixture that drives headers/body through the router
risk: medium, because auth has stricter security constraints
```

### Fixture Row

```json
{
  "id": "resonance-auth-to-rate-limit-001",
  "query_domain": "gateway_auth",
  "target_domain": "rate_limit_routes",
  "query": "webhook callback rejects valid signed request at route boundary",
  "expected_suggestion": "caller_level_route_fixture",
  "false_analogy_controls": [
    "rate-limit config docs only",
    "unrelated OAuth token refresh"
  ],
  "expected_precision_at_5_min": 0.7
}
```

### Guardrails

- Do not auto-apply a cross-domain suggestion.
- Show transfer risk and raw similarity.
- Exclude same-domain hits when the command is in resonance mode.
- Require normal security review for auth, approvals, secrets, network, and
  sandbox paths.

## Example 5: Heartbeat Changed Metric

### Scenario

Heartbeat repeatedly reports disk usage at 85%. Later it reports the same path
at 94%.

### HDC Behavior

The structure is similar, but the metric changed enough that the finding should
remain visible:

```text
cycle 1: novel
cycle 2: repeated
cycle 3: repeated
cycle 8: novel_detail_changed
```

### Fixture Row

```json
{
  "id": "heartbeat-repeated-changed-metric-001",
  "sequence": [
    {"cycle": 1, "text": "Disk /data is 85% full.", "expected": "novel"},
    {"cycle": 2, "text": "Disk /data remains 85% full.", "expected": "repeated"},
    {"cycle": 8, "text": "Disk /data is now 94% full.", "expected": "novel"}
  ],
  "must_not_suppress_cycles": [1, 8]
}
```

### Metrics

| Metric | Gate |
|---|---|
| High-severity novel suppressions | `0` |
| Changed-metric suppressions | `0` |
| Repeated-notification reduction | report only until canary |
| Multi-user leakage | `0` |
| p95 novelty overhead | `< 1 ms` per heartbeat message |

## Example 6: Subagent Coordination Field

### Scenario

Three subagents inspect the same HDC rollout problem. Two report progress on the
same workspace file set. One reports an unresolved PostgreSQL test gap.

### HDC Behavior

The parent sees a bounded coordination field:

```text
progress cluster:
  - child A audited libSQL fingerprint storage
  - child B audited the same area, duplicate of A at 0.89

threat:
  - child C found missing PostgreSQL storage contract

recommendation:
  - do not spawn another storage-audit child
  - spawn/ask for PostgreSQL contract test follow-up if needed
```

### Fixture Row

```json
{
  "id": "coordination-duplicate-subagent-progress-001",
  "parent_run": "run-parent-1",
  "children": [
    {
      "id": "child-a",
      "trace_kind": "progress",
      "summary": "Audited libSQL HDC fingerprint update and listing."
    },
    {
      "id": "child-b",
      "trace_kind": "progress",
      "summary": "Reviewed libSQL HDC fingerprint storage paths."
    },
    {
      "id": "child-c",
      "trace_kind": "threat",
      "summary": "PostgreSQL HDC storage contract coverage is missing."
    }
  ],
  "expected_duplicate_clusters": [["child-a", "child-b"]],
  "must_surface": ["child-c"]
}
```

### Metrics

| Metric | Gate |
|---|---|
| Duplicate child-result reduction | report in shadow |
| Lost terminal child result | `0` |
| Hidden disagreement | `0` |
| Parent answer QA regression | `0` |
| Scope/capability widening | `0` |

## Example 7: Native Memory Derived Index

### Scenario

Reborn/native memory writes a document through `NativeMemoryService`. The native
provider chunks and indexes it. HDC should become a provider-owned derived
index, not a provider-neutral dependency.

### HDC Behavior

```text
NativeMemoryService::write
  -> native repository writes document
  -> native indexer chunks document
  -> optional HDC derived index writes fingerprints
  -> retrieve_context still returns bounded safe summaries
```

### Caller Test

```text
NativeMemoryService::write("db parity rule")
NativeMemoryService::search("backend migration parity")
NativeMemoryService::retrieve_context(...)

assert:
  - HDC-derived score exists only when native HDC is enabled
  - prompt snippets remain wrapped as untrusted memory content
  - no raw HDC vector crosses the provider-neutral prompt boundary
  - disabling native HDC leaves normal search behavior unchanged
```

## Example 8: Counterfactual Prompt Budget Lab

### Scenario

A previous task succeeded but used an expensive prompt context. Offline replay
tests whether the same result holds with context dedup enabled or a smaller
memory snippet budget.

### HDC Behavior

Replay variants:

| Variant | Change | Expected gate |
|---|---|---|
| Baseline | original context | existing recorded success |
| Dedup | drop HDC-similar snippets | same final answer assertions |
| Small budget | half memory snippets | no missing required citation |
| Different model tier | standard model instead of expensive model | same tests pass |

Promotion:

- If dedup passes, stage context-dedup playbook evidence.
- If small budget fails because a citation was missing, stage anti-knowledge
  against dropping that authority class.
- If different model tier passes repeatedly, stage observe-only routing evidence.

### Metrics

| Metric | Gate |
|---|---|
| Replay determinism | recorded inputs/mocks reproduce baseline |
| Live secret use | `0` |
| Variant pass rate | report by task class |
| Cost delta at equal quality | report before routing use |
| Failed variant promotion | anti-knowledge candidate only, not positive advice |

## Example 9: K-Medoids Memory Audit Clustering

### Scenario

A user has 200 memory entries accumulated over 6 months. Many are near-duplicates
or minor variants. The memory audit clusters them and identifies redundancy.

### HDC Behavior

```text
$ ironclaw memory hdc audit --user default --k 10
Cluster 1 (28 entries, medoid: decisions/api-auth.md, intra-sim: 0.82):
  - 15 daily-log fragments about auth decisions
  - 8 notes/auth-*.md variants
  - 5 near-duplicate runbook sections
  Recommendation: Consolidate to 3 canonical entries

Cluster 2 (12 entries, medoid: runbooks/deploy.md, intra-sim: 0.76):
  ...
```

The audit command drives `memory hdc audit` which reads the workspace, builds
HDC fingerprints, and runs k-medoids with k=10. It is read-only in advisory
mode. No memory entries are deleted unless the user confirms.

### Fixture Row

```json
{
  "id": "clustering-memory-audit-001",
  "scope": {"tenant": "t1", "user": "default"},
  "kind": "clustering",
  "entry_count": 200,
  "k": 10,
  "expected_min_cluster_purity": 0.80,
  "expected_max_orphan_rate": 0.05,
  "must_surface_medoid": "decisions/api-auth.md",
  "must_recommend_consolidation_for": ["Cluster 1"]
}
```

### Metrics

| Metric | Gate |
|---|---|
| Cluster purity | `>= 0.80` on synthetic 200-entry corpus |
| Orphan rate | `<= 5%` |
| Memory entries deleted without confirmation | `0` |
| p95 audit latency (200 entries) | `< 500 ms` |
| Scope leakage across users | `0` |

### Caller Test Shape

```text
memory_hdc_audit_handler(user="default", k=10)

assert:
  - clusters returned, medoids named
  - intra-cluster similarity >= min_cluster_purity
  - no entry deleted
  - consolidation recommendation present for high-density clusters
  - scope boundary: no entries from other users appear
```

## Example 10: Native Memory Context Dedup

### Scenario

`NativeMemoryService::retrieve_context` returns 8 snippets, 3 of which are
near-duplicates about the same DB parity rule from different files.

### HDC Behavior

In shadow mode, HDC proposes dropping the 2 lower-authority duplicates. In live
mode, it actually drops them. Either way it reports the token savings.

```text
retrieve_context result:
  - verified-runbook-db-parity (authority: verified_runbook)  [keep]
  - daily-log-db-parity-1      (authority: daily_log)         [shadow-drop]
  - daily-log-db-parity-2      (authority: daily_log)         [shadow-drop]
  - anti-helper-only-tests     (authority: anti_knowledge)    [keep]
  - ...

shadow token savings: 340 tokens (17%)
```

### Fixture Row

```json
{
  "id": "native-context-dedup-db-parity-001",
  "kind": "native_context_dedup",
  "query": "add db-backed memory hdc pattern storage",
  "candidates": [
    {
      "id": "verified-runbook-db-parity",
      "authority": "verified_runbook",
      "content": "Add the DB trait method first, then implement both backends.",
      "token_count": 120,
      "expected_action": "keep"
    },
    {
      "id": "daily-log-db-parity-1",
      "authority": "daily_log",
      "content": "Remember to update the DB trait before both backends.",
      "token_count": 110,
      "expected_action": "drop"
    },
    {
      "id": "daily-log-db-parity-2",
      "authority": "daily_log",
      "content": "DB trait update must precede both backend implementations.",
      "token_count": 115,
      "expected_action": "drop"
    },
    {
      "id": "anti-helper-only-tests",
      "authority": "anti_knowledge",
      "content": "Helper-only tests missed the memory_write side effect.",
      "token_count": 95,
      "expected_action": "keep"
    }
  ],
  "must_keep_ids": ["verified-runbook-db-parity", "anti-helper-only-tests"],
  "min_token_savings": 200,
  "min_token_savings_ratio": 0.10,
  "max_dropped_required_citations": 0
}
```

### Metrics

| Metric | Gate |
|---|---|
| Token savings ratio | `>= 10%` before live mode |
| Dropped required citation count | `0` |
| Shadow vs live agreement | `100%` on fixture set |
| Reborn QA regression | `0` |
| p95 context assembly overhead | `< 2 ms` over baseline |

### Caller Test Shape

Drive through `ProductionMemoryPromptContextService::retrieve_context`:

```text
IRONCLAW_HDC_CONTEXT_DEDUP_MODE=shadow
ProductionMemoryPromptContextService::retrieve_context(query, budget)

assert:
  - shadow_drop events emitted for daily-log-db-parity-{1,2}
  - keep events emitted for verified-runbook and anti-knowledge entries
  - token_savings >= min_token_savings
  - no raw HDC vector in output
  - untrusted memory wrapper present on all retained snippets
```

## Example 11: Subagent Goal Dedup at Spawn Time

### Scenario

A parent spawns 4 subagents for code review. Two of them are assigned tasks that
are HDC-similar — both are reviewing the same workspace storage module.

### HDC Behavior

On the second spawn, `put_goal()` detects similarity >= 0.85 against an
in-flight goal in the same scope. Instead of spawning duplicate work it returns
the existing `TurnRunId`.

```text
spawn("Review workspace storage module: libSQL path")
  -> put_goal() -> novel, assigned TurnRunId: run-child-a

spawn("Audit workspace storage: libSQL fingerprint write path")
  -> put_goal() -> similarity 0.89 to run-child-a
  -> returning existing TurnRunId: run-child-a (no new subagent spawned)

spawn("Review PostgreSQL storage contract coverage")
  -> put_goal() -> novel (0.31 similarity), assigned TurnRunId: run-child-b

spawn("Inspect rate limiting on the HTTP webhook channel")
  -> put_goal() -> novel (0.18 similarity), assigned TurnRunId: run-child-c
```

### Fixture Row

```json
{
  "id": "subagent-goal-dedup-storage-review-001",
  "kind": "subagent_goal_dedup",
  "scope": {"tenant": "t1", "user": "u1", "project": "ironclaw"},
  "goals": [
    {
      "id": "goal-a",
      "text": "Review workspace storage module: libSQL path",
      "expected_action": "novel",
      "expected_run_id": "run-child-a"
    },
    {
      "id": "goal-b",
      "text": "Audit workspace storage: libSQL fingerprint write path",
      "expected_action": "duplicate",
      "expected_existing_run_id": "run-child-a",
      "expected_min_similarity": 0.85
    },
    {
      "id": "goal-c",
      "text": "Review PostgreSQL storage contract coverage",
      "expected_action": "novel",
      "expected_run_id": "run-child-b"
    },
    {
      "id": "goal-d",
      "text": "Inspect rate limiting on the HTTP webhook channel",
      "expected_action": "novel",
      "expected_run_id": "run-child-c"
    }
  ],
  "similarity_threshold": 0.85,
  "max_missed_duplicates": 0,
  "max_false_duplicates": 0
}
```

### Metrics

| Metric | Gate |
|---|---|
| Missed duplicates (novel returned for true duplicate) | `0` |
| False duplicates (duplicate returned for novel) | `0` |
| Returned TurnRunId validity | must be an in-flight goal in same scope |
| Scope leakage (cross-user duplicate match) | `0` |
| p95 `put_goal()` overhead | `< 2 ms` |

## Example 12: Fixture Corpus Validation

### Scenario

The existing fixture scenarios in `tests/fixtures/hdc_memory/` define expected
similarity ranges for each dedup scenario. A fixture harness reads them and
validates that the encoder produces scores within the declared bounds. This
catches encoder drift, tokenizer changes, or hyperparameter regressions before
they affect live behavior.

### HDC Behavior

```text
read manifest.json
for each dedup scenario:
  encode(entry_a), encode(entry_b)
  actual_similarity = cosine(vec_a, vec_b)
  assert actual_similarity in [expected_min, expected_max]

Results:
  exact-duplicate-001:        0.997 in [0.99, 1.00]  PASS
  paraphrase-db-parity-001:   0.831 in [0.75, 0.95]  PASS
  path-coincident-diff-001:   0.213 in [0.15, 0.45]  PASS
  ...
  17/17 scenarios within expected range
```

### Fixture Row (manifest structure)

```json
{
  "version": 1,
  "scenarios": [
    {
      "id": "exact-duplicate-001",
      "kind": "exact_duplicate",
      "entry_a": { "path": "notes/deploy.md", "content": "Run cargo deploy." },
      "entry_b": { "path": "notes/deploy-copy.md", "content": "Run cargo deploy." },
      "expected_similarity_min": 0.99,
      "expected_similarity_max": 1.00
    },
    {
      "id": "paraphrase-db-parity-001",
      "kind": "paraphrase",
      "entry_a": {
        "path": "runbooks/parity.md",
        "content": "Add the DB trait method first, then implement both backends."
      },
      "entry_b": {
        "path": "notes/parity-reminder.md",
        "content": "Always update the shared DB trait before touching either backend."
      },
      "expected_similarity_min": 0.75,
      "expected_similarity_max": 0.95
    },
    {
      "id": "path-coincident-diff-001",
      "kind": "path_coincident_different_content",
      "entry_a": {
        "path": "notes/deploy.md",
        "content": "Run cargo deploy to release the binary."
      },
      "entry_b": {
        "path": "notes/deploy.md",
        "content": "Deploy means pushing the Docker image and updating Kubernetes."
      },
      "expected_similarity_min": 0.15,
      "expected_similarity_max": 0.45
    }
  ],
  "all_scenarios_within_expected_range": true
}
```

### Metrics

| Metric | Gate |
|---|---|
| Scenarios within expected range | `17/17` (all) |
| Encoder drift across releases | alert if any scenario exits its range |
| Harness runtime | `< 1 s` for 17 scenarios |
| New scenario addition | must include expected range from empirical encoder output |

## Benchmark Harness Additions

Use the existing fixture style under `tests/fixtures/hdc_memory/`. Add an
`advanced` manifest section rather than inventing a new benchmark tree.

```json
{
  "advanced": {
    "episode_fingerprints": {
      "file": "advanced/episodes/db-parity-clusters.jsonl",
      "expected_cluster_purity_min": 0.85,
      "max_false_cross_cluster_merge_rate": 0.02
    },
    "anti_knowledge": {
      "file": "advanced/anti_knowledge/libsql-migration-warning.jsonl",
      "max_false_suppression": 0
    },
    "context_dedup": {
      "file": "advanced/context/runbook-vs-daily-log.jsonl",
      "min_token_savings_ratio": 0.15,
      "max_dropped_required_citations": 0
    },
    "resonance": {
      "file": "advanced/resonance/auth-to-rate-limit.jsonl",
      "min_precision_at_5": 0.70,
      "max_false_transfer_rate": 0.10
    },
    "heartbeat_attention": {
      "file": "advanced/heartbeat/repeated-changed-metric.jsonl",
      "max_high_severity_suppression": 0
    },
    "coordination": {
      "file": "advanced/coordination/duplicate-subagent-progress.jsonl",
      "max_lost_terminal_results": 0
    },
    "clustering": {
      "file": "advanced/clustering/memory-topic-audit.jsonl",
      "min_cluster_purity": 0.80,
      "max_orphan_rate": 0.05
    },
    "native_context_dedup": {
      "file": "advanced/native/context-snippet-dedup.jsonl",
      "min_token_savings_ratio": 0.10,
      "max_dropped_required_citations": 0
    },
    "subagent_goal_dedup": {
      "file": "advanced/coordination/goal-fingerprint-dedup.jsonl",
      "max_missed_duplicates": 0,
      "max_false_duplicates": 0
    },
    "fixture_corpus_validation": {
      "file": "fixtures/manifest.json",
      "all_scenarios_within_expected_range": true
    }
  }
}
```

### Synthetic Corpus Generator

The corpus generator should produce deterministic records so benchmarks are
stable:

```rust
pub struct SyntheticPatternCorpusConfig {
    pub seed: u64,
    pub episode_count: usize,
    pub duplicate_ratio: f32,
    pub false_friend_ratio: f32,
    pub domains: Vec<String>,
    pub scopes: Vec<String>,
}
```

Generate:

- exact duplicates,
- near duplicates,
- same-domain unrelated records,
- cross-domain analogies,
- false analogies with shared words but different causal shape,
- anti-knowledge near valid positive advice,
- multi-scope records that must not cross retrieval boundaries.

### Benchmark Commands

```bash
cargo test -p ironclaw_hdc
cargo bench -p ironclaw_hdc
cargo test --features hdc,libsql --test workspace_hdc_dedup --no-fail-fast
cargo test --features hdc --test heartbeat_hdc --no-fail-fast
cargo test --features hdc advanced_hdc_episode_patterns -- --nocapture
cargo test --features hdc advanced_hdc_context_dedup -- --nocapture
cargo test --features hdc --test workspace_hdc_fixtures --no-fail-fast
cargo test --features hdc -p ironclaw_hdc clustering -- --nocapture
```

The last four commands are proposed future targets. They should not be added to
required CI until fixtures and implementation exist.

## Promotion Checklist

Before any advanced HDC behavior becomes live:

- [ ] Feature is behind compile and runtime gates.
- [ ] Existing behavior is unchanged when disabled.
- [ ] Caller-level tests cover the side effect.
- [ ] Scope isolation test proves tenant/user/agent/project boundaries.
- [ ] Metrics do not include secrets, raw transcripts, private paths, or raw
      vectors by default.
- [ ] PostgreSQL/libSQL parity exists for new durable state, or state is
      explicitly in-memory/derived.
- [ ] False suppression/blocking gates pass on fixtures.
- [ ] Prompt wrappers and bounded snippet limits are preserved.
- [ ] Rollback is documented as a flag/config change.
- [ ] `FEATURE_PARITY.md`, subsystem docs, and PR body are checked for updates.

## Practical Build Order

1. Add advanced fixtures and helper tests with no production behavior.
2. Add episode pattern shadow recording.
3. Add diagnostic episode query command.
4. Add anti-knowledge warning in shadow mode.
5. Add context dedup shadow decisions at the memory-context boundary.
6. Add heartbeat metadata persistence only after runner-level tests.
7. Add offline replay/counterfactual lab using hermetic fixtures.
8. Add subagent/routine coordination diagnostics.
9. Consider live advisory prompt inclusion.
10. Only then consider any routing, suppression, or blocking behavior.


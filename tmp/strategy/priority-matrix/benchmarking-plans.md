# Benchmarking Plans: Metrics, Baselines, and Targets

> **Navigation**:
> [README](README.md) |
> [Detailed Rankings](detailed-rankings.md) |
> [Implementation Sketches](implementation-sketches.md) |
> [Quick Wins](quick-wins.md) |
> [Synergy Analysis](synergy-analysis.md) |
> [Benchmarking Plans (you are here)](benchmarking-plans.md) |
> [References](references.md)

For each item: specific metrics to measure, baseline values, targets, and how to measure them.
All measurements should run for 30 days before drawing conclusions. See also:
[../../implementation/benchmarking/01-measurement-framework.md](../../implementation/benchmarking/01-measurement-framework.md)

---

## Rank 1: Metacognitive Monitor

| Metric | Baseline | Target | How to Measure |
|--------|----------|--------|----------------|
| Stuck loop rate per 100 sessions | 5–10 occurrences | ≤2 occurrences | Count sessions where `max_turns` is hit without task completion |
| Average turns wasted in stuck loops | 15–20 turns | ≤4 turns | Average turns between loop start and correction |
| Cost wasted on stuck loops ($/month, $100 baseline) | $5–15 | ≤$3 | `SUM(cost_cents) WHERE session_stuck=true` per month |
| False positive rate (loop detected when making progress) | N/A | ≤5% | Manual review of 100 loop-detection events |
| Time from loop start to correction injection | N/A | ≤4 turns | `turn_detected - turn_loop_started` in events |

**Measurement methodology:** Add a `metacognitive_events` table with columns
`(session_id, turn, pathology_type, unique_ratio, cost_ewma, correction_applied)`.
After 30 days of data, compute stuck loop rate and cost waste.

**Schema:**

```sql
-- PostgreSQL
CREATE TABLE metacognitive_events (
    id          BIGSERIAL PRIMARY KEY,
    session_id  TEXT NOT NULL,
    turn        INTEGER NOT NULL,
    pathology   TEXT NOT NULL,  -- 'stuck_loop', 'cost_runaway', 'contradiction'
    unique_ratio DOUBLE PRECISION,
    cost_ewma   DOUBLE PRECISION,
    correction  TEXT NOT NULL,  -- 'break_loop', 'downgrade_model', 'flag', 'none'
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Alerting thresholds:**
- Alert if false positive rate > 10% in any 7-day window
- Alert if stuck loop rate > 5 per 100 sessions after monitor is active (indicates monitor is not working)

---

## Rank 2: Ebbinghaus Decay

| Metric | Baseline | Target | How to Measure |
|--------|----------|--------|----------------|
| Stale entries (not accessed in 30+ days) in top-10 search results | ~40% | ≤10% | Sample 50 queries, count stale entries in top-10 |
| Search relevance: user confirms top result is useful | ~60% | ≥80% | Thumbs up/down on top memory result per query |
| Memory store size growth rate | Linear (unbounded) | Sub-linear (faded entries excluded from search) | Entry count vs weeks of use |
| Entries filtered by decay per day | 0 | 10–50% of all entries (below FADED_THRESHOLD) | `COUNT(*) WHERE current_strength < 0.05` |
| Time to first access of a retrieved memory (proxy for relevance) | Not tracked | Decreasing over months | Average elapsed since creation for accessed entries |

**Measurement methodology:** Log `(entry_id, query_text, rank_in_results, strength_at_query_time)`
for every search. After 30 days, compute stale-entry rate and strength distribution.

**Schema:**

```sql
CREATE TABLE memory_search_events (
    id              BIGSERIAL PRIMARY KEY,
    query_text      TEXT NOT NULL,
    entry_id        TEXT NOT NULL,
    rank_in_results INTEGER NOT NULL,
    decay_strength  DOUBLE PRECISION NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Week-over-week tracking:**
```sql
-- Stale-entry rate in top-10 results (weekly)
SELECT
    DATE_TRUNC('week', created_at) AS week,
    COUNT(*) FILTER (WHERE rank_in_results <= 10 AND decay_strength < 0.1) AS stale_in_top10,
    COUNT(*) FILTER (WHERE rank_in_results <= 10) AS total_in_top10,
    ROUND(
        100.0 * COUNT(*) FILTER (WHERE rank_in_results <= 10 AND decay_strength < 0.1)
        / NULLIF(COUNT(*) FILTER (WHERE rank_in_results <= 10), 0),
        1
    ) AS stale_pct
FROM memory_search_events
GROUP BY week
ORDER BY week;
```

---

## Rank 3: BLAKE3 Content Dedup

| Metric | Baseline | Target | How to Measure |
|--------|----------|--------|----------------|
| Duplicate memory entries (same content, different path) | Unbounded | 0 | `SELECT content_hash, COUNT(*) FROM workspace_entries GROUP BY content_hash HAVING COUNT(*) > 1` |
| Context window tokens wasted on duplicates per query | 100–500 | 0 | (entries_returned × avg_entry_tokens) - (unique_entries × avg_entry_tokens) |
| Memory write latency overhead from dedup check | Baseline | ≤+5ms overhead | Measure `memory_write` p99 latency before and after |
| Dedup hit rate (merge vs insert ratio) | 0% | 15–40% for active users | `merges / (merges + inserts)` over 30 days |

**Measurement methodology:** Add a `memory_write_log` table with `(timestamp, operation_type, hash, elapsed_ms)`.
The dedup hit rate is `COUNT(merge) / COUNT(*)` per day.

**Schema:**

```sql
CREATE TABLE memory_write_log (
    id             BIGSERIAL PRIMARY KEY,
    operation_type TEXT NOT NULL,  -- 'insert' or 'merge'
    content_hash   BYTEA NOT NULL,
    elapsed_ms     INTEGER NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Dedup hit rate query:**

```sql
SELECT
    DATE_TRUNC('day', created_at) AS day,
    COUNT(*) FILTER (WHERE operation_type = 'merge') AS merges,
    COUNT(*) FILTER (WHERE operation_type = 'insert') AS inserts,
    ROUND(
        100.0 * COUNT(*) FILTER (WHERE operation_type = 'merge') / NULLIF(COUNT(*), 0),
        1
    ) AS merge_pct
FROM memory_write_log
GROUP BY day
ORDER BY day;
```

---

## Rank 4: Robust Statistics

| Metric | Baseline | Target | How to Measure |
|--------|----------|--------|----------------|
| Estimation RMSE (root mean squared error vs actual task duration) | 30–50s | 10–20s | `SQRT(AVG((estimate - actual)^2))` over 100 tasks |
| Estimation bias after LLM timeout outlier | Shifts up 2–3× for hours | No detectable shift | Compare rolling 1h mean before and after each timeout event |
| Estimation coverage (actual within predicted ±2σ) | ~50% (not calibrated) | ≥80% | Count actual durations within [estimate - 2*mad, estimate + 2*mad] |
| Time for estimate to recover from outlier (EMA) | 3–5 hours | Immediate (trimmed mean ignores outlier) | Measure estimate 1 hour after an outlier vs before |

**Measurement methodology:** Log `(task_id, estimated_secs, actual_secs, estimation_method)`.
After switching to robust stats, compare RMSE over 30-day windows.

**RMSE comparison query:**

```sql
SELECT
    estimation_method,
    ROUND(SQRT(AVG(POWER(estimated_secs - actual_secs, 2)))::NUMERIC, 1) AS rmse,
    COUNT(*) AS sample_size
FROM task_estimation_log
WHERE created_at > NOW() - INTERVAL '30 days'
GROUP BY estimation_method;
```

---

## Rank 5: Composable Scorers

| Metric | Baseline | Target | How to Measure |
|--------|----------|--------|----------------|
| Skill selection accuracy (correct skill for task) | Manually measured baseline | +10% relative | A/B test skill selection on 50 labeled tasks |
| Scorer code coverage | Ad-hoc (varies) | ≥90% line coverage | `cargo tarpaulin` on `src/evaluation/scorer.rs` |
| Number of unique scoring implementations (code duplication) | 5–8 scattered | 1 canonical framework | LOC in scoring-related files |
| Time to add a new scorer | 1–2 days (no abstraction) | ≤2 hours (extend trait) | Developer measurement |

**Measurement methodology:** Create a labeled test set of 50 task descriptions with ground-truth
skill assignments. Run skill selection before and after composable scorers and compare accuracy.

**A/B test setup:**

```rust
// In src/skills/attenuate_tools.rs or equivalent:
// Feature flag: COMPOSABLE_SCORER_ENABLED
// When enabled: use WeightedScorer(RecencyScorer + PopularityScorer + TagScorer)
// When disabled: use existing ad-hoc scoring
// Log (task_id, scorer_type, selected_skill, correct_skill) for each run
```

---

## Rank 6: Hierarchical Cancellation

| Metric | Baseline | Target | How to Measure |
|--------|----------|--------|----------------|
| Leaked async tasks after session cancel | Measured via `tokio::runtime::Handle::metrics()` | 0 | Check task count before and after session shutdown |
| Time for all tasks to stop after session cancel | 1–5s (tasks finish in-flight requests) | ≤500ms | Measure from `cancel()` to last task completion |
| Tool call cancelled correctly without killing session | Not supported | Works in 100% of test cases | Unit test: cancel tool token, verify job continues |
| Memory consumption after 10 consecutive session cancels | Grows (leaked tasks) | Stable | Track `tokio::runtime::Handle::metrics().num_alive_tasks()` |

**Measurement methodology:** Integration test that spawns a session with 5 parallel tool calls,
cancels the session, and asserts that all tasks stop within 500ms and no tasks remain alive.

**Test skeleton:**

```rust
#[tokio::test]
async fn session_cancel_cleans_up_all_tasks() {
    let metrics = tokio::runtime::Handle::current().metrics();
    let tasks_before = metrics.num_alive_tasks();

    let session = spawn_test_session_with_5_parallel_tools().await;
    session.cancel().await;

    tokio::time::sleep(Duration::from_millis(600)).await;
    let tasks_after = metrics.num_alive_tasks();
    assert_eq!(tasks_after, tasks_before, "no task leaks after session cancel");
}
```

---

## Rank 7: Cascade Router

| Metric | Baseline | Target | How to Measure |
|--------|----------|--------|----------------|
| Monthly LLM cost ($100 baseline) | $100 | $50–70 | Monthly invoice comparison |
| % requests routed to cheap model | ~30% (static) | 50–70% (learned) | `model_id = 'cheap_model'` count in routing_events |
| Task success rate (no regression) | Baseline | ≤1% degradation | `success / total` from evaluation system |
| Bandit learning convergence (turns to stable policy) | N/A | ≤500 requests per task type | Measure routing decision stability over time |
| Static rule hit rate | N/A | ≥20% of all requests | `routing_stage = 'static_rule'` count |
| Avg latency per request (model switch overhead) | Baseline | ≤50ms overhead | p99 routing decision latency |

**Measurement methodology:** Log every routing decision to a `routing_events` table
with `(request_id, stage, model_selected, context_json, reward, timestamp)`. Track
cost per week and task success rate over 60 days.

**Schema:**

```sql
CREATE TABLE routing_events (
    id              BIGSERIAL PRIMARY KEY,
    request_id      TEXT NOT NULL,
    routing_stage   TEXT NOT NULL,  -- 'static_rule', 'bandit', 'fallback'
    rule_name       TEXT,
    model_selected  TEXT NOT NULL,
    ucb_score       DOUBLE PRECISION,
    reward          DOUBLE PRECISION,  -- filled in after task completion
    context_json    JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Cost tracking query:**

```sql
SELECT
    DATE_TRUNC('week', created_at) AS week,
    routing_stage,
    COUNT(*) AS requests,
    ROUND(AVG(reward)::NUMERIC, 3) AS avg_reward,
    COUNT(*) FILTER (WHERE model_selected LIKE '%haiku%' OR model_selected LIKE '%mini%') AS cheap_count,
    ROUND(
        100.0 * COUNT(*) FILTER (WHERE model_selected LIKE '%haiku%' OR model_selected LIKE '%mini%')
        / NULLIF(COUNT(*), 0),
        1
    ) AS cheap_pct
FROM routing_events
GROUP BY week, routing_stage
ORDER BY week, routing_stage;
```

---

## Rank 8: Cognitive Speed Labels

| Metric | Baseline | Target | How to Measure |
|--------|----------|--------|----------------|
| Classification accuracy (Gamma/Theta/Delta correct) | N/A (no classification) | ≥85% on labeled test set | 100 manually labeled requests, compare classifier output |
| Cost reduction from Gamma routing (cheap model) | 0% (no Gamma tier) | 15–25% of monthly bill | Cost of Gamma-classified requests vs what they would have cost on primary model |
| Misclassified high-complexity tasks (Gamma when should be Theta) | N/A | ≤5% | Sample 50 high-complexity Gamma-labeled tasks |
| Average latency for Gamma requests | Same as all requests | ≤5s p95 | p95 latency for `cognitive_speed = 'Gamma'` requests |

**Measurement methodology:** Add `cognitive_speed` to `routing_events`. Build a 100-item
labeled test set (25 Gamma, 50 Theta, 25 Delta) and run the classifier against it monthly.

**Labeled test set construction:**

```
Gamma (25 items): "What time is it?", "Hello", "Thanks", "Stop", "Yes", ...
  Characteristics: ≤10 words, no code, no tool calls, complexity < 0.2
Theta (50 items): "Explain async Rust", "How do I deploy to AWS?", "Debug this error: ..."
  Characteristics: 11–100 words, possibly code context, moderate complexity
Delta (25 items): "Refactor my entire codebase to use async", "Analyze all my sessions for patterns"
  Characteristics: long, multi-step, involves many tool calls, background tasks
```

---

## Rank 9: HDC Similarity Engine

| Metric | Baseline | Target | How to Measure |
|--------|----------|--------|----------------|
| Memory search latency (p99) for 10K entries | 20–50ms (FTS + embedding) | ≤5ms additional for HDC (total ≤55ms) | Benchmark with criterion |
| Search precision@5 (top-5 results are relevant) | ~65% (FTS + embedding) | ≥75% (with HDC as third signal) | 50 labeled queries, human eval |
| Compositional search accuracy (find "X AND Y" concepts) | Not supported | ≥60% accuracy | 30 labeled compositional queries |
| HDC index build time for 10K entries | N/A | ≤100ms | Benchmark |
| Memory for HDC index (10K entries × 10,240 bits) | N/A | ~12MB (10K × 160 × 8 bytes) | `std::mem::size_of` × entry count |

**Measurement methodology:** Create a 50-query benchmark set with human-rated relevance judgments
(1–5 scale). Compute NDCG@5 before (FTS + embedding) and after (+ HDC). Run at weekly intervals
as the memory grows.

**Criterion benchmark:**

```rust
#[bench]
fn bench_hdc_search_10k(b: &mut Bencher) {
    let mut index = HdcIndex::new();
    for i in 0..10_000 {
        index.insert(format!("entry_{}", i), HyperVector::random());
    }
    let query = HyperVector::random();
    b.iter(|| index.search_top_k(&query, 10));
}
```

---

## Rank 10: Gate Verification Pipeline

| Metric | Baseline | Target | How to Measure |
|--------|----------|--------|----------------|
| Syntax errors in generated code reaching the user | 100% (no gate) | 0% (caught by Rung 1) | Count `compile_error` events in production vs gate violations caught |
| Lint violations in generated code | 100% reach user | 0% with `-D warnings` | Same |
| Test failures in generated code | 100% reach user | ≤10% (some tests can't run in sandbox) | Count test failures in gate vs deployed code |
| Gate pipeline latency (Rungs 1–4, Rust project) | N/A | ≤3 minutes total | Benchmark on a 1K-file Rust project |
| False failure rate (gate fails valid code) | N/A | ≤2% | Track gate failures where manual review confirms code was correct |

**Measurement methodology:** For every code generation task, log whether the gate passed or
failed and which rung caught the issue. Compare to a 30-day historical baseline of how many
generated code blocks had errors when deployed without a gate.

**Schema:**

```sql
CREATE TABLE gate_pipeline_events (
    id              BIGSERIAL PRIMARY KEY,
    task_id         TEXT NOT NULL,
    all_passed      BOOLEAN NOT NULL,
    failing_rung    TEXT,  -- 'compile', 'lint', 'test', 'symbol', or NULL if all passed
    total_elapsed_ms INTEGER NOT NULL,
    compile_elapsed_ms INTEGER,
    lint_elapsed_ms  INTEGER,
    test_elapsed_ms  INTEGER,
    symbol_elapsed_ms INTEGER,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Error catch rate query:**

```sql
SELECT
    DATE_TRUNC('week', created_at) AS week,
    COUNT(*) AS total_runs,
    COUNT(*) FILTER (WHERE all_passed) AS passed,
    COUNT(*) FILTER (WHERE failing_rung = 'compile') AS compile_failures,
    COUNT(*) FILTER (WHERE failing_rung = 'lint') AS lint_failures,
    COUNT(*) FILTER (WHERE failing_rung = 'test') AS test_failures,
    COUNT(*) FILTER (WHERE failing_rung = 'symbol') AS symbol_failures,
    ROUND(100.0 * COUNT(*) FILTER (WHERE all_passed) / NULLIF(COUNT(*), 0), 1) AS pass_rate_pct
FROM gate_pipeline_events
GROUP BY week
ORDER BY week;
```

---

## Shared Measurement Infrastructure

All benchmarking plans share these infrastructure requirements:

### Logging Convention

All measurement events must use `debug!()` logging, never `info!()`. The REPL/TUI renders
`info!()` output and corrupting the terminal is a regression:

```rust
debug!(
    session_id = %session.id,
    metric = "stuck_loop_detected",
    unique_ratio = unique_ratio,
    "metacognitive: pathology detected"
);
```

### 30-Day Minimum Window

Statistical significance requires at least 30 days of data before comparing before/after metrics.
Do not draw conclusions from less than 100 events for any metric.

### A/B Testing Protocol

For features that change behavior (Composable Scorers, Cascade Router), use environment-based
feature flags to enable gradual rollout:

```bash
# Enable for 10% of sessions
FEATURE_COMPOSABLE_SCORER_ROLLOUT=0.10

# Full rollout after 30-day validation
FEATURE_COMPOSABLE_SCORER_ROLLOUT=1.00
```

### Alerting

For each metric with a target, set an alert when the metric exceeds 1.5× the target in any
7-day rolling window. This catches regressions introduced by subsequent changes.

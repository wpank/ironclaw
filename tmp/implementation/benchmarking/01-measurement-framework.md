# Measurement Framework

Use one event shape for microbenchmarks, scenario fixtures, rollout telemetry,
and CI reports. The canonical schema lives in
`../schemas/04-canonical-event-and-persistence-contract.md`.

## Metric Event

```rust
pub struct MetricEvent {
    pub event_id: String,
    pub run_id: String,
    pub feature: String,
    pub feature_flag_id: Option<String>,
    pub variant: String,
    pub stage: String,
    pub timestamp_ms: i64,
    pub latency_ms: Option<u64>,
    pub cost_microusd: Option<u64>,
    pub token_count: Option<u64>,
    pub quality_pass: Option<bool>,
    pub policy_violation: bool,
    pub metadata: serde_json::Value,
}
```

## Comparison Rules

For baseline `A` and candidate `B`, report:

- pass-rate delta in percentage points;
- median and p95 latency delta;
- median and p95 cost delta;
- timeout/fallback/retry deltas;
- safety violation count.

Prefer paired comparisons when the same fixture input can run through baseline
and candidate. Use bootstrap confidence intervals for pass-rate, median, and
p95 deltas when sample size is at least 30. For smaller local fixtures, report
the raw counts and do not promote automatically.

## Decision Rules

| Decision | Rule |
| --- | --- |
| `rollback` | any policy/auth/secret leak, or p95 latency regression > 10% |
| `hold` | target metric does not improve or confidence is too weak |
| `promote` | target improves and all guardrails pass |
| `observe` | candidate collects metrics but cannot affect behavior |

## Bounded Metrics

- Percent values are `0..=100`.
- Rates are `0.0..=1.0`.
- Latency, cost, token, retry, and count metrics are non-negative.
- Cardinality-heavy data is stored as artifact refs, not metric labels.

## Rollout Stages

| Stage | Traffic | Authority |
| --- | --- | --- |
| `local` | fixture or developer run | candidate may act only in test |
| `shadow` | mirrored production-like input | baseline controls behavior |
| `canary` | small opted-in cohort | candidate controls eligible requests |
| `limited` | broader opted-in cohort | candidate controls eligible requests |
| `default` | default path | candidate is baseline, flag cleanup starts |

# Canonical Event And Persistence Contract

This file defines durable experimental telemetry: benchmark events, rollout
events, caller-level gate verdicts, feature exposure, and rollback decisions.

## Canonical Metric Event

```rust
pub struct MetricEvent {
    pub event_id: String,
    pub run_id: String,
    pub turn_id: Option<String>,
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

Field rules:

- `feature`, `variant`, and `stage` are bounded enums in practice.
- `metadata` may include hashed ids, fixture names, redaction status, and
  bounded classifications only.
- `policy_violation=true` overrides all positive metrics.

## Tables

### Metric Events

PostgreSQL and libSQL use the same logical columns:

```sql
CREATE TABLE metric_events (
    event_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    turn_id TEXT,
    feature TEXT NOT NULL,
    feature_flag_id TEXT,
    variant TEXT NOT NULL,
    stage TEXT NOT NULL,
    timestamp_ms BIGINT NOT NULL,
    latency_ms BIGINT,
    cost_microusd BIGINT,
    token_count BIGINT,
    quality_pass BOOLEAN,
    policy_violation BOOLEAN NOT NULL DEFAULT FALSE,
    metadata_json TEXT NOT NULL
);
```

Indexes:

```sql
CREATE INDEX idx_metric_events_feature_time
    ON metric_events(feature, timestamp_ms DESC);
CREATE INDEX idx_metric_events_run
    ON metric_events(run_id, timestamp_ms);
```

### Feature Exposure Events

```sql
CREATE TABLE feature_exposure_events (
    exposure_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    turn_id TEXT,
    feature_flag_id TEXT NOT NULL,
    feature_key TEXT NOT NULL,
    variant TEXT NOT NULL,
    stage TEXT NOT NULL,
    enabled BOOLEAN NOT NULL,
    timestamp_ms BIGINT NOT NULL
);
```

### Gate Verdicts

```sql
CREATE TABLE gate_verdicts (
    verdict_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    feature TEXT NOT NULL,
    rung TEXT NOT NULL,
    status TEXT NOT NULL,
    duration_ms BIGINT NOT NULL,
    artifact_ref TEXT,
    redaction_applied BOOLEAN NOT NULL
);
```

### Signal Records

```sql
CREATE TABLE signal_records (
    signal_id TEXT PRIMARY KEY,
    content_hash TEXT NOT NULL,
    kind TEXT NOT NULL,
    confidence REAL NOT NULL,
    taint_json TEXT NOT NULL,
    origin_ids_json TEXT NOT NULL,
    created_at_ms BIGINT NOT NULL,
    metadata_json TEXT NOT NULL
);
```

### Rollout Decisions

```sql
CREATE TABLE rollout_decisions (
    decision_id TEXT PRIMARY KEY,
    feature TEXT NOT NULL,
    stage TEXT NOT NULL,
    decision TEXT NOT NULL,
    reason TEXT NOT NULL,
    metric_window_start_ms BIGINT NOT NULL,
    metric_window_end_ms BIGINT NOT NULL,
    created_at_ms BIGINT NOT NULL
);
```

`decision` is `hold`, `promote`, `rollback`, or `remove_flag`.

## Migration Order

1. `001_metric_events`
2. `002_feature_exposure_events`
3. `003_gate_verdicts`
4. `004_signal_records`
5. `005_rollout_decisions`

Each migration must have PostgreSQL and libSQL forms plus a dual-backend
round-trip test.

## Retention

| Data | Default retention |
| --- | --- |
| Metric events | 90 days |
| Feature exposures | 90 days |
| Gate artifacts | 30 days, redacted |
| Signal records | product retention policy |
| Rollout decisions | keep indefinitely |

## Idempotency

Events use deterministic ids where possible. Duplicate inserts of the same id
must be safe for retrying a crashed benchmark or rollout collector.

# Canonical Event And Persistence Contract

This document unifies the metric/event shapes used by benchmarking, rollout,
schemas, and examples. Other documents should treat this as the canonical event
contract.

## 1. Canonical Metric Event

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricEvent {
    pub schema_version: u16,
    pub event_id: String,
    pub run_id: String,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub tool_call_id: Option<String>,
    pub feature_flag_id: Option<String>,
    pub feature: String,
    pub variant: String,
    pub scenario: String,
    pub stage: String,
    pub timestamp_ms: i64,
    pub latency_ms: Option<u64>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cost_microusd: Option<u64>,
    pub quality_pass: Option<bool>,
    pub score: Option<f64>,
    pub error_kind: Option<String>,
    pub fallback_used: bool,
    pub approval_required: bool,
    pub policy_violation: bool,
    pub redaction_applied: bool,
}
```

Rules:

- Use `cost_microusd`, not floating-point USD.
- Use `score` for generic scalar quality/relevance values.
- Use `stage` for `local`, `shadow`, `canary`, `limited`, or `default`.
- Keep raw content out of this event.

## 2. Additional Tables

### Metric Events

PostgreSQL:

```sql
CREATE TABLE metric_events (
    event_id TEXT PRIMARY KEY,
    schema_version SMALLINT NOT NULL,
    run_id TEXT NOT NULL,
    thread_id TEXT,
    turn_id TEXT,
    tool_call_id TEXT,
    feature_flag_id TEXT,
    feature TEXT NOT NULL,
    variant TEXT NOT NULL,
    scenario TEXT NOT NULL,
    stage TEXT NOT NULL,
    timestamp_ms BIGINT NOT NULL,
    latency_ms BIGINT,
    input_tokens BIGINT,
    output_tokens BIGINT,
    cost_microusd BIGINT,
    quality_pass BOOLEAN,
    score DOUBLE PRECISION,
    error_kind TEXT,
    fallback_used BOOLEAN NOT NULL DEFAULT FALSE,
    approval_required BOOLEAN NOT NULL DEFAULT FALSE,
    policy_violation BOOLEAN NOT NULL DEFAULT FALSE,
    redaction_applied BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_metric_events_feature_time
    ON metric_events(feature, timestamp_ms DESC);

CREATE INDEX idx_metric_events_run
    ON metric_events(run_id, timestamp_ms);
```

libSQL:

```sql
CREATE TABLE metric_events (
    event_id TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL,
    run_id TEXT NOT NULL,
    thread_id TEXT,
    turn_id TEXT,
    tool_call_id TEXT,
    feature_flag_id TEXT,
    feature TEXT NOT NULL,
    variant TEXT NOT NULL,
    scenario TEXT NOT NULL,
    stage TEXT NOT NULL,
    timestamp_ms INTEGER NOT NULL,
    latency_ms INTEGER,
    input_tokens INTEGER,
    output_tokens INTEGER,
    cost_microusd INTEGER,
    quality_pass INTEGER,
    score REAL,
    error_kind TEXT,
    fallback_used INTEGER NOT NULL DEFAULT 0,
    approval_required INTEGER NOT NULL DEFAULT 0,
    policy_violation INTEGER NOT NULL DEFAULT 0,
    redaction_applied INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_metric_events_feature_time
    ON metric_events(feature, timestamp_ms DESC);

CREATE INDEX idx_metric_events_run
    ON metric_events(run_id, timestamp_ms);
```

### Feature Exposure Events

```sql
CREATE TABLE feature_exposure_events (
    event_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    turn_id TEXT,
    feature_flag_id TEXT NOT NULL,
    feature TEXT NOT NULL,
    variant TEXT NOT NULL,
    stage TEXT NOT NULL,
    enabled INTEGER NOT NULL,
    reason TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL
);

CREATE INDEX idx_feature_exposure_feature_time
    ON feature_exposure_events(feature, created_at_ms DESC);
```

### Gate Verdicts

```sql
CREATE TABLE gate_verdicts (
    verdict_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    rung TEXT NOT NULL,
    status TEXT NOT NULL,
    confidence REAL NOT NULL,
    duration_ms INTEGER NOT NULL,
    artifact_refs_json TEXT NOT NULL,
    remediation TEXT,
    redaction_applied INTEGER NOT NULL DEFAULT 0,
    created_at_ms INTEGER NOT NULL
);

CREATE INDEX idx_gate_verdicts_run_id
    ON gate_verdicts(run_id);
```

### Signal Records

```sql
CREATE TABLE signal_records (
    signal_id TEXT PRIMARY KEY,
    content_hash_blake3 TEXT NOT NULL,
    workspace_path TEXT NOT NULL,
    title TEXT,
    content_type TEXT NOT NULL,
    parent_signal_ids_json TEXT NOT NULL,
    confidence REAL NOT NULL,
    utility REAL NOT NULL,
    novelty REAL NOT NULL,
    half_life_seconds INTEGER,
    taints_json TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    last_accessed_ms INTEGER NOT NULL,
    UNIQUE(workspace_path, content_hash_blake3)
);

CREATE INDEX idx_signal_records_workspace_access
    ON signal_records(workspace_path, last_accessed_ms DESC);
```

### DAG Run Records

```sql
CREATE TABLE dag_run_records (
    run_id TEXT PRIMARY KEY,
    graph_id TEXT NOT NULL,
    owner_turn_id TEXT,
    variant TEXT NOT NULL,
    nodes_json TEXT NOT NULL,
    total_cost_microusd INTEGER NOT NULL DEFAULT 0,
    cancelled INTEGER NOT NULL DEFAULT 0,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);
```

### Reputation Events

```sql
CREATE TABLE reputation_events (
    event_id TEXT PRIMARY KEY,
    actor_id TEXT NOT NULL,
    domain TEXT NOT NULL,
    delta REAL NOT NULL,
    evidence_ref TEXT NOT NULL,
    evaluator TEXT NOT NULL,
    signature TEXT,
    created_at_ms INTEGER NOT NULL
);

CREATE INDEX idx_reputation_actor_domain
    ON reputation_events(actor_id, domain, created_at_ms DESC);
```

### Rollout Decisions

```sql
CREATE TABLE rollout_decisions (
    decision_id TEXT PRIMARY KEY,
    feature TEXT NOT NULL,
    stage TEXT NOT NULL,
    decision TEXT NOT NULL,
    reason TEXT NOT NULL,
    metric_window_start_ms INTEGER NOT NULL,
    metric_window_end_ms INTEGER NOT NULL,
    reviewer TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL
);
```

## 3. Migration Order

```text
001_metric_events
002_feature_exposure_events
003_gate_verdicts
004_signal_records
005_dag_run_records
006_reputation_events
007_rollout_decisions
```

PostgreSQL and libSQL migrations should share numeric prefixes and semantic
names where both systems use versioned migrations. In the current repo,
PostgreSQL migrations live under `migrations/`; libSQL incremental migrations
are appended in `src/db/libsql_migrations.rs`. Rollback should disable callers
before any data cleanup.

## 4. Retention Policy

| Data | Retention |
|---|---|
| metric events | configurable, default 90 days |
| feature exposure events | 180 days |
| gate verdicts | tied to run artifact retention |
| signal records | user/workspace memory retention |
| DAG run records | 180 days or until artifact cleanup |
| reputation events | append-only unless user deletes workspace |
| rollout decisions | permanent audit log unless exported/archived |

## 5. Backfill And Idempotency

- Backfills must be restartable.
- Use `event_id`, `signal_id`, `run_id`, or `decision_id` as idempotency keys.
- Do not backfill raw prompts into metrics.
- Backfill status should be recorded as a rollout decision or migration note.

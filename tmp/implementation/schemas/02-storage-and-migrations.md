# Storage And Migration Sketches

IronClaw uses both PostgreSQL and libSQL/Turso. Any persisted feature from these
documents must either route through an existing dual-backend facade or add trait
methods and backend implementations for both stores in the same change.

## 1. Database Trait Additions

Sketch:

```rust
#[async_trait::async_trait]
pub trait ExperimentalTelemetryStore {
    async fn insert_metric_event(&self, event: MetricEvent) -> anyhow::Result<()>;
    async fn list_metric_events(
        &self,
        feature: ExperimentalFeature,
        since_ms: i64,
        limit: u32,
    ) -> anyhow::Result<Vec<MetricEvent>>;

    async fn insert_gate_verdict(&self, verdict: GateVerdict) -> anyhow::Result<()>;
    async fn get_gate_verdicts(&self, run_id: &str) -> anyhow::Result<Vec<GateVerdict>>;

    async fn upsert_signal_record(&self, signal: SignalRecord) -> anyhow::Result<()>;
    async fn get_signal_by_hash(
        &self,
        workspace_path: &str,
        content_hash_blake3: &str,
    ) -> anyhow::Result<Option<SignalRecord>>;
}
```

Implementation rule:

```text
trait first -> postgres implementation -> libSQL implementation -> shared contract tests
```

## 2. Metric Events Table

PostgreSQL:

```sql
CREATE TABLE experimental_metric_events (
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
    redaction_applied BOOLEAN NOT NULL DEFAULT FALSE,
    payload JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX idx_experimental_metric_events_feature_time
    ON experimental_metric_events(feature, timestamp_ms DESC);
```

libSQL:

```sql
CREATE TABLE experimental_metric_events (
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
    redaction_applied INTEGER NOT NULL DEFAULT 0,
    payload TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_experimental_metric_events_feature_time
    ON experimental_metric_events(feature, timestamp_ms DESC);
```

## 3. Gate Verdicts Table

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

CREATE INDEX idx_gate_verdicts_run_id ON gate_verdicts(run_id);
```

PostgreSQL can use `JSONB` for `artifact_refs_json`, but the Rust trait should
hide that difference and expose `Vec<String>`.

## 4. Signal Records Table

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

## 5. Migration Order

1. Add tables and indexes with no callers.
2. Add write path behind disabled feature flags.
3. Backfill only when the flag is enabled in a local or canary environment.
4. Add read path and caller-level tests.
5. Enable shadow mode.
6. Promote to canary only after dual-backend contract tests pass.

## 6. Contract Test Shape

```rust
pub async fn telemetry_store_contract<S>(store: S)
where
    S: ExperimentalTelemetryStore + Send + Sync,
{
    let event = fixture_metric_event("contract.run.1");
    store.insert_metric_event(event.clone()).await.unwrap();

    let loaded = store
        .list_metric_events(event.feature.clone(), event.timestamp_ms - 1, 10)
        .await
        .unwrap();

    assert!(loaded.iter().any(|row| row.run_id == event.run_id));
}
```

The same contract test should run against PostgreSQL and libSQL test fixtures.

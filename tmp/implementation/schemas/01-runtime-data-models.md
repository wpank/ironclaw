# Runtime Data Models

These models are planning contracts, not generated Rust. They describe the
fields new implementations should converge on when adding metrics, gates,
signals, rollout decisions, and reputation events.

## Feature Keys

Use one key in flags, metrics, fixtures, docs, and dashboards:

| Capability | Runtime flag | Exposure id |
| --- | --- | --- |
| Signal records | `experimental.signal_records` | `flag.signal_records` |
| HDC memory search | `experimental.hdc_memory_search` | `flag.hdc_memory_search` |
| Cascade router | `experimental.cascade_router` | `flag.cascade_router` |
| Progressive gates | `experimental.progressive_gates` | `flag.progressive_gates` |
| Provider conductor | `experimental.provider_conductor` | `flag.provider_conductor` |
| Dream consolidation | `experimental.dream_consolidation` | `flag.dream_consolidation` |
| DAG workflow runner | `experimental.dag_workflow_runner` | `flag.dag_workflow_runner` |
| Workspace code search | `experimental.workspace_code_search` | `flag.workspace_code_search` |
| Local reputation | `experimental.local_reputation` | `flag.local_reputation` |
| Control-plane projection | `experimental.control_plane_projection` | `flag.control_plane_projection` |

All experimental flags default to `false`/`off`.

## Metric Event

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

Rules:

- `stage` is `local`, `shadow`, `canary`, `limited`, or `default`.
- Metric labels use bounded enums or hashes. Put high-cardinality detail in
  redacted artifacts, not labels.
- `quality_pass` may be null for latency/cost-only events.
- `policy_violation=true` is an immediate rollback trigger.

## Feature Exposure Event

```rust
pub struct FeatureExposureEvent {
    pub exposure_id: String,
    pub run_id: String,
    pub turn_id: Option<String>,
    pub feature_flag_id: String,
    pub feature_key: String,
    pub variant: String,
    pub stage: String,
    pub enabled: bool,
    pub timestamp_ms: i64,
}
```

Emit at the caller boundary whenever a feature decision can affect behavior,
including disabled decisions used to prove kill switches.

## Gate Verdict

```rust
pub struct GateVerdict {
    pub verdict_id: String,
    pub run_id: String,
    pub feature: String,
    pub rung: String,
    pub status: String,
    pub duration_ms: u64,
    pub artifact_ref: Option<String>,
    pub redaction_applied: bool,
}
```

`status` is `passed`, `failed`, `blocked`, or `skipped`. Gate artifacts must be
redacted before persistence.

## Signal Record

```rust
pub struct SignalRecord {
    pub signal_id: String,
    pub content_hash: String,
    pub kind: String,
    pub confidence: f64,
    pub taint: Vec<String>,
    pub origin_ids: Vec<String>,
    pub created_at_ms: i64,
    pub metadata: serde_json::Value,
}
```

`confidence` is bounded `0.0..=1.0`. Derived memories keep origin ids and taint
labels so rollback can hide or filter them without deleting data.

## DAG Run Record

```rust
pub struct DagRunRecord {
    pub dag_run_id: String,
    pub run_id: String,
    pub workflow_id: String,
    pub status: String,
    pub started_at_ms: i64,
    pub finished_at_ms: Option<i64>,
    pub node_count: u32,
    pub failed_node_id: Option<String>,
}
```

`status` is `running`, `succeeded`, `failed`, or `cancelled`. Node details may
live in artifacts when the full graph is too large for a DB row.

## Reputation Event

```rust
pub struct ReputationEvent {
    pub reputation_event_id: String,
    pub subject_id: String,
    pub domain: String,
    pub delta: f64,
    pub evidence_ref: String,
    pub timestamp_ms: i64,
}
```

Keep reputation local-first. Chain publishing, if ever enabled, is a separate
flagged adapter and must not be required for core trust decisions.

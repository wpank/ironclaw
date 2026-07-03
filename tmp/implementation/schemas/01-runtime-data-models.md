# Runtime Data Models

These are IronClaw-native schemas for implementing the concepts described in the
numbered analysis files. They are meant to be copied into real Rust modules only
after adapting names to the owning IronClaw subsystem.

## 1. Feature Keys And Variants

Use stable feature keys for flags, metrics, tests, and rollout dashboards.

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentalFeature {
    HdcMemorySearch,
    SignalContentAddressing,
    CascadeRouter,
    ProgressiveGates,
    ProviderConductor,
    DreamConsolidation,
    DagWorkflowRunner,
    EventReplay,
    WorkspaceCodeSearch,
    ExtensionHooks,
    ControlPlaneProjection,
    LocalReputationLedger,
    CognitiveSpeeds,
    FullDreamConsolidation,
    PromptComposition,
    AffectEngine,
    SwarmCoordination,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FeatureVariant {
    pub feature: ExperimentalFeature,
    pub variant: String,
    pub enabled: bool,
    pub reason: String,
}
```

Example:

```json
{
  "feature": "cascade_router",
  "variant": "linucb_shadow",
  "enabled": true,
  "reason": "shadow scoring for eligible low-risk chat turns"
}
```

## 2. Metric Event

Use one metric event shape across microbenchmarks, scenario fixtures, and
production telemetry. This mirrors the canonical contract in
[04-canonical-event-and-persistence-contract.md](04-canonical-event-and-persistence-contract.md).

```rust
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

- `schema_version` starts at `1`; migrations must preserve old records.
- `event_id` is the idempotency key for JSONL import and database writes.
- `cost_microusd` avoids floating point storage drift.
- `quality_pass` is nullable because latency/cost-only events are valid.
- `policy_violation = true` is always a guardrail failure.
- `stage` is one of `local`, `shadow`, `canary`, `limited`, or `default`.

## 3. Gate Verdict

Gate verdicts should be typed enough for the agent, UI, and benchmark reports to
agree on what happened.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateRung {
    Compile,
    Lint,
    UnitTest,
    SymbolCheck,
    GeneratedTest,
    PropertyTest,
    IntegrationTest,
    SecurityReview,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    Passed,
    Failed,
    Skipped,
    TimedOut,
    Blocked,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GateVerdict {
    pub verdict_id: String,
    pub run_id: String,
    pub rung: GateRung,
    pub status: GateStatus,
    pub confidence: f64,
    pub duration_ms: u64,
    pub artifact_refs: Vec<String>,
    pub remediation: Option<String>,
    pub redaction_applied: bool,
}
```

Acceptance checks:

- `confidence` must be clamped to `[0.0, 1.0]`.
- `artifact_refs` must point to bounded, redacted artifacts.
- Failed high-risk gates must block the production caller, not only a helper.

## 4. DAG Run Record

The DAG executor should write a minimal run ledger before any expensive effect.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeState {
    Pending,
    Ready,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Skipped,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DagNodeSnapshot {
    pub node_id: String,
    pub state: NodeState,
    pub started_at_ms: Option<i64>,
    pub finished_at_ms: Option<i64>,
    pub cost_microusd: u64,
    pub error_kind: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DagRunRecord {
    pub run_id: String,
    pub graph_id: String,
    pub owner_turn_id: Option<String>,
    pub variant: String,
    pub nodes: Vec<DagNodeSnapshot>,
    pub total_cost_microusd: u64,
    pub cancelled: bool,
}
```

## 5. Signal Record

Signals are IronClaw memory records with content identity, lineage, decay, and
taint metadata.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignalRecord {
    pub signal_id: String,
    pub content_hash_blake3: String,
    pub workspace_path: String,
    pub title: Option<String>,
    pub content_type: String,
    pub parent_signal_ids: Vec<String>,
    pub confidence: f64,
    pub utility: f64,
    pub novelty: f64,
    pub half_life_seconds: Option<i64>,
    pub taints: Vec<String>,
    pub created_at_ms: i64,
    pub last_accessed_ms: i64,
}
```

Guardrail:

```text
same content_hash_blake3 + same workspace_path -> one canonical SignalRecord
```

## 6. Reputation Event

Keep reputation local first. Chain integration, if any, consumes signed local
events later.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub event_id: String,
    pub actor_id: String,
    pub domain: String,
    pub delta: f64,
    pub evidence_ref: String,
    pub evaluator: String,
    pub created_at_ms: i64,
    pub signature: Option<String>,
}
```

Decay rule:

```rust
pub fn decay_weight(age_seconds: f64, half_life_seconds: f64) -> f64 {
    0.5_f64.powf(age_seconds / half_life_seconds)
}
```

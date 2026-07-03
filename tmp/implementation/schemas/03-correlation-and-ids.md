# Correlation And ID Conventions

Cross-feature analysis only works if metrics, gates, signals, DAG nodes, and
rollout exposure events can be joined without guessing. These conventions define
the identifier chain every implementation should preserve.

## 1. Identifier Chain

```text
workspace_id
  -> session_id
  -> thread_id
  -> turn_id
  -> run_id
  -> node_id/tool_call_id/gate_run_id
  -> metric_event_id/verdict_id/signal_id
```

Required:

- `run_id` on every benchmark, DAG run, gate verdict, and rollout event.
- `turn_id` when the behavior is caused by a user-visible agent turn.
- `feature` and `variant` on every experimental metric.
- `artifact_hash` instead of raw artifact content when possible.

## 2. Suggested Formats

| ID | Format | Example |
|---|---|---|
| `run_id` | `<feature>.<yyyymmddThhmmssZ>.<short_nonce>` | `cascade_router.20260702T120100Z.a13f` |
| `gate_run_id` | `gate.<run_id>.<rung>` | `gate.refactor.20260702T120100Z.a13f.unit_test` |
| `signal_id` | `sig.<blake3_prefix>` | `sig.6fdc2a10b4f8` |
| `metric_event_id` | `met.<run_id>.<seq>` | `met.cascade_router.20260702T120100Z.a13f.0004` |
| `feature_flag_id` | `flag.<feature>` | `flag.progressive_gates` |

## 3. Exposure Event

Every rollout decision should write an exposure event before the feature affects
behavior.

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureExposureEvent {
    pub event_id: String,
    pub run_id: String,
    pub turn_id: Option<String>,
    pub feature_flag_id: String,
    pub feature: String,
    pub variant: String,
    pub stage: String,
    pub enabled: bool,
    pub reason: String,
    pub created_at_ms: i64,
}
```

Example:

```json
{
  "event_id": "met.cascade_router.20260702T120100Z.a13f.exposure",
  "run_id": "cascade_router.20260702T120100Z.a13f",
  "turn_id": "turn_01J...",
  "feature_flag_id": "flag.cascade_router",
  "feature": "cascade_router",
  "variant": "linucb_canary",
  "stage": "canary",
  "enabled": true,
  "reason": "eligible low-risk non-private request",
  "created_at_ms": 1782993660000
}
```

## 4. Join Rules

| Question | Join path |
|---|---|
| Did a gate failure affect quality? | `GateVerdict.run_id -> MetricEvent.run_id` |
| Did a router decision save cost? | `FeatureExposureEvent.run_id -> MetricEvent.run_id` |
| Did a dream memory help later? | `SignalRecord.signal_id -> retrieval event evidence_ref -> later MetricEvent.run_id` |
| Did a DAG node cause a policy issue? | `DagRunRecord.run_id + node_id -> MetricEvent.run_id + error_kind` |
| Did reputation change tool selection? | `ReputationEvent.evidence_ref -> tool_call_id -> MetricEvent.run_id` |

## 5. Cardinality Limits

Do not put unbounded values in metric labels.

Allowed labels:

- feature key
- variant
- stage
- provider family
- gate rung
- status enum
- scenario id

Avoid labels:

- prompt text
- file path with user-specific names
- full model response
- raw command
- raw URL
- memory body


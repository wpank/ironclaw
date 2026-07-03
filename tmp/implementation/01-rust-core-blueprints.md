# Rust Core Blueprints

These sketches describe the narrow Rust primitives worth implementing first.
They are intentionally small; production code should live in the owning module
and be exercised through caller-level tests.

## HDC Fingerprint

Owner: `src/workspace/` or a workspace-owned helper crate.

Use a fixed-width binary vector for approximate memory/code similarity.

```rust
pub struct HdcFingerprint {
    bits: bitvec::vec::BitVec,
}

impl HdcFingerprint {
    pub fn hamming_similarity(&self, other: &Self) -> f64 {
        let len = self.bits.len().min(other.bits.len());
        if len == 0 {
            return 0.0;
        }
        let equal = self
            .bits
            .iter()
            .zip(other.bits.iter())
            .take(len)
            .filter(|(a, b)| a == b)
            .count();
        equal as f64 / len as f64
    }
}
```

Guardrails: no raw memory/file body in metric labels, p95 search latency within
+10%, false duplicate rate <= 2%.

## Content-Addressed Signal

Owner: `src/workspace/` plus DB parity in `src/db/`.

```rust
pub struct SignalRecord {
    pub signal_id: String,
    pub content_hash: String,
    pub kind: SignalKind,
    pub confidence: f64,
    pub taint: Vec<String>,
    pub origin_ids: Vec<String>,
    pub metadata: serde_json::Value,
}
```

Implementation notes:

- Hash normalized content with BLAKE3.
- Store exact identity separately from near-duplicate similarity.
- Preserve origin ids and taint labels for rollback/filtering.
- Do not delete memories during rollback; disable ranking/dedupe use.

## Robust Statistics

Owner: existing estimation/telemetry utility module.

Use medians, p95, MAD, and trimmed mean for heavy-tailed runtime data. Do not
gate rollout on mean-only latency or cost.

```rust
pub struct RobustSummary {
    pub n: usize,
    pub median: f64,
    pub p95: f64,
    pub mad: f64,
}
```

Acceptance: aggregation p95 < 1ms for expected rollout windows and false alert
rate improves on outlier fixtures.

## Cascade Router Core

Owner: `crates/ironclaw_llm/`.

Keep routing authority split:

1. Safety/static policy rejects ineligible providers.
2. Candidate router scores eligible providers in shadow or active mode.
3. Fallback path remains available and tested with the flag off.

Reward shape:

```text
reward = 0.50 * quality_success
       + 0.30 * bounded_cost_savings
       + 0.20 * bounded_latency_score
```

All scores are bounded `0.0..=1.0`. `policy_violation=true` forces reward `0`
and rollback review.

## Provider Conductor Core

Owner: `crates/ironclaw_llm/`.

Use observe mode first. Active mode may bias or pre-trip only after the latency
ramp fixture proves warning lead time without false positives.

Required states:

```rust
pub enum ConductorMode {
    Off,
    Observe,
    Active,
}

pub enum HealthSignal {
    Healthy,
    Degrading,
    PredictedFailure,
}
```

Flag: `experimental.provider_conductor`, default `off`.

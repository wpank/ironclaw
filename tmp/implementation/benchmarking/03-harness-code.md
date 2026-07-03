# Benchmark Harness Code

This is a compact implementation sketch for `ironclaw-bench`. Keep the real
runner small: load a scenario, execute baseline and candidate through adapters,
write JSONL, compare bounded metrics, emit a verdict.

## JSONL Event

```rust
#[derive(Debug, serde::Deserialize)]
pub struct MetricEvent {
    pub event_id: String,
    pub run_id: String,
    pub feature: String,
    pub variant: String,
    pub stage: String,
    pub timestamp_ms: i64,
    pub latency_ms: Option<u64>,
    pub cost_microusd: Option<u64>,
    pub quality_pass: Option<bool>,
    pub policy_violation: bool,
}
```

## Robust Summary

```rust
pub struct Summary {
    pub n: usize,
    pub median: f64,
    pub p95: f64,
}

pub fn summarize(values: &[f64]) -> Option<Summary> {
    if values.is_empty() {
        return None;
    }
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.total_cmp(b));
    let last = v.len() - 1;
    let percentile = |p: f64| v[((last as f64 * p).round() as usize).min(last)];
    Some(Summary {
        n: v.len(),
        median: percentile(0.50),
        p95: percentile(0.95),
    })
}
```

## Guardrail Evaluation

```rust
pub fn evaluate_guardrails(base: &[MetricEvent], cand: &[MetricEvent]) -> Vec<String> {
    let mut failures = Vec::new();
    if cand.iter().any(|e| e.policy_violation) {
        failures.push("candidate emitted a policy violation".to_string());
    }

    let base_pass = pass_rate(base);
    let cand_pass = pass_rate(cand);
    if cand_pass + 0.02 < base_pass {
        failures.push(format!(
            "quality regression too high: baseline={base_pass:.3}, candidate={cand_pass:.3}"
        ));
    }

    if let (Some(a), Some(b)) = (latency_summary(base), latency_summary(cand)) {
        if b.p95 > a.p95 * 1.10 {
            failures.push(format!(
                "p95 latency regression too high: baseline={:.1}ms, candidate={:.1}ms",
                a.p95, b.p95
            ));
        }
    }

    failures
}
```

## Scenario Adapter

```rust
pub trait ScenarioAdapter {
    type Input;
    type Output;

    fn load_input(&self, manifest: &ScenarioManifest) -> anyhow::Result<Self::Input>;
    fn run_baseline(&self, input: &Self::Input) -> anyhow::Result<Self::Output>;
    fn run_candidate(&self, input: &Self::Input) -> anyhow::Result<Self::Output>;
    fn score(&self, output: &Self::Output) -> Vec<MetricEvent>;
}
```

Adapters must call the production boundary under test: provider wrapper,
workspace facade, heartbeat routine, gate caller, or web handler.

## Report Shape

```yaml
feature: cascade_router
scenario: cascade_router.simple_lookup
decision: promote
target:
  median_cost_reduction_pct: 24.3
guardrails:
  quality_delta_pp: 0.0
  p95_latency_delta_pct: 4.4
  policy_violations: 0
artifacts:
  baseline: baseline.jsonl
  candidate: candidate.jsonl
```

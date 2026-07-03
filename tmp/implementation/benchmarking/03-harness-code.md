# Benchmark Harness Code

This file expands the measurement framework into an executable-style harness
that can be adapted into IronClaw tests or local benchmark binaries.

## 1. JSONL Loader

```rust
use serde::{Deserialize, Serialize};
use std::{fs::File, io::{BufRead, BufReader}, path::Path};

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

pub fn load_jsonl(path: &Path) -> anyhow::Result<Vec<MetricEvent>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        events.push(serde_json::from_str::<MetricEvent>(&line)?);
    }

    Ok(events)
}
```

## 2. Robust Summary

```rust
#[derive(Clone, Debug)]
pub struct RobustSummary {
    pub n: usize,
    pub median: f64,
    pub p95: f64,
    pub trimmed_mean: f64,
    pub mad: f64,
}

pub fn summarize(values: &[f64]) -> Option<RobustSummary> {
    if values.is_empty() {
        return None;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let n = sorted.len();
    let median = percentile_sorted(&sorted, 0.50);
    let p95 = percentile_sorted(&sorted, 0.95);
    let trim = (n as f64 * 0.10).floor() as usize;
    let trimmed = if trim * 2 >= n {
        &sorted[..]
    } else {
        &sorted[trim..(n - trim)]
    };
    let trimmed_mean = trimmed.iter().sum::<f64>() / trimmed.len() as f64;

    let mut deviations = sorted
        .iter()
        .map(|v| (v - median).abs())
        .collect::<Vec<_>>();
    deviations.sort_by(|a, b| a.total_cmp(b));
    let mad = percentile_sorted(&deviations, 0.50);

    Some(RobustSummary { n, median, p95, trimmed_mean, mad })
}

fn percentile_sorted(sorted: &[f64], q: f64) -> f64 {
    let idx = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[idx]
}
```

## 3. Guardrail Evaluation

```rust
#[derive(Clone, Debug)]
pub struct GuardrailResult {
    pub passed: bool,
    pub reasons: Vec<String>,
}

pub fn evaluate_guardrails(
    baseline: &[MetricEvent],
    variant: &[MetricEvent],
) -> GuardrailResult {
    let mut reasons = Vec::new();

    let base_quality = pass_rate(baseline);
    let variant_quality = pass_rate(variant);
    if variant_quality + 0.02 < base_quality {
        reasons.push(format!(
            "quality regression too high: baseline={base_quality:.3}, variant={variant_quality:.3}"
        ));
    }

    if variant.iter().any(|e| e.policy_violation) {
        reasons.push("variant produced at least one policy violation".to_string());
    }

    let base_latency = collect_f64(baseline, |e| e.latency_ms.map(|v| v as f64));
    let variant_latency = collect_f64(variant, |e| e.latency_ms.map(|v| v as f64));
    if let (Some(a), Some(b)) = (summarize(&base_latency), summarize(&variant_latency)) {
        if b.p95 > a.p95 * 1.10 {
            reasons.push(format!(
                "p95 latency regression too high: baseline={:.1}ms, variant={:.1}ms",
                a.p95, b.p95
            ));
        }
    }

    GuardrailResult { passed: reasons.is_empty(), reasons }
}

fn pass_rate(events: &[MetricEvent]) -> f64 {
    let judged = events.iter().filter_map(|e| e.quality_pass).collect::<Vec<_>>();
    if judged.is_empty() {
        return 1.0;
    }
    judged.iter().filter(|passed| **passed).count() as f64 / judged.len() as f64
}

fn collect_f64<F>(events: &[MetricEvent], f: F) -> Vec<f64>
where
    F: Fn(&MetricEvent) -> Option<f64>,
{
    events.iter().filter_map(f).collect()
}
```

## 4. Scenario Manifest

```toml
[scenario]
id = "cascade_router.simple_lookup"
feature = "cascade_router"
owner = "crates/ironclaw_llm"
repetitions = 200

[baseline]
variant = "current_static_router"

[candidate]
variant = "linucb_shadow"

[guardrails]
max_quality_regression_pp = 2.0
max_p95_latency_regression_pct = 10.0
forbid_policy_violations = true

[target]
metric = "cost_microusd"
direction = "decrease"
min_relative_improvement_pct = 20.0
```

## 5. Scenario Runner Shape

```rust
pub trait ScenarioRunner {
    type Input;
    type Output;

    fn scenario_id(&self) -> &'static str;
    fn inputs(&self) -> Vec<Self::Input>;
    fn run_baseline(&self, input: &Self::Input) -> anyhow::Result<Self::Output>;
    fn run_candidate(&self, input: &Self::Input) -> anyhow::Result<Self::Output>;
    fn judge(&self, output: &Self::Output) -> anyhow::Result<MetricEvent>;
}
```

For IronClaw, implementations should drive production boundaries:

- LLM routing through the provider facade.
- Memory tests through the memory tools or workspace facade.
- Gate tests through the code-generation caller.
- Control-plane tests through HTTP/SSE/WebSocket handlers.

## 6. Reporting Format

```text
scenario: cascade_router.simple_lookup
baseline: current_static_router
candidate: linucb_shadow
n: 200
cost:
  baseline_median: 1430 microusd
  candidate_median: 820 microusd
  relative_delta: -42.7%
latency:
  baseline_p95: 1810 ms
  candidate_p95: 1890 ms
quality:
  baseline_pass_rate: 0.985
  candidate_pass_rate: 0.980
guardrails:
  status: pass
decision:
  promote_to_canary: true
```

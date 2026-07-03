# Measurement Framework

## Event Schema

Use one event shape across local benchmarks, integration tests, and production
telemetry exports. The source of truth is
[`../schemas/04-canonical-event-and-persistence-contract.md`](../schemas/04-canonical-event-and-persistence-contract.md);
the struct is repeated here so benchmark code can be copied without context
switching.

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

Store as JSONL for local benchmark runs. If promoted to product telemetry, write
through the same durable backend strategy used by IronClaw metrics. Keep raw
prompts, tool outputs, secrets, and user content out of `MetricEvent`; use
artifact references instead.

## Statistical Comparison

For a benchmark comparing baseline `A` to variant `B`:

```rust
pub struct Comparison {
    pub baseline_mean: f64,
    pub variant_mean: f64,
    pub absolute_delta: f64,
    pub relative_delta: f64,
    pub n_baseline: usize,
    pub n_variant: usize,
}

pub fn compare_means(a: &[f64], b: &[f64]) -> Option<Comparison> {
    if a.is_empty() || b.is_empty() {
        return None;
    }
    let mean_a = a.iter().sum::<f64>() / a.len() as f64;
    let mean_b = b.iter().sum::<f64>() / b.len() as f64;
    Some(Comparison {
        baseline_mean: mean_a,
        variant_mean: mean_b,
        absolute_delta: mean_b - mean_a,
        relative_delta: if mean_a.abs() < 1e-12 { 0.0 } else { (mean_b - mean_a) / mean_a },
        n_baseline: a.len(),
        n_variant: b.len(),
    })
}
```

For latency and cost, report median, trimmed mean, p95, and MAD. Mean alone is
not enough because LLM/network measurements are heavy-tailed.

## Statistical Decision Rules

Every benchmark result has one of three outcomes:

| Outcome | Meaning |
|---|---|
| `promote` | target metric improved and all guardrails passed |
| `hold` | result is inconclusive or sample size is too small |
| `rollback` | a guardrail failed or a safety event occurred |

Minimum sample sizes:

| Benchmark type | Minimum `n` |
|---|---:|
| pure microbenchmark | Criterion default plus stable confidence interval |
| hermetic scenario | 50 paired cases |
| model/provider scenario | 200 eligible events |
| canary production window | 200 eligible events or 7 days for low-volume flows |

Comparison rules:

- Prefer paired comparisons when baseline and candidate can run on the same
  input fixture.
- Use bootstrap confidence intervals for medians, p95, pass-rate deltas, and
  cost deltas.
- Treat heavy-tailed latency as p50/p95/MAD, not mean-only.
- If more than five feature variants are compared in one run, report a
  multiple-comparison correction or mark secondary comparisons exploratory.
- Retry flaky infrastructure failures once; retrying quality failures is not
  allowed unless the production caller would retry the same way.

Promotion rule:

```text
promote if:
  target metric improves by the stated threshold
  and 95% bootstrap CI does not cross the no-improvement line
  and every guardrail is green
  and no policy violation occurred
```

## Criterion Microbench Pattern

Use Criterion for pure algorithms: HDC comparison, robust stats, LinUCB scoring,
condition evaluation, RRF fusion.

```rust
use criterion::{criterion_group, criterion_main, Criterion, black_box};

fn bench_hdc_similarity(c: &mut Criterion) {
    let a = HdcVector::seeded(1, 42);
    let b = HdcVector::seeded(1, 43);
    c.bench_function("hdc_similarity_10240_bits", |bencher| {
        bencher.iter(|| black_box(a).similarity(black_box(b)));
    });
}

criterion_group!(benches, bench_hdc_similarity);
criterion_main!(benches);
```

Acceptance examples:

| Algorithm | Target |
|---|---|
| HDC similarity | scan 100k vectors in under 10ms on a recorded developer-laptop baseline; report hardware and vector count |
| RRF fusion | merge 3 lists of 1000 results in under 2ms |
| LinUCB score | score 8 arms x 18 dims in under 1ms |
| Gate condition eval | evaluate 10k JSON path predicates in under 50ms |

These are engineering budgets for initial implementations, not general
performance claims. Record CPU, build profile, corpus size, and fixture seed in
the benchmark artifact.

## Scenario Benchmark Pattern

Use hermetic scenarios for features that call real IronClaw boundaries.

```text
scenario_id: cascade.simple_lookup
input: "What is the local time in Berlin?"
baseline: current SmartRoutingProvider
variant: SmartRoutingProvider + LinUCB
quality_oracle: exact answer contains timezone-adjusted time shape
cost_guardrail: variant cost <= 50% baseline
latency_guardrail: variant p95 <= baseline p95 + 10%
```

Do not benchmark these through helper functions only. Drive the same boundary a
user or agent would drive: LLM provider wrapper, memory facade, ToolDispatcher,
web handler, or routine engine.

## A/B Rollout Quantification

Use fixed rollout stages:

| Stage | Traffic | Exit criteria |
|---|---:|---|
| Shadow | 0% decisions, score only | no panics, state persists, decisions explainable |
| Canary | 5% eligible requests | guardrails green for 200+ requests |
| Limited | 25% eligible requests | cost/latency win with no quality regression |
| Default | 100% eligible requests | rollback switch tested |

For local-only features, replace traffic with fixture count.

# Rollout Metrics

This file defines the concrete metrics and windows behind phrases such as
"guardrails green", "canary passed", and "rollback trigger fired".

## 1. Artifact Naming

Every benchmark or rollout comparison should produce this directory shape:

```text
artifacts/benchmarks/<feature>/<scenario>/<run_id>/
  manifest.toml
  baseline.jsonl
  candidate.jsonl
  comparison.json
  summary.md
  fixtures/
  logs/
```

Rules:

- `run_id` should include UTC date and a short monotonic suffix.
- `baseline.jsonl` and `candidate.jsonl` use the same `MetricEvent` schema.
- `comparison.json` is machine-readable and should not contain raw prompts.
- `summary.md` is the human-readable review artifact.

## 2. Shadow Metrics

Shadow mode means the feature computes a decision but cannot affect the user
path.

Required metrics:

| Metric | Formula | Required window |
|---|---|---|
| `shadow_decision_count` | number of candidate decisions scored | 200+ events or full fixture set |
| `decision_agreement_rate` | candidate decision equals baseline decision / scored decisions | report only |
| `would_have_saved_cost_pct` | `(baseline_cost - candidate_cost) / baseline_cost` | report only |
| `would_have_changed_quality_count` | candidate decisions with different quality oracle result | must be reviewed before canary |
| `shadow_error_count` | panics, serialization failures, missing state | must be zero |

Exit to canary:

```text
shadow_error_count == 0
and no policy_violation events
and every changed decision has an explainable reason
```

## 3. Canary Dashboard

Canary mode affects a limited cohort.

Required dashboard rows:

| Row | Green | Yellow | Red |
|---|---:|---:|---:|
| quality pass rate delta | >= -1pp | -1pp to -2pp | < -2pp |
| p95 latency delta | <= +5% | +5% to +10% | > +10% |
| cost delta for cost features | <= -10% | -10% to 0% | > 0% |
| fallback rate | <= baseline +2pp | +2pp to +5pp | > +5pp |
| policy violations | 0 | 0 | > 0 |
| approval bypasses | 0 | 0 | > 0 |

Canary minimum:

```text
200 eligible events
or 7 days of low-volume production traffic
or full hermetic fixture suite for local-only features
```

## 4. Rollback Triggers

Immediate rollback:

- Any policy violation.
- Any secret leak in a metric, artifact, memory, or event.
- Any auth/origin/rate-limit/body-limit regression.
- Any production panic introduced by the feature.

Metric rollback:

```text
quality_pass_rate_delta < -2 percentage points
or p95_latency_delta > +10%
or fallback_rate_delta > +5 percentage points
or cost_delta > 0 for a cost-saving feature
or false_block_rate > 5% for a blocking feature
```

## 5. Owner Review Cadence

| Stage | Review cadence | Required reviewer |
|---|---|---|
| Local fixture | every run | feature owner |
| Shadow | every 200 events | feature owner |
| Canary | daily | feature owner + subsystem owner |
| Limited default | twice weekly | subsystem owner |
| Full default | weekly until stable | normal operations |

Security-sensitive features require security review before canary.

## 6. Privacy And Redaction

Telemetry may include:

- feature key
- variant
- model family
- status enum
- latency/cost/token counts
- gate rung/status
- artifact hash

Telemetry must not include by default:

- full prompts
- raw user messages
- command output
- secrets
- webhook payloads
- full memory body

If raw content is necessary for a local benchmark, keep it in `fixtures/` and do
not promote that artifact to production telemetry.


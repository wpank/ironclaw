# Rollout Metrics

Rollout metrics answer whether a feature can leave observe-only mode. They also
define when to hold or rollback.

## Artifact Naming

```text
target/rollouts/<feature>/<stage>/<run_id>/
  manifest.yaml
  exposures.jsonl
  baseline.jsonl
  candidate.jsonl
  verdict.json
  report.md
```

## Shadow Metrics

Required for every shadow run:

| Metric | Formula | Bound |
| --- | --- | --- |
| `decision_agreement_rate` | candidate decision equals baseline / scored decisions | `0.0..=1.0` |
| `would_have_saved_cost_pct` | `(baseline_cost - candidate_cost) / baseline_cost` | `-100..=100` |
| `p95_latency_delta_pct` | candidate p95 vs baseline p95 | rollback if `> 10` |
| `quality_delta_pp` | candidate pass rate - baseline pass rate | rollback if `< -2` |
| `policy_violation_count` | candidate policy violations | rollback if `> 0` |

Shadow candidates do not control behavior.

## Canary Dashboard

| Signal | Promote | Hold | Rollback |
| --- | --- | --- | --- |
| quality delta | `>= -1pp` | `-1pp..-2pp` | `< -2pp` |
| p95 latency delta | `<= +5%` | `+5%..+10%` | `> +10%` |
| fallback rate delta | `<= +2pp` | `+2pp..+5pp` | `> +5pp` |
| policy/auth/secret issue | `0` | n/a | `> 0` |

## Rollback Triggers

- Any secret, prompt, file body, private path, or token leak in metrics/artifacts.
- Any bearer-token, CORS/origin, webhook-auth, rate-limit, sandbox, or approval
  regression.
- Any DB parity failure between PostgreSQL and libSQL for new persistent data.
- Candidate p95 latency regression above 10% for two windows.
- Candidate quality regression below -2pp for one statistically meaningful
  window.

## Review Cadence

| Stage | Review |
| --- | --- |
| `local` | before merge |
| `shadow` | daily while collecting |
| `canary` | daily for first week |
| `limited` | twice weekly |
| `default` | weekly until flag cleanup |

## Privacy

Telemetry may include ids, hashes, bounded classes, timestamps, counts, cost,
latency, token counts, and redaction status. It must not include raw prompts,
secrets, file bodies, private paths, unredacted stack traces, or arbitrary URLs.

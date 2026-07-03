# Benchmarking And Quantification

Benchmarks decide whether an experimental feature can move from local/shadow to
canary or default. A feature is not better unless it improves one target metric
and keeps every guardrail inside budget.

| File | Purpose |
| --- | --- |
| `01-measurement-framework.md` | Metric schema, comparison rules, and rollout decisions |
| `02-feature-playbooks.md` | Per-feature targets and guardrails |
| `03-harness-code.md` | Minimal runner and report shapes |
| `04-rollout-metrics.md` | Shadow/canary windows, dashboards, rollback triggers |
| `05-runner-contract.md` | CLI contract for manifests, JSONL events, reports, and CI |
| `scenarios/` | YAML scenario fixtures |

## Standard Metrics

- Quality: pass rate, policy violation count, oracle disagreement.
- Cost: median and p95 `cost_microusd` per request or run.
- Latency: p50, p95, timeout rate.
- Reliability: fallback rate, retry count, cancellation success.
- Safety: secret leak count, redaction failures, auth/origin bypasses.

Metric values must be bounded and comparable between baseline and candidate.
Raw prompts, source bodies, secrets, and private paths stay out of JSONL metrics.

## Minimum Rule

```yaml
target: candidate improves the declared primary metric
guardrails:
  quality: no worse than -2pp unless explicitly tighter
  latency: p95 no worse than +10%
  cost: no unexpected increase when cost is a guardrail
  safety: zero policy, auth, secret, or redaction regressions
decision: promote only when target and guardrails pass
```

## Artifact Layout

```text
target/benchmarks/<feature>/<run_id>/
  manifest.yaml
  baseline.jsonl
  candidate.jsonl
  report.md
  verdict.json
```

`baseline.jsonl` and `candidate.jsonl` use the metric schema in
`../schemas/04-canonical-event-and-persistence-contract.md`.

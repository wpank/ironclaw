# Benchmarking And Quantification

This folder defines how to measure whether the captured concept proposals
actually help IronClaw. The goal is not to produce flattering numbers. The goal
is to make cost, quality, latency, safety, and maintenance tradeoffs visible.

| File | Contents |
|---|---|
| `01-measurement-framework.md` | Metrics schema, harness patterns, A/B method, statistical rules |
| `02-feature-playbooks.md` | Per-feature benchmark plans and acceptance thresholds |
| `03-harness-code.md` | JSONL loader, robust summaries, guardrail evaluator, scenario runner shape |
| `04-rollout-metrics.md` | Shadow/canary metrics, rollback triggers, artifact naming, review cadence |
| `05-runner-contract.md` | Executable benchmark-runner contract for manifests, adapters, canonical JSONL events, reports, and CI thresholds |
| `scenarios/` | Concrete benchmark fixture manifests |

For document-by-document rollout gates, use
[`../05-per-file-action-matrix.md`](../05-per-file-action-matrix.md).
For event shape and persistence guarantees, use
[`../schemas/04-canonical-event-and-persistence-contract.md`](../schemas/04-canonical-event-and-persistence-contract.md).

## Standard Metrics

Every feature should report:

- Quality: pass rate, user correction rate, gate success, retrieval relevance.
- Cost: input tokens, output tokens, USD/request, background USD/day.
- Latency: p50, p95, p99, timeout rate.
- Reliability: error rate, retry rate, fallback rate.
- Safety: approvals requested, denied actions, policy violations, secret leaks.
- Drift: score deltas from baseline and confidence intervals.

## Minimum Rule

A feature is not "better" unless it improves at least one target metric without
regressing a guardrail metric beyond its budget.

Example:

```text
Cascade Router target:
  improve: median cost/request -20% or better on eligible low-risk cases
  guardrails: quality pass rate no worse than -2pp, p95 latency no worse than +10%
```

## Artifact Naming

Use a stable artifact layout for every comparison:

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

The `baseline.jsonl` and `candidate.jsonl` files should use the same metric
schema so comparisons can be recomputed after the run.

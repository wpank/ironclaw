# Benchmark Runner Contract

`ironclaw-bench` is a thin local runner around scenario fixtures. It should be
deterministic by default and require explicit opt-in for live providers.

## Commands

```bash
ironclaw-bench validate tmp/implementation/benchmarking/scenarios
ironclaw-bench run tmp/implementation/benchmarking/scenarios/cascade-router.yaml \
  --out target/benchmarks/cascade_router/run.local
ironclaw-bench compare \
  --baseline target/benchmarks/cascade_router/run.local/baseline.jsonl \
  --candidate target/benchmarks/cascade_router/run.local/candidate.jsonl
```

## Required Outputs

```text
manifest.yaml
baseline.jsonl
candidate.jsonl
verdict.json
report.md
```

`baseline.jsonl` and `candidate.jsonl` use
`../schemas/04-canonical-event-and-persistence-contract.md`.

## JSONL Row

```json
{
  "event_id": "met.run.bench.20260703T101500Z.a13f.0001",
  "run_id": "run.bench.20260703T101500Z.a13f",
  "feature": "cascade_router",
  "feature_flag_id": "flag.cascade_router",
  "variant": "candidate",
  "stage": "local",
  "timestamp_ms": 1783073700000,
  "latency_ms": 412,
  "cost_microusd": 120,
  "quality_pass": true,
  "policy_violation": false,
  "metadata": {
    "fixture_id": "cascade_router.simple_lookup",
    "redaction": "not_required"
  }
}
```

## Exit Codes

| Code | Meaning |
| --- | --- |
| 0 | validation/comparison passed |
| 1 | guardrail failed |
| 2 | invalid manifest or missing required field |
| 3 | runner or adapter error |
| 4 | privacy/security violation |

## Oracle Types

Supported local oracles:

- `deterministic_answer_shape`
- `retrieval_relevance`
- `defect_presence`
- `avoided_degradation`
- `later_retrieval_usefulness`
- `expected_file_and_symbol`

## CI Target

Run `validate` on every fixture in PR checks. Run `run` only for hermetic
fixtures unless the PR explicitly opts into a slower benchmark job.

# Benchmark Runner Contract

The scenario fixtures are intentionally portable. This document defines the
runner contract that turns them into executable evidence.

## 1. Command Shape

```text
ironclaw-bench run \
  --scenario tmp/benchmarking/scenarios/cascade-router.yaml \
  --baseline current_static_router \
  --candidate linucb_shadow \
  --out artifacts/benchmarks/cascade_router/simple_lookup/<run_id>
```

Required subcommands:

| Command | Purpose |
|---|---|
| `validate` | parse fixture, check required fields, no side effects |
| `run` | execute baseline and candidate |
| `compare` | produce comparison.json and summary.md from JSONL |
| `redact` | verify artifact privacy rules |

## 2. Artifact Layout

```text
artifacts/benchmarks/<feature>/<scenario>/<run_id>/
  manifest.yaml
  baseline.jsonl
  candidate.jsonl
  comparison.json
  summary.md
  stderr.log
  fixtures/
```

## 3. JSONL Output

Each line must be the canonical event shape from
[`../schemas/04-canonical-event-and-persistence-contract.md`](../schemas/04-canonical-event-and-persistence-contract.md).

Minimum fields:

```json
{
  "schema_version": 1,
  "event_id": "met.cascade_router.0001",
  "run_id": "cascade_router.20260702T120100Z.a13f",
  "thread_id": null,
  "turn_id": "turn.fixture.0001",
  "tool_call_id": null,
  "feature_flag_id": "flag.cascade_router",
  "feature": "cascade_router",
  "variant": "linucb_shadow",
  "scenario": "cascade_router.simple_lookup",
  "stage": "shadow",
  "timestamp_ms": 1782993660000,
  "latency_ms": 412,
  "input_tokens": 118,
  "output_tokens": 64,
  "cost_microusd": 120,
  "quality_pass": true,
  "score": 0.94,
  "error_kind": null,
  "fallback_used": false,
  "approval_required": false,
  "policy_violation": false,
  "redaction_applied": true
}
```

## 4. Oracle Types

| Oracle | Meaning |
|---|---|
| `deterministic_answer_shape` | answer contains required strings or structured fields |
| `retrieval_relevance` | expected record appears in top-k |
| `defect_presence` | candidate catches known seeded defect |
| `avoided_degradation` | provider switch avoids known bad sequence |
| `later_retrieval_usefulness` | derived memory helps later fixture |
| `expected_file_and_symbol` | code search returns expected file/symbol |

## 5. Failure Modes

| Failure | Outcome |
|---|---|
| fixture parse error | runner exits before side effects |
| oracle missing | result is invalid, not failed |
| policy violation | automatic rollback recommendation |
| baseline failure | fixture invalid unless baseline failure is expected |
| candidate infrastructure failure | rollback recommendation |
| insufficient sample size | hold, not promote |

## 6. CI Target

```text
cargo test benchmark_fixture_validation
ironclaw-bench validate tmp/benchmarking/scenarios/*.yaml
ironclaw-bench compare --baseline baseline.jsonl --candidate candidate.jsonl
```

The runner itself can be implemented later. The contract above is enough to
ensure that scenario files, schemas, and rollout docs agree.

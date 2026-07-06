# Correlation And ID Conventions

Cross-feature analysis only works when every event can be joined without
high-cardinality labels. Use stable ids and keep raw content outside metrics.

## Identifier Chain

```text
run_id
  -> turn_id
  -> feature_key + variant + stage
  -> exposure_id
  -> metric_event_id | verdict_id | signal_id | dag_run_id | reputation_event_id
```

Required fields on every experimental event:

- `run_id`
- `feature` or `feature_key`
- `variant`
- `stage`
- timestamp in milliseconds

## Suggested Formats

| Id | Format | Example |
| --- | --- | --- |
| `run_id` | `run.<kind>.<utc>.<short>` | `run.bench.20260703T101500Z.a13f` |
| `turn_id` | existing session/turn id | `turn.01HZX...` |
| `metric_event_id` | `met.<run_id>.<seq>` | `met.run.bench.20260703T101500Z.a13f.0004` |
| `feature_flag_id` | `flag.<feature>` | `flag.progressive_gates` |
| `variant` | bounded enum | `baseline`, `shadow`, `candidate` |

## Exposure Event Example

```json
{
  "exposure_id": "exp.run.bench.20260703T101500Z.a13f.0001",
  "run_id": "run.bench.20260703T101500Z.a13f",
  "turn_id": "turn.01HZXEXAMPLE",
  "feature_flag_id": "flag.cascade_router",
  "feature_key": "cascade_router",
  "variant": "shadow",
  "stage": "local",
  "enabled": false,
  "timestamp_ms": 1783073700000
}
```

## Join Rules

- Join rollout metrics by `run_id`, `feature`, `variant`, and `stage`.
- Join caller-level tests by `run_id` and exact fixture id.
- Join feature exposure to metrics by `run_id` plus `feature_flag_id`.
- Join derived memories to origin material through `origin_ids`, never through
  raw transcript text.

## Cardinality Limits

Do not put user ids, prompts, private paths, URLs, file names, stack traces, or
unbounded provider error strings in metric labels. Use redacted artifacts and
bounded classifications instead.

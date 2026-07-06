# Feature Rollout Runbooks

All runbooks use the same stages:

```text
off -> local fixture -> shadow -> canary -> limited -> default -> flag cleanup
```

Every stage emits feature exposure events and metric events. Shadow never
changes user-visible behavior.

## HDC Memory Search

Flag: `experimental.hdc_memory_search`, default `off`.

Config:

```toml
[experimental.hdc_memory_search]
enabled = false
mode = "off" # off | shadow | canary | default
```

Promote when top-5 relevance improves by at least 10pp and p95 search latency
stays within +10%. Roll back by setting `enabled=false`; memory search falls back
to the existing full-text/vector path.

## Cascade Router

Flag: `experimental.cascade_router`, default `off`.

Runbook:

1. Local: run `../benchmarking/scenarios/cascade-router.yaml`.
2. Shadow: log candidate provider and reward while static routing acts.
3. Canary: allow candidate only for low-risk eligible requests.
4. Limited/default: expand only if cost improves and quality/safety guardrails
   pass.

Rollback: set `enabled=false`; the static provider router remains
authoritative.

## Progressive Gates

Flag: `experimental.progressive_gates`, default `off`.

Runbook:

1. Local: run `../benchmarking/scenarios/gate-pipeline.yaml`.
2. Shadow: run gates after existing validation and record verdicts only.
3. Canary: block only code-generation/tool-building flows covered by tests.
4. Default: keep rung selection bounded by task complexity.

Rollback: disable the flag; the caller skips `GatePipeline` construction.

## Dream Consolidation

Flag: `experimental.dream_consolidation`, default `off`.

Runbook:

1. Local: run `../benchmarking/scenarios/dream-consolidation.yaml` with a
   deterministic LLM fixture.
2. Shadow: generate redacted derived memories but hide them from retrieval.
3. Canary: expose promoted memories only above confidence threshold.
4. Limited/default: enforce background budget and taint propagation.

Rollback: disable the scheduled job and filter derived memory tags from search.
Do not delete memories; preserve origin ids for audit.

## Provider Conductor

Flag: `experimental.provider_conductor`, default `off`.

Runbook:

1. Local: run `../benchmarking/scenarios/provider-degradation.yaml`.
2. Observe: record health signals without changing circuit state.
3. Canary: bias away from predicted failures for eligible providers.
4. Default: allow active pre-trip only after false-positive and oscillation
   guardrails pass.

Rollback: set mode to `observe` or `off`; reactive circuit breaker behavior
remains authoritative.

## Signal Content Addressing

Flag: `experimental.signal_records`, default `off`.

Runbook:

1. Local: run `../benchmarking/scenarios/memory-dedup.yaml`.
2. Shadow: write candidate signal rows without changing retrieval ranking.
3. Canary: enable soft dedupe for eligible workspaces.
4. Default: keep exact hash identity and near-duplicate decisions auditable.

Rollback: disable dedupe/ranking use; persisted signal rows remain inert.

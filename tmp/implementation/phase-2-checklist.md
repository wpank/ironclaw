# Phase 2 Implementation Checklist

Phase 2 moves from primitives into core runtime enhancements. Each item is
independently shippable and must have a fixture, caller-level test, and rollback
switch before canary.

## Items

| Item | Owner | Flag | Fixture | Primary metric |
| --- | --- | --- | --- | --- |
| Cascade router | `crates/ironclaw_llm/` | `experimental.cascade_router` | `benchmarking/scenarios/cascade-router.yaml` | cost/request down >= 20% |
| HDC memory search | `src/workspace/` | `experimental.hdc_memory_search` | memory/search corpus | top-5 relevance up >= 10pp |
| Progressive gates | `src/tools/` | `experimental.progressive_gates` | `benchmarking/scenarios/gate-pipeline.yaml` | escaped defects down >= 20% |
| Dream consolidation lite | `src/agent/`, workspace | `experimental.dream_consolidation` | `benchmarking/scenarios/dream-consolidation.yaml` | useful-memory hits up >= 10pp |
| Provider conductor | `crates/ironclaw_llm/` | `experimental.provider_conductor` | `benchmarking/scenarios/provider-degradation.yaml` | degraded spend down >= 50% |

## Cascade Router

Implementation checklist:

- Add shadow router behind `experimental.cascade_router`.
- Keep static/safety routing authoritative when disabled or in shadow.
- Emit `FeatureExposureEvent` and `MetricEvent`.
- Persist only bounded routing episode data if DB storage is needed.
- Test through provider factory/wrapper with fixture providers.
- Roll back by disabling the flag.

Guardrails: quality no worse than -2pp, p95 latency no worse than +10%, zero
policy or safety bypasses.

## HDC Memory Search

Implementation checklist:

- Add HDC candidate generation without replacing baseline search.
- Keep private memory bodies out of metrics.
- Gate ranking use behind `experimental.hdc_memory_search`.
- Test through workspace search/write facade.
- Roll back by ignoring HDC candidates.

Guardrails: false duplicate <= 2%, p95 search latency <= +10%, no private
memory leakage.

## Progressive Gates

Implementation checklist:

- Add rung selection for parse/build/lint/unit/security checks.
- Redact command output before persistence.
- Invoke gates only from the real generated-code/tool-building caller.
- Feature flag off skips pipeline construction.
- Roll back by disabling `experimental.progressive_gates`.

Guardrails: false block <= 5%, wall clock <= 120s for the fixture tier, no
secret leak in artifacts.

## Dream Consolidation Lite

Implementation checklist:

- Run only from heartbeat/routine path when flag is enabled.
- Use deterministic local fixtures for tests.
- Write derived memories with confidence, taint, and origin ids.
- Keep derived memories hidden until explicitly promoted.
- Enforce background cost and call-count budgets.
- Roll back by disabling the job and filtering derived rows from retrieval.

Guardrails: background cost <= 25,000 microusd per fixture cycle, max 3 promoted
memories, no sensitive data leak.

## Provider Conductor

Implementation checklist:

- Add observe-mode watcher beside existing circuit breaker.
- Feed latency/error/cost/success observations from provider wrapper.
- Active mode cannot bypass safety routing.
- Test latency ramp through circuit breaker wrapper.
- Roll back by setting mode `off` or `observe`.

Guardrails: healthy-provider false positive <= 3%, oscillation <= 1, p95 latency
regression <= 10%.

## Cross-Item Gate

Before Phase 2 is considered complete:

- All flags default off.
- All flag-off paths prove baseline behavior.
- All YAML fixtures parse and required fields are present.
- All benchmark targets and guardrails are bounded.
- DB changes have PostgreSQL/libSQL parity tests.
- Security review covers auth, secrets, sandboxing, approvals, and outbound
  network behavior where touched.

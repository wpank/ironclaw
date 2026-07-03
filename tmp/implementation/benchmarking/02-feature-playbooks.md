# Feature Benchmark Playbooks

Each playbook names one primary target, guardrails, a fixture, and the caller
boundary that must be exercised. Helper-only tests are not enough.

| Feature | Fixture | Caller boundary | Primary metric | Guardrails |
| --- | --- | --- | --- | --- |
| Signal records / memory dedup | `scenarios/memory-dedup.yaml` | memory write/search tool facade | duplicate storage down >= 30% | top-5 relevance >= 0.95, false duplicate rate <= 2%, write p95 <= 100ms |
| HDC memory search | `scenarios/memory-dedup.yaml` plus search corpus | workspace search facade | top-5 relevance up >= 10pp | p95 search latency <= +10%, no private memory leakage |
| Cascade router | `scenarios/cascade-router.yaml` | LLM provider factory/wrapper | median cost/request down >= 20% | quality pass rate no worse than -2pp, p95 latency <= +10%, zero safety bypasses |
| Progressive gates | `scenarios/gate-pipeline.yaml` | tool/code-generation caller | escaped defect rate down >= 20% | false block rate <= 5%, wall clock <= 120s, artifacts redacted |
| Provider conductor | `scenarios/provider-degradation.yaml` | provider wrapper + circuit breaker | degraded-provider spend down >= 50% | false positive rate <= 3%, oscillation <= 1, healthy p95 latency <= +10% |
| Dream consolidation | `scenarios/dream-consolidation.yaml` | heartbeat/routine -> memory facade | useful-memory hit rate up >= 10pp | background cost <= 25,000 microusd, no sensitive leak, max 3 promoted memories |
| Workspace code search | `scenarios/workspace-code-search.yaml` | workspace indexing/search facade | top-5 symbol recall up >= 15pp | index time <= 250ms/KLOC, memory growth <= 20%, no private paths in metrics |
| Control-plane projection | custom gateway fixture | HTTP/SSE/WebSocket handlers | reconnect event loss = 0 | bearer/CORS/rate limits unchanged, route p95 inside existing budget |

## Promotion Gate

```text
feature flag defaults off
caller-level test passes
scenario fixture parses and runs
target metric improves
all guardrails pass
rollback switch tested
docs updated for changed behavior
```

## Rollback Gate

Rollback immediately on any secret leak, auth/origin regression, policy
violation, unredacted artifact, DB parity failure, or sustained p95 latency
regression above 10%.

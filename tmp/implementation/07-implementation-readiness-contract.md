# Implementation Readiness Contract

This is the auditable bridge from the analysis docs to IronClaw implementation.
No feature should be considered implementation-ready until its row has concrete
answers for owner, entrypoint, schema, migration, flag, caller-level test,
benchmark, rollout metric, rollback switch, and security review.

## 1. Readiness Definition

```text
ready =
  owner module is named
  and caller boundary is named
  and schema/data shape is named
  and DB parity impact is known
  and feature flag exists or is explicitly unnecessary
  and caller-level test is specified
  and benchmark scenario exists
  and rollout metric exists
  and rollback switch is defined
  and security review area is listed
```

## 2. Feature Contract Table

| Feature | Owner module | Caller boundary | Schema | DB impact | Flag | Caller-level test | Benchmark | Rollout metric | Rollback | Security review |
|---|---|---|---|---|---|---|---|---|---|---|
| HDC memory search | `src/workspace/` or memory facade crate | `memory_search`, `memory_write` | `SignalRecord`, HDC bytes | fingerprint side table or column, both DBs | `experimental.hdc_memory_search` | duplicate/paraphrase memory fixture through memory tool | `benchmarking/scenarios/memory-dedup.yaml` | top-5 relevance, p95 search latency | set mode off, ignore HDC candidates | memory privacy, redaction, false merge |
| Signal content addressing | `src/workspace/`, `src/db/` | memory write/read/search facade | `SignalRecord` | `signal_records` table, both DBs | `experimental.signal_records` | exact and near-duplicate writes through memory tool | `memory-dedup.yaml` | duplicate storage reduction | `dedupe_mode=observe` | taint, sensitive memory, data retention |
| Cascade router | `crates/ironclaw_llm/` | `SmartRoutingProvider` or provider factory | `MetricEvent`, `FeatureExposureEvent` | routing episodes optional, metrics/exposure tables | `experimental.cascade_router` | high-risk request bypasses bandit | `cascade-router.yaml` | cost/request, quality delta | disable flag, static router authoritative | provider auth, data residency, safety routing |
| Progressive gates | planned `crates/ironclaw_gate/` | code-generation/tool-building caller | `GateVerdict` | verdict table optional | `experimental.progressive_gates` | compile failure blocks through caller | `gate-pipeline.yaml` | escaped defect rate, false block rate | report-only or disabled | sandbox, approvals, artifact redaction |
| Provider conductor | `crates/ironclaw_llm/` | LLM provider wrapper and circuit breaker | `MetricEvent`, health snapshot | metrics/exposure only | `experimental.provider_conductor` | latency ramp biases routing through wrapper | `provider-degradation.yaml` | degraded-provider spend avoided | mode observe | provider credentials, fallback safety |
| Dream consolidation | `src/agent/heartbeat.rs`, workspace memory | heartbeat/routine engine -> memory facade | `SignalRecord`, `MetricEvent` | derived memory writes, both DBs | `experimental.dream_consolidation` | heartbeat fixture writes tainted derived memory | `dream-consolidation.yaml` | useful-memory hit rate, background spend | disable scheduled job | sensitive data, background budget |
| DAG runner | product workflow/runtime runner | plan parsing -> runner -> executor | `DagRunRecord` | DAG run table if persisted | `experimental.dag_workflow_runner` | representative workflow reaches same final state as serial | custom DAG fixture | wall-clock reduction, skipped required nodes | serial fallback | tool side effects, cancellation |
| EventBus replay | web gateway/events | SSE/WebSocket reconnect handler | event cursor/exposure event | event persistence optional | `experimental.event_replay` | reconnect cursor receives no duplicates | control-plane fixture | event loss count | snapshot refresh fallback | auth, origin, cross-session isolation |
| Code intelligence | `src/workspace/` indexing | workspace search/index facade | symbol/index snapshot | index tables optional | `experimental.workspace_code_search` | fixture repo search through public search boundary | `workspace-code-search.yaml` | top-5 symbol recall | FTS/vector-only fallback | path privacy, source redaction |
| Plugin hooks | extension registry | install/activate/remove/trigger lifecycle | manifest + permission event | none unless audit persisted | `experimental.extension_hooks` | denied network plugin fails closed | plugin fixture | permission escape count | disable extension/trigger | sandbox, network allowlist |
| Local reputation | local trust ledger | tool/model/agent outcome event path | `ReputationEvent` | reputation tables, both DBs | `experimental.local_reputation` | failed tool outcome updates score | reputation fixture | bad-tool selection down | ignore reputation in selection | evidence integrity, appeal path |
| Control-plane projection | web gateway | HTTP route, SSE, WebSocket handlers | `DashboardProjection`, event cursor | projection cache optional | `experimental.control_plane_projection` | auth + reconnect fixture | control-plane event fixture | projection p95, auth fail-open | route disabled, baseline history fetch | bearer auth, CORS, body/rate limits |
| Feature flags | settings/config facade | feature evaluation call site | `FeatureFlag`, `FeatureExposureEvent` | exposure table, both DBs | always required for experiments | kill switch disables feature at caller | rollout fixture | exposure count, disabled effect count | kill switch | config precedence, secrets separation |

## 3. Required Evidence Bundle

```text
feature/
  README.md
  schema.md
  migration-postgres.sql
  migration-libsql.sql
  caller-test.md
  benchmark-summary.md
  rollout-summary.md
  risk-record.md
```

The bundle can be a PR description rather than committed files, but every item
must be answerable.

## 4. Readiness Review Questions

- What side effect does the feature gate?
- Which caller boundary owns that side effect?
- What is the rollback switch and how fast does it take effect?
- What data remains after rollback?
- Which DB backends are affected?
- Which fixture proves the target metric?
- Which guardrail would force rollback?
- Which security invariant could be weakened?

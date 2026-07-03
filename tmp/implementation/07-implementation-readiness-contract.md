# Implementation Readiness Contract

A feature is ready to implement only when this contract has concrete answers.

## Definition

```text
ready =
  owner module known
  and caller boundary known
  and schema/migration impact known
  and runtime flag defaults off
  and caller-level test named
  and benchmark/scenario named
  and rollout metric named
  and rollback switch tested
  and security review scope known
```

## Feature Contract Table

| Feature | Owner | Caller boundary | Schema/DB | Flag | Fixture | Rollback |
| --- | --- | --- | --- | --- | --- | --- |
| Signal records | `src/workspace/`, `src/db/` | memory write/search tools | `SignalRecord`, both DBs | `experimental.signal_records` | `memory-dedup.yaml` | disable dedupe/ranking |
| HDC memory search | `src/workspace/` | workspace search facade | fingerprint side data | `experimental.hdc_memory_search` | `memory-dedup.yaml` | ignore HDC candidates |
| Cascade router | `crates/ironclaw_llm/` | provider factory/wrapper | metric/exposure events | `experimental.cascade_router` | `cascade-router.yaml` | static router authoritative |
| Progressive gates | `src/tools/` | generated-code/tool caller | gate verdicts optional | `experimental.progressive_gates` | `gate-pipeline.yaml` | skip gate pipeline |
| Provider conductor | `crates/ironclaw_llm/` | circuit breaker wrapper | metric/exposure events | `experimental.provider_conductor` | `provider-degradation.yaml` | observe/off mode |
| Dream consolidation | `src/agent/`, workspace | heartbeat -> memory facade | derived signal rows | `experimental.dream_consolidation` | `dream-consolidation.yaml` | disable job, hide derived rows |
| DAG runner | product workflow/runtime | plan -> runner -> executor | optional DAG run table | `experimental.dag_workflow_runner` | custom DAG fixture | serial fallback |
| Event replay | web gateway | HTTP/SSE/WebSocket reconnect | cursor/replay data | `experimental.event_replay` | gateway reconnect fixture | snapshot/history fallback |
| Workspace code search | `src/workspace/` | index/search facade | index snapshot optional | `experimental.workspace_code_search` | `workspace-code-search.yaml` | FTS/vector fallback |
| Local reputation | trust/reputation | outcome event path | reputation tables, both DBs | `experimental.local_reputation` | reputation fixture | ignore reputation in selection |

## Evidence Bundle

- Changed-file list and owner.
- Feature flag inventory entry.
- Caller-level test name.
- Fixture or benchmark report.
- Migration pair and dual-backend test, if persistence changes.
- Rollback note.
- Security review note.

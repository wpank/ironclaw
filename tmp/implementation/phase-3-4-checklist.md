# Phase 3 And 4 Implementation Checklist

Phase 3 changes runtime architecture; Phase 4 adds adaptive or advanced
capabilities. These should not start until the relevant Phase 2 baseline,
fixture, and rollback path exist.

## Phase 3 Items

| Item | Owner | Flag | Dependency | Validation |
| --- | --- | --- | --- | --- |
| DAG workflow runner | product workflow/runtime | `experimental.dag_workflow_runner` | plan parsing and tool dispatcher | final state equals serial baseline; wall clock down >= 15% |
| Event replay | web gateway | `experimental.event_replay` | existing session/event streams | reconnect loses 0 events and leaks no cross-session data |
| Cognitive speed labels | `src/agent/`, router metadata | `experimental.cognitive_speeds` | cascade router consumes tier hints | urgent latency improves; no background starvation |
| Full dream consolidation | `src/agent/`, workspace | `experimental.full_dream_consolidation` | dream consolidation lite | derived memories useful, redacted, budgeted |
| Checkpoints/resume | agent/runtime persistence | `experimental.resumable_checkpoints` | event ids and cancellation | resumed run has no duplicate side effects |

## Phase 4 Items

| Item | Owner | Flag | Dependency | Validation |
| --- | --- | --- | --- | --- |
| Workspace code search | `src/workspace/` | `experimental.workspace_code_search` | index/search facade | top-5 symbol recall up >= 15pp; no private paths in metrics |
| Prompt composition | prompt builder/agent | `experimental.prompt_composition` | stable scoring and metrics | tokens/request down >= 15%; quality no worse than -2pp |
| Affect engine | `src/agent/` | `experimental.affect_engine` | cognitive metadata | safety routing unchanged; update p95 < 1ms |
| Local reputation | trust/reputation + DB | `experimental.local_reputation` | outcome event path | bad-tool selection down; DB parity |
| Optional chain bridge | reputation adapter | `experimental.near_reputation_bridge` | local reputation stable | chain failure degrades to local-only |
| Swarm coordination | agent coordination | `experimental.swarm_coordination` | DAG/checkpoint/cancellation | conflicting edits serialize; throughput improves |

## Required Checklist For Each Item

```text
[ ] owning module and caller boundary named
[ ] flag inventory entry defaults off
[ ] schema and migration impact reviewed
[ ] caller-level or whole-path test added
[ ] benchmark/fixture target and guardrails bounded
[ ] rollout runbook and rollback switch updated
[ ] security review completed for touched auth/secrets/network/sandbox paths
[ ] docs updated when behavior changes
```

## Cross-Layer Requirements

- Subagent or multi-run coordination must use the existing runner/driver/executor
  path; do not add a second agent loop.
- Trusted trigger ingress remains limited to trigger-worker-owned minting and
  private conversation-owned construction.
- Runtime features fail closed when config cannot load.
- Background work has explicit budget, cancellation, and redaction behavior.
- Persistent data can be made inert on rollback without deleting user data.

## Validation Gate

Phase 3/4 work is canary-ready only when:

- feature flag off restores baseline behavior;
- caller-level tests exercise the real side-effect boundary;
- scenario or benchmark evidence passes target and guardrails;
- DB parity exists for every new table or query path;
- rollout docs identify owner, metric window, and rollback command.

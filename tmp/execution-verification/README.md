# Execution and Verification

This directory covers optional execution-control and verification systems for IronClaw. The docs are concise planning notes aligned to current code boundaries, not source-port recipes.

## Documents

| Document | Use It For | First Ship Candidate |
| --- | --- | --- |
| [Conductor Anomaly](conductor-anomaly.md) | Detect stuck work, provider degradation, cost/time pressure, and repeated failures. | Shadow monitors over existing events. |
| [Gate Verification](gate-verification.md) | Progressive validation before accepting tool/code/job outputs. | Caller-level gates around existing tool and Reborn execution paths. |
| [DAG Execution](dag-execution.md) | Declarative bounded workflows and dependency-aware execution. | Read-only workflow planning and tool-cell prototype behind a flag. |
| [Orchestrator Swarm](orchestrator-swarm.md) | Multi-job/subagent coordination, recovery, worktree isolation, and audit. | Scheduler/Reborn orchestration extensions; no second agent loop. |
| [Runtime Infrastructure](runtime-infrastructure.md) | Eventing, cancellation, lifecycle, resource ledger, and dashboard projections. | Shared observability and cancellation primitives. |

## Shared Constraints

- Preserve current entrypoints: `src/agent/`, `src/tools/`, `crates/ironclaw_llm/`, `src/bridge/`, and Reborn crates own their domains.
- Feature behavior changes must be flag-gated and rolled out through shadow/advisory modes before enforcement.
- Side effects must go through existing callers: `ToolDispatcher`, `execute_tool_with_safety`, provider chain, scheduler, bridge adapters, or Reborn runner.
- New persistence requires PostgreSQL and libSQL parity through the shared DB trait.
- Add caller-level tests where the side effect happens. Helper-only tests are not sufficient for gates, routing, auth, approvals, DB writes, or tool execution.

# Orchestrator and Swarm Coordination

Orchestration coordinates multiple jobs, subagents, worktrees, and recovery paths. In IronClaw it must extend the existing scheduler and Reborn execution stack. It must not create another agent loop.

## Current Boundaries

| Concern | Owner |
| --- | --- |
| v1 jobs and subtasks | `src/agent/scheduler.rs`, `src/worker/job.rs`, `src/agent/task.rs` |
| job state | `src/context::JobState`, `ContextManager` |
| Reborn missions/subagents | Reborn runner/driver/executor and composition crates |
| tools | `src/tools/execute.rs`, `src/tools/dispatch.rs` |
| audit | `ActionRecord`, job events, LLM call records |
| web gateway | `src/channels/web/`, `src/bridge/router.rs` |
| worktrees/sandbox | existing workspace/sandbox/container policies |

The AGENTS invariant is strict: subagent spawn creates and wires child runs only. Child planning, execution, tools, gates, retries, checkpointing, and completion go through the existing Reborn path.

## Orchestration Capabilities

| Capability | First Implementation |
| --- | --- |
| Multi-job planning | Build dependency metadata for existing scheduler jobs. |
| Wave scheduling | Start independent jobs only when resource and file-conflict policy allows. |
| Recovery | Classify failed jobs and route through existing retry/repair mechanisms. |
| Worktree isolation | Allocate worktrees through existing workspace/sandbox policy. |
| Audit chain | Link existing audit records; do not duplicate tool audit. |
| Coordination hints | Store advisory state for sibling jobs/subagents. |

## Event Model

Use append-only events for orchestration state:

- plan created,
- dependency added,
- job/run started,
- tool/LLM/gate event reference observed,
- job/run completed or failed,
- recovery decision proposed,
- recovery action taken,
- merge/review accepted.

Events should reference existing records instead of copying large payloads. If event storage is DB-backed, implement both PostgreSQL and libSQL.

## Safety Rules

- Product adapters, host-runtime handlers, and workflow code must not mint trusted inbound requests.
- Orchestrator-initiated tool execution must go through `ToolDispatcher`, existing worker execution, or Reborn effect adapters.
- File-conflict inference is advisory until it has tests against real worktree/sandbox behavior.
- Recovery actions must be idempotent or explicitly guarded by event sequence checks.
- Cross-tenant coordination is forbidden unless the existing scope model explicitly allows it.

## Rollout

| Mode | Behavior |
| --- | --- |
| `off` | Existing scheduler/Reborn behavior only. |
| `shadow` | Build orchestration plan and warnings; do not alter scheduling. |
| `advisory` | Surface plan, conflicts, and recovery suggestions. |
| `enforce` | Apply selected scheduling/recovery policies. |

Start with shadow conflict detection and recovery classification. Scheduling enforcement and speculative execution should be last.

## Tests

- Unit-test dependency graph validation and cycle detection.
- Scheduler-level test: orchestration metadata does not break existing `JobState` transitions.
- Caller-level test: orchestrated tool work still produces `ActionRecord` through the existing path.
- Reborn subagent test: child runs execute through existing runner/driver/executor path.
- Worktree test: conflicting file edits are isolated or serialized according to policy.
- Crash recovery test: replaying orchestration events does not re-run completed side effects.

## Do Not Claim Yet

- central-vs-swarm performance advantages,
- tamper-proof audit beyond what local hash/event checks prove,
- automatic recovery effectiveness,
- safe speculative execution.

Measure those locally after the minimal orchestration layer exists.

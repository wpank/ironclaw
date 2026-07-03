# Runtime Infrastructure

Runtime infrastructure is the shared substrate for cancellation, eventing, lifecycle state, resource accounting, checkpoints, and status projections. IronClaw already has many of these pieces. The task is to consolidate carefully, not to introduce a parallel runtime.

## Current Pieces

| Capability | Existing Home |
| --- | --- |
| Job state | `src/context::JobState` |
| Thread/turn state | `src/agent/session.rs` |
| Scheduler | `src/agent/scheduler.rs` |
| Heartbeat/routines | `src/agent/heartbeat.rs`, `src/agent/routine_engine.rs` |
| Tool execution | `src/tools/execute.rs`, `src/tools/dispatch.rs` |
| LLM resilience | `crates/ironclaw_llm` retry/circuit/failover/cache |
| Cost/resource accounting | `src/agent/cost_guard.rs`, Reborn budget gates |
| Gateway status | `src/channels/web/`, SSE/WebSocket broadcasts |
| Reborn runtime | `crates/ironclaw_engine`, Reborn composition/runtime crates |

## Desired Primitives

| Primitive | Purpose | First Use |
| --- | --- | --- |
| Cancellation tree | Propagate user cancel/shutdown through child work. | Jobs, routines, Reborn runs. |
| Event envelope | Normalize status/audit events without losing source ids. | Gateway/debug projections. |
| Lifecycle state | Make run/job/thread states explicit and inspectable. | Dashboard and recovery. |
| Resource ledger | Track LLM/tool/cost/time usage per scope. | Budget gates and anomaly monitors. |
| Checkpoint envelope | Store resumable state with version and scope. | Reborn/job recovery. |
| Projection store | Build UI/status views from events. | Gateway and operator diagnostics. |

Each primitive should be introduced only where it removes duplication or enables a tested caller-level behavior.

## Eventing

Events should be small, scoped, and reference durable records:

- tenant/user/project/job/thread/run ids,
- event kind and version,
- source subsystem,
- timestamp,
- redacted summary,
- optional reference ids for `ActionRecord`, LLM call, gate, or workspace document.

Do not publish raw secrets, raw OAuth tokens, or unredacted tool payloads to dashboard/event streams.

## Cancellation

Cancellation must respect ownership:

- user interrupt cancels the current thread/run through existing session or Reborn paths,
- job cancel uses scheduler/job state,
- routine cancel affects only the routine run,
- shutdown cascades to owned background tasks.

Tests should prove cancellation reaches the real side-effecting caller, not only a token helper.

## Lifecycle

Do not replace existing state machines wholesale. Add adapters or projections first:

| Existing State | Projection Example |
| --- | --- |
| `JobState::Pending/InProgress/Stuck/...` | job lifecycle |
| `ThreadState::Idle/Processing/AwaitingApproval/...` | thread lifecycle |
| Reborn run status | mission/run lifecycle |

If a new canonical lifecycle is needed later, migrate one subsystem at a time with compatibility tests.

## Checkpoints

A checkpoint must include:

- version,
- scope and owner ids,
- run/job/thread id,
- last durable event sequence,
- pending gate ids,
- tool/model surface fingerprint,
- safe local state.

It must not include secrets or unscoped handles. Restoring a checkpoint must not silently re-execute completed side effects.

## Rollout

1. Add read-only projections over existing state.
2. Add scoped event envelopes for new events only.
3. Add cancellation adapters where user-visible cancel is weak.
4. Add resource ledger entries where budgets already exist.
5. Add checkpoints for one runner with caller-level recovery tests.

## Tests

- Unit-test event serialization and redaction.
- Caller-level test: cancel through gateway or CLI reaches the run/job that is doing work.
- Caller-level test: budget ledger records an LLM call through the configured provider/bridge path.
- Recovery test: restore uses checkpoint plus durable events and does not duplicate tool actions.
- Projection test: gateway/debug state is scoped per tenant/user.

## Cautions

- Do not promise orphan-free shutdown, exact recovery, or dashboard freshness without tests.
- Avoid `info!`/`warn!` noise from background runtime paths that can corrupt REPL/TUI output.
- Keep feature flags and defaults conservative. Runtime infrastructure changes have broad blast radius.

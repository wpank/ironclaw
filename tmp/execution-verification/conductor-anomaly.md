# Conductor Anomaly Detection

A conductor observes execution and recommends or triggers interventions when work is likely stuck, wasteful, or unsafe. In IronClaw this should start as a set of monitors around existing loops, not as a separate execution engine.

## Current Boundaries

| Signal | Current Source |
| --- | --- |
| LLM failures and latency | `crates/ironclaw_llm` retry, circuit breaker, failover, provider calls |
| Tool outcomes | `src/tools/execute.rs`, `src/tools/dispatch.rs`, `src/context::ActionRecord` |
| Job state | `src/context::JobState`, `src/agent/scheduler.rs`, `src/agent/self_repair.rs` |
| Thread/turn state | `src/agent/session.rs`, `src/agent/thread_ops.rs` |
| Cost pressure | `src/agent/cost_guard.rs`, `src/bridge/cost_guard_gate.rs` |
| Gateway/Reborn status | `src/channels/web/`, `src/bridge/`, Reborn runtime/composition crates |

## Watchers

Start with deterministic watchers that can explain their evidence:

| Watcher | Evidence | Initial Action |
| --- | --- | --- |
| Repeated tool failure | same tool, same normalized error, same scope | shadow warning |
| Retry exhaustion | provider/tool retries exceed configured budget | advisory intervention |
| Context pressure | usage near model/context policy limit | suggest compaction |
| Cost overrun | budget gate near or at limit | pause or fail through existing budget path |
| Time overrun | elapsed time beyond job/routine policy | advisory then cancel if caller supports it |
| Stuck loop | repeated equivalent LLM/tool iterations | ask for clarification or fail job |
| Spec drift | output no longer matches requested files/tasks | require review or gate failure |
| Provider degradation | consecutive transient errors or high latency | rely on provider chain; expose status |
| Gate churn | approval/gate repeats without progress | fail after existing retry cap |

Avoid opaque learned interventions until deterministic monitors are reliable.

## Intervention Levels

| Level | Meaning | Examples |
| --- | --- | --- |
| Observe | record event only | metric, trace event, debug status |
| Advise | suggest a response | compaction hint, model escalation hint |
| Gate | require existing approval/review path | user approval, review gate, budget gate |
| Act | use existing executor action | cancel job, mark failed, switch configured fallback |

The first release should support observe and advise. Gate/act requires caller-level tests at the exact boundary that changes behavior.

## Rollout

1. `off`: no monitors.
2. `shadow`: emit monitor findings without changing behavior.
3. `advisory`: show findings in debug/status surfaces.
4. `enforce`: enable one intervention at a time with per-watcher kill switches.

Do not auto-restart jobs, switch models, or mutate plans until shadow data proves the watcher is specific enough for the target workflow.

## Persistence

For each finding, store:

- scope: tenant/user/project/job/thread/run,
- watcher id and version,
- evidence summary,
- source event ids,
- proposed action,
- actual action taken,
- outcome if later known.

DB-backed findings must be implemented for PostgreSQL and libSQL through the shared trait. A log-only shadow prototype is acceptable before persistence.

## Tests

- Unit-test watcher predicates with normalized evidence.
- Caller-level test: repeated tool failure through `execute_tool_with_safety` or `ToolDispatcher` produces a finding.
- Caller-level test: provider degradation through the LLM provider chain does not bypass existing retry/circuit breaker behavior.
- Scheduler test: a stuck job intervention uses `JobState` transitions that are already valid.
- Gateway/Reborn test when findings are exposed to users or alter run state.

## Implementation Notes

- Forecasting or bandit-based intervention selection is later work. Deterministic evidence comes first.
- Provider health in this layer is job/run-level context; provider fail-fast remains owned by `crates/ironclaw_llm`.
- Comments that promise exact stuck-loop prevention or recovery need enforcement tests or softer wording.

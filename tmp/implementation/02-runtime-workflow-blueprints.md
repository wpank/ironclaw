# Runtime Workflow Blueprints

These runtime patterns must integrate through existing IronClaw boundaries.
Avoid parallel loops or side-effect paths.

## DAG Workflow Runner

Owner: product workflow / runtime runner.

Contract:

- Load a typed workflow definition.
- Validate node ids, dependencies, inputs, and side-effect permissions.
- Execute independent nodes concurrently only when dependency and file-touch
  constraints allow it.
- Route all tool/model/workspace effects through existing runners.
- Fall back to serial execution when the feature flag is off.

Flag: `experimental.dag_workflow_runner`, default `off`.

Caller test: drive plan parsing -> runner -> executor and compare final state
with a serial baseline.

## Progressive Gate Pipeline

Owner: `src/tools/` or a verification-owned crate.

Rungs:

| Rung | Gate | Blocks |
| --- | --- | --- |
| 0 | format/parse | malformed generated code |
| 1 | compile/build | broken generated artifact |
| 2 | lint/unit | local correctness regressions |
| 3 | integration/security smoke | caller-visible or security-sensitive regressions |

Feature flag: `experimental.progressive_gates`, default `off`.

Caller test: trigger the actual code-generation/tool-building caller and assert
the failing gate blocks the side effect.

## Provider Health Watcher

Owner: `crates/ironclaw_llm/`.

Input events: provider id, latency, cost, success/failure, fallback, timestamp.

Output: bounded `HealthSignal` plus optional routing bias. In observe mode,
signals are logged and measured only. In active mode, they may bias eligible
requests but cannot bypass safety routing.

## Event Replay

Owner: web gateway / event stream implementation.

Requirements:

- Cursor ids are scoped to session/user authorization.
- Reconnect emits no duplicates and no cross-session events.
- Replay buffer is bounded.
- Disabled flag falls back to snapshot/history fetch.

Flag: `experimental.event_replay`, default `off`.

Caller test: connect, consume events, reconnect from cursor, and assert no loss,
duplicates, or auth bypass.

## Cancellation

Cancellation must flow from session/run to tool/model/workspace calls through
the existing dispatcher. Tests should verify side effects stop at the real
dispatcher boundary, not only at a token helper.

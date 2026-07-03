# Affect Engine

An affect engine is an internal control system, not a personality layer. It converts operational events into a compact behavioral state that can bias routing, budgets, retry policy, and intervention thresholds.

It must never make user-facing claims such as "I feel frustrated." The only safe external representation is task-grounded language: blocked, retrying, asking for approval, reducing scope, or escalating the model.

## Scope

In scope:

- Classify operational state from concrete signals: tool success/failure, gate results, retries, elapsed time, cost pressure, context pressure, approvals, user correction, and task completion.
- Produce small control outputs: `behavioral_state`, `confidence_delta`, `tier_bias`, `turn_budget_factor`, `exploration_bias`, and `intervention_threshold_bias`.
- Persist enough state to avoid thrashing across turns.
- Expose metrics and shadow decisions before enforcement.

Out of scope:

- User-visible emotion simulation.
- Hidden prompt injection into normal chat history.
- Standalone model routing that bypasses `crates/ironclaw_llm`.
- A separate memory store that competes with `src/workspace/`.

## Minimal Model

Use a three-value PAD-style vector as an implementation detail:

| Dimension | Meaning In IronClaw | Example Inputs |
| --- | --- | --- |
| Pleasure | Outcome trend. | gate passed/failed, user accepted/rejected result, job completed/failed |
| Arousal | Pressure and urgency. | time budget, retries, context pressure, queued work |
| Dominance | Control over current approach. | repeated same failure, tool availability, successful repair |

Classify the vector into six operational states:

| State | Meaning | Default Policy |
| --- | --- | --- |
| `engaged` | Normal work with useful progress. | Keep defaults. |
| `struggling` | Repeated failures or low control. | Escalate checks; consider stronger model; reduce speculative branches. |
| `coasting` | Low pressure, high control. | Prefer cheap model and smaller budgets. |
| `exploring` | Novel work with uncertain path. | Allow more exploration under budget. |
| `focused` | High pressure with progress. | Preserve context; avoid broad branching. |
| `resting` | Idle or consolidation window. | Allow heartbeat/consolidation work if enabled. |

Thresholds must be configurable and should use hysteresis so a single pass/fail does not flip policy repeatedly.

## Integration Points

| Area | Current Boundary | Integration Guidance |
| --- | --- | --- |
| Chat/job/container loop | `src/agent/agentic_loop.rs`, `dispatcher.rs`, `src/worker/job.rs`, `src/worker/container.rs` | Observe loop outcomes through delegate hooks or existing post-iteration paths. Do not fork the loop. |
| Reborn execution | `crates/ironclaw_engine/`, `src/bridge/effect_adapter.rs`, `src/bridge/router.rs` | Attach to effect/runner status and gate outcomes, not UI-only state. |
| Tool execution | `src/tools/execute.rs`, `src/tools/dispatch.rs`, `src/context::ActionRecord` | Record tool outcome features after safety and audit paths run. |
| LLM routing | `crates/ironclaw_llm::{LlmProvider, SmartRoutingProvider}` | Pass a small optional routing hint; the LLM crate still owns final provider choice. |
| Cost | `src/agent/cost_guard.rs`, `src/bridge/cost_guard_gate.rs` | Treat cost pressure as an input, not a replacement for hard budget gates. |
| Memory | `src/workspace/` | Store durable summaries or learned thresholds only when explicitly useful; never delete LLM records. |

## Rollout

1. `off`: no state computed.
2. `shadow`: compute state and log proposed policy changes; no behavior changes.
3. `advisory`: expose state in metrics/debug status and allow manual inspection.
4. `enforce`: allow narrow policy changes such as routing bias or retry threshold adjustment.

Use a separate kill switch for each enforcement surface: routing, budget modulation, retry/intervention, and memory retrieval. A bad classifier must be easy to disable without disabling unrelated systems.

## Tests

- Unit-test vector update, decay, hysteresis, and state classification.
- Caller-level test: a tool failure through `execute_tool_with_safety` or `ToolDispatcher` records the expected event.
- Caller-level test: a chat/job/Reborn run that repeats the same failure enters shadow `struggling` state without altering the user-visible response.
- Provider-chain test: routing bias is passed to the adaptive routing layer only when the feature flag is enabled.
- Persistence test: both PostgreSQL and libSQL load/store state through the shared database abstraction, or the feature stays workspace-only until DB parity exists.

## Implementation Notes

- Keep PAD values internal and normalized. Store versioned state so thresholds can change without corrupting older records.
- Avoid model-name string rewrites. Route by configured model roles or provider metadata.
- If affect-aware memory retrieval is added, cap it as a tie-breaker. Retrieval relevance and tenant isolation remain primary.
- Any comment claiming cross-layer enforcement must be backed by tests at the real call site.

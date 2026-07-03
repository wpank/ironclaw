# IronClaw Integration Recipes

These recipes translate the Roko-derived concepts into IronClaw work items with
concrete caller boundaries and tests.

## 1. Metacognitive Monitor

Target files:

- `src/agent/agent_loop.rs`
- `src/agent/cost_guard.rs`
- optional new `src/agent/metacognition.rs`

Minimum behavior:

1. Record a fingerprint for each iteration: tool name, stable argument hash,
   model id, file-write count, and cost.
2. Detect repeated action loops, no-progress loops, and projected budget overrun.
3. Return an intervention enum rather than directly mutating state.
4. Let the agent loop decide whether to inject guidance, downgrade model, or
   terminate.

Caller-level tests:

- Drive an agent loop fixture through repeated identical tool calls and assert a
  corrective event is emitted.
- Drive a cost-growth sequence through the real cost guard path and assert
  projected overrun is detected before the hard limit.

## 2. HDC Memory Similarity

Target files:

- new `crates/ironclaw_hdc/`
- `crates/ironclaw_memory_native/src/search.rs`
- memory service facade tests

Minimum behavior:

1. Compute deterministic HDC fingerprints for memory body, title, tags, and
   source type.
2. Store fingerprint bytes beside existing memory chunks or in a side table.
3. Add an optional HDC candidate list to existing FTS/vector fusion.
4. Fuse with Reciprocal Rank Fusion rather than replacing existing search.

Caller-level tests:

- Insert three semantically related memories with different wording, search once
  through the memory facade, and assert the structurally related result appears
  even when FTS terms do not match.
- Verify disabled HDC indexing preserves current FTS/vector results.

## 3. Cascade Router

Target files:

- `crates/ironclaw_llm/src/smart_routing.rs`
- `crates/ironclaw_llm/src/costs.rs`
- provider integration tests

Minimum behavior:

1. Keep existing static routing as Stage 1.
2. Add a Stage 2 confidence guard that falls back to the primary provider when
   context is out of distribution.
3. Add Stage 3 LinUCB only when telemetry is enabled and at least two candidate
   models exist.
4. Persist per-arm state through an IronClaw-owned durable store or a small JSON
   state file only in local-dev mode.

Caller-level tests:

- Through `SmartRoutingProvider`, simulate cheap model success and verify the
  cheap arm gains score.
- Simulate a high-risk request and verify static rules bypass the bandit.
- Simulate missing telemetry and verify current behavior is unchanged.

## 4. Gate Pipeline For Generated Code

Target files:

- new `crates/ironclaw_gate/` or `src/verification/`
- tool-builder or code-generation callers
- `src/tools/dispatch.rs` only if a gate is exposed as a tool

Minimum behavior:

1. Rung selector computes max rung from changed files, declared scope, and risk.
2. Gate runner emits structured verdicts.
3. Artifacts are retained with bounded size and redaction.
4. Failures return remediation text that can be fed back to the agent.

Caller-level tests:

- Drive the code-generation caller with a compile error and assert the gate
  failure is visible to the caller, not only to a gate helper.
- Verify stdout/stderr truncation and secret redaction.

## 5. Conductor Provider Health

Target files:

- `crates/ironclaw_llm/src/circuit_breaker.rs`
- optional new `crates/ironclaw_llm/src/provider_health.rs`
- LLM factory composition in `crates/ironclaw_llm/src/lib.rs`

Minimum behavior:

1. Observe latency, status, retry count, cost, and quality outcome.
2. Forecast next-step failure risk with Holt/EWMA.
3. Emit routing bias or circuit-break hints.
4. Do not directly disable providers without the existing circuit-breaker policy
   participating.

Caller-level tests:

- Through an LLM provider wrapper, simulate increasing latency and verify the
  conductor emits a warning before the reactive circuit breaker trips.
- Simulate alternating provider failures and verify oscillation cooldown.

## 6. Dream Consolidation Heartbeat

Target files:

- heartbeat/routine engine files under `src/agent/`
- memory write paths
- cost guard integration

Minimum behavior:

1. Select candidate episodes by surprise, utility, recency, and risk.
2. Rehearse only under an explicit background budget.
3. Write derived memories with `LlmGenerated`, `Derived`, and confidence tags.
4. Promote only after later retrieval or gate success confirms usefulness.

Caller-level tests:

- Seed session outcomes, run the heartbeat routine, and assert memory writes go
  through the memory tool/facade with correct taint metadata.
- Verify a monthly budget cap prevents consolidation.


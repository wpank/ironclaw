# IronClaw Integration Recipes

Each recipe names the owner boundary, the first shippable slice, and the test
that proves the feature is wired through the caller.

## Metacognitive Monitor

- Owner: `src/agent/`.
- Slice: detect repeated failed actions, contradiction loops, and runaway
  retries; emit a bounded intervention event.
- Flag: `experimental.metacognitive_monitor`, default `off`.
- Test: run a conversation fixture through the agent turn loop and assert the
  monitor changes retry policy without altering approval rules.

## Signal Memory Dedup

- Owner: `src/workspace/` and `src/db/`.
- Slice: exact BLAKE3 identity plus soft near-duplicate candidate flag.
- Flag: `experimental.signal_records`, default `off`.
- Test: write exact and near-duplicate memories through the memory tool against
  PostgreSQL and libSQL.

## Cascade Router

- Owner: `crates/ironclaw_llm/`.
- Slice: shadow scorer that records candidate provider/reward while static
  routing remains authoritative.
- Flag: `experimental.cascade_router`, default `off`.
- Test: route fixture requests through the provider factory and assert high-risk
  requests bypass the candidate.

## Progressive Gates

- Owner: `src/tools/`.
- Slice: compile/lint/unit/security rungs with redacted artifacts.
- Flag: `experimental.progressive_gates`, default `off`.
- Test: generated-code workflow hits the real caller and blocks on a failing
  security rung.

## Provider Conductor

- Owner: `crates/ironclaw_llm/`.
- Slice: observe-mode latency/error watcher alongside the existing circuit
  breaker.
- Flag: `experimental.provider_conductor`, default `off`.
- Test: latency ramp fixture emits `PredictedFailure` before the reactive
  breaker would trip while observe mode keeps behavior unchanged.

## Dream Consolidation

- Owner: `src/agent/heartbeat.rs` plus workspace memory facade.
- Slice: deterministic, budgeted replay job that writes redacted, tainted,
  derived memories hidden from retrieval until promoted.
- Flag: `experimental.dream_consolidation`, default `off`.
- Test: heartbeat fixture writes derived memory through the real memory facade
  and kill switch prevents future writes.

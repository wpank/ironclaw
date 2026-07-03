# Detailed Rankings

This file explains the priority matrix in implementation terms. "Rank" here
means strategy priority, not permission to skip the owning subsystem docs.

## Start First

| Rank | Candidate | Score | Recommendation | Main risk |
|------|-----------|-------|----------------|-----------|
| 1 | Robust statistics | 4.20 | Add small, tested helpers for median, trimmed mean, and MAD; then use them where heavy-tailed cost or latency data currently skews estimates. | Overusing robust filters can hide legitimate regime changes. Keep raw observations available. |
| 2 | BLAKE3 memory dedup | 4.30 | Add exact-content hashing to memory writes and merge exact duplicates by bumping metadata. | False merge is unlikely with exact hashes, but user-facing output must say when a merge happened. |
| 3 | Ebbinghaus memory decay | 4.50 | Store decay metadata and archive stale entries; never delete workspace memory. | Archiving useful identity/system context. Exempt identity and system paths explicitly. |
| 4 | Metacognitive monitor | 4.55 | Extend existing stuck-loop handling with alternating-pattern detection and spend projection. | False intervention can interrupt a useful long-running task. Start observe-only or conservative. |
| 5 | Composable scorers | 3.90 | Introduce a minimal `Scorer` interface for quality checks and gate inputs. | Creating an abstraction before callers exist. Keep the first version tied to one real caller. |
| 6 | Hierarchical cancellation | 3.50 | Propagate cancellation through session, turn, and tool/process boundaries using existing async primitives. | Unit tests on tokens are insufficient; verify spawned processes and tool calls stop. |

Build order differs from raw score because robust statistics and dedup are
lower-risk foundations for later measurement and memory work.

## Validate Then Build

| Rank | Candidate | Score | Recommendation | Entry condition |
|------|-----------|-------|----------------|-----------------|
| 7 | Cascade router | 4.15 | Wrap existing smart routing with shadow-mode contextual-bandit decisions. Static safety overrides remain pre-bandit rules. | At least one baseline dataset with cost, latency, selected model, and quality outcome. |
| 8 | Gate verification expansion | 3.85 | Extend existing gate/builder validation in rungs: compile, lint, tests, symbol checks. | Clear language scope and command-sandbox rules. |
| 9 | HDC similarity signal | 3.85 | Prototype as a third retrieval signal behind current FTS/vector RRF. | Offline retrieval benchmark shows a lift on compositional queries. |
| 10 | Cognitive speed labels | 3.50 | Add a small enum only if it feeds routing, budget, or UX decisions. | A caller needs the label; otherwise this is taxonomy without leverage. |

These are not first-pass features. They change core selection, verification, or
retrieval behavior and need measured rollouts.

## Conditional Backlog

| Candidate | Score | Keep | Defer because |
|-----------|-------|------|---------------|
| Enhanced heartbeat | 3.30 | Background consolidation can improve memory hygiene after decay exists. | It introduces autonomous LLM spend and must be budget-capped. |
| DAG execution engine | 3.35 | Useful for repeatable multi-step workflows. | Reborn runner/driver/executor already own child-run execution; avoid a second agent loop. |
| EventBus with replay ring | 3.25 | Replay is useful for web status and debugging. | Check existing web SSE and Reborn event stores before adding another bus. |
| Conductor anomaly detection | 3.25 | Predictive provider health can improve routing. | It needs routing episodes and provider metrics first. |
| Resumable checkpoints | 3.20 | Valuable for long tasks and crash recovery. | IronClaw already has turn/run state; define the missing recovery case before adding state. |
| Declarative TOML tools | 3.15 | Could make extension setup easier. | Existing WASM/MCP extension lifecycle should remain the default path. |
| User engagement PAD tracker | 3.00 | A lightweight engagement signal may help personalization. | Affective modeling is easy to overfit and hard to validate. |

## Research Only

| Candidate | Score | Strategy |
|-----------|-------|----------|
| Full dream consolidation | 2.80 | Keep notes. Build only after decay, HDC, budget caps, and background isolation have shipped. |
| Budget composition / VCG | 2.75 | Start with deterministic cache-aware prompt ordering before considering auction logic. |
| Code intelligence | 2.65 | Consider a Rust-only symbol index if workspace search data shows code retrieval pain. |
| NEAR on-chain reputation | 2.30 | Prove an off-chain trust score first; do not add chain dependencies for internal scoring. |
| Pheromone system | 2.25 | Treat as tagged workspace memory only if multi-agent coordination becomes active. |
| Pure state-machine extraction | 2.20 | Do not pursue until the Reborn runtime stabilizes around a smaller, testable boundary. |
| Full affect engine | 2.10 | Defer. Engagement signals are enough for near-term UX decisions. |
| TDA / sheaves | 1.55 | Keep as research citation material, not product roadmap. |

## Implementation Location Guidance

Use existing ownership boundaries first:

| Area | Preferred location |
|------|--------------------|
| Agent loop monitoring | `src/agent/agentic_loop.rs`, with shared helpers in `src/agent/` only if reuse appears. |
| Cost and estimate robustness | `src/estimation/` plus small utility helpers. |
| Workspace memory hygiene | `src/workspace/` and `src/tools/builtin/memory.rs`, tested through memory tool behavior. |
| Model routing experiments | `crates/ironclaw_llm/src/smart_routing.rs` or adjacent modules; preserve existing provider contracts. |
| Gate validation | Existing `crates/ironclaw_engine/src/gate/` and `src/tools/builder/validation.rs` before new crates. |
| Workflow execution | Existing Reborn runner/driver/executor path. Do not create a second agent execution loop. |
| Web status and replay | `src/channels/web/platform/sse.rs`, web feature modules, and existing event stores. |

## Promotion Criteria

A candidate can move from strategy to implementation when all are true:

- The owning subsystem doc has been read.
- The first PR can be described in one sentence.
- The change has an observable metric or a caller-level regression test.
- Rollback is a flag flip, an inert metadata field, or a small revert.
- The implementation does not require copying captured architecture wholesale.

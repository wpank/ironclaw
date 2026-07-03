# Captured Architecture Overview

This overview summarizes the captured Roko architecture in terms useful for
IronClaw planning. It does not require, assume, or validate access to an
external checkout. For naming details, see [terminology-glossary.md](terminology-glossary.md).

## Core Model

| Captured idea | Meaning | IronClaw translation posture |
|---|---|---|
| Signal / Engram | A content-addressed record for observations, tool traces, verdicts, memories, and derived knowledge. | Model as workspace memory or event data with explicit provenance, taint, and persistence rules. |
| Store / Substrate | Durable persistence for signals and indexes. | Extend IronClaw DB traits first and keep PostgreSQL/libSQL parity. |
| Cell | A typed computation boundary. | Prefer existing tools, workflow steps, routines, or module-owned services. |
| Graph | Ordered or dependency-aware execution of cells. | Use the existing runner/workflow/orchestration path; do not create a second agent loop. |
| Verify / Gate | Structured acceptance checks for artifacts or decisions. | Test through the caller that triggers the side effect. |
| Compose / Route | Prompt/context assembly and provider selection. | Keep routing inside the owning LLM and agent modules. |
| Bus / Trigger | Event flow and external ingress. | Preserve channel, routine, webhook, and auth boundaries. |

## Layer Map

| Layer | Captured responsibility | Local IronClaw anchors |
|---|---|---|
| Data and math | Signal identity, scores, decay, HDC/VSA, statistics. | `src/workspace/`, `src/db/`, targeted crates if the abstraction becomes shared. |
| Runtime | Event flow, cancellation, lifecycle, persistence, metrics. | `src/agent/`, `src/channels/`, `src/db/`, `crates/ironclaw_turns/`. |
| Capabilities | Tools, MCP, memory, indexing, verification. | `src/tools/`, `src/workspace/`, `src/extensions/`. |
| Orchestration | Graph execution, planning, recovery, coordination. | Reborn runner/driver/executor paths and product workflow boundaries. |
| Surfaces | CLI, web gateway, editor/control-plane projections. | `src/channels/web/`, channel abstractions, frontend projections. |

## Request Loop

1. Channel or trigger input becomes an inbound request.
2. Session, thread, and turn state classify the request.
3. Prompt/context composition selects identity, memory, skills, tools, and task
   state.
4. The agent loop alternates model calls and approved tool dispatch.
5. Verification and persistence record outcomes before the response is emitted.
6. Optional learning or maintenance jobs consume durable outcomes later.

The captured architecture often expresses this as a cognitive loop. In IronClaw,
keep it grounded in existing module ownership and caller-level tests.

## Translation Guardrails

- Rebuild concepts as IronClaw-native modules or crates.
- Keep action execution behind `ToolDispatcher`.
- Keep trusted trigger ingress restricted to the trigger-worker and conversation
  boundaries already defined by IronClaw.
- Treat affect, reputation, and learned routing as advisory until a hard safety
  boundary enforces them.
- For persistence changes, add the shared DB operation first, then both
  backends.
- For user-visible or cross-layer behavior, add a caller-level or whole-path
  contract test.

## Local References

- Captured family map: [source-corpus-map.md](source-corpus-map.md)
- Comparison: [roko-vs-ironclaw.md](roko-vs-ironclaw.md)
- Flow walkthrough: [end-to-end-flow.md](end-to-end-flow.md)
- Research sources: [research-citations.md](research-citations.md)
- Practical examples: [examples/README.md](examples/README.md)

# Captured Roko vs IronClaw

This comparison is scoped to the captured reference corpus and the local
IronClaw workspace. It does not make claims beyond those inputs. When the table
says "no equivalent identified," it means no equivalent was identified in the
local pass.

## Comparison Summary

| Concern | Captured Roko pattern | IronClaw anchor or posture |
|---|---|---|
| Primary shape | Protocol-oriented cognitive runtime built around Signal, Cell, Graph, Store, Bus, and Verify concepts. | Product agent runtime with channels, tools, workspace memory, DB-backed sessions, and web gateway. |
| Persistence | Captured Store/Substrate model for Signal/Engram records and indexes. | Shared DB abstractions with PostgreSQL/libSQL parity plus workspace memory semantics. |
| Tool execution | Tools modeled as Cells with verification and trust metadata. | `ToolDispatcher`, WASM/MCP/tool paths, approval and sandbox policy. |
| Routing | Captured cascade and bandit-routing designs. | `crates/ironclaw_llm/` provider routing and cost controls; learned routing should start observe-only. |
| Verification | Captured progressive gate pipeline and structured verdicts. | Caller-owned checks and tests; expand through production boundaries. |
| Memory | Captured Signal/Engram identity, decay, HDC, AntiKnowledge, and dream consolidation. | Workspace memory, search, and storage layers; any decay/HDC work must preserve file-like memory semantics. |
| Supervision | Captured Conductor/watchers for stuck loops and provider health. | Existing self-repair, cost guard, circuit breakers, job/session state, and web projections. |
| Channels | Captured ACP/MCP/control-plane ideas. | ChannelManager, web gateway, SSE/WebSocket, webhook, CLI, WASM channels, and MCP tools. |
| Extensions | Captured plugin and capability registry ideas. | IronClaw extension lifecycle, WASM sandbox, MCP integration, authentication/configuration flows. |
| On-chain trust | Captured chain/reputation marketplace concepts. | Treat as future or optional integration; local approvals and sandboxing remain hard gates. |

## IronClaw Strengths To Preserve

- Clear composition roots in `src/app.rs` and module-owned initialization.
- Shared DB trait with PostgreSQL/libSQL parity expectations.
- Web gateway layered on the same agent/session/tool systems.
- Bearer/origin/auth/body-limit discipline for browser-facing paths.
- WASM, MCP, and extension lifecycle separation.
- Caller-level test discipline for side effects.

## Captured Ideas Worth Translating Carefully

| Captured idea | Why it is useful | First safe IronClaw step |
|---|---|---|
| Structured gate feedback | Turns failures into actionable repair input. | Add typed verdicts at one caller boundary. |
| Provider health bias | Avoids spending on degraded providers before hard failure. | Observe-only metrics and circuit-breaker integration. |
| Memory dedup with provenance | Reduces repeated facts without losing source statements. | Soft duplicate candidates before hard merges. |
| Derived-memory confidence | Makes background learning useful without polluting retrieval. | Taint and source-link all derived memories. |
| Code intelligence | Improves context selection for multi-file work. | Symbol-aware search behind workspace/MCP boundaries. |
| Event projection | Helps operators debug long-running work. | Reconnect/idempotency tests for SSE/WebSocket flows. |

## Non-Negotiable Translation Rules

- Do not add `roko-*` dependencies.
- Do not mint trusted inbound requests outside the existing trusted ingress
  boundary.
- Do not bypass `ToolDispatcher` for actions.
- Do not weaken auth, CORS/origin checks, body limits, sandboxing, or secret
  handling.
- Do not treat advisory learning, affect, reputation, or HDC similarity as a
  hard safety decision without explicit verification.
- Do not claim cost savings, crash safety, or truthfulness unless local
  implementation and benchmark evidence support the claim.

## Related Reference

- [source-corpus-map.md](source-corpus-map.md)
- [architecture-overview.md](architecture-overview.md)
- [v2-implementation-summary.md](v2-implementation-summary.md)
- [examples/failure-scenarios.md](examples/failure-scenarios.md)

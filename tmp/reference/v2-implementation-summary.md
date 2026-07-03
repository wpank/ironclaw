# v2 Implementation Summary

This is the concise IronClaw implementation bridge for the captured Roko v2
spec and v2-depth materials. It intentionally avoids live-source assumptions;
captured paths are provenance labels.

## Translation Priorities

| Captured v2 idea | IronClaw-native target | First verification point |
|---|---|---|
| Signal/Store model | Workspace memory facade plus DB-backed repositories | DB trait tests for PostgreSQL and libSQL |
| Cell/Graph execution | Product workflow or DAG-style runner owned by the relevant module | Caller-level workflow test |
| Verify pipeline | Compile/lint/test/security gates around generated artifacts | Test through the caller that triggers verification |
| Cascade routing | `crates/ironclaw_llm/` provider wrapper or router | Provider-selection contract test |
| Dream consolidation | Heartbeat/routine job writing durable workspace memories | Routine integration test |
| Code intelligence | Workspace indexer or MCP server with symbol-aware retrieval | MCP/workspace search contract test |

## Required Local References

- [source-corpus-map.md](source-corpus-map.md) for captured family mapping.
- [terminology-glossary.md](terminology-glossary.md) for naming.
- [v2-depth-research.md](v2-depth-research.md) for detailed source summaries.
- [plans-catalog.md](plans-catalog.md) for captured implementation-plan patterns.
- `CLAUDE.md` and subsystem `CLAUDE.md` files before changing IronClaw code.

## Guardrails

Rebuild concepts as IronClaw-owned crates or modules. Preserve `ToolDispatcher`
as the action path, maintain PostgreSQL/libSQL parity for persistence, and add
caller-level tests for side effects.

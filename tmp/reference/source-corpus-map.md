# Captured Source Corpus Map

This map summarizes the local captured corpus. Captured `roko-*` names identify
source-corpus families only; they are not dependencies, local files, or
implementation evidence. For planning, follow the local summaries and rebuild
the needed behavior as IronClaw-native modules.

## Crate Families

| Family | Captured identifiers | Captured responsibility | Local follow-up |
|---|---|---|---|
| Core data model | `roko-core`, `roko-primitives` | Signal/Engram records, score vectors, decay, PAD, HDC, math primitives, core traits | [terminology-glossary.md](terminology-glossary.md), [../core-concepts/universal-engram.md](../core-concepts/universal-engram.md), [../core-concepts/mathematical-primitives.md](../core-concepts/mathematical-primitives.md) |
| Runtime substrate | `roko-runtime`, `roko-fs` | Event bus, cancellation, lifecycle, append-only storage, metrics sinks | [../execution-verification/runtime-infrastructure.md](../execution-verification/runtime-infrastructure.md), [../context-memory/persistence-storage.md](../context-memory/persistence-storage.md) |
| Agent loop | `roko-agent`, `roko-std` | Agent trait, tool loop, streaming, checkpoints, retry, built-in tools | [roko-vs-ironclaw.md](roko-vs-ironclaw.md), [../agent-intelligence/agent-patterns.md](../agent-intelligence/agent-patterns.md) |
| Workflow execution | `roko-graph`, `roko-orchestrator` | Cell graph execution, plan phases, wave scheduling, recovery, worktree isolation | [plans-catalog.md](plans-catalog.md), [../execution-verification/dag-execution.md](../execution-verification/dag-execution.md) |
| Verification | `roko-gate` | Progressive gates, adaptive thresholds, forensic replay, contracts, feedback | [../execution-verification/gate-verification.md](../execution-verification/gate-verification.md) |
| Learning | `roko-learn`, `roko-compose`, `roko-conductor` | Model routing, prompt allocation, provider health, anomaly detection | [../agent-intelligence/online-learning.md](../agent-intelligence/online-learning.md), [../execution-verification/conductor-anomaly.md](../execution-verification/conductor-anomaly.md) |
| Memory/cognition | `roko-neuro`, `roko-dreams`, `roko-daimon` | Knowledge store, dream consolidation, affect, somatic markers | [../agent-intelligence/dream-consolidation.md](../agent-intelligence/dream-consolidation.md), [../agent-intelligence/affect-engine.md](../agent-intelligence/affect-engine.md) |
| Integration | `roko-plugin`, `roko-acp`, MCP server crates, `roko-serve`, `roko-agent-server` | Plugins, triggers, editor protocol, control plane, sidecars | [../ecosystem/plugin-extension.md](../ecosystem/plugin-extension.md), [../ecosystem/mcp-editor-integration.md](../ecosystem/mcp-editor-integration.md) |
| Code intelligence | `roko-index`, `roko-lang-rust`, `roko-lang-typescript`, `roko-lang-go` | Symbol extraction, dependency graphs, HDC code fingerprints, language providers | [../context-memory/code-intelligence.md](../context-memory/code-intelligence.md), [../context-memory/language-support.md](../context-memory/language-support.md) |
| Chain/economics | `roko-chain`, Solidity contracts, chain watcher apps | Identity, reputation, bounties, token economics, oracle simulation | [../ecosystem/chain-reputation/README.md](../ecosystem/chain-reputation/README.md), [../ecosystem/smart-contracts/README.md](../ecosystem/smart-contracts/README.md) |

## Design Document Families

| Captured doc family | Purpose | Local summary |
|---|---|---|
| `docs/v1/00-architecture` | Early cognitive architecture, three speeds, five layers, loop model | [architecture-overview.md](architecture-overview.md), [../core-concepts/cognitive-architecture.md](../core-concepts/cognitive-architecture.md) |
| `docs/v1/06-neuro` | HDC/VSA, memory, cross-domain resonance | [../core-concepts/hyperdimensional-computing/README.md](../core-concepts/hyperdimensional-computing/README.md), [../context-memory/README.md](../context-memory/README.md) |
| `docs/v1/09-daimon` | PAD, appraisal, somatic markers, behavioral states | [../agent-intelligence/affect-engine.md](../agent-intelligence/affect-engine.md) |
| `docs/v1/10-dreams` | Replay, imagination, staging, threat rehearsal | [../agent-intelligence/dream-consolidation.md](../agent-intelligence/dream-consolidation.md) |
| `docs/v1/13-coordination` | Stigmergy, pheromones, specialization | [../execution-verification/orchestrator-swarm.md](../execution-verification/orchestrator-swarm.md) |
| `docs/v1/15-code-intelligence` | Symbol graph and code fingerprinting | [../context-memory/code-intelligence.md](../context-memory/code-intelligence.md) |
| `docs/v1/21-references` | Academic citation files | [research-citations.md](research-citations.md), [research-citations/README.md](research-citations/README.md) |
| `docs/v2` | Unified specification vocabulary and APIs | [v2-implementation-summary.md](v2-implementation-summary.md), [terminology-glossary.md](terminology-glossary.md) |
| `docs/v2-depth` | Algorithmic deep dives and implementation commentary | [v2-depth-research.md](v2-depth-research.md) |
| `plans` | TOML plan catalog, task tiers, verification commands | [plans-catalog.md](plans-catalog.md) |

## Captured Dependency Policy

For IronClaw implementation planning:

1. Do not add `roko-*` path dependencies.
2. Do not cite captured source labels as implementation evidence.
3. Treat captured source identifiers as provenance labels only.
4. Rebuild the required pieces as IronClaw-native crates or modules.
5. Preserve IronClaw rules: `ToolDispatcher` for actions, dual PostgreSQL/libSQL persistence, security review for auth/network/secrets, caller-level tests for side effects.
6. Treat absence claims as captured-corpus claims unless IronClaw code or local reference docs verify them.

## Translation Table

| Captured pattern | IronClaw-native target |
|---|---|
| `Store<Engram>` | Workspace memory service facade plus DB-backed repositories |
| `Cell::execute(Vec<Engram>)` | Tool-dispatched task step returning structured JSON and cost metadata |
| `Gate::verify(Context, Engram)` | Caller-level verification around generated artifacts or tool outputs |
| `CascadeRouter` | Extension of `crates/ironclaw_llm/src/smart_routing.rs` |
| `Conductor` watcher | Provider health monitor layered above `CircuitBreakerProvider` |
| `DreamRunner` | Heartbeat/routine job that reads session outcomes and writes durable memory |
| `EventBus<T>` | Broadcast stream feeding TUI/web/SSE/metrics projections |
| `ACP session` | Structured web/editor session protocol over existing channel abstractions |
| `RokoLayout` | IronClaw workspace/memory paths, not captured layout directories |

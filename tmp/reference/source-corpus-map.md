# Captured Source Corpus Map

This map replaces the need for external Roko source access by describing the captured
corpus structure in operational terms. It is intentionally compact: the deep
details live in the concept category folders and the buildable sketches live in
`../implementation/`.

## Crate Families

| Family | Captured identifiers | Responsibility | Primary docs |
|---|---|---|---|
| Core data model | `roko-core`, `roko-primitives` | Signal/Engram, score vectors, decay, PAD, HDC, math primitives, core traits | 01, 03, 10, 11, 13 |
| Runtime substrate | `roko-runtime`, `roko-fs` | Event bus, cancellation, lifecycle, state hub, append-only storage, metrics sinks | 14, 20 |
| Agent loop | `roko-agent`, `roko-std` | Agent trait, translators, streaming, checkpoints, retry, composition, scorers | 17 |
| Workflow execution | `roko-graph`, `roko-orchestrator` | Cell graph execution, plan phases, event sourcing, wave scheduling, recovery | 04, 15, 28 |
| Verification | `roko-gate` | Progressive gates, adaptive thresholds, forensic replay, contracts, feedback | 05 |
| Learning | `roko-learn`, `roko-compose`, `roko-conductor` | Model routing, prompt allocation, provider health, anomaly detection | 06, 07, 09 |
| Memory/cognition | `roko-neuro`, `roko-dreams`, `roko-daimon` | Knowledge store, dreams, affect, somatic markers, offline consolidation | 01, 02, 03, 10, 13 |
| Integration | `roko-plugin`, `roko-acp`, MCP server crates, `roko-serve`, `roko-agent-server` | Plugins, triggers, editor protocol, control plane, sidecars | 16, 21, 23 |
| Code intelligence | `roko-index`, `roko-lang-rust`, `roko-lang-typescript`, `roko-lang-go` | Symbol extraction, dependency graphs, HDC code fingerprints, language providers | 12, 22 |
| Chain/economics | `roko-chain`, Solidity contracts, chain watcher apps | Identity, reputation, bounties, token economics, oracle simulation | 08, 24 |

## Design Document Families

| Captured doc family | Purpose | Where it is summarized |
|---|---|---|
| `docs/v1/00-architecture` | Early cognitive architecture, three speeds, five layers, loop model | 13, 26 |
| `docs/v1/06-neuro` | HDC/VSA, memory, cross-domain resonance | 01, 10 |
| `docs/v1/09-daimon` | PAD, appraisal, somatic markers, behavioral states | 03 |
| `docs/v1/10-dreams` | Replay, imagination, staging, threat rehearsal | 02 |
| `docs/v1/13-coordination` | Stigmergy, pheromones, specialization | 13, 15 |
| `docs/v1/15-code-intelligence` | Symbol graph and code fingerprinting | 12, 22 |
| `docs/v1/21-references` | Academic citation files | 25 |
| `docs/v2` | Unified specification vocabulary and APIs | 26, 27 |
| `docs/v2-depth` | Algorithmic deep dives and implementation commentary | 27 |
| `plans` | TOML plan catalog, task tiers, verification commands | 28 |

## No External Dependency Policy

For IronClaw implementation planning:

1. Do not add `roko-*` path dependencies.
2. Do not link to a Roko Git remote as implementation evidence.
3. Treat captured source identifiers as provenance labels only.
4. Rebuild the required pieces as IronClaw-native crates or modules.
5. Preserve IronClaw rules: `ToolDispatcher` for actions, dual PostgreSQL/libSQL persistence, security review for auth/network/secrets, caller-level tests for side effects.

## Translation Table

| Roko pattern | IronClaw-native target |
|---|---|
| `Store<Engram>` | Workspace memory service facade plus DB-backed repositories |
| `Cell::execute(Vec<Engram>)` | Tool-dispatched task step returning structured JSON and cost metadata |
| `Gate::verify(Context, Engram)` | Caller-level verification around generated artifacts or tool outputs |
| `CascadeRouter` | Extension of `crates/ironclaw_llm/src/smart_routing.rs` |
| `Conductor` watcher | Provider health monitor layered above `CircuitBreakerProvider` |
| `DreamRunner` | Heartbeat/routine job that reads session outcomes and writes durable memory |
| `EventBus<T>` | Broadcast stream feeding TUI/web/SSE/metrics projections |
| `ACP session` | Structured web/editor session protocol over existing channel abstractions |
| `RokoLayout` | IronClaw workspace/memory paths, not `.roko` |

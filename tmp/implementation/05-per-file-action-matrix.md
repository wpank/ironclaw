# Per-File Implementation And Measurement Matrix

This matrix connects every canonical analysis file to a concrete IronClaw-native
implementation path and a quantification plan. It is intended as the operational
companion to the 28 numbered documents: read the deep dive for theory and source
context, then use this file to decide what code to write, what caller path to
test, and what metric must move before the feature is considered useful.

Any external source-path labels in older analysis notes are provenance only.
They are not required files, live links, or implementation inputs. The targets
below should be built inside IronClaw-owned modules, crates, database traits,
runtime paths, or extension APIs.

## Acceptance Pattern

Every feature should ship with:

1. A narrow, disabled-by-default implementation behind an IronClaw-owned flag,
   config key, or runtime policy.
2. A caller-level test that drives the real boundary where the behavior matters.
3. A benchmark or recorded scenario that captures baseline and candidate metrics.
4. A rollback path that returns to current IronClaw behavior without data loss.

Minimum evidence:

```text
feature_result =
  improves(target_metric)
  && does_not_regress(quality_guardrail)
  && does_not_regress(security_guardrail)
  && has_persistence_or_config_migration_plan_when_needed
```

## Canonical File Matrix

| Doc | IronClaw target | First shippable artifact | Caller-level test | Quantification gate |
|---|---|---|---|---|
| [01 - HDC](../core-concepts/hyperdimensional-computing/README.md) | `src/workspace/` hybrid retrieval and dedup scoring | `HdcFingerprint` plus `HdcIndex` facade behind memory search config | Drive `memory_search` and `memory_write` through the workspace tool boundary with repeated and near-duplicate inputs | p95 HDC candidate generation < 10 ms for 100k fingerprints; duplicate write rate down 20% without relevance loss > 2 pp |
| [02 - Dreams](../agent-intelligence/dream-consolidation.md) | heartbeat/background runtime plus workspace memory promotion | idle-time consolidation job that samples recent turns, proposes summaries, and writes staged memories | Run heartbeat cycle against a fixture workspace and verify promoted memories are searchable and attributed | useful-memory hit rate +10pp; background token spend within daily budget; no secret or private channel leakage |
| [03 - Affect](../agent-intelligence/affect-engine.md) | agent policy hints and response-style modulation | `AgentAffectState` with PAD updates from task outcomes, exposed only as low-weight routing metadata | Submit tasks through the agent loop and verify PAD affects policy metadata without changing approval rules | user correction rate non-increasing; no safety bypasses; p95 update cost < 1 ms |
| [04 - DAG](../execution-verification/dag-execution.md) | product workflow and Reborn runner path | minimal DAG executor for deterministic, typed workflow steps and dependency ordering | Drive a representative product workflow through plan parsing -> runner -> executor, not just graph helper tests | parallel wall-clock improvement >= 15% on independent steps; identical final state versus serial baseline |
| [05 - Gates](../execution-verification/gate-verification.md) | tool execution, code-edit verification, CI-style checks | progressive gate runner for compile, lint, test, and smoke checks with rung selection | Trigger a code-edit workflow and assert the real caller blocks submission on failing gates | escaped defect rate down; p95 gate overhead bounded by complexity tier; false block rate < 5% |
| [06 - Conductor](../execution-verification/conductor-anomaly.md) | provider health, runtime telemetry, circuit breaking | watcher ensemble for latency, error, cost, and quality with Holt forecast | Route real LLM provider calls through mocked provider responses and verify health changes affect routing | degraded-provider spend down 50%; fallback success rate up; no p95 latency regression > 10% in healthy state |
| [07 - Learning](../agent-intelligence/online-learning.md) | `crates/ironclaw_llm/` routing and cost controls | LinUCB or epsilon-greedy model selector with audit logs and static safety constraints | Drive actual LLM routing facade with fixture providers and assert selected provider, audit fields, and fallback path | cost/request down 20%; pass rate no worse than -2 pp; confidence intervals reported |
| [08 - Reputation](../ecosystem/chain-reputation/README.md) | local trust ledger and optional NEAR identity bridge | off-chain 7-domain reputation table with half-life decay and signed events | Submit tool/agent outcomes through the event path and verify reputation updates persist in both DB backends | bad-actor selection rate down in simulations; update p95 < 20 ms; Postgres/libSQL parity |
| [09 - Composition](../context-memory/budget-composition.md) | prompt builder, memory/tool/history packing | budget allocator that scores context candidates by utility, recency, and cache position | Drive prompt construction through the real agent request path and inspect packed sections | input tokens/request down 15%; answer quality no worse than -2 pp; cache-hit tokens up |
| [10 - Signal](../core-concepts/universal-engram.md) | workspace memory substrate and DB schema | content-addressed memory object with BLAKE3 id, lineage, decay, and taint fields | Write, update, and search memory through tools against Postgres and libSQL | duplicate storage down 30%; query p95 unchanged; taint propagation covered by tests |
| [11 - Math](../core-concepts/mathematical-primitives.md) | shared statistics utilities and trace analysis experiments | robust stats module: median, MAD, trimmed mean, Hodges-Lehmann | Exercise runtime metric aggregation through the caller that triggers thresholds | outlier-induced false alerts down 50%; CPU overhead < 1 ms per aggregation window |
| [12 - Code Intelligence](../context-memory/code-intelligence.md) | workspace code indexing and search | symbol graph plus RRF merge with existing full-text/vector search | Index a fixture repo and search through the exposed workspace search boundary | top-5 symbol retrieval +15 pp; indexing time tracked per KLOC; memory growth bounded |
| [13 - Cognitive Architecture](../core-concepts/cognitive-architecture.md) | runtime scheduling labels and agent coordination policy | cognitive speed metadata for reactive, reflective, and consolidation work | Submit mixed tasks and verify scheduling policy chooses correct queues and budgets | urgent task latency down; background work stays inside budget; no starvation over 24h simulation |
| [14 - Runtime](../execution-verification/runtime-infrastructure.md) | event bus, cancellation, lifecycle projections | bounded replay event bus and hierarchical cancellation token wiring | Drive cancellation from session -> tool call and verify side effects stop at the real dispatcher | cancellation p95 < 100 ms; replay memory bounded; no orphaned subprocesses |
| [15 - Orchestrator](../execution-verification/orchestrator-swarm.md) | multi-run coordination and event-sourced planning | wave scheduler with conflict detection for file touch sets | Run multi-task plan fixtures through scheduler and verify conflicting edits serialize | parallel throughput +20% on independent tasks; conflict-induced retry rate down |
| [16 - Plugins](../ecosystem/plugin-extension.md) | extension registry, WASM channels/tools, lifecycle hooks | declarative event source or feedback hook routed through existing extension lifecycle | Install, activate, trigger, and remove a fixture extension through public registry APIs | activation/deactivation reliable over 100 cycles; no permission escape; hot reload debounce tested |
| [17 - Agent Patterns](../agent-intelligence/agent-patterns.md) | agent loop guardrails and checkpointing | metacognitive monitor for stuck loops, contradiction, runaway cost, and retries | Drive a conversation fixture through agent turn handling and assert monitor intervention | runaway token spend down; successful completion unchanged; false intervention rate < 5% |
| [18 - Roadmap](../strategy/integration-roadmap.md) | release planning and dependency sequencing | milestone tracker linking features to flags, tests, docs, and rollback notes | Validate roadmap entries against files changed in feature branches | each shipped feature has owner, metric, rollout gate, and rollback; no orphan feature flags |
| [19 - Priority](../strategy/priority-matrix/README.md) | prioritization process and ROI scoring | machine-readable scoring sheet or TOML manifest for candidate features | Recompute scores from canonical weights and verify ordering is deterministic | score changes explainable; top-10 plan fits available engineering budget |
| [20 - Persistence](../context-memory/persistence-storage.md) | DB-backed storage parity plus append-only audit options | Signal/event audit log abstraction implemented for Postgres and libSQL | Run the same persistence contract suite against both backends | backend parity; crash replay test passes; compaction does not lose lineage |
| [21 - MCP/ACP](../ecosystem/mcp-editor-integration.md) | web gateway, MCP client/server exposure, editor session protocol | typed session event protocol with permission events and streaming updates | Drive web/API session through JSON-RPC or SSE boundary with mocked editor client | event loss zero in fixture; reconnect resumes from cursor; permission denial enforced |
| [22 - Language](../context-memory/language-support.md) | language provider abstractions for workspace indexing | Rust provider first, with trait shape ready for TS/Go providers | Index fixture projects through workspace indexer and verify symbols/imports/build hints | symbol precision/recall measured; parse latency per KLOC; graceful fallback on parser errors |
| [23 - Control Plane](../ecosystem/control-plane.md) | browser gateway, routes, SSE/WebSocket, state projections | unified state projection for sessions, tools, costs, health, and events | Exercise HTTP routes and event streams through web gateway integration tests | route p95 latency tracked; event stream reconnect passes; auth/origin checks unchanged |
| [24 - Contracts](../ecosystem/smart-contracts/README.md) | local reputation first, optional NEAR contract prototypes | NEAR-compatible identity/reputation interface plus local mock implementation | Run local reputation flows and, if enabled, contract simulation with mocked signer | no mainnet dependency; failed chain calls degrade to local ledger; gas/storage estimates recorded |
| [25 - Citations](../reference/research-citations/README.md) | documentation, design-review support, experiment rationale | citation-to-feature map embedded in design docs and benchmark plans | Lint docs for every experimental feature having a source and metric | every research-backed feature cites source and states measurable hypothesis |
| [26 - Architecture](../reference/architecture-overview.md) | overall IronClaw transfer architecture | architecture decision record set for adopted primitives: Signal, Cell, Graph, Bus, Store | Check ADR links from implementation PRs and verify docs match code behavior | reduced ambiguity in reviews; no implementation merged without security/persistence decision |
| [27 - v2 Depth](../reference/v2-depth-research.md) | long-horizon research backlog and concept taxonomy | curated backlog labels for signal algebra, graph execution, telemetry, security, tools, UI | Validate backlog items link to an owner doc, metric, and risk class | research backlog searchable; stale/high-risk items reviewed monthly |
| [28 - Plans](../reference/plans-catalog.md) | task planning format, verification recipes, model tiers | IronClaw task-plan TOML schema for internal automation and agent runs | Parse and execute fixture plans through existing runner boundaries | plan validation catches schema errors; task estimates within +/-25%; verification commands reproducible |

## Audit Report Coverage

| File | Use | Required follow-through |
|---|---|---|
| [quality-report.md](../reference/quality-report.md) | Cross-document consistency and completeness tracker | Re-run audits after any document changes; update counts, self-contained-source wording, and link checks |

## Benchmark Artifact Checklist

For each implemented feature, add or update one of:

- A `criterion` benchmark for local deterministic operations.
- A hermetic integration scenario with mocked providers, DBs, or network
  services.
- A recorded Reborn QA fixture when model/tool choice or final state matters.
- A dashboard/event metric when production behavior must be tracked after
  rollout.

## Practical Rollout Template

```text
Feature:
  owner module:
  flag or config:
  flag default:
  shadow mode:
  canary cohort:
  baseline scenario:
  candidate scenario:
  primary metric:
  guardrail metrics:
  rollback trigger:
  rollback validation:
  caller-level tests:
  caller boundary under test:
  persistence impact:
  Postgres migration:
  libSQL migration:
  security review areas:
  security reviewer/checklist:
  rollback:
```

## Quantification Gate Definitions

Use these definitions when a matrix row says "down", "unchanged", or "bounded".

| Phrase | Concrete definition |
|---|---|
| `cost/request down X%` | `(baseline_median_cost - candidate_median_cost) / baseline_median_cost >= X` over the same scenario set |
| `quality no worse than -2pp` | `candidate_pass_rate + 0.02 >= baseline_pass_rate` with the same oracle |
| `p95 latency no worse than +10%` | `candidate_p95 <= baseline_p95 * 1.10` |
| `escaped defect rate down` | defects reaching user review or CI after feature / total generated-code fixtures decreases by at least the stated threshold |
| `false block rate < 5%` | valid changes blocked / total valid-change fixtures is below 0.05 |
| `useful-memory hit rate +10pp` | later tasks where retrieved memory is judged useful increases by at least 10 percentage points |
| `duplicate storage down 30%` | canonical memory records per repeated-fact fixture decreases by at least 30% without false merges above 2% |
| `bounded background spend` | background cost stays below configured daily microusd cap in every fixture and canary day |
| `event loss zero` | reconnect fixture observes every event id exactly once after dedupe |
| `backend parity` | the shared DB contract suite passes against PostgreSQL and libSQL with the same expected records |

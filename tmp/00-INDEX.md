# Roko --> IronClaw: Technology Transfer Analysis

## About This Collection

This is the master index for a 28-document technical analysis comparing two Rust-based AI agent systems: **Roko** and **IronClaw**. The analysis identifies novel architectural concepts in Roko that could be adopted into IronClaw, documents them with code-level detail, and proposes concrete integration plans ranked by impact, effort, and risk.

**Total scope**: 28 analysis documents, ~43,800 lines, covering 18+ Roko crates (~200K lines of Rust, 1,600+ tests) mapped against IronClaw's production codebase.

**Date**: 2026-07-02

---

## Executive Summary

### What Is Roko?

Roko is a Rust toolkit for building "agents that build themselves." It is an 18-crate, 200K+ line system designed for autonomous software development. Given a product requirements document (PRD), Roko generates an implementation plan, dispatches LLM-powered agents to execute tasks in parallel, validates output through a 7-step progressive verification pipeline (compile, lint, test, symbol check, generated tests, property tests, integration tests), persists every result as a content-addressed data object called a "Signal" (formerly "Engram"), and feeds outcomes back into its learning subsystems so it improves over time.

What makes Roko unusual is not the LLM integration (which is standard) but the *harness* wrapping the LLM. It draws on neuroscience, economics, topology, and cybernetics to build subsystems that most AI agent frameworks lack:

- **Hyperdimensional computing** (HDC) for sub-millisecond similarity search across 100K+ items with no model inference -- just bitwise operations on 10,240-bit binary vectors.
- **Dream consolidation** -- a biologically-inspired offline learning system modeled on mammalian sleep stages (NREM replay, REM counterfactual generation, hypnagogic creativity).
- **Contextual bandit model routing** that learns which LLM provider to use for each request type, projecting 30-50% cost savings.
- **An affect engine** that models agent emotional state using the PAD (Pleasure-Arousal-Dominance) model from psychology and Damasio's somatic marker hypothesis for decision shortcuts.
- **On-chain reputation** with Solidity smart contracts for soulbound identity passports, stake-backed reputation, and a bounty marketplace.

Roko is self-hosting: it develops itself using its own pipeline. It includes an HTTP control plane with ~85 REST routes, per-agent HTTP sidecars, an interactive ratatui TUI dashboard, and support for Claude, Gemini, Perplexity, OpenRouter, Ollama, and any OpenAI-compatible API. It communicates with code editors via ACP (Agent Communication Protocol), a JSON-RPC 2.0 protocol, and exposes five MCP (Model Context Protocol) servers.

### What Is IronClaw?

IronClaw is a secure personal AI assistant built in Rust, emphasizing user-first security, defense in depth, and multi-channel access with proactive background execution. While Roko focuses on autonomous software development, IronClaw focuses on being a trustworthy, extensible assistant platform that can operate across communication channels (CLI/TUI, web browser, Telegram, Gmail, HTTP webhooks, WASM-sandboxed channels) while maintaining strong security guarantees.

IronClaw's key architectural properties:

- **Extracted crates** for safety (`ironclaw_safety` -- prompt injection detection, validation, leak prevention), LLM integration (`ironclaw_llm` -- multi-provider with NEAR AI, OpenAI, Anthropic, Bedrock, Ollama, and others), an engine (`ironclaw_engine`), and skills (`ironclaw_skills`).
- **A WASM-sandboxed tool system** with wasmtime, fuel metering, memory limits, network allowlisting, and credential injection. All actions route through `ToolDispatcher::dispatch()` for audit, safety, and policy enforcement (the "everything goes through tools" principle).
- **Workspace-based persistent memory** with hybrid search (full-text search + vector embeddings merged via Reciprocal Rank Fusion). Four tools: `memory_search`, `memory_write`, `memory_read`, `memory_tree`.
- **A heartbeat system** for proactive background execution (reads HEARTBEAT.md every 30 minutes), a Docker-based execution sandbox, lifecycle hooks, an extension registry, and a SKILL.md prompt extension system.
- **Dual-backend persistence** (PostgreSQL + libSQL/Turso), an agent operating within a job state machine (Pending -> InProgress -> Completed -> Submitted -> Accepted), and OS service management for daemon installation.

### Why Compare Them?

Both systems are Rust-based AI agent platforms, but they occupy different niches and have made different architectural bets:

| Dimension | Roko | IronClaw |
|-----------|------|----------|
| **Primary purpose** | Autonomous software development | Secure personal AI assistant |
| **Security model** | Basic role auth, pre/post checks | Full WASM sandbox, Docker isolation, safety crate, network proxy |
| **Channel access** | CLI + HTTP API | CLI/TUI, web, Telegram, Gmail, HTTP, WASM channels |
| **Persistence** | Append-only JSONL files | Dual-backend database (PostgreSQL + libSQL) |
| **Learning** | Contextual bandits, dream consolidation, HDC | EMA-based estimation, heartbeat (static file) |
| **Verification** | 7-rung progressive gate pipeline | Basic tool builder validation |
| **Extension model** | TOML manifests, plugin trait, event sources | WASM sandbox, MCP, extension registry, SKILL.md |
| **Multi-agent** | Swarm orchestration, wave scheduling, pheromones | Single-agent with job state machine |
| **On-chain** | 13 Solidity contracts (EVM simulator) | None (planned for NEAR) |

Each has novel concepts the other lacks. This analysis identifies 30+ novel concepts from Roko that could be adopted into IronClaw as modular, reusable components, prioritized by impact, effort, and architectural fit. It also documents where IronClaw is ahead (security, channels, persistence) so the transfer is informed rather than aspirational.

---

## Key Statistics

| Metric | Value |
|--------|-------|
| Roko crates analyzed | 18+ |
| Roko lines of code | ~200,000 |
| Roko tests | 1,600+ |
| Analysis documents produced | 28 (docs 01-28) |
| Total analysis lines | ~43,800 |
| Novel concepts identified | 30+ |
| Estimated LOC for top 10 features | ~5,250-5,950 |
| High-priority concepts | 7 |
| Medium-priority concepts | 12 |
| Low-priority concepts | 3 |

---

## Key Concepts Glossary

Terms used throughout this document collection, in alphabetical order. Each entry names the document(s) where the concept is covered in depth.

| Term | Definition | Primary Document(s) |
|------|-----------|---------------------|
| **ACP** | Agent Communication Protocol. Roko's JSON-RPC 2.0 protocol for bidirectional communication between agents and code editors (like Zed). Supports streaming session updates, permission gates, and event bridging. | [21](21-mcp-editor-integration.md) |
| **Affect Engine (Daimon)** | Roko's emotional modeling subsystem. Uses PAD vectors (see below) across three temporal layers (emotion/mood/temperament) to modulate agent response style. Named "Daimon" in the Roko codebase (`roko-daimon` crate). | [03](03-affect-engine.md) |
| **BLAKE3** | A cryptographic hash function used by Roko for content-addressing Engrams/Signals. Enables deduplication (identical content produces identical hashes), tamper detection (any change alters the hash), and lineage tracking (parent hashes chain into a DAG). | [10](10-universal-engram.md), [15](15-orchestrator-swarm.md) |
| **Cascade Router** | A 3-stage LLM model selection pipeline: static rules (language, safety flags) -> confidence check (is the model reliable enough?) -> LinUCB bandit selection (which model maximizes expected reward given cost constraints?). | [07](07-online-learning.md) |
| **Cell** | Roko's universal unit of computation in the DAG execution engine. Any operation (LLM call, shell command, gate check, file transform) is wrapped as a Cell with typed inputs, typed outputs, a cost budget, and a verification contract. Cells compose into Graphs. | [04](04-dag-execution.md) |
| **CEP** | Complex Event Processing. In the Conductor (doc 06), detects compound patterns across multiple watchers (e.g., "latency spike AND error rate increase within 30 seconds" triggers a circuit break that neither metric alone would justify). | [06](06-conductor-anomaly.md) |
| **C-factor** | Collective intelligence metric inspired by Woolley et al. (2010). Measures 5 axes: turn-taking equality, peer prediction accuracy, citation reciprocity, delivery rate, and HDC diversity. Used to evaluate multi-agent swarm quality. | [13](13-cognitive-architecture.md) |
| **Conductor** | Roko's ensemble anomaly detection system (`roko-conductor` crate). Runs 10 watchers (latency, error rate, token usage, cost, quality, throughput, saturation, availability, drift, coherence) with predictive circuit breaking via Holt exponential smoothing. | [06](06-conductor-anomaly.md) |
| **CUSUM / EWMA / BOCPD** | Three statistical methods for detecting shifts in metric distributions. CUSUM (Cumulative Sum) detects persistent drifts; EWMA (Exponentially Weighted Moving Average) detects gradual trends; BOCPD (Bayesian Online Change Point Detection) detects abrupt regime changes. Used in the Gate pipeline (doc 05) for adaptive threshold adjustment. | [05](05-gate-verification.md), [06](06-conductor-anomaly.md) |
| **DAG** | Directed Acyclic Graph. A graph where edges have direction and no cycles exist. In doc 04, Roko defines workflows as DAGs of Cells: each Cell is a node, edges carry data and conditions (always/on-success/on-failure/when-predicate), and topological sort determines execution order. | [04](04-dag-execution.md), [15](15-orchestrator-swarm.md) |
| **Dream Consolidation** | A background offline learning system modeled on mammalian sleep neuroscience. Five subsystems: NREM replay (strengthen useful memories), REM imagination (generate counterfactuals), hypnagogic creativity (discover cross-domain insights), threat rehearsal (rehearse failure scenarios), and a staging buffer (promote knowledge through confidence tiers). | [02](02-dream-consolidation.md) |
| **Ebbinghaus Forgetting Curve** | A mathematical model of memory decay from experimental psychology (Ebbinghaus, 1885). Retention = e^(-t/S) where S is stability (strengthened by retrieval). In Roko, memories that are accessed frequently have higher stability and decay slower; unaccessed memories fade toward zero relevance. | [10](10-universal-engram.md) |
| **Engram / Signal** | Roko's universal data type for knowledge. In the v1 codebase it is called "Engram"; the v2 specification renames it "Signal" but the structure is identical. Content-addressed via BLAKE3 hash, scored on 7 axes, tagged with decay parameters, and linked to parent Engrams/Signals forming a provenance DAG. | [10](10-universal-engram.md), [20](20-persistence-storage.md) |
| **FTS** | Full-Text Search. Both Roko and IronClaw support text search over stored data. IronClaw uses FTS5 (SQLite's full-text search engine) combined with vector embeddings via RRF. | [12](12-code-intelligence.md) |
| **Gate** | A verification checkpoint in Roko's gate pipeline (`roko-gate` crate). Each "rung" is a Gate that takes code as input and produces a verdict (pass/fail with confidence score). The 7 rungs progress from cheap/fast (compile check) to expensive/thorough (integration tests). | [05](05-gate-verification.md) |
| **HDC** | Hyperdimensional Computing (also called Vector Symbolic Architecture / VSA). A computing paradigm using very high-dimensional binary vectors (10,240 bits in Roko) where similarity is measured by Hamming distance (number of differing bits). Three operations: XOR (bind -- associate two concepts), majority vote (bundle -- merge multiple concepts), and rotation (sequence -- encode order). ~2.5ns per comparison, 100K vectors scanned in <1ms. | [01](01-hyperdimensional-computing.md) |
| **Holt Exponential Smoothing** | A forecasting method that tracks both level and trend of a time series. The Conductor uses it to predict where a metric (e.g., LLM latency) will be in N seconds, enabling *predictive* circuit breaking -- tripping the breaker before the metric crosses the threshold, rather than after. | [06](06-conductor-anomaly.md) |
| **KORAI** | Roko's hypothetical utility token with 1% daily demurrage (idle tokens lose value over time, incentivizing active use). Part of the on-chain economic model. | [08](08-chain-reputation.md), [24](24-smart-contracts.md) |
| **LinUCB** | Linear Upper Confidence Bound. A contextual bandit algorithm that learns a linear relationship between context features and expected reward, then selects the action (LLM provider) with the highest upper confidence bound. Used in Roko's cascade router for model selection. | [07](07-online-learning.md) |
| **MCP** | Model Context Protocol. An open protocol for connecting AI models to external tools and data sources. Roko has 5 MCP server crates; IronClaw has a full MCP client with HTTP/stdio/Unix transports. | [21](21-mcp-editor-integration.md) |
| **PAD Vector** | Pleasure-Arousal-Dominance. A 3-dimensional vector from psychology (Mehrabian & Russell, 1974) representing emotional state. Each dimension ranges from -1.0 to +1.0. The 8 octants (sign combinations) map to named states: +P+A+D = Exuberant, -P+A-D = Anxious, etc. Used in Roko's affect engine. | [03](03-affect-engine.md) |
| **Pheromone** | In Roko's stigmergic coordination model, a typed, decaying marker left in a shared environment. Seven types: threat, opportunity, wisdom, alpha, pattern, anomaly, consensus. Agents read pheromones deposited by other agents (or their past selves) to coordinate without direct communication -- similar to how ants coordinate via chemical trails. | [13](13-cognitive-architecture.md) |
| **PRD** | Product Requirements Document. In Roko, the input that starts the development pipeline: a structured document describing what to build, which Roko decomposes into tasks, assigns to agents, and executes through the gate pipeline. | [26](26-roko-architecture-overview.md) |
| **RRF** | Reciprocal Rank Fusion. A method for merging ranked lists from different search strategies. Score = sum(1/(k + rank_i)) across all strategies. Both IronClaw (memory search) and Roko (code intelligence) use RRF to merge FTS, vector, graph, and HDC results. | [12](12-code-intelligence.md) |
| **Somatic Marker** | From Damasio's neuroscience theory: a cached emotional response to a decision context that provides a "gut feeling" shortcut. In Roko, implemented as an 8-dimensional strategy vector stored in a k-d tree. When a similar decision context arises, the stored marker biases the decision without full deliberation. | [03](03-affect-engine.md) |
| **Soulbound NFT** | A non-transferable, non-fungible token used for agent identity. In Roko's on-chain model (ERC-8004), each agent has a soulbound passport NFT recording its capabilities, reputation history, and trust tier. Cannot be sold or traded -- identity is permanent. | [08](08-chain-reputation.md), [24](24-smart-contracts.md) |
| **Stigmergy** | Indirect coordination through environmental modification. Originally observed in social insects (ants leave pheromone trails; termites build structures based on what others have already built). In Roko, agents deposit digital pheromones that persist and decay, enabling cross-session coordination without direct messaging. | [13](13-cognitive-architecture.md) |
| **Thompson Sampling** | A Bayesian approach to the exploration-exploitation tradeoff. Maintains a probability distribution over expected reward for each option, samples from each distribution, and picks the option with the highest sample. Used in Roko for adaptive threshold learning (Conductor) and context budget allocation (Composition). | [06](06-conductor-anomaly.md), [09](09-budget-composition.md) |
| **TraceRank** | A PageRank-inspired algorithm for computing agent reputation from interaction graphs. Reputation flows through successful collaborations. Includes collusion detection: if two agents disproportionately validate each other's work, their mutual reputation transfer is dampened. | [08](08-chain-reputation.md) |
| **VCG Auction** | Vickrey-Clarke-Groves auction. A mechanism design concept where bidders report truthful valuations because the pricing rule makes truth-telling the dominant strategy. In Roko's prompt composition system, subsystems (skills, memory, tools, history) bid for context window space via VCG, ensuring each subsystem reports its true value for inclusion. | [09](09-budget-composition.md) |

---

## Complete Document Index

### Concept Analysis Documents (01-17)

These 17 documents each analyze a specific novel concept from Roko's codebase, with real code examples, theoretical foundations, and concrete IronClaw integration proposals.

| # | Title | Summary | Priority | Effort |
|---|-------|---------|----------|--------|
| [01](01-hyperdimensional-computing.md) | **Hyperdimensional Computing** | 10,240-bit binary vectors for ultra-fast semantic similarity via bitwise operations. Encodes knowledge, code, and queries as high-dimensional vectors where XOR = bind, majority vote = bundle, rotation = sequence. Six applications documented: knowledge fingerprinting, role-filler encoding, cross-domain resonance, code fingerprinting, admission control (anti-knowledge), and context assembly scoring. ~2.5ns per comparison, 100K vectors scanned in <1ms. False positive analysis with threshold selection math. | HIGH | ~800-1,200 LOC |
| [02](02-dream-consolidation.md) | **Dream Consolidation** | Biologically-inspired offline learning system with 5 subsystems modeled on mammalian sleep neuroscience: NREM replay (Mattar-Daw utility-weighted experience replay), REM imagination (LLM-generated counterfactual scenarios), hypnagogic creativity (4-layer cross-domain insight pipeline), threat rehearsal (failure scenario replay with strategy generation), and a staging buffer (Raw -> Replayed -> Validated -> Promoted confidence lifecycle). Scheduling governed by session idleness detection and token budgets. | HIGH | ~1,200-1,500 LOC |
| [03](03-affect-engine.md) | **Affect Engine (Daimon)** | PAD (Pleasure-Arousal-Dominance) emotional vectors with 3 temporal layers (emotion/mood/temperament per the ALMA model from Gebhard 2005), 8 octant states, OCC appraisal theory for event evaluation, Scherer's component process model, somatic markers (Damasio's hypothesis) for decision shortcuts via k-d tree lookup with 15% contrarian blending, 6 behavioral states, Nietzsche vitality phases (Camel/Lion/Child), and Bower's mood-congruent memory retrieval. | MEDIUM | ~300-1,500 LOC |
| [04](04-dag-execution.md) | **DAG Execution Engine** | TOML-defined directed acyclic graph workflows with Cell (universal computation unit) registry via factory pattern, conditional edges (always/on-success/on-failure/when-predicate with deep-path field evaluation), budget tracking (tokens + cost + wall-clock deadline), hot graphs (tick-driven resident execution for long-running processes), plan-to-graph conversion pipeline, and 5 built-in Cell implementations. Replaces linear job chaining with declarative, parallelizable workflows. | HIGH | ~2,300-2,800 LOC |
| [05](05-gate-verification.md) | **Gate Verification Pipeline** | 7-rung progressive verification (compile -> lint -> test -> symbol check -> generated test -> property test -> integration test) with complexity-driven rung selection, adaptive thresholds via 3 statistical methods (CUSUM, EWMA, BOCPD), a process reward model (promise + progress scoring), gate composition operators (parallel/voting/fallback), forensic causal chain reconstruction, acceptance contracts, agent feedback filtering, Hotelling's T-squared joint anomaly detection, and PELT offline change point detection. | HIGH | ~1,700-2,300 LOC |
| [06](06-conductor-anomaly.md) | **Conductor Anomaly Detection** | Ensemble of 10 watchers (latency, error rate, token usage, cost, quality, throughput, saturation, availability, drift, coherence) with predictive circuit breaking via Holt exponential smoothing, compound pattern detection (CEP), Thompson Sampling for adaptive threshold learning, Yerkes-Dodson pressure framework (performance degrades at both too-low and too-high arousal), 4-level federation hierarchy (turn/session/agent/fleet), self-healing with oscillation detection and stabilization mode, a diagnosis engine, health monitor, stuck detection, and routing bias based on provider health. | MEDIUM | ~2,100-2,600 LOC |
| [07](07-online-learning.md) | **Online Learning (Cascade Router)** | LinUCB contextual bandit with 18-dimensional context vectors for LLM model selection. 3-stage cascade (static rules -> confidence check -> UCB selection). Pareto frontier optimization across 4 dimensions (pass rate, cost, latency, reliability). Per-provider circuit breakers with Beta-Binomial Bayesian confidence. EWMA anomaly detection. Episode logging and routing audit trail. Curriculum ordering. Active inference tier selection. Projects 30-50% LLM cost reduction. | HIGH | ~1,650-1,900 LOC |
| [08](08-chain-reputation.md) | **On-Chain Reputation and Trust** | Soulbound NFT identity passports (ERC-8004) with 4 capability tiers, 7-domain EMA reputation (code quality, reliability, accuracy, creativity, collaboration, security, efficiency) with 30-day half-life decay, bounty marketplace with 3 hiring models (first-come/competitive/collaborative), TraceRank (PageRank for agent reputation with collusion detection), X402 micropayments protocol, ISFR weighted-median oracle, and KORAI token economics with 1% demurrage. 13 Solidity contracts. | MEDIUM | ~1,100-1,500 LOC |
| [09](09-budget-composition.md) | **Budget-Constrained Composition** | VCG auction-based prompt assembly where subsystems (skills, memory, tools, history) bid for context window space. 9-layer cache-aware prompt builder with strategic U-shaped placement (important content at beginning/end, following the "Lost in the Middle" research). Thompson Sampling learning bidders, 3 context tiers (surgical/focused/full), multi-patch foraging via active inference (Expected Free Energy from Friston's Free Energy Principle), cost attribution, and budget prediction. | MEDIUM | ~1,100-1,500 LOC |
| [10](10-universal-engram.md) | **Universal Engram** | Content-addressed (BLAKE3) universal data type with 7-axis scoring (confidence, novelty, utility, reputation, precision, salience, coherence), 4 decay variants (none, half-life, TTL, Ebbinghaus forgetting curve with spaced-repetition strengthening), lineage tracking (parent DAG forming a provenance graph), 8 taint types with propagation rules (unverified, external, user-generated, LLM-generated, sensitive, ephemeral, derived, disputed), 30+ Kind variants, Signal/Pulse duality, Ed25519 attestation, and demurrage economics. | HIGH | ~750-850 LOC + DB migration |
| [11](11-mathematical-primitives.md) | **Mathematical Primitives** | Five mathematical frameworks: topological data analysis (Takens delay embedding, Vietoris-Rips persistent homology, persistence landscapes/barcodes for execution trace shape analysis), cellular sheaves (multi-source consistency checking via Sheaf Laplacian and coboundary operator), Riemannian geometry (cost manifold optimization with geodesics for configuration navigation), tropical algebra (max-plus semiring for DAG critical path analysis), and robust statistics (trimmed mean, MAD, Hodges-Lehmann estimator for outlier-resistant metrics). | LOW | ~100-2,000+ LOC |
| [12](12-code-intelligence.md) | **Code Intelligence** | 4-mode hybrid code indexing: symbol index (functions, structs, traits, exports), graph index (dependency graph with 5 edge types + PageRank for importance ranking), HDC index (structural fingerprints for similar-code search and clone detection), and FTS5 full-text search. Results merged via RRF. Privacy/context overlays for redaction and boosting. Spans 4 crates: roko-core (traits), roko-index (engine), roko-lang-rust/typescript/go (language providers). | MEDIUM | ~2,500-3,000+ LOC |
| [13](13-cognitive-architecture.md) | **Cognitive Architecture** | Three cognitive speeds (Gamma/reactive ~5-15s for tool calls, Theta/reflective ~75s for planning, Delta/consolidation ~hours for learning). 5-layer architecture (Runtime -> Framework -> Scaffold -> Harness -> Orchestration) with strict downward dependencies derived from protocol topological sort. Stigmergic coordination via 7 pheromone types with SINR interference prevention. C-factor collective intelligence measurement (5 axes). Morphogenetic agent specialization via Turing reaction-diffusion patterns. The core thesis: "the scaffold IS the product." | HIGH | ~200-500 LOC core |
| [14](14-runtime-infrastructure.md) | **Runtime Infrastructure** | EventBus with bounded replay ring buffer and filtered subscriptions. Hierarchical cancellation tokens (session -> job -> tool call, with cascading cancel that respects ancestry). FIPA-informed lifecycle state machine with Kubernetes-style probes (liveness/readiness/startup). Pure state machine + effect driver separation (all logic is a pure function from state + event to state + effects; effects are executed by an impure driver). Process supervision for OS-level subprocess management. StateHub dashboard projections for unified system state views. | MEDIUM | ~200-1,000+ LOC |
| [15](15-orchestrator-swarm.md) | **Orchestrator and Swarm** | Pure state machine orchestrator with event sourcing (all state transitions captured as replayable events in a journal). Unified cross-plan task DAG merging multiple plans into a single dependency graph. Wave scheduling (compute wavefronts of independent tasks for parallel execution). 3-level recovery (task retry -> subgraph replacement -> full replan). File-conflict inference for safe parallelism (two tasks touching the same file cannot run in parallel). BLAKE3 hash-linked audit chain for tamper-evident history. Live DAG mutation (add/remove tasks while the DAG is executing). Pheromone-based swarm coordination for multi-agent systems. | MEDIUM | ~1,900-2,300 LOC |
| [16](16-plugin-extension.md) | **Plugin and Extension System** | EventSource trait (push-based event streaming from external sources like file watchers and cron schedulers). FeedbackCollector trait (structured feedback collection from users or evaluators). 4-tier extensibility model in v1 (prompts -> profiles -> declarative TOML tools -> full SDK) expanding to 5-tier in v2. TOML manifests with typed permission declarations. Filesystem hot-reload with debouncing. v2 extension system design: 8 layers, 22 hooks, 6 decision enums. v2 trigger system for event-driven Cell activation. Composable scorers (add/multiply/threshold/cap/fallback). Role-based profiles. | LOW | ~900-1,100 LOC |
| [17](17-agent-patterns.md) | **Agent Patterns** | 10 design patterns for agent loops: Translator (pluggable wire format abstraction across LLM providers), streaming event reassembly (reconstructing structured events from streaming token output), resumable checkpoints (serialize agent state for crash recovery), metacognitive monitor (stuck loop detection, contradiction detection, cost runaway detection), harness adapter (uniform wrapping of external tools), composable scorers (add/multiply/threshold/cap/fallback combinators), task runner with budget guardrails, retry policy with classified errors (transient/permanent/overload), agent composition operators (sequential/parallel/race/fallback), and warm session reuse with resume validation. | MEDIUM | ~400-1,200 LOC |

### Strategy Documents (18-19)

These two documents synthesize the concept analyses into actionable implementation plans.

| # | Title | Summary |
|---|-------|---------|
| [18](18-integration-roadmap.md) | **Integration Roadmap** | 4-phase adoption plan: Phase 1 (quick wins, 1-2 weeks each -- robust statistics, metacognitive monitor, Ebbinghaus decay, BLAKE3 dedup). Phase 2 (core enhancements, 2-4 weeks -- cascade router, HDC, gate pipeline rungs 1-4, enhanced heartbeat/dreams). Phase 3 (architecture evolution, 4-8 weeks -- DAG engine, conductor, cognitive speeds, full dreams). Phase 4 (advanced features, ongoing -- NEAR identity, pheromones, code intelligence, affect engine, VCG auction). Includes dependency graph between features and IronClaw system context. |
| [19](19-priority-matrix.md) | **Priority Matrix** | Impact vs. effort quadrant ranking of all 25+ concepts, scored on 5 axes (user impact 30%, system impact 20%, implementation effort 25%, risk 15%, dependency burden 10%). Stars (high impact, low effort): metacognitive monitor, Ebbinghaus decay, BLAKE3 dedup, robust stats. Big bets (high impact, high effort): cascade router, DAG execution, full dreams, gate pipeline. Nice-to-have: composable scorers, cognitive speed labels, declarative TOML tools. Long-term: on-chain reputation, full affect engine, TDA/sheaves, code intelligence, VCG auction, pheromones. Estimated ~5,250-5,950 LOC for top 10 features. |

### Deep-Dive Reference Documents (20-28)

These 9 documents provide additional depth on specific Roko subsystems, its broader architecture, research foundations, and implementation plans. They were produced after the initial concept analyses (01-19) to fill gaps identified during review.

| # | Title | Summary |
|---|-------|---------|
| [20](20-persistence-storage.md) | **Persistence and Storage Layer** | Complete technical analysis of Roko's `roko-fs` crate -- the filesystem-backed persistence layer. Covers why append-only JSONL was chosen over traditional databases (crash safety by construction, human readability, no WAL complexity), the signal substrate (typed JSONL stores with in-memory indices), append-log architecture with replay-on-startup, file-level locking, compaction strategies, and the tradeoffs vs. IronClaw's dual-backend database approach (PostgreSQL + libSQL). Concludes that IronClaw's database approach is stronger for production but Roko's content-addressing and decay semantics should be adopted. |
| [21](21-mcp-editor-integration.md) | **ACP and MCP Editor Integration** | Complete technical reference to Roko's Agent Communication Protocol (ACP) and its five MCP server crates. Covers the ACP JSON-RPC 2.0 wire protocol (13 method types), session management with typed state machines, the workflow pipeline state machine (Draft -> InReview -> Implementing -> Verifying -> Complete), streaming session updates via SSE, permission-based action gates, per-session MCP server integration, the 5 MCP crates (tools, context, prompts, resources, bridge), builtin tool system, event bridging and forwarding, configuration hot-reload, and knowledge injection. Analysis of what IronClaw could adopt: structured ACP-style session protocol, MCP server exposure (IronClaw currently only has MCP client), permission gates. |
| [22](22-language-support.md) | **Multi-Language Code Analysis** | Deep analysis of Roko's structural code analysis system spanning `roko-core` (trait contracts), `roko-index` (analysis engine), and 3 language provider crates (`roko-lang-rust`, `roko-lang-typescript`, `roko-lang-go`). Covers the `BuildSystem` and `LanguageProvider` trait abstractions, dual-mode Rust parser (heuristic regex for speed, tree-sitter for accuracy), TypeScript provider with tsconfig resolution, Go provider with module graph parsing, polyglot project detection pipeline, symbol extraction, dependency edge classification (5 edge types), and integration with the graph/HDC/FTS indexing from doc 12. |
| [23](23-control-plane.md) | **Control Plane and API Server** | Architecture of Roko's separation between control plane and agent runtime. Covers the centralized HTTP server (~85 REST routes across 7 functional groups: agent management, plan execution, task operations, model/provider management, system operations, knowledge/memory, event streaming), per-agent HTTP sidecars with relay bus aggregation, WebSocket and SSE streaming, the agent roster/registry pattern, credential injection via sidecars, health probes, and how IronClaw could adopt a structured control plane for its existing webhook/gateway architecture. |
| [24](24-smart-contracts.md) | **Smart Contract Architecture** | Walkthrough of Roko's 13 Solidity contracts for AI agent economic infrastructure: MockERC20 (test token), RoleRegistry (access control), AgentRegistry (lightweight identity), IdentityRegistry (ERC-8004 soulbound passports), WorkerRegistry (stake + reputation + tiers), ReputationRegistry (7-domain scores), BountyMarket (programmable escrow with 3 hiring models), ConsortiumValidator (2-of-3 validation committee), DataFeed (oracle with freshness guarantees), FeeDistributor (proportional revenue sharing), IdentityPassport (ERC-721 with 8004 soulbound extension), ReputationModule (standalone reputation logic), and KnowledgeRegistry (content-addressed knowledge with bounties). Analysis of NEAR equivalents for each contract. |
| [25](25-research-citations.md) | **Research Citations Bibliography** | Comprehensive bibliography of every academic and technical reference found across the Roko codebase, organized by 18 topic areas: HDC/VSA, dream consolidation, affect/emotion, active inference, contextual bandits, statistical process control, TDA, VCG auctions, stigmergic coordination, memory consolidation, cybernetics, cognitive architectures, collective intelligence, dual-process cognition, mechanism design, graph algorithms, information theory, and philosophy. For each reference: the original paper, the concept it introduces, how Roko adapts it, which crate implements it, and relevance to IronClaw. |
| [26](26-roko-architecture-overview.md) | **Roko Architecture Overview** | Comprehensive first-time-reader reference for the entire Roko system. Covers vision and design philosophy ("the scaffold IS the product"), the Synapse architecture (five-primitive model: Signal, Cell, Graph, Bus, Store), the five-layer model (L0 Runtime through L4 Orchestration), crate dependency graph, data flow through the system, the universal cognitive loop (observe-orient-decide-act-learn), key data types, agent lifecycle and coordination, gate pipeline, learning subsystems, knowledge/memory (Neuro), affect engine (Daimon), offline consolidation (Dreams), architectural decisions, deployment model, security model, and comparison with IronClaw. |
| [27](27-v2-depth-research.md) | **v2-Depth Research Catalog** | Catalog of all 155 depth documents (plus 30 support files) across 23 thematic sections in Roko's `docs/v2-depth/` directory. These represent the algorithmic and theoretical layer beneath Roko's unified specification -- 422 source documents totaling 8.8MB absorbed into a structured knowledge base. Organized into sections (signal algebra, demurrage economics, composition operators, graph execution, agent runtime, telemetry, learning loops, memory/dreams/stigmergy, connectivity, tools/MCP, configuration, marketplace, UI/UX, security, registries, deployment, research foundations, code intelligence). Includes top 20 must-read documents and IronClaw relevance groupings. |
| [28](28-plans-catalog.md) | **Implementation Plans Catalog** | Catalog of every implementation plan in Roko's `plans/` directory -- 27 TOML-defined plans (P08-P34) organized into 12 groups (bug fixes, plan runner infrastructure, resilience, safety gates, CLI/TUI, ACP/editor integration, PRD/workspace tooling, learning/intelligence, provider/model UX, onboarding, note workflow, verification sweep). Each plan has tasks with tier classification (mechanical/focused/integrative/architectural), model hints, LoC budgets, file targets, dependency chains, read-context, anti-patterns, and multi-phase verification commands. Cross-plan patterns and novel approaches highlighted. Most relevant for IronClaw: the task TOML format, tier classification system, verification command patterns, and anti-pattern documentation. |

---

## How to Read This Collection

### For the Implementer: "I need to build something this week"

Start with the decision framework, then go to the specific concept:

1. **[19 - Priority Matrix](19-priority-matrix.md)** -- See the impact/effort quadrant. The "Stars" quadrant (high impact, low effort) is your starting point.
2. **[18 - Integration Roadmap](18-integration-roadmap.md)** -- Phase 1 items (robust stats, metacognitive monitor, Ebbinghaus decay, BLAKE3 dedup) are all <500 lines each with zero dependencies.
3. Then read the specific concept document for whatever you are building:
   - Metacognitive monitor -> [17 - Agent Patterns](17-agent-patterns.md) (Pattern 4)
   - Ebbinghaus decay or BLAKE3 dedup -> [10 - Universal Engram](10-universal-engram.md)
   - Robust statistics -> [11 - Mathematical Primitives](11-mathematical-primitives.md) (Section 5)
   - Cascade router -> [07 - Online Learning](07-online-learning.md)

### For the Architect: "I need to understand what is possible"

Start with the big picture, then drill into the novel subsystems:

1. **[26 - Architecture Overview](26-roko-architecture-overview.md)** -- Understand Roko's overall structure, the five-layer model, and how the subsystems compose.
2. **[13 - Cognitive Architecture](13-cognitive-architecture.md)** -- The theoretical framework: three cognitive speeds, five layers, pheromone coordination, collective intelligence.
3. **[04 - DAG Execution](04-dag-execution.md)** -- How workflows are defined and executed. This is the backbone of Roko's orchestration.
4. **[15 - Orchestrator and Swarm](15-orchestrator-swarm.md)** -- Multi-agent coordination, event sourcing, wave scheduling.
5. **[23 - Control Plane](23-control-plane.md)** -- How the system is observed and controlled at runtime.
6. **[14 - Runtime Infrastructure](14-runtime-infrastructure.md)** -- The runtime primitives everything else is built on.
7. **[18 - Integration Roadmap](18-integration-roadmap.md)** -- The 4-phase adoption plan with dependency ordering.

### For the Researcher: "I want to understand the novel ideas"

Start with the most intellectually distinctive concepts:

1. **[01 - Hyperdimensional Computing](01-hyperdimensional-computing.md)** -- The most novel data structure. Binary vectors for near-instant semantic similarity with no model inference.
2. **[02 - Dream Consolidation](02-dream-consolidation.md)** -- Biologically-inspired offline learning modeled on NREM/REM sleep neuroscience.
3. **[03 - Affect Engine](03-affect-engine.md)** -- Emotional modeling with PAD vectors, somatic markers, and Nietzsche's vitality phases.
4. **[10 - Universal Engram](10-universal-engram.md)** -- Content-addressed knowledge with 7-axis scoring, forgetting curves, and taint propagation.
5. **[09 - Budget-Constrained Composition](09-budget-composition.md)** -- VCG auction theory applied to context window allocation.
6. **[25 - Research Citations](25-research-citations.md)** -- The complete bibliography: every academic paper behind every concept.
7. **[27 - v2-Depth Research Catalog](27-v2-depth-research.md)** -- The 155-document research layer beneath the specification.

### For the Cost Optimizer: "I want to reduce LLM spend"

1. **[07 - Online Learning](07-online-learning.md)** -- The single highest-ROI document. Contextual bandit model routing projects 30-50% cost reduction.
2. **[09 - Budget-Constrained Composition](09-budget-composition.md)** -- VCG auction for context window allocation. Cache-aware prompt ordering directly reduces API costs via prompt caching.
3. **[06 - Conductor](06-conductor-anomaly.md)** -- Predictive circuit breaking prevents wasting tokens on degraded providers.

### For the Memory/Knowledge Engineer: "I want better memory quality"

1. **[10 - Universal Engram](10-universal-engram.md)** -- Multi-axis scoring, decay variants, content-addressed deduplication, lineage tracking, source tainting.
2. **[02 - Dream Consolidation](02-dream-consolidation.md)** -- Confidence staging (Raw -> Replayed -> Validated -> Promoted), NREM replay for strengthening, hypnagogic creativity for cross-domain insights.
3. **[01 - Hyperdimensional Computing](01-hyperdimensional-computing.md)** -- HDC fingerprints for novelty detection and redundancy prevention in memory writes.
4. **[20 - Persistence and Storage](20-persistence-storage.md)** -- Append-only JSONL design, content-addressing patterns, compaction strategies.

### For the Reliability Engineer: "I want fewer failures"

1. **[06 - Conductor](06-conductor-anomaly.md)** -- 10-watcher ensemble with predictive circuit breaking, oscillation detection, compound pattern detection.
2. **[17 - Agent Patterns](17-agent-patterns.md)** -- Metacognitive monitor (stuck loop detection, cost runaway detection), resumable checkpoints for crash recovery.
3. **[14 - Runtime Infrastructure](14-runtime-infrastructure.md)** -- Hierarchical cancellation tokens, EventBus with replay, FIPA lifecycle with health probes.
4. **[05 - Gate Verification](05-gate-verification.md)** -- Progressive verification with adaptive thresholds and forensic causal chains.

### For the Code Generation Engineer: "I want better code output"

1. **[05 - Gate Verification](05-gate-verification.md)** -- 7-rung progressive verification with complexity-driven rung selection and adaptive thresholds.
2. **[12 - Code Intelligence](12-code-intelligence.md)** -- Multi-modal code indexing with PageRank, HDC fingerprints, and RRF hybrid search.
3. **[22 - Language Support](22-language-support.md)** -- Structural code analysis with language providers for Rust, TypeScript, and Go.
4. **[04 - DAG Execution](04-dag-execution.md)** -- Declarative TOML workflows for multi-step code generation pipelines.

### For the Blockchain/Identity Engineer: "I want on-chain agent infrastructure"

1. **[24 - Smart Contracts](24-smart-contracts.md)** -- Full walkthrough of all 13 Solidity contracts with NEAR port analysis.
2. **[08 - On-Chain Reputation](08-chain-reputation.md)** -- Soulbound passports, 7-domain reputation, TraceRank, bounty marketplace, KORAI tokenomics.

### Complete Ordered Reading (All 28 Documents)

If you want to read everything, this order minimizes forward references:

1. [26](26-roko-architecture-overview.md) -- Architecture overview (provides context for everything)
2. [10](10-universal-engram.md) -- Universal Engram (the core data type referenced everywhere)
3. [01](01-hyperdimensional-computing.md) -- HDC (referenced by code intelligence, dreams, composition)
4. [13](13-cognitive-architecture.md) -- Cognitive architecture (the organizational framework)
5. [04](04-dag-execution.md) -- DAG execution (the workflow backbone)
6. [05](05-gate-verification.md) -- Gate verification (quality assurance for all outputs)
7. [07](07-online-learning.md) -- Online learning / cascade router
8. [02](02-dream-consolidation.md) -- Dream consolidation
9. [03](03-affect-engine.md) -- Affect engine
10. [06](06-conductor-anomaly.md) -- Conductor anomaly detection
11. [09](09-budget-composition.md) -- Budget-constrained composition
12. [11](11-mathematical-primitives.md) -- Mathematical primitives
13. [12](12-code-intelligence.md) -- Code intelligence
14. [14](14-runtime-infrastructure.md) -- Runtime infrastructure
15. [15](15-orchestrator-swarm.md) -- Orchestrator and swarm
16. [16](16-plugin-extension.md) -- Plugin and extension system
17. [17](17-agent-patterns.md) -- Agent patterns
18. [08](08-chain-reputation.md) -- On-chain reputation
19. [20](20-persistence-storage.md) -- Persistence and storage
20. [21](21-mcp-editor-integration.md) -- ACP and MCP editor integration
21. [22](22-language-support.md) -- Multi-language code analysis
22. [23](23-control-plane.md) -- Control plane and API server
23. [24](24-smart-contracts.md) -- Smart contract architecture
24. [25](25-research-citations.md) -- Research citations bibliography
25. [27](27-v2-depth-research.md) -- v2-depth research catalog
26. [28](28-plans-catalog.md) -- Implementation plans catalog
27. [18](18-integration-roadmap.md) -- Integration roadmap
28. [19](19-priority-matrix.md) -- Priority matrix (best read last, after understanding all concepts)

---

## Quick Reference: Concepts to Implementation

This table maps each major concept to its IronClaw integration point, the document with full details, and the type of change required.

### Drop-In Enhancements (Modify Existing Modules)

| Concept | IronClaw Target | Document | What Changes |
|---------|----------------|----------|-------------|
| Ebbinghaus memory decay | `src/workspace/` | [10](10-universal-engram.md) | Add decay field to memory entries. Accessed entries strengthen; unaccessed ones fade. DB migration required. |
| BLAKE3 content dedup | `src/workspace/` | [10](10-universal-engram.md) | Hash memory content before writing. Merge duplicate entries instead of creating new ones. |
| Robust statistics | `src/estimation/`, `src/evaluation/` | [11](11-mathematical-primitives.md) | Replace mean/stddev with trimmed mean and MAD (Median Absolute Deviation). ~100 lines. |
| Metacognitive monitor | `src/agent/` | [17](17-agent-patterns.md) | Add stuck-loop detection (repeated tool calls), contradiction detection, and cost-runaway detection to the agent loop. |
| Cognitive speed labels | `src/agent/` | [13](13-cognitive-architecture.md) | Label existing reactive/reflective/background splits as Gamma/Theta/Delta. Formalize timing expectations. |
| Enhanced heartbeat | `src/agent/heartbeat.rs` | [02](02-dream-consolidation.md) | Upgrade heartbeat from "read a file" to "replay experiences, rehearse failures, generate insights." |
| U-shaped prompt placement | `crates/ironclaw_engine/` | [09](09-budget-composition.md) | Place high-priority content at beginning and end of context window (not just beginning). |
| Hierarchical cancellation | `src/agent/`, `src/tools/` | [14](14-runtime-infrastructure.md) | Add parent-child cancellation tokens so canceling a session cascades to jobs, tool calls, and subprocesses. |

### New Crate Additions

| Concept | Proposed Crate | Document | What It Provides |
|---------|---------------|----------|-----------------|
| Hyperdimensional computing | `crates/ironclaw_hdc/` | [01](01-hyperdimensional-computing.md) | Sub-millisecond similarity search for memory, skills, tools. Drop-in alongside existing vector search. |
| Cascade router | `crates/ironclaw_router/` | [07](07-online-learning.md) | Bandit-based LLM model selection. Learns which provider to use per request type. 30-50% cost savings. |
| DAG execution | `crates/ironclaw_graph/` | [04](04-dag-execution.md) | Declarative TOML workflows replacing linear job chaining. Enables parallel task execution. |
| Gate pipeline | `crates/ironclaw_gate/` | [05](05-gate-verification.md) | Progressive verification for tool builder output and code generation. |
| Dream consolidation | `crates/ironclaw_dreams/` | [02](02-dream-consolidation.md) | Background offline learning during idle periods. Natural extension of heartbeat. |
| Reputation tracking | `crates/ironclaw_reputation/` | [08](08-chain-reputation.md) | NEAR agent identity, extension/tool reputation, trust tiers. |

### Module Enhancements in Existing Crates

| Concept | Target Crate | Document | What It Provides |
|---------|-------------|----------|-----------------|
| Conductor monitoring | `crates/ironclaw_llm/` | [06](06-conductor-anomaly.md) | LLM provider health monitoring with predictive circuit breaking. |
| Somatic markers | `src/profile.rs` | [03](03-affect-engine.md) | Decision shortcuts based on cached emotional responses to similar contexts. |
| Code intelligence | `src/workspace/` or new crate | [12](12-code-intelligence.md), [22](22-language-support.md) | Structural code understanding for workspace, impact analysis. |
| Event sourcing | `src/agent/`, `src/db/` | [15](15-orchestrator-swarm.md) | Capture all state transitions as replayable events. Crash recovery, debugging, auditing. |
| MCP server exposure | `src/tools/mcp/` | [21](21-mcp-editor-integration.md) | Expose IronClaw tools as MCP servers (currently only MCP client). |
| Pheromone coordination | `src/workspace/` | [13](13-cognitive-architecture.md) | Cross-session learning via typed, decaying markers. |

---

## Novel Concepts Not Found in IronClaw

These are capabilities Roko has that IronClaw currently lacks entirely.

### Tier 1 -- High Novelty, Strong Fit

| Concept | Roko Source | What It Enables | IronClaw Gap |
|---------|------------|-----------------|-------------|
| **Hyperdimensional computing** | `roko-primitives`, `roko-neuro` | Sub-millisecond similarity search across 100K+ items with no model inference. Compositional: bind two concepts and search for the combination. | IronClaw has FTS + vector search but no compositional, inference-free similarity engine. |
| **Dream consolidation** | `roko-dreams` | Autonomous offline learning: replay experiences, generate counterfactuals, discover cross-domain insights, rehearse failures. | IronClaw's heartbeat reads a static HEARTBEAT.md file. No replay, no counterfactual generation, no insight synthesis. |
| **Cascade model routing** | `roko-learn` | Learns which LLM to use per request type via contextual bandits. Projects 30-50% cost savings. | IronClaw routes to a single configured provider. No per-request model selection, no learning from outcomes. |
| **Gate verification pipeline** | `roko-gate` | 7-rung progressive verification with complexity-driven rung selection and adaptive thresholds. | IronClaw's tool builder has basic validation. No progressive verification, no adaptive thresholds, no forensic causal chains. |
| **Content-addressed engrams** | `roko-core` | Universal BLAKE3-hashed data type with 7-axis scoring, 4 decay variants, lineage DAG, and taint propagation. | IronClaw memory entries are key-value with namespace/tags. No content addressing, no decay, no lineage, no taint tracking. |
| **Metacognitive monitor** | `roko-agent`, `roko-std` | Detects stuck loops, contradictions, and cost runaway in the agent loop. | IronClaw has no self-monitoring for agent pathologies. |

### Tier 2 -- High Novelty, Requires More Design

| Concept | Roko Source | What It Enables |
|---------|------------|-----------------|
| **Stigmergic pheromones** | `roko-core`, docs | Indirect multi-session coordination via typed, decaying markers (threat, opportunity, wisdom, alpha, pattern, anomaly, consensus). |
| **DAG execution engine** | `roko-graph` | TOML-defined workflows with conditional routing, budget tracking, and hot (resident) graphs. |
| **Predictive circuit breaking** | `roko-conductor` | Holt exponential smoothing forecasts provider degradation before it causes failures. |
| **Somatic markers** | `roko-daimon` | Decision shortcuts: 8D strategy space + k-d tree retrieval + 15% contrarian blending to avoid local optima. |
| **Event sourcing** | `roko-orchestrator` | All orchestrator state transitions captured as replayable events. Enables crash recovery, debugging, auditing. |
| **VCG auction for context** | `roko-compose` | Subsystems bid truthfully for context window space. Thompson Sampling learns which content is most valuable. |
| **ACP session protocol** | `roko-acp` | Structured JSON-RPC 2.0 bidirectional protocol for editor integration with streaming updates and permission gates. |
| **Control plane separation** | `roko-serve` | ~85-route HTTP API with per-agent sidecars, relay bus, and fleet-level aggregation. |

### Tier 3 -- Specialized / Research-Grade

| Concept | Roko Source | What It Enables |
|---------|------------|-----------------|
| **TDA / sheaves / Riemannian geometry** | `roko-primitives` | Topological analysis of execution traces, multi-source consistency checking, optimal configuration paths. |
| **Morphogenetic specialization** | docs | Turing reaction-diffusion patterns for emergent role assignment in multi-agent swarms. |
| **C-factor measurement** | docs | 5-axis collective intelligence metric (turn-taking equality, peer prediction, citation reciprocity, delivery rate, HDC diversity). |
| **KORAI token economics** | `roko-chain` | 1% lazy demurrage, X402 micropayments, ISFR weighted-median oracle. |
| **TraceRank** | `roko-chain` | PageRank-based reputation with collusion detection. |
| **Multi-language structural parsing** | `roko-lang-*` | Dual-mode parsers (heuristic + tree-sitter) for Rust, TypeScript, Go with polyglot project detection. |
| **Self-development plans** | `plans/` | TOML-defined implementation plans with tier classification, model hints, LoC budgets, anti-patterns, and multi-phase verification. |

---

## What IronClaw Already Does Well

These are capabilities IronClaw has that Roko does not, or where IronClaw's implementation is notably stronger.

### Security and Safety

| Capability | IronClaw | Roko |
|-----------|---------|------|
| **WASM sandboxing** | Full wasmtime sandbox with fuel metering, memory limits, network allowlisting, credential injection. | No WASM sandboxing -- tools run natively. |
| **Docker execution sandbox** | Per-project containers with bind-mount isolation, sandbox daemon, 3-tier policy (ReadOnly/WorkspaceWrite/FullAccess). | No container sandboxing for code execution. |
| **Safety pipeline** | Extracted `ironclaw_safety` crate: prompt injection detection, validation, leak detection, policy enforcement. | Basic safety layer (role auth, pre/post checks) but less mature. |
| **Network proxy** | Domain allowlist proxy with credential injection and CONNECT tunnel for sandbox containers. | No network proxy or allowlisting. |
| **Secrets management** | AES-256-GCM encrypted secrets with OS keychain for master key. | Simple dotenv-based secrets. |

### Multi-Channel Access

| Capability | IronClaw | Roko |
|-----------|---------|------|
| **Channel diversity** | CLI/TUI, web gateway (browser UI), HTTP webhooks, WASM channels, Telegram, Gmail -- all unified through Channel trait. | CLI + HTTP API only. No chat channels. |
| **Webhook server** | Unified HTTP server composing all webhook routes with secret validation. | Webhook ingestion but no channel abstraction. |
| **WASM channels** | Channels can be implemented as WASM modules with bundled discovery and capability declarations. | No channel extensibility. |

### Extensibility Model

| Capability | IronClaw | Roko |
|-----------|---------|------|
| **Extension registry** | Full catalog with manifests, artifact specs, installer (download, verify, install WASM artifacts). | Plugin system with TOML manifests but no registry/installer. |
| **MCP integration** | Full MCP client with HTTP/stdio/Unix transports, session management, factory dispatch. | MCP config passthrough to Claude CLI; 5 MCP server crates but no client consumption. |
| **Lifecycle hooks** | 6 hook points (BeforeInbound, BeforeToolCall, BeforeOutbound, OnSessionStart, OnSessionEnd, TransformResponse). | No lifecycle hook system. |
| **Tool dispatch principle** | "Everything goes through tools" -- all actions route through ToolDispatcher for audit trail, safety, and channel-agnostic dispatch. Pre-commit hook enforces this. | Tools exist but no enforcement of universal dispatch. |

### Persistence and Memory

| Capability | IronClaw | Roko |
|-----------|---------|------|
| **Dual-backend database** | PostgreSQL + libSQL/Turso. All features support both backends. | Append-only JSONL files. No database. |
| **Hybrid memory search** | FTS + vector embedding search merged via Reciprocal Rank Fusion (RRF). | Signal substrate with basic querying. |
| **Identity files** | AGENTS.md, SOUL.md, USER.md, IDENTITY.md injected into system prompt. | Role templates but no persistent identity files. |

### Operational Features

| Capability | IronClaw | Roko |
|-----------|---------|------|
| **Tunnel abstraction** | Pluggable tunnel providers (Cloudflare, ngrok, Tailscale, custom command) for public internet exposure. | No tunnel abstraction. |
| **Service management** | OS service management (launchd/systemd) for daemon installation. | Basic daemon with launchd support. |
| **Psychographic profiling** | 9-dimension user profile analysis framework. | PAD vectors (3 dimensions) -- different approach, less user-focused. |
| **Cost/time/value estimation** | EMA-learning estimation module. | Efficiency tracking but no predictive estimation. |

---

## Recommended Next Steps

### Immediate (This Sprint)

These require <500 lines each, have zero dependencies, and deliver measurable value:

1. **Metacognitive Monitor** (doc 17) -- Add stuck-loop detection and cost-runaway detection to the agent loop. Prevents the most common agent failure mode.
2. **Ebbinghaus Decay for Memory** (doc 10) -- Add spaced-repetition decay to memory entries. Frequently-accessed memories strengthen; unused ones fade.
3. **BLAKE3 Content Deduplication** (doc 10) -- Hash memory content before writing. Merge instead of duplicating.
4. **Robust Statistics** (doc 11) -- Replace mean/stddev with trimmed mean and MAD in estimation/evaluation modules.

### Near-Term (Next 2-4 Weeks)

These are the highest-ROI medium-effort features:

5. **Cascade Router** (doc 07) -- Bandit-based model selection. The single largest cost-reduction opportunity.
6. **HDC Similarity Engine** (doc 01) -- New `crates/ironclaw_hdc/`. Drop-in similarity for memory, skills, and tools.
7. **Gate Pipeline Rungs 1-4** (doc 05) -- Progressive verification for tool builder and code generation output.
8. **Enhanced Heartbeat** (doc 02) -- NREM replay and threat rehearsal. Upgrade the heartbeat from "read a file" to "learn from experience."

### Medium-Term (Next 1-2 Months)

9. **DAG Execution Engine** (doc 04) -- Declarative workflow orchestration for complex multi-step jobs.
10. **Conductor** (doc 06) -- Production-grade LLM provider health monitoring with predictive circuit breaking.

### See Also

- **[18 - Integration Roadmap](18-integration-roadmap.md)** for the full 4-phase plan with dependency graph.
- **[19 - Priority Matrix](19-priority-matrix.md)** for the complete impact/effort ranking of all 30+ concepts.

---

## Methodology

**Source**: Roko repository at `/Users/will/dev/nunchi/roko/roko/` (18+ crates, ~200K LOC, 1,600+ tests).

**Process**: Each of Roko's 18+ crates was analyzed for novel concepts, architectural patterns, and algorithms not present in IronClaw. Concepts were extracted, documented with concrete code examples and integration proposals, mapped to specific IronClaw modules, and ranked by impact, effort, and risk. Nine additional deep-dive documents (20-28) were produced to cover Roko's persistence layer, editor integration protocols, multi-language code analysis, control plane architecture, smart contract suite, research bibliography, overall architecture, v2 research corpus, and implementation plans. The analysis focused on what is genuinely novel (not just a different implementation of something IronClaw already has) and what would be adoptable as modular components (not requiring wholesale architectural changes).

**Date**: 2026-07-02

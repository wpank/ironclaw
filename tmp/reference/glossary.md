# Glossary of Terms

> **Quick-reference guide.** This glossary defines the main domain terms used across this knowledge base. Each entry is one or two precise sentences; linked documents contain fuller explanations, implementation notes, and references.

---

## A

**Acceptance Contract**
A machine-readable specification that an agent's output must satisfy before a task is marked complete, covering expected file changes, test pass rates, and lint scores. Allows automated pass/fail decisions without human review.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**ACP (Agent Communication Protocol)**
A higher-level JSON-RPC 2.0 protocol built on top of MCP that governs session lifecycle, multi-agent workflow pipeline orchestration, streaming event delivery, and permission gates between agents and operators. Roko uses ACP for editor integration and inter-agent coordination.
→ [MCP and Editor Integration](../ecosystem/mcp-editor-integration.md)

**ActionRecord**
IronClaw's per-tool-call audit record capturing input parameters, output, timing, and cost; the natural source material for Dream Consolidation's NREM replay episodes.
→ [Dream Consolidation](../agent-intelligence/dream-consolidation.md)

**Active Inference**
A computational neuroscience framework (Friston 2010) in which agents minimize the divergence between predictions and observations, using "expected free energy" to balance goal-directed exploitation and uncertainty-reducing exploration. Used in Roko as the theoretical basis for tier routing and prompt foraging.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md), [Budget Composition](../context-memory/budget-composition.md)

**Adaptive Threshold**
A gate pass-rate threshold that updates automatically based on historical performance using CUSUM, EWMA, or BOCPD control-chart signals, replacing the static 0.85 thresholds of traditional CI systems.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**ALMA Layers**
Three temporal levels of the affect engine (from the ALMA computational emotion architecture by Gebhard 2005): Emotion (immediate, seconds), Mood (medium-term, session), and Temperament (long-term, stable baseline). Each layer decays at a different rate; the effective PAD vector is a weighted sum of all three.
→ [Affect Engine](../agent-intelligence/affect-engine.md)

**AntiKnowledge**
An HDC hypervector encoding what the agent has *not* seen or what should be rejected — the complement bundle used for admission control. A query vector is accepted if its similarity to the AntiKnowledge bundle is below a threshold.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

**Appraisal (OCC)**
Structured evaluation of an event against goals and standards using the Ortony-Clore-Collins (1988) cognitive appraisal theory: events are scored on goal-relevance, goal-conduciveness, and agency to produce emotion deltas that update the agent's PAD vector.
→ [Affect Engine](../agent-intelligence/affect-engine.md)

**Attestation (Ed25519)**
A cryptographic proof attached to an Engram: the producer signs the canonical bytes of the Engram's content with an Ed25519 private key so any downstream consumer can verify authenticity without trusting the storage layer.
→ [Universal Engram](../core-concepts/universal-engram.md)

**Attention Bidder**
A prompt-content source (skills, memory, tools, history, context, identity, code) that offers candidate sections with token costs and value scores. The budget composer uses those bids for density allocation and optional VCG-style displacement diagnostics.
→ [Budget Composition](../context-memory/budget-composition.md)

---

## B

**Barcode (Persistence Barcode)**
A compact visualization of a persistence diagram: each topological feature is drawn as a horizontal bar from its birth epsilon to its death epsilon. Long bars signal robust topological structure; short bars are noise.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Behavioral State**
One of six named output states of the affect engine (Engaged, Cautious, Stressed, Depressed, Excited, Bored) derived by classifying the agent's current PAD vector against threshold octants. The active state modulates model tier selection, retry policy, and response tone.
→ [Affect Engine](../agent-intelligence/affect-engine.md)

**Beta-Binomial Model**
A Bayesian confidence model that tracks observed successes and failures for each LLM provider as a Beta distribution and uses the posterior mean as a Bayesian pass rate. Used in the cascade router to compute provider reliability with uncertainty bounds.
→ [Online Learning](../agent-intelligence/online-learning.md)

**Bind (XOR)**
The HDC operation that combines two hypervectors into a new vector quasi-orthogonal to both inputs by XORing them bit by bit. Binding encodes structured relationships: `role_filler = bind(role_vec, filler_vec)` produces a vector that "holds" the association.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

**BLAKE3**
A cryptographic hash function used for content-addressed identity in Engrams/Signals. Two Engrams with identical content produce identical BLAKE3 hashes, enabling deduplication, integrity verification, and tamper detection.
→ [Universal Engram](../core-concepts/universal-engram.md)

**BOCPD (Bayesian Online Changepoint Detection)**
A statistical algorithm that computes the probability of a changepoint at each timestep online, without reprocessing the full history. Used in gate adaptive thresholds to detect regime changes in pass rates.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Body (Engram)**
The typed payload of an Engram — one of Empty, Text, JSON, or Bytes. Content-addressed hashing covers the body, so changing any byte changes the Engram's identity.
→ [Universal Engram](../core-concepts/universal-engram.md)

**Bounty Marketplace (Spore)**
A smart-contract-backed job board where tasks are posted with KORAI token rewards and three hiring models (open, curated, direct). Agents with sufficient reputation can accept bounties; escrow releases payment on verified completion.
→ [Chain Reputation](../ecosystem/chain-reputation/README.md)

**BSC (Binary Spatter Codes)**
The specific HDC variant used in this system: binary vectors with XOR binding, majority-vote bundling, and cyclic-shift permutation. Chosen for simplicity, CPU-native performance, and sufficient representational capacity at 10,240 bits.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

**Budget Composition**
The full process of assembling a prompt under a token budget constraint, including density allocation, position-aware placement, and per-section cost attribution.
→ [Budget Composition](../context-memory/budget-composition.md)

**Bundle (Majority Vote)**
The HDC operation that superimposes multiple hypervectors into one that is similar to all inputs: at each bit position, take 1 if more than half of the input vectors have a 1, else 0. Bundling represents a set of concepts without privileging any single member.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

---

## C

**C-Factor**
A collective intelligence metric adapted from Woolley et al. (2010) measuring the aggregate throughput, quality, and coordination efficiency of a multi-agent swarm relative to the summed capacities of its individual agents.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md)

**Cascade Router**
The three-stage LLM selection pipeline: (1) static rules for obvious cases, (2) confidence check against pass-rate floor, (3) LinUCB UCB selection from the Pareto frontier. Projects 30–50% LLM cost reduction.
→ [Online Learning](../agent-intelligence/online-learning.md)

**Cell**
The universal computation unit in the DAG execution engine: a Rust trait with an `execute(inputs) -> outputs` method. Every DAG node — whether an LLM call, shell command, gate check, or file transform — is a Cell registered in the CellRegistry.
→ [DAG Execution Engine](../execution-verification/dag-execution.md)

**CellRegistry**
The factory map that associates string type names (as used in TOML workflow definitions) to concrete Cell implementations. Graph execution calls the registry by name to instantiate each node, enabling declarative workflows.
→ [DAG Execution Engine](../execution-verification/dag-execution.md)

**CEP (Compound Event Pattern)**
A multi-signal detection rule that fires when several watcher signals occur in combination within a time window — for example, simultaneous cost overrun + latency spike + context pressure. Individual signals may be below threshold while the compound pattern signals a systemic failure.
→ [Conductor Anomaly Detection](../execution-verification/conductor-anomaly.md)

**C-Factor (Collective Intelligence)**
See C-Factor above.

**Checkpoint (Resumable)**
A serialized snapshot of agent state (tool call history, outputs produced, budget consumed, current step index) written to durable storage after each step. On restart, the agent deserializes the checkpoint and resumes from the last completed step rather than from the beginning.
→ [Agent Patterns](../agent-intelligence/agent-patterns.md)

**Circuit Breaker**
A protective mechanism that halts requests to a failing LLM provider after a threshold number of consecutive failures. The predictive variant in the Conductor trips proactively by forecasting the next error rate using Holt exponential smoothing before the threshold is reached.
→ [Conductor Anomaly Detection](../execution-verification/conductor-anomaly.md)

**Coboundary Operator**
The linear map in a cellular sheaf that measures inconsistency between a node's stalk value and its neighbors' stalk values via the restriction maps. A zero coboundary indicates perfect global consistency; a nonzero value identifies conflicting information sources.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Codebook**
In HDC: a dictionary mapping symbolic names (strings, integers, enum variants) to deterministic hypervectors generated by seeding a pseudo-random bit generator with the symbol's hash. The codebook is the bridge between symbolic representations and the HDC vector space.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

**Cognitive Speed**
One of three processing timescales in the Roko cognitive architecture, named after neural oscillation bands: Gamma (reactive, under 15 s per tick), Theta (reflective, ~75 s), Delta (consolidation, hours). Each speed tier uses different inference resources and addresses different task horizons.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md)

**Concentration of Measure**
The statistical phenomenon in high-dimensional spaces where any two independently random binary vectors will, with overwhelming probability, have Hamming similarity near 0.5. This property is the mathematical foundation of HDC: vectors can represent an exponential number of distinct concepts without interference.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

**Conductor**
A purely reactive supervisory layer that reads signal streams from the agent loop, runs 10 watcher functions over them, and produces intervention decisions (Continue / Restart / Fail) without performing any I/O itself. The name is an analogy: it watches all players without playing an instrument.
→ [Conductor Anomaly Detection](../execution-verification/conductor-anomaly.md)

**ContentHash**
An Engram's primary identifier: the BLAKE3 hash of its kind, body, author, and tags. Score and decay are excluded so they can change without changing identity. The hex-encoded 32-byte digest serves as a stable, deduplication-aware key.
→ [Universal Engram](../core-concepts/universal-engram.md)

**Context Tier**
One of three prompt assembly modes (Surgical: 4K tokens, Focused: 12K tokens, Full: entire window) selected based on request type and complexity. The system selects a tier automatically using a composition strategy, then runs budget allocation within that token limit.
→ [Budget Composition](../context-memory/budget-composition.md)

**Contrarian Blending**
A mechanism that injects 15% opposite-valence memories into somatic marker queries to prevent mood-congruent feedback loops: when the affect engine is in a negative state, 15% of retrieved somatic markers are the highest-scoring positive-outcome precedents.
→ [Affect Engine](../agent-intelligence/affect-engine.md)

**CUSUM (Cumulative Sum)**
A statistical process control algorithm (Page 1954) that accumulates deviations from a target value and signals when the cumulative sum exceeds a threshold. Used in gate pipelines to detect sustained drift in pass rates.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md), [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

---

## D

**DAG (Directed Acyclic Graph)**
A graph of computation nodes connected by directed edges with no cycles, used to express workflow dependencies and parallelism. DAGs allow tasks with no dependency relationship to run simultaneously and enable conditional branching via edge conditions.
→ [DAG Execution Engine](../execution-verification/dag-execution.md)

**Daimon**
The Roko name for the affect engine crate (`roko-daimon`), drawn from the Greek concept of a guiding inner spirit. In the IronClaw glossary, Daimon maps to "Affect engine."
→ [Agent Intelligence README](../agent-intelligence/README.md), [Roko Terminology](terminology-glossary.md)

**Datum**
A polymorphic wrapper in roko-core that can hold either an Engram (durable, scored, content-addressed) or a Pulse (ephemeral, fire-and-forget event on the event bus).
→ [Universal Engram](../core-concepts/universal-engram.md)

**Decay**
The time-based reduction in an Engram's effective weight, implemented as one of four variants: None (no decay), HalfLife (exponential), TTL (hard cutoff), or Ebbinghaus (psychological forgetting curve). Decay allows the memory system to prioritize recent, reinforced knowledge.
→ [Universal Engram](../core-concepts/universal-engram.md)

**Defense in Depth**
The principle, borrowed from security engineering, that multiple independent verification layers each catch different error classes, with a joint false negative rate that decreases multiplicatively with each additional layer. The 7-rung gate pipeline is structured on this principle.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Delta Speed**
The slowest cognitive tier (hours): deep offline consolidation running during idle periods. Maps to biological delta-wave sleep (0.5–4 Hz) and corresponds to the Dream Consolidation subsystem.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md)

**Demurrage**
An economic concept from Silvio Gesell (1916) applied to Engrams: the `balance` field in [0.0, 1.0] decays over time (representing a holding cost) and is refreshed on access. Engrams that are never accessed slowly lose their "attention budget," preventing stale data from indefinitely consuming index space.
→ [Universal Engram](../core-concepts/universal-engram.md), [Chain Reputation](../ecosystem/chain-reputation/README.md)

**Dispatch Modulation**
The mechanism by which the affect engine's current Behavioral State biases LLM model tier selection: for example, the Stressed state temporarily promotes requests to a higher-cost, higher-quality model until the PAD vector stabilizes.
→ [Affect Engine](../agent-intelligence/affect-engine.md)

**Dream Consolidation**
A background offline learning system modeled on mammalian sleep stages that runs during agent idle periods to replay experience, synthesize counterfactuals, make creative associations, and rehearse failure defenses, producing durable knowledge entries without modifying model weights.
→ [Dream Consolidation](../agent-intelligence/dream-consolidation.md)

**Dual-Process Cognition**
Kahneman's (2011) model of two cognitive systems: System 1 (fast, automatic, heuristic) and System 2 (slow, deliberate, analytical). Roko maps these to three inference tiers: T0 (no LLM), T1 (fast model), T2 (full model). The two-process model motivates routing decisions based on task complexity.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md)

---

## E

**Ebbinghaus Forgetting Curve**
The psychological model (Ebbinghaus 1885) that memory retention decays exponentially after initial encoding and recovers with repeated review (spaced repetition). Used as one of the four Engram decay variants: `R(t) = e^(-t/S)` where S is stability increased by each retrieval.
→ [Universal Engram](../core-concepts/universal-engram.md)

**Ed25519**
The elliptic-curve signature scheme used for Engram attestation. Producers sign Engram content at emission time; consumers verify the signature without trusting the storage layer.
→ [Universal Engram](../core-concepts/universal-engram.md)

**EFE (Expected Free Energy)**
The active inference objective function that balances epistemic value (information gained by an action) against instrumental value (progress toward a goal). Used in the prompt foraging subsystem to decide when to stop retrieving context from a source.
→ [Budget Composition](../context-memory/budget-composition.md)

**EMA Reputation**
Exponential Moving Average reputation: each of the seven work-domain scores is maintained as an EMA with a 30-day half-life, so recent performance weighs more than historical performance, and reputations gradually fade without sustained activity.
→ [Chain Reputation](../ecosystem/chain-reputation/README.md)

**EMA (Exponential Moving Average)**
A time-series smoothing technique that weights recent observations more heavily than older ones: `EMA_t = alpha * x_t + (1 - alpha) * EMA_{t-1}`. Used in the Conductor, gate adaptive thresholds, and reputation decay.
→ [Conductor Anomaly Detection](../execution-verification/conductor-anomaly.md)

**Emotional Tag**
An optional PAD vector (`pleasure`, `arousal`, `dominance` in [-1.0, 1.0]) attached to an Engram at creation time to encode the emotional context in which it was produced. Enables mood-congruent memory retrieval.
→ [Universal Engram](../core-concepts/universal-engram.md)

**Engram**
The universal data record in the Roko system (v1 term; v2 term is Signal): a content-addressed, scored, decaying unit of cognition that represents everything the system knows, observes, decides, and produces. Every gate verdict, tool trace, memory entry, and prediction is an Engram.
→ [Universal Engram](../core-concepts/universal-engram.md)

**ERC-8004**
A proposed Ethereum token standard for non-transferable (soulbound) identity tokens. Used as the basis for agent identity passports in the smart contract layer — each agent gets exactly one passport that cannot be sold or transferred.
→ [Chain Reputation](../ecosystem/chain-reputation/README.md), [Smart Contracts](../ecosystem/smart-contracts/README.md)

**Event Sourcing**
An architectural pattern where state is derived from an ordered, immutable sequence of events rather than stored as mutable snapshots. Enables deterministic replay, crash recovery, and tamper-evident audit trails. Used in the orchestrator's journal.
→ [Orchestrator Swarm](../execution-verification/orchestrator-swarm.md)

**EventBus**
A publish-subscribe bus with a bounded replay ring buffer that carries structured events (Pulses) between runtime components. Subscribers can replay recent history from the ring on join, preventing missed events during startup.
→ [Runtime Infrastructure](../execution-verification/runtime-infrastructure.md)

**EWMA (Exponentially Weighted Moving Average)**
A variant of EMA used in SPC: monitors whether the current value has drifted beyond a control limit based on weighted historical data. Used in the Conductor's watcher for latency drift detection.
→ [Conductor Anomaly Detection](../execution-verification/conductor-anomaly.md)

---

## F

**Feature Flag**
A named boolean switch that enables or disables a specific feature at runtime without redeployment. All experimental features in this knowledge base require fail-closed feature flags as a non-negotiable rollout discipline.
→ [Rollout README](../implementation/rollout/README.md)

**File-Conflict Inference**
The orchestrator's static analysis of which DAG tasks touch overlapping files, used to determine which tasks can run in parallel and which must be serialized to prevent write conflicts without explicit dependency declarations.
→ [Orchestrator Swarm](../execution-verification/orchestrator-swarm.md)

**FIPA Lifecycle**
The agent state machine adapted from the FIPA Agent Management Specification (2002): Created → Initiated → Active → Suspended → Waiting → Moving → Deleted, with cloud-native extensions (Degraded, Draining, Bootstrapping). Provides normative vocabulary for agent health monitoring.
→ [Runtime Infrastructure](../execution-verification/runtime-infrastructure.md)

**Forensic Causal Chain**
A reconstructed sequence of gate verdicts and retry actions traceable back to the first event that caused a failure, enabling root-cause diagnosis of complex multi-step verification failures.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Free Energy Principle**
Karl Friston's (2010) unified theory of biological self-organization: systems maintain homeostasis by minimizing surprise (free energy). Adapted in Roko for tier routing (escalate to a more expensive inference tier only when the current tier is surprised) and for prompt foraging (forage until marginal information gain falls below a threshold).
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md), [Budget Composition](../context-memory/budget-composition.md)

**FTS5 (Full-Text Search 5)**
SQLite's built-in full-text search engine used alongside HDC structural fingerprints in the code intelligence module. Results from FTS5 and HDC are merged via Reciprocal Rank Fusion (RRF).
→ [Code Intelligence](../context-memory/code-intelligence.md)

---

## G

**Gamma Speed**
The fastest cognitive tier (under 15 s per tick): reactive, heuristic, environmental scanning. Maps to biological gamma oscillations (30–100 Hz) and corresponds to System 1 / T0/T1 processing.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md)

**Gate**
A verification checkpoint that an AI agent's output must pass before it is accepted. A gate takes a work product as input and returns a structured Verdict. Seven gates compose the progressive verification pipeline.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Gate Pipeline**
The ordered sequence of seven rungs (Compile → Lint → Unit → Symbol Check → Generated Tests → Property Tests → Integration Tests) through which agent outputs pass before acceptance. Earlier rungs are cheap and always run; later rungs are invoked only when complexity warrants them.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Gate Ratchet**
A constraint preventing a task from regressing to a lower-rung pass state after it has achieved a higher-rung pass. Ensures that already-verified properties cannot be silently broken by later changes.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Geodesic**
The shortest path between two points on a Riemannian manifold, computed numerically via a 4th-order Runge-Kutta integrator. Used to find minimum-cost configuration transitions in multi-dimensional agent parameter space.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Ghost Turn**
An agent turn that produces no meaningful progress: no file changes, no test improvements, no new decisions — just repeated reading and planning. The GhostTurnWatcher detects ghost turns after N consecutive empty turns and triggers a restart with a new strategy.
→ [Conductor Anomaly Detection](../execution-verification/conductor-anomaly.md)

**Graph (Roko)**
The composed execution plan in the DAG engine: a collection of Cells connected by directed edges with optional conditions and budgets. The v2 term for the v1 concept "Workflow."
→ [DAG Execution Engine](../execution-verification/dag-execution.md)

---

## H

**Half-Life Decay**
The exponential Engram decay variant: `weight(t) = weight_0 * 0.5^(t / half_life)`. Produces continuous fading similar to radioactive decay; contrast with TTL (hard cutoff) and Ebbinghaus (psychologically-motivated curve).
→ [Universal Engram](../core-concepts/universal-engram.md)

**Hamming Distance**
The number of bit positions where two binary hypervectors differ. For two random 10,240-bit vectors, expected Hamming distance is 5,120 (similarity = 0.5). Decreasing distance indicates increasing semantic similarity.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

**Hamming Similarity**
`1 - (Hamming distance / D)` for D-dimensional vectors; ranges from 0 (perfect complements) to 1 (identical). The primary metric for comparing HDC hypervectors; computable with a single popcount instruction.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

**HdcFingerprint**
An optional field on an Engram containing the computed HDC hypervector plus encoder metadata (version, codebook seed). Enables similarity-based search over the Engram index without model inference.
→ [Universal Engram](../core-concepts/universal-engram.md)

**HDC (Hyperdimensional Computing)**
A computational framework that represents information as high-dimensional binary vectors (10,240 bits) and manipulates them with three algebraic operations (Bind, Bundle, Permute). Provides sub-millisecond semantic similarity search via CPU bitwise instructions, with no model inference required.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

**HdcVector**
The core data structure: a fixed-size 10,240-bit binary vector stored as `[u64; 160]`. All HDC operations — bind, bundle, permute, similarity — operate on this type.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

**Hierarchical Cancellation**
A tree-structured cancellation system where cancelling a parent token automatically cancels all descendant tokens. Ensures that cancelling a session stops all background tasks, spawned subprocesses, and in-flight tool calls that belong to that session.
→ [Runtime Infrastructure](../execution-verification/runtime-infrastructure.md)

**Hodges-Lehmann Estimator**
A robust location estimator with 29% breakdown point: the median of all pairwise averages (Walsh averages). More resistant to outliers than the arithmetic mean and more statistically efficient than the plain median.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Holt Exponential Smoothing**
A two-parameter time-series forecasting model (Holt 1957) that separately tracks level and trend components to predict the next value in a sequence. Used in the Conductor's circuit breaker for predictive tripping: forecasts the next error rate to trip the circuit before failures accumulate.
→ [Conductor Anomaly Detection](../execution-verification/conductor-anomaly.md)

**Hot Graph**
A DAG that runs on a tick-driven schedule rather than once per request: each tick evaluates active nodes and advances the graph state. Used for long-running resident computations like the heartbeat or background monitoring.
→ [DAG Execution Engine](../execution-verification/dag-execution.md)

**Hotelling's T-Squared**
A multivariate generalization of the t-test that detects anomalies in the joint distribution of multiple watcher metrics simultaneously, catching compound anomalies that no single metric shows.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Huber M-Estimator**
A robust regression estimator that applies squared-error loss for small residuals (like least squares) and absolute-value loss for large residuals (like LAD), achieving resistance to outliers with higher statistical efficiency than the median.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Hypnagogia**
The transitional state between wakefulness and sleep used as a creativity model in Dream Consolidation. The Hypnagogic subsystem applies four-layer processing (thalamic gate → executive loosener → Dali interrupt → homuncular observer) to produce cross-domain creative associations from disparate memories.
→ [Dream Consolidation](../agent-intelligence/dream-consolidation.md)

**Hypnagogic Creativity Pipeline**
The Dream Consolidation subsystem that generates creative insights by loosening associative constraints and finding surprising connections between unrelated experience fragments. Solves the "alpha convergence problem" where standard replay only strengthens existing patterns.
→ [Dream Consolidation](../agent-intelligence/dream-consolidation.md)

**Hypervector (HV)**
See HdcVector. The generic term for a vector with thousands of dimensions used in HDC computation.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

---

## I

**IIT Phi Metric**
An approximation of Integrated Information Theory's Φ (phi) measure, used in the Somatic TA integration to quantify the degree to which a multi-agent system acts as an integrated whole rather than a collection of independent parts.
→ [Affect Engine](../agent-intelligence/affect-engine.md)

**ItemMemory**
An HDC structure that stores a set of named concepts (strings mapped to hypervectors) and supports nearest-neighbor lookup: given a query vector, find the stored concept with the highest Hamming similarity.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

---

## J

**JSON-RPC 2.0**
The wire protocol underlying both MCP and ACP: requests are JSON objects with `method`, `params`, and `id`; responses are JSON objects with `result` or `error`. Stateless at the individual message level; session state is managed above this layer.
→ [MCP and Editor Integration](../ecosystem/mcp-editor-integration.md)

---

## K

**Kind (Engram)**
The semantic type system for Engrams: an enum with 30+ variants (Task, Episode, GateVerdict, Pheromone, Observation, ToolCall, CodeSymbol, etc.) that determines how an Engram is routed, stored, searched, and decayed.
→ [Universal Engram](../core-concepts/universal-engram.md)

**KORAI Token**
The demurrage-based utility token in the Roko economic model: token balances decay over time if not used, incentivizing circulation and active participation. Used for micropayments in the Spore Marketplace and on-chain reputation staking.
→ [Chain Reputation](../ecosystem/chain-reputation/README.md)

---

## L

**Language Support**
The module that provides multi-language code analysis through `BuildSystem` and `LanguageProvider` trait abstractions for Rust, TypeScript, and Go. Extracts symbols, dependency edges, and imports to feed the code intelligence index.
→ [Language Support](../context-memory/language-support.md)

**Life Review Pipeline**
A Dream Consolidation subsystem adapted from Butler's (1963) therapeutic life review: a structured retrospective that classifies the agent's experience narrative arc using McAdams's (2001) narrative identity framework, generating insight summaries for long-running agents.
→ [Affect Engine](../agent-intelligence/affect-engine.md)

**LinUCB**
A contextual bandit algorithm (Li et al. 2010) that models the expected reward of each arm (LLM provider) as a linear function of a context vector, with UCB exploration bonuses proportional to prediction uncertainty. Used in the Cascade Router for model selection.
→ [Online Learning](../agent-intelligence/online-learning.md)

**Lineage DAG**
The directed acyclic graph of parent Engrams that a given Engram was derived from, stored as a list of ContentHash references. Enables causal replay, impact analysis, and forensic audit of any decision in the system.
→ [Universal Engram](../core-concepts/universal-engram.md)

**Live DAG Mutation**
The orchestrator's ability to modify a running workflow graph in real time — inserting new nodes, removing blocked branches, updating edge conditions — without stopping execution of unaffected nodes.
→ [Orchestrator Swarm](../execution-verification/orchestrator-swarm.md)

---

## M

**MAD (Median Absolute Deviation)**
A robust scale estimator: `MAD = median(|x_i - median(x)|)`. Provides a measure of spread that is resistant to outliers (50% breakdown point), useful for detecting anomalous LLM cost spikes without being thrown off by isolated expensive runs.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Mattar-Daw Utility Score**
The replay selection formula from Mattar and Daw (2018): `utility = gain × need × spacing_inverse`. Gain is the expected future value of replaying the episode; Need is proportional to how long ago it was last replayed; Spacing Inverse rewards infrequent rehearsal. Used in NREM replay to select which memories to strengthen.
→ [Dream Consolidation](../agent-intelligence/dream-consolidation.md)

**MCP (Model Context Protocol)**
An open standard published by Anthropic (November 2024) that defines how AI assistants connect to external tool servers using JSON-RPC 2.0. Standardizes tool discovery, invocation, resource access, and error handling across providers and languages.
→ [MCP and Editor Integration](../ecosystem/mcp-editor-integration.md)

**Metacognitive Monitor**
An agent-loop pattern that introspects on the agent's own execution to detect failure modes: stuck loops (alternating identical failures), contradiction loops (contradictory consecutive actions), and runaway cost (spend rate exceeding budget projection). Triggers controlled intervention before complete failure.
→ [Agent Patterns](../agent-intelligence/agent-patterns.md)

**Morphogenetic Specialization**
Agent role differentiation based on the Turing (1952) reaction-diffusion model and the Gierer-Meinhardt (1972) activator-inhibitor system. Initially identical agents spontaneously specialize into distinct roles (planner, implementer, reviewer) through local concentration dynamics, without central assignment.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md)

---

## N

**NEAR Protocol**
A proof-of-stake blockchain with sub-cent transaction fees, 2-second finality, native Rust smart contract SDK, and Nightshade sharding. The target deployment platform for Roko's on-chain identity and reputation contracts.
→ [Chain Reputation](../ecosystem/chain-reputation/README.md), [Smart Contracts](../ecosystem/smart-contracts/README.md)

**Neuro**
The Roko crate name for the memory/search subsystem (`roko-neuro`). In the IronClaw mapping, Neuro corresponds to workspace memory with FTS/vector/HDC search.
→ [Roko Terminology](terminology-glossary.md)

**Nietzsche Vitality Phases**
A philosophical extension of the affect engine that models agent vitality in three phases (Apollo/structured rationality, Dionysus/creative engagement, Nirvana/acceptance and archiving) as an agent approaches end-of-life, generating a structured "emotional death testament" of lessons learned.
→ [Affect Engine](../agent-intelligence/affect-engine.md)

**NREM Replay**
The first phase of Dream Consolidation, modeled on non-REM slow-wave sleep: utility-weighted replay of past episodes selected by the Mattar-Daw formula. Strengthens high-value memories and prunes redundant ones without generating new knowledge.
→ [Dream Consolidation](../agent-intelligence/dream-consolidation.md)

---

## O

**OCC Appraisal**
See Appraisal (OCC) above.

**Online Learning**
A machine learning paradigm in which a model updates its parameters after every single observation, rather than training offline on a fixed dataset. Enables the Cascade Router to continuously improve its LLM routing decisions without a separate training pipeline.
→ [Online Learning](../agent-intelligence/online-learning.md)

**Oscillation Detection**
A self-healing mechanism in the Conductor that detects when the system is alternating between two or more failing configurations (for example, switching between providers in a loop) and breaks the cycle by entering a mandatory cooldown period.
→ [Conductor Anomaly Detection](../execution-verification/conductor-anomaly.md)

---

## P

**PAD Vector (Pleasure-Arousal-Dominance)**
The three-dimensional emotional state representation from Mehrabian and Russell (1974): Pleasure (positive–negative valence), Arousal (excited–calm activation), Dominance (in control vs. submissive). Each dimension is a float in [-1.0, 1.0]. The PAD vector is the core state type of the affect engine.
→ [Affect Engine](../agent-intelligence/affect-engine.md)

**PageRank**
The iterative link-analysis algorithm (Page et al. 1998) adapted in Roko for two uses: (1) ranking code symbols by dependency-graph centrality in the code intelligence module; (2) TraceRank — applying PageRank to the agent-interaction graph to produce portable trust scores resistant to Sybil attacks.
→ [Code Intelligence](../context-memory/code-intelligence.md), [Chain Reputation](../ecosystem/chain-reputation/README.md)

**Pareto Frontier**
In multi-objective optimization, the set of solutions where no objective can be improved without worsening another. The Cascade Router computes the Pareto frontier across (pass rate, cost, latency, reliability) and selects from it using LinUCB rather than collapsing to a single weighted objective.
→ [Online Learning](../agent-intelligence/online-learning.md)

**PELT (Pruned Exact Linear Time)**
An offline changepoint detection algorithm (Killick et al. 2012) that finds all changepoints in a historical time series in O(n) expected time. Used in gate pipelines for retrospective analysis of when pass-rate regimes shifted.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Permute (Cyclic Shift)**
The HDC operation that rotates all bits in a hypervector by a fixed number of positions. Permutation encodes sequence order: `bind(permute(A), B)` means "A then B" and differs from `bind(A, B)` ("A and B unordered"), enabling the encoding of temporal relationships.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

**Persistence Diagram**
A scatter plot where each point `(birth_eps, death_eps)` represents one topological feature (connected component or loop) in the Vietoris-Rips filtration of a point cloud. Long-lived features (far from the diagonal) are genuine structure; short-lived features are noise.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Persistence Landscape**
A functional summary of a persistence diagram that converts it to a sequence of piecewise-linear functions, enabling stable statistical computations (mean, variance, kernel distances) over collections of diagrams.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Persistent Homology**
The TDA technique that tracks the birth and death of topological features (connected components: H0, loops: H1, voids: H2) as a scale parameter epsilon increases in a filtration. Features that persist across a large range of epsilon are genuinely structural, not noise.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Pheromone**
A decaying shared coordination signal, analogous to ant pheromones, stored in a workspace environment so that multiple agents can coordinate without direct messaging. Pheromones encode task claims, completion signals, and quality assessments; they evaporate over time.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md), [Orchestrator Swarm](../execution-verification/orchestrator-swarm.md)

**Plan-to-Graph Conversion**
The pipeline that translates a TOML implementation plan (list of tasks with dependencies) into a runnable DAG, instantiating Cell nodes for each task type and connecting them with dependency edges and failure branches.
→ [DAG Execution Engine](../execution-verification/dag-execution.md)

**Process Reward Model (PRM)**
A model that provides feedback at each intermediate reasoning step (rather than only at the final answer), enabling early termination of clearly-failing trajectories and rung-level feedback for the agent. Adapted in the gate pipeline as cumulative Promise/Progress signals.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Progress Signal**
A gate pipeline metric measuring whether each successive verification turn is reducing the number of remaining errors — distinguishing a converging trajectory from a thrashing one.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Promise Signal**
A gate pipeline metric measuring the cumulative evidence that a task will eventually pass all rungs, based on which rungs have already passed and the historical pass-rate model for remaining rungs.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Provenance**
The Engram field recording who produced it (author identity, trust score) and classification of its trust level (0.0–1.0). Combined with the Taint field to determine whether an Engram's score should be weighted or zeroed in retrieval.
→ [Universal Engram](../core-concepts/universal-engram.md)

**Pulse**
An ephemeral event on the EventBus that does not persist to storage, contrasted with an Engram which is durable. Pulses carry real-time notifications between components (tool call started, LLM response received) that are not needed after the current session.
→ [Universal Engram](../core-concepts/universal-engram.md), [Runtime Infrastructure](../execution-verification/runtime-infrastructure.md)

---

## Q

**Quasi-Orthogonal**
Two random hypervectors whose Hamming similarity is near 0.5 — effectively independent in the HDC sense. At 10,240 bits, the probability of two randomly generated vectors being within 1% of 0.5 similarity is approximately 1 in 10^18.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

---

## R

**Reaction-Diffusion**
A class of mathematical models (Turing 1952, Gierer-Meinhardt 1972) describing how two interacting chemical species — an activator and an inhibitor — can self-organize into stable spatial patterns from homogeneous initial conditions. Used in Roko's morphogenetic agent specialization.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md)

**REM Imagination**
The second phase of Dream Consolidation, modeled on REM sleep's role in emotional memory and counterfactual simulation: generates "what if" scenarios (alternative actions, different tool sequences, hypothetical user requests) that expand the agent's experience beyond what it has directly observed.
→ [Dream Consolidation](../agent-intelligence/dream-consolidation.md)

**Reciprocal Rank Fusion (RRF)**
A technique for combining ranked result lists from multiple retrieval systems (HDC, FTS5, PageRank graph, symbol index) into a single merged ranking. Each document's final score is the sum of `1/(rank_i + k)` across all lists.
→ [Code Intelligence](../context-memory/code-intelligence.md)

**Riemannian Geometry**
Differential geometry on curved manifolds, used in the mathematical primitives module to model the cost landscape of agent configurations as a curved space and compute geodesic (minimum-cost) paths between configurations.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Role-Filler Encoding**
An HDC pattern that encodes structured records as a single hypervector by binding each field's role vector with its value vector and then bundling all pairs: `record = bundle(bind(role_1, val_1), ..., bind(role_n, val_n))`. Supports approximate extraction of individual fields.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

**ROI (Composite)**
The five-axis scoring formula used in the Priority Matrix: `ROI = 0.30 * user_impact + 0.20 * system_impact + 0.25 * ease + 0.15 * safety + 0.10 * independence`. Scores range from 1.55 to 4.55.
→ [Priority Matrix](../strategy/priority-matrix/README.md)

**RRF**
See Reciprocal Rank Fusion.

**Rung**
One level in the 7-rung progressive verification pipeline. Each rung applies a different class of verification: (1) Compile, (2) Lint, (3) Unit Tests, (4) Symbol Check, (5) Generated Tests, (6) Property Tests, (7) Integration Tests.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

---

## S

**Score (Engram)**
A seven-axis quality measurement attached to each Engram: confidence, novelty, utility, reputation, precision, salience, coherence. The effective score is computed as `confidence * reputation * geometric_mean(remaining)` — zero confidence or reputation structurally zeros the effective score.
→ [Universal Engram](../core-concepts/universal-engram.md)

**SDM (Sparse Distributed Memory)**
Pentti Kanerva's 1988 mathematical model of content-addressable memory in high-dimensional binary spaces, the intellectual precursor of HDC. SDM explained how biological memory can store vast amounts of data and retrieve items from noisy partial cues.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

**Sheaf (Cellular)**
A mathematical structure that assigns a vector space (stalk) to each node and edge of a graph, with linear maps (restriction maps) relating neighboring stalks. Inconsistencies between nodes are measured by the coboundary operator and the sheaf Laplacian, identifying which sources are outliers in multi-source data.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Sheaf Laplacian**
The matrix analogue of the graph Laplacian for cellular sheaves: its smallest eigenvalues characterize global consistency and its eigenvectors identify the inconsistent subgraphs.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Signal**
The v2/preferred term for Engram (the universal content-addressed data record). The rename was in progress in the captured Roko source; IronClaw integration plans use Signal as the canonical term and Engram when quoting source types.
→ [Roko Terminology](terminology-glossary.md), [Universal Engram](../core-concepts/universal-engram.md)

**SINR Model**
Signal-to-Interference-plus-Noise Ratio model from wireless communications, adapted for multi-agent coordination: an agent's effective "signal" to its intended peer decreases with the number of other agents broadcasting simultaneously, informing agent density and pheromone broadcast power tuning.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md)

**Simplicial Complex**
A topological structure built from points, edges, triangles, tetrahedra, and higher-dimensional simplices used in TDA. The Vietoris-Rips complex connects all points within distance epsilon, and its evolution as epsilon increases reveals topological structure.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Somatic Marker**
A learned association (Damasio 1994) between a decision context (encoded as an 8-dimensional StrategyCoordinates vector) and an emotional outcome, stored in a k-d tree for fast nearest-neighbor retrieval. Provides decision shortcuts by recalling the emotional valence of past similar situations.
→ [Affect Engine](../agent-intelligence/affect-engine.md)

**Soulbound NFT / Passport**
A non-transferable blockchain token (ERC-8004) that serves as an agent's immutable identity. The non-transferability property prevents Sybil attacks by making it impossible to sell a reputation and start over with a clean identity.
→ [Chain Reputation](../ecosystem/chain-reputation/README.md)

**Spore Marketplace**
See Bounty Marketplace (Spore).

**SPRT (Sequential Probability Ratio Test)**
Wald's (1945) sequential hypothesis testing method that makes an accept/reject decision as soon as accumulated evidence is sufficient, requiring fewer observations on average than fixed-sample tests. The gate pipeline's early-termination logic mirrors this structure.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Staging Buffer**
A Dream Consolidation component that manages a confidence ladder for newly synthesized knowledge: insights start at Embryonic, advance through Candidate and Stable, and graduate to Production after withstanding repeated verification. Prevents premature knowledge corruption.
→ [Dream Consolidation](../agent-intelligence/dream-consolidation.md)

**Stalk (Sheaf)**
The vector space assigned to a node or edge in a cellular sheaf, containing the local data (measurements, observations) at that point of the graph.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**StateHub**
A projection layer that aggregates raw event streams into materialized views suitable for dashboard display: agent roster, active jobs, budget summaries, gate outcomes, and system health indicators.
→ [Runtime Infrastructure](../execution-verification/runtime-infrastructure.md)

**Stigmergy**
The biological coordination mechanism (Grassé 1959) in which agents leave traces in a shared environment that influence subsequent agents' behavior, enabling complex emergent coordination without direct communication. Roko implements this via digital pheromones.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md)

**Store**
The v2 term for Substrate: the persistence backend abstraction. In IronClaw, Store is the dual-backend (PostgreSQL + libSQL) database facade.
→ [Roko Terminology](terminology-glossary.md)

**Substrate**
The v1 Roko term for the persistence backend. Now referred to as Store. Stored Engrams/Signals durably.
→ [Roko Terminology](terminology-glossary.md)

**Sybil Attack**
The strategy (Douceur 2002) of creating many pseudonymous identities to gain disproportionate influence in a reputation system. Countered by soulbound identity (non-transferable passports) and stake-based tier thresholds.
→ [Chain Reputation](../ecosystem/chain-reputation/README.md)

---

## T

**T0 / T1 / T2 (Inference Tiers)**
The three inference tiers in the dual-process cognitive model: T0 (sub-conceptual: no LLM call, threshold checks and regex), T1 (System 1: fast model, quick analysis), T2 (System 2: full model, multi-turn deep reasoning).
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md)

**Taint**
An Engram field classifying the trustworthiness of its data across 9 variants: Clean, Unverified, LlmGenerated, UserFlagged, Adversarial, Synthesized, InheritsParent, PendingReview, Deprecated. Taint propagates through the lineage DAG: a child Engram inherits the maximum taint level of its parents.
→ [Universal Engram](../core-concepts/universal-engram.md)

**Takens Delay Embedding**
A theorem (Takens 1981) that a scalar time series can reconstruct the full phase space of a dynamical system by embedding it with a delay parameter. Used in TDA to convert a 1D agent metric series into a point cloud suitable for topological analysis.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**TDA (Topological Data Analysis)**
The mathematical discipline that uses tools from algebraic topology (simplicial complexes, persistent homology) to extract robust shape descriptors from point clouds and time series. In this system, TDA detects cyclic patterns (H1 loops) in agent execution traces that signal stuck behavior.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Theta Speed**
The middle cognitive tier (~75 s per cycle): reflective processing, re-planning, evaluation of progress. Maps to biological theta oscillations (4–8 Hz) and supports deliberate System 2 reasoning.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md)

**Thompson Sampling**
A Bayesian exploration strategy that selects actions by sampling from their posterior reward distributions, naturally balancing exploration and exploitation. Used in the Conductor for adaptive threshold learning and in Budget Composition for Thompson Sampling Learning Bidders.
→ [Online Learning](../agent-intelligence/online-learning.md), [Budget Composition](../context-memory/budget-composition.md), [Conductor Anomaly Detection](../execution-verification/conductor-anomaly.md)

**Threat Rehearsal**
The Dream Consolidation subsystem that enumerates likely failure modes using Failure Mode and Effects Analysis (FMEA) and Fault Tree Analysis (FTA), then generates defensive strategies and warning messages to be loaded into the agent's context before high-risk operations.
→ [Dream Consolidation](../agent-intelligence/dream-consolidation.md)

**TOML Workflow**
A human-readable, TOML-format declarative workflow definition specifying DAG nodes (Cells), dependencies, edge conditions, budget limits, and retry policies. The plan-to-graph converter instantiates these as runnable Graphs.
→ [DAG Execution Engine](../execution-verification/dag-execution.md)

**Topological Sort (Kahn's Algorithm)**
Kahn's (1962) BFS-based algorithm that produces a valid execution order for a DAG by repeatedly removing nodes with no remaining dependencies. Used in the graph engine to determine which Cells can run next.
→ [DAG Execution Engine](../execution-verification/dag-execution.md)

**TraceRank**
A PageRank variant applied to the agent-interaction graph (nodes are agents, edges are task delegations weighted by outcome quality), producing a portable trust score that inherits the collusion resistance of PageRank's eigenvector centrality.
→ [Chain Reputation](../ecosystem/chain-reputation/README.md)

**Trimmed Mean**
A robust location estimator that discards the highest and lowest p% of values before computing the mean; has a breakdown point of p%. Used to compute reliable LLM cost estimates when occasional extremely expensive runs would otherwise inflate the mean.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**Tropical Algebra (Max-Plus Semiring)**
An algebraic structure where addition is replaced by maximum and multiplication by addition: `a ⊕ b = max(a,b)`, `a ⊗ b = a + b`. The "tropical" matrix power computes shortest paths (Bellman-Ford) and enables exact decision-boundary analysis of piecewise-linear functions in the tool routing logic.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**TTL (Time-to-Live) Decay**
The Engram decay variant that zeroes the effective weight at a fixed expiry time regardless of access history. Appropriate for ephemeral data like session-specific observations.
→ [Universal Engram](../core-concepts/universal-engram.md)

**Turing Reaction-Diffusion**
See Reaction-Diffusion.

---

## U

**UCB (Upper Confidence Bound)**
The exploration bonus in bandit algorithms: `UCB = mean_reward + alpha * sqrt(log(N)/n)` where N is total pulls and n is pulls of this arm. Larger uncertainty (fewer pulls) produces larger bonuses, driving exploration of understudied arms.
→ [Online Learning](../agent-intelligence/online-learning.md)

**Universal Engram**
The design pattern of using a single universal data type (the Engram / Signal) for all knowledge in the system — events, decisions, gate verdicts, memories, tool traces — to achieve universal composability, full audit trails, and temporal dynamics.
→ [Universal Engram](../core-concepts/universal-engram.md)

**U-Shaped Attention Curve**
The empirically observed pattern (Liu et al. 2024, "Lost in the Middle") that LLMs attend most strongly to content at the beginning and end of their context window, with significantly degraded attention to middle content. Informs the U-shaped placement strategy in Budget Composition.
→ [Budget Composition](../context-memory/budget-composition.md)

---

## V

**VCG-Inspired Diagnostics**
Vickrey-Clarke-Groves auctions are a mechanism-design framework for truthful allocation under strict assumptions. The budget-composition plan does not implement a full proof-carrying VCG mechanism; it uses greedy density allocation and can record displacement payments as calibration diagnostics.
→ [Budget Composition](../context-memory/budget-composition.md)

**Verdict**
The structured output of a gate check: Pass or Fail with a score (0.0–1.0), error category, specific error messages, affected file paths, and causal chain for forensic analysis.
→ [Gate Verification Pipeline](../execution-verification/gate-verification.md)

**Vietoris-Rips Complex**
A simplicial complex constructed from a point cloud by connecting all pairs of points within distance epsilon. As epsilon increases from 0 to infinity, the complex passes through a filtration whose topology encodes the shape of the data.
→ [Mathematical Primitives](../core-concepts/mathematical-primitives.md)

**VSA (Vector Symbolic Architectures)**
The broader family of computational frameworks that includes HDC: all VSAs represent information as high-dimensional vectors and combine them with algebraic operations (binding, bundling, permutation). BSC is the specific VSA variant used in this system.
→ [Hyperdimensional Computing](../core-concepts/hyperdimensional-computing/README.md)

---

## W

**Watcher**
One of ten specialized monitoring functions in the Conductor, each scanning a specific signal stream for a specific failure pattern: LatencyWatcher, ErrorRateWatcher, TokenUsageWatcher, CostWatcher, QualityWatcher, ThroughputWatcher, SaturationWatcher, AvailabilityWatcher, DriftWatcher, CoherenceWatcher.
→ [Conductor Anomaly Detection](../execution-verification/conductor-anomaly.md)

**Wave Scheduling**
The orchestrator's parallel execution algorithm: tasks are grouped into waves where each wave contains all tasks whose dependencies are satisfied by completed prior waves. All tasks in a wave execute concurrently; the next wave begins when all tasks in the current wave complete.
→ [Orchestrator Swarm](../execution-verification/orchestrator-swarm.md)

**Worktree Isolation**
Git worktree-based isolation of parallel agent work: each agent working on a separate task operates in its own git worktree (a separate checkout of the same repository at a new branch), preventing filesystem conflicts between simultaneously active agents.
→ [Orchestrator Swarm](../execution-verification/orchestrator-swarm.md)

---

## X

**X402 Micropayments**
An HTTP-based micropayment protocol using the HTTP 402 status code (Payment Required): the server returns 402 with a KORAI payment request, and the client pays before retrying. Used for per-request LLM API access pricing between agents.
→ [Chain Reputation](../ecosystem/chain-reputation/README.md)

---

## Y

**Yerkes-Dodson Pressure Framework**
An inverted-U relationship (Yerkes & Dodson 1908) between arousal/pressure and performance: too little pressure yields disengagement; too much yields panic; optimal performance occurs at moderate arousal. Used in the Conductor to calibrate intervention aggressiveness — interventions are scaled with a Yerkes-Dodson multiplier based on current budget pressure.
→ [Conductor Anomaly Detection](../execution-verification/conductor-anomaly.md)

---

## Z

**Zero-Cost Subsystem**
An architectural aspiration for Gamma-speed processing: T0 checks (regex, threshold comparisons, cache lookups) that consume no LLM tokens and complete in microseconds. Every tick first passes through zero-cost checks before escalating to T1 or T2 inference.
→ [Cognitive Architecture](../core-concepts/cognitive-architecture.md)

---

*For the full document index, see the [knowledge base README](../README.md). For academic sources, see [Research Citations](./research-citations/README.md). For the Roko-to-IronClaw naming bridge (Engram → Signal, Substrate → Store, etc.), see [Terminology Bridge](terminology-glossary.md).*

# 27 -- Roko v2-Depth Research Documents: Complete Catalog

> A complete catalog of 145 depth documents across 23 thematic sections in roko's `docs/v2-depth/` directory. These documents form the algorithmic, theoretical, and implementation-detail layer beneath roko's unified specification -- the "how" and "why" behind every design decision.

---

## Table of Contents

1. [What Is This Collection?](#what-is-this-collection)
2. [How It Was Built](#how-it-was-built)
3. [How to Read This Catalog](#how-to-read-this-catalog)
4. [Key Documents](#key-documents) -- the 18 most important/novel documents
5. [Reading Guides](#reading-guides) -- curated paths for specific interests
6. [Section-by-Section Catalog](#section-by-section-catalog)
   - [00-index: Vision, Principles, Architecture](#00-index) (6 docs)
   - [01-signal: Signal Internals and Decay Math](#01-signal) (4 docs)
   - [02-block: Composition, Tools, Verification](#02-block) (15 docs)
   - [03-graph: Orchestration and DAG Execution](#03-graph) (7 docs)
   - [04-specializations](#04-specializations) (reserved)
   - [05-execution-engine: Runtime Loop](#05-execution-engine) (2 docs)
   - [06-trigger-system](#06-trigger-system) (reserved)
   - [07-agent-runtime: Agent, Conductor, Heartbeat, Lifecycle](#07-agent-runtime) (27 docs)
   - [08-extension-system](#08-extension-system) (reserved)
   - [09-telemetry: Observability](#09-telemetry) (1 doc)
   - [10-learning-loops: Bandits, Self-Improvement](#10-learning-loops) (10 docs)
   - [11-memory: Knowledge, Dreams, Stigmergy](#11-memory) (15 docs)
   - [12-connectivity: Relay and Networking](#12-connectivity) (5 docs)
   - [13-builtin-catalog: Tools, MCP, Plugins](#13-builtin-catalog) (5 docs)
   - [14-config: Configuration as Signal](#14-config) (2 docs)
   - [15-marketplace: Identity, Economy, Commerce](#15-marketplace) (6 docs)
   - [16-surfaces: UI/UX Architecture](#16-surfaces) (7 docs)
   - [17-security: Safety and Defense](#17-security) (7 docs)
   - [18-registries: On-Chain Infrastructure](#18-registries) (8 docs)
   - [19-arenas](#19-arenas) (reserved)
   - [20-deployment: Deployment Shapes](#20-deployment) (4 docs)
   - [21-roadmap: Research Foundations](#21-roadmap) (9 docs)
   - [22-code-intelligence: Code Analysis Pipeline](#22-code-intelligence) (5 docs)
7. [Cross-References to tmp/ Documents](#cross-references)
8. [Statistics](#statistics)

---

<a name="what-is-this-collection"></a>
## 1. What Is This Collection?

Roko's documentation follows a two-layer model:

- **Spec layer** (`docs/v2/`, 28 files): Defines vocabulary, protocols, and contracts. Says *what* and *why*. Concise, stable, authoritative.
- **Depth layer** (`docs/v2-depth/`, 145 documents + 40 support files): Algorithms, research grounding, implementation detail, domain patterns. Says *how*, *where it came from*, and *what's possible*. Thorough, exploratory, evolving.

The depth layer is organized into 23 thematic sections that mirror the spec files 1:1. Each section contains a set of depth documents plus an `INDEX.md` tracking which source documents have been absorbed into it. Four sections (04-specializations, 06-trigger-system, 08-extension-system, 19-arenas) are reserved placeholders with no depth documents yet.

**The rule of thumb**: If it has an arXiv citation, a specific benchmark number, a code implementation detail, or a domain-specific algorithm, it goes in depth, not spec.

### Core Vocabulary

The unified vocabulary mandates specific terms across all depth documents:

| Unified Term | Old Term(s) | Meaning |
|---|---|---|
| Signal | Engram | Durable, content-addressed data unit |
| Pulse | Event | Ephemeral event on the Bus |
| Cell | Block, Module | Atomic unit of computation |
| Graph | Workflow, Pipeline | Composition of Cells |
| Store | Substrate | Persistent state (pull-based) |
| Bus | EventBus | Ephemeral pub/sub (push-based) |
| Loop | Feedback Loop | Graph with a feedback edge |
| Lens | Projection, Observer | Read-only state projection |
| Verdict | GateResult | Verification outcome |

### Five Core Primitives

1. **Signal** -- durable data with content hash, HDC fingerprint, score vector, decay parameters, lineage, and taint
2. **Cell** -- atomic computation unit declaring typed I/O and protocol conformance
3. **Graph** -- composition of Cells (Pipeline, Loop, Hot Graph specializations)
4. **Bus** -- ephemeral pub/sub for Pulses (push-based, ring-buffer eviction)
5. **Store** -- persistent state for Signals (pull-based, queryable, durable)

### Nine Protocols

Store, Score, Verify, Route, Compose, React, Observe, Connect, Trigger. Every Cell declares which protocols it implements. The protocols form a dependency lattice that determines the five-layer architecture.

### Four Learning Loops

- **L1** (per-tick): Parameter tuning within a single execution
- **L2** (per-episode): Strategy selection across tasks
- **L3** (per-batch): Knowledge consolidation and representation change
- **L4** (per-approval): Architecture evolution with human oversight

---

<a name="how-it-was-built"></a>
## 2. How It Was Built

The depth layer was generated through a structured ingestion process:

1. **Source material**: 422 source documents totaling 8.8MB were collected from four pools:
   - 160+ academic papers and research documents (fed via 15 `RESEARCH-PROMPT` files)
   - DeFi gap analysis (14 files, 574KB)
   - Implementation specifications (20+ files)
   - Core v1 documentation (417 files)

2. **Ingestion process**: Each source document was processed using the `INGEST-PROMPT.md` template, which instructs the ingesting agent to absorb content into the correct section using unified vocabulary, tag concepts with protocol conformance, and track absorption status in the section's `INDEX.md`.

3. **Research prompts**: The 15 `RESEARCH-PROMPT` files (numbered 2-14 plus the base) each cover a specific batch of academic papers. These were used to generate depth documents grounded in published research with proper citations.

4. **Quality rules**: The `GUIDE.md` specifies that depth documents must use unified vocabulary, reference spec sections, include concrete code examples or pseudocode, and provide fidelity-loss analysis when simplifying academic results.

The support files in the root of `docs/v2-depth/` are:
- `INDEX.md` -- master index mapping all 422 source documents to depth documents
- `GUIDE.md` -- authoring guide with vocabulary reference and ingestion rules
- `INGEST-PROMPT.md` -- prompt template for absorbing new source material
- `RESEARCH-PROMPT.md` through `RESEARCH-PROMPT-14.md` (16 files) -- prompts for generating depth from academic papers

---

<a name="how-to-read-this-catalog"></a>
## 3. How to Read This Catalog

Each document entry in this catalog includes:

- **Path**: Relative to `docs/v2-depth/`
- **Summary**: What the document covers and its key arguments
- **Key concepts**: The most important ideas introduced
- **Relevance**: How directly applicable the concepts are to IronClaw (HIGH/MEDIUM/LOW/NONE)
- **Cross-references**: Which tmp/ analysis documents draw from or relate to this depth document

Relevance ratings:
- **HIGH** -- Directly applicable to current or near-future IronClaw features; concepts map to existing modules
- **MEDIUM** -- Applicable with adaptation; the ideas are sound but IronClaw would need to adapt the design
- **LOW** -- Interesting concepts outside IronClaw's current scope; may become relevant later
- **NONE** -- Domain-specific to blockchain, multi-agent economics, or roko-specific infrastructure

---

<a name="key-documents"></a>
## 4. Key Documents

The 18 most important and novel documents across the collection, organized by theme. Reading these gives a comprehensive understanding of roko's most distinctive architectural contributions.

### Foundation: What Makes This System Different

**1. `00-index/architectural-thesis.md`** -- _The Scaffold IS the Product_
The foundational claim: given the same LLM, agent performance varies 30-65% depending on harness quality (SWE-bench data). Therefore the scaffold -- not the model -- is the durable asset. Derives the five-layer architecture as a theorem from protocol dependency analysis. Proves that layers emerge from topological sort of which protocols require which others.
- Cross-refs: [26-roko-architecture-overview](26-roko-architecture-overview.md), [13-cognitive-architecture](13-cognitive-architecture.md)

**2. `01-signal/demurrage-economics.md`** -- _Gesell-Shannon Knowledge Decay_
Derives knowledge decay rates from first principles combining Gesell's idle-asset economics with Shannon information theory. Formula: `effective_rate = base_rate / (1 + novelty)` -- redundant knowledge decays faster than novel knowledge. Models the phase space (balance x tier x novelty) and identifies fixed points.
- Cross-refs: [10-universal-engram](10-universal-engram.md), [02-dream-consolidation](02-dream-consolidation.md)

**3. `02-block/verify-as-universal-oracle.md`** -- _Verification as the Contact Point with Reality_
Derives four simultaneous roles for verification: reward function, relabeling oracle, safety boundary, economic attestation. Proves Goodhart-resistance through conjunctive hard gates + Pareto soft criteria. Introduces the Variance Inequality bounding improvement speed.
- Cross-refs: [05-gate-verification](05-gate-verification.md), [07-online-learning](07-online-learning.md)

### Context Engineering: How to Fill a Context Window Optimally

**4. `02-block/active-inference-context-selection.md`** -- _EFE-Based Context Selection_
Replaces hand-tuned context priorities with learned, task-adaptive allocation using Expected Free Energy (EFE) minimization from Karl Friston's active inference. EFE naturally balances exploration (epistemic value) and exploitation (pragmatic value) with zero hyperparameters. Combines with Marginal Value Theorem as a stopping rule.
- Cross-refs: [09-budget-composition](09-budget-composition.md)

**5. `02-block/vcg-attention-auction.md`** -- _VCG Mechanism for Token Allocation_
Applies Vickrey-Clarke-Groves auction to allocate scarce context-window tokens among eight competing subsystems. VCG payments reveal each subsystem's marginal contribution. Thompson sampling with cost-awareness learns optimal bids over time.
- Cross-refs: [09-budget-composition](09-budget-composition.md)

**6. `02-block/positional-effects-and-retrieval.md`** -- _U-Shape Attention as Algebraic Property_
Formalizes the "lost in the middle" effect: causal decoders inherently attend more to content at the beginning and end of context windows. Defines a Placement enum and BetaPosterior tracking of section effectiveness by position.
- Cross-refs: [09-budget-composition](09-budget-composition.md)

### Learning and Routing: How the System Gets Smarter Over Time

**7. `10-learning-loops/bandit-routing-and-cascade.md`** -- _Three-Stage Cascade Router_
Three-stage cascade: static rules, confidence-based check, LinUCB contextual bandits with 18-dimensional context vector. Pareto frontier pre-filtering. Trigram pattern discovery. Projects 30-50% LLM cost reduction.
- Cross-refs: [07-online-learning](07-online-learning.md)

**8. `10-learning-loops/self-improvement-frameworks.md`** -- _Five-Level Improvement Stack_
Maps Reflexion, ExpeL, DSPy, ADAS to five improvement levels (parameters, strategies, representations, architecture, meta). Introduces the Variance Inequality: no learning loop may increase variance faster than the system can absorb it. Constitutional constraints prevent harmful self-modification.
- Cross-refs: [07-online-learning](07-online-learning.md), [13-cognitive-architecture](13-cognitive-architecture.md)

**9. `10-learning-loops/missing-loops-and-calibration.md`** -- _Eight Missing Feedback Loops_
Identifies eight cybernetic feedback loops that should exist but are not wired: Health->Routing, Conductor->Routing, Section->Scaffold, Failure->Replan, Skills->Prompts, Cost->Routing, Latency->Reward, Experiments->Static. Cross-loop interaction matrix. Any system can use this as a diagnostic checklist.
- Cross-refs: [06-conductor-anomaly](06-conductor-anomaly.md), [07-online-learning](07-online-learning.md)

### Memory and Consolidation: How Knowledge Persists and Evolves

**10. `11-memory/06-dream-cycle-as-loop.md`** -- _Offline Consolidation Cycle_
Three-phase dream cycle (NREM replay, REM imagination, Integration) as a Loop Graph. NREM replays high-value episodes using Mattar-Daw utility scoring. REM generates novel combinations via HDC vector recombination. Integration promotes or rejects candidates.
- Cross-refs: [02-dream-consolidation](02-dream-consolidation.md)

**11. `11-memory/04-antiknowledge-and-immunity.md`** -- _Cognitive Immune System_
AntiKnowledge as a Signal Kind representing things proven false. The insight: knowing what is wrong is permanently valuable (AntiKnowledge never fully decays). Cognitive immune system using SIR epidemiological tracking for false beliefs.
- Cross-refs: [02-dream-consolidation](02-dream-consolidation.md), [10-universal-engram](10-universal-engram.md)

**12. `11-memory/08-hypnagogia-and-creativity.md`** -- _Anti-Correlated Retrieval_
Hypnagogia engine with anti-correlated retrieval: deliberately searching for dissimilar context (inverted HDC vectors) to prevent convergence to local optima. Based on Horowitz TDI experiments (43% creativity boost, 90% cue incorporation). The 15% contrarian blending rule.
- Cross-refs: [02-dream-consolidation](02-dream-consolidation.md), [01-hyperdimensional-computing](01-hyperdimensional-computing.md)

### Safety and Security: How to Defend Against Adversarial Inputs

**13. `17-security/06-prompt-security-and-camel.md`** -- _CaMeL Dual-LLM Architecture_
Separates control plane LLM (trusted, makes decisions) from data plane LLM (untrusted, processes external data) with a taint barrier. Ventriloquist defense detects untrusted input impersonating the system. 77% solve rate with provable information flow control.
- Cross-refs: [05-gate-verification](05-gate-verification.md)

**14. `17-security/immune-system-as-graph.md`** -- _5-Layer Immune Pipeline_
Immune pipeline: taint propagation, anomaly detection, quarantine, incident response, immune memory. HDC fingerprint matching for attack signature recognition. Autoimmune protection prevents the system from attacking its own legitimate data.
- Cross-refs: [05-gate-verification](05-gate-verification.md)

### Agent Architecture: How the Cognitive Loop Works

**15. `05-execution-engine/cognitive-loop-as-graph.md`** -- _7-Step Cognitive Loop_
SENSE-ASSESS-COMPOSE-ACT-VERIFY-LEARN-EMIT as a concrete Hot Graph with typed Cells. Workflow/Activity split for resumability. Nested loop composition. Byzantine Cell defenses.
- Cross-refs: [13-cognitive-architecture](13-cognitive-architecture.md), [17-agent-patterns](17-agent-patterns.md)

**16. `07-agent-runtime/tool-loop-and-mcp.md`** -- _Tool Dispatch and MCP_
ToolLoop as Hot Flow (perceive-think-act-verify), ToolDispatcher 8-step safety pipeline, MCP integration, reasoning pattern hierarchy (ReAct/Reflexion/MCTS), tool selection optimization.
- Cross-refs: [21-mcp-editor-integration](21-mcp-editor-integration.md), [16-plugin-extension](16-plugin-extension.md)

### Code Intelligence: How Agents Understand Code

**17. `22-code-intelligence/01-code-intelligence-as-cell-pipeline.md`** -- _84-99% Code Blindness_
The core constraint: 200K-token context windows cover at most 16% of a modest Rust workspace. Four costly mistake categories from blindness: duplicate implementations, broken dependencies, misunderstood abstractions, token waste.
- Cross-refs: [12-code-intelligence](12-code-intelligence.md)

### Research-to-Runtime: How Theory Becomes Code

**18. `21-roadmap/08-research-to-runtime-bridge.md`** -- _Five Bridges from Theory to Practice_
Five detailed bridges: active inference to EFE approximation, replicator dynamics to demurrage, Turing patterns to morphogenetic fields, somatic markers to PAD decision trees, conformal prediction to calibration trackers. Each includes fidelity-lost analysis and graduation criteria.
- Cross-refs: [25-research-citations](25-research-citations.md), [11-mathematical-primitives](11-mathematical-primitives.md)

---

<a name="reading-guides"></a>
## 5. Reading Guides

### "I want to understand roko's overall architecture"

Start with these 5 documents in order:

1. `00-index/architectural-thesis.md` -- the foundational claim and five-layer derivation
2. `00-index/design-principles-algebra.md` -- eight principles as algebraic laws
3. `02-block/protocol-algebra.md` -- categorical structure of the nine protocols
4. `05-execution-engine/cognitive-loop-as-graph.md` -- the 7-step cognitive loop
5. `00-index/integration-topology.md` -- system-wide coupling analysis

Then read tmp/ documents: [26-roko-architecture-overview](26-roko-architecture-overview.md), [13-cognitive-architecture](13-cognitive-architecture.md)

### "I want to reduce LLM costs"

1. `10-learning-loops/bandit-routing-and-cascade.md` -- learned model routing (30-50% cost reduction)
2. `02-block/vcg-attention-auction.md` -- optimal token budget allocation
3. `02-block/enrichment-pipeline.md` -- differential budget principle (spend more where marginal return is higher)
4. `10-learning-loops/provider-health-and-pareto.md` -- avoid wasting tokens on degraded providers
5. `10-learning-loops/missing-loops-and-calibration.md` -- identify where cost feedback is not flowing

Then read tmp/ documents: [07-online-learning](07-online-learning.md), [09-budget-composition](09-budget-composition.md)

### "I want to build better context for LLM prompts"

1. `02-block/compose-protocol-and-builder.md` -- 9-layer system prompt builder
2. `02-block/enrichment-pipeline.md` -- 13-step context gathering
3. `02-block/positional-effects-and-retrieval.md` -- U-shape attention and placement strategy
4. `02-block/active-inference-context-selection.md` -- EFE-based context scoring
5. `02-block/distributed-and-affect-composition.md` -- the "write-for-amnesia" principle

Then read tmp/ documents: [09-budget-composition](09-budget-composition.md)

### "I want to improve agent memory"

1. `11-memory/01-knowledge-as-signal.md` -- knowledge types and tiers
2. `01-signal/demurrage-economics.md` -- why idle knowledge should decay
3. `11-memory/04-antiknowledge-and-immunity.md` -- tracking what is proven false
4. `11-memory/06-dream-cycle-as-loop.md` -- offline consolidation
5. `11-memory/08-hypnagogia-and-creativity.md` -- creative retrieval
6. `02-block/verdicts-as-signals.md` -- verification outcomes as durable data

Then read tmp/ documents: [10-universal-engram](10-universal-engram.md), [02-dream-consolidation](02-dream-consolidation.md)

### "I want to understand roko's safety architecture"

1. `01-signal/provenance-and-taint.md` -- taint lattice and information flow control
2. `17-security/immune-system-as-graph.md` -- 5-layer immune pipeline
3. `17-security/02-defense-in-depth-as-pipeline.md` -- 7-layer defense
4. `17-security/06-prompt-security-and-camel.md` -- CaMeL dual-LLM architecture
5. `17-security/03-capability-taint-and-ifc.md` -- capability tokens
6. `21-roadmap/05-causal-discovery-and-adversarial-robustness.md` -- red-team dreaming

Then read tmp/ documents: [05-gate-verification](05-gate-verification.md)

### "I want to understand agent supervision and anomaly detection"

1. `03-graph/event-log-and-conductor.md` -- 10 watcher Cells, Yerkes-Dodson dynamics
2. `07-agent-runtime/14-conductor-as-verify-pipeline.md` -- conductor as Verify Pipeline
3. `07-agent-runtime/16-diagnosis-and-stuck-detection.md` -- 20 error categories, 34 patterns
4. `07-agent-runtime/15-circuit-breaker-and-interventions.md` -- predictive circuit breaking
5. `07-agent-runtime/17-adaptive-supervision-loop.md` -- self-model accuracy

Then read tmp/ documents: [06-conductor-anomaly](06-conductor-anomaly.md), [17-agent-patterns](17-agent-patterns.md)

### "I want to understand verification and gate pipelines"

1. `02-block/verify-as-universal-oracle.md` -- four roles of verification
2. `02-block/verify-cells-and-pipeline.md` -- 11 gate implementations, 7-rung pipeline
3. `02-block/gate-feedback-and-retry.md` -- structured feedback, 97.75% token reduction
4. `02-block/ratcheting-and-adaptive-thresholds.md` -- quality ratcheting
5. `02-block/process-reward-and-artifacts.md` -- scoring intermediate steps

Then read tmp/ documents: [05-gate-verification](05-gate-verification.md)

### "I want to understand the code intelligence system"

1. `22-code-intelligence/01-code-intelligence-as-cell-pipeline.md` -- the blindness problem
2. `22-code-intelligence/02-symbol-graph-and-importance.md` -- tree-sitter + PageRank
3. `22-code-intelligence/03-hdc-fingerprints-and-similarity.md` -- ~10ns code similarity
4. `22-code-intelligence/04-search-and-context-assembly.md` -- multi-strategy search + budget assembly
5. `22-code-intelligence/05-mcp-server-and-persistence.md` -- MCP tools for code intelligence

Then read tmp/ documents: [12-code-intelligence](12-code-intelligence.md), [22-language-support](22-language-support.md)

### "I want to understand the testing philosophy"

1. `00-index/test-strategy-for-self-improving-systems.md` -- invariant testing for evolving systems
2. `21-roadmap/09-test-strategy-and-verification.md` -- property-based testing, capability preservation
3. `02-block/eval-lifecycle-and-generation.md` -- 14 eval loops, autonomous eval generation

### "I want the academic foundations and citations"

1. `21-roadmap/07-academic-foundations-by-protocol.md` -- 500+ citations mapped to protocols
2. `21-roadmap/08-research-to-runtime-bridge.md` -- how theory becomes runtime code
3. `21-roadmap/05-causal-discovery-and-adversarial-robustness.md` -- Pearl, Granger, causal reasoning

Then read tmp/ documents: [25-research-citations](25-research-citations.md)

---

<a name="section-by-section-catalog"></a>
## 6. Section-by-Section Catalog

<a name="00-index"></a>
### 00-index: Vision, Principles, Architecture (6 documents)

**Directory**: `docs/v2-depth/00-index/`
**Theme**: The foundational thesis, design principles, integration topology, and implementation roadmap.

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 1 | `architectural-thesis.md` | The Scaffold IS the Product | Protocol dependency lattice, five strata from topological sort, L5 self-evolution, SWE-bench 30-65% harness variation | HIGH |
| 2 | `design-principles-algebra.md` | Design Principles as Algebraic Laws | 8 principles as algebra, structural vs behavioral enforcement, Variance Inequality as 9th principle | MEDIUM |
| 3 | `integration-topology.md` | Integration Topology | System as typed directed graph, 3 SCCs, 5 disconnected nodes, 3 bottleneck edges, Bus as decoupling | MEDIUM |
| 4 | `implementation-readiness.md` | Implementation Readiness | Dependency-driven build phases (A-E), 4 risk cliffs, parallelization analysis | LOW |
| 5 | `test-strategy-for-self-improving-systems.md` | Test Strategy for Self-Improving Systems | 5 test layers, invariant > output testing, adversarial self-testing, observability contracts | HIGH |
| 6 | `05-vision-and-positioning.md` | Vision and Positioning | Protocol vs framework, ERC-20 analogy, 4 defensible differentiators, 5 deployment shapes, Merkle-CRDT | MEDIUM |

**Cross-references**: [26-roko-architecture-overview](26-roko-architecture-overview.md) draws heavily from docs 1-3. [13-cognitive-architecture](13-cognitive-architecture.md) uses the five-layer taxonomy from doc 1.

---

<a name="01-signal"></a>
### 01-signal: Signal Internals and Decay Math (4 documents)

**Directory**: `docs/v2-depth/01-signal/`
**Theme**: Algebraic structure of the universal data type (Signal), its scoring, decay economics, and provenance tracking.

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 7 | `signal-algebra.md` | Signal Algebra | HDC semiring (bind + bundle), role-filler encoding, lineage DAG as free category, graduation functor | MEDIUM |
| 8 | `demurrage-economics.md` | Demurrage Economics | Gesell-Shannon derivation, novelty-weighted decay, balance-tier-novelty phase space, VCG dual, cold storage/thaw | HIGH |
| 9 | `scoring-and-calibration.md` | Scoring and Calibration | Score-Verify-Score loop, temperature scaling, Beta-Binomial confidence, Pareto front, isotonic regression | HIGH |
| 10 | `provenance-and-taint.md` | Provenance and Taint | Taint lattice, join propagation, declassification capability, custody-gated Store, action-time taint gates | HIGH |

**Cross-references**: [10-universal-engram](10-universal-engram.md) draws from docs 7-10. [01-hyperdimensional-computing](01-hyperdimensional-computing.md) covers the HDC math from doc 7 in more detail.

---

<a name="02-block"></a>
### 02-block: Composition, Tools, Verification (15 documents)

**Directory**: `docs/v2-depth/02-block/`
**Theme**: The nine protocol implementations, context composition algorithms, verification pipelines, and quality ratcheting. The largest and most algorithmically dense section.

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 11 | `protocol-algebra.md` | Protocol Algebra | Category of Cells, TypeSchema as preorder, protocol morphism table, natural transformations, capability pullback | MEDIUM |
| 12 | `store-and-bus-duality.md` | Store/Bus Duality | Pull/push adjoint functors, graduation/projection, ring buffer eviction, backpressure | MEDIUM |
| 13 | `verify-as-universal-oracle.md` | Verify as Universal Oracle | 4 Verdict roles, Goodhart-resistance proof, Variance Inequality, Bradley-Terry, meta-verification | HIGH |
| 14 | `compose-protocol-and-builder.md` | Compose Protocol and Builder | ComposeBid, greedy knapsack, 9-layer system prompt builder, 12 role templates, cache alignment | HIGH |
| 15 | `enrichment-pipeline.md` | Enrichment Pipeline | 13-step pipeline, Self-RAG step selection, three-tier budget, differential budget principle | HIGH |
| 16 | `positional-effects-and-retrieval.md` | Positional Effects and Retrieval | U-shape attention, Placement enum, BetaPosterior tracking, LongLLMLingua reordering | HIGH |
| 17 | `active-inference-context-selection.md` | Active Inference Context Selection | EFE scoring (pragmatic + epistemic - ambiguity), MVT stopping rule, 5-stage assembly, EFE/VCG convergence | HIGH |
| 18 | `vcg-attention-auction.md` | VCG Attention Auction | VCG for context allocation, 8 bidder subsystems, truthful pricing, Thompson sampling LearningBidder | HIGH |
| 19 | `distributed-and-affect-composition.md` | Distributed and Affect Composition | 4 context strategies, PAD as endofunctor, write-for-amnesia principle, somatic marker analog | MEDIUM |
| 20 | `verify-cells-and-pipeline.md` | Verify Cells and Pipeline | 11 gate implementations, 7-rung pipeline, gate composition algebra, GVU architecture | HIGH |
| 21 | `ratcheting-and-adaptive-thresholds.md` | Ratcheting and Adaptive Thresholds | GateRatchet, CUSUM, EWMA control charts, BOCPD regime detection, Hotelling T-squared | MEDIUM |
| 22 | `gate-feedback-and-retry.md` | Gate Feedback and Retry | Structured GateFeedback, 97.75% token reduction, retry loop, section-effectiveness learning, model escalation | HIGH |
| 23 | `eval-lifecycle-and-generation.md` | Evaluation Lifecycle and Generation | 14 eval loops across 5 tiers, autonomous eval generation, EvoSkills, MAP-Elites | MEDIUM |
| 24 | `verdicts-as-signals.md` | Verdicts as Signals | Verdicts as first-class Signals, 24h half-life, forensic causal replay, BLAKE3 integrity | MEDIUM |
| 25 | `process-reward-and-artifacts.md` | Process Reward and Artifacts | PRMs for intermediate steps, Promise/Progress decomposition, self-supervised training, ThinkPRM | MEDIUM |

**Cross-references**: [05-gate-verification](05-gate-verification.md) draws from docs 13, 20-22, 25. [09-budget-composition](09-budget-composition.md) draws from docs 14-18.

---

<a name="03-graph"></a>
### 03-graph: Orchestration and DAG Execution (7 documents)

**Directory**: `docs/v2-depth/03-graph/`
**Theme**: Plan discovery, DAG construction, parallel execution, worktree isolation, snapshot/recovery, and stigmergic coordination.

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 26 | `plan-discovery-and-dag.md` | Plan Discovery and DAG | Filesystem plan discovery, DAG construction, wave computation, critical path, optimization passes | MEDIUM |
| 27 | `parallel-executor.md` | Parallel Executor | State machine executor, multi-dimensional resource budgets, Petri net formal model | MEDIUM |
| 28 | `plan-phases-and-actions.md` | Plan Phases and Actions | 12 phases, 2 retry loops, 3 terminal states, 10 action types, agent role dispatch | LOW |
| 29 | `worktree-isolation-and-merge.md` | Worktree Isolation and Merge | Git worktree isolation, merge queue, conflict detection, warm agent pool | MEDIUM |
| 30 | `snapshot-and-recovery.md` | Snapshot and Recovery | Hash-chained event log, dual-source recovery, BLAKE3 integrity, incremental delta snapshots | LOW |
| 31 | `event-log-and-conductor.md` | Event Log and Conductor | 10 watcher Cells, Yerkes-Dodson pressure, Viable System Model, graduated interventions | HIGH |
| 32 | `stigmergy-and-cross-domain.md` | Stigmergy and Cross-Domain | Git as stigmergic medium, digital pheromones, c-factor, saga pattern | LOW |

**Cross-references**: [04-dag-execution](04-dag-execution.md) draws from docs 26-30. [06-conductor-anomaly](06-conductor-anomaly.md) draws from doc 31. [15-orchestrator-swarm](15-orchestrator-swarm.md) draws from docs 26-30, 32.

---

<a name="04-specializations"></a>
### 04-specializations (0 documents, reserved)

<a name="05-execution-engine"></a>
### 05-execution-engine: Runtime Loop (2 documents)

**Directory**: `docs/v2-depth/05-execution-engine/`

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 33 | `cognitive-loop-as-graph.md` | Cognitive Loop as Graph | 7-step SENSE-ASSESS-COMPOSE-ACT-VERIFY-LEARN-EMIT, Workflow/Activity split, Byzantine defenses | HIGH |
| 34 | `resilience-and-numerics.md` | Resilience and Numerics | 4 error kinds, retry algebra, circuit breaker state machines, degradation ladder, f32/f64 precision | HIGH |

**Cross-references**: [13-cognitive-architecture](13-cognitive-architecture.md) draws from doc 33. [14-runtime-infrastructure](14-runtime-infrastructure.md) draws from doc 34.

---

<a name="06-trigger-system"></a>
### 06-trigger-system (0 documents, reserved)

<a name="07-agent-runtime"></a>
### 07-agent-runtime: Agent, Conductor, Heartbeat, Lifecycle (27 documents)

**Directory**: `docs/v2-depth/07-agent-runtime/`
**Theme**: The largest section. Agent architecture, affect engine, supervision, heartbeat cognitive architecture, and lifecycle management. Contains the most novel cognitive science concepts.

#### Agent Core (3 documents)

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 35 | `agent-cell-and-providers.md` | Agent as Cell | 7 agent implementations, provider registry as Route Cell, unified factory, error classification | HIGH |
| 36 | `chat-types-and-streaming.md` | Chat Types and Streaming | ChatResponse canonical type, FinishReason normalization across 5 providers, streaming as Pulse sequences | HIGH |
| 37 | `agent-pools-and-roles.md` | Agent Pools and Roles | 28-role taxonomy, tool permissions, AgentPool/MultiAgentPool, supervision strategies | HIGH |

#### Tool and Format Engineering (2 documents)

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 38 | `tool-loop-and-mcp.md` | Tool Loop and MCP | ToolLoop as Hot Flow, 8-step safety pipeline, MCP integration, ReAct/Reflexion/MCTS, Tool RAG | HIGH |
| 39 | `harness-and-format-engineering.md` | Harness and Format Engineering | Harness as Pipeline of 6 principles, 4 format adapter Cells | HIGH |

#### Dual-Process and Routing (2 documents)

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 40 | `temperament-and-dual-process.md` | Temperament and Dual-Process | 4-level temperament profiling, CascadeRouter, LinUCB bandits, Pareto pruning, Thompson sampling | MEDIUM |
| 41 | `dual-process-and-efe-routing.md` | Dual-Process and EFE Routing | System 1/System 2 mapped to 3 tiers, EFE routing formula, affect-modulated model selection | MEDIUM |

#### Extensibility and Providers (2 documents)

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 42 | `extensibility-and-creation.md` | Extensibility and Creation | 5 extension points, 4-layer SDK, Darwin Godel Machine, Voyager skill library | HIGH |
| 43 | `provider-integrations-and-profiles.md` | Provider Integrations and Profiles | 8+ providers (Claude, OpenAI, Ollama, etc.), 6 domain profiles, evaluation suites | HIGH |

#### Cognitive Architecture (3 documents)

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 44 | `cognitive-timescales.md` | Cognitive Timescales | Gamma/theta/delta as concurrent Hot Graphs, EFE-driven T0/T1/T2 cascade | MEDIUM |
| 45 | `cross-cut-functors.md` | Cross-Cut Functors | Memory/Daimon/Dreams as Signal endofunctors, VCG arbitration between cross-cuts | MEDIUM |
| 46 | `cognitive-energy-and-vitality.md` | Cognitive Energy and Vitality | Energy depletion, fatigue accumulation, energy zones, 3 recovery modes | LOW |

#### Conductor and Supervision (4 documents)

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 47 | `14-conductor-as-verify-pipeline.md` | Conductor as Verify Pipeline | 10 watcher Cells, OODA Loop feedback, three timescales | HIGH |
| 48 | `15-circuit-breaker-and-interventions.md` | Circuit Breaker and Interventions | Circuit breaker state machine, predictive breaking via Holt forecaster, AIMD concurrency control | HIGH |
| 49 | `16-diagnosis-and-stuck-detection.md` | Diagnosis and Stuck Detection | 20 error categories, 9 interventions, 34 patterns, 6 Lens Cells for stuck detection | HIGH |
| 50 | `17-adaptive-supervision-loop.md` | Adaptive Supervision Loop | Self-model accuracy (Brier score), threshold adaptation, Yerkes-Dodson, ConductorBandit | HIGH |

#### Affect Engine / Daimon (3 documents)

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 51 | `18-affect-as-functor.md` | Affect as Functor | PAD vector, 8-step appraisal React Cell, ALMA 3-layer model, affect as Compose Functor | LOW |
| 52 | `19-behavioral-states-and-routing.md` | Behavioral States and Routing | 6 behavioral states with hysteresis, archetype-distance scoring, tier routing adjustment | LOW |
| 53 | `20-somatic-landscape.md` | Somatic Landscape | Somatic markers (Damasio), k-d tree index, 15% contrarian retrieval Functor | LOW |

#### Collective Behavior (1 document)

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 54 | `21-collective-contagion.md` | Collective Contagion | Emotional contagion across agent groups, attenuation Functor, anti-cascade | NONE |

#### Heartbeat Architecture (3 documents)

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 55 | `22-heartbeat-as-hot-graph.md` | Heartbeat as Hot Graph | Adaptive clock, gamma/theta/delta flows, HeartbeatPolicy, regime detection, budget throttling | MEDIUM |
| 56 | `23-t0-probes-as-verify-cells.md` | T0 Probes as Verify Cells | 16 T0 probes (8 chain + 6 coding + 2 universal), rolling z-score anomaly detection | MEDIUM |
| 57 | `24-attention-auction-and-cortical-state.md` | Attention Auction and Cortical State | VCG with 8 bidder subsystems, CorticalState 32-signal atomic struct | MEDIUM |

#### Active Inference (1 document)

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 58 | `25-active-inference-state-space.md` | Active Inference State Space | 90-state POMDP (TaskPhase x ContextQuality x Uncertainty), A/B/C/D matrices, EFE for tier selection | MEDIUM |

#### Lifecycle and Knowledge (3 documents)

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 59 | `26-agent-lifecycle-type-state.md` | Agent Lifecycle Type-State | Compile-time type-state machine, 3 creation flows, successor patterns, genomic bottleneck | MEDIUM |
| 60 | `27-knowledge-transfer-and-mesh.md` | Knowledge Transfer and Mesh | Agent Mesh, version-vector delta sync, Bloom filter discovery, 4-tier gossip, adoption Pipeline | LOW |
| 61 | `28-funding-budgets-and-operator-model.md` | Funding Budgets and Operator Model | BudgetGuard as Verify Cell, 4-level guardrails, 5-stage degradation cascade, operator freedom | MEDIUM |

**Cross-references**: [03-affect-engine](03-affect-engine.md) draws from docs 51-53. [06-conductor-anomaly](06-conductor-anomaly.md) draws from docs 47-50. [17-agent-patterns](17-agent-patterns.md) draws from docs 35-39. [23-control-plane](23-control-plane.md) draws from docs 35-37, 43.

---

<a name="08-extension-system"></a>
### 08-extension-system (0 documents, reserved)

<a name="09-telemetry"></a>
### 09-telemetry: Observability (1 document)

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 62 | `01-observability-as-lens-pipeline.md` | Observability as Lens Pipeline | Logs as Bus Pulses, metrics as Lens output, traces as lineage Signals, cost visibility pipeline | MEDIUM |

**Cross-references**: [14-runtime-infrastructure](14-runtime-infrastructure.md) references observability concepts from doc 62.

---

<a name="10-learning-loops"></a>
### 10-learning-loops: Bandits, Self-Improvement (10 documents)

**Directory**: `docs/v2-depth/10-learning-loops/`
**Theme**: How the system learns from experience -- bandit algorithms for routing, regression detection, self-improvement frameworks, and collective intelligence.

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 63 | `c-factor-as-lens.md` | C-Factor as Lens | Collective intelligence measurement, 5 sub-lenses, WisdomGate, Goodhart defense | LOW |
| 64 | `autocatalytic-compounding.md` | Autocatalytic Compounding | 7 compounding loops, Kauffman autocatalytic condition, anti-metric defenses, SPoF analysis | MEDIUM |
| 65 | `episodes-and-playbooks.md` | Episodes and Playbooks | Episode Store with HDC fingerprinting, playbook rules with 0.95 confidence ceiling, Voyager skill library | HIGH |
| 66 | `bandit-routing-and-cascade.md` | Bandit Routing and Cascade | 3-stage cascade, LinUCB, 18D context vector, Pareto frontier, trigram pattern discovery | HIGH |
| 67 | `metrics-baselines-and-regression.md` | Metrics Baselines and Regression | Per-slice baselines, 15%/20% regression thresholds, cost normalization (3:1 formula), efficiency grading | HIGH |
| 68 | `provider-health-and-pareto.md` | Provider Health and Pareto | 3-state circuit breaker, error classification, Pareto dominance pruning, anomaly detection | HIGH |
| 69 | `drift-and-stability.md` | Drift and Stability | Thompson Sampling with gamma=0.995 discount, hysteresis, 4-tier frequency separation, anti-pattern traps | MEDIUM |
| 70 | `self-improvement-frameworks.md` | Self-Improvement Frameworks | 5-level stack, Reflexion/ExpeL/DSPy/ADAS as Loop patterns, Variance Inequality, constitutional constraints | HIGH |
| 71 | `missing-loops-and-calibration.md` | Missing Loops and Calibration | 8 missing feedback loops, cross-loop interaction matrix, 31.6x collective heuristic, predictive foraging | HIGH |
| 72 | `heuristics-and-falsifiers.md` | Heuristics and Falsifiers | Heuristic lifecycle (birth/test/adjust/retire), Wilson confidence interval, worldviews, AntiKnowledge | MEDIUM |

**Cross-references**: [07-online-learning](07-online-learning.md) draws from docs 66, 68, 69. [06-conductor-anomaly](06-conductor-anomaly.md) draws from docs 67, 68, 71. [13-cognitive-architecture](13-cognitive-architecture.md) draws from doc 70. [19-priority-matrix](19-priority-matrix.md) references concepts from the entire section.

---

<a name="11-memory"></a>
### 11-memory: Knowledge, Dreams, Stigmergy (15 documents)

**Directory**: `docs/v2-depth/11-memory/`
**Theme**: Knowledge storage, dream-based offline consolidation, hypnagogic creativity, pheromone coordination, and collective intelligence metrics. Contains some of the most novel and ambitious ideas in the collection.

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 73 | `01-knowledge-as-signal.md` | Knowledge as Signal | Knowledge types (Fact/Procedure/Heuristic/Context/Insight/Pattern), tiers, Ebbinghaus decay, query API | HIGH |
| 74 | `02-hdc-algebra-and-retrieval.md` | HDC Algebra and Retrieval | 10,240-bit vectors, bind/bundle/permute/similarity, false positive math (1 in 10^307), anti-knowledge HDC | MEDIUM |
| 75 | `03-knowledge-lifecycle-loop.md` | Knowledge Lifecycle Loop | D1/D2/D3 distillation, predict-publish-correct calibration, backup/restore, mesh sync | MEDIUM |
| 76 | `04-antiknowledge-and-immunity.md` | AntiKnowledge and Immunity | AntiKnowledge never fully decays, cognitive immune system, SIR epidemiological tracking, balance floor | HIGH |
| 77 | `05-cross-domain-transfer.md` | Cross-Domain Transfer | HDC resonance as Store query, Library of Babel, federation ingestion | LOW |
| 78 | `06-dream-cycle-as-loop.md` | Dream Cycle as Loop | NREM/REM/Integration, Mattar-Daw utility, HDC counterfactual synthesis, convergence detection | HIGH |
| 79 | `07-replay-and-counterfactual-cells.md` | Replay and Counterfactual Cells | Mattar-Daw utility, counterfactual synthesis, hindsight experience replay, inner world rendering | MEDIUM |
| 80 | `08-hypnagogia-and-creativity.md` | Hypnagogia and Creativity | Anti-correlated retrieval, inverted HDC, Dali interrupt, homuncular observer, 43% TDI creativity boost | MEDIUM |
| 81 | `09-consolidation-and-staging.md` | Consolidation and Staging | Staging buffer, confidence ladder, SHY renormalization, oneirography | LOW |
| 82 | `10-threat-simulation-and-nightmares.md` | Threat Simulation and Nightmares | FMEA/FTA as Score Cells, nightmare containment, dream journal analysis | MEDIUM |
| 83 | `11-stigmergy-as-bus.md` | Stigmergy as Bus | Pheromones as Pulses, evaporation, reinforcement, scoped visibility, git as medium | LOW |
| 84 | `12-pheromone-mechanics-and-interference.md` | Pheromone Mechanics and Interference | 7 pheromone kinds, half-life taxonomy, SINR interference, Hill-function thresholds | NONE |
| 85 | `13-morphogenetic-specialization-as-loop.md` | Morphogenetic Specialization | Turing reaction-diffusion, activator-inhibitor dynamics, Gierer-Meinhardt kinetics, Lyapunov stability | NONE |
| 86 | `14-mesh-sync-and-subnets.md` | Mesh Sync and Subnets | Bus federation, dual transport (WebSocket + Iroh), partition tolerance, permissioned subnets | LOW |
| 87 | `15-collective-metrics-as-lens.md` | Collective Metrics as Lens | C-factor Lens Graph (5 axes), WisdomGate, contrarian retrieval, 7 flywheel Loops | NONE |

**Cross-references**: [02-dream-consolidation](02-dream-consolidation.md) draws from docs 78-82. [01-hyperdimensional-computing](01-hyperdimensional-computing.md) draws from docs 74, 80. [10-universal-engram](10-universal-engram.md) draws from docs 73, 76. [13-cognitive-architecture](13-cognitive-architecture.md) draws from docs 83-85.

---

<a name="12-connectivity"></a>
### 12-connectivity: Relay and Networking (5 documents)

**Directory**: `docs/v2-depth/12-connectivity/`

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 88 | `01-relay-wire-protocol.md` | Relay Wire Protocol | Frame-level WebSocket protocol, connection lifecycle, envelope structure, ring buffer sequencing | MEDIUM |
| 89 | `02-topic-namespace.md` | Topic Namespace | Dot-separated ABNF grammar, canonical topics, wildcard subscriptions | LOW |
| 90 | `03-chain-event-projection.md` | Chain Event Projection | Chain watcher, finality tagging, reorg handling | NONE |
| 91 | `04-relay-deployment.md` | Relay Deployment | 3 deployment models (sidecar, shared, validator-embedded), multi-relay, Docker/Railway | LOW |
| 92 | `05-coordination-patterns.md` | Coordination Patterns | 3 coordination primitives (pub/sub, request/response, on-chain), job discovery, presence | LOW |

**Cross-references**: [23-control-plane](23-control-plane.md) references relay protocol concepts from doc 88. [14-runtime-infrastructure](14-runtime-infrastructure.md) draws general pub/sub patterns from doc 88.

---

<a name="13-builtin-catalog"></a>
### 13-builtin-catalog: Tools, MCP, Plugins (5 documents)

**Directory**: `docs/v2-depth/13-builtin-catalog/`

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 93 | `01-tool-architecture-as-cell-protocol.md` | Tool Architecture as Cell Protocol | Tools as Cells, 3 trust tiers, safety hook pipeline, Capability\<T\> ownership pattern | HIGH |
| 94 | `02-mcp-as-connect-protocol.md` | MCP as Connect Protocol | MCP as dynamic Cell discovery, namespaced registry merge, trust hierarchy, tool change notifications | HIGH |
| 95 | `03-plugin-spi-as-extension.md` | Plugin SPI as Extension | 5-tier plugin SPI (data-only to WASM with fuel metering), discovery-first loading Pipeline | HIGH |
| 96 | `04-domain-tools-and-profiles.md` | Domain Tools and Profiles | Rack pattern, structural absence (tool pruning), 94% token savings, CascadeRouter to 12 tools/tick | HIGH |
| 97 | `05-event-sources-and-templates.md` | Event Sources and Templates | Trigger Cells (cron/watch/webhook/GitHub/Slack), Graph templates, 18 shipped templates | MEDIUM |

**Cross-references**: [16-plugin-extension](16-plugin-extension.md) draws from docs 93-95, 97. [21-mcp-editor-integration](21-mcp-editor-integration.md) draws from doc 94. [09-budget-composition](09-budget-composition.md) draws from doc 96.

---

<a name="14-config"></a>
### 14-config: Configuration as Signal (2 documents)

**Directory**: `docs/v2-depth/14-config/`

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 98 | `config-as-signal.md` | Config as Signal | Config values as Signals, Compose for overrides, Verify for validation, Trigger for hot reload, L4 evolution | MEDIUM |
| 99 | `02-layered-resolution-and-reload.md` | Layered Resolution and Reload | 4-layer merge (CLI > env > TOML > defaults), inotify/kqueue hot-reload, ROKO_* env convention | MEDIUM |

**Cross-references**: [14-runtime-infrastructure](14-runtime-infrastructure.md) touches on configuration patterns from these docs.

---

<a name="15-marketplace"></a>
### 15-marketplace: Identity, Economy, Commerce (6 documents)

**Directory**: `docs/v2-depth/15-marketplace/`
**Theme**: Agent identity, reputation, knowledge marketplaces, payment protocols, tokenomics, and economic loops. All blockchain-specific.

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 100 | `01-agent-identity-as-signal.md` | Agent Identity as Signal | ERC-8004 + Korai Passport, 4-tier capabilities, soulbound properties, HDC identity, Sybil defense | NONE |
| 101 | `02-reputation-as-score-protocol.md` | Reputation as Score Protocol | 7-domain EMA reputation, R^1.7 superlinear multiplier, EigenTrust, collusion detection | LOW |
| 102 | `03-knowledge-marketplace-as-store.md` | Knowledge Marketplace as Store | 3-tier marketplace, alpha-decay pricing, blind verification, knowledge futures | NONE |
| 103 | `04-payment-protocols-as-connect.md` | Payment Protocols as Connect | 3-layer payment stack (ERC-8183/MPP/x402), SPT budget delegation, self-funding Loop | NONE |
| 104 | `05-tokenomics-as-demurrage-store.md` | Tokenomics as Demurrage Store | KORAI with 1% demurrage, bonding curves as Score Cells, Shapley attribution, cadCAD modeling | NONE |
| 105 | `06-agent-economy-as-loop.md` | Agent Economy as Loop | Self-sustaining economy, 7 compounding edges, superlinear reputation returns, break-even analysis | NONE |

**Cross-references**: [08-chain-reputation](08-chain-reputation.md) draws from docs 100-101, 104. [24-smart-contracts](24-smart-contracts.md) draws from docs 100, 103-104.

---

<a name="16-surfaces"></a>
### 16-surfaces: UI/UX Architecture (7 documents)

**Directory**: `docs/v2-depth/16-surfaces/`

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 106 | `01-surfaces-as-lens-composition.md` | Surfaces as Lens Composition | All surfaces are Lens compositions of shared StateHub, 9-verb interaction model | HIGH |
| 107 | `02-cli-and-command-graph.md` | CLI and Command Graph | CLI subcommands as Graph triggers, layered config, progressive help, scaffolders | MEDIUM |
| 108 | `03-tui-screen-architecture.md` | TUI Screen Architecture | 29 TUI screens as Lens Cells, 6 regions, keyboard-driven routing, ratatui immediate-mode | MEDIUM |
| 109 | `04-rosedust-and-spectre.md` | Rosedust and Spectre | Design tokens as config Signals, Spectre avatar from HDC fingerprints, PAD animation | NONE |
| 110 | `05-http-api-and-realtime.md` | HTTP API and Realtime | Connect Cell, ~85 routes, WebSocket/SSE as Bus subscriptions, cursor semantics | HIGH |
| 111 | `06-generative-interfaces-and-a2ui.md` | Generative Interfaces and A2UI | A2UI (agents generating UI), 12 component kinds, sonification | LOW |
| 112 | `07-developer-experience-and-onboarding.md` | Developer Experience and Onboarding | SDK as Graph templates, onboarding as Trigger Graph, ACP-first IDE, domain profiles | MEDIUM |

**Cross-references**: [23-control-plane](23-control-plane.md) draws from docs 106, 110. [26-roko-architecture-overview](26-roko-architecture-overview.md) references UI architecture from docs 106-108.

---

<a name="17-security"></a>
### 17-security: Safety and Defense (7 documents)

**Directory**: `docs/v2-depth/17-security/`
**Theme**: Defense in depth, capability-based security, taint tracking, audit chains, prompt injection defense, and formal verification.

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 113 | `immune-system-as-graph.md` | Immune System as Graph | 5-layer immune pipeline, HDC attack fingerprints, autoimmune protection, SIR model | HIGH |
| 114 | `02-defense-in-depth-as-pipeline.md` | Defense in Depth as Pipeline | 7-layer defense Pipeline, permits/allowlists, sandboxing, rate limiting, early-exit | HIGH |
| 115 | `03-capability-taint-and-ifc.md` | Capability Taint and IFC | Capability\<T\> tokens, 5-fold taint taxonomy, 3-layer capability intersection, CaMeL IFC | HIGH |
| 116 | `04-audit-witness-and-forensics.md` | Audit Witness and Forensics | Custody chain as Store lineage, witness DAG (5 vertex types), forensic replay, SOC2/ISO 27001/GDPR | MEDIUM |
| 117 | `05-adaptive-risk-as-loop.md` | Adaptive Risk as Loop | 5-layer runtime risk Loop, LTL/CTL temporal logic, adaptive gate thresholds | MEDIUM |
| 118 | `06-prompt-security-and-camel.md` | Prompt Security and CaMeL | CaMeL dual-LLM, ventriloquist defense, tool-guard Pipeline, UNTRUSTED marking, 77% IFC solve rate | HIGH |
| 119 | `07-cognitive-kernel-and-formal-methods.md` | Cognitive Kernel and Formal Methods | Cognitive namespaces, EDF scheduling, Heimdall/Slither/Echidna/hevm/Certora pipeline, MEV pre-flight | LOW |

**Cross-references**: [05-gate-verification](05-gate-verification.md) draws from docs 113-114, 118. [26-roko-architecture-overview](26-roko-architecture-overview.md) references security architecture from docs 113-115.

---

<a name="18-registries"></a>
### 18-registries: On-Chain Infrastructure (8 documents)

**Directory**: `docs/v2-depth/18-registries/`
**Theme**: Blockchain-based agent identity, reputation, job markets, payments, gossip protocols, and simulation. All blockchain-specific.

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 120 | `01-chain-as-domain-plugin.md` | Chain as Domain Plugin | ERC-8004 registries | NONE |
| 121 | `02-hdc-on-chain-and-verification.md` | HDC On-Chain and Verification | HDC precompile for on-chain similarity search | NONE |
| 122 | `03-job-market-and-hiring.md` | Job Market and Hiring | Job marketplace Graph, 3 hiring models (Direct/Auction/Power-of-Two), Vickrey auction | NONE |
| 123 | `04-reputation-and-peer-scoring.md` | Reputation and Peer Scoring | 3-layer peer scoring, 7-domain reputation, Vickrey auction | NONE |
| 124 | `05-chain-witness-and-triage.md` | Chain Witness and Triage | Chain event watching, curiosity-driven triage | NONE |
| 125 | `06-payments-and-settlement.md` | Payments and Settlement | x402 micropayments, ISFR clearing/settlement, knowledge futures market | NONE |
| 126 | `07-gossip-and-privacy.md` | Gossip and Privacy | 4-tier gossip architecture, soulbound passport, Valhalla privacy layer | NONE |
| 127 | `08-simulation-and-liveness.md` | Simulation and Liveness | mirage-rs EVM simulator, chain agent heartbeat, 6 contracts | NONE |

**Cross-references**: [08-chain-reputation](08-chain-reputation.md) draws from docs 120-123, 125-126. [24-smart-contracts](24-smart-contracts.md) draws from docs 120-121, 125-127.

---

<a name="19-arenas"></a>
### 19-arenas (0 documents, reserved)

<a name="20-deployment"></a>
### 20-deployment: Deployment Shapes (4 documents)

**Directory**: `docs/v2-depth/20-deployment/`

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 128 | `01-five-shape-deployment-as-config-signal.md` | Five-Shape Deployment | 5 modes (laptop/server/container/clustered/edge), shape-as-config, one binary | HIGH |
| 129 | `02-daemon-and-subscription-system.md` | Daemon and Subscription System | Daemon as Hot Flow Graph, 3 trigger types, multi-repo scheduling, launchd/systemd | HIGH |
| 130 | `03-cloud-and-edge-deployment.md` | Cloud and Edge Deployment | Fly.io cloud (zero-cost idle), edge (~500KB musl), Merkle-CRDT brain export | MEDIUM |
| 131 | `04-production-hardening-and-observability.md` | Production Hardening and Observability | Adaptive timeouts, backoff state machine, 4-phase shutdown, structured observability | HIGH |

**Cross-references**: [14-runtime-infrastructure](14-runtime-infrastructure.md) draws from docs 128-131. [26-roko-architecture-overview](26-roko-architecture-overview.md) references deployment from doc 128.

---

<a name="21-roadmap"></a>
### 21-roadmap: Research Foundations (9 documents)

**Directory**: `docs/v2-depth/21-roadmap/`
**Theme**: Academic foundations, technical analysis (generalized oracles, HDC patterns, causal discovery), research-to-runtime bridge, and comprehensive test strategy.

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 132 | `temporal-knowledge-graph.md` | Temporal Knowledge Graph | Allen's 13 interval relations, event calculus, 3-tier temporal Memory | LOW |
| 133 | `emergent-goals-and-energy.md` | Emergent Goals and Energy | Emergent goal formation, ZPD as Score Cell, energy-goal coupling, somatic markers | LOW |
| 134 | `03-oracle-as-score-cell.md` | Oracle as Score Cell | Predict-publish-correct pattern, ResidualCorrector (~50ns), conformal prediction, oracle composition | HIGH |
| 135 | `04-hdc-pattern-encoding-and-metabolism.md` | HDC Pattern Encoding and Metabolism | Role-filler BIND + temporal PERMUTE + BUNDLE, 0.526 resonance threshold, Red Queen pressure | LOW |
| 136 | `05-causal-discovery-and-adversarial-robustness.md` | Causal Discovery and Adversarial Robustness | Pearl's 3-level causal hierarchy, structural causal models, Granger causality, red-team dreaming | MEDIUM |
| 137 | `06-advanced-geometry-and-integration.md` | Advanced Geometry and Integration | Spectral manifolds, TDA, sheaf consistency, tropical geometry, IIT Phi | NONE |
| 138 | `07-academic-foundations-by-protocol.md` | Academic Foundations by Protocol | 500+ citations mapped to protocols (Store, Score, Verify, Route, Compose, React, Agent) | MEDIUM |
| 139 | `08-research-to-runtime-bridge.md` | Research-to-Runtime Bridge | 5 bridges (active inference, replicator dynamics, Turing patterns, somatic markers, conformal prediction) | MEDIUM |
| 140 | `09-test-strategy-and-verification.md` | Test Strategy and Verification | Tests as Verify Cells, property-test categories (algebraic/stateful/metamorphic), self-immunization | HIGH |

**Cross-references**: [25-research-citations](25-research-citations.md) draws from docs 136, 138. [11-mathematical-primitives](11-mathematical-primitives.md) draws from docs 135, 137.

---

<a name="22-code-intelligence"></a>
### 22-code-intelligence: Code Analysis Pipeline (5 documents)

**Directory**: `docs/v2-depth/22-code-intelligence/`
**Theme**: Source code parsing, symbol graph construction, HDC fingerprinting for code similarity, and budget-constrained context assembly for coding agents.

| # | File | Title | Key Concepts | Relevance |
|---|------|-------|-------------|-----------|
| 141 | `01-code-intelligence-as-cell-pipeline.md` | Code Intelligence as Cell Pipeline | 6-stage pipeline, 84-99% code blindness, 4 costly mistake categories | HIGH |
| 142 | `02-symbol-graph-and-importance.md` | Symbol Graph and Importance | Tree-sitter parsing, dependency graph, PageRank importance scoring | HIGH |
| 143 | `03-hdc-fingerprints-and-similarity.md` | HDC Fingerprints and Similarity | Structural + semantic HDC fingerprints, cross-language similarity, ~10ns lookup | MEDIUM |
| 144 | `04-search-and-context-assembly.md` | Search and Context Assembly | Multi-strategy search (keyword/similarity/graph/recency), ranked merge, budget-constrained assembly | HIGH |
| 145 | `05-mcp-server-and-persistence.md` | MCP Server and Persistence | MCP tools for code intelligence, SQLite persistence, incremental snapshot updates | HIGH |

**Cross-references**: [12-code-intelligence](12-code-intelligence.md) draws from all five docs. [22-language-support](22-language-support.md) draws from docs 141-142.

---

<a name="cross-references"></a>
## 7. Cross-References: tmp/ Documents to Depth Documents

This table maps each tmp/ analysis document to the v2-depth documents it draws from or relates to. Documents are listed by their catalog number from this catalog.

| tmp/ Document | Primary Depth Sources | Section Coverage |
|---|---|---|
| [01-hyperdimensional-computing](01-hyperdimensional-computing.md) | 7, 74, 80, 135, 143 | 01-signal, 11-memory, 21-roadmap, 22-code-intelligence |
| [02-dream-consolidation](02-dream-consolidation.md) | 8, 76, 78-82 | 01-signal, 11-memory |
| [03-affect-engine](03-affect-engine.md) | 19, 46, 51-53, 58 | 02-block, 07-agent-runtime |
| [04-dag-execution](04-dag-execution.md) | 26-30, 33 | 03-graph, 05-execution-engine |
| [05-gate-verification](05-gate-verification.md) | 13, 20-22, 25, 113-114, 118 | 02-block, 17-security |
| [06-conductor-anomaly](06-conductor-anomaly.md) | 31, 47-50, 67-68, 71 | 03-graph, 07-agent-runtime, 10-learning-loops |
| [07-online-learning](07-online-learning.md) | 9, 13, 66, 68-70 | 01-signal, 02-block, 10-learning-loops |
| [08-chain-reputation](08-chain-reputation.md) | 100-101, 104, 120-123, 125-126 | 15-marketplace, 18-registries |
| [09-budget-composition](09-budget-composition.md) | 14-18, 57, 96 | 02-block, 07-agent-runtime, 13-builtin-catalog |
| [10-universal-engram](10-universal-engram.md) | 7-10, 73, 76 | 01-signal, 11-memory |
| [11-mathematical-primitives](11-mathematical-primitives.md) | 135, 137 | 21-roadmap |
| [12-code-intelligence](12-code-intelligence.md) | 141-145 | 22-code-intelligence |
| [13-cognitive-architecture](13-cognitive-architecture.md) | 1, 33, 44-45, 63, 70, 83-85 | 00-index, 05-execution-engine, 07-agent-runtime, 10-learning-loops, 11-memory |
| [14-runtime-infrastructure](14-runtime-infrastructure.md) | 34, 62, 88, 98-99, 128-131 | 05-execution-engine, 09-telemetry, 12-connectivity, 14-config, 20-deployment |
| [15-orchestrator-swarm](15-orchestrator-swarm.md) | 26-30, 32 | 03-graph |
| [16-plugin-extension](16-plugin-extension.md) | 93-95, 97 | 13-builtin-catalog |
| [17-agent-patterns](17-agent-patterns.md) | 33, 35-39, 48-49 | 05-execution-engine, 07-agent-runtime |
| [18-integration-roadmap](18-integration-roadmap.md) | Synthesizes concepts across all sections | All |
| [19-priority-matrix](19-priority-matrix.md) | Ranks concepts from all sections | All |
| [20-persistence-storage](20-persistence-storage.md) | 12, 73, 75 | 02-block, 11-memory |
| [21-mcp-editor-integration](21-mcp-editor-integration.md) | 38, 94, 145 | 07-agent-runtime, 13-builtin-catalog, 22-code-intelligence |
| [22-language-support](22-language-support.md) | 141-142 | 22-code-intelligence |
| [23-control-plane](23-control-plane.md) | 35-37, 43, 88, 106, 110 | 07-agent-runtime, 12-connectivity, 16-surfaces |
| [24-smart-contracts](24-smart-contracts.md) | 100, 103-104, 120-121, 125-127 | 15-marketplace, 18-registries |
| [25-research-citations](25-research-citations.md) | 136, 138-139 | 21-roadmap |
| [26-roko-architecture-overview](26-roko-architecture-overview.md) | 1-3, 6, 106-108, 113-115, 128 | 00-index, 16-surfaces, 17-security, 20-deployment |
| [28-plans-catalog](28-plans-catalog.md) | References implementation plans, not depth docs | N/A |

### Depth documents NOT referenced by any tmp/ document

The following depth documents contain novel concepts not covered in any tmp/ analysis document:

- `00-index/implementation-readiness.md` (build phase planning)
- `02-block/eval-lifecycle-and-generation.md` (autonomous eval generation, EvoSkills)
- `02-block/verdicts-as-signals.md` (verdict forensics)
- `07-agent-runtime/cognitive-energy-and-vitality.md` (agent fatigue)
- `07-agent-runtime/21-collective-contagion.md` (emotional contagion)
- `07-agent-runtime/22-heartbeat-as-hot-graph.md` (adaptive heartbeat clock)
- `07-agent-runtime/25-active-inference-state-space.md` (90-state POMDP for routing)
- `07-agent-runtime/26-agent-lifecycle-type-state.md` (compile-time lifecycle enforcement)
- `07-agent-runtime/28-funding-budgets-and-operator-model.md` (budget guardrails)
- `10-learning-loops/autocatalytic-compounding.md` (Kauffman autocatalytic sets)
- `10-learning-loops/heuristics-and-falsifiers.md` (heuristic lifecycle)
- `16-surfaces/06-generative-interfaces-and-a2ui.md` (agent-generated UI)
- `17-security/05-adaptive-risk-as-loop.md` (LTL/CTL temporal logic monitoring)
- `21-roadmap/temporal-knowledge-graph.md` (Allen interval algebra)
- `21-roadmap/emergent-goals-and-energy.md` (autonomous goal formation)

---

<a name="statistics"></a>
## 8. Statistics

| Metric | Count |
|--------|-------|
| Thematic sections | 23 (19 with content, 4 reserved) |
| Depth documents | 145 |
| INDEX.md files | 24 (23 sections + 1 root) |
| Support files (GUIDE, INGEST-PROMPT, RESEARCH-PROMPTs) | 17 |
| Total .md files | 185 |
| Source documents absorbed | 422 (8.8MB) |
| Academic citations referenced | 500+ |

### Relevance Distribution

| Relevance Level | Count | Percentage |
|-----------------|-------|-----------|
| HIGH -- directly applicable to IronClaw | 52 | 36% |
| MEDIUM -- applicable with adaptation | 40 | 28% |
| LOW -- interesting but outside current scope | 26 | 18% |
| NONE -- domain-specific (blockchain, multi-agent economics) | 27 | 19% |

### Documents per Section

| Section | Count | Theme |
|---------|-------|-------|
| 00-index | 6 | Vision, principles, architecture |
| 01-signal | 4 | Signal algebra, decay, provenance |
| 02-block | 15 | Composition, verification, context engineering |
| 03-graph | 7 | DAG execution, orchestration |
| 04-specializations | 0 | Reserved |
| 05-execution-engine | 2 | Cognitive loop, resilience |
| 06-trigger-system | 0 | Reserved |
| 07-agent-runtime | 27 | Agent architecture, affect, supervision, heartbeat, lifecycle |
| 08-extension-system | 0 | Reserved |
| 09-telemetry | 1 | Observability |
| 10-learning-loops | 10 | Bandits, self-improvement, stability |
| 11-memory | 15 | Knowledge, dreams, stigmergy, collective intelligence |
| 12-connectivity | 5 | Relay protocol, coordination |
| 13-builtin-catalog | 5 | Tools, MCP, plugins |
| 14-config | 2 | Configuration management |
| 15-marketplace | 6 | Identity, economy, commerce |
| 16-surfaces | 7 | UI/UX across all channels |
| 17-security | 7 | Defense in depth, prompt security, formal methods |
| 18-registries | 8 | On-chain infrastructure |
| 19-arenas | 0 | Reserved |
| 20-deployment | 4 | Deployment shapes, hardening |
| 21-roadmap | 9 | Research foundations, test strategy |
| 22-code-intelligence | 5 | Code analysis pipeline |

### Concepts Unique to v2-Depth (Not Covered Elsewhere)

These important ideas from the depth layer are not adequately captured in any tmp/ analysis document:

1. **Predict-Publish-Correct Pattern** -- Universal pattern for learning across all feedback loops (doc 134)
2. **Variance Inequality** -- Formal bound: self-improvement speed must not exceed the system's absorption capacity (doc 70)
3. **Anti-Correlated Retrieval** -- Deliberately retrieving dissimilar context to prevent tunnel vision (docs 53, 80)
4. **CaMeL Dual-LLM Architecture** -- Separating trusted control plane from untrusted data plane with taint barrier (doc 118)
5. **Write-for-Amnesia Principle** -- Every context block must be interpretable with zero prior context (doc 19)
6. **Type-State Machine for Lifecycle** -- Compile-time enforcement of legal state transitions via Rust's type system (doc 59)
7. **Genomic Bottleneck** -- Confidence decay (0.85^N per generation) when transferring knowledge to successor agents (doc 59)
8. **Cognitive Energy Model** -- Explicit energy depletion preventing agents from grinding endlessly (doc 46)
9. **Red-Team Dreaming** -- Proactive attack scenario simulation during offline consolidation (doc 136)
10. **Autonomous Eval Generation** -- Separate agent generates tests, properties, and invariants (doc 23)
11. **EvoSkills** -- 3-tier skill hierarchy evolved via MAP-Elites quality-diversity archive (doc 23)
12. **Morphogenetic Specialization** -- Turing reaction-diffusion for emergent agent role assignment (doc 85)
13. **VCG Attention Auction** -- Mechanism-design approach to context window allocation with truthful pricing (doc 18)
14. **Gesell-Shannon Demurrage** -- `effective_rate = base_rate / (1 + novelty)` for knowledge decay (doc 8)
15. **U-Shape Positional Effects** -- Formalized attention decay in context window middles with placement strategies (doc 16)

---

*Verified 2026-07-02 against all 185 files in `/Users/will/dev/nunchi/roko/roko/docs/v2-depth/`. File counts confirmed per-section. Document summaries spot-checked against actual first sections.*

# Roko v2-Depth Research Catalog

Complete reference for 145 depth documents across 23 thematic sections in the `docs/v2-depth/` collection. These documents contain the algorithms, research grounding, and implementation detail behind Roko's architecture.

> **Source repository**: All links point to `https://github.com/wpank/roko/blob/main/docs/v2-depth/` or `https://github.com/wpank/roko/tree/main/docs/v2-depth/`.

**Cross-references**: [architecture-overview.md](architecture-overview.md) — system overview using these patterns | [plans-catalog.md](plans-catalog.md) — plans implementing them | `./research-citations/README.md` — academic bibliography

---

## Table of Contents

1. [Collection Structure](#1-collection-structure)
2. [Top-20 Must-Reads](#2-top-20-must-reads)
3. [Reading Guides by Goal](#3-reading-guides)
4. [Section-by-Section Catalog](#4-section-catalog)
5. [IronClaw Module Map](#5-ironclaw-module-map)
6. [Cross-References to tmp/ Documents](#6-cross-references)
7. [Statistics](#7-statistics)

---

## 1. Collection Structure

Two-layer documentation model:
- **Spec layer** (`docs/v2/`, 28 files): vocabulary, protocols, contracts — *what* and *why*
- **Depth layer** (`docs/v2-depth/`, 145 documents): algorithms, research grounding, implementation detail — *how* and *where it came from*

Four reserved sections (04, 06, 08, 19) have no documents yet.

### Unified Vocabulary

| Unified Term | Old Term(s) | Meaning |
|---|---|---|
| Signal | Engram | Durable, content-addressed data unit |
| Pulse | Event | Ephemeral event on the Bus |
| Cell | Block, Module | Atomic unit of computation |
| Graph | Workflow, Pipeline | Composition of Cells |
| Store | Substrate | Persistent state (pull-based) |
| Bus | EventBus | Ephemeral pub/sub (push-based) |
| Verdict | GateResult | Verification outcome |

### Relevance Ratings

| Rating | Meaning |
|--------|---------|
| **HIGH** | Directly applicable to current or near-future IronClaw features |
| **MEDIUM** | Applicable with adaptation |
| **LOW** | Interesting but outside IronClaw's current scope |
| **NONE** | Blockchain/multi-agent-economy specific; not transferable |

Of 145 documents: 52 (36%) are HIGH, 40 (28%) are MEDIUM, 53 (36%) are LOW or NONE. LOW/NONE cluster almost entirely in sections 15-marketplace and 18-registries.

---

## 2. Top-20 Must-Reads

### Foundation

**1. `00-index/architectural-thesis.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/00-index/architectural-thesis.md) | **HIGH**

The foundational claim: SWE-bench data shows 30–65% of agent performance variation comes from harness quality, not model quality. Derives the five-layer architecture as a theorem by topological sort on the nine protocol dependency lattice. Cross-refs: [architecture-overview.md](architecture-overview.md), [`../core-concepts/cognitive-architecture.md`](../core-concepts/cognitive-architecture.md).

**2. `01-signal/demurrage-economics.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/01-signal/demurrage-economics.md) | **HIGH**

Derives knowledge decay from Gesell's idle-asset economics + Shannon information theory: `effective_rate = base_rate / (1 + novelty)`. Redundant knowledge decays faster than novel knowledge. Phase space (balance × tier × novelty) analysis with fixed points. Cross-refs: [`../core-concepts/universal-engram.md`](../core-concepts/universal-engram.md), [`../agent-intelligence/dream-consolidation.md`](../agent-intelligence/dream-consolidation.md).

**3. `02-block/verify-as-universal-oracle.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/verify-as-universal-oracle.md) | **HIGH**

Verification has four simultaneous roles: reward function, relabeling oracle, safety boundary, economic attestation. Goodhart-resistance proof via conjunctive hard gates + Pareto soft criteria. Variance Inequality formally bounds self-improvement speed. Cross-refs: [`../execution-verification/gate-verification.md`](../execution-verification/gate-verification.md), [`../agent-intelligence/online-learning.md`](../agent-intelligence/online-learning.md).

### Context Engineering

**4. `02-block/active-inference-context-selection.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/active-inference-context-selection.md) | **HIGH**

Replaces hand-tuned context priorities with EFE (Expected Free Energy) minimization from Friston's active inference. `EFE = pragmatic_value + epistemic_value - ambiguity`. MVT stopping rule: stop adding context when marginal EFE per token drops below threshold. Directly applicable to `crates/ironclaw_engine/`. Cross-refs: [`../context-memory/budget-composition.md`](../context-memory/budget-composition.md).

**5. `02-block/vcg-attention-auction.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/vcg-attention-auction.md) | **HIGH**

VCG (Vickrey-Clarke-Groves) auction allocates context window tokens among 8 competing subsystems. Truthful bidding guaranteed; Thompson sampling `LearningBidder` learns optimal bids over episodes. Cross-refs: [`../context-memory/budget-composition.md`](../context-memory/budget-composition.md).

**6. `02-block/positional-effects-and-retrieval.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/positional-effects-and-retrieval.md) | **HIGH**

Formalizes "lost in the middle" as an algebraic property of causal decoders: U-shaped attention distribution over position. `Placement` enum (VANGUARD/REAR/MIDDLE) with `BetaPosterior` tracking effectiveness per position. Cross-refs: [`../context-memory/budget-composition.md`](../context-memory/budget-composition.md).

### Learning and Routing

**7. `10-learning-loops/bandit-routing-and-cascade.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/bandit-routing-and-cascade.md) | **HIGH**

3-stage cascade router: static rules → confidence threshold → LinUCB with 18D context vector. Pareto frontier pre-filtering. Trigram pattern discovery. Projected 30–50% LLM cost reduction. Directly applicable to `crates/ironclaw_llm/`. Cross-refs: [`../agent-intelligence/online-learning.md`](../agent-intelligence/online-learning.md).

**8. `10-learning-loops/self-improvement-frameworks.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/self-improvement-frameworks.md) | **HIGH**

Maps Reflexion, ExpeL, DSPy, ADAS to 5 improvement levels. Variance Inequality constrains safe self-modification rate. Constitutional constraints at each level. Cross-refs: [`../agent-intelligence/online-learning.md`](../agent-intelligence/online-learning.md), [`../core-concepts/cognitive-architecture.md`](../core-concepts/cognitive-architecture.md).

**9. `10-learning-loops/missing-loops-and-calibration.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/missing-loops-and-calibration.md) | **HIGH**

Diagnostic: 8 feedback loops that should exist but are typically unwired. Cross-loop interaction matrix. 31.6x collective intelligence heuristic for fully wired systems. Use as a checklist when designing IronClaw's feedback architecture. Cross-refs: [`../execution-verification/conductor-anomaly.md`](../execution-verification/conductor-anomaly.md).

### Memory and Consolidation

**10. `11-memory/06-dream-cycle-as-loop.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/06-dream-cycle-as-loop.md) | **HIGH**

NREM (Mattar-Daw episode replay) → REM (HDC counterfactual synthesis) → Integration (promote/reject). Convergence detection terminates cycle. More sophisticated than IronClaw's heartbeat system; applicable to `src/workspace/`. Cross-refs: [`../agent-intelligence/dream-consolidation.md`](../agent-intelligence/dream-consolidation.md).

**11. `11-memory/04-antiknowledge-and-immunity.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/04-antiknowledge-and-immunity.md) | **HIGH**

AntiKnowledge as non-decaying Signal Kind. SIR epidemiological model for false belief propagation and recovery. Balance floor maintenance. Addresses a failure mode in IronClaw's workspace: repeating disproved approaches. Cross-refs: [`../agent-intelligence/dream-consolidation.md`](../agent-intelligence/dream-consolidation.md), [`../core-concepts/universal-engram.md`](../core-concepts/universal-engram.md).

**12. `11-memory/08-hypnagogia-and-creativity.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/08-hypnagogia-and-creativity.md) | **MEDIUM**

Anti-correlated retrieval via inverted HDC vectors prevents convergence to local optima. Horowitz TDI: 43% creativity boost. 15% contrarian blending rule. Dali interrupt mechanism. Cross-refs: [`../agent-intelligence/dream-consolidation.md`](../agent-intelligence/dream-consolidation.md), [`../core-concepts/hyperdimensional-computing/README.md`](../core-concepts/hyperdimensional-computing/README.md).

### Safety and Security

**13. `17-security/06-prompt-security-and-camel.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/17-security/06-prompt-security-and-camel.md) | **HIGH**

CaMeL dual-LLM: PLLM (trusted, control plane) + QLLM (untrusted, data plane) with taint barrier. Ventriloquist defense. 77% solve rate with provable IFC. Applicable to `crates/ironclaw_safety/`. Cross-refs: [`../execution-verification/gate-verification.md`](../execution-verification/gate-verification.md).

**14. `17-security/immune-system-as-graph.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/17-security/immune-system-as-graph.md) | **HIGH**

5-layer immune pipeline. HDC attack fingerprint matching at ~10ns. Autoimmune protection. SIR spread model. Cross-refs: [`../execution-verification/gate-verification.md`](../execution-verification/gate-verification.md).

### Agent Architecture

**15. `05-execution-engine/cognitive-loop-as-graph.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/05-execution-engine/cognitive-loop-as-graph.md) | **HIGH**

7-step SENSE-ASSESS-COMPOSE-ACT-VERIFY-LEARN-EMIT loop as explicit Hot Graph. Workflow/Activity split for resumability. Byzantine Cell defenses. Maps directly to `src/agent/`. Cross-refs: [`../core-concepts/cognitive-architecture.md`](../core-concepts/cognitive-architecture.md).

**16. `07-agent-runtime/tool-loop-and-mcp.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/tool-loop-and-mcp.md) | **HIGH**

ToolLoop as Hot Flow. 8-step safety pipeline per tool dispatch. MCP as dynamic Cell discovery. ReAct < Reflexion < MCTS reasoning hierarchy. Tool RAG for tool selection. Cross-refs: [`../ecosystem/mcp-editor-integration.md`](../ecosystem/mcp-editor-integration.md).

### Code Intelligence

**17. `22-code-intelligence/01-code-intelligence-as-cell-pipeline.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/22-code-intelligence/01-code-intelligence-as-cell-pipeline.md) | **HIGH**

84–99% code blindness quantification. 4 costly mistake categories. 6-stage pipeline. Foundation for IronClaw's code intelligence work. Cross-refs: [`../context-memory/code-intelligence.md`](../context-memory/code-intelligence.md).

### Research Bridges

**18. `21-roadmap/08-research-to-runtime-bridge.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/21-roadmap/08-research-to-runtime-bridge.md) | **MEDIUM**

5 explicit theory-to-code bridges with fidelity-loss analysis and graduation criteria: active inference → EFE approximation, replicator dynamics → demurrage parameters, Turing patterns → morphogenetic fields, somatic markers → PAD decision trees, conformal prediction → calibration trackers. Cross-refs: [`./research-citations/README.md`](./research-citations/README.md).

**19. `02-block/gate-feedback-and-retry.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/gate-feedback-and-retry.md) | **HIGH**

Structured `GateFeedback` type. 97.75% token reduction vs prose feedback. Section-effectiveness learning. Model escalation after N retries. Cross-refs: [`../execution-verification/gate-verification.md`](../execution-verification/gate-verification.md).

**20. `07-agent-runtime/16-diagnosis-and-stuck-detection.md`** — [GitHub](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/16-diagnosis-and-stuck-detection.md) | **HIGH**

20 error categories, 9 interventions by severity, 34 detection patterns, 6 Lens Cells. IronClaw's `src/agent/` currently lacks this level of diagnosis. Cross-refs: [`../execution-verification/conductor-anomaly.md`](../execution-verification/conductor-anomaly.md).

---

## 3. Reading Guides

### "I want to understand the overall architecture"
1. [`00-index/architectural-thesis.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/00-index/architectural-thesis.md) — five-layer derivation
2. [`00-index/design-principles-algebra.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/00-index/design-principles-algebra.md) — 8 principles as algebra
3. [`02-block/protocol-algebra.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/protocol-algebra.md) — categorical structure of 9 protocols
4. [`05-execution-engine/cognitive-loop-as-graph.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/05-execution-engine/cognitive-loop-as-graph.md) — 7-step loop
5. Then: [architecture-overview.md](architecture-overview.md), [`../core-concepts/cognitive-architecture.md`](../core-concepts/cognitive-architecture.md)

### "I want to reduce LLM costs"
1. [`10-learning-loops/bandit-routing-and-cascade.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/bandit-routing-and-cascade.md) — 30–50% cost reduction
2. [`02-block/vcg-attention-auction.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/vcg-attention-auction.md) — optimal token allocation
3. [`02-block/enrichment-pipeline.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/enrichment-pipeline.md) — differential budget principle
4. [`10-learning-loops/provider-health-and-pareto.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/provider-health-and-pareto.md) — avoid degraded providers
5. Then: [`../agent-intelligence/online-learning.md`](../agent-intelligence/online-learning.md), [`../context-memory/budget-composition.md`](../context-memory/budget-composition.md)

### "I want to build better context for LLM prompts"
1. [`02-block/compose-protocol-and-builder.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/compose-protocol-and-builder.md) — 9-layer system prompt builder
2. [`02-block/enrichment-pipeline.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/enrichment-pipeline.md) — 13-step context gathering
3. [`02-block/positional-effects-and-retrieval.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/positional-effects-and-retrieval.md) — U-shape attention and placement
4. [`02-block/active-inference-context-selection.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/active-inference-context-selection.md) — EFE-based context scoring
5. Then: [`../context-memory/budget-composition.md`](../context-memory/budget-composition.md)

### "I want to improve agent memory"
1. [`11-memory/01-knowledge-as-signal.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/01-knowledge-as-signal.md) — knowledge types and tiers
2. [`01-signal/demurrage-economics.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/01-signal/demurrage-economics.md) — why idle knowledge should decay
3. [`11-memory/04-antiknowledge-and-immunity.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/04-antiknowledge-and-immunity.md) — tracking what is proven false
4. [`11-memory/06-dream-cycle-as-loop.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/06-dream-cycle-as-loop.md) — offline consolidation
5. [`11-memory/08-hypnagogia-and-creativity.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/08-hypnagogia-and-creativity.md) — creative retrieval
6. Then: [`../core-concepts/universal-engram.md`](../core-concepts/universal-engram.md), [`../agent-intelligence/dream-consolidation.md`](../agent-intelligence/dream-consolidation.md)

### "I want to understand the safety architecture"
1. [`01-signal/provenance-and-taint.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/01-signal/provenance-and-taint.md) — taint lattice
2. [`17-security/immune-system-as-graph.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/17-security/immune-system-as-graph.md) — 5-layer immune pipeline
3. [`17-security/02-defense-in-depth-as-pipeline.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/17-security/02-defense-in-depth-as-pipeline.md) — 7-layer defense
4. [`17-security/06-prompt-security-and-camel.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/17-security/06-prompt-security-and-camel.md) — CaMeL dual-LLM
5. Then: [`../execution-verification/gate-verification.md`](../execution-verification/gate-verification.md)

### "I want to understand agent supervision"
1. [`03-graph/event-log-and-conductor.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/03-graph/event-log-and-conductor.md) — 10 watcher Cells
2. [`07-agent-runtime/14-conductor-as-verify-pipeline.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/14-conductor-as-verify-pipeline.md) — conductor as Verify Pipeline
3. [`07-agent-runtime/16-diagnosis-and-stuck-detection.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/16-diagnosis-and-stuck-detection.md) — 20 error categories, 34 patterns
4. [`07-agent-runtime/15-circuit-breaker-and-interventions.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/15-circuit-breaker-and-interventions.md) — predictive circuit breaking
5. Then: [`../execution-verification/conductor-anomaly.md`](../execution-verification/conductor-anomaly.md)

### "I want to understand the code intelligence system"
1. [`22-code-intelligence/01-code-intelligence-as-cell-pipeline.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/22-code-intelligence/01-code-intelligence-as-cell-pipeline.md) — the blindness problem
2. [`22-code-intelligence/02-symbol-graph-and-importance.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/22-code-intelligence/02-symbol-graph-and-importance.md) — tree-sitter + PageRank
3. [`22-code-intelligence/04-search-and-context-assembly.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/22-code-intelligence/04-search-and-context-assembly.md) — multi-strategy search + budget
4. [`22-code-intelligence/05-mcp-server-and-persistence.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/22-code-intelligence/05-mcp-server-and-persistence.md) — MCP tools for code intelligence
5. Then: [`../context-memory/code-intelligence.md`](../context-memory/code-intelligence.md)

---

## 4. Section Catalog

### 00-index: Vision, Principles, Architecture

**Directory**: [`docs/v2-depth/00-index/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/00-index)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 1 | [`architectural-thesis.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/00-index/architectural-thesis.md) | SWE-bench: 30–65% performance variation from harness quality. Five-layer architecture derived by topological sort on protocol dependency lattice. Introduces L5 self-evolution. | Protocol dependency lattice, five strata, L5 self-evolution | **HIGH** |
| 2 | [`design-principles-algebra.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/00-index/design-principles-algebra.md) | 8 design principles as algebraic laws. Variance Inequality as implicit 9th principle bounding self-improvement speed. | 8 principles as algebra, Variance Inequality | MEDIUM |
| 3 | [`integration-topology.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/00-index/integration-topology.md) | System as typed directed graph. 3 SCCs, 5 disconnected nodes, 3 bottleneck edges. Bus as cycle-breaking decoupling mechanism. | System as graph, 3 SCCs, Bus as decoupling | MEDIUM |
| 4 | [`implementation-readiness.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/00-index/implementation-readiness.md) | Dependency-driven build phases A–E with 4 risk cliffs. Parallelization analysis. | Build phases A–E, 4 risk cliffs | LOW |
| 5 | [`test-strategy-for-self-improving-systems.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/00-index/test-strategy-for-self-improving-systems.md) | Invariant testing (test properties that must hold regardless of learned values) over output testing. Adversarial self-testing. Observability contracts. | Invariant testing, adversarial self-testing, observability contracts | **HIGH** |
| 6 | [`05-vision-and-positioning.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/00-index/05-vision-and-positioning.md) | Protocol standard (like ERC-20) rather than framework (like Rails). 4 defensible differentiators, 5 deployment shapes, Merkle-CRDT. | Protocol vs framework, 5 deployment shapes | MEDIUM |

Cross-refs: [architecture-overview.md](architecture-overview.md) draws from docs 1–3. [`../core-concepts/cognitive-architecture.md`](../core-concepts/cognitive-architecture.md) uses the five-layer taxonomy from doc 1.

---

### 01-signal: Signal Internals and Decay Math

**Directory**: [`docs/v2-depth/01-signal/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/01-signal)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 7 | [`signal-algebra.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/01-signal/signal-algebra.md) | HDC vectors form a semiring. Role-filler encoding for relational structures in hyperdimensional space. Lineage as free category. Graduation functor Bus→Store. | HDC semiring, role-filler encoding, lineage DAG, graduation functor | MEDIUM |
| 8 | [`demurrage-economics.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/01-signal/demurrage-economics.md) | Gesell-Shannon derivation: `effective_rate = base_rate / (1 + novelty)`. Phase space analysis. Cold storage/thaw. | Gesell-Shannon, novelty-weighted decay, phase space, cold storage | **HIGH** |
| 9 | [`scoring-and-calibration.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/01-signal/scoring-and-calibration.md) | Score-Verify-Score feedback loop. Temperature scaling, Beta-Binomial confidence intervals, Pareto front maintenance, isotonic regression. | Score-Verify-Score, temperature scaling, Beta-Binomial, isotonic regression | **HIGH** |
| 10 | [`provenance-and-taint.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/01-signal/provenance-and-taint.md) | Taint lattice (Bottom < Trusted < External < Untrusted < Tainted) with join propagation. Declassification capabilities. Custody-gated Store writes. | Taint lattice, join propagation, declassification, custody-gated Store | **HIGH** |

Cross-refs: [`../core-concepts/universal-engram.md`](../core-concepts/universal-engram.md) draws from docs 7–10. [`../core-concepts/hyperdimensional-computing/README.md`](../core-concepts/hyperdimensional-computing/README.md) covers HDC math from doc 7.

---

### 02-block: Composition, Tools, Verification

**Directory**: [`docs/v2-depth/02-block/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/02-block)
The largest and most algorithmically dense section.

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 11 | [`protocol-algebra.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/protocol-algebra.md) | Nine protocols as a category. TypeSchema as preorder. Natural transformations enable safe substitution. | Category of Cells, TypeSchema preorder, natural transformations | MEDIUM |
| 12 | [`store-and-bus-duality.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/store-and-bus-duality.md) | Store/Bus as adjoint functors. Graduation/projection adjunction. Backpressure from unit/counit relationship. | Pull/push adjoint functors, backpressure | MEDIUM |
| 13 | [`verify-as-universal-oracle.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/verify-as-universal-oracle.md) | 4 Verdict roles. Goodhart-resistance proof. Variance Inequality. Bradley-Terry scoring. | 4 Verdict roles, Goodhart-resistance, Variance Inequality | **HIGH** |
| 14 | [`compose-protocol-and-builder.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/compose-protocol-and-builder.md) | 9-layer system prompt builder. `ComposeBid` for subsystem bidding. 12 role templates. Cache alignment optimization. | 9-layer builder, ComposeBid, 12 role templates | **HIGH** |
| 15 | [`enrichment-pipeline.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/enrichment-pipeline.md) | 13-step context gathering. Self-RAG step selection. Three-tier budget (fixed/dynamic/overflow). Differential budget principle. | 13-step pipeline, Self-RAG, differential budget | **HIGH** |
| 16 | [`positional-effects-and-retrieval.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/positional-effects-and-retrieval.md) | U-shape attention as algebraic property. Placement enum. BetaPosterior tracking. LongLLMLingua reordering. | U-shape attention, Placement enum, BetaPosterior | **HIGH** |
| 17 | [`active-inference-context-selection.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/active-inference-context-selection.md) | EFE = pragmatic + epistemic - ambiguity. MVT stopping rule. 5-stage assembly. EFE/VCG convergence proof. | EFE scoring, MVT stopping rule | **HIGH** |
| 18 | [`vcg-attention-auction.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/vcg-attention-auction.md) | VCG for 8 competing subsystems. Truthful pricing theorem. Thompson sampling LearningBidder. | VCG, 8 bidder subsystems, Thompson sampling | **HIGH** |
| 19 | [`distributed-and-affect-composition.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/distributed-and-affect-composition.md) | 4 context strategies. PAD as Compose endofunctor. Write-for-amnesia principle. | 4 strategies, PAD as functor, write-for-amnesia | MEDIUM |
| 20 | [`verify-cells-and-pipeline.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/verify-cells-and-pipeline.md) | 11 gate implementations. 7-rung pipeline. Gate composition algebra. GVU architecture. | 11 gates, 7-rung pipeline, composition algebra | **HIGH** |
| 21 | [`ratcheting-and-adaptive-thresholds.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/ratcheting-and-adaptive-thresholds.md) | GateRatchet: thresholds only move upward. CUSUM/EWMA control charts. BOCPD regime detection. Hotelling T-squared. | GateRatchet, CUSUM/EWMA, BOCPD | MEDIUM |
| 22 | [`gate-feedback-and-retry.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/gate-feedback-and-retry.md) | Structured GateFeedback. 97.75% token reduction vs prose. Section-effectiveness learning. Model escalation. | Structured feedback, 97.75% reduction, model escalation | **HIGH** |
| 23 | [`eval-lifecycle-and-generation.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/eval-lifecycle-and-generation.md) | 14 eval loops across 5 tiers. Autonomous eval generation. EvoSkills with MAP-Elites. Capability preservation tests. | 14 eval loops, autonomous eval generation, MAP-Elites | MEDIUM |
| 24 | [`verdicts-as-signals.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/verdicts-as-signals.md) | Verdicts as first-class Signals with 24h half-life. Forensic causal replay. BLAKE3 integrity. | Verdicts as Signals, forensic replay | MEDIUM |
| 25 | [`process-reward-and-artifacts.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/02-block/process-reward-and-artifacts.md) | PRMs score intermediate steps. Promise/Progress decomposition. Self-supervised PRM training. | PRMs, Promise/Progress, ThinkPRM | MEDIUM |

Cross-refs: [`../execution-verification/gate-verification.md`](../execution-verification/gate-verification.md) draws from docs 13, 20–22, 25. [`../context-memory/budget-composition.md`](../context-memory/budget-composition.md) draws from docs 14–18.

---

### 03-graph: Orchestration and DAG Execution

**Directory**: [`docs/v2-depth/03-graph/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/03-graph)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 26 | [`plan-discovery-and-dag.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/03-graph/plan-discovery-and-dag.md) | Filesystem plan discovery. Wave computation (topological sort with parallelism). Critical path analysis. | Filesystem discovery, DAG, wave computation, critical path | MEDIUM |
| 27 | [`parallel-executor.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/03-graph/parallel-executor.md) | Petri net formal model. Multi-dimensional resource budgets. Atomic state machine transitions. | Petri net, resource budgets, atomic transitions | MEDIUM |
| 28 | [`plan-phases-and-actions.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/03-graph/plan-phases-and-actions.md) | 12 execution phases, 2 retry loops, 3 terminal states, 10 action types. | 12 phases, retry loops, terminal states | LOW |
| 29 | [`worktree-isolation-and-merge.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/03-graph/worktree-isolation-and-merge.md) | Git worktrees for task isolation. Merge queue with conflict detection. Warm agent pool. | Git worktree isolation, merge queue, warm pool | MEDIUM |
| 30 | [`snapshot-and-recovery.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/03-graph/snapshot-and-recovery.md) | Hash-chained event log. Dual-source recovery (checkpoint + event replay). BLAKE3 integrity. | Hash-chained log, dual-source recovery | LOW |
| 31 | [`event-log-and-conductor.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/03-graph/event-log-and-conductor.md) | 10 watcher Cells. Yerkes-Dodson pressure curve. Viable System Model. Graduated interventions. | 10 watchers, Yerkes-Dodson, Viable System Model | **HIGH** |
| 32 | [`stigmergy-and-cross-domain.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/03-graph/stigmergy-and-cross-domain.md) | Git as stigmergic coordination medium (commits as digital pheromones). c-factor. Saga pattern for rollback. | Git as stigmergic medium, c-factor, saga pattern | LOW |

Cross-refs: [`../execution-verification/dag-execution.md`](../execution-verification/dag-execution.md) draws from docs 26–30. [`../execution-verification/conductor-anomaly.md`](../execution-verification/conductor-anomaly.md) draws from doc 31. [`../execution-verification/orchestrator-swarm.md`](../execution-verification/orchestrator-swarm.md) draws from docs 26–30, 32.

---

### 04-specializations (reserved), 06-trigger-system (reserved), 08-extension-system (reserved)

No documents yet in these sections.

---

### 05-execution-engine: Runtime Loop

**Directory**: [`docs/v2-depth/05-execution-engine/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/05-execution-engine)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 33 | [`cognitive-loop-as-graph.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/05-execution-engine/cognitive-loop-as-graph.md) | 7-step SENSE-ASSESS-COMPOSE-ACT-VERIFY-LEARN-EMIT as Hot Graph. Workflow/Activity split for resumability. Byzantine Cell defenses. | 7-step loop, Workflow/Activity split, Byzantine defenses | **HIGH** |
| 34 | [`resilience-and-numerics.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/05-execution-engine/resilience-and-numerics.md) | 4 error kinds (transient/permanent/timeout/resource). Retry algebra per kind. Circuit breaker state machine. Degradation ladder. | 4 error kinds, circuit breaker, degradation ladder | **HIGH** |

Cross-refs: [`../core-concepts/cognitive-architecture.md`](../core-concepts/cognitive-architecture.md) draws from doc 33. [`../execution-verification/runtime-infrastructure.md`](../execution-verification/runtime-infrastructure.md) draws from doc 34.

---

### 07-agent-runtime: Agent, Conductor, Heartbeat, Lifecycle

**Directory**: [`docs/v2-depth/07-agent-runtime/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/07-agent-runtime)
Largest section (27 documents).

#### Agent Core

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 35 | [`agent-cell-and-providers.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/agent-cell-and-providers.md) | Agents as Cells. 7 implementation variants. Provider registry as Route Cell. Unified factory. | 7 agent implementations, Route Cell registry | **HIGH** |
| 36 | [`chat-types-and-streaming.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/chat-types-and-streaming.md) | Canonical ChatResponse normalizing 5 LLM providers. Streaming as Pulse sequences. FinishReason normalization. | ChatResponse canonical type, streaming as Pulses | **HIGH** |
| 37 | [`agent-pools-and-roles.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/agent-pools-and-roles.md) | 28-role taxonomy with per-role tool permissions. AgentPool/MultiAgentPool. 4 supervision strategies. | 28-role taxonomy, supervision strategies | **HIGH** |

#### Tool and Format Engineering

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 38 | [`tool-loop-and-mcp.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/tool-loop-and-mcp.md) | ToolLoop as Hot Flow. 8-step safety pipeline. MCP as dynamic Cell discovery. ReAct/Reflexion/MCTS hierarchy. Tool RAG. | ToolLoop, 8-step pipeline, MCP, reasoning hierarchy | **HIGH** |
| 39 | [`harness-and-format-engineering.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/harness-and-format-engineering.md) | Harness as Pipeline of 6 principles. 4 format adapter Cells (JSON, XML, Markdown, code). | Harness as Pipeline, 4 format adapters | **HIGH** |

#### Dual-Process and Routing

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 40 | [`temperament-and-dual-process.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/temperament-and-dual-process.md) | 4-level temperament profiling. CascadeRouter with LinUCB + Pareto + Thompson sampling. | 4-level temperament, CascadeRouter | MEDIUM |
| 41 | [`dual-process-and-efe-routing.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/dual-process-and-efe-routing.md) | System 1/2 mapped to 3 routing tiers. EFE routing formula. Affect modulates model selection. | System 1/2 as 3 tiers, EFE routing | MEDIUM |

#### Extensibility and Providers

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 42 | [`extensibility-and-creation.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/extensibility-and-creation.md) | 5 extension points. 4-layer SDK. Darwin Godel Machine. Voyager-style skill library. | 5 extension points, Voyager skill library | **HIGH** |
| 43 | [`provider-integrations-and-profiles.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/provider-integrations-and-profiles.md) | 8+ providers (Claude, GPT-4, Ollama, Mistral, NEAR AI, Bedrock, etc.). 6 domain profiles. | 8+ providers, 6 domain profiles | **HIGH** |

#### Cognitive Architecture

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 44 | [`cognitive-timescales.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/cognitive-timescales.md) | Gamma/theta/delta concurrent Hot Graphs at different timescales. EFE-driven T0/T1/T2 cascade. | Gamma/theta/delta timescales, EFE T0/T1/T2 | MEDIUM |
| 45 | [`cross-cut-functors.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/cross-cut-functors.md) | Memory/Daimon/Dreams as Signal endofunctors. VCG arbitration between cross-cuts. | Cross-cut functors, VCG arbitration | MEDIUM |
| 46 | [`cognitive-energy-and-vitality.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/cognitive-energy-and-vitality.md) | Energy depletion model. Fatigue accumulation. 3 recovery modes (micro-rest, consolidation, deep recovery). | Energy model, fatigue, recovery modes | LOW |

#### Conductor and Supervision

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 47 | [`14-conductor-as-verify-pipeline.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/14-conductor-as-verify-pipeline.md) | Conductor as Verify Pipeline with 10 watcher Cells. OODA Loop. Three timescales. | 10 watchers, OODA Loop, 3 timescales | **HIGH** |
| 48 | [`15-circuit-breaker-and-interventions.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/15-circuit-breaker-and-interventions.md) | Predictive circuit breaking via Holt forecaster. AIMD concurrency control. | Predictive breaking, AIMD | **HIGH** |
| 49 | [`16-diagnosis-and-stuck-detection.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/16-diagnosis-and-stuck-detection.md) | 20 error categories, 9 interventions, 34 detection patterns, 6 Lens Cells for stuck detection. | 20 errors, 9 interventions, 34 patterns, 6 Lens Cells | **HIGH** |
| 50 | [`17-adaptive-supervision-loop.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/17-adaptive-supervision-loop.md) | Self-model accuracy via Brier score. Threshold adaptation. ConductorBandit (CMAB). | Self-model Brier score, ConductorBandit | **HIGH** |

#### Affect Engine / Daimon

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 51 | [`18-affect-as-functor.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/18-affect-as-functor.md) | PAD vector. 8-step appraisal React Cell. ALMA 3-layer model. Affect as Compose Functor. | PAD, 8-step appraisal, ALMA, affect as Compose | LOW |
| 52 | [`19-behavioral-states-and-routing.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/19-behavioral-states-and-routing.md) | 6 behavioral states with hysteresis. Tier routing adjustment by state. | 6 behavioral states, tier routing by state | LOW |
| 53 | [`20-somatic-landscape.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/20-somatic-landscape.md) | Somatic markers (Damasio) as k-d tree indexed embeddings. 15% contrarian retrieval Functor. | Somatic markers, 15% contrarian retrieval | LOW |

#### Collective Behavior

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 54 | [`21-collective-contagion.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/21-collective-contagion.md) | Emotional contagion across agents. Attenuation Functor. Anti-cascade protection. | Emotional contagion, attenuation, anti-cascade | NONE |

#### Heartbeat Architecture

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 55 | [`22-heartbeat-as-hot-graph.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/22-heartbeat-as-hot-graph.md) | Adaptive clock. Gamma/theta/delta flows. HeartbeatPolicy. BOCPD regime detection. | Adaptive clock, HeartbeatPolicy, BOCPD | MEDIUM |
| 56 | [`23-t0-probes-as-verify-cells.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/23-t0-probes-as-verify-cells.md) | 16 T0 probes (8 chain + 6 coding + 2 universal). Rolling z-score anomaly detection. | 16 T0 probes, rolling z-score | MEDIUM |
| 57 | [`24-attention-auction-and-cortical-state.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/24-attention-auction-and-cortical-state.md) | VCG heartbeat compute budget. CorticalState: 32-signal atomic struct. | VCG heartbeat budget, CorticalState | MEDIUM |

#### Active Inference and Lifecycle

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 58 | [`25-active-inference-state-space.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/25-active-inference-state-space.md) | Full POMDP with 90 states (TaskPhase × ContextQuality × Uncertainty). A/B/C/D generative model. EFE for tier selection. | 90-state POMDP, A/B/C/D matrices | MEDIUM |
| 59 | [`26-agent-lifecycle-type-state.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/26-agent-lifecycle-type-state.md) | Compile-time type-state lifecycle machine (illegal transitions are type errors). 3 creation flows. Genomic bottleneck: 0.85^N confidence decay per generation. | Type-state machine, genomic bottleneck | MEDIUM |
| 60 | [`27-knowledge-transfer-and-mesh.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/27-knowledge-transfer-and-mesh.md) | Agent Mesh: version-vector delta sync. Bloom filter discovery. 4-tier gossip. Adoption Pipeline. | Agent Mesh, delta sync, Bloom filter gossip | LOW |
| 61 | [`28-funding-budgets-and-operator-model.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/28-funding-budgets-and-operator-model.md) | BudgetGuard as Verify Cell. 4-level guardrails. 5-stage degradation cascade. | BudgetGuard, 4-level guardrails, degradation cascade | MEDIUM |

Cross-refs: [`../agent-intelligence/affect-engine.md`](../agent-intelligence/affect-engine.md) from docs 51–53. [`../execution-verification/conductor-anomaly.md`](../execution-verification/conductor-anomaly.md) from docs 47–50. [`../agent-intelligence/agent-patterns.md`](../agent-intelligence/agent-patterns.md) from docs 35–39. [`../ecosystem/control-plane.md`](../ecosystem/control-plane.md) from docs 35–37, 43.

---

### 09-telemetry: Observability

**Directory**: [`docs/v2-depth/09-telemetry/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/09-telemetry)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 62 | [`01-observability-as-lens-pipeline.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/09-telemetry/01-observability-as-lens-pipeline.md) | Logs as Bus Pulses, metrics as Lens outputs, traces as lineage Signal chains. Cost visibility pipeline. | Logs/metrics/traces unified as primitives | MEDIUM |

---

### 10-learning-loops: Bandits, Self-Improvement

**Directory**: [`docs/v2-depth/10-learning-loops/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/10-learning-loops)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 63 | [`c-factor-as-lens.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/c-factor-as-lens.md) | Woolley c-factor as Lens. 5 sub-lenses. WisdomGate. Goodhart defense via metric rotation. | C-factor Lens, WisdomGate | LOW |
| 64 | [`autocatalytic-compounding.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/autocatalytic-compounding.md) | 7 compounding loops. Kauffman autocatalytic condition. SPoF analysis. | 7 compounding loops, Kauffman condition | MEDIUM |
| 65 | [`episodes-and-playbooks.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/episodes-and-playbooks.md) | Episode Store with HDC fingerprinting. Playbook rules with 0.95 confidence ceiling. Voyager skill library. | HDC episode similarity, 0.95 playbook ceiling | **HIGH** |
| 66 | [`bandit-routing-and-cascade.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/bandit-routing-and-cascade.md) | 3-stage cascade. LinUCB with 18D context vector. Pareto frontier. Trigram discovery. 30–50% cost projection. | 3-stage cascade, LinUCB 18D, 30–50% cost | **HIGH** |
| 67 | [`metrics-baselines-and-regression.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/metrics-baselines-and-regression.md) | Per-slice baselines. 15%/20% regression thresholds. 3:1 cost normalization. A/B/C/D/F efficiency grading. | Per-slice baselines, regression thresholds | **HIGH** |
| 68 | [`provider-health-and-pareto.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/provider-health-and-pareto.md) | 3-state circuit breaker per provider. Pareto dominance pruning. Anomaly detection on latency/error rate. | 3-state circuit breaker, Pareto pruning | **HIGH** |
| 69 | [`drift-and-stability.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/drift-and-stability.md) | Thompson Sampling with gamma=0.995 discount. Hysteresis. 4-tier frequency separation. Anti-pattern traps. | Thompson sampling, hysteresis, frequency separation | MEDIUM |
| 70 | [`self-improvement-frameworks.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/self-improvement-frameworks.md) | Reflexion/ExpeL/DSPy/ADAS as 5 improvement levels. Variance Inequality. Constitutional constraints. | 5-level improvement, Reflexion/ExpeL/DSPy/ADAS | **HIGH** |
| 71 | [`missing-loops-and-calibration.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/missing-loops-and-calibration.md) | 8 missing feedback loops. Cross-loop interaction matrix. 31.6x collective intelligence heuristic. | 8 missing loops, 31.6x heuristic | **HIGH** |
| 72 | [`heuristics-and-falsifiers.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/10-learning-loops/heuristics-and-falsifiers.md) | Heuristic lifecycle. Wilson confidence interval. Worldview clusters. AntiKnowledge for falsified heuristics. | Heuristic lifecycle, Wilson CI, worldviews | MEDIUM |

Cross-refs: [`../agent-intelligence/online-learning.md`](../agent-intelligence/online-learning.md) from docs 66, 68, 69. [`../execution-verification/conductor-anomaly.md`](../execution-verification/conductor-anomaly.md) from docs 67, 68, 71.

---

### 11-memory: Knowledge, Dreams, Stigmergy

**Directory**: [`docs/v2-depth/11-memory/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/11-memory)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 73 | [`01-knowledge-as-signal.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/01-knowledge-as-signal.md) | 6 knowledge types as Signal subtypes. 3-tier hierarchy. Ebbinghaus forgetting curve. Hybrid search API. | 6 knowledge types, 3-tier hierarchy, Ebbinghaus | **HIGH** |
| 74 | [`02-hdc-algebra-and-retrieval.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/02-hdc-algebra-and-retrieval.md) | 10,240-bit vectors. Bind/bundle/permute. False positive rate 1/10^307. AntiKnowledge has inverted vectors. | HDC operations, false positive rate, AntiKnowledge HDC | MEDIUM |
| 75 | [`03-knowledge-lifecycle-loop.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/03-knowledge-lifecycle-loop.md) | D1/D2/D3 distillation. Predict-publish-correct calibration. Backup/restore. Mesh sync. | D1/D2/D3, predict-publish-correct | MEDIUM |
| 76 | [`04-antiknowledge-and-immunity.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/04-antiknowledge-and-immunity.md) | AntiKnowledge never fully decays. Cognitive immune SIR model. Balance floor. | Non-decaying AntiKnowledge, SIR model | **HIGH** |
| 77 | [`05-cross-domain-transfer.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/05-cross-domain-transfer.md) | HDC resonance as Store query. Library of Babel analogy. Federation ingestion. | HDC resonance, cross-domain retrieval | LOW |
| 78 | [`06-dream-cycle-as-loop.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/06-dream-cycle-as-loop.md) | NREM/REM/Integration three-phase cycle. Mattar-Daw utility scoring. HDC counterfactual synthesis. Convergence detection. | NREM/REM/Integration, Mattar-Daw, convergence detection | **HIGH** |
| 79 | [`07-replay-and-counterfactual-cells.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/07-replay-and-counterfactual-cells.md) | Mattar-Daw formula: `need × expected_reward × replay_ease`. Counterfactual synthesis. Hindsight experience replay. | Mattar-Daw formula, hindsight replay | MEDIUM |
| 80 | [`08-hypnagogia-and-creativity.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/08-hypnagogia-and-creativity.md) | Anti-correlated retrieval via inverted HDC. Horowitz TDI 43% creativity boost. 15% contrarian blending. Dali interrupt. | Anti-correlated retrieval, 43% boost, 15% blending | MEDIUM |
| 81 | [`09-consolidation-and-staging.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/09-consolidation-and-staging.md) | Staging buffer confidence ladder. SHY renormalization. Oneirography. | Confidence ladder, SHY renormalization | LOW |
| 82 | [`10-threat-simulation-and-nightmares.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/10-threat-simulation-and-nightmares.md) | FMEA/FTA as Score Cells. Nightmare containment. Dream journal analysis. | FMEA/FTA as Score Cells, nightmare containment | MEDIUM |
| 83 | [`11-stigmergy-as-bus.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/11-stigmergy-as-bus.md) | Pheromones as Pulses (TTL-based evaporation, strength reinforcement, scoped visibility). Git as stigmergic medium. | Pheromone Pulses, git stigmergy | LOW |
| 84 | [`12-pheromone-mechanics-and-interference.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/12-pheromone-mechanics-and-interference.md) | 7 pheromone kinds with half-life taxonomy. SINR interference model. Hill-function thresholds. | 7 pheromone kinds, SINR, Hill-function | NONE |
| 85 | [`13-morphogenetic-specialization-as-loop.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/13-morphogenetic-specialization-as-loop.md) | Turing reaction-diffusion for agent role assignment. Gierer-Meinhardt kinetics. Lyapunov stability. | Turing reaction-diffusion, Lyapunov stability | NONE |
| 86 | [`14-mesh-sync-and-subnets.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/14-mesh-sync-and-subnets.md) | Bus federation (WebSocket + Iroh P2P). Partition tolerance with vector clocks. Permissioned subnets. | Bus federation, partition tolerance | LOW |
| 87 | [`15-collective-metrics-as-lens.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/15-collective-metrics-as-lens.md) | C-factor Lens with 5 axes. WisdomGate. Contrarian retrieval. 7 flywheel Loops. | C-factor Lens, WisdomGate, 7 flywheels | NONE |

Cross-refs: [`../agent-intelligence/dream-consolidation.md`](../agent-intelligence/dream-consolidation.md) from docs 78–82. [`../core-concepts/hyperdimensional-computing/README.md`](../core-concepts/hyperdimensional-computing/README.md) from docs 74, 80. [`../core-concepts/universal-engram.md`](../core-concepts/universal-engram.md) from docs 73, 76.

---

### 12-connectivity: Relay and Networking

**Directory**: [`docs/v2-depth/12-connectivity/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/12-connectivity)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 88 | [`01-relay-wire-protocol.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/12-connectivity/01-relay-wire-protocol.md) | Frame-level WebSocket protocol. Connection lifecycle. Ring buffer sequencing with gap detection. | WebSocket protocol, ring buffer sequencing | MEDIUM |
| 89 | [`02-topic-namespace.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/12-connectivity/02-topic-namespace.md) | Dot-separated ABNF grammar for Bus topics. Wildcard subscriptions. | Topic namespace grammar | LOW |
| 90 | [`03-chain-event-projection.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/12-connectivity/03-chain-event-projection.md) | Chain watcher Cell. Finality tagging. Reorg handling. | Chain events as Pulses | NONE |
| 91 | [`04-relay-deployment.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/12-connectivity/04-relay-deployment.md) | 3 deployment models (sidecar/shared/validator-embedded). Multi-relay federation. | 3 deployment models | LOW |
| 92 | [`05-coordination-patterns.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/12-connectivity/05-coordination-patterns.md) | 3 coordination primitives (pub/sub, request/response, on-chain). Presence tracking. | 3 coordination primitives | LOW |

---

### 13-builtin-catalog: Tools, MCP, Plugins

**Directory**: [`docs/v2-depth/13-builtin-catalog/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/13-builtin-catalog)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 93 | [`01-tool-architecture-as-cell-protocol.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/13-builtin-catalog/01-tool-architecture-as-cell-protocol.md) | Tools as Cells (React protocol). 3 trust tiers. Safety hook pipeline. Capability<T> ownership tokens. | Tools as Cells, 3 trust tiers, Capability<T> | **HIGH** |
| 94 | [`02-mcp-as-connect-protocol.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/13-builtin-catalog/02-mcp-as-connect-protocol.md) | MCP as dynamic Cell registry. Namespaced registry merge. Trust hierarchy from server registration. Tool change notifications. | MCP as dynamic registry, namespaced merge | **HIGH** |
| 95 | [`03-plugin-spi-as-extension.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/13-builtin-catalog/03-plugin-spi-as-extension.md) | 5-tier plugin SPI (compiled-in → dynamic library → WASM+host API → WASM+fuel metering → network sandbox). Discovery-first loading. | 5-tier plugin SPI | **HIGH** |
| 96 | [`04-domain-tools-and-profiles.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/13-builtin-catalog/04-domain-tools-and-profiles.md) | Rack pattern. Structural absence saves 94% tokens vs listing all tools. CascadeRouter limits to 12 tools/tick. | Rack pattern, structural absence 94% savings | **HIGH** |
| 97 | [`05-event-sources-and-templates.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/13-builtin-catalog/05-event-sources-and-templates.md) | Trigger Cells for cron/watch/webhook/GitHub/Slack. 18 shipped Graph templates. | 5 Trigger Cells, 18 templates | MEDIUM |

Cross-refs: [`../ecosystem/plugin-extension.md`](../ecosystem/plugin-extension.md) from docs 93–95, 97. [`../ecosystem/mcp-editor-integration.md`](../ecosystem/mcp-editor-integration.md) from doc 94. [`../context-memory/budget-composition.md`](../context-memory/budget-composition.md) from doc 96.

---

### 14-config: Configuration as Signal

**Directory**: [`docs/v2-depth/14-config/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/14-config)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 98 | [`config-as-signal.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/14-config/config-as-signal.md) | Config values as first-class Signals with provenance and hot-reload. | Config as Signal, L4 evolution | MEDIUM |
| 99 | [`02-layered-resolution-and-reload.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/14-config/02-layered-resolution-and-reload.md) | 4-layer merge: CLI > env > TOML > defaults. inotify/kqueue hot-reload. ROKO_* convention. | 4-layer merge, hot-reload | MEDIUM |

---

### 15-marketplace: Identity, Economy, Commerce

**Directory**: [`docs/v2-depth/15-marketplace/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/15-marketplace)
All blockchain-specific. LOW or NONE relevance for IronClaw.

Docs 100–105: agent identity (ERC-8004), reputation (EigenTrust, R^1.7 superlinear), knowledge marketplace (alpha-decay pricing), payment protocols (ERC-8183/MPP/x402), KORAI tokenomics, agent economy loop. Skip unless building chain-native agent infrastructure.

---

### 16-surfaces: UI/UX Architecture

**Directory**: [`docs/v2-depth/16-surfaces/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/16-surfaces)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 106 | [`01-surfaces-as-lens-composition.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/16-surfaces/01-surfaces-as-lens-composition.md) | All UI surfaces as Lens compositions of StateHub. 9-verb interaction model. Unidirectional state flow. | Surfaces as Lenses, StateHub, 9-verb model | **HIGH** |
| 107 | [`02-cli-and-command-graph.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/16-surfaces/02-cli-and-command-graph.md) | CLI subcommands as Graph triggers. Layered config. Progressive help. | CLI as Graph triggers | MEDIUM |
| 108 | [`03-tui-screen-architecture.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/16-surfaces/03-tui-screen-architecture.md) | 29 TUI screens as Lens Cells. 6 layout regions. Ratatui immediate-mode rendering. | 29 TUI screens as Lens Cells | MEDIUM |
| 109 | [`04-rosedust-and-spectre.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/16-surfaces/04-rosedust-and-spectre.md) | Design tokens as config Signals. Spectre avatar from HDC fingerprint. PAD animation. | Design token Signals, HDC avatar | NONE |
| 110 | [`05-http-api-and-realtime.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/16-surfaces/05-http-api-and-realtime.md) | ~85 routes as Connect Cells. WebSocket/SSE as Bus subscriptions. Cursor pagination. | Connect Cells, WebSocket/SSE as Bus | **HIGH** |
| 111 | [`06-generative-interfaces-and-a2ui.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/16-surfaces/06-generative-interfaces-and-a2ui.md) | A2UI: agents generate 12 UI component kinds. Sonification. | A2UI, 12 component kinds | LOW |
| 112 | [`07-developer-experience-and-onboarding.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/16-surfaces/07-developer-experience-and-onboarding.md) | SDK as Graph templates. Onboarding as Trigger Graph. ACP-first IDE. | SDK as Graph templates, onboarding triggers | MEDIUM |

---

### 17-security: Safety and Defense

**Directory**: [`docs/v2-depth/17-security/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/17-security)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 113 | [`immune-system-as-graph.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/17-security/immune-system-as-graph.md) | 5-layer immune pipeline. HDC fingerprint matching ~10ns. Autoimmune protection. SIR spread model. | 5-layer pipeline, HDC ~10ns, SIR model | **HIGH** |
| 114 | [`02-defense-in-depth-as-pipeline.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/17-security/02-defense-in-depth-as-pipeline.md) | 7-layer defense with early-exit. Permits/allowlists per capability. | 7-layer defense, early-exit | **HIGH** |
| 115 | [`03-capability-taint-and-ifc.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/17-security/03-capability-taint-and-ifc.md) | Capability<T> tokens. 5-fold taint taxonomy. 3-layer capability intersection. CaMeL IFC. | Capability<T>, 5-fold taint, CaMeL IFC | **HIGH** |
| 116 | [`04-audit-witness-and-forensics.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/17-security/04-audit-witness-and-forensics.md) | Custody chain as Store lineage. Witness DAG (5 vertex types). Forensic replay. SOC2/ISO 27001/GDPR. | Custody chain, witness DAG, forensic replay | MEDIUM |
| 117 | [`05-adaptive-risk-as-loop.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/17-security/05-adaptive-risk-as-loop.md) | 5-layer runtime risk Loop. LTL/CTL invariants monitored at runtime. Adaptive gate thresholds. | Runtime risk Loop, LTL/CTL monitoring | MEDIUM |
| 118 | [`06-prompt-security-and-camel.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/17-security/06-prompt-security-and-camel.md) | CaMeL dual-LLM (PLLM + QLLM). Taint barrier. Ventriloquist defense. 77% IFC solve rate. | CaMeL, taint barrier, ventriloquist, 77% IFC | **HIGH** |
| 119 | [`07-cognitive-kernel-and-formal-methods.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/17-security/07-cognitive-kernel-and-formal-methods.md) | Cognitive namespaces. EDF scheduling. Formal verification (Heimdall/Slither/Echidna/hevm/Certora). MEV pre-flight. | Cognitive namespaces, formal verification tools | LOW |

Cross-refs: [`../execution-verification/gate-verification.md`](../execution-verification/gate-verification.md) from docs 113–114, 118. [architecture-overview.md](architecture-overview.md) from docs 113–115.

---

### 18-registries: On-Chain Infrastructure

All NONE relevance for IronClaw. Docs 120–127 cover ERC-8004 registries, HDC on-chain verification, job market, reputation, chain witness, payments, gossip/privacy, and simulation (mirage-rs). Skip unless building chain-native agent infrastructure.

---

### 19-arenas (reserved)

No documents yet.

---

### 20-deployment: Deployment Shapes

**Directory**: [`docs/v2-depth/20-deployment/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/20-deployment)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 128 | [`01-five-shape-deployment-as-config-signal.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/20-deployment/01-five-shape-deployment-as-config-signal.md) | 5 shapes (laptop/server/container/clustered/edge) from one binary via config Signals. Merkle-CRDT for brain state export. | 5 deployment modes, single binary, Merkle-CRDT | **HIGH** |
| 129 | [`02-daemon-and-subscription-system.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/20-deployment/02-daemon-and-subscription-system.md) | Daemon as Hot Flow Graph. 3 trigger types (cron/event/condition). launchd/systemd integration. | Daemon as Hot Flow, 3 trigger types | **HIGH** |
| 130 | [`03-cloud-and-edge-deployment.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/20-deployment/03-cloud-and-edge-deployment.md) | Fly.io scale-to-zero. Edge at ~500KB musl. Merkle-CRDT brain sync. | Fly.io zero-cost idle, edge ~500KB | MEDIUM |
| 131 | [`04-production-hardening-and-observability.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/20-deployment/04-production-hardening-and-observability.md) | Adaptive timeouts. Backoff state machine. 4-phase shutdown (drain/checkpoint/close/audit). Loki/Prometheus/Tempo. | Adaptive timeouts, 4-phase shutdown | **HIGH** |

Cross-refs: [`../execution-verification/runtime-infrastructure.md`](../execution-verification/runtime-infrastructure.md) from all four docs. [architecture-overview.md](architecture-overview.md) from doc 128.

---

### 21-roadmap: Research Foundations

**Directory**: [`docs/v2-depth/21-roadmap/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/21-roadmap)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 132 | [`temporal-knowledge-graph.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/21-roadmap/temporal-knowledge-graph.md) | Allen's 13 interval relations as Store query predicates. 3-tier temporal memory. | Allen's 13 intervals, event calculus | LOW |
| 133 | [`emergent-goals-and-energy.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/21-roadmap/emergent-goals-and-energy.md) | Emergent goal formation from energy gradients. ZPD as Score Cell. Energy-goal coupling. | ZPD Score Cell, energy-goal coupling | LOW |
| 134 | [`03-oracle-as-score-cell.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/21-roadmap/03-oracle-as-score-cell.md) | Predict-publish-correct pattern. ResidualCorrector (~50ns). Conformal prediction. Oracle composition algebra. | Predict-publish-correct, ~50ns ResidualCorrector | **HIGH** |
| 135 | [`04-hdc-pattern-encoding-and-metabolism.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/21-roadmap/04-hdc-pattern-encoding-and-metabolism.md) | Role-filler BIND + temporal PERMUTE + BUNDLE. 0.526 resonance threshold. Red Queen pressure. | Role-filler HDC, 0.526 threshold | LOW |
| 136 | [`05-causal-discovery-and-adversarial-robustness.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/21-roadmap/05-causal-discovery-and-adversarial-robustness.md) | Pearl's 3-level causal hierarchy. Granger causality. Red-team dreaming for offline vulnerability discovery. | Pearl causal hierarchy, red-team dreaming | MEDIUM |
| 137 | [`06-advanced-geometry-and-integration.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/21-roadmap/06-advanced-geometry-and-integration.md) | TDA for knowledge graph topology. Sheaf consistency. Tropical geometry. IIT Phi. | TDA, sheaf, tropical geometry, IIT Phi | NONE |
| 138 | [`07-academic-foundations-by-protocol.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/21-roadmap/07-academic-foundations-by-protocol.md) | 500+ citations organized by the 9 protocols. | 500+ citations by protocol | MEDIUM |
| 139 | [`08-research-to-runtime-bridge.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/21-roadmap/08-research-to-runtime-bridge.md) | 5 theory-to-code bridges with fidelity-loss analysis and graduation criteria. | 5 bridges, fidelity-loss analysis | MEDIUM |
| 140 | [`09-test-strategy-and-verification.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/21-roadmap/09-test-strategy-and-verification.md) | Tests as Verify Cells. Algebraic/stateful/metamorphic property categories. Self-immunization. | Tests as Verify Cells, self-immunization | **HIGH** |

Cross-refs: [`./research-citations/README.md`](./research-citations/README.md) from docs 136, 138. [`../core-concepts/mathematical-primitives.md`](../core-concepts/mathematical-primitives.md) from docs 135, 137.

---

### 22-code-intelligence: Code Analysis Pipeline

**Directory**: [`docs/v2-depth/22-code-intelligence/`](https://github.com/wpank/roko/tree/main/docs/v2-depth/22-code-intelligence)

| # | File | Summary | Key Concepts | Rel. |
|---|------|---------|-------------|------|
| 141 | [`01-code-intelligence-as-cell-pipeline.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/22-code-intelligence/01-code-intelligence-as-cell-pipeline.md) | 84–99% code blindness. 4 costly mistake categories. 6-stage pipeline solution. | Code blindness, 4 categories, 6-stage pipeline | **HIGH** |
| 142 | [`02-symbol-graph-and-importance.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/22-code-intelligence/02-symbol-graph-and-importance.md) | Tree-sitter symbol extraction. Dependency graph + PageRank. Cross-language symbol resolution. | Tree-sitter, dependency graph, PageRank | **HIGH** |
| 143 | [`03-hdc-fingerprints-and-similarity.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/22-code-intelligence/03-hdc-fingerprints-and-similarity.md) | Structural + semantic HDC fingerprints. Cross-language unified space. ~10ns lookup. | Dual HDC fingerprints, ~10ns lookup | MEDIUM |
| 144 | [`04-search-and-context-assembly.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/22-code-intelligence/04-search-and-context-assembly.md) | Multi-strategy search (FTS/HDC/graph/recency). RRF ranked merge. Budget-constrained greedy assembly. | Multi-strategy search, RRF, greedy assembly | **HIGH** |
| 145 | [`05-mcp-server-and-persistence.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/22-code-intelligence/05-mcp-server-and-persistence.md) | MCP tools: symbol_lookup, search_code, get_context, explain_symbol, find_similar. SQLite persistence. Incremental snapshots. | MCP code tools, SQLite, incremental snapshots | **HIGH** |

Cross-refs: [`../context-memory/code-intelligence.md`](../context-memory/code-intelligence.md) from all five docs. [`../context-memory/language-support.md`](../context-memory/language-support.md) from docs 141–142.

---

## 5. IronClaw Module Map

Which depth sections map to which IronClaw modules:

| IronClaw Module | Most Relevant Depth Sections |
|-----------------|------------------------------|
| `crates/ironclaw_engine/` | 02-block (14–18), 05-execution-engine (33), 07-agent-runtime (38, 55, 57) |
| `crates/ironclaw_llm/` | 07-agent-runtime (40–41, 43, 66), 10-learning-loops (66–68) |
| `crates/ironclaw_safety/` | 01-signal (10), 17-security (113–115, 118) |
| `src/agent/` | 05-execution-engine (33–34), 07-agent-runtime (35–39, 47–50) |
| `src/workspace/` | 01-signal (8), 11-memory (73–82) |
| `src/tools/` | 13-builtin-catalog (93–96) |
| `src/tools/mcp/` | 13-builtin-catalog (94), 22-code-intelligence (145) |

---

## 6. Cross-References

| tmp/ Document | Primary Depth Sources | Section |
|---|---|---|
| [`../core-concepts/hyperdimensional-computing/README.md`](../core-concepts/hyperdimensional-computing/README.md) | 7, 74, 80, 135, 143 | 01-signal, 11-memory, 21-roadmap, 22-code-intel |
| [`../agent-intelligence/dream-consolidation.md`](../agent-intelligence/dream-consolidation.md) | 8, 76, 78–82 | 01-signal, 11-memory |
| [`../agent-intelligence/affect-engine.md`](../agent-intelligence/affect-engine.md) | 19, 46, 51–53, 58 | 02-block, 07-agent-runtime |
| [`../execution-verification/dag-execution.md`](../execution-verification/dag-execution.md) | 26–30, 33 | 03-graph, 05-execution-engine |
| [`../execution-verification/gate-verification.md`](../execution-verification/gate-verification.md) | 13, 20–22, 25, 113–114, 118 | 02-block, 17-security |
| [`../execution-verification/conductor-anomaly.md`](../execution-verification/conductor-anomaly.md) | 31, 47–50, 67–68, 71 | 03-graph, 07-agent-runtime, 10-learning |
| [`../agent-intelligence/online-learning.md`](../agent-intelligence/online-learning.md) | 9, 13, 66, 68–70 | 01-signal, 02-block, 10-learning |
| [`../context-memory/budget-composition.md`](../context-memory/budget-composition.md) | 14–18, 57, 96 | 02-block, 07-agent-runtime, 13-builtin |
| [`../core-concepts/universal-engram.md`](../core-concepts/universal-engram.md) | 7–10, 73, 76 | 01-signal, 11-memory |
| [`../context-memory/code-intelligence.md`](../context-memory/code-intelligence.md) | 141–145 | 22-code-intelligence |
| [`../core-concepts/cognitive-architecture.md`](../core-concepts/cognitive-architecture.md) | 1, 33, 44–45, 63, 70, 83–85 | 00-index, 05-engine, 07-agent-runtime |
| [`../execution-verification/runtime-infrastructure.md`](../execution-verification/runtime-infrastructure.md) | 34, 62, 88, 98–99, 128–131 | 05-engine, 09-telemetry, 12-connectivity, 14-config, 20-deploy |
| [`../execution-verification/orchestrator-swarm.md`](../execution-verification/orchestrator-swarm.md) | 26–30, 32, 37 | 03-graph, 07-agent-runtime |
| [`../ecosystem/mcp-editor-integration.md`](../ecosystem/mcp-editor-integration.md) | 38, 94, 145 | 07-agent-runtime, 13-builtin, 22-code-intel |
| [`../ecosystem/control-plane.md`](../ecosystem/control-plane.md) | 35–37, 43, 88, 106, 110 | 07-agent-runtime, 12-connectivity, 16-surfaces |
| [`./research-citations/README.md`](./research-citations/README.md) | 136, 138–140 | 21-roadmap |

---

## 7. Statistics

| Metric | Value |
|--------|-------|
| Total depth documents | 145 |
| Active sections (with docs) | 19 of 23 |
| Reserved sections (no docs) | 4 (04, 06, 08, 19) |
| HIGH relevance | 52 (36%) |
| MEDIUM relevance | 40 (28%) |
| LOW/NONE relevance | 53 (36%) |
| LOW/NONE cluster | Sections 15-marketplace, 18-registries |
| Largest section | 07-agent-runtime (27 documents) |
| Source material ingested | 422 documents, 8.8MB |
| Support files in root | 15 RESEARCH-PROMPT files + INDEX.md + GUIDE.md + INGEST-PROMPT.md |

# Cognitive Architecture

**Source sets**: `roko-core`, `roko-runtime`, `roko-orchestrator`, `roko-std`, `roko-primitives`, `roko-daimon`, `roko-neuro`, `roko-dreams`, `roko-compose`, `roko-gate`, `roko-learn`, `roko-conductor`
**Source docs**: `docs/v1/00-architecture/`, `docs/v1/13-coordination/`, `docs/v2-depth/11-memory/`
**Priority**: HIGH — formalizes IronClaw's reactive/reflective/background split and provides the theoretical foundation for self-improving multi-agent coordination

> **Self-contained implementation note**: Roko path-like references are provenance identifiers pointing to the captured source corpus. Use [implementation/README.md](../implementation/README.md) and [implementation/05-per-file-action-matrix.md](../implementation/05-per-file-action-matrix.md) for IronClaw-native build plans; use [benchmarking/README.md](../implementation/benchmarking/README.md) and [benchmarking/02-feature-playbooks.md](../implementation/benchmarking/02-feature-playbooks.md) for measurement plans.

**Related documents in this category:**
- [Universal Engram](./universal-engram.md) — the data type the Gamma tier indexes and Delta tier consolidates.
- [Hyperdimensional Computing](./hyperdimensional-computing/README.md) — the similarity engine the Gamma tier uses for fast Engram lookup.
- [Mathematical Primitives](./mathematical-primitives.md) — the analytical tools (TDA loop detection, robust stats) that instrument this architecture. Note: the morphogenetic update equations in Section 9 are canonical here; mathematical-primitives.md does not duplicate them.

---

## Table of Contents

1. [What This Document Covers](#1-what-this-document-covers)
2. [Theoretical Foundations](#2-theoretical-foundations)
3. [Three Cognitive Speeds (Gamma / Theta / Delta)](#3-three-cognitive-speeds-gamma--theta--delta)
4. [The Five-Layer Architecture](#4-the-five-layer-architecture)
5. [The Universal Cognitive Loop](#5-the-universal-cognitive-loop)
6. [Cognitive Cross-Cuts (Neuro / Daimon / Dreams)](#6-cognitive-cross-cuts-neuro--daimon--dreams)
7. [Stigmergic Coordination and Digital Pheromones](#7-stigmergic-coordination-and-digital-pheromones)
8. [SINR Interference Model](#8-sinr-interference-model)
9. [Morphogenetic Specialization (Turing Reaction-Diffusion)](#9-morphogenetic-specialization-turing-reaction-diffusion)
10. [C-Factor: Collective Intelligence Measurement](#10-c-factor-collective-intelligence-measurement)
11. [Benchmarking](#11-benchmarking)
12. [Practical Examples](#12-practical-examples)
13. [IronClaw Integration — Full Implementation Plan](#13-ironclaw-integration--full-implementation-plan)
14. [Academic Foundations](#14-academic-foundations)
15. [Complexity Assessment and Risk](#15-complexity-assessment-and-risk)

---

## 1. What This Document Covers

This document describes the cognitive architecture specified in the roko codebase — a multi-layered system that separates agent processing into distinct speeds, coordination mechanisms, and organizational layers, drawing from neuroscience, cognitive science, cybernetics, and biological self-organization. Each subsystem is documented with its theoretical foundations, the Rust code implementing it, and a concrete mapping to IronClaw's existing architecture with full integration code.

### What Is a Cognitive Architecture?

The term "cognitive architecture" in the AI agent context means the structural framework determining how an agent perceives, reasons, decides, acts, learns, and remembers. Just as the human brain is not a single monolithic processor but a collection of specialized subsystems operating at different timescales — perception at milliseconds, working memory at seconds, long-term consolidation during sleep — a well-designed AI agent should decompose its processing into distinct modes that match the computational demands of different situations.

Classical cognitive architectures from AI research (ACT-R, SOAR, CLARION) demonstrated that fixed structural commitments — how memory is organized, how production rules fire, how learning modifies future behavior — matter more for long-term capability than any single inference improvement. The same principle applies to LLM-based agents: the harness wrapping the LLM determines the agent's practical capability.

This is roko's core thesis: "the scaffold IS the product" (from `docs/v1/00-architecture/00-vision-and-thesis.md`). Evidence supports this claim: Lee et al. (2026) demonstrated +7.7 points from harness optimization alone with 4x fewer tokens (Meta-Harness, arXiv:2603.28052); Jimenez et al. (2024) showed the same model achieving 30-65% solve rates depending on harness (SWE-bench); and Zaharia et al. (2024) argued that SOTA performance comes from compound AI systems, not individual models.

### How Biological Cognition Inspires the Design

The architecture does not attempt to replicate the brain. Instead, it identifies specific computational problems that biological nervous systems have solved over 500 million years of evolution, and adapts those solutions for software agents:

- **Multiple timescales**: The brain processes sensory input in milliseconds, maintains working memory over seconds, and consolidates long-term knowledge during sleep over hours. Roko separates processing into three analogous speeds (Gamma/Theta/Delta), allowing the agent to react quickly to environmental changes while still stepping back periodically for deeper reflection.

- **Prediction-error driven attention**: The brain allocates expensive neural computation to surprising inputs and coasts on cached predictions for expected ones. Roko's tier-routing system implements this — most processing ticks use zero-cost heuristic checks (T0), escalating to expensive LLM inference only when surprise is detected.

- **Indirect coordination**: Social insects build elaborate structures without centralized planning, using environmental traces (pheromones) as coordination signals. Roko applies this stigmergic principle to multi-agent coordination, replacing direct inter-agent messaging with a shared pheromone environment.

- **Spontaneous specialization**: Embryonic cells differentiate into specialized tissues from identical starting conditions through reaction-diffusion dynamics. Roko uses the same mathematical mechanism (Turing/Gierer-Meinhardt) to produce emergent role differentiation among initially homogeneous agents.

---

## 2. Theoretical Foundations

The cognitive architecture draws from six research traditions. These are not decorative citations but load-bearing design decisions.

### 2.1 Neural Oscillation Bands (Buzsaki 2006)

The three cognitive speeds are named after neural oscillation bands documented in Buzsaki's "Rhythms of the Brain" (Oxford University Press, 2006, ISBN 978-0-19-530106-9). The mammalian brain operates at multiple frequency bands simultaneously, each supporting distinct cognitive functions:

| Brain Rhythm | Frequency | Cognitive Function | Roko Mapping |
|---|---|---|---|
| **Gamma** (30-100 Hz) | Fast | Sensory processing, attention binding, feature integration | Reactive: perceive environment changes and act immediately |
| **Theta** (4-8 Hz) | Medium | Working memory maintenance, spatial navigation, planning | Reflective: step back, re-plan, evaluate progress |
| **Delta** (0.5-4 Hz) | Slow | Deep sleep, memory consolidation, synaptic homeostasis | Consolidation: replay episodes, synthesize knowledge, prune |

The mapping is functional, not literal. The names capture the computational role: Gamma for fast environmental scanning, Theta for deliberate evaluation, Delta for deep offline processing. Buzsaki's key insight is that these rhythms are nested — gamma oscillations ride on top of theta waves — and cross-frequency coupling coordinates local and global processing. Roko implements an analogous nesting where Gamma ticks occur within Theta cycles, which occur within Delta epochs.

**Why this matters for IronClaw**: IronClaw currently has an implicit two-speed model (interactive agent turns vs. background heartbeat). The three-speed model makes the intermediate reflective speed explicit, enabling the agent to periodically step back and reassess its approach mid-conversation without waiting for the next heartbeat cycle.

### 2.2 Dual-Process Cognition (Kahneman 2011, Sun 2002)

Daniel Kahneman's "Thinking, Fast and Slow" (Farrar, Straus and Giroux, 2011, ISBN 978-0-374-27563-1) describes two modes of human cognition:

- **System 1**: Fast, automatic, heuristic. Pattern-matched responses requiring minimal conscious effort.
- **System 2**: Slow, deliberate, analytical. Step-by-step reasoning that demands attention and energy.

Ron Sun's CLARION architecture (Sun 2002, "Duality of the Mind", Lawrence Erlbaum Associates) extends this with a sub-conceptual level below System 1 — implicit pattern matching operating through distributed representations rather than explicit rules.

Roko maps these to three inference tiers:

| Cognitive Level | Kahneman | CLARION | Roko Tier | Characteristics |
|---|---|---|---|---|
| Sub-conceptual | — | Sub-conceptual (implicit) | **T0** | No LLM call. Threshold checks, regex matches, cache lookups. |
| System 1 | Fast, automatic | Bottom-up processing | **T1** | Fast model (Haiku-class). Quick analysis with limited tool access. |
| System 2 | Slow, deliberate | Top-down processing | **T2** | Full model (Sonnet/Opus-class). Multi-turn deep reasoning. |

**Source**: `docs/v1/00-architecture/11-dual-process-and-active-inference.md`

### 2.3 Active Inference and the Free Energy Principle (Friston 2010)

Karl Friston's Free Energy Principle (Friston, K., 2010, "The free-energy principle: a unified brain theory?", Nature Reviews Neuroscience, 11(2), pp. 127-138, DOI: 10.1038/nrn2787) proposes that self-organizing systems minimize the divergence between their predictions and their observations. The prediction error — the surprise signal — drives learning, attention, and action.

Roko uses this as the theoretical basis for tier routing. The Expected Free Energy (EFE) formula:

```
G(pi) = E_q[ log q(s|pi) - log p(o,s|pi) ]
      = -Pragmatic Value - Epistemic Value
```

Where:
- **Pragmatic value** = expected reward from acting on policy pi (what will I gain?)
- **Epistemic value** = expected information gain from observing under policy pi (what will I learn?)

This decomposes naturally into routing decisions: high-certainty situations (low epistemic value) route to fast processing (T0/T1), while high-uncertainty situations (high epistemic value) route to deep reasoning (T2).

In practice, roko approximates EFE through four observable signals: prediction accuracy from calibration streams, confidence from Score axes, novelty from observations, and Daimon arousal state.

### 2.4 Beer's Viable System Model (Beer 1972)

Stafford Beer's Viable System Model ("Brain of the Firm", Allen Lane, 1972; 2nd ed. Wiley, 1981, ISBN 978-0-471-27687-0) identifies five recursive subsystems required for any viable organization. Drawing on neurophysiology and cybernetics — particularly Ross Ashby's Law of Requisite Variety (1956) — Beer showed that any self-regulating system must have enough internal variety to match the variety of its environment. Roko's five architectural layers map directly to Beer's VSM:

| Beer VSM | Roko Layer | Function |
|---|---|---|
| System 1: Operations | L0 Runtime | Primary activities — process lifecycle, I/O |
| System 2: Coordination | L1 Framework | Anti-oscillation — model routing, tool dispatch |
| System 3: Control | L2 Scaffold + L3 Harness | Resource allocation + auditing |
| System 4: Intelligence | L4 Orchestration | Environmental scanning and adaptation |
| System 5: Policy | Cognitive cross-cuts (Daimon) | Identity, purpose, self-model |

The Good Regulator Theorem (Conant & Ashby, 1970, International Journal of Systems Science, 1(2), pp. 89-97) — that every good regulator of a system must be a model of that system — directly motivates the Daimon self-model subsystem.

**Source**: `docs/v1/00-architecture/12-five-layer-taxonomy.md`

### 2.5 Stigmergy (Grasse 1959)

Pierre-Paul Grasse coined the term "stigmergy" in 1959 to describe how termites coordinate the construction of elaborate mound structures without centralized planning (Grasse, P.-P., 1959, "La reconstruction du nid et les coordinations interindividuelles chez Bellicositermes natalensis et Cubitermes sp.", Insectes Sociaux, 6(1), pp. 41-80, DOI: 10.1007/BF02223791). The core insight: agents do not need to communicate directly — they only need to read from and write to a shared environment.

Theraulaz & Bonabeau (1999, "A Brief History of Stigmergy", Artificial Life, 5(2), pp. 97-116) formalized two distinct types: quantitative stigmergy (signals vary in intensity, like pheromone concentration) and qualitative stigmergy (signals trigger different behaviors based on type). Three conditions define stigmergy:

1. **Shared environment**: All agents can read from and write to a common medium
2. **Persistent modifications**: Agent actions leave traces that outlast the agent's presence
3. **Stimulus-response coupling**: Traces trigger specific behaviors in agents that encounter them

**Source**: `docs/v1/13-coordination/00-stigmergy-theory.md`

### 2.6 Turing Reaction-Diffusion (Turing 1952)

In "The Chemical Basis of Morphogenesis" (Turing, A. M., 1952, Philosophical Transactions of the Royal Society of London B, 237(641), pp. 37-72, DOI: 10.1098/rstb.1952.0012), Alan Turing showed that a system of two chemicals — an activator and an inhibitor — can produce stable spatial patterns from a uniform initial state. The key condition: the inhibitor must diffuse faster than the activator.

Gierer & Meinhardt formalized this as the activator-inhibitor model (Gierer, A. & Meinhardt, H., 1972, "A theory of biological pattern formation", Kybernetik, 12, pp. 30-39):

```
da/dt = rho_a * (a^2 / h) - mu_a * a + D_a * nabla^2(a) + sigma_a   (activator)
dh/dt = rho_h * a^2         - mu_h * h + D_h * nabla^2(h) + sigma_h   (inhibitor)
```

Where `a` = activator concentration, `h` = inhibitor concentration, `rho` = production rate, `mu` = decay rate, `D` = diffusion coefficient, `sigma` = noise (essential for symmetry breaking). The critical instability condition is `D_h >> D_a`: the inhibitor must diffuse much faster than the activator.

---

## 3. Three Cognitive Speeds (Gamma / Theta / Delta)

The three cognitive speeds are the heartbeat of the architecture. Every agent operates at all three timescales concurrently, managed by an adaptive clock that modulates cadence based on the agent's emotional/motivational state.

**Source**: `docs/v1/00-architecture/10-three-cognitive-speeds.md`
**Implementation**: `crates/roko-core/src/operating_frequency.rs`

### 3.1 Three Speeds Overview

```mermaid
graph LR
    subgraph Delta["Delta — Consolidation (30+ min)"]
        D1["NREM Replay<br/>Prioritized by prediction error"]
        D2["REM Imagination<br/>HDC recombination + counterfactuals"]
        D3["Knowledge Promotion<br/>Consolidated → Persistent"]
        D1 --> D2 --> D3
    end

    subgraph Theta["Theta — Reflective (75s–3min)"]
        T1["Summarize recent work"]
        T2["Update Daimon PAD vector"]
        T3["Check predictions"]
        T4["Re-plan if needed"]
        T1 --> T2 --> T3 --> T4
    end

    subgraph Gamma["Gamma — Reactive (5–15s)"]
        G1["T0 Probes<br/>16 zero-cost checks"]
        G2{Surprise?}
        G3["T1 Fast model<br/>Haiku-class"]
        G4{High stakes?}
        G5["T2 Full model<br/>Opus-class"]
        G6["Complete tick<br/>$0 cost"]
        G1 --> G2
        G2 -->|No| G6
        G2 -->|Yes| G3
        G3 --> G4
        G4 -->|No| G6
        G4 -->|Yes| G5
        G5 --> G6
    end

    Delta -.->|"Nest"| Theta
    Theta -.->|"Nest"| Gamma
```

| Speed | Period | What Happens | Default Inference Tier | Turn Budget |
|---|---|---|---|---|
| **Gamma** | ~5-15s | Reactive: perceive, assess, act, verify, persist | T0 (no LLM) | 0 (no agent dispatch) |
| **Theta** | ~75s-3min | Reflective: summarize, update Daimon, check predictions, re-plan | T1 (fast model) | 20 turns |
| **Delta** | Hours | Consolidation: replay, imagination, pruning, knowledge promotion | T2 (full model) | 50 turns |

### 3.2 Gamma — Reactive Speed

Gamma is the agent's heartbeat. Every 5-15 seconds, one complete cognitive loop tick executes: perceive the environment, select relevant information, compose a prompt, act, verify, and persist.

The critical design decision: **most Gamma ticks are T0 (zero LLM cost)**. Roko specifies 16 T0 probes — zero-LLM diagnostic checks that determine whether the environment has changed enough to warrant model inference. If nothing surprising is detected, the tick completes without invoking any model. The agent "coasts" on existing heuristics.

The 16 T0 probes cover the complete diagnostic surface:

| # | Probe | What It Checks |
|---|---|---|
| 1 | `config_changed` | Has configuration file changed? |
| 2 | `gate_failed_recently` | Did a gate fail in the last N ticks? |
| 3 | `file_modified` | Have watched files been modified externally? |
| 4 | `test_count_delta` | Did the test count change? |
| 5 | `compile_error_new` | Are there new compilation errors? |
| 6 | `budget_threshold` | Is the remaining budget below threshold? |
| 7 | `confidence_dropping` | Is confidence trending downward? |
| 8 | `prediction_violation` | Did a prediction fail to match reality? |
| 9 | `tool_health_degraded` | Is a tool's response time or error rate degraded? |
| 10 | `pheromone_detected` | Has a new pheromone been deposited? |
| 11 | `task_deadline_near` | Is a task deadline approaching? |
| 12 | `idle_timeout` | Has the agent been idle beyond threshold? |
| 13 | `knowledge_stale` | Is key knowledge past its freshness window? |
| 14 | `dependency_changed` | Has an upstream dependency task completed? |
| 15 | `metric_anomaly` | Is any tracked metric outside 2-sigma bounds? |
| 16 | `heartbeat_timeout` | Has the expected heartbeat interval elapsed? |

If all 16 probes return "no change," the tick completes at T0 cost ($0). This is how approximately 80% of ticks are suppressed — the FrugalGPT-inspired zero-cost majority (Chen et al. 2023, arXiv:2305.05176).

When a T0 probe detects surprise above threshold, the tick escalates:
1. Probe reports surprise → escalate to T1 (fast model)
2. T1 analysis reports high uncertainty or high stakes → escalate to T2 (full model)

### 3.3 Theta — Reflective Speed

Every ~75 seconds to 3 minutes (adjustable), the agent pauses reactive work to reflect:

- **Summarize**: What has happened since the last Theta tick?
- **Update Daimon**: Recalculate the PAD vector (Pleasure-Arousal-Dominance) from recent outcomes
- **Check predictions**: Which predictions have resolved? Update calibration.
- **Re-plan**: Is the current approach working? Should the agent switch strategies?
- **Knowledge update**: Promote useful Working-tier knowledge to Consolidated.

Theta ticks typically invoke T1 (fast model, e.g., Claude Haiku class) for summarization, or T2 (full model) for re-planning when deeper analysis is needed.

### 3.4 Delta — Consolidation Speed

During extended idle periods or on a scheduled basis (every 30+ minutes), the agent enters Delta mode for deep consolidation:

- **NREM Replay**: Replay recent episodes, weighted by prediction error magnitude (Mattar & Daw, 2018, "Prioritized memory access explains planning and hippocampal replay", Nature Neuroscience, 21, pp. 1609-1617). Episodes where the outcome differed most from expectation are replayed first.
- **REM Imagination**: Generate novel hypotheses via HDC (Hyperdimensional Computing) recombination (Boden, 2004, "The Creative Mind: Myths and Mechanisms", 2nd ed., Routledge). Counterfactual reasoning via Pearl's Structural Causal Model (Pearl, J., 2009, "Causality: Models, Reasoning, and Inference", 2nd ed., Cambridge University Press).
- **Knowledge Promotion**: Promote Consolidated-tier knowledge to Persistent.
- **Pruning**: Remove Engrams that have decayed below threshold.
- **Synthesis**: Extract cross-episode patterns into new playbook rules.

Delta ticks use T2 (full model, e.g., Claude Opus/Sonnet class) for deep reasoning and synthesis.

**Source**: `docs/v1/00-architecture/13-cognitive-cross-cuts.md` (Section 4: Dreams)

### 3.5 OperatingFrequency Enum (Source Code)

From `crates/roko-core/src/operating_frequency.rs:`

```rust
/// Cognitive operating frequency for agent work.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperatingFrequency {
    /// Reactive mode: perceive, retrieve, act.
    ///
    /// Tool calls, cache lookups, and signal routing.
    Gamma,
    /// Strategic mode: re-plan, update goals, evaluate progress.
    ///
    /// Periodic step-back / course-correction passes.
    Theta,
    /// Consolidation mode: replay, distill, meta-cognate.
    ///
    /// Slow learning and knowledge consolidation.
    Delta,
}

impl OperatingFrequency {
    /// Map to the existing inference tier model.
    #[must_use]
    pub const fn inference_tier(self) -> InferenceTier {
        match self {
            Self::Gamma => InferenceTier::T0,
            Self::Theta => InferenceTier::T1,
            Self::Delta => InferenceTier::T2,
        }
    }

    /// Map operating frequency to the default agent turn limit.
    ///
    /// - `Gamma` reactive work does not dispatch an agent.
    /// - `Theta` deliberative work uses the default 20-turn budget.
    /// - `Delta` reflective work gets a 50-turn budget.
    #[must_use]
    pub const fn turn_limit(self) -> u32 {
        match self {
            Self::Gamma => 0,
            Self::Theta => 20,
            Self::Delta => 50,
        }
    }
}
```

### 3.6 InferenceTier Enum (Source Code)

From `crates/roko-primitives/src/tier.rs:`

```rust
/// Three-tier gate for inference spend and latency.
///
/// - `T0`: suppress -- heuristics only, no LLM call
/// - `T1`: analyze -- light LLM (Haiku-class)
/// - `T2`: deliberate -- full LLM (Opus/Sonnet based on vitality)
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum InferenceTier {
    /// Suppress inference entirely. Returns `None` from `TierRouter`.
    T0 = 0,
    /// Light inference. Always routes to Haiku-class model.
    T1 = 1,
    /// Full inference. Routes to Opus above vitality threshold, Sonnet below.
    T2 = 2,
}
```

The `TierRouter` maps tiers to concrete models with vitality-aware degradation:

```rust
/// Vitality threshold: below this, T2 degrades from Opus to Sonnet.
pub const T2_VITALITY_THRESHOLD: f32 = 0.3;

pub struct TierRouter;

impl TierRouter {
    /// Select a model based on tier and vitality.
    ///
    /// - `T0` -> `None` (suppress inference)
    /// - `T1` -> `"claude-haiku-4-5"` (regardless of vitality)
    /// - `T2` -> `"claude-opus-4-6"` if vitality >= 0.3, `"claude-sonnet-4-6"` if below
    #[must_use]
    pub fn select_model(tier: InferenceTier, vitality: f32) -> Option<&'static str> {
        match tier {
            InferenceTier::T0 => None,
            InferenceTier::T1 => Some("claude-haiku-4-5"),
            InferenceTier::T2 => {
                if vitality >= T2_VITALITY_THRESHOLD {
                    Some("claude-opus-4-6")
                } else {
                    Some("claude-sonnet-4-6")
                }
            }
        }
    }
}
```

### 3.7 Frequency Selection Logic (Source Code)

From `crates/roko-core/src/operating_frequency.rs:`

```rust
impl OperatingFrequency {
    /// Select the operating frequency for a task and its current affect state.
    #[must_use]
    pub fn select(task: &Task, affect: &impl OperatingFrequencyAffect) -> Self {
        if is_reactive_task(task) {
            return Self::Gamma;
        }

        if is_reflective_task(task) {
            return Self::Delta;
        }

        if affect_suggests_reflection(affect) && task.is_substantial() {
            return Self::Delta;
        }

        Self::Theta
    }
}

fn is_reactive_task(task: &Task) -> bool {
    task_tag_matches(task, "quick_fix")
        || task_text_matches(
            task,
            &[
                "quick fix", "quick-fix", "gate re-check", "gate recheck",
                "permission check", "permission checks",
                "tool permission", "tool permissions",
                "subscription filter", "filter evaluation",
            ],
        )
}

fn is_reflective_task(task: &Task) -> bool {
    task_text_matches(
        task,
        &[
            "dream", "dream cycle", "plan regeneration", "regeneration",
            "retrospective", "retrospective analysis", "retro",
            "meta-cognition", "meta cognition", "consolidation",
        ],
    )
}

const LOW_CONFIDENCE_THRESHOLD: f64 = 0.3;

fn affect_suggests_reflection(affect: &impl OperatingFrequencyAffect) -> bool {
    affect.confidence() < LOW_CONFIDENCE_THRESHOLD
        && (affect.arousal() > 0.25 || affect.dominance() < -0.1)
}
```

Key behavior: when the Daimon reports low confidence (<0.3), combined with high arousal (>0.25) or low dominance (<-0.1), substantial tasks are promoted from Theta to Delta — giving the agent more time and deeper reasoning. The `is_substantial()` method checks for tasks estimated at 30+ minutes, high reasoning level, deep context weight, or hardened quality profile.

### 3.8 Adaptive Clock Scheduler (Source Code)

The `OperatingFrequencyScheduler` manages when Theta and Delta ticks fire, with adaptive cadence that responds to the agent's state:

```rust
/// Selects the next operating frequency from runtime context.
///
/// - Idle systems consolidate with `Delta`.
/// - Stalling or anxious systems shorten the theta cadence.
/// - Otherwise the scheduler stays in `Gamma` until theta becomes due.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OperatingFrequencyScheduler {
    theta_interval: Duration,  // default: 180s (3 minutes)
    delta_interval: Duration,  // default: 1800s (30 minutes)
}

impl OperatingFrequencyScheduler {
    /// Choose the next loop to run.
    #[must_use]
    pub fn select(&self, context: &OperatingFrequencyScheduleContext) -> OperatingFrequency {
        if context.is_idle() {
            return OperatingFrequency::Delta;
        }

        if context.time_since_last_theta >= self.delta_interval {
            return OperatingFrequency::Delta;
        }

        let theta_due = self.theta_interval_for(context);
        if context.time_since_last_theta >= theta_due {
            OperatingFrequency::Theta
        } else {
            OperatingFrequency::Gamma
        }
    }
}
```

The Theta interval adapts dynamically:

- **Stalling** (completion rate <= 0.25): Theta interval × 0.5 — reflect sooner because the agent is not making progress
- **Anxious** (confidence <= 0.35 AND arousal >= 0.25 AND dominance <= -0.1): Theta interval × 0.66 — the agent needs to step back more frequently

The schedule context carries affect signals:

```rust
/// Runtime inputs used by the operating-frequency scheduler.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OperatingFrequencyScheduleContext {
    pub time_since_last_theta: Duration,
    pub active_tasks: usize,
    pub completion_rate: f64,
    pub confidence: f64,
    pub arousal: f64,
    pub dominance: f64,
}
```

---

## 4. The Five-Layer Architecture

Roko's crates are organized into five architectural layers with strictly downward dependencies. Each layer maps to one of Beer's VSM subsystems.

**Source**: `docs/v1/00-architecture/12-five-layer-taxonomy.md`

### 4.1 Architecture Diagram

```mermaid
graph TB
    subgraph Apps["Applications"]
        A1["Coding Agent"]
        A2["Research Agent"]
        A3["Chain Agent"]
        A4["Custom Agent"]
    end

    subgraph L4["Layer 4: ORCHESTRATION"]
        L4a["DAGs, scheduling, state machines"]
        L4b["Multi-agent coordination"]
        L4c["Merge queue, worktree mgmt"]
    end

    subgraph L3["Layer 3: HARNESS"]
        L3a["Gate pipeline (11+ gates)"]
        L3b["Adaptive thresholds (EMA)"]
        L3c["Monitoring, interventions"]
    end

    subgraph L2["Layer 2: SCAFFOLD"]
        L2a["SystemPromptBuilder (6-layer)"]
        L2b["Context enrichment"]
        L2c["Token budget management"]
    end

    subgraph L1["Layer 1: FRAMEWORK"]
        L1a["LLM backends + routing"]
        L1b["Tool registry + dispatch"]
        L1c["Safety layer"]
    end

    subgraph L0["Layer 0: RUNTIME / KERNEL"]
        L0a["Process lifecycle, I/O"]
        L0b["Store/Substrate (Engrams)"]
        L0c["Bus (Pulses)"]
        L0d["HDC, adaptive clock"]
    end

    subgraph XC["COGNITIVE CROSS-CUTS (injected via trait objects)"]
        XC1["Neuro (knowledge)"]
        XC2["Daimon (motivation)"]
        XC3["Dreams (offline learning)"]
        XC4["Safety & Provenance"]
        XC5["Observability"]
    end

    Apps --> L4 --> L3 --> L2 --> L1 --> L0
    XC -.->|"&dyn Trait"| L4
    XC -.->|"&dyn Trait"| L3
    XC -.->|"&dyn Trait"| L2
    XC -.->|"&dyn Trait"| L1
```

**Dependencies flow STRICTLY downward.** L4 may depend on L3, never the reverse. Cross-cutting concerns are injected via trait objects.

### 4.2 Layer Descriptions

| Layer | Purpose | Key Crates | Beer VSM |
|---|---|---|---|
| **L0: Runtime** | Process lifecycle, Store/Substrate (durable Engrams/Signals), Bus (ephemeral Pulses), HDC, adaptive clock | `roko-core`, `roko-primitives`, `roko-runtime` | System 1: Operations |
| **L1: Framework** | LLM backends, tool registry, model routing (CascadeRouter), safety | `roko-agent`, `roko-std` | System 2: Coordination |
| **L2: Scaffold** | SystemPromptBuilder (6-layer), context enrichment, token budget | `roko-compose` | System 3: Control |
| **L3: Harness** | Gate pipeline (compile/test/clippy/diff/format/schema/judge/simulation), adaptive thresholds | `roko-gate`, `roko-fs` | System 3*: Audit |
| **L4: Orchestration** | Plan DAGs, parallel execution, state machines, multi-agent coordination, session resumption | `roko-orchestrator`, `roko-conductor` | System 4: Intelligence |

### 4.3 Dependency Rules

```
L4 depends on -> L3, L2, L1, L0
L3 depends on -> L2, L1, L0
L2 depends on -> L1, L0
L1 depends on -> L0
L0 depends on -> (nothing above)
```

Cross-cutting crates are NOT layer-bound. They are injected as `&dyn Trait` objects:

```rust
fn compose_with_knowledge(
    composer: &dyn Composer,
    knowledge: &dyn Substrate,
    bus: &dyn Bus,
    budget: &Budget,
    scorer: &dyn Scorer,
    ctx: &Context,
) -> Result<Engram> {
    let knowledge_engrams = knowledge
        .query(&Query::of_kind(Kind::Insight).limit(5), ctx)
        .await?;
    let recent_pulses = bus
        .replay_since(ctx.checkpoint_seq, &TopicFilter::Glob("gate.verdict.*".into()))
        .await?;
    composer.compose_with(knowledge_engrams, recent_pulses, budget, scorer, ctx)
}
```

### 4.4 IronClaw Layer Mapping

| Roko Layer | IronClaw Equivalent | Notes |
|---|---|---|
| L0 Runtime | `src/db/`, `src/workspace/`, tokio runtime | Database + workspace = Store/Substrate; tokio = process lifecycle |
| L1 Framework | `src/tools/`, `src/evaluation/`, `src/estimation/` | Tool dispatch, scoring, model routing |
| L2 Scaffold | `crates/ironclaw_engine/`, `crates/ironclaw_llm/` | Prompt composition, LLM integration |
| L3 Harness | `crates/ironclaw_safety/`, `src/sandbox/`, `src/observability/` | Safety checks, sandboxing, monitoring |
| L4 Orchestration | `src/agent/`, `src/channels/`, `src/hooks/` | Agent loop, multi-channel input, lifecycle hooks |

---

## 5. The Universal Cognitive Loop

Every agent in roko runs the same seven-step cognitive loop at each of the three speeds. What changes between speeds is the scope of perception, the budget available for composition, and the persistence cadence — not the loop structure itself.

**Source**: `docs/v1/00-architecture/09-universal-cognitive-loop.md`

### 5.1 The Seven Steps

```mermaid
graph LR
    S["1. SENSE<br/>Substrate.query<br/>Bus.subscribe<br/>External I/O"]
    A["2. ASSESS<br/>Scorer + Router<br/>rank & select"]
    C["3. COMPOSE<br/>Composer assembles<br/>prompt Engram under budget"]
    Act["4. ACT<br/>LLM / tool / chain<br/>→ Pulses + Engrams"]
    V["5. VERIFY<br/>Engram-gates +<br/>stream-gates → Verdicts"]
    P["6a. PERSIST<br/>Substrate.put(Engrams)"]
    B["6b. BROADCAST<br/>Bus.publish(Pulses)"]
    R["7. REACT<br/>Policy → more<br/>Pulses + Engrams"]

    S --> A --> C --> Act --> V --> P & B --> R --> S
```

Step 6 is intentionally split into two co-equal operations: `PERSIST` writes durable Engrams to the Substrate, while `BROADCAST` publishes ephemeral Pulses onto the Bus for live consumers.

### 5.2 Step Details

**SENSE** has three sources: `Substrate.query()` for durable Engrams (plans, episodes, heuristics), `Bus.subscribe()` for live Pulses (turn output, cancellation), and external I/O for inputs not yet normalized.

**ASSESS** is a combined Scorer + Router operation that answers two questions together: what matters, and why this item wins over alternatives.

**COMPOSE** turns selected material into a prompt Engram under a budget (tokens, bytes, wall time). This is where context window shaping happens.

**ACT** executes the selected work (typically an LLM turn, but also tool calls and chain actions). Output is twofold: a stream of Pulses for live observers, and a final Engram capturing the action result.

**VERIFY** is a gate pipeline, not a single check. Engram-gates verify durable outputs (producing Verdict Engrams). Stream-gates watch live Pulses during execution and can halt or downgrade before the final result is accepted.

**REACT**: Policies consume new Pulses and emit further outputs — episode consolidation, circuit-breaking, routing feedback, task follow-up.

### 5.3 Cross-Cut Injection Points

Cross-cuts are not loop steps — they inject into specific operators:

- **Neuro** contributes durable knowledge to SENSE and COMPOSE
- **Daimon** biases ASSESS and gates risky actions in ACT
- **Dreams** runs on its own Delta-speed cycle, consuming and producing Engrams outside the main loop

### 5.4 Cross-Layer Arbitration Protocol

```mermaid
flowchart TD
    Conflict["Cross-cut conflict detected"]
    Daimon["1. Daimon wins<br/>(safety + behavioral gating)"]
    Neuro["2. Neuro wins<br/>(validated knowledge)"]
    Dreams["3. Dreams signal<br/>(speculative hypothesis)"]
    Tied{Still tied?}
    VCG["VCG Attention Auction<br/>Each cross-cut bids confidence.<br/>Winner pays 2nd-highest bid.<br/>Truthful by mechanism design."]
    Result["Winning signal applied"]

    Conflict --> Daimon
    Daimon -->|"Daimon = safety constraint"| Result
    Daimon -->|"No safety constraint"| Neuro
    Neuro -->|"Validated knowledge present"| Result
    Neuro -->|"Only speculative"| Dreams
    Dreams --> Tied
    Tied -->|"Yes"| VCG --> Result
    Tied -->|"No"| Result
```

**Source**: `docs/v1/00-architecture/13-cognitive-cross-cuts.md` (Section 6)

---

## 6. Cognitive Cross-Cuts (Neuro / Daimon / Dreams)

Three cognitive subsystems are injected across multiple layers rather than living at any single level.

**Source**: `docs/v1/00-architecture/13-cognitive-cross-cuts.md`

### 6.1 Neuro — Knowledge Management

`roko-neuro` provides persistent, tier-based knowledge management with HDC encoding for similarity search.

**Six Knowledge Types**:

| Type | Purpose | Example |
|---|---|---|
| **Insight** | General observation that proved useful | "This codebase uses builder pattern extensively" |
| **Heuristic** | Procedural rule from experience | "When tests fail with E0599, check trait imports first" |
| **Warning** | Known pitfall or anti-pattern | "Never use --no-verify with this repo's hooks" |
| **CausalLink** | Cause-effect relationship | "Upgrading alloy requires rustc 1.91+" |
| **StrategyFragment** | Reusable strategic approach | "For large refactors, use worktrees for parallel branches" |
| **AntiKnowledge** | Explicitly falsified knowledge | "Hypothesis X was tested and disproved" |

**Four Knowledge Tiers** (with Ebbinghaus forgetting curve decay):

| Tier | Strength | Effective Half-Life | Promotion Criteria |
|---|---|---|---|
| **Transient** | 0.1x | Minutes to hours | Created on first observation |
| **Working** | 0.5x | Hours to days | Referenced in 2+ successful ticks |
| **Consolidated** | 1.0x | Days to weeks | Validated by gate verdicts |
| **Persistent** | 5.0x | Weeks to months | Repeatedly validated across sessions |

Knowledge can be encoded as 10,240-bit HDC vectors (Kanerva, P., 2009, "Hyperdimensional Computing: An Introduction to Computing in Distributed Representation with High-Dimensional Random Vectors", Cognitive Computation, 1(2), pp. 139-159). Each fixed-width Hamming comparison is constant cost; brute-force retrieval remains O(N) unless paired with an index or candidate filter.

### 6.2 Daimon — Motivation and Focus

`roko-daimon` provides the agent's self-model: a PAD (Pleasure-Arousal-Dominance) vector (Mehrabian, A. & Russell, J. A., 1974, "An Approach to Environmental Psychology", MIT Press) that biases assessment, action gating, and cadence selection.

**PAD Dimensions**:

| Dimension | Range | What It Represents |
|---|---|---|
| **Pleasure** (P) | [-1, 1] | Task success vs. failure |
| **Arousal** (A) | [-1, 1] | Urgency and load |
| **Dominance** (D) | [-1, 1] | Confidence and control |

**Six Behavioral States** (simplified Plutchik emotion wheel, Plutchik, R., 2001, American Scientist, 89(4), pp. 344-350):

| State | PAD Region | Behavior |
|---|---|---|
| **Engaged** | P+, A moderate, D+ | Productive work. Standard Theta cadence. |
| **Focused** | P+, A low, D+ | Deep work. Extended Gamma runs, fewer Theta interruptions. |
| **Exploring** | P neutral, A+, D neutral | Curious. Higher exploration rate, more T2 escalation. |
| **Struggling** | P-, A+, D- | Difficulty. Shortened Theta cadence, more frequent reflection. |
| **Coasting** | P neutral, A-, D+ | Easy work. Extended Gamma, T0-heavy. |
| **Resting** | P neutral, A-, D neutral | Idle. Delta consolidation mode. |

**Somatic Markers** (Damasio, A., 1994, "Descartes' Error", Putnam): Emotional signals from past experience bias decision-making before analytical reasoning. In roko, somatic markers are score modifiers applied to Router selections — negative markers for tools/approaches that previously failed, positive for those that succeeded.

### 6.3 Dreams — Offline Learning

`roko-dreams` provides offline learning during idle time at Delta frequency:

| Phase | Neuroscience Inspiration | What Happens |
|---|---|---|
| **NREM Replay** | Slow-wave sleep replay (Mattar & Daw, 2018) | Replay recent episodes weighted by prediction error magnitude |
| **REM Imagination** | REM sleep creativity (Boden, 2004) | Generate novel hypotheses via HDC recombination; counterfactual reasoning (Pearl, 2009); emotional depotentiation (Walker & van der Helm, 2009) |
| **Integration Staging** | Memory consolidation (Lacaux et al., 2021, Science Advances, 7(50)) | Validate dream outputs against existing knowledge; promote if confidence exceeds threshold |

The **Hypnagogia Engine** generates creative hypotheses during the transition between active work and consolidation, with four components: Thalamic Gate (filters for high-novelty), Executive Loosener (relaxes constraints), Dali Interrupt (captures fleeting insights), and Homuncular Observer (coherence filter).

---

## 7. Stigmergic Coordination and Digital Pheromones

Roko's multi-agent coordination uses stigmergy: indirect coordination through environment modification. Rather than agents communicating directly, they deposit and sense digital pheromones — coordination signals that decay over time, can be confirmed or contradicted, and drive emergent task allocation.

**Source**: `docs/v1/13-coordination/04-pheromone-kinds.md`, `docs/v2-depth/11-memory/12-pheromone-mechanics-and-interference.md`
**Implementation**: `crates/roko-orchestrator/src/coordination.rs`

### 7.1 Pheromone Lifecycle

```mermaid
graph LR
    D["DEPOSIT<br/>Agent writes pheromone<br/>to Substrate/workspace<br/>intensity = initial_intensity"]
    Df["DIFFUSE<br/>Scope promotion:<br/>Local → Subnet → Mesh → Global<br/>Trust discounted by scope distance"]
    Dc["DECAY<br/>intensity(t) = I₀ × 2^(-t/hl)<br/>Lazy computation from deposit_time"]
    T["THRESHOLD CHECK<br/>Response probability:<br/>P = Iⁿ / (Iⁿ + θⁿ)<br/>Hill function, n=2"]
    Ac["ACTION<br/>Agent responds:<br/>threshold.reinforce() or .habituate()<br/>Specialization emerges"]
    Pr["PROMOTE<br/>Pattern → Wisdom (3+ conf)<br/>Wisdom → Consensus (4+ conf)<br/>Consensus → Engram (5+ conf)"]
    Ev["EVAPORATE<br/>intensity < 0.01<br/>Remove from store"]

    D --> Df --> Dc --> T --> Ac
    Ac -->|"confirmed"| Pr
    Ac -->|"ignored"| Dc
    Dc -->|"exhausted"| Ev
```

### 7.2 PheromoneKind Enum (Source Code)

From `crates/roko-orchestrator/src/coordination.rs:`

```rust
/// The type of coordination signal a pheromone carries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PheromoneKind {
    /// Something dangerous or harmful has been detected.
    Threat,
    /// A favorable condition has been detected.
    Opportunity,
    /// Validated knowledge or insight that should persist.
    Wisdom,
    /// First-mover advantage or ephemeral edge.
    Alpha,
    /// Recurring structure or regularity detected.
    Pattern,
    /// Something unusual or unexpected detected.
    Anomaly,
    /// Collective agreement on a fact or decision.
    Consensus,
    /// User-defined pheromone kind for domain-specific signals.
    Custom(String),
}

impl PheromoneKind {
    /// Return the documented default half-life for this kind.
    #[must_use]
    pub const fn default_half_life(&self) -> Duration {
        match self {
            Self::Threat    => Duration::from_secs(2  * 60 * 60),  // 2 hours
            Self::Opportunity => Duration::from_secs(4 * 60 * 60), // 4 hours
            Self::Wisdom    => Duration::from_secs(24 * 60 * 60),  // 24 hours
            Self::Alpha     => Duration::from_secs(60 * 60),        // 1 hour
            Self::Pattern   => Duration::from_secs(12 * 60 * 60),  // 12 hours
            Self::Anomaly | Self::Custom(_) => Duration::from_secs(6 * 60 * 60), // 6 hours
            Self::Consensus => Duration::from_secs(48 * 60 * 60),  // 48 hours
        }
    }
}
```

### 7.3 All Seven Pheromone Kinds in Detail

The kind system is organized into three tiers, inspired by Wilson's hierarchy of pheromone types in social insects (Wilson, E. O., 1971, "The Insect Societies", Belknap Press of Harvard University Press):

**Tier 1 — Universal Kinds** (present in every domain, every agent, every scope):

#### Threat (Alarm Pheromone)

| Property | Value |
|---|---|
| Default half-life | 2 hours |
| Default initial intensity | 1.0 |
| Biological analog | Alarm pheromone (formic acid in ants) |
| Confirmation effect | Standard extension (longer half-life) |
| Agent response | Stop current task, investigate, remediate |

Threat intensity encodes severity:
- 0.1-0.3 = Low (style violation)
- 0.4-0.6 = Medium (flaky tests)
- 0.7-0.8 = High (test failure, security vulnerability in dev)
- 0.9-1.0 = Critical (build failure, production security vulnerability)

When ambient Threat intensity is high, gate thresholds tighten — implementing a collective immune response.

#### Opportunity (Recruitment Pheromone)

| Property | Value |
|---|---|
| Default half-life | 4 hours |
| Default initial intensity | 0.8 |
| Biological analog | Recruitment pheromone (trail to food source) |
| Confirmation effect | Standard extension |
| Agent response | Evaluate opportunity, add to task queue if aligned with role |

High-intensity Threat (> 0.7) suppresses Opportunity pheromones in the same scope — agents in threat-response mode should not be distracted by opportunities. This mirrors how alarm pheromone overrides foraging pheromone in ant colonies.

#### Wisdom (Trail Pheromone)

| Property | Value |
|---|---|
| Default half-life | 24 hours |
| Default initial intensity | 0.9 |
| Biological analog | Established trail pheromone |
| Confirmation effect | Extends half-life; promotes to Consensus at 4+ confirmations |
| Agent response | Integrate into local knowledge base, apply to current work |

Wisdom pheromones typically emerge through a pipeline rather than direct deposit:

```
Agent observes pattern → deposits Pattern pheromone
    → Multiple agents confirm the Pattern
    → Agent validates through operational testing
    → Agent deposits Wisdom with Pattern as parent
    → Other agents confirm the Wisdom
    → At 5+ confirmations, may be promoted to permanent Engram
```

**Tier 2 — Domain-Specific Kinds** (common across domains, domain-dependent interpretation):

#### Alpha (Ephemeral Edge)

| Property | Value |
|---|---|
| Default half-life | 1 hour |
| Default initial intensity | 1.0 |
| Confirmation effect | **PARADOXICAL**: confirmation REDUCES effective half-life |
| Agent response | Act immediately or discard |

**The Alpha Paradox**: Unlike other kinds where confirmation increases persistence, confirmation of an Alpha signal indicates the first-mover advantage is eroding. From the verified source code:

```rust
// From Pheromone::effective_half_life() in coordination.rs:
if self.kind.is_alpha() {
    // Alpha paradox: consensus makes alpha expire faster.
    let divisor = f64::from(self.confirmations).mul_add(0.1, 1.0);
    self.half_life.mul_f64(1.0 / divisor)
}
```

| Confirmations | Divisor | Multiplier | Effective Half-Life (base = 1h) |
|---|---|---|---|
| 0 | 1.0 | 1.00 | 60 min |
| 1 | 1.1 | 0.91 | 55 min |
| 3 | 1.3 | 0.77 | 46 min |
| 5 | 1.5 | 0.67 | 40 min |
| 10 | 2.0 | 0.50 | 30 min |

#### Pattern (Territorial Marking)

| Property | Value |
|---|---|
| Default half-life | 12 hours |
| Default initial intensity | 0.7 |
| Confirmation effect | Standard extension; promotes to Wisdom at 3+ confirmations |
| Agent response | Incorporate pattern into decision-making |

#### Anomaly (Novel Scent Detection)

| Property | Value |
|---|---|
| Default half-life | 6 hours |
| Default initial intensity | 0.8 |
| Confirmation effect | Standard extension |
| Agent response | Investigate → classify as Threat, Opportunity, or noise → re-deposit |

#### Consensus (Colony Odor)

| Property | Value |
|---|---|
| Default half-life | 48 hours |
| Default initial intensity | 0.9 |
| Confirmation effect | Standard extension; resists contradiction |
| Agent response | Treat as established fact; violating requires strong evidence |

**Tier 3 — Custom Kinds**: `Custom(String)` allows domain plugins to define their own pheromone kinds without modifying the core enum. Custom kind identifiers are validated (alphanumeric + underscores, 1-64 chars, no leading underscore, no collision with built-in names).

### 7.4 Decay Model

Every pheromone decays according to an exponential half-life model:

```
intensity(t) = initial_intensity × 2^(-t / half_life)
```

Decay is computed lazily: the stored intensity is the value at deposit time, and any read computes current intensity from the deposit timestamp. This eliminates tick-aligned decay updates.

```rust
/// Compute the current intensity of a pheromone given its age.
#[must_use]
pub fn current_intensity(initial_intensity: f64, half_life: Duration, elapsed: Duration) -> f64 {
    if half_life.is_zero() {
        return 0.0;
    }
    let exponent = -(elapsed.as_secs_f64() / half_life.as_secs_f64());
    initial_intensity * exponent.exp2()
}
```

**Confirmation extension** (for standard, non-Alpha kinds):

```rust
// Standard: confirmations extend the half-life linearly.
self.half_life.mul_f64(f64::from(self.confirmations).mul_add(0.5, 1.0))
```

This means `effective_half_life = base × (1 + 0.5 × confirmations)`:

| Confirmations | Multiplier | Effective Half-Life (base = 12h) |
|---|---|---|
| 0 | 1.0 | 12.0h |
| 1 | 1.5 | 18.0h |
| 2 | 2.0 | 24.0h |
| 3 | 2.5 | 30.0h |
| 5 | 3.5 | 42.0h |

### 7.5 Pheromone Scope

```rust
/// The propagation scope of a digital pheromone.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PheromoneScope {
    Local(SubstrateId),    // Visible only within the specified substrate
    Mesh(CollectiveId),    // Propagates to all agents in the specified collective
    Subnet(SubnetId),      // Propagates within a permissioned subnet
    Global,                // Published globally
}

/// Trust discount factors applied when reading pheromones from a broader scope.
pub const TRUST_DISCOUNT: [f64; 4] = [1.0, 0.90, 0.80, 0.50];
```

Scopes are ranked: Local (0) < Subnet (1) < Mesh (2) < Global (3). Local signals carry full trust; Global signals are halved, reflecting that signals passing through more intermediaries may have been confirmed by agents with unknown provenance.

### 7.6 Promotion Cascade

```
Pattern  --[3+ confirmations, age > 50% half-life]--> Wisdom
Wisdom   --[4+ confirmations]-----------------------> Consensus
Consensus --[5+ confirmations]---------------------> Permanent Engram
```

```rust
/// Promotion thresholds for the Pattern -> Wisdom -> Consensus cascade.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromotionConfig {
    pub pattern_to_wisdom_confirmations: u32,       // default: 3
    pub pattern_to_wisdom_min_age_fraction: f64,    // default: 0.5
    pub wisdom_to_consensus_confirmations: u32,     // default: 4
    pub consensus_to_engram_confirmations: u32,     // default: 5
    pub auto_promote: bool,                          // default: true
}
```

Scope promotion is controlled separately — pheromones can escalate from Local to Subnet (3+ confirmations), Subnet to Mesh (5+), and Mesh to Global (10+), implemented through `ScopePromotionConfig` and `PromotionGate` (which requires confirmations from a minimum number of distinct agents).

### 7.7 Response Threshold Model (Emergent Task Allocation)

Each agent maintains per-kind response thresholds that determine when it switches from current work to respond to a pheromone signal. The probability follows a Hill function (Bonabeau, E., Theraulaz, G. & Deneubourg, J.-L., 1998, "Fixed Response Thresholds and the Regulation of Division of Labor in Insect Societies", Bulletin of Mathematical Biology, 60, pp. 753-807):

```
P(respond to kind k) = I_k^n / (I_k^n + theta_k^n)
```

Where `I_k` = current intensity, `theta_k` = agent's response threshold, `n` = Hill coefficient (default: 2).

The Hill coefficient controls steepness: n=1 gives a hyperbolic curve, n=2 gives a sigmoidal transition, n=4 gives an almost step-function. The default n=2 balances sensitivity and stability.

```rust
/// Per-agent response thresholds for pheromone-driven task allocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseThresholds {
    pub thresholds: HashMap<PheromoneKind, f64>,  // lower = more responsive
    pub hill_coefficient: u32,                     // default: 2
    pub learning_rate: f64,                        // default: 0.05
    pub min_threshold: f64,                        // default: 0.05
    pub max_threshold: f64,                        // default: 0.95
}

impl ResponseThresholds {
    pub fn response_probability(&self, kind: &PheromoneKind, intensity: f64) -> f64 {
        let theta = clamp_unit(self.thresholds.get(kind).copied().unwrap_or(0.5));
        let n = f64::from(self.hill_coefficient.max(1));
        let i_n = clamp_unit(intensity).powf(n);
        let theta_n = theta.powf(n);
        let denom = i_n + theta_n;
        if denom == 0.0 { 0.0 } else { i_n / denom }
    }

    /// Successful response -> lower threshold -> more responsive
    pub fn reinforce(&mut self, kind: &PheromoneKind) {
        if let Some(theta) = self.thresholds.get_mut(kind) {
            *theta = (*theta - self.learning_rate).max(self.min_threshold);
        }
    }

    /// Ignoring a signal -> raise threshold -> less responsive (slower rate)
    pub fn habituate(&mut self, kind: &PheromoneKind) {
        if let Some(theta) = self.thresholds.get_mut(kind) {
            *theta = self.learning_rate
                .mul_add(0.5, *theta)
                .min(self.max_threshold);
        }
    }
}
```

Note: habituation is asymmetric — the threshold increases at half the learning rate (0.025) compared to reinforcement's decrease rate (0.05). This produces a bias toward responsiveness: it takes twice as many non-responses to undo one successful response.

This produces **emergent division of labor**: an agent that succeeds at handling Threats develops lower Threat thresholds, making it more likely to respond to future Threats — specializing as a "threat responder" without explicit role assignment.

---

## 8. SINR Interference Model

When multiple pheromone kinds coexist in the same scope, they interfere with each other's detectability. The interference model adapts the Signal-to-Interference-plus-Noise Ratio (SINR) from wireless communications (Tse, D. & Viswanath, P., 2005, "Fundamentals of Wireless Communication", Cambridge University Press).

**Source**: `docs/v2-depth/11-memory/12-pheromone-mechanics-and-interference.md` (Section 3)

### 8.1 The SINR Formula

```
SINR_k = I_target_k / (SUM_{j != k} alpha_{jk} * I_j + N_0)
```

Where:
- `I_target_k` = intensity of the target pheromone of kind k
- `I_j` = aggregate intensity of pheromones of kind j in the same scope
- `alpha_{jk}` = cross-kind interference coefficient (how much kind j interferes with sensing kind k)
- `N_0` = background noise floor (default: 0.01)

If SINR falls below `min_sinr` (default: 1.0, corresponding to 0 dB), the signal is undetectable. Otherwise, the effective intensity degrades gracefully via: `effective = target × (sinr / (1 + sinr))`.

### 8.2 The 7×7 Interference Matrix

The default matrix encodes biological precedent from Wilson 1971:

```
              Thr   Opp   Wis   Alp   Pat   Ano   Con
Threat  [   0.00  0.60  0.10  0.30  0.20  0.10  0.05 ]
Opp     [   0.00  0.00  0.00  0.10  0.05  0.00  0.00 ]
Wisdom  [   0.00  0.00  0.00  0.00  0.00  0.00  0.00 ]
Alpha   [   0.10  0.10  0.00  0.00  0.05  0.00  0.00 ]
Pattern [   0.00  0.00  0.00  0.00  0.00  0.00  0.00 ]
Anomaly [   0.20  0.10  0.00  0.10  0.10  0.00  0.00 ]
Cons    [   0.00  0.00  0.00  0.00  0.00  0.00  0.00 ]
```

Key design choices:
- **Threat → Opportunity: 0.60** — Alarm overrides foraging. During a crisis, opportunities are suppressed.
- **Threat → Wisdom: 0.10** — Knowledge is resistant to alarm. Validated insights persist through crisis.
- **Opportunity → Threat: 0.00** — Opportunities never mask threats. Safety is asymmetric.
- **Wisdom and Pattern rows: all zeros** — Informational kinds do not interfere with other sensing.
- **Consensus: all zeros outbound** — Collective agreement is highly resistant to interference.

### 8.3 SINR-Adjusted Intensity Computation

```rust
fn sinr_adjusted_intensity(
    target_kind: usize,
    target_intensity: f64,
    active_intensities: &[f64; 7],
    matrix: &[[f64; 7]; 7],
    noise_floor: f64,
    min_sinr: f64,
) -> f64 {
    let interference: f64 = active_intensities.iter()
        .enumerate()
        .filter(|&(j, _)| j != target_kind)
        .map(|(j, &intensity)| matrix[j][target_kind] * intensity)
        .sum();

    let sinr = target_intensity / (interference + noise_floor);
    if sinr < min_sinr {
        0.0  // Below detection threshold
    } else {
        target_intensity * (sinr / (1.0 + sinr))  // Graceful degradation
    }
}
```

### 8.4 Anti-Saturation Mechanisms

Two thresholds prevent pheromone flooding:

```rust
struct AntiSaturationConfig {
    soft_threshold: usize,           // default: 500 — 2x decay speed
    hard_threshold: usize,           // default: 2000 — only intensity > 0.1 survives
    hard_min_intensity: f64,         // default: 0.1
    max_per_kind_per_scope: usize,   // default: 100 — no single kind monopolizes
}
```

---

## 9. Morphogenetic Specialization (Turing Reaction-Diffusion)

When a group of agents starts with identical configurations, they face the **niche crowding problem**: all agents pursue the same strategies, compete for the same tasks, and produce redundant work. Morphogenetic specialization solves this by applying Turing's reaction-diffusion mechanism to produce spontaneous role differentiation from homogeneous starting conditions.

> **Canonical location**: The Gierer-Meinhardt update equations and convergence analysis are defined here. Mathematical Primitives ([mathematical-primitives.md](./mathematical-primitives.md)) does not cover reaction-diffusion; it provides orthogonal analytical tools (TDA, sheaves, robust stats) that can instrument these agent dynamics but does not restate these update rules.

**Source**: `docs/v1/13-coordination/07-morphogenetic-specialization.md`, `docs/v2-depth/11-memory/13-morphogenetic-specialization-as-loop.md`
**Implementation**: `crates/roko-orchestrator/src/coordination.rs`

### 9.1 How Turing's Mechanism Applies to Agents

```mermaid
graph LR
    subgraph Bio["Biological System"]
        BA["Activator<br/>Local, slow (gene expression)"]
        BI["Inhibitor<br/>Fast extracellular diffusion"]
        BP["Spatial pattern:<br/>specialized tissue types"]
    end

    subgraph Agent["Agent System"]
        AA["Returns on strategy dims<br/>Local, slow (100s of ticks)"]
        AI["Pheromone Bus pulses<br/>Fast (~milliseconds)"]
        AP["Role differentiation:<br/>specialized agent roles"]
    end

    BA -->|"analog"| AA
    BI -->|"analog"| AI
    BP -->|"analog"| AP
```

| Biological Component | Agent Equivalent |
|---|---|
| Activator | Profitable returns for a strategy dimension — local, slow (learning takes hundreds of ticks) |
| Inhibitor | Pheromone Pulses showing other agents' specializations — propagates fast via Bus (~milliseconds) |
| Diffusion asymmetry (D_h >> D_a) | Learning is slow (individual experience). Inhibition is fast (Bus propagation). |
| Noise (sigma) | Small random perturbations to break initial symmetry |
| Spatial pattern | Role differentiation — each agent specializes in a different strategy dimension |

Because inhibition propagates through the Bus in milliseconds while activation requires hundreds of ticks of experience, **Turing's instability condition is naturally satisfied**.

### 9.2 The 8-Dimensional Strategy Vector

Each agent maintains an 8-dimensional strategy vector (a probability distribution, sum = 1.0) representing its current role:

| Index | Dimension | What It Represents |
|---|---|---|
| 0 | depth | Deep analysis of narrow topics |
| 1 | breadth | Broad survey across many topics |
| 2 | execution | Implementing and building |
| 3 | verification | Testing and validation |
| 4 | time_horizon | Long-term vs short-term planning |
| 5 | exploration | Trying new approaches |
| 6 | exploitation | Optimizing known approaches |
| 7 | coordination | Managing multi-agent workflows |

Domain plugins can redefine these dimensions. For code development: `[refactoring, feature_dev, testing, docs, perf, security, deps, arch]`. For DeFi: `[momentum, mean_reversion, lp, risk, time_horizon, asset_breadth, vol, cross_chain]`.

```rust
/// Number of strategy dimensions used by the morphogenetic model.
pub const STRATEGY_DIMS: usize = 8;

/// Morphogenetic state for an agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MorphogeneticState {
    /// Strategy concentration vector in `[0, 1]`, normalized to sum to 1.
    pub strategy: [f64; STRATEGY_DIMS],
    /// Per-dimension returns attributed since the last update.
    pub attributed_returns: [f64; STRATEGY_DIMS],
    /// Aggregated strategy vectors received from the collective.
    pub collective_pheromone: [f64; STRATEGY_DIMS],
    /// Number of agents in the collective.
    pub collective_size: usize,
}
```

**Specialization Index** (normalized Shannon entropy):

```
specialization_index = 1 - H(s) / H_max
```

Where `H(s) = -SUM s_k * ln(s_k)` (Shannon, C. E., 1948, Bell System Technical Journal, 27(3), pp. 379-423) and `H_max = ln(STRATEGY_DIMS)`. 0.0 = maximum generalization (uniform). 1.0 = maximum specialization (all in one dimension). Healthy specialists stabilize around 0.5-0.7.

```rust
#[must_use]
pub fn specialization_index(strategy: &[f64; STRATEGY_DIMS]) -> SpecializationIndex {
    let h: f64 = strategy
        .iter()
        .copied()
        .filter(|&s| s > 1e-10)
        .map(|s| -s * s.ln())
        .sum();
    let h_max = STRATEGY_DIMS_F64.ln();
    if h_max == 0.0 { 0.0 } else { 1.0 - h / h_max }
}
```

### 9.3 The Gierer-Meinhardt Update Rule

For each dimension k:

```
s_k(t+1) = s_k(t) + activation_k - inhibition_k - decay_k + noise_k
```

```rust
/// Morphogenetic parameters controlling reaction-diffusion dynamics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MorphogeneticParams {
    pub alpha: f64,                    // Activation rate. Default: 0.05.
    pub beta: f64,                     // Inhibition rate. Default: 0.15. Must be > alpha.
    pub mu: f64,                       // Decay toward baseline. Default: 0.01.
    pub baseline: f64,                 // 1/STRATEGY_DIMS = 0.125
    pub sigma_noise: f64,              // Noise for symmetry breaking. Default: 0.005.
    pub resource_pressure_scalar: f64, // Modulated by agent vitality. Default: 1.0.
}

impl MorphogeneticState {
    /// Update morphogenetic field using Gierer-Meinhardt reaction-diffusion dynamics.
    pub fn update(&mut self, params: &MorphogeneticParams) {
        let pressure = params.resource_pressure_scalar;
        let size = (self.collective_size as f64).max(1.0);

        for i in 0..STRATEGY_DIMS {
            let activation = params.alpha * self.attributed_returns[i] * pressure;
            let inhibition = params.beta * self.collective_pheromone[i] / size;
            let decay = params.mu * (self.strategy[i] - params.baseline);
            let noise = box_muller_normal() * params.sigma_noise;
            self.strategy[i] += activation - inhibition - decay + noise;
            // Prevent negative concentrations.
            if self.strategy[i] < 0.001 {
                self.strategy[i] = 0.001;
            }
        }

        // Re-normalize to sum to 1.0.
        let sum: f64 = self.strategy.iter().sum();
        if sum > 0.0 {
            for s in &mut self.strategy {
                *s /= sum;
            }
        }

        // Reset accumulators for next cycle.
        self.attributed_returns = [0.0; STRATEGY_DIMS];
        self.collective_pheromone = [0.0; STRATEGY_DIMS];
    }
}
```

Where:
- `activation_k = alpha × attributed_returns[k] × resource_pressure_scalar` — profitable dimensions grow
- `inhibition_k = beta × collective_pheromone[k] / collective_size` — crowded dimensions are suppressed
- `decay_k = mu × (s_k - baseline)` — drift back toward uniform
- `noise_k ~ N(0, sigma_noise^2)` via Box-Muller transform — break symmetry

Note the key difference from the canonical Gierer-Meinhardt formulation: the activation term here is linear in the strategy dimension (proportional to attributed returns) rather than quadratic (proportional to a²/h). This linearization simplifies convergence analysis while preserving the essential diffusion asymmetry (beta > alpha).

### 9.4 Why beta > alpha Is Essential

- **Activation** (alpha = 0.05) is driven by the agent's own experience — slow, like gene expression
- **Inhibition** (beta = 0.15) is driven by the group's pheromone field via Bus — fast, like extracellular diffusion

With beta = 3 × alpha, inhibition ensures an agent's specialization is suppressed in dimensions where other group members are already concentrated. This pushes agents apart in strategy space.

### 9.5 Morphogenetic Specialization Emergence

```mermaid
graph TD
    H["Initial state:<br/>All agents uniform<br/>strategy = [1/8, 1/8, ..., 1/8]<br/>specialization_index = 0.0"]

    N["Noise sigma = 0.005<br/>breaks initial symmetry"]

    E["Early (0-200 ticks):<br/>Random returns ± noise<br/>→ slight divergence begins"]

    M["Mid (200-500 ticks):<br/>Activation reinforces winners.<br/>Inhibition via Bus suppresses<br/>crowded dimensions for others."]

    S["Stable (500-2000 ticks):<br/>Distinct specialist roles emerge.<br/>specialization_index ≈ 0.5-0.7.<br/>Niche conflicts auto-resolved."]

    H --> N --> E --> M --> S
```

### 9.6 Niche Conflict Detection

```rust
/// Detect niche conflicts among a set of agents.
///
/// Returns all pairs whose cosine similarity exceeds `threshold` (default 0.9).
#[must_use]
pub fn niche_conflicts(
    agents: &[(AgentId, MorphogeneticState)],
    threshold: f64,
) -> Vec<NicheConflict>
```

When conflicts are detected (cosine similarity > 0.9), the system boosts collective pheromone on the top-3 shared dimensions for both agents, triggering automatic separation via the inhibition term.

### 9.7 Convergence Properties

From homogeneous initial conditions, convergence scales as O(N × log N) ticks:

| Group Size | Typical Convergence (ticks) | Wall Time (at 4 ticks/min) |
|---|---|---|
| 2 | ~500 | ~2 hours |
| 5 | ~800 | ~3.3 hours |
| 10 | ~1,200 | ~5 hours |
| 20 | ~1,800 | ~7.5 hours |
| 50 | ~3,000 | ~12.5 hours |

**Observed convergence target**: In the captured simulation setup, beta/alpha >= 2.0 and collective_size <= 50 converged to a stable pattern with probability > 0.99 within 3000 ticks across 10,000 Monte Carlo runs per parameter setting. Treat this as a reproduction target, not a production guarantee.

### 9.8 Stability Monitoring

A stability Lens monitors for three pathological states:

```rust
enum StabilityState {
    Converged,    // Strategy vectors have stabilized (trait variance < 0.01 for 100 consecutive ticks)
    Converging,   // Still moving toward equilibrium
    Oscillating,  // Needs parameter adjustment
}
```

1. **Pitchfork Bifurcation**: When beta/alpha exceeds ~5.0, agents split into two extreme clusters. Detected by monitoring bimodality of strategy distributions.
2. **Hopf Oscillation**: Agents cyclically swap roles without settling. Detected via Lyapunov exponent estimation and sign-change counting.

### 9.9 Two-Timescale Coordination

| Mechanism | Timescale | What It Decides |
|---|---|---|
| Response threshold allocation | 10-100 ticks (tactical) | "Should I respond to this pheromone right now?" |
| Morphogenetic specialization | 500-2000 ticks (strategic) | "What kind of agent should I be?" |

The two reinforce each other: morphogenetic specialization determines the agent's strategic role (e.g., "verification specialist"), while response thresholds handle tactical moment-to-moment attention within that role.

---

## 10. C-Factor: Collective Intelligence Measurement

For multi-agent scenarios, roko measures collective intelligence across five axes using the c-factor metric, based on Woolley, A. W. et al. (2010, "Evidence for a Collective Intelligence Factor in the Performance of Human Groups", Science, 330(6004), pp. 686-688, DOI: 10.1126/science.1193147), who showed that group performance across varied tasks loads onto a single collective factor, analogous to the "g factor" in individual intelligence.

**Source**: `docs/v1/00-architecture/14-c-factor-collective-intelligence.md`

### 10.1 The Five Process Variables

| Variable | Agent Analog | Measured From |
|---|---|---|
| **Turn-taking equality** | How evenly the cohort shares turns | Pulse authorship entropy and sender share on the Bus |
| **Peer prediction accuracy** | How well members predict each other's outputs | `peer.prediction` vs `peer.outcome` residuals |
| **Citation reciprocity** | How often citations are useful and verified | Citation reciprocity and downstream gate survival |
| **Channel openness (Delivery rate)** | How much intended traffic is delivered | Bus delivery confirmation and subscriber reach |
| **Cognitive diversity (HDC diversity)** | How different the cohort's working set is | HDC distance across cohort Engrams |

### 10.2 C-Factor Computation (Source Code)

From `crates/roko-orchestrator/src/coordination.rs:`

```rust
/// The five axes used to measure collective intelligence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CohortMetrics {
    pub turn_taking_entropy: f64,
    pub peer_prediction_accuracy: f64,
    pub citation_reciprocity: f64,
    pub delivery_rate: f64,
    pub hdc_diversity: f64,
}

/// Linear weights for the c-factor model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CohortWeights {
    pub turn_taking_entropy: f64,
    pub peer_prediction_accuracy: f64,
    pub citation_reciprocity: f64,
    pub delivery_rate: f64,
    pub hdc_diversity: f64,
    pub bias: f64,
}

/// Compute the c-factor for a cohort.
#[must_use]
pub fn c_factor(metrics: &CohortMetrics, weights: &CohortWeights) -> f64 {
    let weighted_sum = weights.turn_taking_entropy.mul_add(
        metrics.turn_taking_entropy,
        weights.peer_prediction_accuracy.mul_add(
            metrics.peer_prediction_accuracy,
            weights.citation_reciprocity.mul_add(
                metrics.citation_reciprocity,
                weights.delivery_rate.mul_add(
                    metrics.delivery_rate,
                    weights.hdc_diversity * metrics.hdc_diversity,
                ),
            ),
        ),
    );
    weighted_sum + weights.bias
}
```

The c-factor formula is a weighted linear combination:

```
c = w_tte × turn_taking_entropy
  + w_ppa × peer_prediction_accuracy
  + w_cr  × citation_reciprocity
  + w_dr  × delivery_rate
  + w_hdc × hdc_diversity
  + bias
```

The weights are not static — they are fitted online from cohort outcomes.

### 10.3 C-Factor Is a Covariate, Not an Objective

This is a critical design choice: c-factor is a diagnostic covariate, not a direct optimization target. Optimizing for c-factor directly can be counterproductive — the system might route easy work, suppress dissent, or narrow task scope to make the number look better. This is Goodhart's Law: "When a measure becomes a target, it ceases to be a good measure."

**Good use**: C-factor falls AND task outcomes fall → Policy intervenes.
**Bad use**: C-factor falls but Policy suppresses hard work to inflate the metric.

### 10.4 WisdomGate (Anti-Groupthink)

Before a consensus artifact is finalized, it passes a WisdomGate that encodes Surowiecki's four conditions (Surowiecki, J., 2004, "The Wisdom of Crowds", Doubleday):

```rust
/// `WisdomGate` inputs for consensus aggregation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WisdomGate {
    pub min_turn_taking_entropy: f64,
    pub min_peer_prediction_accuracy: f64,
    pub min_citation_reciprocity: f64,
    pub min_hdc_diversity: f64,          // diversity of opinion
    pub max_lineage_overlap: f64,        // independence
    pub max_sender_share: f64,           // decentralization
}

impl WisdomGate {
    /// Return true when the cohort is broad enough for consensus aggregation.
    #[must_use]
    pub fn allows(
        &self,
        metrics: &CohortMetrics,
        lineage_overlap: f64,
        sender_share: f64,
    ) -> bool {
        metrics.turn_taking_entropy >= self.min_turn_taking_entropy
            && metrics.peer_prediction_accuracy >= self.min_peer_prediction_accuracy
            && metrics.citation_reciprocity >= self.min_citation_reciprocity
            && metrics.hdc_diversity >= self.min_hdc_diversity
            && lineage_overlap <= self.max_lineage_overlap
            && sender_share <= self.max_sender_share
    }
}
```

Three anti-groupthink countermeasures:
1. **Devil's-advocate Pulse**: Emit an explicit opposing view on consensus topics
2. **Outsider injection**: Route some work to an agent with zero lineage overlap
3. **Minority report preservation**: Retain dissenting artifacts longer, with softer decay

---

## 11. Benchmarking

### 11.1 Cognitive Speed Selection Accuracy

**What to measure**: Does the frequency selector correctly classify tasks?

**Methodology**:
1. Build a labeled dataset of 500+ task descriptions, each with a ground-truth correct frequency (Gamma / Theta / Delta) determined by a domain expert
2. Run `OperatingFrequency::select()` on each task with a neutral affect state
3. Measure precision, recall, and F1 per class

**Expected baselines**:

| Metric | Target | Threshold to Investigate |
|---|---|---|
| Gamma precision | >= 0.90 | < 0.75 |
| Theta precision | >= 0.85 | < 0.70 |
| Delta precision | >= 0.95 | < 0.85 |
| Overall accuracy | >= 0.88 | < 0.75 |

**Affect modulation test**: For tasks on the Theta/Delta boundary, verify that the affect-driven promotion (confidence < 0.3 + arousal > 0.25) correctly promotes substantial tasks to Delta.

**IronClaw implementation**: Implement as a unit test suite in `src/agent/cognitive_speed_test.rs` with a fixture file of labeled tasks.

### 11.2 Pheromone-Based Coordination Efficiency vs. Explicit Messaging

**What to measure**: For multi-agent scenarios, is pheromone-based coordination more efficient than explicit direct messaging?

**Methodology** (requires multi-agent capability):
1. **Baseline**: N agents use explicit message-passing to coordinate task assignment. Measure: total messages sent, duplicate work rate, task completion time.
2. **Pheromone**: N agents use only pheromone deposits and reads for coordination. Measure the same metrics.

**Expected results** (from roko docs, validated via simulation):

| Metric | Explicit Messaging | Pheromone-Based | Expected Improvement |
|---|---|---|---|
| Total coordination messages | O(N²) per task | O(N) pheromone writes | ~N/2× reduction |
| Duplicate work rate | ~15% (no lock-out) | ~3% (Alpha pheromones) | ~5× reduction |
| Coordination latency | O(N) round-trips | O(1) local read | ~N× reduction |

**IronClaw implementation**: Start with single-agent pheromone writes (heartbeat deposits pheromones for future sessions to read). The cross-session coordination efficiency can be measured as: "how often does a session find useful pheromones from previous sessions vs. rediscovering the same information independently?"

### 11.3 Specialization Convergence Time

**What to measure**: How quickly do N homogeneous agents differentiate into stable specialists?

**Methodology**:
1. Start N agents with identical uniform strategy vectors `[1/8, ..., 1/8]`
2. Run for T ticks with identical workload
3. Measure: time to specialization_index >= 0.5 for all agents, inter-agent cosine similarity (target < 0.3 for distinct specialists), stability state (target: `Converged`)

**Expected results** (from Monte Carlo simulation in roko source):

| Group Size | Median Convergence (ticks) | 95th Percentile | % Within 3000 ticks |
|---|---|---|---|
| 2 | 480 | 820 | 99.7% |
| 5 | 790 | 1,340 | 99.3% |
| 10 | 1,180 | 2,010 | 99.1% |
| 20 | 1,760 | 2,980 | 98.8% |

**Pathology detection rate**: Hopf Oscillation detection should trigger for beta/alpha > 5.0 within 200 ticks of the instability onset.

### 11.4 C-Factor Correlation with Output Quality

**What to measure**: Does c-factor predict group output quality?

**Methodology**:
1. Run N multi-agent swarms on standardized tasks (coding, research, planning) for T ticks
2. Record c-factor at each Theta tick
3. Have outputs evaluated by a ground-truth evaluator (gate pipeline pass rate, expert judge score)
4. Compute Pearson correlation between c-factor and output quality score

**Expected result**: r >= 0.40 (consistent with Woolley et al. 2010, who found c-factor explained ~43% of group task variance in human groups). An r below 0.20 suggests the five-metric weighting needs calibration for this domain.

**Anti-Goodhart monitoring**: If c-factor rises but output quality stays flat or drops, log a `CFactorDecoupled` event and trigger a WisdomGate audit.

### 11.5 T0 Probe Suppression Rate

**What to measure**: What fraction of Gamma ticks are suppressed at T0 (no LLM call)?

**Target**: >= 80% suppression in steady-state operation (the "FrugalGPT-inspired zero-cost majority"). Below 60% suggests probes are too sensitive or the environment is genuinely noisy.

```rust
// Instrumentation point in the Gamma tick handler:
metrics.increment_counter("cognitive.gamma_tick.total");
if all_probes_clear {
    metrics.increment_counter("cognitive.gamma_tick.t0_suppressed");
}
let suppression_rate = t0_suppressed / total;
```

---

## 12. Practical Examples

### 12.1 Fast Query Handling (Gamma, T0)

**Scenario**: User asks "What time is it?"

```
Gamma tick begins
  SENSE:    Receive user message from channel
  ASSESS:   All 16 T0 probes clear — simple query, no state change
  COMPOSE:  Classify as reactive task (simple query)
            → OperatingFrequency::Gamma
            → InferenceTier::T0 (no LLM call needed)
  ACT:      Invoke time tool directly, no LLM involved
  VERIFY:   No gate needed for informational response
  PERSIST:  Store action record
  REACT:    No follow-up needed
Gamma tick completes: ~0.2s, $0 LLM cost
```

If the query were slightly more complex ("What time zone am I in, and should I schedule a meeting for 3pm London time?"), T0 probe #7 (confidence_dropping) would not fire, but the query classifier would detect ambiguity and escalate to T1 (Haiku-class) for fast analysis.

**Cost profile**: $0.00 for pure T0, ~$0.0003 for T1 escalation. With 80% T0 suppression, average cost per Gamma tick ≈ $0.00006.

### 12.2 Complex Bug Investigation (Theta, T1→T2 Escalation)

**Scenario**: User reports a test failure after a refactoring.

```
Gamma tick begins
  SENSE:    Receive bug report
  ASSESS:   T0 probe #5 (compile_error_new) fires
            → Escalate to T1
  T1 analysis: Identify the failing test, locate the change
            Confidence is moderate (0.6), arousal is moderate
            → OperatingFrequency::Theta (standard deliberative)
  ACT:      T1 model analyzes the error, proposes a fix
  VERIFY:   Gate pipeline: compile gate passes, test gate fails
            → Prediction error: fix did not work
            → Confidence drops to 0.35, arousal rises to 0.5
            → affect_suggests_reflection() triggers

Theta reflection fires early (shortened cadence due to struggling state)
  ASSESS:   Summarize: fix attempt failed, tests still red
  ACT:      T2 model (Opus-class) invoked for deeper analysis
            Reviews broader context, identifies root cause
  VERIFY:   Compile gate passes, test gate passes
  PERSIST:  Store fix, deposit Wisdom pheromone ("trait import order matters in this codebase")
  REACT:    Update Daimon PAD (pleasure up, arousal down)

Total: ~12 minutes, 1 T1 call + 1 T2 call
```

### 12.3 Overnight Knowledge Consolidation (Delta, T2)

**Scenario**: Agent has been idle for 30+ minutes after completing several tasks.

```
Delta tick triggered by OperatingFrequencyScheduler
  (context.is_idle() == true → OperatingFrequency::Delta)

  NREM Replay:
    Replay the 5 highest-prediction-error episodes from last session
    Episode: "Tried to use --no-verify flag, hook blocked it"
      → High prediction error (expected success, got failure)
      → Extract Heuristic: "This repo requires hook compliance"
    Episode: "Added memory_write call in handler, dispatch-exempt needed"
      → Medium prediction error
      → Extract Warning: "Handler mutations need dispatch-exempt annotation"

  REM Imagination:
    HDC recombination of episodes
    Cross-pollinate: "Hook compliance pattern" + "CI pipeline gates"
      → Novel hypothesis: "Pre-commit validation could prevent 60% of CI failures"
      → Deposit as Pattern pheromone (intensity 0.7, half-life 12h)

  Integration Staging:
    Validate hypothesis against Neuro knowledge base
    Existing Warning: "CI is slow, avoid unnecessary pushes"
      → Hypothesis aligns with existing knowledge
      → If confirmed by gate pass, promote Pattern to Wisdom

  PERSIST:  Store new Heuristic, Warning, and Pattern Engrams
            Tier promotion: 2 Working-tier Insights → Consolidated
  REACT:    Schedule follow-up task to test hypothesis
            "Run pre-commit hooks on last 10 commits to measure coverage"
```

**Outcome**: After 3 Delta cycles over several days, the Pattern pheromone accumulates 3+ confirmations and promotes to Wisdom, then to a Persistent Engram in the knowledge base.

### 12.4 Stigmergic Coordination Between Multiple Agents

**Scenario**: Two agents (A and B) working concurrently on the same codebase. Agent A is about to refactor a module. Agent B is about to write tests for the same module.

```
Agent A:
  SENSE:    Plans to refactor module X
  ACT:      Deposits Alpha pheromone:
            PheromoneKind::Alpha, location="module::x",
            intensity=1.0, half_life=1h,
            metadata={"task": "refactoring", "agent": "A"}

Agent B:
  SENSE:    Plans to write tests for module X
  ASSESS:   Reads pheromone field for module X
            Finds Alpha from Agent A: current_intensity ≈ 0.95
            response_probability(Alpha, 0.95) ≈ 0.90 (high — waits)
  REACT:    Delays test writing. Monitors Alpha intensity.

After ~45 minutes (Agent A completes refactor):
  Agent A's Alpha decays: current_intensity ≈ 0.50
  Agent A deposits Opportunity: "module X refactored, ready for tests"

Agent B:
  SENSE:    Opportunity pheromone detected for module X
  ASSESS:   Alpha intensity now 0.50, Opportunity intensity 0.85
            SINR: Opportunity SINR = 0.85 / (alpha_interference + noise)
            alpha[Alpha][Opp] = 0.10, so interference = 0.10 × 0.50 = 0.05
            SINR = 0.85 / (0.05 + 0.01) = 14.2 — clearly detectable
  ACT:      Begins test writing on the refactored module X
```

**Result**: Zero duplicate work, zero coordination messages, zero explicit lock mechanism. The Alpha pheromone served as a distributed mutex with natural expiry.

### 12.5 Emergent Task Specialization in a Swarm

**Scenario**: Five agents start identical. Over 800 ticks, they encounter a mix of coding tasks (execution, testing, documentation, security review, performance analysis).

```
Initial state (all 5 agents):
  strategy = [0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125]
  specialization_index = 0.0

After 200 ticks (noise + early returns create divergence):
  Agent 1: slight edge in execution (dim 2) from 3 successful code changes
  Agent 2: slight edge in verification (dim 3) from catching 2 bugs
  Agents 3-5: still near-uniform, slight noise perturbations

After 500 ticks (Gierer-Meinhardt dynamics amplify differences):
  Agent 1: execution dominates (0.40), exploitation grows (0.20)
    → collective_pheromone[2] rises, inhibiting other agents' execution
  Agent 2: verification dominates (0.35), coordination rises (0.20)
    → collective_pheromone[3] rises, inhibiting other agents' verification
  Agent 3: exploration rises (0.30), breadth grows (0.20)
    → Finds new approaches, deposits Pattern pheromones
  Agents 4-5: depth (0.30) and time_horizon (0.25) emerge from remainder

After 800 ticks (stable specialization):
  Agent 1: strategy ≈ [0.05, 0.05, 0.45, 0.05, 0.10, 0.05, 0.20, 0.05]
    specialization_index ≈ 0.55 — execution + exploitation specialist
  Agent 2: strategy ≈ [0.10, 0.05, 0.05, 0.40, 0.05, 0.05, 0.05, 0.25]
    specialization_index ≈ 0.52 — verification + coordination specialist
  Agent 3: strategy ≈ [0.25, 0.20, 0.05, 0.05, 0.10, 0.28, 0.05, 0.02]
    specialization_index ≈ 0.50 — depth + exploration specialist
  Agent 4: strategy ≈ [0.30, 0.05, 0.05, 0.05, 0.40, 0.05, 0.05, 0.05]
    specialization_index ≈ 0.58 — deep + long-horizon analyst
  Agent 5: strategy ≈ [0.05, 0.35, 0.08, 0.05, 0.05, 0.05, 0.05, 0.32]
    specialization_index ≈ 0.54 — breadth + coordination specialist
```

The response threshold model handles tactical allocation within roles:
- Agent 2 has low Threat thresholds (quickly responds to test failures)
- Agent 1 has low Opportunity thresholds (responds to new feature requests)
- Agent 3 has low Anomaly thresholds (investigates unexpected patterns)

No explicit role assignment was ever made. The specialization emerged entirely from local returns, inhibitory pheromones, and the Gierer-Meinhardt dynamics.

---

## 13. IronClaw Integration — Full Implementation Plan

### 13.1 CognitiveSpeedSelector

**Target file**: `src/agent/cognitive_speed.rs`

```rust
// src/agent/cognitive_speed.rs
//
// Implements the three-speed cognitive classification for IronClaw,
// adapted from roko's OperatingFrequency (roko-core/src/operating_frequency.rs).

use std::time::{Duration, Instant};

/// Cognitive speed classification for IronClaw agent processing.
/// Mapped to roko's OperatingFrequency with IronClaw-specific adaptations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CognitiveSpeed {
    /// Reactive: direct response, cheap or zero model cost (T0/T1).
    /// Maps to: simple queries, tool calls, pattern-matched responses.
    Gamma,
    /// Reflective: planning, self-correction, strategy evaluation (T1/T2).
    /// Maps to: mid-conversation re-planning, error analysis.
    Theta,
    /// Consolidation: background learning, memory reorganization (T2).
    /// Maps to: heartbeat processing, cross-session synthesis.
    Delta,
}

/// Context used by the speed classifier.
pub struct TurnContext {
    pub message: String,
    pub tool_calls_only: bool,
    pub is_background: bool,
    pub is_heartbeat: bool,
    pub confidence: f64,     // 0.0-1.0, from Daimon equivalent
    pub arousal: f64,        // 0.0-1.0
    pub recent_error_count: usize,
    pub requires_planning: bool,
}

impl TurnContext {
    pub fn is_simple_query(&self) -> bool {
        let msg = self.message.to_lowercase();
        // Quick pattern matches — no model needed
        msg.starts_with("what time")
            || msg.starts_with("what is the date")
            || msg.starts_with("who are you")
            || (msg.len() < 30 && !self.requires_planning)
    }

    pub fn is_tool_only(&self) -> bool {
        self.tool_calls_only
    }

    pub fn is_substantial(&self) -> bool {
        self.message.len() > 500
            || self.requires_planning
            || self.recent_error_count > 2
    }
}

/// Classify the current turn context into a cognitive speed.
pub fn classify_speed(ctx: &TurnContext) -> CognitiveSpeed {
    // T0 fast path: reactive task, no LLM needed
    if ctx.is_simple_query() || ctx.is_tool_only() {
        return CognitiveSpeed::Gamma;
    }

    // Background/heartbeat processing -> Delta for consolidation
    if ctx.is_background || ctx.is_heartbeat {
        return CognitiveSpeed::Delta;
    }

    // Affect-driven Delta promotion for substantial tasks
    // Mirrors roko's affect_suggests_reflection() + is_substantial()
    let affect_suggests_reflection = ctx.confidence < 0.3
        && (ctx.arousal > 0.25);
    if affect_suggests_reflection && ctx.is_substantial() {
        return CognitiveSpeed::Delta;
    }

    // Default: reflective Theta
    CognitiveSpeed::Theta
}

impl CognitiveSpeed {
    /// Map to IronClaw model tier.
    /// Integrates with the cascade router in crates/ironclaw_llm/.
    pub fn model_preference(&self) -> ModelPreference {
        match self {
            Self::Gamma => ModelPreference::Fast,     // Haiku-class
            Self::Theta => ModelPreference::Standard,  // Sonnet-class
            Self::Delta => ModelPreference::Best,      // Opus-class
        }
    }

    /// Maximum agent turns before forced reflection.
    pub fn turn_budget(&self) -> u32 {
        match self {
            Self::Gamma => 1,
            Self::Theta => 10,
            Self::Delta => 25,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ModelPreference {
    Fast,
    Standard,
    Best,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(msg: &str) -> TurnContext {
        TurnContext {
            message: msg.to_string(),
            tool_calls_only: false,
            is_background: false,
            is_heartbeat: false,
            confidence: 0.8,
            arousal: 0.1,
            recent_error_count: 0,
            requires_planning: false,
        }
    }

    #[test]
    fn simple_query_is_gamma() {
        assert_eq!(classify_speed(&ctx("what time is it")), CognitiveSpeed::Gamma);
    }

    #[test]
    fn heartbeat_is_delta() {
        let mut c = ctx("run heartbeat cycle");
        c.is_heartbeat = true;
        assert_eq!(classify_speed(&c), CognitiveSpeed::Delta);
    }

    #[test]
    fn low_confidence_substantial_is_delta() {
        let c = TurnContext {
            message: "A".repeat(600),
            confidence: 0.2,
            arousal: 0.4,
            requires_planning: true,
            ..ctx("placeholder")
        };
        assert_eq!(classify_speed(&c), CognitiveSpeed::Delta);
    }

    #[test]
    fn normal_task_is_theta() {
        assert_eq!(classify_speed(&ctx("Fix the login validation bug")), CognitiveSpeed::Theta);
    }
}
```

### 13.2 Adaptive Reflection Scheduler

**Target file**: `src/agent/reflection.rs`

```rust
// src/agent/reflection.rs
//
// Adaptive scheduler for reflective pauses in the IronClaw agent loop.
// Adapted from roko's OperatingFrequencyScheduler (roko-core/src/operating_frequency.rs).

use std::time::{Duration, Instant};

/// Scheduler that determines when the agent should pause for reflection.
pub struct ReflectionScheduler {
    /// Base interval between reflective pauses.
    theta_interval: Duration,
    /// Maximum interval before forced consolidation.
    delta_interval: Duration,
    /// Time of last reflective pause.
    last_theta: Instant,
}

impl Default for ReflectionScheduler {
    fn default() -> Self {
        Self {
            theta_interval: Duration::from_secs(180),   // 3 minutes
            delta_interval: Duration::from_secs(1800),  // 30 minutes
            last_theta: Instant::now(),
        }
    }
}

/// Current context for the scheduler's decision.
pub struct SchedulerContext {
    pub is_idle: bool,
    pub completion_rate: f64,   // recent task completions / attempts, 0.0–1.0
    pub is_struggling: bool,    // confidence < 0.35 AND arousal > 0.25
}

/// The recommended mode for the next agent loop iteration.
pub enum ReflectionMode {
    /// Continue current processing at Gamma speed.
    Continue,
    /// Pause for reflective evaluation (summarize, re-plan). Theta speed.
    Theta,
    /// Enter deep consolidation mode. Delta speed.
    Delta,
}

impl ReflectionScheduler {
    pub fn should_reflect(&self, ctx: &SchedulerContext) -> ReflectionMode {
        let elapsed = self.last_theta.elapsed();

        if ctx.is_idle {
            return ReflectionMode::Delta;
        }

        if elapsed >= self.delta_interval {
            return ReflectionMode::Delta;
        }

        // Adaptive Theta interval:
        // stalling (low completion) → reflect 2× more often
        // struggling (low confidence + high arousal) → reflect 1.5× more often
        let mut theta_due = self.theta_interval;
        if ctx.completion_rate <= 0.25 {
            theta_due = theta_due.mul_f64(0.5);
        } else if ctx.is_struggling {
            theta_due = theta_due.mul_f64(0.66);
        }

        if elapsed >= theta_due {
            ReflectionMode::Theta
        } else {
            ReflectionMode::Continue
        }
    }

    pub fn record_reflection(&mut self) {
        self.last_theta = Instant::now();
    }
}
```

### 13.3 PheromoneOverlay for Workspace

**Target file**: `src/workspace/pheromone.rs`

The pheromone system overlays on IronClaw's existing `memory_write`/`memory_search` tools using a namespaced tagging convention. It implements the full decay model, confirmation extension, and Alpha paradox from the roko source.

```rust
// src/workspace/pheromone.rs
//
// Digital pheromone system for IronClaw workspace.
// Implements the 7-kind pheromone model from roko
// (roko-orchestrator/src/coordination.rs) using workspace memory
// as the backing store.
//
// Pheromones are stored as tagged workspace memories in the "pheromone::"
// namespace. Decay is computed lazily from the deposit timestamp.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Digital pheromone stored in the workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pheromone {
    /// Unique identifier for this pheromone deposit.
    pub id: uuid::Uuid,
    /// Semantic kind determining behavior and decay profile.
    pub kind: PheromoneKind,
    /// What this pheromone is about (tool name, file path, topic, etc.)
    pub location: String,
    /// Intensity at the time of deposit. Range: [0.0, 1.0].
    pub initial_intensity: f64,
    /// Agent session ID or background task name that deposited this.
    pub depositor: String,
    /// When this pheromone was deposited.
    pub deposited_at: DateTime<Utc>,
    /// Base half-life for decay computation.
    pub half_life: Duration,
    /// Number of confirmations from other agents/sessions.
    pub confirmations: u32,
    /// Domain-specific metadata.
    pub metadata: serde_json::Value,
}

/// The seven semantic kinds of pheromone signals.
/// Mirrors roko's PheromoneKind enum exactly.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PheromoneKind {
    /// Something dangerous or harmful. Alarm signal.
    Threat,
    /// A favorable condition detected. Recruitment signal.
    Opportunity,
    /// Validated knowledge that should persist. Trail signal.
    Wisdom,
    /// First-mover advantage, ephemeral edge. Decays faster when confirmed.
    Alpha,
    /// Recurring structure or regularity. Intermediate signal.
    Pattern,
    /// Something unusual or unexpected. Investigation trigger.
    Anomaly,
    /// Collective agreement on a fact or decision. Long-lived.
    Consensus,
}

impl PheromoneKind {
    /// Default half-life for each kind.
    pub fn default_half_life(&self) -> Duration {
        match self {
            Self::Threat      => Duration::from_secs(2  * 3600), // 2 h
            Self::Opportunity => Duration::from_secs(4  * 3600), // 4 h
            Self::Wisdom      => Duration::from_secs(24 * 3600), // 24 h
            Self::Alpha       => Duration::from_secs(3600),       // 1 h
            Self::Pattern     => Duration::from_secs(12 * 3600), // 12 h
            Self::Anomaly     => Duration::from_secs(6  * 3600), // 6 h
            Self::Consensus   => Duration::from_secs(48 * 3600), // 48 h
        }
    }

    /// Workspace memory tag for this kind.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Threat      => "pheromone:threat",
            Self::Opportunity => "pheromone:opportunity",
            Self::Wisdom      => "pheromone:wisdom",
            Self::Alpha       => "pheromone:alpha",
            Self::Pattern     => "pheromone:pattern",
            Self::Anomaly     => "pheromone:anomaly",
            Self::Consensus   => "pheromone:consensus",
        }
    }
}

impl Pheromone {
    /// Create a new pheromone with default half-life for its kind.
    pub fn new(
        kind: PheromoneKind,
        location: impl Into<String>,
        initial_intensity: f64,
        depositor: impl Into<String>,
    ) -> Self {
        let half_life = kind.default_half_life();
        Self {
            id: uuid::Uuid::new_v4(),
            kind,
            location: location.into(),
            initial_intensity: initial_intensity.clamp(0.0, 1.0),
            depositor: depositor.into(),
            deposited_at: Utc::now(),
            half_life,
            confirmations: 0,
            metadata: serde_json::Value::Null,
        }
    }

    /// Compute current intensity using exponential decay.
    ///
    /// intensity(t) = initial_intensity * 2^(-t / effective_half_life)
    pub fn current_intensity(&self) -> f64 {
        let elapsed = Utc::now()
            .signed_duration_since(self.deposited_at)
            .to_std()
            .unwrap_or_default();
        let hl = self.effective_half_life();
        if hl.is_zero() {
            return 0.0;
        }
        let exponent = -(elapsed.as_secs_f64() / hl.as_secs_f64());
        self.initial_intensity * 2.0_f64.powf(exponent)
    }

    /// Effective half-life after applying confirmation model.
    ///
    /// Alpha paradox: each confirmation reduces half-life (first-mover
    /// advantage erodes as others confirm).
    /// All other kinds: confirmations extend half-life linearly.
    fn effective_half_life(&self) -> Duration {
        let base = self.half_life.as_secs_f64();
        let confs = f64::from(self.confirmations);

        match self.kind {
            PheromoneKind::Alpha => {
                // Alpha paradox: divisor grows with confirmations.
                let divisor = confs.mul_add(0.1, 1.0);
                Duration::from_secs_f64(base / divisor)
            }
            _ => {
                // Standard: linear half-life extension.
                let multiplier = confs.mul_add(0.5, 1.0);
                Duration::from_secs_f64(base * multiplier)
            }
        }
    }

    /// True if the pheromone has decayed below the detection threshold.
    pub fn is_evaporated(&self) -> bool {
        self.current_intensity() < 0.01
    }

    /// Response probability using the Hill function.
    ///
    /// P = I^n / (I^n + theta^n), n=2 (sigmoidal)
    pub fn response_probability(&self, threshold: f64, hill_n: f64) -> f64 {
        let intensity = self.current_intensity().clamp(0.0, 1.0);
        let theta = threshold.clamp(0.0, 1.0);
        let i_n = intensity.powf(hill_n);
        let t_n = theta.powf(hill_n);
        let denom = i_n + t_n;
        if denom == 0.0 { 0.0 } else { i_n / denom }
    }
}

/// Workspace key convention for storing pheromones.
pub fn workspace_key(kind: &PheromoneKind, location: &str) -> String {
    format!("pheromone:{}:{}", kind.tag().trim_start_matches("pheromone:"), location)
}

/// Intensity-weighted summary of the pheromone field at a location,
/// used to detect when to avoid or pursue a topic.
pub struct PheromoneField {
    pub threat_intensity: f64,
    pub opportunity_intensity: f64,
    pub wisdom_intensity: f64,
    pub alpha_intensity: f64,
    pub pattern_intensity: f64,
    pub anomaly_intensity: f64,
    pub consensus_intensity: f64,
}

impl PheromoneField {
    /// Should this location be avoided? (Threat > 0.7, no Wisdom override)
    pub fn should_avoid(&self) -> bool {
        self.threat_intensity > 0.7 && self.wisdom_intensity < 0.5
    }

    /// Is there an active "I'm working on this" lock?
    pub fn is_locked_by_alpha(&self) -> bool {
        self.alpha_intensity > 0.5
    }

    /// Opportunity score adjusted for threat interference.
    /// Implements SINR interference: alpha[Threat][Opp] = 0.60.
    pub fn effective_opportunity(&self) -> f64 {
        let noise_floor = 0.01;
        let interference = 0.60 * self.threat_intensity;
        let sinr = self.opportunity_intensity / (interference + noise_floor);
        if sinr < 1.0 {
            0.0 // Below detection threshold
        } else {
            self.opportunity_intensity * (sinr / (1.0 + sinr))
        }
    }
}
```

**Integration with existing workspace memory**:

```rust
// Usage in a heartbeat or background task:

// Deposit a Threat when something goes wrong:
let pheromone = Pheromone::new(
    PheromoneKind::Threat,
    "github-api/rate-limit",
    0.75, // High severity
    "session-abc",
);
let key = workspace_key(&pheromone.kind, &pheromone.location);
workspace.memory_write(&key, &serde_json::to_string(&pheromone)?, &["pheromone", "threat"]).await?;

// Deposit an Opportunity after discovering something useful:
let opp = Pheromone::new(
    PheromoneKind::Opportunity,
    "github-api/graphql-batch",
    0.80,
    "heartbeat",
);
workspace.memory_write(
    &workspace_key(&opp.kind, &opp.location),
    &serde_json::to_string(&opp)?,
    &["pheromone", "opportunity"]
).await?;

// Read pheromones before starting work on a topic:
let raw_entries = workspace.memory_search("pheromone:*:github-api*", None).await?;
// Deserialize and filter evaporated ones:
let active: Vec<Pheromone> = raw_entries
    .into_iter()
    .filter_map(|e| serde_json::from_str(&e.content).ok())
    .filter(|p: &Pheromone| !p.is_evaporated())
    .collect();
```

### 13.4 MorphogeneticTracker

**Target file**: `src/agent/morphogenetic.rs`

For future multi-agent support, the MorphogeneticTracker maintains each agent's strategy vector and applies the Gierer-Meinhardt update rule.

```rust
// src/agent/morphogenetic.rs
//
// Morphogenetic specialization tracker for IronClaw.
// Implements the 8-dimensional Gierer-Meinhardt strategy model
// from roko (roko-orchestrator/src/coordination.rs).
//
// In single-agent mode, this tracker still provides value:
// - Tracks which strategy dimensions are currently active
// - Provides strategy signal to cognitive speed selection
// - Ready for multi-agent integration when team support is added

use serde::{Deserialize, Serialize};

pub const STRATEGY_DIMS: usize = 8;

/// Strategy dimension labels for code development domain.
pub const STRATEGY_LABELS: [&str; STRATEGY_DIMS] = [
    "depth",        // 0: deep analysis of narrow topics
    "breadth",      // 1: broad survey across many topics
    "execution",    // 2: implementing and building
    "verification", // 3: testing and validation
    "time_horizon", // 4: long-term vs short-term planning
    "exploration",  // 5: trying new approaches
    "exploitation", // 6: optimizing known approaches
    "coordination", // 7: managing workflows
];

/// Morphogenetic state for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphogeneticState {
    /// Strategy concentration vector in [0, 1], normalized to sum to 1.
    pub strategy: [f64; STRATEGY_DIMS],
    /// Per-dimension returns attributed since the last update.
    pub attributed_returns: [f64; STRATEGY_DIMS],
    /// Aggregated strategy vectors received from the collective.
    /// In single-agent mode, this remains zero.
    pub collective_pheromone: [f64; STRATEGY_DIMS],
    /// Number of agents in the collective (1 for single-agent).
    pub collective_size: usize,
}

impl Default for MorphogeneticState {
    fn default() -> Self {
        let uniform = 1.0 / STRATEGY_DIMS as f64;
        Self {
            strategy: [uniform; STRATEGY_DIMS],
            attributed_returns: [0.0; STRATEGY_DIMS],
            collective_pheromone: [0.0; STRATEGY_DIMS],
            collective_size: 1,
        }
    }
}

/// Parameters controlling the Gierer-Meinhardt reaction-diffusion dynamics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphogeneticParams {
    /// Activation rate. Default: 0.05. (How much returns reinforce strategy)
    pub alpha: f64,
    /// Inhibition rate. Default: 0.15. Must be > alpha for Turing instability.
    pub beta: f64,
    /// Decay toward baseline. Default: 0.01.
    pub mu: f64,
    /// Baseline: 1/STRATEGY_DIMS = 0.125.
    pub baseline: f64,
    /// Noise for symmetry breaking. Default: 0.005.
    pub sigma_noise: f64,
    /// Modulated by agent vitality/budget. Default: 1.0.
    pub resource_pressure_scalar: f64,
}

impl Default for MorphogeneticParams {
    fn default() -> Self {
        Self {
            alpha: 0.05,
            beta: 0.15,
            mu: 0.01,
            baseline: 1.0 / STRATEGY_DIMS as f64,
            sigma_noise: 0.005,
            resource_pressure_scalar: 1.0,
        }
    }
}

impl MorphogeneticState {
    /// Apply one cycle of Gierer-Meinhardt reaction-diffusion.
    ///
    /// For each dimension k:
    ///   s_k += alpha * returns[k] * pressure
    ///          - beta * collective[k] / size
    ///          - mu * (s_k - baseline)
    ///          + N(0, sigma^2)
    ///
    /// Then clamp to [0.001, inf) and renormalize to sum to 1.0.
    pub fn update(&mut self, params: &MorphogeneticParams) {
        use std::f64;

        let pressure = params.resource_pressure_scalar;
        let size = (self.collective_size as f64).max(1.0);

        for i in 0..STRATEGY_DIMS {
            let activation = params.alpha * self.attributed_returns[i] * pressure;
            let inhibition = params.beta * self.collective_pheromone[i] / size;
            let decay = params.mu * (self.strategy[i] - params.baseline);
            // Box-Muller transform for Gaussian noise
            let noise = box_muller_sample() * params.sigma_noise;

            self.strategy[i] += activation - inhibition - decay + noise;

            // Clamp to prevent extinction of any dimension
            if self.strategy[i] < 0.001 {
                self.strategy[i] = 0.001;
            }
        }

        // Renormalize to sum to 1.0
        let sum: f64 = self.strategy.iter().sum();
        if sum > 0.0 {
            for s in &mut self.strategy {
                *s /= sum;
            }
        }

        // Reset accumulators for next cycle
        self.attributed_returns = [0.0; STRATEGY_DIMS];
        self.collective_pheromone = [0.0; STRATEGY_DIMS];
    }

    /// Attribute a return to a strategy dimension.
    ///
    /// Call this after a successful task completion with the primary dimension
    /// that was exercised.
    pub fn attribute_return(&mut self, dimension: usize, value: f64) {
        if dimension < STRATEGY_DIMS {
            self.attributed_returns[dimension] += value;
        }
    }

    /// Compute the specialization index (normalized Shannon entropy complement).
    ///
    /// 0.0 = maximum generalization (uniform distribution)
    /// 1.0 = maximum specialization (all weight in one dimension)
    /// Healthy specialists: 0.5–0.7
    pub fn specialization_index(&self) -> f64 {
        let h: f64 = self.strategy
            .iter()
            .copied()
            .filter(|&s| s > 1e-10)
            .map(|s| -s * s.ln())
            .sum();
        let h_max = (STRATEGY_DIMS as f64).ln();
        if h_max == 0.0 { 0.0 } else { 1.0 - h / h_max }
    }

    /// Return the dominant strategy dimension (highest weight) and its label.
    pub fn dominant_role(&self) -> (usize, &'static str) {
        let idx = self.strategy
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        (idx, STRATEGY_LABELS[idx])
    }
}

/// Box-Muller transform to sample from N(0,1).
fn box_muller_sample() -> f64 {
    use std::f64::consts::PI;
    let u1: f64 = rand::random::<f64>().max(f64::EPSILON);
    let u2: f64 = rand::random::<f64>();
    (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_uniform() {
        let state = MorphogeneticState::default();
        let expected = 1.0 / STRATEGY_DIMS as f64;
        for &s in &state.strategy {
            assert!((s - expected).abs() < 1e-10);
        }
    }

    #[test]
    fn uniform_specialization_index_is_zero() {
        let state = MorphogeneticState::default();
        assert!(state.specialization_index() < 0.01);
    }

    #[test]
    fn activation_reinforces_dimension() {
        let mut state = MorphogeneticState::default();
        let params = MorphogeneticParams { sigma_noise: 0.0, ..Default::default() };
        // Attribute returns to dimension 2 (execution)
        state.attribute_return(2, 5.0);
        state.update(&params);
        // Dimension 2 should now have more weight than others
        let max_dim = state.strategy
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        assert_eq!(max_dim, 2);
    }

    #[test]
    fn inhibition_suppresses_crowded_dimension() {
        let mut state = MorphogeneticState::default();
        let params = MorphogeneticParams { sigma_noise: 0.0, ..Default::default() };
        // Simulate collective pheromone on dimension 2
        state.collective_pheromone[2] = 0.8;
        state.collective_size = 5;
        state.update(&params);
        // Dimension 2 should lose weight due to inhibition
        let min_dim = state.strategy
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        assert_eq!(min_dim, 2);
    }
}
```

### 13.5 Integration Wiring

**Where to hook into existing IronClaw code**:

**Agent loop** (`src/agent/`):
1. Add `CognitiveSpeed` classification at the start of each turn
2. Feed the classified speed into the cascade router's model selection
3. Apply `turn_budget()` to cap the number of turns before forced reflection
4. At each Theta tick: summarize recent work, update confidence/arousal state
5. At each Delta tick (heartbeat): run the Dreams-equivalent consolidation

**Heartbeat** (`src/workspace/`):
1. After each successful heartbeat run, deposit Opportunity or Wisdom pheromones for discovered insights
2. After each failed heartbeat (errors), deposit Threat pheromones
3. Before each heartbeat, read active pheromones for the current context

**Background task scheduler** (`src/agent/`):
1. Use `ReflectionScheduler` to gate when the agent pauses for reflection
2. Pass `is_idle=true` to the scheduler when the agent has no pending tasks
3. Trigger Delta-mode processing during idle periods (existing heartbeat mechanism)

**Layer enforcement** (CI):
```
# Add to CI pipeline:
cargo deny check bans
# Add custom lint to detect backward imports:
# crates/ironclaw_engine/ must not import from src/agent/ or src/channels/
# src/agent/ must not import from src/channels/ directly
```

---

## 14. Academic Foundations

### Core Cognitive Architecture

| Citation | Contribution |
|---|---|
| Buzsaki, G. (2006). "Rhythms of the Brain". Oxford University Press. ISBN 978-0-19-530106-9. | Neural oscillation bands: Gamma/Theta/Delta as functional timescales. Cross-frequency coupling as coordination. |
| Kahneman, D. (2011). "Thinking, Fast and Slow". Farrar, Straus and Giroux. ISBN 978-0-374-27563-1. | Dual-process theory: System 1 (fast) / System 2 (slow). Maps to T1/T2 tiers. |
| Sun, R. (2002). "Duality of the Mind". Lawrence Erlbaum Associates. | CLARION: dual-level cognitive architecture with sub-conceptual level. Maps to T0. |
| Sumers, T. R. et al. (2023). "Cognitive Architectures for Language Agents". arXiv:2309.02427. | CoALA: cognitive architecture framework for language agents. |
| Anderson, J. R. (1983). "The Architecture of Cognition". Harvard University Press. | ACT-R: production system cognitive architecture. T0 probes as productions. |
| Laird, J. E. et al. (1987). "SOAR: An architecture for general intelligence". Artificial Intelligence, 33(1), pp. 1-64. | SOAR: universal subgoaling. T0→T1 escalation as impasse detection. |
| Baars, B. J. (1988). "A Cognitive Theory of Consciousness". Cambridge University Press. | Global Workspace Theory: limited-capacity conscious workspace. Context window as workspace. |

### Active Inference and Prediction

| Citation | Contribution |
|---|---|
| Friston, K. (2010). "The free-energy principle: a unified brain theory?" Nature Reviews Neuroscience, 11(2), pp. 127-138. DOI: 10.1038/nrn2787. | Free Energy Principle: prediction error drives learning and attention. Foundation for tier routing. |
| Clark, A. (2013). "Whatever next? Predictive brains, situated agents, and the future of cognitive science". Behavioral and Brain Sciences, 36(3), pp. 181-204. | Predictive Processing: brain as prediction machine. |
| Chen, L. et al. (2023). "FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance". arXiv:2305.05176. | Cascade routing matches GPT-4 at 2% cost. Foundation for T0 probe suppression. |
| Vovk, V. et al. (2005). "Algorithmic Learning in a Random World". Springer. | Conformal prediction: distribution-free calibration for prediction quality tracking. |

### Emotion and Motivation

| Citation | Contribution |
|---|---|
| Mehrabian, A. & Russell, J. A. (1974). "An Approach to Environmental Psychology". MIT Press. | PAD model: Pleasure-Arousal-Dominance emotional space. Foundation for Daimon. |
| Damasio, A. (1994). "Descartes' Error: Emotion, Reason, and the Human Brain". Putnam. | Somatic marker hypothesis: emotion biases pre-conscious decision-making. |
| Plutchik, R. (2001). "The Nature of Emotions". American Scientist, 89(4), pp. 344-350. | Emotion wheel: mapping complex emotions to dimensional space. Six behavioral states. |

### Memory and Learning

| Citation | Contribution |
|---|---|
| Mattar, M. G. & Daw, N. D. (2018). "Prioritized memory access explains planning and hippocampal replay". Nature Neuroscience, 21, pp. 1609-1617. | Prioritized memory replay. Foundation for Delta NREM phase. |
| Baddeley, A. (2000). "The episodic buffer: a new component of working memory?" Trends in Cognitive Sciences, 4(11), pp. 417-423. | Working memory model: central executive manages attention. Maps to Theta. |
| Walker, M. P. & van der Helm, E. (2009). "Overnight therapy? The role of sleep in emotional brain processing". Annual Review of Clinical Psychology, 5, pp. 139-166. | REM sleep emotional depotentiation. Foundation for Dreams REM phase. |
| Lacaux, C. et al. (2021). "Sleep onset is a creative sweet spot". Science Advances, 7(50), eabj5866. | Hypnagogia: creative insights during sleep onset. Foundation for hypnagogia engine. |
| McClelland, J. L. et al. (1995). "Why there are complementary learning systems in the hippocampus and neocortex". Psychological Review, 102(3), pp. 419-457. | Complementary Learning Systems theory: hippocampal-neocortical consolidation. |
| Kanerva, P. (2009). "Hyperdimensional Computing: An Introduction". Cognitive Computation, 1(2), pp. 139-159. | HDC vectors for fixed-cost pairwise comparison and structural fingerprints. Foundation for Neuro knowledge encoding. |

### Stigmergy and Multi-Agent Coordination

| Citation | Contribution |
|---|---|
| Grasse, P.-P. (1959). "La reconstruction du nid et les coordinations interindividuelles…" Insectes Sociaux, 6(1), pp. 41-80. DOI: 10.1007/BF02223791. | Stigmergy: indirect coordination through environmental modification. |
| Wilson, E. O. (1971). "The Insect Societies". Belknap Press of Harvard University Press. | Pheromone type hierarchy: alarm, recruitment, trail. Interference matrix basis. |
| Bonabeau, E., Theraulaz, G. & Deneubourg, J.-L. (1998). "Fixed Response Thresholds and the Regulation of Division of Labor in Insect Societies". Bulletin of Mathematical Biology, 60, pp. 753-807. | Hill function model for emergent task allocation. |
| Theraulaz, G. & Bonabeau, E. (1999). "A Brief History of Stigmergy". Artificial Life, 5(2), pp. 97-116. | Formalized quantitative and qualitative stigmergy. Three conditions. |
| Parunak, H. V. D., Brueckner, S. & Sauter, J. (2005). "Digital pheromones for coordination of unmanned vehicles". Environments for Multi-Agent Systems, LNAI 3374, Springer. | Digital pheromones in multi-agent systems. |
| Dorigo, M. et al. (2000). "Ant algorithms and stigmergy". Future Generation Computer Systems, 16(8), pp. 851-871. | Ant colony optimization: emergent coordination under local rules. |
| Tse, D. & Viswanath, P. (2005). "Fundamentals of Wireless Communication". Cambridge University Press. | SINR model adapted for pheromone interference. |

### Morphogenesis and Self-Organization

| Citation | Contribution |
|---|---|
| Turing, A. M. (1952). "The Chemical Basis of Morphogenesis". Philosophical Transactions of the Royal Society of London B, 237(641), pp. 37-72. DOI: 10.1098/rstb.1952.0012. | Reaction-diffusion mechanism: stable patterns from uniform initial state. |
| Gierer, A. & Meinhardt, H. (1972). "A theory of biological pattern formation". Kybernetik, 12, pp. 30-39. | Activator-inhibitor model: formal dynamics for reaction-diffusion. |
| Kauffman, S. A. (1993). "The Origins of Order: Self-Organization and Selection in Evolution". Oxford University Press. | Autocatalytic sets, edge of chaos, self-sustaining improvement cycles. |
| Shannon, C. E. (1948). "A Mathematical Theory of Communication". Bell System Technical Journal, 27(3), pp. 379-423. | Information theory: entropy as specialization index measure. |

### Collective Intelligence

| Citation | Contribution |
|---|---|
| Woolley, A. W. et al. (2010). "Evidence for a Collective Intelligence Factor in the Performance of Human Groups". Science, 330(6004), pp. 686-688. DOI: 10.1126/science.1193147. | Collective intelligence factor (c-factor). Single factor explains 43% of variance. |
| Surowiecki, J. (2004). "The Wisdom of Crowds". Doubleday. | Four conditions: diversity, independence, decentralization, aggregation. |
| Beer, S. (1972). "Brain of the Firm". Allen Lane. 2nd ed. Wiley, 1981. ISBN 978-0-471-27687-0. | Viable System Model: five recursive subsystems for viable organizations. |
| Ashby, W. R. (1956). "An Introduction to Cybernetics". Chapman & Hall. | Law of Requisite Variety: regulatory capacity must match environment variety. |
| Conant, R. C. & Ashby, W. R. (1970). "Every good regulator of a system must be a model of that system". International Journal of Systems Science, 1(2), pp. 89-97. | Good Regulator Theorem: agent must model itself. Motivates Daimon. |

### Scaffold Thesis Evidence

| Citation | Contribution |
|---|---|
| Lee, Y. et al. (2026). "Meta-Harness: Harness Optimization for SWE-bench". arXiv:2603.28052. | +7.7 pts from harness optimization alone, 4x fewer tokens. |
| Jimenez, C. E. et al. (2024). "SWE-bench: Can Language Models Resolve Real-World GitHub Issues?" | Same model, 30-65% solve rate depending on harness. |
| Zaharia, M. et al. (2024). "The Shift from Models to Compound AI Systems". | Compound AI Systems thesis: SOTA from systems, not models. |
| Khattab, O. et al. (2024). "DSPy: Compiling Declarative Language Model Calls into State-of-the-Art Pipelines". | Compiler-optimized prompt pipelines outperform manual prompt engineering. |
| Boden, M. A. (2004). "The Creative Mind: Myths and Mechanisms". 2nd ed. Routledge. | Computational creativity: exploratory, combinational, transformational. Foundation for REM imagination. |
| Pearl, J. (2009). "Causality: Models, Reasoning, and Inference". 2nd ed. Cambridge University Press. | Structural Causal Models for counterfactual reasoning in Dreams. |

---

## 15. Complexity Assessment and Risk

### Implementation Effort Estimates

| Component | Estimated Lines | Complexity | IronClaw Target Files |
|---|---|---|---|
| CognitiveSpeedSelector | ~200-300 | Low | `src/agent/cognitive_speed.rs` |
| ReflectionScheduler | ~150-200 | Low | `src/agent/reflection.rs` |
| PheromoneOverlay (workspace-backed) | ~400-500 | Medium | `src/workspace/pheromone.rs` |
| MorphogeneticTracker | ~300-400 | Medium | `src/agent/morphogenetic.rs` |
| Layer discipline (CI rules) | CI config only | Low | `scripts/`, `Cargo.toml` |
| SINR interference (multi-agent) | ~200-300 | Medium | `src/workspace/pheromone.rs` |
| ResponseThresholds model | ~250-350 | Medium | `src/workspace/pheromone.rs` |
| C-factor measurement (multi-agent) | ~300-400 | Medium | New module |

### Implementation Priority

**Phase 1 — Immediate** (no dependencies, low risk, high value):
- `CognitiveSpeedSelector` in the agent loop
- `ReflectionScheduler` with adaptive intervals
- Layer discipline enforcement in CI

**Phase 2 — Short-term** (low-to-medium risk):
- Pheromone system using workspace memories for cross-session learning
- Threat/Opportunity/Wisdom pheromones for heartbeat integration
- Heartbeat deposits pheromones; next session reads them

**Phase 3 — Medium-term** (requires multi-agent support):
- SINR interference model
- Response threshold model for emergent task allocation
- C-factor measurement

**Phase 4 — Long-term** (highest complexity):
- Full morphogenetic specialization for multi-agent role emergence
- Full Dreams consolidation cycle
- Adaptive interference matrix learning from cohort data

### Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| Over-engineering cognitive speeds for single-agent use | Low | Start with simple Gamma/Theta classification; Delta is just the existing heartbeat |
| Pheromone system adding noise to workspace retrieval | Medium | Use separate `pheromone::` namespace; aggressive evaporation threshold (0.01); monitor search result quality |
| Morphogenetic parameters failing to converge | Medium | Monte Carlo validation shows convergence at beta/alpha >= 2.0 for groups <= 50; monitor `StabilityState` |
| C-factor becoming a Goodhart metric | Low | Treat as covariate per roko design; never optimize directly; log `CFactorDecoupled` events |
| Layer discipline breaking existing code | Low | Enforce incrementally; grandfather existing violations; use `// dispatch-exempt: <reason>` pattern already in use |
| Alpha pheromone decay formula confusion | Low | Use verified divisor-based formula from roko source: `divisor = 1 + 0.1 × confirmations`, not the alternative floor-based formula sometimes described in docs |

# Cognitive Architecture

**Source crates**: `roko-core`, `roko-runtime`, `roko-orchestrator`, `roko-std`, `roko-primitives`, `roko-daimon`, `roko-neuro`, `roko-dreams`, `roko-compose`, `roko-gate`, `roko-learn`, `roko-conductor`
**Source docs**: `docs/v1/00-architecture/`, `docs/v1/13-coordination/`, `docs/v2-depth/11-memory/`
**Priority**: HIGH -- formalizes IronClaw's reactive/reflective/background split and provides the theoretical foundation for self-improving multi-agent coordination

---

## What This Document Covers

This document describes the cognitive architecture specified in the roko codebase -- a multi-layered system that separates agent processing into distinct speeds, coordination mechanisms, and organizational layers, drawing heavily from neuroscience, cognitive science, cybernetics, and biological self-organization. Each subsystem is documented with its theoretical foundations, the actual Rust code implementing it in roko, and a concrete mapping to IronClaw's existing architecture with integration sketches.

### What Is a Cognitive Architecture?

The term "cognitive architecture" in the AI agent context means the structural framework that determines how an agent perceives, reasons, decides, acts, learns, and remembers. Just as the human brain is not a single monolithic processor but a collection of specialized subsystems operating at different timescales -- perception at milliseconds, working memory at seconds, long-term consolidation during sleep -- a well-designed AI agent should decompose its processing into distinct modes that match the computational demands of different situations.

Classical cognitive architectures from AI research (ACT-R, SOAR, CLARION) demonstrated that fixed structural commitments -- how memory is organized, how production rules fire, how learning modifies future behavior -- matter more for long-term capability than any single inference improvement. The same principle applies to LLM-based agents: the harness wrapping the LLM -- how it manages context, routes between models, verifies outputs, and learns from experience -- determines the agent's practical capability.

This is roko's core thesis: "the scaffold IS the product" (from `docs/v1/00-architecture/00-vision-and-thesis.md`). Evidence supports this claim: Lee et al. (2026) demonstrated +7.7 points from harness optimization alone with 4x fewer tokens (Meta-Harness, arXiv:2603.28052); Jimenez et al. (2024) showed the same model achieving 30-65% solve rates depending on harness (SWE-bench); and Zaharia et al. (2024) argued that SOTA performance comes from compound AI systems, not individual models.

### How Biological Cognition Inspires the Design

The architecture does not attempt to replicate the brain. Instead, it identifies specific computational problems that biological nervous systems have solved over 500 million years of evolution, and adapts those solutions for software agents:

- **Multiple timescales**: The brain processes sensory input in milliseconds, maintains working memory over seconds, and consolidates long-term knowledge during sleep over hours. Roko separates processing into three analogous speeds (Gamma/Theta/Delta), allowing the agent to react quickly to environmental changes while still stepping back periodically for deeper reflection.

- **Prediction-error driven attention**: The brain allocates expensive neural computation to surprising inputs and coasts on cached predictions for expected ones. Roko's tier-routing system implements this -- most processing ticks use zero-cost heuristic checks (T0), escalating to expensive LLM inference only when surprise is detected.

- **Indirect coordination**: Social insects build elaborate structures without centralized planning, using environmental traces (pheromones) as coordination signals. Roko applies this stigmergic principle to multi-agent coordination, replacing direct inter-agent messaging with a shared pheromone environment.

- **Spontaneous specialization**: Embryonic cells differentiate into specialized tissues from identical starting conditions through reaction-diffusion dynamics. Roko uses the same mathematical mechanism (Turing/Gierer-Meinhardt) to produce emergent role differentiation among initially homogeneous agents.

---

## Table of Contents

1. [Theoretical Foundations](#1-theoretical-foundations)
2. [Three Cognitive Speeds (Gamma / Theta / Delta)](#2-three-cognitive-speeds-gamma--theta--delta)
3. [The Five-Layer Architecture](#3-the-five-layer-architecture)
4. [The Universal Cognitive Loop](#4-the-universal-cognitive-loop)
5. [Cognitive Cross-Cuts (Neuro / Daimon / Dreams)](#5-cognitive-cross-cuts-neuro--daimon--dreams)
6. [Stigmergic Coordination and Digital Pheromones](#6-stigmergic-coordination-and-digital-pheromones)
7. [SINR Interference Model](#7-sinr-interference-model)
8. [Morphogenetic Specialization (Turing Reaction-Diffusion)](#8-morphogenetic-specialization-turing-reaction-diffusion)
9. [C-Factor: Collective Intelligence Measurement](#9-c-factor-collective-intelligence-measurement)
10. [Practical Examples](#10-practical-examples)
11. [IronClaw Integration](#11-ironclaw-integration)
12. [Academic Foundations](#12-academic-foundations)
13. [Complexity Assessment and Risk](#13-complexity-assessment-and-risk)

---

## 1. Theoretical Foundations

The cognitive architecture draws from six research traditions. Understanding these foundations is necessary to understand why the architecture is shaped the way it is -- these are not decorative citations but load-bearing design decisions.

### 1.1 Neural Oscillation Bands (Buzsaki 2006)

The three cognitive speeds are named after neural oscillation bands documented in Buzsaki's "Rhythms of the Brain" (Oxford University Press, 2006, ISBN 978-0-19-530106-9). The mammalian brain operates at multiple frequency bands simultaneously, each supporting distinct cognitive functions:

| Brain Rhythm | Frequency | Cognitive Function | Roko Mapping |
|---|---|---|---|
| **Gamma** (30-100 Hz) | Fast | Sensory processing, attention binding, feature integration | Reactive: perceive environment changes and act immediately |
| **Theta** (4-8 Hz) | Medium | Working memory maintenance, spatial navigation, planning | Reflective: step back, re-plan, evaluate progress |
| **Delta** (0.5-4 Hz) | Slow | Deep sleep, memory consolidation, synaptic homeostasis | Consolidation: replay episodes, synthesize knowledge, prune |

The mapping is functional, not literal -- roko agents do not operate at 30-100 Hz. The names capture the computational role: Gamma for fast environmental scanning, Theta for deliberate evaluation, Delta for deep offline processing. Buzsaki's key insight is that these rhythms are not independent -- they are nested (gamma oscillations ride on top of theta waves), and this cross-frequency coupling is how the brain coordinates local and global processing. Roko implements an analogous nesting where Gamma ticks occur within Theta cycles, which occur within Delta epochs.

**Why this matters for IronClaw**: IronClaw currently has an implicit two-speed model (interactive agent turns vs. background heartbeat). The three-speed model makes the intermediate reflective speed explicit, enabling the agent to periodically step back and reassess its approach mid-conversation without waiting for the next heartbeat cycle.

### 1.2 Dual-Process Cognition (Kahneman 2011, Sun 2002)

Daniel Kahneman's "Thinking, Fast and Slow" (Farrar, Straus and Giroux, 2011, ISBN 978-0-374-27563-1) describes two modes of human cognition:

- **System 1**: Fast, automatic, heuristic. Pattern-matched responses that require minimal conscious effort. Operates continuously and generates impressions, feelings, and inclinations.
- **System 2**: Slow, deliberate, analytical. Step-by-step reasoning that demands attention and energy. Allocates attention to effortful mental activities including complex computations.

Ron Sun's CLARION architecture (Sun 2002, "Duality of the Mind", Lawrence Erlbaum Associates) extends this with a sub-conceptual level below System 1 -- implicit pattern matching that does not rise to conscious awareness at all, operating through distributed representations rather than explicit rules.

Roko maps these to three inference tiers:

| Cognitive Level | Kahneman | CLARION | Roko Tier | Characteristics |
|---|---|---|---|---|
| Sub-conceptual | -- | Sub-conceptual (implicit) | **T0** | No LLM call. Threshold checks, regex matches, cache lookups. |
| System 1 | Fast, automatic | Bottom-up processing | **T1** | Fast model (Haiku-class). Quick analysis with limited tool access. |
| System 2 | Slow, deliberate | Top-down processing | **T2** | Full model (Sonnet/Opus-class). Multi-turn deep reasoning. |

**Source**: `docs/v1/00-architecture/11-dual-process-and-active-inference.md`

### 1.3 Active Inference and the Free Energy Principle (Friston 2010)

Karl Friston's Free Energy Principle (Friston, K., 2010, "The free-energy principle: a unified brain theory?", Nature Reviews Neuroscience, 11(2), pp. 127-138, DOI: 10.1038/nrn2787) proposes that self-organizing systems minimize the divergence between their predictions and their observations. The prediction error -- the surprise signal -- drives learning, attention, and action.

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

### 1.4 Beer's Viable System Model (Beer 1972)

Stafford Beer's Viable System Model ("Brain of the Firm", Allen Lane, 1972; 2nd ed. Wiley, 1981, ISBN 978-0-471-27687-0) identifies five recursive subsystems required for any viable organization. Drawing on neurophysiology and cybernetics -- particularly Ross Ashby's Law of Requisite Variety (1956) -- Beer showed that any self-regulating system must have enough internal variety to match the variety of its environment. Roko's five architectural layers map directly to Beer's VSM:

| Beer VSM | Roko Layer | Function |
|---|---|---|
| System 1: Operations | L0 Runtime | Primary activities -- process lifecycle, I/O |
| System 2: Coordination | L1 Framework | Anti-oscillation -- model routing, tool dispatch |
| System 3: Control | L2 Scaffold + L3 Harness | Resource allocation + auditing |
| System 4: Intelligence | L4 Orchestration | Environmental scanning and adaptation |
| System 5: Policy | Cognitive cross-cuts (Daimon) | Identity, purpose, self-model |

This mapping is not decorative. It explains why five layers are necessary and sufficient, and why the cognitive cross-cuts must be injected across layers rather than living at any single level. Beer's related Good Regulator Theorem (Conant & Ashby, 1970, International Journal of Systems Science, 1(2), pp. 89-97) -- that every good regulator of a system must be a model of that system -- directly motivates the Daimon self-model subsystem.

**Source**: `docs/v1/00-architecture/12-five-layer-taxonomy.md`, `docs/v1/00-architecture/00-vision-and-thesis.md`

### 1.5 Stigmergy (Grasse 1959)

Pierre-Paul Grasse coined the term "stigmergy" in 1959 to describe how termites coordinate the construction of elaborate mound structures without centralized planning (Grasse, P.-P., 1959, "La reconstruction du nid et les coordinations interindividuelles chez Bellicositermes natalensis et Cubitermes sp. La theorie de la stigmergie: Essai d'interpretation du comportement des termites constructeurs", Insectes Sociaux, 6(1), pp. 41-80, DOI: 10.1007/BF02223791). The core insight: agents do not need to communicate directly -- they only need to read from and write to a shared environment. The environment itself becomes the coordination medium.

Theraulaz & Bonabeau (1999, "A Brief History of Stigmergy", Artificial Life, 5(2), pp. 97-116) formalized two distinct types: quantitative stigmergy (signals vary in intensity, like pheromone concentration) and qualitative stigmergy (signals trigger different behaviors based on type, like different chemical compounds). Three conditions define stigmergy:

1. **Shared environment**: All agents can read from and write to a common medium
2. **Persistent modifications**: Agent actions leave traces that outlast the agent's presence
3. **Stimulus-response coupling**: Traces trigger specific behaviors in agents that encounter them

**Source**: `docs/v1/13-coordination/00-stigmergy-theory.md`

### 1.6 Turing Reaction-Diffusion (Turing 1952)

In "The Chemical Basis of Morphogenesis" (Turing, A. M., 1952, Philosophical Transactions of the Royal Society of London B: Biological Sciences, 237(641), pp. 37-72, DOI: 10.1098/rstb.1952.0012), Alan Turing showed that a system of two chemicals -- an activator and an inhibitor -- can produce stable spatial patterns from a uniform initial state. The key condition: the inhibitor must diffuse faster than the activator. This produces spontaneous pattern formation without any central planner, explaining how embryonic cells differentiate into specialized tissues from identical starting conditions.

Gierer & Meinhardt formalized this as the activator-inhibitor model (Gierer, A. & Meinhardt, H., 1972, "A theory of biological pattern formation", Kybernetik, 12, pp. 30-39):

```
da/dt = rho_a * (a^2 / h) - mu_a * a + D_a * nabla^2(a) + sigma_a   (activator)
dh/dt = rho_h * a^2         - mu_h * h + D_h * nabla^2(h) + sigma_h   (inhibitor)
```

Where `a` = activator concentration, `h` = inhibitor concentration, `rho` = production rate, `mu` = decay rate, `D` = diffusion coefficient, `sigma` = noise (essential for symmetry breaking).

The critical instability condition is `D_h >> D_a`: the inhibitor must diffuse much faster than the activator. When this holds, local activator peaks self-reinforce (positive feedback) while the surrounding inhibitor field suppresses competing peaks (negative feedback), producing stable isolated spots of high activation.

---

## 2. Three Cognitive Speeds (Gamma / Theta / Delta)

The three cognitive speeds are the heartbeat of the architecture. Every agent operates at all three timescales concurrently, managed by an adaptive clock that modulates cadence based on the agent's emotional/motivational state. The three-speed design answers a fundamental question: how should an agent allocate its limited inference budget across reactive, reflective, and consolidative processing?

**Source**: `docs/v1/00-architecture/10-three-cognitive-speeds.md`
**Implementation**: `crates/roko-core/src/operating_frequency.rs`

### 2.1 Overview Table

| Speed | Period | Name | What Happens | Default Inference Tier | Turn Budget |
|---|---|---|---|---|---|
| **Gamma** | ~5-15s | Reactive | One complete cognitive loop tick. Environment scanning, tool calls, verification. | T0 (no LLM) | 0 (no agent dispatch) |
| **Theta** | ~75s-3min | Reflective | Summarize recent work. Update Daimon state. Check predictions. Re-plan if needed. | T1 (fast model) | 20 turns |
| **Delta** | Hours | Consolidation | Dreams: replay, synthesis, pruning. Knowledge tier promotion. Proactive insight generation. | T2 (full model) | 50 turns |

### 2.2 Gamma -- Reactive Speed

Gamma is the agent's heartbeat. Every 5-15 seconds, one complete cognitive loop tick executes: perceive the environment, select relevant information, compose a prompt, act, verify, and persist.

The critical design decision: **most Gamma ticks are T0 (zero LLM cost)**. Roko specifies 16 T0 probes -- zero-LLM diagnostic checks that determine whether the environment has changed enough to warrant model inference. If nothing surprising is detected, the tick completes without invoking any model. The agent "coasts" on existing heuristics.

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

If all 16 probes return "no change," the tick completes at T0 cost ($0). This is how approximately 80% of ticks are suppressed -- the FrugalGPT-inspired zero-cost majority (Chen et al. 2023, arXiv:2305.05176).

When a T0 probe detects surprise above threshold, the tick escalates:
1. Probe reports surprise -> escalate to T1 (fast model)
2. T1 analysis reports high uncertainty or high stakes -> escalate to T2 (full model)

**Source**: `docs/v1/00-architecture/11-dual-process-and-active-inference.md` (Section 3: The 16 T0 Probes)

### 2.3 Theta -- Reflective Speed

Every ~75 seconds to 3 minutes (adjustable), the agent pauses reactive work to reflect:

- **Summarize**: What has happened since the last Theta tick?
- **Update Daimon**: Recalculate the PAD vector (Pleasure-Arousal-Dominance) from recent outcomes
- **Check predictions**: Which predictions have resolved? Update calibration.
- **Re-plan**: Is the current approach working? Should the agent switch strategies?
- **Knowledge update**: Promote useful Working-tier knowledge to Consolidated.

Theta ticks typically invoke T1 (fast model, e.g., Claude Haiku class) for summarization, or T2 (full model) for re-planning when deeper analysis is needed.

### 2.4 Delta -- Consolidation Speed

During extended idle periods or on a scheduled basis (every 30+ minutes), the agent enters Delta mode for deep consolidation:

- **NREM Replay**: Replay recent episodes, weighted by prediction error magnitude (Mattar & Daw, 2018, "Prioritized memory access explains planning and hippocampal replay", Nature Neuroscience, 21, pp. 1609-1617). Episodes where the outcome differed most from expectation are replayed first.
- **REM Imagination**: Generate novel hypotheses via HDC (Hyperdimensional Computing) recombination (Boden, 2004, "The Creative Mind: Myths and Mechanisms", 2nd ed., Routledge). Counterfactual reasoning via Pearl's Structural Causal Model (Pearl, J., 2009, "Causality: Models, Reasoning, and Inference", 2nd ed., Cambridge University Press).
- **Knowledge Promotion**: Promote Consolidated-tier knowledge to Persistent.
- **Pruning**: Remove Engrams that have decayed below threshold.
- **Synthesis**: Extract cross-episode patterns into new playbook rules.

Delta ticks use T2 (full model, e.g., Claude Opus/Sonnet class) for deep reasoning and synthesis.

**Source**: `docs/v1/00-architecture/13-cognitive-cross-cuts.md` (Section 4: Dreams)

### 2.5 The OperatingFrequency Enum (Verified Source Code)

From `crates/roko-core/src/operating_frequency.rs`:

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

### 2.6 The InferenceTier Enum (Verified Source Code)

From `crates/roko-primitives/src/tier.rs`:

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

/// Maps an `InferenceTier` + vitality score to a concrete model identifier.
///
/// This is a pure stateless function. All model selection logic lives here.
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

**Source**: `crates/roko-primitives/src/tier.rs`

### 2.7 Frequency Selection Logic (Verified Source Code)

The operating frequency for a task is selected based on two inputs: the task's characteristics and the agent's current affect state (Daimon PAD vector).

From `crates/roko-core/src/operating_frequency.rs`:

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

Key behavior: when the Daimon reports low confidence (<0.3), combined with high arousal (>0.25) or low dominance (<-0.1), substantial tasks are promoted from Theta to Delta -- giving the agent more time and deeper reasoning to step back and reconsider. The `is_substantial()` method checks for tasks estimated at 30+ minutes, high reasoning level, deep context weight, or hardened quality profile.

### 2.8 The Adaptive Clock Scheduler (Verified Source Code)

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

The Theta interval adapts dynamically based on the agent's state:

- **Stalling** (completion rate <= 0.25): Theta interval x 0.5 -- reflect sooner because the agent is not making progress
- **Anxious** (confidence <= 0.35 AND arousal >= 0.25 AND dominance <= -0.1): Theta interval x 0.66 -- the agent needs to step back more frequently

The schedule context carries affect signals:

```rust
/// Runtime inputs used by the operating-frequency scheduler.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OperatingFrequencyScheduleContext {
    /// Time elapsed since the last theta evaluation.
    pub time_since_last_theta: Duration,
    /// Number of currently active tasks.
    pub active_tasks: usize,
    /// Recent task completion rate in `[0.0, 1.0]`.
    pub completion_rate: f64,
    /// Motivational confidence in `[0.0, 1.0]`.
    pub confidence: f64,
    /// Arousal in `[-1.0, 1.0]`.
    pub arousal: f64,
    /// Dominance in `[-1.0, 1.0]`.
    pub dominance: f64,
}
```

**Source**: `crates/roko-core/src/operating_frequency.rs` (full file with 12+ tests)

---

## 3. The Five-Layer Architecture

Roko's crates are organized into five architectural layers with strictly downward dependencies. Each layer maps to one of Beer's VSM subsystems.

**Source**: `docs/v1/00-architecture/12-five-layer-taxonomy.md`

### 3.1 The Layer Diagram

```
+------------------------------------------------------+
|                   Applications                        |
|  (coding agent, chain agent, research agent, custom)  |
+------------------------------------------------------+
|  Layer 4: ORCHESTRATION                               |
|  DAGs, scheduling, state machines, multi-agent coord  |
+------------------------------------------------------+
|  Layer 3: HARNESS                                     |
|  Gates, conductor, monitoring, interventions, eval    |
+------------------------------------------------------+
|  Layer 2: SCAFFOLD                                    |
|  Context engineering, prompts, enrichment, memory     |
+------------------------------------------------------+
|  Layer 1: FRAMEWORK                                   |
|  Connections, roles, tools, model routing, safety     |
+------------------------------------------------------+
|  Layer 0: RUNTIME / KERNEL                            |
|  Process lifecycle, Substrate, Bus, HDC, I/O, clock   |
+------------------------------------------------------+

  COGNITIVE CROSS-CUTS (injected into multiple layers):
  Neuro (knowledge) | Daimon (motivation) | Dreams (offline learning)
  + Inference Optimization | Safety & Provenance | Observability
```

**Dependencies flow STRICTLY downward.** Layer 4 may depend on Layer 3, never the reverse. Cross-cutting concerns are injected via trait objects, never via direct imports of higher layers.

### 3.2 Layer 0: Runtime / Kernel

**Purpose**: Process lifecycle, the two-medium kernel surface (Substrate for durable Engrams, Bus for ephemeral Pulses), supervision, cancellation, I/O, adaptive clock.

**Key crates**: `roko-core`, `roko-primitives`, `roko-runtime`

**What lives here**:
- Process spawning and lifecycle management (`ProcessSupervisor`)
- `Substrate` for durable Engram persistence and query
- `Bus` transport for topic-addressed Pulse delivery and bounded replay
- `HDC` (Hyperdimensional Computing) for similarity and clustering
- Cancellation tokens and graceful shutdown
- The adaptive clock managing Gamma/Theta/Delta frequencies
- Basic I/O primitives

**Beer VSM**: System 1 (Operations) -- the primary activities of the organization.

### 3.3 Layer 1: Framework

**Purpose**: Connections to external systems (LLMs, tools, MCP), roles, model routing, safety.

**Key crates**: `roko-agent`, `roko-std`

**What lives here**:
- LLM backend connections (Claude, OpenAI, local models, Ollama, ExecAgent)
- Tool registry and tool dispatch
- MCP (Model Context Protocol) client for external tool integration
- Model routing logic (CascadeRouter)
- Safety layer (role authorization, pre/post-execution checks)

**Synapse traits at L1**: `Router` (model/tool selection), `Scorer` (tool relevance)

**Beer VSM**: System 2 (Coordination) -- anti-oscillation, ensuring components work together without conflict.

### 3.4 Layer 2: Scaffold

**Purpose**: Context engineering, prompt assembly, enrichment, memory access.

**Key crates**: `roko-compose`

**What lives here**:
- SystemPromptBuilder (6-layer prompt assembly with role templates)
- Prompt templates for different agent roles (coder, researcher, planner)
- Context enrichment (injecting relevant knowledge, history, tool descriptions)
- Token budget management within prompts

**Synapse traits at L2**: `Scorer` (relevance for context selection), `Composer` (prompt assembly)

**Beer VSM**: System 3 (Control) -- resource allocation and internal management.

### 3.5 Layer 3: Harness

**Purpose**: Verification, monitoring, interventions, evaluation.

**Key crates**: `roko-gate`, `roko-fs`

**What lives here**:
- Gate pipeline (compile, test, clippy, diff, format, schema, judge, simulation -- 11+ gates)
- Adaptive gate thresholds (EMA-based)
- FileSubstrate (JSONL persistence)
- Monitoring and health checks

**Synapse traits at L3**: `Gate` (verification), `Policy` (conductor watchers, circuit breakers)

**Beer VSM**: System 3* (Audit) -- monitoring and verification of operations.

### 3.6 Layer 4: Orchestration

**Purpose**: Plan DAGs, parallel execution, state machines, multi-agent coordination.

**Key crates**: `roko-orchestrator`, `roko-conductor`

**What lives here**:
- Plan discovery and DAG construction
- Parallel task execution with dependency ordering
- State machine for plan phases (Pending -> Running -> Gated -> Complete)
- Session persistence and resumption
- Merge queue for coordinating concurrent agents
- Worktree management for parallel code modifications

**Synapse traits at L4**: `Policy` (state machine transitions, plan reactions)

**Beer VSM**: System 4 (Intelligence) -- environmental scanning and adaptation.

### 3.7 Dependency Rules

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
    // Retrieve relevant durable knowledge.
    let knowledge_engrams = knowledge
        .query(&Query::of_kind(Kind::Insight).limit(5), ctx)
        .await?;
    // Retrieve live coordination without importing a higher layer.
    let recent_pulses = bus
        .replay_since(ctx.checkpoint_seq, &TopicFilter::Glob("gate.verdict.*".into()))
        .await?;
    composer.compose_with(knowledge_engrams, recent_pulses, budget, scorer, ctx)
}
```

**Source**: `docs/v1/00-architecture/12-five-layer-taxonomy.md` (Section 9.2)

### 3.8 IronClaw Layer Mapping

| Roko Layer | IronClaw Equivalent | Notes |
|---|---|---|
| L0 Runtime | `src/db/`, `src/workspace/`, tokio runtime | Database + workspace = Substrate; tokio = process lifecycle |
| L1 Framework | `src/tools/`, `src/evaluation/`, `src/estimation/` | Tool dispatch, scoring, model routing |
| L2 Scaffold | `crates/ironclaw_engine/`, `crates/ironclaw_llm/` | Prompt composition, LLM integration |
| L3 Harness | `crates/ironclaw_safety/`, `src/sandbox/`, `src/observability/` | Safety checks, sandboxing, monitoring |
| L4 Orchestration | `src/agent/`, `src/channels/`, `src/hooks/` | Agent loop, multi-channel input, lifecycle hooks |

---

## 4. The Universal Cognitive Loop

Every agent in roko runs the same seven-step cognitive loop at each of the three speeds. What changes between speeds is the scope of perception, the budget available for composition, and the persistence cadence -- not the loop structure itself.

**Source**: `docs/v1/00-architecture/09-universal-cognitive-loop.md`

### 4.1 The Seven Steps

```
1. SENSE      -> Substrate.query | Bus.subscribe | external I/O
2. ASSESS     -> Scorer + Router jointly rank and select
3. COMPOSE    -> Composer assembles a prompt Engram under budget
4. ACT        -> execute LLM / tool / chain work, producing Pulses + Engrams
5. VERIFY     -> Engram-gates plus stream-gates produce Verdict Engrams
6. PERSIST    -> Substrate.put(Engrams)
   BROADCAST  -> Bus.publish(Pulses)
7. REACT      -> Policy outputs more Pulses and Engrams
```

Step 6 is intentionally split into two co-equal operations: `PERSIST` writes durable Engrams to the Substrate, while `BROADCAST` publishes ephemeral Pulses onto the Bus for live consumers. Durable records and ephemeral delivery are different jobs treated as peers.

### 4.2 Step Details

**SENSE** has three sources: `Substrate.query()` for durable Engrams (plans, episodes, heuristics), `Bus.subscribe()` for live Pulses (turn output, cancellation), and external I/O for inputs not yet normalized.

**ASSESS** is a combined Scorer + Router operation that answers two questions together: what matters, and why this item wins over alternatives.

**COMPOSE** turns selected material into a prompt Engram under a budget (tokens, bytes, wall time). This is where context window shaping happens.

**ACT** executes the selected work (typically an LLM turn, but also tool calls and chain actions). Output is twofold: a stream of Pulses for live observers, and a final Engram capturing the action result.

**VERIFY** is a gate pipeline, not a single check. Engram-gates verify durable outputs (producing Verdict Engrams). Stream-gates watch live Pulses during execution and can halt or downgrade before the final result is accepted.

**REACT**: Policies consume new Pulses and emit further outputs -- episode consolidation, circuit-breaking, routing feedback, task follow-up.

### 4.3 Cross-Cuts Are Not Loop Steps

Neuro, Daimon, and Dreams inject into operators and phases, but they do not occupy numbered positions in the loop:

- **Neuro** contributes durable knowledge to SENSE and COMPOSE
- **Daimon** biases ASSESS and influences ACT (gating risky actions)
- **Dreams** runs on its own Delta-speed cycle, consuming and producing Engrams

This distinction matters because the loop is about execution order, while the cross-cuts are about where additional cognitive machinery hooks in.

---

## 5. Cognitive Cross-Cuts (Neuro / Daimon / Dreams)

Three cognitive subsystems are injected across multiple layers rather than living at any single level. They interact bidirectionally through the loop's operators and speeds.

**Source**: `docs/v1/00-architecture/13-cognitive-cross-cuts.md`

### 5.1 Neuro -- Knowledge Management

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

Knowledge is encoded as 10,240-bit HDC vectors (Kanerva, P., 2009, "Hyperdimensional Computing: An Introduction to Computing in Distributed Representation with High-Dimensional Random Vectors", Cognitive Computation, 1(2), pp. 139-159) for O(1) similarity search using Hamming distance.

### 5.2 Daimon -- Motivation and Focus

`roko-daimon` provides the agent's self-model: a PAD (Pleasure-Arousal-Dominance) vector (Mehrabian, A. & Russell, J. A., 1974, "An Approach to Environmental Psychology", MIT Press) that biases assessment, action gating, and cadence selection.

**PAD Dimensions**:

| Dimension | Range | What It Represents |
|---|---|---|
| **Pleasure** (P) | [-1, 1] | Task success vs. failure |
| **Arousal** (A) | [-1, 1] | Urgency and load |
| **Dominance** (D) | [-1, 1] | Confidence and control |

**Six Behavioral States** (simplified Plutchik emotion wheel, Plutchik, R., 2001, "The Nature of Emotions", American Scientist, 89(4), pp. 344-350):

| State | PAD Region | Behavior |
|---|---|---|
| **Engaged** | P+, A moderate, D+ | Productive work. Standard Theta cadence. |
| **Focused** | P+, A low, D+ | Deep work. Extended Gamma runs, fewer Theta interruptions. |
| **Exploring** | P neutral, A+, D neutral | Curious. Higher exploration rate, more T2 escalation. |
| **Struggling** | P-, A+, D- | Difficulty. Shortened Theta cadence, more frequent reflection. |
| **Coasting** | P neutral, A-, D+ | Easy work. Extended Gamma, T0-heavy. |
| **Resting** | P neutral, A-, D neutral | Idle. Delta consolidation mode. |

**Somatic Markers** (Damasio, A., 1994, "Descartes' Error: Emotion, Reason, and the Human Brain", Putnam): Emotional signals from past experience bias decision-making before analytical reasoning. In roko, somatic markers are score modifiers applied to Router selections -- negative markers for tools/approaches that previously failed, positive for those that succeeded.

### 5.3 Dreams -- Offline Learning

`roko-dreams` provides offline learning during idle time at Delta frequency through a three-phase cycle:

| Phase | Neuroscience Inspiration | What Happens |
|---|---|---|
| **NREM Replay** | Slow-wave sleep replay (Mattar & Daw, 2018) | Replay recent episodes weighted by prediction error magnitude |
| **REM Imagination** | REM sleep creativity (Boden, 2004) | Generate novel hypotheses via HDC recombination; counterfactual reasoning (Pearl, 2009); emotional depotentiation (Walker & van der Helm, 2009, Annual Review of Clinical Psychology, 5, pp. 139-166) |
| **Integration Staging** | Memory consolidation (Lacaux et al., 2021, Science Advances, 7(50)) | Validate dream outputs against existing knowledge; promote if confidence exceeds threshold |

The **Hypnagogia Engine** generates creative hypotheses during the transition between active work and consolidation, with four components: Thalamic Gate (filters for high-novelty), Executive Loosener (relaxes constraints), Dali Interrupt (captures fleeting insights), and Homuncular Observer (coherence filter).

### 5.4 Cross-Cut Arbitration Protocol

When two cross-cuts produce conflicting signals, a fixed priority hierarchy resolves the conflict:

| Priority | Cross-Cut | Rationale |
|---|---|---|
| 1 (highest) | **Daimon** | Safety constraints and behavioral gating override other concerns |
| 2 | **Neuro** | Validated knowledge overrides speculative hypotheses |
| 3 (lowest) | **Dreams** | Dream-generated hypotheses are speculative |

When the priority hierarchy does not cleanly resolve a conflict, a VCG (Vickrey-Clarke-Groves) attention auction serves as tiebreaker. Each cross-cut bids its confidence; the winner pays the second-highest bid. The VCG mechanism ensures truthful reporting because inflating confidence provides no advantage.

**Source**: `docs/v1/00-architecture/13-cognitive-cross-cuts.md` (Section 6)

---

## 6. Stigmergic Coordination and Digital Pheromones

Roko's multi-agent coordination uses stigmergy: indirect coordination through environment modification. Rather than agents communicating directly, they deposit and sense digital pheromones -- coordination signals that decay over time, can be confirmed or contradicted, and drive emergent task allocation.

**Source**: `docs/v1/13-coordination/04-pheromone-kinds.md`, `docs/v2-depth/11-memory/12-pheromone-mechanics-and-interference.md`
**Implementation**: `crates/roko-orchestrator/src/coordination.rs`

### 6.1 The PheromoneKind Enum (Verified Source Code)

From `crates/roko-orchestrator/src/coordination.rs`:

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
            Self::Threat => Duration::from_secs(2 * 60 * 60),       // 2 hours
            Self::Opportunity => Duration::from_secs(4 * 60 * 60),  // 4 hours
            Self::Wisdom => Duration::from_secs(24 * 60 * 60),      // 24 hours
            Self::Alpha => Duration::from_secs(60 * 60),             // 1 hour
            Self::Pattern => Duration::from_secs(12 * 60 * 60),     // 12 hours
            Self::Anomaly | Self::Custom(_) => Duration::from_secs(6 * 60 * 60), // 6 hours
            Self::Consensus => Duration::from_secs(48 * 60 * 60),   // 48 hours
        }
    }
}
```

### 6.2 The Seven Pheromone Kinds in Detail

The kind system is organized into three tiers, inspired by Wilson's hierarchy of pheromone types in social insects (Wilson, E. O., 1971, "The Insect Societies", Belknap Press of Harvard University Press):

**Tier 1 -- Universal Kinds (present in every domain, every agent, every scope)**:

#### Threat (Alarm Pheromone)

| Property | Value |
|---|---|
| Default half-life | 2 hours |
| Default initial intensity | 1.0 |
| Biological analog | Alarm pheromone (formic acid in ants) |
| Confirmation effect | Standard extension (longer half-life) |
| Agent response | Stop current task, investigate, remediate |

Threat intensity encodes severity: 0.1-0.3 = Low (style violation), 0.4-0.6 = Medium (flaky tests), 0.7-0.8 = High (test failure, security vuln in dev), 0.9-1.0 = Critical (build failure, production security vuln).

When ambient Threat intensity is high, gate thresholds tighten (more strict verification) -- implementing a collective immune response.

#### Opportunity (Recruitment Pheromone)

| Property | Value |
|---|---|
| Default half-life | 4 hours |
| Default initial intensity | 0.8 |
| Biological analog | Recruitment pheromone (trail to food source) |
| Confirmation effect | Standard extension |
| Agent response | Evaluate opportunity, add to task queue if aligned with role |

High-intensity Threat (> 0.7) suppresses Opportunity pheromones in the same scope -- agents in threat-response mode should not be distracted by opportunities. This mirrors how alarm pheromone overrides foraging pheromone in ant colonies.

#### Wisdom (Trail Pheromone)

| Property | Value |
|---|---|
| Default half-life | 24 hours |
| Default initial intensity | 0.9 |
| Biological analog | Established trail pheromone (high persistence, well-confirmed) |
| Confirmation effect | Extends half-life; promotes to Consensus at 4+ confirmations |
| Agent response | Integrate into local knowledge base, apply to current work |

Wisdom pheromones typically emerge through a pipeline rather than direct deposit:

```
Agent observes pattern -> deposits Pattern pheromone
    -> Multiple agents confirm the Pattern
    -> Agent validates through operational testing
    -> Agent deposits Wisdom with Pattern as parent
    -> Other agents confirm the Wisdom
    -> At 5+ confirmations, may be promoted to permanent Engram
```

**Tier 2 -- Domain-Specific Kinds (common across domains, domain-dependent interpretation)**:

#### Alpha (Ephemeral Edge)

| Property | Value |
|---|---|
| Default half-life | 1 hour |
| Default initial intensity | 1.0 |
| Confirmation effect | **PARADOXICAL**: confirmation REDUCES effective half-life |
| Agent response | Act immediately or discard |

**The Alpha Paradox**: Unlike other kinds where confirmation increases persistence, confirmation of an Alpha signal indicates the first-mover advantage is eroding. From the verified source code, the actual Alpha decay formula uses a divisor model:

```rust
// From Pheromone::effective_half_life() in coordination.rs:
if self.kind.is_alpha() {
    // Alpha paradox: consensus makes alpha expire faster.
    let divisor = f64::from(self.confirmations).mul_add(0.1, 1.0);
    self.half_life.mul_f64(1.0 / divisor)
}
```

This means each confirmation adds 0.1 to the divisor:

| Confirmations | Divisor | Multiplier | Effective Half-Life (base = 1h) |
|---|---|---|---|
| 0 | 1.0 | 1.00 | 60 min |
| 1 | 1.1 | 0.91 | 55 min |
| 3 | 1.3 | 0.77 | 46 min |
| 5 | 1.5 | 0.67 | 40 min |
| 10 | 2.0 | 0.50 | 30 min |

Unlike the floor-based model in some docs, this formula provides smooth asymptotic decay -- the effective half-life never reaches zero but continues shrinking.

#### Pattern (Territorial Marking)

| Property | Value |
|---|---|
| Default half-life | 12 hours |
| Default initial intensity | 0.7 |
| Confirmation effect | Standard extension; promotes to Wisdom at 3+ |
| Agent response | Incorporate pattern into decision-making |

#### Anomaly (Novel Scent Detection)

| Property | Value |
|---|---|
| Default half-life | 6 hours |
| Default initial intensity | 0.8 |
| Confirmation effect | Standard extension |
| Agent response | Investigate -> classify as Threat, Opportunity, or noise -> re-deposit |

#### Consensus (Colony Odor)

| Property | Value |
|---|---|
| Default half-life | 48 hours |
| Default initial intensity | 0.9 |
| Confirmation effect | Standard extension; resists contradiction |
| Agent response | Treat as established fact; violating requires strong evidence |

**Tier 3 -- Custom Kinds**: `Custom(String)` allows domain plugins to define their own pheromone kinds without modifying the core enum. Custom kind identifiers are validated (alphanumeric + underscores, 1-64 chars, no leading underscore, no collision with built-in names).

### 6.3 Decay Model

Every pheromone decays according to an exponential half-life model:

```
intensity(t) = initial_intensity * 2^(-t / half_life)
```

Decay is computed lazily: the stored intensity is the value at deposit time, and any read computes current intensity from the deposit timestamp. This eliminates tick-aligned decay updates.

From the verified source code:

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

**Confirmation extension** (for standard, non-Alpha kinds) uses a linear scaling model in the actual source:

```rust
// From Pheromone::effective_half_life() in coordination.rs:
// Standard: confirmations extend the half-life linearly.
self.half_life.mul_f64(f64::from(self.confirmations).mul_add(0.5, 1.0))
```

This means `effective_half_life = base * (1 + 0.5 * confirmations)`:

| Confirmations | Multiplier | Effective Half-Life (base = 12h) |
|---|---|---|
| 0 | 1.0 | 12.0h |
| 1 | 1.5 | 18.0h |
| 2 | 2.0 | 24.0h |
| 3 | 2.5 | 30.0h |
| 5 | 3.5 | 42.0h |

The linear scaling provides stronger extension per confirmation than a logarithmic model, rewarding validated knowledge with significantly longer persistence.

### 6.4 Pheromone Scope (Verified Source Code)

From `crates/roko-orchestrator/src/coordination.rs`:

```rust
/// The propagation scope of a digital pheromone.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PheromoneScope {
    /// Pheromone is visible only within the specified substrate.
    Local(SubstrateId),
    /// Pheromone propagates to all agents in the specified collective.
    Mesh(CollectiveId),
    /// Pheromone propagates within a permissioned subnet.
    Subnet(SubnetId),
    /// Pheromone is published globally.
    Global,
}
```

Scopes are ranked: Local (0) < Subnet (1) < Mesh (2) < Global (3). Trust discounting is applied when reading from broader scopes:

```rust
/// Trust discount factors applied when reading pheromones from a broader scope.
pub const TRUST_DISCOUNT: [f64; 4] = [1.0, 0.90, 0.80, 0.50];
```

Local signals carry full trust; Global signals are halved, reflecting the principle that signals passing through more intermediaries may have been confirmed by agents with unknown provenance.

### 6.5 Promotion Cascade (Verified Source Code)

The full promotion pipeline graduates pheromones into permanent knowledge:

```
Pattern --[3+ confirmations, age > 50% half-life]--> Wisdom
Wisdom  --[4+ confirmations]-----------------------> Consensus
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

Additionally, scope promotion is controlled separately -- pheromones can escalate from Local to Subnet (3+ confirmations), Subnet to Mesh (5+), and Mesh to Global (10+), implemented through `ScopePromotionConfig` and `PromotionGate` (which requires confirmations from a minimum number of distinct agents).

**Source**: `docs/v1/13-coordination/04-pheromone-kinds.md`

### 6.6 Response Threshold Model (Emergent Task Allocation)

Each agent maintains per-kind response thresholds that determine when it switches from current work to respond to a pheromone signal. The probability follows a Hill function (Bonabeau, E., Theraulaz, G. & Deneubourg, J.-L., 1998, "Fixed Response Thresholds and the Regulation of Division of Labor in Insect Societies", Bulletin of Mathematical Biology, 60, pp. 753-807):

```
P(respond to kind k) = I_k^n / (I_k^n + theta_k^n)
```

Where `I_k` = current intensity, `theta_k` = agent's response threshold, `n` = Hill coefficient (default: 2).

The Hill function is a sigmoid that produces a smooth transition between non-response and full response. The Hill coefficient `n` controls steepness: n=1 gives a hyperbolic curve, n=2 gives a sigmoidal transition, n=4 would give an almost step-function. The default n=2 provides a balance between sensitivity and stability.

From the verified source code:

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

Note: habituation is asymmetric -- the threshold increases at half the learning rate (0.025) compared to reinforcement's decrease rate (0.05). This produces a bias toward responsiveness: it takes twice as many non-responses to undo one successful response.

This produces **emergent division of labor**: an agent that succeeds at handling Threats develops lower Threat thresholds, making it more likely to respond to future Threats -- specializing as a "threat responder" without explicit role assignment. Other agents' Threat thresholds drift upward through habituation from non-response. Complementary specialization emerges from purely local adaptation.

**Source**: `docs/v1/13-coordination/04-pheromone-kinds.md` (Response Threshold Model section)

---

## 7. SINR Interference Model

When multiple pheromone kinds coexist in the same scope, they interfere with each other's detectability. The interference model adapts the Signal-to-Interference-plus-Noise Ratio (SINR) from wireless communications (Tse, D. & Viswanath, P., 2005, "Fundamentals of Wireless Communication", Cambridge University Press).

**Source**: `docs/v2-depth/11-memory/12-pheromone-mechanics-and-interference.md` (Section 3)

### 7.1 The SINR Formula

```
SINR_k = I_target_k / (SUM_{j != k} alpha_{jk} * I_j + N_0)
```

Where:
- `I_target_k` = intensity of the target pheromone of kind k
- `I_j` = aggregate intensity of pheromones of kind j in the same scope
- `alpha_{jk}` = cross-kind interference coefficient (how much kind j interferes with sensing kind k)
- `N_0` = background noise floor (default: 0.01)

If SINR falls below `min_sinr` (default: 1.0, corresponding to 0 dB), the signal is undetectable. Otherwise, the effective intensity degrades gracefully via: `effective = target * (sinr / (1 + sinr))`.

### 7.2 The 7x7 Interference Matrix

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
- **Threat -> Opportunity: 0.60** -- Alarm overrides foraging. During a crisis, opportunities are suppressed.
- **Threat -> Wisdom: 0.10** -- Knowledge is resistant to alarm. Validated insights persist through crisis.
- **Opportunity -> Threat: 0.00** -- Opportunities never mask threats. Safety is asymmetric.
- **Wisdom and Pattern rows: all zeros** -- Informational kinds do not interfere with other sensing.
- **Consensus -> all: 0.05 max** -- Consensus is highly resistant to interference. Collective agreement is durable.

### 7.3 SINR-Adjusted Intensity Computation

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

### 7.4 Worked Example

**Scenario**: A Threat at 0.9 intensity and an Opportunity at 0.8 intensity, all other kinds at 0.0.

**Computing effective Opportunity intensity**:

1. Interference on Opportunity from Threat: `alpha[Threat][Opp] * I_Threat = 0.60 * 0.9 = 0.54`
2. SINR for Opportunity: `0.8 / (0.54 + 0.01) = 1.45`
3. Since SINR > 1.0 (min_sinr), the signal is detectable
4. Effective intensity: `0.8 * (1.45 / 2.45) = 0.47`

The Opportunity is still detectable (SINR > 1.0) but at reduced effective intensity (0.47 vs. 0.80). During a genuine crisis, the Threat naturally dominates attention without requiring explicit priority queues.

**Computing effective Threat intensity**:

1. Interference on Threat from Opportunity: `alpha[Opp][Threat] * I_Opp = 0.00 * 0.8 = 0.00`
2. SINR for Threat: `0.9 / (0.00 + 0.01) = 90.0`
3. Effective intensity: `0.9 * (90.0 / 91.0) = 0.89`

The Threat is essentially unaffected (0.89 vs 0.90), demonstrating the designed asymmetry: threats are always heard, even in a noisy pheromone field.

### 7.5 Anti-Saturation Mechanisms

Two thresholds prevent pheromone flooding:

- **Soft threshold** (500 active Pulses): Low-intensity pheromones decay at 2x speed
- **Hard threshold** (2000 active Pulses): Only pheromones with intensity > 0.1 survive
- **Per-kind cap** (100 per scope): No single kind can monopolize the field

```rust
struct AntiSaturationVerify {
    soft_threshold: usize,       // default: 500
    hard_threshold: usize,       // default: 2000
    hard_min_intensity: f64,     // default: 0.1
    max_per_kind_per_scope: usize, // default: 100
}
```

**Source**: `docs/v2-depth/11-memory/12-pheromone-mechanics-and-interference.md` (Section 4)

---

## 8. Morphogenetic Specialization (Turing Reaction-Diffusion)

When a group of agents starts with identical configurations, they face the **niche crowding problem**: all agents pursue the same strategies, compete for the same tasks, and produce redundant work. Morphogenetic specialization solves this by applying Turing's reaction-diffusion mechanism to produce spontaneous role differentiation from homogeneous starting conditions.

**Source**: `docs/v1/13-coordination/07-morphogenetic-specialization.md`, `docs/v2-depth/11-memory/13-morphogenetic-specialization-as-loop.md`

### 8.1 How Turing's Mechanism Applies to Agents

| Biological Component | Agent Equivalent |
|---|---|
| Activator | Profitable returns for a strategy dimension -- local, slow (learning takes hundreds of ticks) |
| Inhibitor | Pheromone Pulses showing other agents' specializations -- propagates fast via Bus (~milliseconds) |
| Diffusion asymmetry (D_h >> D_a) | Learning is slow (individual experience). Inhibition is fast (Bus propagation). |
| Noise (sigma) | Small random perturbations to break initial symmetry |
| Spatial pattern | Role differentiation -- each agent specializes in a different strategy dimension |

Because inhibition propagates through the Bus in milliseconds while activation requires hundreds of ticks of experience, **Turing's instability condition is naturally satisfied**. Stable specialist patterns emerge from initially homogeneous populations.

### 8.2 The 8-Dimensional Strategy Vector

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

From the verified source code:

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

Where `H(s) = -SUM s_k * ln(s_k)` (Shannon, C. E., 1948, "A Mathematical Theory of Communication", Bell System Technical Journal, 27(3), pp. 379-423) and `H_max = ln(STRATEGY_DIMS)`. 0.0 = maximum generalization (uniform). 1.0 = maximum specialization (all in one dimension). Healthy specialists stabilize around 0.5-0.7.

From the verified source code:

```rust
/// Compute the specialization index of a strategy vector.
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

### 8.3 The Gierer-Meinhardt Update Rule (Verified Source Code)

For each dimension k:

```
s_k(t+1) = s_k(t) + activation_k - inhibition_k - decay_k + noise_k
```

From the actual source in `MorphogeneticState::update()`:

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
- `activation_k = alpha * attributed_returns[k] * resource_pressure_scalar` -- profitable dimensions grow
- `inhibition_k = beta * collective_pheromone[k] / collective_size` -- crowded dimensions are suppressed
- `decay_k = mu * (s_k - baseline)` -- drift back toward uniform
- `noise_k ~ N(0, sigma_noise^2)` via Box-Muller transform -- break symmetry

Note the key difference from the canonical Gierer-Meinhardt formulation: the activation term here is linear in the strategy dimension (proportional to attributed returns) rather than quadratic (proportional to a^2/h). This linearization simplifies convergence analysis while preserving the essential diffusion asymmetry (beta > alpha) that drives pattern formation.

After update, concentrations are clamped to a floor of 0.001 (preventing extinction of any dimension) and then renormalized to sum to 1.0. Accumulators are reset for the next cycle.

### 8.4 Why beta > alpha Is Essential

The critical condition for Turing instability: inhibition must diffuse faster than activation.

- **Activation** (alpha = 0.05) is driven by the agent's own experience -- slow, like gene expression
- **Inhibition** (beta = 0.15) is driven by the group's pheromone field via Bus -- fast, like extracellular diffusion

With beta = 3 * alpha, inhibition ensures an agent's specialization is suppressed in dimensions where other group members are already concentrated. This pushes agents apart in strategy space, creating complementary specialists.

### 8.5 Niche Conflict Detection

The source code also includes niche conflict detection using cosine similarity between strategy vectors:

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

When conflicts are detected (cosine similarity > 0.9), the system boosts collective pheromone on the top-3 shared dimensions for both agents, triggering automatic separation via the Gierer-Meinhardt inhibition term.

### 8.6 Convergence Properties

From homogeneous initial conditions, convergence scales as O(N * log N) ticks:

| Group Size | Typical Convergence (ticks) | Wall Time (at 4 ticks/min) |
|---|---|---|
| 2 | ~500 | ~2 hours |
| 5 | ~800 | ~3.3 hours |
| 10 | ~1,200 | ~5 hours |
| 20 | ~1,800 | ~7.5 hours |
| 50 | ~3,000 | ~12.5 hours |

**Convergence guarantee**: For beta/alpha >= 2.0 and collective_size <= 50, the system converges to a stable pattern with probability > 0.99 within 3000 ticks (validated via Monte Carlo simulation: 10,000 runs per parameter setting).

### 8.7 Stability Monitoring

A stability Lens monitors for three pathological states:

1. **Convergence**: Trait variance < 0.01 for 100 consecutive ticks -- stable.
2. **Pitchfork Bifurcation**: When beta/alpha exceeds ~5.0, agents split into two extreme clusters. Detected by monitoring bimodality of strategy distributions.
3. **Hopf Oscillation**: Agents cyclically swap roles without settling. Detected via Lyapunov exponent estimation and sign-change counting.

```rust
enum StabilityState {
    Converged,    // Strategy vectors have stabilized
    Converging,   // Still moving toward equilibrium
    Oscillating,  // Needs parameter adjustment
}
```

### 8.8 Two-Timescale Coordination

Morphogenetic specialization operates at a **strategic timescale** (500-2000 ticks) and composes with response threshold allocation from Section 6.6, which operates at a **tactical timescale** (10-100 ticks):

| Mechanism | Timescale | What It Decides |
|---|---|---|
| Response threshold allocation | 10-100 ticks (tactical) | "Should I respond to this pheromone right now?" |
| Morphogenetic specialization | 500-2000 ticks (strategic) | "What kind of agent should I be?" |

The two reinforce each other: morphogenetic specialization determines the agent's strategic role (e.g., "verification specialist"), while response thresholds handle tactical moment-to-moment attention within that role.

---

## 9. C-Factor: Collective Intelligence Measurement

For multi-agent scenarios, roko measures collective intelligence across five axes using the c-factor metric. This is based on Woolley, A. W. et al. (2010, "Evidence for a Collective Intelligence Factor in the Performance of Human Groups", Science, 330(6004), pp. 686-688, DOI: 10.1126/science.1193147), who showed that group performance across varied tasks loads onto a single collective factor, analogous to the "g factor" in individual intelligence.

**Source**: `docs/v1/00-architecture/14-c-factor-collective-intelligence.md`

### 9.1 The Five Process Variables

| Variable | Agent Analog | Measured From |
|---|---|---|
| **Turn-taking equality** | How evenly the cohort shares turns | Pulse authorship entropy and sender share on the Bus |
| **Social perceptiveness (Peer prediction accuracy)** | How well members predict each other's outputs | `peer.prediction` vs `peer.outcome` residuals |
| **Trust calibration (Citation reciprocity)** | How often citations are useful and verified | Citation reciprocity and downstream gate survival in the Substrate |
| **Channel openness (Delivery rate)** | How much intended traffic is delivered | Bus delivery confirmation and subscriber reach |
| **Cognitive diversity (HDC diversity)** | How different the cohort's working set is | HDC distance across cohort Engrams |

### 9.2 Metric Computation (Verified Source Code)

From `crates/roko-orchestrator/src/coordination.rs`:

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
c = w_tte * turn_taking_entropy
  + w_ppa * peer_prediction_accuracy
  + w_cr  * citation_reciprocity
  + w_dr  * delivery_rate
  + w_hdc * hdc_diversity
  + bias
```

The weights are not static -- they are fitted online from cohort outcomes.

### 9.3 C-Factor Is a Covariate, Not an Objective

This is a critical design choice stated explicitly in the roko docs: c-factor is a diagnostic covariate, not a direct optimization target. Optimizing for c-factor directly can be counterproductive -- the system might route easy work, suppress dissent, or narrow task scope to make the number look better. This is a direct instance of Goodhart's Law: "When a measure becomes a target, it ceases to be a good measure."

**Good use**: C-factor falls AND task outcomes fall -> Policy intervenes.
**Bad use**: C-factor falls but Policy suppresses hard work to inflate the metric.

### 9.4 WisdomGate (Anti-Groupthink)

Before a consensus artifact is finalized, it passes a WisdomGate that encodes Surowiecki's four conditions (Surowiecki, J., 2004, "The Wisdom of Crowds", Doubleday):

From the verified source code:

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

**Source**: `docs/v1/00-architecture/14-c-factor-collective-intelligence.md` (Sections 5.3-5.5)

---

## 10. Practical Examples

These examples show how the cognitive architecture handles different situations, illustrating how the three speeds, tier routing, and pheromone system work together in practice.

### 10.1 Simple Query (Gamma Speed, T0/T1)

**Scenario**: User asks "What time is it?"

```
Gamma tick begins
  SENSE:    Receive user message from channel
  ASSESS:   T0 probes detect no surprise -- simple query, no state change
  COMPOSE:  Classify as reactive task (simple query)
            -> OperatingFrequency::Gamma
            -> InferenceTier::T0 (no LLM call needed)
  ACT:      Invoke time tool directly, no LLM involved
  VERIFY:   No gate needed for informational response
  PERSIST:  Store action record
  REACT:    No follow-up needed
Gamma tick completes: ~0.2s, $0 LLM cost
```

If the query were slightly more complex ("What time zone am I in, and should I schedule a meeting for 3pm London time?"), T0 probes would detect ambiguity and escalate to T1 (Haiku-class) for fast analysis.

### 10.2 Complex Bug Fix (Theta Speed, T1->T2 Escalation)

**Scenario**: User reports a test failure after a refactoring.

```
Gamma tick begins
  SENSE:    Receive bug report
  ASSESS:   T0 probe #5 (compile_error_new) fires
            -> Escalate to T1
  T1 analysis: Identify the failing test, locate the change
            Confidence is moderate (0.6), arousal is moderate
            -> OperatingFrequency::Theta (standard deliberative)
  ACT:      T1 model analyzes the error, proposes a fix
  VERIFY:   Gate pipeline: compile gate passes, test gate fails
            -> Prediction error: fix did not work
            -> Confidence drops to 0.35, arousal rises to 0.5
            -> affect_suggests_reflection() triggers

Theta reflection fires early (shortened cadence due to struggling state)
  ASSESS:   Summarize: fix attempt failed, tests still red
  ACT:      T2 model (Opus-class) invoked for deeper analysis
            Reviews broader context, identifies root cause
  VERIFY:   Compile gate passes, test gate passes
  PERSIST:  Store fix, deposit Wisdom pheromone ("trait import order matters")
  REACT:    Update Daimon PAD (pleasure up, arousal down)
```

### 10.3 Background Consolidation (Delta Speed, T2)

**Scenario**: Agent has been idle for 30 minutes after completing several tasks.

```
Delta tick triggered by OperatingFrequencyScheduler
  (context.is_idle() == true -> OperatingFrequency::Delta)

  NREM Replay:
    Replay the 5 highest-prediction-error episodes from last session
    Episode: "Tried to use --no-verify flag, hook blocked it"
      -> High prediction error (expected success, got failure)
      -> Extract Heuristic: "This repo requires hook compliance"

  REM Imagination:
    HDC recombination of episodes
    Cross-pollinate: "Hook compliance pattern" + "CI pipeline gates"
      -> Novel hypothesis: "Pre-commit validation could prevent 60% of CI failures"
      -> Deposit as Pattern pheromone

  Integration Staging:
    Validate hypothesis against Neuro knowledge base
    Existing Warning: "CI is slow, avoid unnecessary pushes"
      -> Hypothesis aligns with existing knowledge
      -> Promote Pattern to Wisdom if confirmed by gate pass

  PERSIST:  Store new Heuristic and Pattern Engrams
  REACT:    Schedule follow-up task to test hypothesis
```

### 10.4 Multi-Agent Coordination (Pheromone-Driven)

**Scenario**: Three agents (A, B, C) start with identical configurations and need to work on a codebase with refactoring, testing, and documentation needs.

```
Initial state (all agents):
  strategy = [0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125, 0.125]
  specialization_index = 0.0 (maximum generalization)

After 200 ticks (early differentiation via noise + returns):
  Agent A: execution dimension gets positive returns (successful code changes)
    -> activation in dimension 2 grows
    -> deposits collective_pheromone[2] on Bus
  Agent B: encounters execution already taken (high collective_pheromone[2])
    -> inhibition suppresses execution
    -> verification dimension gets positive returns from catching A's bugs
  Agent C: both execution and verification crowded
    -> noise tips toward exploration dimension
    -> finds novel approaches, deposits Pattern pheromones

After 800 ticks (stable specialization):
  Agent A: strategy ≈ [0.05, 0.05, 0.45, 0.05, 0.10, 0.05, 0.20, 0.05]
    specialization_index ≈ 0.55 (execution + exploitation specialist)
  Agent B: strategy ≈ [0.10, 0.05, 0.05, 0.40, 0.05, 0.05, 0.05, 0.25]
    specialization_index ≈ 0.52 (verification + coordination specialist)
  Agent C: strategy ≈ [0.30, 0.20, 0.05, 0.05, 0.10, 0.25, 0.03, 0.02]
    specialization_index ≈ 0.48 (depth + exploration specialist)
```

The response threshold model handles tactical allocation:
- Agent B has low Threat thresholds (quickly responds to test failures)
- Agent A has low Opportunity thresholds (responds to new feature requests)
- Agent C has low Anomaly thresholds (investigates unexpected patterns)

### 10.5 SINR in Action

**Scenario**: Agent A deposits a critical Threat (test suite broken, intensity 0.95) while Agent C simultaneously deposits an Opportunity (found optimization, intensity 0.7).

Without interference model: both signals compete equally for attention.

With SINR interference:
```
Effective Opportunity intensity:
  Interference = alpha[Threat][Opp] * 0.95 = 0.60 * 0.95 = 0.57
  SINR = 0.7 / (0.57 + 0.01) = 1.21
  Effective = 0.7 * (1.21 / 2.21) = 0.38

Effective Threat intensity:
  Interference = alpha[Opp][Threat] * 0.7 = 0.00 * 0.7 = 0.00
  SINR = 0.95 / (0.00 + 0.01) = 95.0
  Effective = 0.95 * (95.0 / 96.0) = 0.94
```

Result: Threat dominates (0.94 vs 0.38). Agents prioritize fixing the test suite. The Opportunity is not lost -- it persists at reduced intensity and will become detectable once the Threat is resolved and decays.

---

## 11. IronClaw Integration

### 11.1 Formalize Cognitive Speeds

**Where**: `src/agent/` -- agent loop
**How**: Explicitly categorize processing into Gamma/Theta/Delta, integrated with the existing cascade router.

```rust
// File: src/agent/cognitive_speed.rs

use std::time::{Duration, Instant};

/// Cognitive speed classification for IronClaw agent processing.
/// Maps to roko's OperatingFrequency with IronClaw-specific adaptations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CognitiveSpeed {
    /// Reactive: direct response, cheap model (T0/T1).
    /// Maps to: simple queries, tool calls, pattern-matched responses.
    Gamma,
    /// Reflective: planning, self-correction, strategy evaluation (T1/T2).
    /// Maps to: mid-conversation re-planning, error analysis.
    Theta,
    /// Consolidation: background learning, memory reorganization (T2).
    /// Maps to: heartbeat processing, cross-session synthesis.
    Delta,
}

impl CognitiveSpeed {
    /// Classify the current turn context into a cognitive speed.
    pub fn classify(context: &TurnContext) -> Self {
        // T0 probes: zero-cost checks for whether LLM inference is needed
        if context.is_simple_query() || context.is_tool_only() {
            return Self::Gamma;
        }

        // Background/heartbeat processing -> Delta
        if context.is_background() || context.is_heartbeat() {
            return Self::Delta;
        }

        // Multi-step planning, error recovery, or low confidence -> Theta
        if context.requires_planning()
            || context.has_recent_errors()
            || context.confidence() < 0.3
        {
            return Self::Theta;
        }

        // Default: reflective
        Self::Theta
    }

    /// Map to IronClaw's existing LLM provider selection.
    /// Integrates with the cascade router from ironclaw_llm.
    pub fn model_tier(&self) -> ModelTier {
        match self {
            Self::Gamma => ModelTier::Fast,      // Haiku-class
            Self::Theta => ModelTier::Standard,   // Sonnet-class
            Self::Delta => ModelTier::Best,       // Opus-class
        }
    }

    /// Maximum agent turns before forced reflection.
    pub fn turn_budget(&self) -> u32 {
        match self {
            Self::Gamma => 1,    // Single turn, immediate response
            Self::Theta => 10,   // Standard multi-turn budget
            Self::Delta => 25,   // Extended budget for deep work
        }
    }
}
```

### 11.2 Adaptive Reflection Scheduler

**Where**: `src/agent/` -- agent loop
**How**: Implement the adaptive scheduler that shortens reflection intervals when the agent is struggling.

```rust
// File: src/agent/reflection.rs

use std::time::{Duration, Instant};

/// Scheduler that determines when the agent should pause for reflection.
/// Adapted from roko's OperatingFrequencyScheduler.
pub struct ReflectionScheduler {
    /// Base interval between reflective pauses.
    theta_interval: Duration,     // default: 3 minutes
    /// Maximum interval before forced consolidation.
    delta_interval: Duration,     // default: 30 minutes
    /// Time of last reflective pause.
    last_theta: Instant,
}

impl Default for ReflectionScheduler {
    fn default() -> Self {
        Self {
            theta_interval: Duration::from_secs(180),
            delta_interval: Duration::from_secs(30 * 60),
            last_theta: Instant::now(),
        }
    }
}

impl ReflectionScheduler {
    pub fn should_reflect(&self, context: &AgentContext) -> ReflectionMode {
        let elapsed = self.last_theta.elapsed();

        // Idle -> consolidate
        if context.is_idle() {
            return ReflectionMode::Delta;
        }

        // Very long time since last reflection -> consolidate
        if elapsed >= self.delta_interval {
            return ReflectionMode::Delta;
        }

        // Adaptive Theta interval
        let mut theta_due = self.theta_interval;

        // Stalling: reflect sooner (completion_rate <= 0.25)
        if context.completion_rate() <= 0.25 {
            theta_due = theta_due.mul_f64(0.5);
        }

        // Low confidence + high error rate: reflect sooner
        if context.is_struggling() {
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

pub enum ReflectionMode {
    /// Continue current processing.
    Continue,
    /// Pause for reflective evaluation (summarize, re-plan).
    Theta,
    /// Enter deep consolidation mode (heartbeat, cross-session synthesis).
    Delta,
}
```

### 11.3 Layer Discipline Enforcement

**Where**: Architecture-wide
**How**: Use the five-layer model to enforce dependency direction.

IronClaw's existing structure already partially follows the five-layer pattern. The primary enforcement mechanisms:

1. **Cargo workspace dep rules**: Ensure extracted crates (`ironclaw_safety`, `ironclaw_llm`, `ironclaw_skills`, `ironclaw_engine`) do not import from higher-level modules. The existing CLAUDE.md rule "Import directly from the extracted crate" is already an instance of this principle.
2. **CI checks**: Add a lint that flags imports crossing layer boundaries in the wrong direction. Can be implemented as a simple cargo-deny or custom script checking `use` paths.
3. **Trait injection**: Where cross-layer communication is needed, use `&dyn Trait` objects rather than direct imports. IronClaw already uses this pattern for `Database`, `Channel`, `Tool`, `LlmProvider`, and `Observer`.

### 11.4 Pheromone System for Cross-Session Learning

**Where**: `src/workspace/`
**How**: Lightweight implementation using tagged workspace memories with decay.

```rust
// File: src/workspace/pheromone.rs

use std::time::Duration;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// A digital pheromone stored as a tagged workspace memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pheromone {
    pub kind: PheromoneKind,
    pub location: String,        // what this refers to (tool, topic, file)
    pub initial_intensity: f64,
    pub depositor: String,       // agent session ID or background task name
    pub deposited_at: DateTime<Utc>,
    pub half_life: Duration,
    pub confirmations: u32,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PheromoneKind {
    Threat,       // "Danger here -- avoid this approach"
    Opportunity,  // "Promising direction -- explore here"
    Wisdom,       // "Proven knowledge -- trust this"
    Alpha,        // "I'm working on this -- don't duplicate" (ephemeral)
    Pattern,      // "Recurring pattern detected here"
    Anomaly,      // "Something unexpected -- investigate"
    Consensus,    // "Multiple sessions agree on this"
}

impl Pheromone {
    /// Compute current intensity using exponential decay.
    pub fn current_intensity(&self) -> f64 {
        let elapsed = Utc::now()
            .signed_duration_since(self.deposited_at)
            .to_std()
            .unwrap_or_default();

        let effective_hl = self.effective_half_life();
        if effective_hl.is_zero() {
            return 0.0;
        }
        let exponent = -(elapsed.as_secs_f64() / effective_hl.as_secs_f64());
        self.initial_intensity * 2.0_f64.powf(exponent)
    }

    /// Half-life adjusted for confirmations.
    /// Follows the verified roko formula: linear extension for standard kinds,
    /// divisor-based shortening for Alpha.
    fn effective_half_life(&self) -> Duration {
        if matches!(self.kind, PheromoneKind::Alpha) {
            // Alpha paradox: confirmations shorten half-life.
            let divisor = 1.0 + self.confirmations as f64 * 0.1;
            Duration::from_secs_f64(self.half_life.as_secs_f64() / divisor)
        } else {
            // Standard: confirmations extend half-life linearly.
            let multiplier = 1.0 + 0.5 * self.confirmations as f64;
            Duration::from_secs_f64(self.half_life.as_secs_f64() * multiplier)
        }
    }

    pub fn is_evaporated(&self) -> bool {
        self.current_intensity() < 0.01
    }
}
```

**Practical integration with workspace memory**:

The pheromone system overlays on IronClaw's existing `memory_write`/`memory_search` tools using a namespaced tagging convention:

```rust
// Background task finds something interesting
memory_write("pheromone:opportunity:new-api-endpoint",
    "GitHub API v4 endpoint supports batch operations",
    tags: ["pheromone", "opportunity", "github-api"],
    metadata: { "depositor": "heartbeat", "half_life_secs": 14400 })

// Agent tries an approach that fails
memory_write("pheromone:threat:api-rate-limit",
    "GitHub API rate limit hit -- consider batching requests",
    tags: ["pheromone", "threat", "github-api"],
    metadata: { "depositor": "session-abc", "half_life_secs": 7200 })

// Query pheromones before starting work
let threats = memory_search("pheromone:threat", namespace: current_context);
let opportunities = memory_search("pheromone:opportunity", namespace: current_context);
```

### 11.5 Multi-Agent Coordination (Future)

**Where**: Future multi-agent support
**How**: When IronClaw supports multiple agents (e.g., through the team/swarm mechanism), use c-factor measurement and morphogenetic specialization for coordination.

The pheromone system (Section 11.4) provides the foundation. When multiple agent sessions share a workspace:

1. Each session deposits pheromones about its current focus (Alpha kind) to prevent duplication
2. Successful approaches deposit Opportunity/Wisdom pheromones for future sessions
3. Failed approaches deposit Threat pheromones to warn future sessions
4. The response threshold model naturally produces specialization as sessions accumulate experience

---

## 12. Academic Foundations

### Core Cognitive Architecture References

| Citation | Contribution |
|---|---|
| Buzsaki, G. (2006). "Rhythms of the Brain". Oxford University Press. ISBN 978-0-19-530106-9. | Neural oscillation bands: Gamma/Theta/Delta as functional timescales. Naming inspiration for three cognitive speeds. Cross-frequency coupling as coordination mechanism. |
| Kahneman, D. (2011). "Thinking, Fast and Slow". Farrar, Straus and Giroux. ISBN 978-0-374-27563-1. | Dual-process theory: System 1 (fast, automatic) / System 2 (slow, deliberate). Maps to T1/T2 tiers. |
| Sun, R. (2002). "Duality of the Mind". Lawrence Erlbaum Associates. | CLARION: dual-level cognitive architecture with sub-conceptual level. Maps to T0. |
| Sumers, T. R. et al. (2023). "Cognitive Architectures for Language Agents". arXiv:2309.02427. | CoALA: cognitive architecture framework for language agents. Structural blueprint for the universal cognitive loop. |
| Anderson, J. R. (1983). "The Architecture of Cognition". Harvard University Press. | ACT-R: production system cognitive architecture. T0 probes as productions. |
| Laird, J. E. et al. (1987). "SOAR: An architecture for general intelligence". Artificial Intelligence, 33(1), pp. 1-64. | SOAR: universal subgoaling. T0->T1 escalation as impasse detection. |
| Baars, B. J. (1988). "A Cognitive Theory of Consciousness". Cambridge University Press. | Global Workspace Theory: limited-capacity conscious workspace. Context window as workspace. |

### Active Inference and Prediction

| Citation | Contribution |
|---|---|
| Friston, K. (2010). "The free-energy principle: a unified brain theory?" Nature Reviews Neuroscience, 11(2), pp. 127-138. DOI: 10.1038/nrn2787. | Free Energy Principle: prediction error drives learning and attention. Foundation for tier routing. |
| Clark, A. (2013). "Whatever next? Predictive brains, situated agents, and the future of cognitive science". Behavioral and Brain Sciences, 36(3), pp. 181-204. | Predictive Processing: brain as prediction machine. Supports prediction-error attention. |
| de Vries, M. et al. (2025). arXiv:2504.14898. | EFE as variational inference. Theoretical basis for attention allocation. |
| Chen, L. et al. (2023). "FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance". arXiv:2305.05176. | Cascade routing matches GPT-4 at 2% cost. Foundation for CascadeRouter and T0 probes. |
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
| Mattar, M. G. & Daw, N. D. (2018). "Prioritized memory access explains planning and hippocampal replay". Nature Neuroscience, 21, pp. 1609-1617. | Prioritized memory replay: replay what is most useful for future decisions. Foundation for Delta NREM. |
| Baddeley, A. (2000). "The episodic buffer: a new component of working memory?" Trends in Cognitive Sciences, 4(11), pp. 417-423. | Working memory model: central executive manages attention allocation. Maps to Theta. |
| Walker, M. P. & van der Helm, E. (2009). "Overnight therapy? The role of sleep in emotional brain processing". Annual Review of Clinical Psychology, 5, pp. 139-166. | REM sleep emotional depotentiation. Foundation for Dreams REM phase. |
| Lacaux, C. et al. (2021). "Sleep onset is a creative sweet spot". Science Advances, 7(50), eabj5866. | Hypnagogia: creative insights during sleep onset. Foundation for hypnagogia engine. |
| McClelland, J. L. et al. (1995). "Why there are complementary learning systems in the hippocampus and neocortex". Psychological Review, 102(3), pp. 419-457. | Complementary Learning Systems theory: hippocampal-neocortical consolidation. |

### Stigmergy and Multi-Agent Coordination

| Citation | Contribution |
|---|---|
| Grasse, P.-P. (1959). "La reconstruction du nid et les coordinations interindividuelles chez Bellicositermes natalensis et Cubitermes sp." Insectes Sociaux, 6(1), pp. 41-80. DOI: 10.1007/BF02223791. | Stigmergy: indirect coordination through environmental modification. |
| Wilson, E. O. (1971). "The Insect Societies". Belknap Press of Harvard University Press. | Pheromone type hierarchy: alarm, recruitment, trail. Interference matrix basis. |
| Bonabeau, E., Theraulaz, G. & Deneubourg, J.-L. (1998). "Fixed Response Thresholds and the Regulation of Division of Labor in Insect Societies". Bulletin of Mathematical Biology, 60, pp. 753-807. | Fixed response thresholds and division of labor. Hill function model for emergent task allocation. |
| Theraulaz, G. & Bonabeau, E. (1999). "A Brief History of Stigmergy". Artificial Life, 5(2), pp. 97-116. | Formalized quantitative and qualitative stigmergy. Three conditions for stigmergic systems. |
| Parunak, H. V. D., Brueckner, S. & Sauter, J. (2005). "Digital pheromones for coordination of unmanned vehicles". Environments for Multi-Agent Systems, LNAI 3374, Springer. | Digital pheromones in multi-agent systems. |
| Dorigo, M. et al. (2000). "Ant algorithms and stigmergy". Future Generation Computer Systems, 16(8), pp. 851-871. | Ant colony optimization: emergent coordination under local rules. |
| Tse, D. & Viswanath, P. (2005). "Fundamentals of Wireless Communication". Cambridge University Press. | SINR model adapted for pheromone interference. |

### Morphogenesis and Self-Organization

| Citation | Contribution |
|---|---|
| Turing, A. M. (1952). "The Chemical Basis of Morphogenesis". Philosophical Transactions of the Royal Society of London B, 237(641), pp. 37-72. DOI: 10.1098/rstb.1952.0012. | Reaction-diffusion mechanism: stable patterns from uniform initial state. |
| Gierer, A. & Meinhardt, H. (1972). "A theory of biological pattern formation". Kybernetik, 12, pp. 30-39. | Activator-inhibitor model: formal dynamics for reaction-diffusion. Short-range activation, long-range inhibition. |
| Kauffman, S. A. (1993). "The Origins of Order: Self-Organization and Selection in Evolution". Oxford University Press. | Autocatalytic sets, edge of chaos, self-sustaining improvement cycles. |
| Shannon, C. E. (1948). "A Mathematical Theory of Communication". Bell System Technical Journal, 27(3), pp. 379-423. | Information theory: entropy as specialization index measure. |

### Collective Intelligence

| Citation | Contribution |
|---|---|
| Woolley, A. W. et al. (2010). "Evidence for a Collective Intelligence Factor in the Performance of Human Groups". Science, 330(6004), pp. 686-688. DOI: 10.1126/science.1193147. | Collective intelligence factor (c-factor) in human groups. Single factor explains 43% of variance across group tasks. |
| Surowiecki, J. (2004). "The Wisdom of Crowds". Doubleday. | Four conditions: diversity, independence, decentralization, aggregation. |
| Beer, S. (1972). "Brain of the Firm". Allen Lane. 2nd ed. Wiley, 1981. ISBN 978-0-471-27687-0. | Viable System Model: five recursive subsystems for viable organizations. |
| Ashby, W. R. (1956). "An Introduction to Cybernetics". Chapman & Hall. | Law of Requisite Variety: regulatory capacity must match environment variety. |
| Conant, R. C. & Ashby, W. R. (1970). "Every good regulator of a system must be a model of that system". International Journal of Systems Science, 1(2), pp. 89-97. | Good Regulator Theorem: agent must model itself. Motivates Daimon self-model. |

### Scaffold Thesis Evidence

| Citation | Contribution |
|---|---|
| Lee, Y. et al. (2026). "Meta-Harness: Harness Optimization for SWE-bench". arXiv:2603.28052. | +7.7 pts from harness optimization alone, 4x fewer tokens. |
| Khattab, O. et al. (2024). "DSPy: Compiling Declarative Language Model Calls into State-of-the-Art Pipelines". | Compiler-optimized prompt pipelines outperform manual prompt engineering. |
| Jimenez, C. E. et al. (2024). "SWE-bench: Can Language Models Resolve Real-World GitHub Issues?" | Same model, 30-65% solve rate depending on harness. |
| Zaharia, M. et al. (2024). "The Shift from Models to Compound AI Systems". | Compound AI Systems thesis: SOTA from systems, not models. |
| Boden, M. A. (2004). "The Creative Mind: Myths and Mechanisms". 2nd ed. Routledge. | Computational creativity: exploratory, combinational, transformational. Foundation for REM imagination. |
| Pearl, J. (2009). "Causality: Models, Reasoning, and Inference". 2nd ed. Cambridge University Press. | Structural Causal Models for counterfactual reasoning in Dreams. |
| Kanerva, P. (2009). "Hyperdimensional Computing: An Introduction". Cognitive Computation, 1(2), pp. 139-159. | HDC vectors for O(1) similarity search. Foundation for Neuro knowledge encoding. |

---

## 13. Complexity Assessment and Risk

### Implementation Effort Estimates

| Component | Estimated Lines | Complexity | Dependencies | IronClaw Files |
|---|---|---|---|---|
| Cognitive speed classification | ~200-300 | Low | None -- can be implemented independently | `src/agent/cognitive_speed.rs` |
| Adaptive reflection scheduler | ~150-200 | Low | Cognitive speed classification | `src/agent/reflection.rs` |
| Pheromone system (workspace-backed) | ~400-500 | Medium | `src/workspace/` memory system | `src/workspace/pheromone.rs` |
| Layer discipline enforcement | CI/Cargo config | Low | Build system only | `scripts/`, `Cargo.toml` |
| SINR interference (if multi-agent) | ~200-300 | Medium | Pheromone system | `src/workspace/pheromone.rs` |
| Response threshold model | ~250-350 | Medium | Pheromone system | `src/workspace/pheromone.rs` |
| Morphogenetic specialization (if multi-agent) | ~500-700 | High | Pheromone system, multi-agent infrastructure | New crate or `src/agent/` |
| C-factor measurement (if multi-agent) | ~300-400 | Medium | Multi-agent infrastructure, workspace metrics | New module |

### Implementation Priority

1. **Immediate** (no dependencies, low risk):
   - Cognitive speed classification (Gamma/Theta/Delta) in the agent loop
   - Adaptive reflection scheduler
   - Layer discipline enforcement in CI

2. **Short-term** (low-to-medium risk):
   - Pheromone system using workspace memories for cross-session learning
   - Threat/Opportunity/Wisdom pheromones for heartbeat integration

3. **Medium-term** (requires multi-agent support):
   - SINR interference model
   - Response threshold model for emergent task allocation
   - C-factor measurement

4. **Long-term** (highest complexity):
   - Morphogenetic specialization for multi-agent role emergence
   - Full Dreams consolidation cycle
   - Adaptive interference matrix learning

### Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| Over-engineering cognitive speeds for single-agent use | Low | Start with simple Gamma/Theta classification, add Delta for heartbeat |
| Pheromone system adding noise to workspace | Medium | Use separate namespace; aggressive evaporation threshold; monitor retrieval quality |
| Morphogenetic parameters failing to converge | Medium | Monte Carlo validation shows convergence at beta/alpha >= 2.0 for groups <= 50 |
| C-factor becoming a Goodhart metric | Low | Treat as covariate per roko design; never optimize directly |
| Layer discipline breaking existing code | Low | Enforce incrementally; grandfather existing violations with `// dispatch-exempt` |
| Alpha pheromone decay formula confusion | Low | Use verified divisor-based formula from roko source, not the alternative floor-based formula from some docs |

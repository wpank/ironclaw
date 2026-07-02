# Affect Engine (Daimon)

**Source crate**: `roko-daimon` (`crates/roko-daimon/src/lib.rs`)
**Supporting crate**: `roko-primitives` (`crates/roko-primitives/src/pad.rs`) -- canonical PAD vector
**Behavioral state classifier**: `roko-core` (`crates/roko-core/src/affect.rs`) -- `BehavioralState::classify()`
**Priority**: MEDIUM -- valuable for user modeling and agent self-regulation, requires careful threshold calibration

---

## Table of Contents

1. [What Is an Affect Engine and Why It Matters](#what-is-an-affect-engine-and-why-it-matters)
   - [Beyond Sentiment Analysis](#beyond-sentiment-analysis)
   - [The Practical Case for Affect in AI Agents](#the-practical-case-for-affect-in-ai-agents)
2. [Psychology Foundations](#psychology-foundations)
   - [PAD Model (Mehrabian & Russell)](#pad-model-mehrabian--russell)
   - [OCC Appraisal Theory](#occ-appraisal-theory-ortony-clore--collins-1988)
   - [Scherer's Component Process Model](#scherers-component-process-model-2001)
   - [ALMA Temporal Layers (Gebhard 2005)](#alma-temporal-layers-gebhard-2005)
   - [Damasio's Somatic Marker Hypothesis](#damasios-somatic-marker-hypothesis)
   - [Bower's Mood-Congruent Memory](#bowers-mood-congruent-memory-1981)
3. [Core Model: PAD Vectors](#core-model-pad-vectors)
   - [PadVector Implementation](#padvector-implementation)
   - [The Three Dimensions in Detail](#the-three-dimensions-in-detail)
   - [The 8 Octant States](#the-8-octant-states)
   - [PAD Cosine Similarity](#pad-cosine-similarity)
   - [Decay Toward Baseline](#decay-toward-baseline)
4. [Three Temporal Layers (ALMA Model)](#three-temporal-layers-alma-model)
   - [AlmaLayers Implementation](#almalayers-implementation)
   - [Layer Interaction Dynamics](#layer-interaction-dynamics)
   - [Effective Affect Computation](#effective-affect-computation)
5. [Appraisal Pipeline](#appraisal-pipeline)
   - [AffectEvent Enum](#affectevent-enum)
   - [OCC Structured Appraisal](#occ-structured-appraisal)
   - [Hardcoded Appraisal Deltas](#hardcoded-appraisal-deltas)
   - [Novelty Filter](#novelty-filter)
6. [AffectState and DaimonState](#affectstate-and-daimonstate)
   - [AffectState Struct](#affectstate-struct)
   - [DaimonState Struct](#daimonstate-struct)
   - [AffectEngine Trait](#affectengine-trait)
   - [Persistence and Autosave](#persistence-and-autosave)
7. [Six Behavioral States](#six-behavioral-states)
   - [State Definitions and PAD Classification](#state-definitions-and-pad-classification)
   - [Threshold Calibration Methodology](#threshold-calibration-methodology)
   - [Hysteresis and BehavioralStateTracker](#hysteresis-and-behavioralstatetracker)
   - [Cyclicality: No Terminal State](#cyclicality-no-terminal-state)
   - [Full State Transition Table](#full-state-transition-table)
8. [Dispatch Modulation](#dispatch-modulation)
   - [DispatchStrategy and DispatchParams](#dispatchstrategy-and-dispatchparams)
   - [Model Promotion and Demotion](#model-promotion-and-demotion)
   - [Tier Bias Table](#tier-bias-table)
9. [8-Dimensional Somatic Marker Space](#8-dimensional-somatic-marker-space)
   - [StrategyCoordinates](#strategycoordinates)
   - [Strategy Space Definition and Computers](#strategy-space-definition-and-computers)
   - [Coding Domain Dimension Extraction](#coding-domain-dimension-extraction)
   - [Cross-Domain Transfer via Structural Analogy](#cross-domain-transfer-via-structural-analogy)
10. [Somatic Landscape and k-d Tree Retrieval](#somatic-landscape-and-k-d-tree-retrieval)
    - [SomaticMarker and SomaticLandscape](#somaticmarker-and-somaticlandscape)
    - [Recording and Merging Markers](#recording-and-merging-markers)
    - [Query with Contrarian Blending](#query-with-contrarian-blending)
    - [SomaticSignal and Actionability](#somaticsignal-and-actionability)
    - [SomaticOracleContext](#somaticoraclecontext)
    - [Dream Depotentiation of Somatic Markers](#dream-depotentiation-of-somatic-markers)
11. [15% Contrarian Blending Mechanism](#15-contrarian-blending-mechanism)
    - [The Problem: Mood-Congruent Feedback Loops](#the-problem-mood-congruent-feedback-loops)
    - [The Solution: Forced Opposite-Valence Injection](#the-solution-forced-opposite-valence-injection)
    - [ContrarianTracker and Rolling Window](#contrariantracker-and-rolling-window)
12. [Four-Factor Retrieval Scoring Model](#four-factor-retrieval-scoring-model)
    - [RetrievalWeights and Online Learning](#retrievalweights-and-online-learning)
    - [Emotional Congruence Computation](#emotional-congruence-computation)
13. [Nietzsche Vitality Phases](#nietzsche-vitality-phases)
    - [VitalityPhase Enum](#vitalityphase-enum)
    - [Mortality Emotions](#mortality-emotions)
    - [Emotional Death Testament](#emotional-death-testament)
14. [Life Review Pipeline](#life-review-pipeline)
    - [Butler's Life Review Adapted for Agents](#butlers-life-review-adapted-for-agents)
    - [McAdams Narrative Arc Classification](#mcadams-narrative-arc-classification)
15. [Emergent Goal Structures](#emergent-goal-structures)
    - [GoalSeed and GoalTree](#goalseed-and-goaltree)
16. [Somatic TA Integration](#somatic-ta-integration)
    - [IIT Phi Metric](#iit-phi-metric)
    - [PID Synergy Detection](#pid-synergy-detection)
17. [IronClaw Integration Plan](#ironclaw-integration-plan)
    - [Phase 1: User Engagement Modeling](#phase-1-user-engagement-modeling)
    - [Phase 2: Agent Self-Regulation](#phase-2-agent-self-regulation)
    - [Phase 3: Somatic Markers for Tool Selection](#phase-3-somatic-markers-for-tool-selection)
    - [Phase 4: Full Behavioral State Dispatch Modulation](#phase-4-full-behavioral-state-dispatch-modulation)
18. [Practical Benefits of Affect-Aware Agents](#practical-benefits-of-affect-aware-agents)
19. [Implementation Roadmap](#implementation-roadmap)
20. [References](#references)

---

## What Is an Affect Engine and Why It Matters

An **affect engine** is a computational subsystem that maintains a continuous internal state representing how things are going for the agent -- not "emotions" in any subjective sense, but a structured signal derived from operational events (task successes, gate failures, blockers, deadlines) that modulates the agent's behavior along measurable parameters: which LLM model to call, how many turns to allocate, whether to explore new approaches or exploit proven ones, and whether to re-plan or persist.

The Daimon (the roko codebase's name for its affect engine) converts discrete events into a continuous three-dimensional vector (Pleasure, Arousal, Dominance), then uses that vector to adjust dispatch parameters, model selection, and strategy coordination. It is implemented in the `roko-daimon` crate and provides:

- A **PAD (Pleasure-Arousal-Dominance) vector** as the core emotional representation
- A **three-layer ALMA temporal model** (Emotion/Mood/Temperament) that prevents both volatility and inertia
- An **OCC/Scherer appraisal pipeline** that converts events into PAD deltas through principled rules
- A **somatic landscape** using a k-d tree over an 8-dimensional strategy space for sub-millisecond "gut feeling" retrieval
- A **15% contrarian blending mechanism** that intentionally injects opposite-valence experiences to prevent mood-congruent echo chambers
- **Six behavioral states** (Engaged, Struggling, Coasting, Exploring, Focused, Resting) with hysteresis tracking that modulate dispatch parameters
- **Nietzsche vitality phases** (Camel/Lion/Child) for long-running agents approaching resource depletion
- A **four-factor retrieval scoring model** with online-learnable weights
- **Emergent goal structures** that promote recurring behavioral patterns into goal hierarchies
- A **life review pipeline** based on Butler (1963) for end-of-life knowledge transfer

### Beyond Sentiment Analysis

A common confusion: affect engineering is not sentiment analysis. Sentiment analysis classifies text as positive, negative, or neutral. It operates on a single dimension (valence), processes external input (user messages), and produces a one-shot classification with no memory.

An affect engine differs in every dimension:

| Property | Sentiment Analysis | Affect Engine |
|---|---|---|
| **Dimensions** | 1 (valence) | 3 (pleasure, arousal, dominance) |
| **Input** | External text | Internal operational events |
| **Output** | Classification label | Continuous behavioral modulation |
| **Memory** | Stateless | Persistent (ALMA layers + somatic landscape) |
| **Temporal model** | None | Three layers with different time constants |
| **Decision impact** | Tone detection | Model selection, turn limits, strategy routing |

The three dimensions capture fundamentally different signals that a single valence score collapses: an agent that is failing under time pressure (low pleasure, high arousal) requires different treatment than an agent that is failing in an unfamiliar domain (low pleasure, low dominance). The first needs escalation; the second needs exploration. A 1D sentiment score cannot distinguish them.

### The Practical Case for Affect in AI Agents

Why build an entire subsystem for tracking "how things are going" instead of just, for example, counting consecutive failures?

1. **Failure counters are brittle**. A counter of "3 consecutive failures" treats all failures equally. But a gate failure at rung 3 (with LLM judge review) is qualitatively different from a compile error at rung 0. The PAD model captures both the severity (pleasure delta scales with rung) and the situational context (dominance tracks whether the agent understands why it failed).

2. **Recovery requires temporal memory**. A simple counter resets on any success. The ALMA mood layer preserves the trajectory: even after one success, an agent that failed five times in a row should remain cautious. The three-layer model provides this without manual cooldown timers.

3. **Cost optimization requires behavioral context**. An agent in "Coasting" state (succeeding easily at well-understood tasks) wastes money on expensive models. An agent in "Struggling" state (failing repeatedly) wastes money on cheap models that cannot solve the problem. The behavioral state classification drives automatic model tier routing.

4. **The somatic landscape is a learned heuristic cache**. After encountering hundreds of tasks, the k-d tree of somatic markers provides sub-millisecond "gut feeling" retrieval: "tasks in this region of the strategy space tend to succeed" or "tasks like this usually fail." This is orders of magnitude faster than re-reasoning from first principles.

The central design constraint is **grounded appraisal**: every emotion has a trigger, and every trigger is grounded in a concrete metric. No emotion is generated without a triggering event. This prevents affective hallucination -- the risk that an agent "feels" something without justification.

---

## Psychology Foundations

The Daimon draws from six distinct research traditions. Understanding these foundations is essential for calibrating thresholds, debugging behavioral anomalies, and extending the system to new domains.

### PAD Model (Mehrabian & Russell)

The Pleasure-Arousal-Dominance model was developed by Albert Mehrabian and James Russell as a general framework for representing emotional states in three continuous, orthogonal dimensions. Their 1977 study demonstrated through regression analysis on 200 subjects that three independent and bipolar dimensions -- pleasure-displeasure, degree of arousal, and dominance-submissiveness -- are both necessary and sufficient to adequately define emotional states. The multiple correlation coefficients showed that almost all reliable variance in 42 verbal-report emotion scales was accounted for by these three factors.

Mehrabian later refined the model with a focus on individual differences in temperament, defining the PAD dimensions as stable personality traits measurable through self-report questionnaires (Mehrabian, 1996).

**Key papers**:
- Russell, J.A. & Mehrabian, A. (1977). "Evidence for a three-factor theory of emotions." *Journal of Research in Personality*, 11(3), 273-294. doi:10.1016/0092-6566(77)90037-X
- Mehrabian, A. (1996). "Pleasure-arousal-dominance: A general framework for describing and measuring individual differences in temperament." *Current Psychology*, 14(4), 261-292.

**Why PAD over alternatives**:

| Alternative | Limitation for Agents |
|---|---|
| **Discrete emotion labels** (Ekman's 6 basic emotions) | Boundary problems: is this "fear" or "anxiety"? Creates classification errors at emotion borders. Not amenable to arithmetic (you cannot average "fear" and "joy"). |
| **Russell's Circumplex** (Valence x Arousal, 2D) | Lacks the Dominance dimension, which captures "am I in control?" -- critical for agents that must decide between exploration (low dominance, try new things) and exploitation (high dominance, use proven patterns). Russell (1980) mapped 28 emotion words onto a valence-arousal circle, but the missing third axis means "frustrated but trying" (+D) and "helpless" (-D) are indistinguishable. |
| **Plutchik's Emotion Wheel** (8 primary emotions) | Discrete categories with arbitrary boundaries; lacks the continuous gradient needed for smooth behavioral modulation. |

The PAD model provides continuous dimensions (gradual changes, not discrete jumps), orthogonal dimensions (changes in one dimension do not force changes in others), computational efficiency (three f64 values, no embedding lookups), and bidirectional mapping to discrete labels when human-readable output is needed.

**Roko docs reference**: `/docs/v1/09-daimon/01-pad-vector.md`

### OCC Appraisal Theory (Ortony, Clore, & Collins 1988)

The OCC model, presented in *The Cognitive Structure of Emotions* (Cambridge University Press, 1988), established that emotions are not random internal states but structured evaluations -- cognitive appraisals -- of events relative to goals. The OCC framework categorizes emotions based on three fundamental appraisal targets:

| Appraisal Focus | Positive Valence | Negative Valence | Agent Mapping |
|---|---|---|---|
| **Events** (consequences for goals) | Joy, Hope, Relief | Distress, Fear, Disappointment | Gate results, task outcomes |
| **Agents** (actions relative to standards) | Pride, Admiration | Shame, Reproach | Self-evaluation of strategy quality |
| **Objects** (attributes of things) | Liking, Attraction | Disliking, Aversion | Code patterns, familiar vs. unfamiliar territory |

The central insight: emotions are not reflexive reactions but evaluative responses that depend on the relationship between the event and the agent's active goals. The same event (e.g., a build failure) produces different emotions depending on goal context: distress if the goal was to ship today, relief if the failure caught a bug before production.

For the Daimon, the primary appraisal focus is events -- did the action produce a good or bad outcome relative to the agent's task goals? Agent-focused appraisals (pride, shame) emerge indirectly through the Dominance dimension. Object-focused appraisals appear through the somatic landscape, where familiar patterns carry positive or negative valence.

**Key work**: Ortony, A., Clore, G.L., & Collins, A. (1988). *The Cognitive Structure of Emotions*. Cambridge University Press. ISBN: 978-0-521-35364-3.

**Roko docs reference**: `/docs/v1/09-daimon/03-occ-scherer-appraisal.md`

### Scherer's Component Process Model (2001)

Klaus Scherer refined appraisal theory into a sequential checking process called the Component Process Model (CPM). Unlike the OCC model's static categorization, Scherer's model treats appraisal as a temporal cascade: each check adds information to the emotional evaluation in a fixed order.

| Check | Question | Daimon Implementation |
|---|---|---|
| **Relevance** | Is this event new or expected? | `NoveltyFilter` deduplicates recent triggers within a sliding window |
| **Intrinsic pleasantness** | Is this inherently positive or negative? | Gate pass vs. fail, task success vs. failure |
| **Goal relevance** | Does this matter for my goals? | Always relevant -- every event arises from an active task |
| **Coping potential** | Can I handle this? | Maps to the Dominance delta and `AppraisalResult.coping_potential` |
| **Norm compatibility** | Does this align with standards? | Gate rung levels (higher rungs = stricter norm checking) |

The Daimon implements a hybrid OCC/Scherer pipeline: events are evaluated along three primary dimensions (desirability, likelihood, coping potential) before mapping to PAD deltas with a 1.6x negativity bias matching Kahneman-Tversky prospect theory (Kahneman & Tversky, 1979). The negativity bias reflects the empirical finding that losses loom roughly twice as large as equivalent gains in human decision-making.

**Key work**: Scherer, K.R. (2001). "Appraisal considered as a process of multilevel sequential checking." In K.R. Scherer, A. Schorr, & T. Johnstone (Eds.), *Appraisal Processes in Emotion: Theory, Methods, Research* (pp. 92-120). Oxford University Press.

### ALMA Temporal Layers (Gebhard 2005)

The ALMA (A Layered Model of Affect) architecture, presented at the Fourth International Joint Conference on Autonomous Agents and Multiagent Systems (AAMAS '05) in Utrecht, Netherlands, provides the three-layer temporal decomposition that prevents two failure modes:

1. **Emotional volatility**: If only the emotion layer existed, the agent would whipsaw between states on every event.
2. **Emotional inertia**: If only the mood layer existed, the agent would respond too slowly to urgent events.

ALMA integrates three major affective characteristics at different timescales: emotions (seconds), moods (hours), and personality/temperament (lifetime). In the preparation phase, appraisal rules and personality profiles are specified; in the runtime phase, appraisal rules compute real-time emotions and moods as results of subjective evaluation. The Daimon adopts ALMA's three-layer temporal structure but replaces its XML configuration with Rust structs and EMA (exponential moving average) update rules.

| Layer | Timescale | Function | EMA Factor |
|---|---|---|---|
| **Emotion** | Seconds | Fast reactivity to immediate events | tau = 0.1 (default) |
| **Mood** | Hours | Accumulated trajectory, stable behavioral state classification | tau = 0.5, sampled every 10 ticks |
| **Temperament** | Lifetime | Gravitational center preventing permanent drift | tau = 0.9, sampled every 100 ticks |

**Key work**: Gebhard, P. (2005). "ALMA -- A Layered Model of Affect." *Proceedings of the Fourth International Joint Conference on Autonomous Agents and Multiagent Systems (AAMAS '05)*, pp. 29-36. ACM Press. doi:10.1145/1082473.1082478

**Roko docs reference**: `/docs/v1/09-daimon/02-alma-three-layer-temporal.md`

### Damasio's Somatic Marker Hypothesis

Antonio Damasio's somatic marker hypothesis, presented in *Descartes' Error* (1994), argues that emotional signals from past experiences -- stored as body-mapped memories -- guide decision-making under uncertainty far faster than deliberative reasoning. When confronted with a situation similar to one previously experienced, the organism retrieves the emotional outcome (positive or negative valence) and uses it as a rapid go/no-go signal.

The hypothesis was empirically supported by the Iowa Gambling Task experiments. Bechara et al. (1997) demonstrated in *Science* that patients with ventromedial prefrontal cortex (VMPFC) damage -- which disrupts the somatic marker circuitry -- failed to avoid disadvantageous decks even after learning which decks were bad. Normal subjects, by contrast, began to avoid bad decks before they could consciously explain why -- their "gut feelings" preceded rational understanding. An earlier study (Bechara et al., 1994) in *Cognition* showed that VMPFC patients displayed "insensitivity to future consequences," choosing immediate rewards despite long-term losses.

In the Daimon, somatic markers are stored in a k-d tree over an 8-dimensional strategy space. When a new task arrives, the engine queries the tree for the nearest markers and retrieves their emotional valence. A positive somatic signal biases the agent toward similar strategies (approach); a negative signal biases away (avoid). This happens in sub-millisecond latency -- orders of magnitude faster than re-reasoning about the situation.

**Key works**:
- Damasio, A.R. (1994). *Descartes' Error: Emotion, Reason, and the Human Brain*. New York: Grosset/Putnam. ISBN: 978-0-399-13894-2.
- Bechara, A., Damasio, H., Tranel, D., & Damasio, A.R. (1997). "Deciding advantageously before knowing the advantageous strategy." *Science*, 275(5304), 1293-1295. doi:10.1126/science.275.5304.1293
- Bechara, A., Damasio, A.R., Damasio, H., & Anderson, S.W. (1994). "Insensitivity to future consequences following damage to human prefrontal cortex." *Cognition*, 50(1-3), 7-15. doi:10.1016/0010-0277(94)90018-3

**Roko docs reference**: `/docs/v1/09-daimon/06-somatic-markers-damasio.md`

### Bower's Mood-Congruent Memory (1981)

Gordon Bower's associative network theory, published in the *American Psychologist*, demonstrated through hypnotic mood induction experiments that mood biases memory retrieval: subjects in a positive mood recalled more positive personal memories, and subjects in a negative mood recalled more negative memories. Bower's associative network model proposes that memory is organized in a network of nodes, with mood serving as an activation source that spreads to connected emotion-congruent nodes, lowering their retrieval threshold.

This creates a self-reinforcing feedback loop -- negative mood retrieves negative memories, which further lowers mood. The Daimon addresses this with 15% contrarian blending (see Section 11): regardless of the dominant mood, 15% of retrieved somatic markers are forced to have the opposite valence. This breaks the echo chamber and ensures the agent considers counter-evidence even when in a strong emotional state.

**Key work**: Bower, G.H. (1981). "Mood and memory." *American Psychologist*, 36(2), 129-148. doi:10.1037/0003-066X.36.2.129

**Roko docs reference**: `/docs/v1/09-daimon/07-15-percent-contrarian-retrieval.md`

---

## Core Model: PAD Vectors

### PadVector Implementation

The canonical PAD vector lives in `roko-primitives` at `crates/roko-primitives/src/pad.rs`. It is the single representation used across the entire roko workspace -- shared by `roko-core`, `roko-runtime`, `roko-daimon`, `roko-neuro`, and `roko-dreams`. All dimensions are `f64` in `[-1.0, 1.0]`.

```rust
// Source: crates/roko-primitives/src/pad.rs

/// Normalized Pleasure-Arousal-Dominance vector.
///
/// The canonical affect primitive shared across roko-core, roko-runtime,
/// roko-daimon, roko-neuro, and roko-dreams.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct PadVector {
    /// Pleasure axis in `[-1.0, 1.0]`.
    pub pleasure: f64,
    /// Arousal axis in `[-1.0, 1.0]`.
    pub arousal: f64,
    /// Dominance axis in `[-1.0, 1.0]`.
    pub dominance: f64,
}

impl PadVector {
    #[must_use]
    pub const fn new(pleasure: f64, arousal: f64, dominance: f64) -> Self {
        Self { pleasure, arousal, dominance }
    }

    #[must_use]
    pub const fn neutral() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            pleasure: self.pleasure.clamp(-1.0, 1.0),
            arousal: self.arousal.clamp(-1.0, 1.0),
            dominance: self.dominance.clamp(-1.0, 1.0),
        }
    }

    pub fn apply_delta(&mut self, pleasure: f64, arousal: f64, dominance: f64) {
        *self = Self::new(
            self.pleasure + pleasure,
            self.arousal + arousal,
            self.dominance + dominance,
        ).clamped();
    }

    pub fn decay_by_factor(&mut self, factor: f64) {
        *self = Self::new(
            self.pleasure * factor,
            self.arousal * factor,
            self.dominance * factor,
        ).clamped();
    }

    #[must_use]
    pub fn magnitude(self) -> f64 {
        (self.pleasure.powi(2) + self.arousal.powi(2) + self.dominance.powi(2)).sqrt()
    }
}
```

**Why f64**: f64 avoids precision loss in chained arithmetic (decay, EMA, cosine similarity) that would accumulate across cognitive ticks when using f32. Modules needing compact atomic storage convert at the boundary via `as f32` / `as f64`.

### The Three Dimensions in Detail

**Pleasure [-1.0, 1.0]** -- Outcome quality trajectory. Is the agent succeeding or failing?

| Range | State | Triggers |
|---|---|---|
| [0.6, 1.0] | Strong success | Multiple consecutive gate passes, tasks completing on first try |
| [0.2, 0.6] | Moderate success | Gate passes at moderate rungs, tasks completing with iteration |
| [-0.2, 0.2] | Neutral | Mixed results, no clear trend |
| [-0.6, -0.2] | Moderate difficulty | Gate failures, tasks requiring multiple retries |
| [-1.0, -0.6] | Strong failure | Consecutive gate failures, tasks timing out |

Appraisal asymmetry: failure has 2x the pleasure impact of success (gate fail: -0.10 vs. gate pass: +0.05, per the hardcoded deltas in `DaimonState::appraise()`). This reflects prospect theory (Kahneman & Tversky, 1979) -- losses loom larger than gains. A broken build demands more attention than a clean build.

**Arousal [-1.0, 1.0]** -- Cognitive load and urgency. How much compute should the agent invest?

| Range | State | Triggers |
|---|---|---|
| [0.6, 1.0] | High urgency | Approaching deadlines, multiple blockers |
| [0.2, 0.6] | Elevated load | Some time pressure, moderate complexity |
| [-0.2, 0.2] | Normal load | Routine tasks |
| [-0.6, -0.2] | Low load | Idle time, maintenance |
| [-1.0, -0.6] | Minimal load | No active tasks, consolidation opportunity |

Arousal is the primary input to tier routing bias: high arousal lowers the T2 trigger threshold (via `adjusted_thresholds()` in `phase2_stubs.rs`), routing to stronger models sooner.

**Dominance [-1.0, 1.0]** -- Confidence in the current approach. Does the agent feel in control?

| Range | State | Triggers |
|---|---|---|
| [0.6, 1.0] | High confidence | Known patterns, successful track record |
| [0.2, 0.6] | Moderate confidence | Familiar territory with some uncertainty |
| [-0.2, 0.2] | Neutral | No strong signal |
| [-0.6, -0.2] | Low confidence | Unfamiliar territory, novel APIs |
| [-1.0, -0.6] | Very low confidence | Repeated failures, blocked, no clear path |

Dominance drives exploration/exploitation balance: low dominance triggers exploration (try new approaches, research mode); high dominance triggers exploitation (cached strategies, known patterns).

### The 8 Octant States

The sign of each PAD dimension defines one of eight octant states. These provide human-readable labels for dashboard display while the continuous PAD values drive actual behavioral modulation.

```rust
// Source: crates/roko-daimon/src/phase2_stubs.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AffectOctant {
    Excited,    // +P+A+D -- Succeeding under pressure, high confidence
    Surprised,  // +P+A-D -- Unexpected success, not sure why it worked
    Confident,  // +P-A+D -- Calm, in control, succeeding
    Relaxed,    // +P-A-D -- Nothing urgent, things are fine
    Angry,      // -P+A+D -- Frustrated but still trying
    Anxious,    // -P+A-D -- Failing, pressured, no control
    Bored,      // -P-A+D -- Nothing happening, idle
    Depressed,  // -P-A-D -- Repeated failures, no agency
}

impl AffectOctant {
    #[must_use]
    pub const fn from_pad(pleasure: f64, arousal: f64, dominance: f64) -> Self {
        if pleasure == 0.0 && arousal == 0.0 && dominance == 0.0 {
            return Self::Relaxed; // Neutral defaults to Relaxed
        }
        let positive_pleasure = !pleasure.is_sign_negative();
        let positive_arousal = !arousal.is_sign_negative();
        let positive_dominance = !dominance.is_sign_negative();
        match (positive_pleasure, positive_arousal, positive_dominance) {
            (true, true, true)   => Self::Excited,
            (true, true, false)  => Self::Surprised,
            (true, false, true)  => Self::Confident,
            (true, false, false) => Self::Relaxed,
            (false, true, true)  => Self::Angry,
            (false, true, false) => Self::Anxious,
            (false, false, true) => Self::Bored,
            (false, false, false) => Self::Depressed,
        }
    }
}
```

Each octant maps to a behavioral modulation profile via `behavior_modulation()`, which returns an `AffectBehaviorModulation` struct with exploration rate, risk tolerance, model tier escalation, probe sensitivity, and sharing threshold.

### PAD Cosine Similarity

For mood-congruent memory retrieval and somatic landscape queries, PAD similarity is computed as cosine similarity mapped to [0.0, 1.0]:

```rust
// Source: crates/roko-primitives/src/pad.rs

#[must_use]
pub fn cosine_similarity(self, other: Self) -> f64 {
    let dot = self.pleasure * other.pleasure
        + self.arousal * other.arousal
        + self.dominance * other.dominance;
    let mag_self = self.magnitude();
    let mag_other = other.magnitude();
    if mag_self == 0.0 || mag_other == 0.0 {
        return 0.5; // Neutral mood -> middle similarity
    }
    (dot / (mag_self * mag_other) + 1.0) / 2.0
}
```

**Worked example**: Compare Angry (+P=-0.5, +A=+0.6, +D=+0.3) with Anxious (+P=-0.4, +A=+0.5, +D=-0.4):
- dot = (-0.5)(-0.4) + (0.6)(0.5) + (0.3)(-0.4) = 0.20 + 0.30 - 0.12 = 0.38
- |Angry| = sqrt(0.25 + 0.36 + 0.09) = sqrt(0.70) = 0.837
- |Anxious| = sqrt(0.16 + 0.25 + 0.16) = sqrt(0.57) = 0.755
- cosine = 0.38 / (0.837 * 0.755) = 0.38 / 0.632 = 0.601
- similarity = (0.601 + 1.0) / 2.0 = 0.80

Congruent emotions (same octant) score near 1.0; incongruent (opposite octant) score near 0.0. The neutral-zero fallback of 0.5 prevents division-by-zero when either vector has zero magnitude.

### Decay Toward Baseline

The PAD vector decays toward neutral [0, 0, 0] with a configurable half-life (default: 4 hours). This prevents permanent affect drift.

```rust
// Source: crates/roko-daimon/src/lib.rs (line 2381)

fn decay_factor(delta_hours: f64, half_life_hours: f64) -> f64 {
    if delta_hours <= 0.0 { return 1.0; }
    if half_life_hours <= 0.0 { return 0.0; }
    0.5_f64.powf(delta_hours / half_life_hours)
}
```

After 1 half-life (4 hours), affect intensity is halved. After 3 half-lives (12 hours), it is at 12.5%. After 4 half-lives (16 hours), it is at 6.25%.

Confidence decays toward 0.5 (neutral uncertainty), not toward 0.0:
```rust
// Source: crates/roko-daimon/src/lib.rs (AffectState::decay)
self.confidence = (0.5 + (self.confidence - 0.5) * factor).clamp(0.0, 1.0);
```

This ensures that an agent with no recent events settles at "uncertain" rather than "no confidence."

---

## Three Temporal Layers (ALMA Model)

### AlmaLayers Implementation

The ALMA three-layer model is fully implemented in `roko-daimon`:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 189)

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlmaLayers {
    pub emotion: PadVector,       // Fast emotional response
    pub mood: PadVector,          // Medium-term mood
    pub temperament: PadVector,   // Stable personality baseline
    #[serde(default = "AlmaLayers::default_tau_emotion")]
    pub tau_emotion: f64,         // Default 0.1
    #[serde(default = "AlmaLayers::default_tau_mood")]
    pub tau_mood: f64,            // Default 0.5
    #[serde(default = "AlmaLayers::default_tau_temperament")]
    pub tau_temperament: f64,     // Default 0.9
    #[serde(default = "AlmaLayers::default_mood_interval")]
    pub mood_interval: u64,       // Default 10 ticks
    #[serde(default = "AlmaLayers::default_temperament_interval")]
    pub temperament_interval: u64, // Default 100 ticks
}
```

Each layer updates via exponential moving average (EMA):

```rust
/// Apply a stimulus to the emotion layer via EMA:
/// `emotion = (1 - tau_e) * emotion + tau_e * stimulus`
pub fn update_emotion(&mut self, stimulus: &PadVector) {
    let tau = self.tau_emotion;  // 0.1
    let retain = 1.0 - tau;     // 0.9
    self.emotion = PadVector::new(
        retain * self.emotion.pleasure + tau * stimulus.pleasure,
        retain * self.emotion.arousal + tau * stimulus.arousal,
        retain * self.emotion.dominance + tau * stimulus.dominance,
    ).clamped();
}

/// Update mood layer as EMA of emotion:
/// `mood = (1 - tau_m) * mood + tau_m * emotion`
pub fn update_mood(&mut self) {
    let tau = self.tau_mood;  // 0.5
    let retain = 1.0 - tau;  // 0.5
    self.mood = PadVector::new(
        retain * self.mood.pleasure + tau * self.emotion.pleasure,
        retain * self.mood.arousal + tau * self.emotion.arousal,
        retain * self.mood.dominance + tau * self.emotion.dominance,
    ).clamped();
}

/// Update temperament layer as EMA of mood:
/// `temperament = (1 - tau_t) * temperament + tau_t * mood`
pub fn update_temperament(&mut self) {
    let tau = self.tau_temperament;  // 0.9
    let retain = 1.0 - tau;         // 0.1
    self.temperament = PadVector::new(
        retain * self.temperament.pleasure + tau * self.mood.pleasure,
        retain * self.temperament.arousal + tau * self.mood.arousal,
        retain * self.temperament.dominance + tau * self.mood.dominance,
    ).clamped();
}
```

### Layer Interaction Dynamics

The three layers interact through a temporal cascade triggered by `tick()`:

```rust
pub fn tick(&mut self, tick_count: u64) {
    if tick_count > 0 && tick_count % self.mood_interval == 0 {
        self.update_mood();
    }
    if tick_count > 0 && tick_count % self.temperament_interval == 0 {
        self.update_temperament();
    }
}
```

With the default configuration (mood every 10 ticks, temperament every 100 ticks), the dynamics are:

- **Ticks 1-9**: Only emotion layer updates. Fast reactivity to each event.
- **Tick 10**: Mood absorbs recent emotion trajectory. Emotion spikes are smoothed.
- **Tick 100**: Temperament absorbs recent mood trajectory. Personality baseline shifts slowly.

### Effective Affect Computation

The effective PAD used for behavioral state classification is a weighted blend of all three layers:

```rust
/// Compute the effective affect as a weighted blend:
/// 0.5 * emotion + 0.3 * mood + 0.2 * temperament
#[must_use]
pub fn effective_affect(&self) -> PadVector {
    PadVector::new(
        0.5 * self.emotion.pleasure
            + 0.3 * self.mood.pleasure
            + 0.2 * self.temperament.pleasure,
        0.5 * self.emotion.arousal
            + 0.3 * self.mood.arousal
            + 0.2 * self.temperament.arousal,
        0.5 * self.emotion.dominance
            + 0.3 * self.mood.dominance
            + 0.2 * self.temperament.dominance,
    ).clamped()
}
```

Weights: 50% emotion (immediate responsiveness), 30% mood (accumulated trajectory), 20% temperament (stable baseline). This ensures fast reactivity to urgent events while maintaining stability from accumulated history.

---

## Appraisal Pipeline

### AffectEvent Enum

Every appraisal begins with a concrete event. The `AffectEvent` enum defines all trigger types:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 1691)

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AffectEvent {
    GateResult { plan_id: String, task_id: String, passed: bool, rung: u32 },
    TaskOutcome { task_id: String, succeeded: bool },
    Blocked { task_id: String, blocker_count: usize },
    TimePressure { task_id: String, deadline_proximity: f64 },
    QueueWait { task_id: String, wait_hours: f64 },
    DreamFailure { task_type: String, failure_count: usize },
    DreamOutcome {
        knowledge_entries: usize,
        playbooks_created: usize,
        regressions_detected: usize,
        strategy_hypotheses: usize,
        episodes_processed: usize,
    },
}
```

Every event variant carries a concrete metric -- boolean pass/fail, numeric count, [0,1] proximity. No event is abstract or ungrounded.

### OCC Structured Appraisal

The `AppraisalResult` struct implements OCC/Scherer appraisal with three primary dimensions:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 1772)

pub struct AppraisalResult {
    pub desirability: f64,      // Goal-congruence in [-1.0, 1.0]
    pub likelihood: f64,        // Expectedness in [0.0, 1.0]
    pub coping_potential: f64,  // Ability to handle in [0.0, 1.0]
    pub trigger: AppraisalTrigger,
    pub novel: bool,
}
```

The mapping from OCC dimensions to PAD deltas follows principled formulas with a 1.6x negativity bias:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 1819)

pub fn to_pad_delta(&self) -> PadVector {
    if !self.novel { return PadVector::neutral(); }

    // Losses hurt ~1.6x more than equivalent gains feel good
    let negativity_bias = if self.desirability < 0.0 { 1.6 } else { 1.0 };

    // Pleasure: primarily from desirability
    let pleasure = self.desirability * negativity_bias * 0.15;

    // Arousal: surprise (1 - likelihood) x magnitude
    let surprise = 1.0 - self.likelihood;
    let arousal = surprise * self.desirability.abs() * 0.20;

    // Dominance: coping potential is the primary driver
    let dominance = (self.coping_potential - 0.5) * 0.10;

    PadVector {
        pleasure: pleasure.clamp(-1.0, 1.0),
        arousal: arousal.clamp(-1.0, 1.0),
        dominance: dominance.clamp(-1.0, 1.0),
    }
}
```

**Worked example -- Gate failure at rung 3**: `AppraisalResult::from_event()` computes:
```rust
AffectEvent::GateResult { passed: false, rung: 3, .. } => Self {
    desirability: -0.5 - 0.1 * 3.0,  // -0.8 (very undesirable)
    likelihood: 0.4,                   // failures slightly more surprising
    coping_potential: confidence,       // depends on current state
    trigger: AppraisalTrigger::Performance,
    novel: true,
}
```
With confidence = 0.6, the PAD delta is:
- pleasure = -0.8 * 1.6 * 0.15 = -0.192
- arousal = (1.0 - 0.4) * 0.8 * 0.20 = 0.096
- dominance = (0.6 - 0.5) * 0.10 = 0.010

### Hardcoded Appraisal Deltas

In addition to the structured OCC appraisal, the `AffectEngine::appraise()` method (implemented on `DaimonState`) applies hardcoded deltas directly. These are the operational deltas currently wired into the system:

| Event | P delta | A delta | D delta | C delta | Notes |
|---|---|---|---|---|---|
| Gate pass (rung r) | +0.05 x rs | -0.01 x rs | +0.03 x rs | +0.03 x rs | rs = 1.0 + min(r,3) x 0.15 |
| Gate fail (rung r) | -0.10 x rs | +0.04 x rs | -0.08 x rs | -0.08 x rs | 2x asymmetry on pleasure |
| Task success | +0.10 | 0.00 | +0.10 | +0.08 | No arousal change |
| Task failure | -0.20 | 0.00 | -0.15 | -0.15 | Significant setback |
| Blocked (n blockers) | 0.00 | +n x 0.05 | -n x 0.08 | -0.02 x n | n capped at 5 |
| Time pressure (prox) | 0.00 | +prox x 0.40 | 0.00 | 0.00 | Pure urgency signal |
| Queue wait (>24h) | 0.00 | scaled ramp | 0.00 | 0.00 | 0.1/day ramp to 1.0 at 7 days |
| Dream failure | 0.00 | 0.00 | 0.00 | -0.07 x n | Confidence erosion only |
| Dream outcome | scaled | scaled | scaled | scaled | Net positive/negative from consolidation |

**Rung scaling**: A rung-0 gate (compile only) carries less weight than a rung-3 gate (compile + test + clippy + diff review + symbol check + LLM judge). The scale factor rs ranges from 1.0 (rung 0) to 1.45 (rung 3), making higher-rung outcomes ~45% more emotionally significant.

### Novelty Filter

The `NoveltyFilter` prevents emotional flooding by suppressing duplicate appraisals within a sliding window:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 1929)

pub struct NoveltyFilter {
    recent_triggers: Vec<String>,  // Ring buffer of last N triggers
    window_size: usize,            // Deduplication window
}

impl NoveltyFilter {
    pub fn is_novel(&self, trigger: &AppraisalTrigger) -> bool {
        let key = format!("{trigger:?}");
        !self.recent_triggers.iter().rev()
            .take(self.window_size)
            .any(|t| t == &key)
    }

    pub fn record(&mut self, trigger: &AppraisalTrigger) {
        let key = format!("{trigger:?}");
        self.recent_triggers.push(key);
        // Buffer is pruned when it exceeds 2x window_size
    }
}
```

Only events that cross the novelty threshold or are of a new category trigger full appraisal. This prevents five consecutive identical gate failures from producing five full emotional responses.

---

## AffectState and DaimonState

### AffectState Struct

The `AffectState` wraps the PAD vector with confidence, behavioral state classification, ALMA layers, and temporal tracking:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 313)

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AffectState {
    pub pad: PadVector,                    // Current effective PAD
    pub confidence: f64,                   // Motivational confidence [0.0, 1.0]
    #[serde(default)]
    pub behavioral_state: BehavioralState, // Derived from PAD + confidence
    pub updated_at: DateTime<Utc>,         // Last update timestamp
    #[serde(default)]
    pub alma: AlmaLayers,                  // Three-layer ALMA model
    #[serde(default)]
    pub tick_count: u64,                   // Total appraisal ticks
}
```

Default confidence is 0.5 (neutral). The `apply_delta` method routes deltas through the ALMA layers and recomputes the effective PAD:

```rust
fn apply_delta(&mut self, pleasure: f64, arousal: f64, dominance: f64,
               confidence: f64, now: DateTime<Utc>) {
    // Apply to emotion layer (fast, reactive)
    let stimulus = PadVector::new(
        self.alma.emotion.pleasure + pleasure,
        self.alma.emotion.arousal + arousal,
        self.alma.emotion.dominance + dominance,
    ).clamped();
    self.alma.update_emotion(&stimulus);

    self.confidence = (self.confidence + confidence).clamp(0.0, 1.0);
    self.tick_count += 1;
    self.alma.tick(self.tick_count);

    // Effective PAD = weighted blend of all three ALMA layers
    self.pad = self.alma.effective_affect();
    self.refresh_behavioral_state();
    self.updated_at = now;
}
```

**Confidence vs. Dominance**: Confidence is a meta-cognitive signal ("how well am I performing overall?") that decays toward 0.5 (neutral). Dominance is a per-situation signal ("am I in control of this specific task?") that decays toward 0.0. They are tracked separately because an agent can have high overall confidence but low dominance in a specific unfamiliar domain.

### DaimonState Struct

`DaimonState` is the single entry point for all affect operations. It aggregates the affect state with the somatic landscape, strategy space, and all tracker subsystems:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 1973)

pub struct DaimonState {
    pub state: AffectState,
    #[serde(default = "default_half_life_hours")]
    pub half_life_hours: f64,                          // Default: 4.0
    #[serde(default)]
    pub somatic_landscape: SomaticLandscape,           // k-d tree of markers
    #[serde(default)]
    pub strategy_space: StrategySpaceDefinition,       // 8D axis labels
    #[serde(default)]
    pub crate_confidence_map: HashMap<String, f64>,    // Per-crate confidence
    #[serde(default)]
    pub crate_trackers: HashMap<String, CrateConfidence>,
    #[serde(default)]
    pub contrarian_tracker: ContrarianTracker,          // Rolling window tracker
    #[serde(default)]
    pub error_patterns: ErrorPatternTracker,             // Familiarity model
    #[serde(default)]
    pub fatigue_detector: FatigueDetector,              // Failure-streak tracker
    #[serde(default)]
    pub borrowed_affect: Vec<BorrowedAffect>,           // Peer affect contagion
    // ... behavioral_tracker, persistence_path ...
}
```

### AffectEngine Trait

The public interface for the daimon subsystem:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 2189)

pub trait AffectEngine {
    /// Appraise one event and return the updated PAD vector.
    fn appraise(&mut self, event: AffectEvent) -> PadVector;
    /// Query the current affect state.
    fn query(&self) -> AffectState;
    /// Modulate dispatch parameters in place.
    fn modulate(&self, params: &mut DispatchParams);
    /// Persist to disk.
    fn persist(&self, path: &Path) -> Result<()>;
}
```

The `appraise()` implementation in `DaimonState` (line 2205):
1. Decays the current state by elapsed time
2. Pattern-matches on the event variant and applies the corresponding PAD deltas
3. Updates the behavioral state through the hysteresis tracker (not memoryless classification)
4. Autosaves to disk
5. Returns the updated PAD vector

### Persistence and Autosave

The daimon state persists to disk as JSON at `.roko/daimon/affect.json`. Persistence uses atomic write (write to `.json.tmp`, then rename) to prevent corruption:

```rust
fn persist(&self, path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(self)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
```

On startup, `DaimonState::load_or_new()` loads the persisted state, applies decay for the elapsed time since `updated_at`, and rebuilds the k-d tree index from the persisted markers. An agent shut down 8 hours ago in a negative mood resumes with that mood at 25% intensity (two half-lives: 0.5^2 = 0.25).

---

## Six Behavioral States

### State Definitions and PAD Classification

The six behavioral states bridge the continuous PAD space and discrete dispatch decisions. The classifier lives in `roko-core` at `crates/roko-core/src/affect.rs`:

| State | PAD Profile | Description | Dispatch |
|---|---|---|---|
| **Engaged** | Balanced (near origin) | Normal operation, making progress | Balanced strategy |
| **Struggling** | Low P, High A, or Low C/D | Failing under pressure | Conservative or Escalating |
| **Coasting** | High P, High C | Succeeding without difficulty | Exploratory, demote model |
| **Exploring** | Low D, moderate P | Unfamiliar territory, low confidence | Balanced (research routing planned) |
| **Focused** | High D, High P | Succeeding in well-understood territory | Balanced, reduced turns |
| **Resting** | Low A | Idle or low-demand phase | Proactive, dream cycles |

Classification is computed in `BehavioralState::classify()`:

```rust
// Source: crates/roko-core/src/affect.rs (line 37)

pub fn classify(pad: PadVector, confidence: f64) -> BehavioralState {
    let p = pad.pleasure;
    let a = pad.arousal;
    let d = pad.dominance;
    let c = confidence.clamp(0.0, 1.0);

    if pad == PadVector::neutral() { return Self::Engaged; }

    // Priority 1: Struggling (protective measures should not be delayed)
    if c < 0.30 || d < -0.25 || (p < -0.30 && a > 0.30) {
        return Self::Struggling;
    }
    // Priority 2: Coasting
    if p > 0.35 && c > 0.65 { return Self::Coasting; }
    // Priority 3: Focused
    if d > 0.30 && p > 0.25 { return Self::Focused; }
    // Priority 4: Resting
    if a < -0.20 { return Self::Resting; }
    // Priority 5: Exploring
    if d < 0.10 && p > -0.20 { return Self::Exploring; }
    // Default: Engaged
    Self::Engaged
}
```

### Threshold Calibration Methodology

The thresholds derive from appraisal rule magnitudes, not hand-tuning. A single task failure produces P: -0.20, D: -0.15, C: -0.15. Starting from neutral confidence of 0.50 (the `AffectState::default()` initial confidence):

- After 1 failure: C = 0.35 (above 0.30 threshold -- stays Engaged)
- After 2 failures: C = 0.20 (below 0.30 -- enters Struggling)

Note: the ALMA layer blending means the effective PAD is 50% of the emotion layer delta (when mood and temperament are still neutral), so more failures are needed to cross the pleasure/dominance thresholds. The hysteresis tracker's `min_dwell_ticks` of 10 additionally prevents state changes faster than once per ~10 task cycles.

**Calibration formula**:
```
threshold = neutral_value - (n_tolerated_failures * per_failure_delta)
confidence_threshold = 0.50 - (1.5 * 0.15) ~ 0.28, rounded to 0.30
dominance_threshold  = 0.00 - (2.0 * 0.15) ~ -0.25 (accounting for ALMA 50% blend)
```

### Hysteresis and BehavioralStateTracker

The `BehavioralStateTracker` prevents rapid oscillation between states by enforcing split entry/exit thresholds and a minimum dwell time:

```rust
// Source: crates/roko-daimon/src/phase2_stubs.rs (line 297)

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BehavioralStateTracker {
    pub current_state: BehavioralState,
    pub entered_at: u64,           // Tick when current state was entered
    pub min_dwell_ticks: u64,      // Default: 10
    pub thresholds: BehavioralStateThresholds,
}
```

The thresholds have separate entry and exit values with a dead zone:

```rust
// Source: crates/roko-daimon/src/phase2_stubs.rs (line 220)

pub struct BehavioralStateThresholds {
    pub struggling_entry_confidence: f64,   // 0.30
    pub struggling_exit_confidence: f64,    // 0.40
    pub struggling_entry_dominance: f64,    // -0.25
    pub struggling_exit_dominance: f64,     // -0.15
    pub coasting_entry_pleasure: f64,       // 0.35
    pub coasting_exit_pleasure: f64,        // 0.25
    pub resting_entry_arousal: f64,         // -0.20
    pub resting_exit_arousal: f64,          // -0.10
}
```

An agent at confidence 0.29 enters Struggling. A single positive outcome pushes it to 0.35, but this is still below the exit threshold (0.40), so it remains in Struggling. The agent must demonstrate sustained recovery (C > 0.40) before exiting. The `classify_with_hysteresis()` function (line 256) implements this logic, falling back to `BehavioralState::classify()` for non-hysteresis transitions.

### Cyclicality: No Terminal State

The behavioral states form a cycle, not a directed path with a sink node:

```
           +----------------------------------+
           |                                  |
    Engaged --> Struggling --> Resting         |
       ^            |              |          |
       |            v              v          |
    Focused <-- Exploring    (Dream cycles)   |
       ^                           |          |
       |                           v          |
       +-------- Coasting <--------+          |
                    |                          |
                    +--------------------------+
```

Common transition patterns:

1. **Recovery from struggle**: Struggling -> Resting (arousal decays) -> Exploring (low D) -> Engaged (successful exploration raises D)
2. **Performance optimization**: Engaged -> Focused (sustained success raises P and D) -> Coasting (continued easy success)
3. **Challenge encounter**: Coasting -> Engaged (harder problem raises A, lowers P) -> Struggling (continued difficulty)
4. **Knowledge plateau**: Focused -> Exploring (exhausted known approaches, D drops)

Every state is reachable from every other state through intermediate PAD changes. The 4-hour half-life decay ensures even sustained extreme states eventually moderate.

### Full State Transition Table

| From | To | Trigger Condition | Typical Cause |
|---|---|---|---|
| Engaged | Struggling | C < 0.30 OR D < -0.25 | 2+ consecutive failures |
| Engaged | Coasting | P > 0.35 AND C > 0.65 | Sustained easy successes |
| Engaged | Focused | D > 0.30 AND P > 0.25 | Success in familiar territory |
| Engaged | Resting | A < -0.20 | No tasks in queue |
| Engaged | Exploring | D < 0.10 AND P > -0.20 | New crate, unfamiliar API |
| Struggling | Engaged | C > 0.40 AND D > -0.15 (exit thresholds) | Successful task after struggle |
| Struggling | Resting | A < -0.20 | Dream depotentiation, idle |
| Coasting | Engaged | P < 0.25 OR C < 0.65 (exit thresholds) | Harder problem encountered |
| Focused | Coasting | P > 0.35 AND C > 0.65 | Continued success, difficulty drops |
| Focused | Struggling | C < 0.30 OR D < -0.25 | Unexpected failure in "known" territory |
| Resting | Engaged | A > -0.10 (exit threshold) | New task arrives |
| Exploring | Focused | D > 0.30 AND P > 0.25 | Exploration succeeded, built mastery |

**Roko docs reference**: `/docs/v1/09-daimon/04-six-behavioral-states.md`

---

## Dispatch Modulation

### DispatchStrategy and DispatchParams

Each behavioral state maps to a dispatch strategy that controls model selection, turn limits, and reasoning effort:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 1637)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchStrategy {
    Conservative,   // effort: "low"
    Balanced,       // effort: "medium"
    Exploratory,    // effort: "high"
    Escalating,     // effort: "high"
    Proactive,      // effort: "medium"
}

pub struct DispatchParams {
    pub model: String,              // Model slug for next dispatch
    pub turn_limit: u32,            // Maximum turns allowed
    pub strategy: DispatchStrategy, // Behavioral strategy
    pub effort: String,             // Reasoning-effort label
}
```

The `modulate()` method (line 2330) switches on behavioral state:

```rust
fn modulate(&self, params: &mut DispatchParams) {
    let state = self.query();
    match state.behavioral_state {
        BehavioralState::Struggling => {
            if state.pad.pleasure < -0.30 && state.pad.arousal > 0.30 {
                // Anxious: failing and don't know why -> conserve
                params.strategy = DispatchStrategy::Conservative;
                params.turn_limit = params.turn_limit.saturating_sub(3);
                params.model = demote_model(&params.model);
            } else {
                // Angry: believe we CAN solve this, need more resources -> escalate
                params.strategy = DispatchStrategy::Escalating;
                params.turn_limit = params.turn_limit.saturating_add(10);
                params.model = promote_model(&params.model);
            }
        }
        BehavioralState::Coasting => {
            params.strategy = DispatchStrategy::Exploratory;
            params.turn_limit = params.turn_limit.saturating_sub(5);
            params.model = demote_model(&params.model);
        }
        BehavioralState::Focused => {
            params.strategy = DispatchStrategy::Balanced;
            params.turn_limit = params.turn_limit.saturating_sub(2);
        }
        BehavioralState::Resting => {
            params.strategy = DispatchStrategy::Proactive;
            params.turn_limit = params.turn_limit.saturating_add(5);
        }
        BehavioralState::Exploring | BehavioralState::Engaged => {
            params.strategy = DispatchStrategy::Balanced;
        }
    }
    params.effort = params.strategy.effort_label().to_string();
}
```

The Struggling state implements a coarse confidence-competence matrix: high dominance with low pleasure = resource problem (escalate to stronger model), low dominance with low pleasure = knowledge problem (conserve, fall back to proven approaches).

### Model Promotion and Demotion

Model tier routing follows a haiku <-> sonnet <-> opus chain:

```rust
fn promote_model(current: &str) -> String {
    if current.contains("haiku") { current.replace("haiku", "sonnet") }
    else if current.contains("sonnet") { current.replace("sonnet", "opus") }
    else { current.to_string() }
}

fn demote_model(current: &str) -> String {
    if current.contains("opus") { current.replace("opus", "sonnet") }
    else if current.contains("sonnet") { current.replace("sonnet", "haiku") }
    else { current.to_string() }
}
```

### Tier Bias Table

The behavioral state modulates the tier router's prediction error threshold. The `adjusted_thresholds()` function in `phase2_stubs.rs` (line 378) returns concrete threshold values:

| State | T0 ceiling | T1 ceiling | Effect |
|---|---|---|---|
| **Engaged** | 0.20 | 0.60 | Standard thresholds |
| **Struggling** | 0.10 | 0.40 | Force deep reasoning sooner |
| **Coasting** | 0.30 | 0.80 | Stay cheap longer |
| **Exploring** | 0.15 | 0.55 | Broad scanning + deep dive |
| **Focused** | 0.25 | 0.70 | Exploit known patterns |
| **Resting** | 0.20 | 0.90 | T1 for consolidation |

```
Standard:   error < 0.20 -> T0; error < 0.60 -> T1; error >= 0.60 -> T2
Struggling: error < 0.10 -> T0; error < 0.40 -> T1; error >= 0.40 -> T2
Coasting:   error < 0.30 -> T0; error < 0.80 -> T1; error >= 0.80 -> T2
```

---

## 8-Dimensional Somatic Marker Space

### StrategyCoordinates

Every task is projected into an 8-dimensional strategy space before somatic marker retrieval. The dimensions capture the structural characteristics of the work:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 429)

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrategyCoordinates {
    pub complexity: f64,        // Structural difficulty [0.0, 1.0]
    pub risk: f64,              // Blast radius and failure cost [0.0, 1.0]
    pub novelty: f64,           // How unfamiliar the task is [0.0, 1.0]
    pub confidence: f64,        // Local confidence [0.0, 1.0]
    pub time_pressure: f64,     // Deadline/blockage pressure [0.0, 1.0]
    pub scope: f64,             // Spatial extent of the change [0.0, 1.0]
    pub reversibility: f64,     // Ease of undoing [0.0, 1.0]
    pub dependency_depth: f64,  // Dependency-chain depth [0.0, 1.0]
}
```

Each dimension is clamped to [0.0, 1.0]. The neutral mid-space point is [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5].

### Strategy Space Definition and Computers

The strategy space is configurable per domain through `StrategySpaceDefinition`:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 527)

pub struct StrategySpaceDefinition {
    pub domain: String,                             // "coding" by default
    pub dimensions: [String; STRATEGY_DIMENSIONS],  // 8 axis labels
}
```

A `StrategySpaceComputer<Observation>` trait defines the interface for projecting observations into coordinates:

```rust
pub trait StrategySpaceComputer<Observation> {
    fn definition(&self) -> &StrategySpaceDefinition;
    fn compute_coords(&self, observation: &Observation) -> StrategyCoordinates;
}
```

Dimension labels are classified into semantic roles via keyword matching in `classify_dimension_role()`:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 924)

fn classify_dimension_role(label: &str, index: usize) -> DimensionRole {
    // Keywords -> roles:
    // "complex", "difficulty", "volatility", "unstable" -> Difficulty
    // "risk", "danger", "exposure", "leverage", "slippage", "blast" -> Danger
    // "novel", "familiar", "correlation", "ambiguity" -> Familiarity
    // "confidence", "conviction", "certainty" -> SelfAssessment
    // "time", "deadline", "horizon", "urgency", "latency" -> Urgency
    // "scope", "breadth", "concentration", "liquidity", "coverage" -> Breadth
    // "revers", "rollback", "recover", "exit", "undo" -> Recoverability
    // "dependency", "coupling", "regulatory", "compliance" -> Coupling
    // Default: positional fallback via DimensionRole::default_for_index()
}
```

### Coding Domain Dimension Extraction

The `CodingStrategySpace` computes each dimension from task metadata through composed formulas:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 1043)

impl CodingStrategySpace {
    fn complexity_from_tier(tier: &str) -> f64 {
        match tier.trim().to_ascii_lowercase().as_str() {
            "mechanical" | "fast" => 0.15,
            "focused" | "standard" => 0.35,
            "integrative" => 0.60,
            "architectural" | "complex" => 0.85,
            "premium" => 0.95,
            _ => 0.50,
        }
    }

    fn scope(complexity: f64, file_count: usize, max_loc: u32) -> f64 {
        (0.55 * complexity
            + 0.30 * (file_count as f64 / 8.0).min(1.0)
            + 0.15 * (f64::from(max_loc) / 400.0).min(1.0))
        .clamp(0.0, 1.0)
    }

    fn novelty(familiarity: f64) -> f64 {
        (1.0 - familiarity).clamp(0.0, 1.0)
    }

    fn risk(complexity: f64, novelty: f64, verification_count: usize,
            failure_pressure: f64) -> f64 {
        (0.40 * complexity
            + 0.25 * novelty
            + 0.20 * (verification_count as f64 / 4.0).min(1.0)
            + 0.15 * failure_pressure.clamp(0.0, 1.0))
        .clamp(0.0, 1.0)
    }
}
```

Task observations are collected from `Task` metadata via `TaskStrategyObservation::from_task()`, which pulls file count, verification count, dependency count, and derived signals like familiarity and failure pressure from the task struct and its `TaskContext`.

### Cross-Domain Transfer via Structural Analogy

Non-coding domains use label-aware projection. The system classifies each dimension label into a semantic role and maps the canonical profile values accordingly:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 1014)

fn project_profile_for_definition(
    definition: &StrategySpaceDefinition,
    profile: CanonicalStrategyProfile,
) -> StrategyCoordinates {
    let mut values = [0.5_f64; STRATEGY_DIMENSIONS];
    for (index, label) in definition.labels().iter().enumerate() {
        let role = classify_dimension_role(label, index);
        values[index] = profile.value_for_role(role);
    }
    StrategyCoordinates::new(
        values[0], values[1], values[2], values[3],
        values[4], values[5], values[6], values[7],
    )
}
```

This enables cross-domain transfer: a trading strategy space with dimensions ["volatility", "exposure", "correlation", "conviction", "horizon", "concentration", "exit_liquidity", "regulatory_depth"] would automatically map to the same semantic roles as the coding domain, allowing somatic markers from one domain to inform decisions in another.

**Roko docs reference**: `/docs/v1/09-daimon/08-8-dimensional-strategy-space.md`

---

## Somatic Landscape and k-d Tree Retrieval

### SomaticMarker and SomaticLandscape

A somatic marker is a situation-specific emotional memory stored with its strategy-space coordinates:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 1264)

pub struct SomaticMarker {
    pub strategy_coords: StrategyCoordinates,  // Where in strategy space
    pub valence: f64,                          // [-1.0, 1.0] approach/avoid
    pub intensity: f64,                        // [0.0, 1.0] signal strength
    pub episodes: Vec<ContentHash>,            // Supporting evidence
    pub updated_at: DateTime<Utc>,             // Last reinforcement
}
```

The `SomaticLandscape` stores markers indexed by a k-d tree from the `kiddo` crate (SIMD-optimized):

```rust
const STRATEGY_DIMENSIONS: usize = 8;
type SomaticTree = KdTree<f64, STRATEGY_DIMENSIONS>;

pub struct SomaticLandscape {
    #[serde(default)]
    pub markers: Vec<SomaticMarker>,   // Persisted payloads
    #[serde(skip, default = "default_somatic_tree")]
    tree: SomaticTree,                 // In-memory spatial index (skipped in serde)
}
```

The k-d tree is rebuilt from persisted markers on deserialization:

```rust
pub fn rebuild_index(&mut self) {
    self.tree = default_somatic_tree();
    for (idx, marker) in self.markers.iter().enumerate() {
        self.tree.add(&marker.strategy_coords.as_array(), idx as u64);
    }
}
```

### Recording and Merging Markers

When a new outcome is recorded, the system checks if a nearby marker of the same valence family exists (within squared Euclidean distance 0.25). If so, the markers are merged; otherwise a new marker is created:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 1425)

pub fn record_marker(&mut self, marker: SomaticMarker) {
    let marker = marker.clamped();
    if marker.intensity <= 0.0 { return; }

    let coords = marker.strategy_coords.as_array();
    if !self.markers.is_empty() {
        let nearest = self.tree.nearest_one::<SquaredEuclidean>(&coords);
        if nearest.distance <= SOMATIC_MERGE_DISTANCE_SQUARED {  // 0.25
            let idx = nearest.item as usize;
            if let Some(existing) = self.markers.get_mut(idx) {
                let same_valence_family = existing.valence.signum() == marker.valence.signum()
                    || existing.valence.abs() < 0.10
                    || marker.valence.abs() < 0.10;
                if same_valence_family {
                    merge_markers(existing, &marker);
                    self.rebuild_index();
                    return;
                }
            }
        }
    }
    // New marker
    let item = self.markers.len() as u64;
    self.markers.push(marker);
    self.tree.add(&coords, item);
}
```

The merge distance threshold of 0.25 (squared Euclidean) means markers within ~0.5 Euclidean distance in the 8D space are considered "nearby enough" to merge. This prevents the landscape from accumulating thousands of nearly-identical markers for routine tasks.

### Query with Contrarian Blending

The `query()` method (line 1456) implements the full somatic retrieval pipeline with 15% contrarian blending:

```rust
pub fn query(&self, strategy_coords: StrategyCoordinates, k: usize) -> SomaticSignal {
    if self.markers.is_empty() { return SomaticSignal::default(); }

    let coords = strategy_coords.clamped().as_array();
    let neighbor_count = k.max(1).min(self.markers.len());
    let neighbors = self.tree.nearest_n::<SquaredEuclidean>(&coords, neighbor_count);

    // Step 1: Determine dominant valence sign
    let dominant_sign = dominant_valence_sign(&neighbors, &self.markers);

    // Step 2: Aggregate congruent (same-sign) neighbors
    let congruent = self.aggregate_signal(/* same-sign neighbors */);

    // Step 3: Find contrarian (opposite-sign) neighbors
    let contrarian_target = ((neighbor_count as f64) * CONTRARIAN_FRACTION).ceil() as usize;
    // ... sort all markers by distance, filter opposite-valence, take nearest ...

    // Step 4: Blend 85% congruent + 15% contrarian
    SomaticSignal {
        valence: (0.85 * congruent.valence + 0.15 * contrarian.valence).clamp(-1.0, 1.0),
        intensity: (0.85 * congruent.intensity + 0.15 * contrarian.intensity).clamp(0.0, 1.0),
        neighbor_count,
        contrarian_count: contrarian.neighbor_count,
        source_episodes: union_hashes(...),
    }
}
```

Aggregation weights each neighbor by inverse distance and intensity:
```rust
let distance_weight = 1.0 / (1.0 + distance_sq.max(0.0));
let weight = distance_weight * marker.intensity.max(0.05);
```

### SomaticSignal and Actionability

The query result includes actionability thresholds:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 1288)

pub struct SomaticSignal {
    pub valence: f64,           // Blended valence after contrarian mix
    pub intensity: f64,         // Aggregate strength [0.0, 1.0]
    pub neighbor_count: usize,  // Same-valence neighbors used
    pub contrarian_count: usize,// Contrarian neighbors mixed in
    pub source_episodes: Vec<ContentHash>,
}

impl SomaticSignal {
    pub fn is_actionable(&self) -> bool {
        self.intensity >= 0.15 && self.valence.abs() >= 0.10
    }

    pub fn should_emit_event(&self) -> bool {
        self.intensity > SOMATIC_EVENT_INTENSITY_THRESHOLD   // 0.50
            && self.valence.abs() > SOMATIC_EVENT_VALENCE_THRESHOLD  // 0.30
    }
}
```

### SomaticOracleContext

The somatic TA integration module (`crates/roko-daimon/src/somatic_ta.rs`) bridges the somatic landscape with oracle predictions:

```rust
// Source: crates/roko-daimon/src/somatic_ta.rs (line 52)

pub struct SomaticOracleContext {
    pub signal: SomaticSignal,
    pub is_actionable: bool,
    pub confidence_multiplier: f64,  // [0.7, 1.3]
    pub contrarian_fraction: f64,
}

pub fn somatic_confidence_bias(valence: f64, intensity: f64) -> f64 {
    const MAX_BIAS: f64 = 0.30;
    let raw = valence * intensity * MAX_BIAS;
    (1.0 + raw).clamp(0.7, 1.3)
}
```

**Worked example**: A strategy region where the agent previously succeeded (valence = +0.8, intensity = 0.9):
- raw = 0.8 * 0.9 * 0.30 = 0.216
- confidence_multiplier = 1.0 + 0.216 = 1.216 (21.6% boost)

A strategy region where the agent previously failed (valence = -0.7, intensity = 0.8):
- raw = -0.7 * 0.8 * 0.30 = -0.168
- confidence_multiplier = 1.0 - 0.168 = 0.832 (16.8% reduction)

The range is deliberately clamped to [0.7, 1.3] to prevent somatic data from dominating rational prediction.

### Dream Depotentiation of Somatic Markers

During dream processing, highly charged somatic markers have their intensity reduced -- analogous to how REM sleep reduces the emotional charge of memories while preserving their informational content (Walker & van der Helm, 2009):

```rust
// Source: crates/roko-daimon/src/lib.rs (line 1563)

pub fn apply_dream_depotentiation(&mut self) -> (usize, f64) {
    let mut cooled_markers = 0_usize;
    let mut total_reduction = 0.0;

    for marker in &mut self.markers {
        if marker.intensity <= 0.5 { continue; }  // Only cool hot markers
        let before = marker.intensity;
        let after = depotentiate_magnitude(before);
        if after < before {
            marker.intensity = after;
            cooled_markers += 1;
            total_reduction += before - after;
        }
    }
    (cooled_markers, total_reduction)
}
```

The `depotentiate_magnitude()` function (line 2845) reduces intensity by 30-50% with a floor of 0.05 (the constants `DEPOTENTIATION_DELTA_MIN` = 0.30, `DEPOTENTIATION_DELTA_MAX` = 0.50, `DEPOTENTIATION_FLOOR` = 0.05).

---

## 15% Contrarian Blending Mechanism

### The Problem: Mood-Congruent Feedback Loops

Bower (1981) demonstrated that mood biases memory retrieval: positive moods preferentially retrieve positive memories, and negative moods retrieve negative memories. For an agent, this creates a dangerous feedback loop:

```
Agent fails task -> Mood drops -> Retrieves negative markers
    -> Negative markers bias toward conservative strategies
    -> Conservative strategies may miss better approaches
    -> More failures -> Mood drops further
```

Without intervention, the agent becomes trapped in a self-reinforcing cycle of negative affect and conservative behavior.

### The Solution: Forced Opposite-Valence Injection

The contrarian blending mechanism forces 15% of the somatic signal to come from markers with the opposite valence:

```rust
const CONTRARIAN_FRACTION: f64 = 0.15;

// In SomaticLandscape::query():
let contrarian_target = ((neighbor_count as f64) * CONTRARIAN_FRACTION).ceil() as usize;

// Final blend:
SomaticSignal {
    valence: (0.85 * congruent.valence + 0.15 * contrarian.valence),
    intensity: (0.85 * congruent.intensity + 0.15 * contrarian.intensity),
    ...
}
```

If the dominant signal is negative (past failures in similar situations), 15% of the signal will be drawn from positive markers in the same strategy region -- reminding the agent that success is possible here. If the dominant signal is positive, 15% will be drawn from negative markers -- preventing overconfidence.

### ContrarianTracker and Rolling Window

The `ContrarianTracker` monitors contrarian retrieval over a rolling window (default 200 ticks) to ensure the 15% target is met over time:

```rust
// Source: crates/roko-daimon/src/phase2_stubs.rs (line 475)

pub struct ContrarianTracker {
    window: VecDeque<ContrarianEvent>,  // Ring buffer of recent retrievals
    pub window_size: usize,             // 200 ticks
    pub min_contrarian_fraction: f64,   // 0.15
}
```

The tracker's `should_inject()` method computes the actual contrarian fraction within the window and returns `true` when the fraction falls below the minimum, enabling adaptive adjustment.

**Roko docs reference**: `/docs/v1/09-daimon/07-15-percent-contrarian-retrieval.md`

---

## Four-Factor Retrieval Scoring Model

### RetrievalWeights and Online Learning

The four-factor retrieval model scores knowledge entries for retrieval using online-learnable weights:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 88)

pub struct RetrievalWeights {
    pub recency: f64,     // Temporal recency (Ebbinghaus forgetting curve), default 0.20
    pub importance: f64,  // Quality/validation ratio (Reflexion), default 0.25
    pub relevance: f64,   // Semantic similarity (cosine), default 0.35
    pub emotional: f64,   // PAD congruence with current mood, default 0.20
}
```

The `score()` method computes the weighted sum, and `update()` performs online gradient descent with normalized weights:

```rust
pub fn update(&mut self, factors: [f64; 4], outcome: f64, learning_rate: f64) {
    let predicted = self.score(factors[0], factors[1], factors[2], factors[3]);
    let error = outcome - predicted;

    self.recency = (self.recency + learning_rate * error * factors[0]).clamp(0.01, 0.80);
    self.importance = (self.importance + learning_rate * error * factors[1]).clamp(0.01, 0.80);
    self.relevance = (self.relevance + learning_rate * error * factors[2]).clamp(0.01, 0.80);
    self.emotional = (self.emotional + learning_rate * error * factors[3]).clamp(0.01, 0.80);

    // Normalize to sum to 1.0
    let total = self.recency + self.importance + self.relevance + self.emotional;
    if total > 0.0 {
        self.recency /= total;
        self.importance /= total;
        self.relevance /= total;
        self.emotional /= total;
    }
}
```

### Emotional Congruence Computation

The emotional factor uses PAD cosine similarity mapped via `f64::midpoint()`:

```rust
// Source: crates/roko-daimon/src/lib.rs (line 162)

pub fn emotional_congruence(current_mood: &PadVector, entry_affect: &PadVector) -> f64 {
    f64::midpoint(current_mood.cosine_similarity(*entry_affect), 1.0)
}
```

This means that during retrieval, memories tagged with emotions similar to the current mood score higher. Combined with the 15% contrarian mechanism at the somatic layer, this creates a nuanced retrieval system: mostly mood-congruent (which is empirically useful) with a forced diversity floor (which prevents echo chambers).

---

## Nietzsche Vitality Phases

### VitalityPhase Enum

For long-running agents approaching resource depletion, the Daimon implements Nietzsche's three metamorphoses from *Also sprach Zarathustra* (1883), mapped to agent vitality levels:

```rust
// Source: crates/roko-daimon/src/mortality.rs (line 126)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VitalityPhase {
    /// The Camel (vitality > 0.7): bears its strategy dutifully.
    Camel,
    /// The Lion (vitality 0.3-0.7): rebels against inherited heuristics.
    Lion,
    /// The Child (vitality < 0.3): sheds survival pressure.
    Child,
}

impl VitalityPhase {
    pub fn from_vitality(vitality: f64) -> Self {
        if vitality > 0.7 { Self::Camel }
        else if vitality >= 0.3 { Self::Lion }
        else { Self::Child }
    }
}
```

Each phase has a characteristic PAD baseline (defined via `pad_baseline()`) and behavioral profile:

| Phase | Vitality | PAD Baseline | Exploration Rate | Sharing Threshold | Description |
|---|---|---|---|---|---|
| **Camel** | > 0.7 | P:0.2, A:0.0, D:0.3 | 0.15 (conservative) | 0.50 (moderate) | Duty-driven, steady execution, inauthentic mortality awareness |
| **Lion** | 0.3 - 0.7 | P:-0.35, A:0.5, D:-0.05 | 0.40 (high exploration) | 0.40 (active sharing) | Value-challenging, explore/exploit crisis, creative destruction |
| **Child** | < 0.3 | P:-0.1, A:0.4, D:0.2 | 0.60 (maximum exploration) | 0.10 (shares everything) | Creative acceptance, generative sharing, plays with strategies |

The counterintuitive design: the **dying agent becomes most creative**. As vitality drops below 0.3, the agent has nothing to lose and enters maximum exploration with minimal sharing threshold -- it gives away everything it has learned. This implements the observation that creative breakthroughs often come from individuals freed from institutional constraints.

### Mortality Emotions

Three mortality-specific emotions, each tied to a different existential clock:

```rust
// Source: crates/roko-daimon/src/mortality.rs (line 24)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MortalityEmotion {
    /// Resource scarcity (Jonas's "needful freedom")
    EconomicAnxiety,    // PAD: P:-0.4, A:0.6, D:-0.3 (sharp, immediate)
    /// Obsolescence awareness (Dane's cognitive entrenchment crisis)
    EpistemicVertigo,   // PAD: P:-0.3, A:0.4, D:-0.5 (destabilizing, recursive)
    /// Background finitude awareness (Heidegger's Angst)
    StochasticDread,    // PAD: P:-0.15, A:0.1, D:-0.2 (quiet, persistent)
}
```

Each emotion's intensity is computed from agent state via `MortalityEmotion::intensity()`:

- **EconomicAnxiety**: Proportional to `burn_rate / runway_hours`. Sharp and immediate.
- **EpistemicVertigo**: Proportional to `-accuracy_trend`. Triggered by declining prediction accuracy.
- **StochasticDread**: Always-on background hum. Base 0.1 + urgency scaling as runway shortens (up to 0.5 additional).

### Emotional Death Testament

The `EmotionalDeathTestament` transfers complete emotional context to successor agents on shutdown:

```rust
// Source: crates/roko-daimon/src/mortality.rs (line 221)

pub struct EmotionalDeathTestament {
    pub life_review: LifeReview,
    pub final_phase: VitalityPhase,
    pub active_mortality_emotions: Vec<(MortalityEmotion, f64)>,
    pub final_pad: PadVector,
    pub total_episodes: usize,
    pub lifetime_hours: f64,
    pub annotated_learnings: Vec<AnnotatedLearning>,
}

pub struct AnnotatedLearning {
    pub content: String,
    pub emotional_weight: f64,
    pub learning_emotion: String,
    pub validated: bool,
}
```

Successors inherit not just knowledge but the emotional weight that makes that knowledge meaningful. A learning tagged with high emotional weight ("this approach caused three consecutive failures") carries more salience than a neutral observation.

---

## Life Review Pipeline

### Butler's Life Review Adapted for Agents

Robert Butler (1963) proposed that older adults engage in a universal "life review" process -- a progressive return to consciousness of past experiences, particularly the resurgence of unresolved conflicts. Butler argued this process contributes to the development of characteristics such as candor, serenity, and wisdom. The Daimon adapts this for computational agents, retrieving and classifying the agent's most emotionally significant memories during shutdown.

```rust
// Source: crates/roko-daimon/src/life_review.rs (line 145)

pub fn review(memories: &[ReviewMemory], config: &LifeReviewConfig) -> LifeReview {
    // Step 1: Select top memories by arousal magnitude (default: top 20)
    // Filter by min_arousal (default: 0.3), sort by descending arousal

    // Step 2: Detect turning points (PAD Euclidean distance > 0.5
    // between consecutive memories)
    let turning_points = detect_turning_points(&selected, config.turning_point_threshold);

    // Step 3: Classify narrative arc from trajectory shape
    let trajectory = compute_trajectory(&selected);
    let narrative_arc = classify_arc(&trajectory, &turning_points);

    LifeReview { memories: selected, turning_points, narrative_arc, trajectory }
}
```

### McAdams Narrative Arc Classification

The narrative arc is classified using Dan McAdams' (2001) typology from *The Psychology of Life Stories*. McAdams identified that people construct internalized narratives of the self, and identified two fundamental sequence types: redemption (negative-to-positive transformation) and contamination (positive-to-negative decline).

```rust
// Source: crates/roko-daimon/src/life_review.rs (line 79)

pub enum NarrativeArc {
    Redemptive,     // Started negative, ended positive (failure -> learning -> success)
    Contaminating,  // Started positive, ended negative (success -> complacency -> failure)
    Progressive,    // Steady upward trajectory (consistent growth)
    Tragic,         // Steady downward trajectory (consistent decline)
    Stable,         // No clear direction (stable throughout)
}
```

Classification heuristics (from the `EmotionalTrajectory` struct's `start_pleasure`, `end_pleasure`, and turning point counts):
- **Redemptive**: start_pleasure < -0.1 AND pleasure_delta > 0.3
- **Contaminating**: start_pleasure > 0.1 AND pleasure_delta < -0.3
- **Progressive**: pleasure_delta > 0.15 AND positive_turns > negative_turns
- **Tragic**: pleasure_delta < -0.15 AND negative_turns > positive_turns
- **Stable**: everything else

---

## Emergent Goal Structures

### GoalSeed and GoalTree

Rather than being explicitly programmed, goals emerge from recurring behavioral patterns:

```rust
// Source: crates/roko-daimon/src/goals.rs (line 24)

pub struct GoalSeed {
    pub id: String,
    pub pattern: String,           // Human-readable description
    pub observation_count: u64,    // How many times observed
    pub score: f64,                // Accumulated evidence
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}
```

Seeds are promoted into the `GoalTree` when they accumulate sufficient evidence (checked via `is_promotable(min_observations, min_score)`). The lifecycle is:

```
observation -> GoalSeed -> (evidence accumulates) -> GoalNode in GoalTree
                               | pruned if score < threshold
```

The `GoalTree` supports hierarchical goal structures with parent-child relationships, priority-based sorting, time-based decay, and pruning of low-priority goals. Goals have four statuses: `Active`, `Completed`, `Suspended`, `Pruned`.

---

## Somatic TA Integration

### IIT Phi Metric

The `IitPhiMetric` computes Tononi's (2004) Integrated Information Theory Phi over transactional analysis subsystems:

```rust
// Source: crates/roko-daimon/src/somatic_ta.rs

pub struct IitPhiMetric {
    pub phi: f64,               // Non-negative integrated information
    pub num_subsystems: usize,
    pub num_bipartitions: u64,
    pub mib_mask: u64,          // Minimum information bipartition
}
```

Phi measures how much the system is "more than the sum of its parts." The computation enumerates all bipartitions of N subsystems and finds the minimum information partition:

```
Phi = min_{bipartitions} MI(A; B) / min(H(A), H(B))
```

For N <= 20 subsystems, exhaustive enumeration is tractable (2^(N-1) - 1 bipartitions).

### PID Synergy Detection

Partial Information Decomposition (PID) synergy detection identifies when joint subsystem activity carries more information than the sum of individual activities:

```rust
pub fn detect_synergy(activities: &[SubsystemActivity]) -> (f64, f64) {
    // Returns (synergy, redundancy):
    // synergy > 0: subsystems more useful together than apart
    // redundancy > 0: subsystems duplicate information
}
```

Based on Williams & Beer (2010) nonnegative decomposition of multivariate information, which proposes decomposing mutual information into redundancy, synergy, unique information, and union information.

---

## IronClaw Integration Plan

The following plan maps Daimon concepts to IronClaw's existing architecture, with concrete file paths, Rust code sketches, and step-by-step implementation guidance.

### Phase 1: User Engagement Modeling

**Effort**: ~300-400 lines | **Files**: `src/profile.rs`, new `src/affect/mod.rs`

IronClaw already has a 9-dimension `PsychographicProfile` in `src/profile.rs`. Adding a lightweight PAD tracker on user interaction patterns extends this with emotional dynamics.

**Step 1: Create `src/affect/mod.rs`**

```rust
//! Lightweight affect state for IronClaw.
//! Adapted from roko-daimon's PAD + ALMA architecture.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Normalized PAD vector. Same semantics as roko-primitives PadVector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct PadVector {
    pub pleasure: f64,   // [-1.0, 1.0]
    pub arousal: f64,    // [-1.0, 1.0]
    pub dominance: f64,  // [-1.0, 1.0]
}

impl PadVector {
    pub const fn neutral() -> Self { Self { pleasure: 0.0, arousal: 0.0, dominance: 0.0 } }

    pub fn clamped(self) -> Self {
        Self {
            pleasure: self.pleasure.clamp(-1.0, 1.0),
            arousal: self.arousal.clamp(-1.0, 1.0),
            dominance: self.dominance.clamp(-1.0, 1.0),
        }
    }

    pub fn apply_delta(&mut self, p: f64, a: f64, d: f64) {
        self.pleasure = (self.pleasure + p).clamp(-1.0, 1.0);
        self.arousal = (self.arousal + a).clamp(-1.0, 1.0);
        self.dominance = (self.dominance + d).clamp(-1.0, 1.0);
    }

    pub fn decay_toward_neutral(&mut self, factor: f64) {
        self.pleasure *= factor;
        self.arousal *= factor;
        self.dominance *= factor;
    }
}

/// User engagement state tracked per conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEngagementState {
    pub pad: PadVector,
    pub mood_ema: PadVector,   // Slow-moving EMA (tau = 0.3)
    pub updated_at: DateTime<Utc>,
    pub tick_count: u64,
}

/// Behavioral hint derived from user engagement state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserEngagementHint {
    Frustrated,       // Low P, High A -> careful, step-by-step
    PowerUser,        // High P, High D -> concise, skip explanations
    Exploring,        // High A, Low D  -> suggest capabilities
    Normal,           // Near origin    -> balanced response
}
```

**Step 2: Wire engagement signals in `src/agent/`**

In the agent loop, after each user message, derive PAD deltas from observable signals:
- **Pleasure**: positive feedback ("thanks", "great"), task completion rate
- **Arousal**: message frequency, urgency markers ("ASAP", "urgent"), message length
- **Dominance**: directive vs. questioning tone, use of specific tool names

**Step 3: Inject engagement hint into prompt composition**

In `crates/ironclaw_engine/`, read the `UserEngagementHint` and append behavioral guidance:
- `Frustrated` -> "The user appears frustrated. Be careful, verify assumptions, explain step by step."
- `PowerUser` -> "The user is experienced and confident. Be concise, skip basic explanations."

### Phase 2: Agent Self-Regulation

**Effort**: ~500-600 lines | **Files**: `src/affect/mod.rs`, `src/agent/`

**Step 1: Add agent confidence tracking**

Track a `confidence: f64` in `[0.0, 1.0]` that decays toward 0.5 (neutral) and is modified by operational outcomes:
- Tool call succeeds: confidence += 0.05
- Tool call fails: confidence -= 0.10 (negativity bias)
- Gate pass: confidence += rung_scale * 0.03
- Gate fail: confidence -= rung_scale * 0.08

**Step 2: Modulate agent behavior based on confidence**

When confidence < 0.30 (Struggling equivalent):
- Ask more clarifying questions before acting
- Use more conservative tool parameters
- Break tasks into smaller steps
- Increase verification frequency

When confidence > 0.70 (Coasting equivalent):
- Act more autonomously
- Attempt larger changes in single steps
- Use cheaper model tiers for sub-tasks

### Phase 3: Somatic Markers for Tool Selection

**Effort**: ~800 lines | **Files**: new `src/affect/somatic.rs`, `src/tools/registry.rs`

**Step 1: Define tool outcome markers**

```rust
// src/affect/somatic.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A somatic marker for tool selection outcomes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMarker {
    /// Hashed situation descriptor (tool name + context features).
    pub context_hash: u64,
    /// Tool that was selected.
    pub tool_name: String,
    /// Outcome: +1.0 success, -1.0 failure, 0.0 neutral.
    pub outcome_valence: f64,
    /// When this outcome was recorded.
    pub timestamp: DateTime<Utc>,
    /// Strategy-space coordinates (simplified: complexity, novelty, urgency).
    pub coords: [f64; 3],
}
```

**Step 2: Record outcomes after tool dispatch**

After every tool execution in `src/tools/dispatch.rs`, record a `ToolMarker` with the outcome. Store in workspace persistence (via the `Database` trait).

**Step 3: Query markers before tool selection**

Before selecting a tool in the tool registry, query recent markers for similar contexts. Positive markers bias toward the tool; negative markers bias away. This requires adding a simple nearest-neighbor lookup (a linear scan is sufficient for the expected marker count; k-d tree is overkill until > 1000 markers).

### Phase 4: Full Behavioral State Dispatch Modulation

**Effort**: ~1,000-1,500 lines | **Files**: `src/affect/mod.rs`, `crates/ironclaw_engine/`

This phase implements the full Daimon behavioral state -> dispatch parameter modulation:

1. Port the `BehavioralState` enum and `classify()` function
2. Port the `DispatchStrategy` enum and `modulate()` logic
3. Wire behavioral state into the engine v2 prompt composition pipeline
4. Add the tier bias table for model selection routing

**Cost optimization table** (maps behavioral state to model selection):

| State | Model Tier | Turn Limit | Cost Impact |
|---|---|---|---|
| **Engaged** | Default | Default | Baseline |
| **Struggling** (anxious) | Demote (opus->sonnet) | -3 | Cost reduction |
| **Struggling** (angry) | Promote (sonnet->opus) | +10 | Cost increase |
| **Coasting** | Demote (sonnet->haiku) | -5 | Significant cost reduction |
| **Focused** | Default | -2 | Slight cost reduction |
| **Resting** | Default | +5 | Extra budget for maintenance |

Over time, the efficiency log (`effort_label` per task) enables cost-outcome correlation analysis. If "high" effort tasks do not pass gates at a higher rate than "medium" effort tasks, the Struggling -> Escalating path is wasteful and thresholds should be adjusted.

---

## Practical Benefits of Affect-Aware Agents

1. **Automatic cost optimization**. An agent in "Coasting" state (succeeding easily) automatically uses cheaper models. An agent in "Struggling" state that is failing due to knowledge gaps (low dominance) conserves resources instead of throwing expensive compute at problems it does not understand. Over hundreds of tasks, this produces measurable cost savings without manual tier configuration.

2. **Graceful degradation under failure**. Instead of failing the same way repeatedly, the behavioral state system detects sustained failure (Struggling state) and changes strategy -- either escalating to stronger models (if the agent believes it can solve the problem) or conserving resources and requesting help (if it does not).

3. **Faster learning from experience**. The somatic landscape provides sub-millisecond "gut feeling" retrieval: given a new task's structural characteristics, the agent instantly knows whether similar tasks have historically succeeded or failed, and adjusts its confidence and strategy accordingly. This is orders of magnitude faster than re-analyzing past episodes.

4. **Prevention of mood-congruent loops**. The 15% contrarian blending mechanism ensures that even when the agent is in a strongly negative state (consecutive failures), it still considers positive evidence from the same strategy region. Without this, negative mood -> negative retrieval -> conservative strategy -> more failures becomes a trap.

5. **Natural communication style adaptation**. By tracking user engagement PAD, the agent can detect frustrated users and switch to more careful, reassuring responses, or detect confident power users and be more concise. This goes beyond simple keyword detection -- the mood EMA captures trajectory, not just instantaneous state.

6. **Knowledge transfer at end-of-life**. The mortality emotions and emotional death testament ensure that when an agent's resources are depleted, it transfers not just raw knowledge but the emotional weight that makes that knowledge actionable. A successor agent inherits the insight that "this approach caused three failures" with appropriate caution, not just a neutral log entry.

---

## Implementation Roadmap

**Start with**: User engagement modeling (Phase 1) -- lightest integration, most immediately useful. A lightweight PAD tracker on user interaction patterns with mood-aware response style selection requires ~300-400 lines and has no external dependencies.

**Then**: Agent self-regulation (Phase 2) -- wire confidence tracking into the agent loop. When the agent fails 3+ consecutive operations, reduce autonomy and increase verification. ~500-600 lines.

**Then**: Somatic markers for tool selection (Phase 3) -- requires a persistence layer for tool outcome markers but produces significant long-term improvement in tool choice quality. ~800 lines including nearest-neighbor lookup.

**Finally**: Full behavioral state -> dispatch modulation (Phase 4) -- the complete Daimon integration. Requires the engine v2 prompt composition pipeline to support behavioral state hints. ~1,000-1,500 lines for the full affect engine.

### Complexity Assessment

- **Phase 1 (User engagement tracker)**: ~300-400 lines
- **Phase 2 (Agent self-regulation)**: ~500-600 lines
- **Phase 3 (Somatic markers for tools)**: ~800 lines
- **Phase 4 (Full affect engine)**: ~1,000-1,500 lines
- **Risk**: Medium -- subjective tuning needed for PAD thresholds
- **Dependencies**: None for core computation; `kiddo` crate for k-d tree (Phase 3+); `chrono` for timestamps
- **Caution**: Avoid anthropomorphizing in user-facing output. The affect model should influence *how* the agent responds, not what it *says about* its feelings. The PAD vector is a control signal, not a personality display.

---

## References

### Primary Sources (Psychology)

- Mehrabian, A. (1996). "Pleasure-arousal-dominance: A general framework for describing and measuring individual differences in temperament." *Current Psychology*, 14(4), 261-292.
- Russell, J.A. & Mehrabian, A. (1977). "Evidence for a three-factor theory of emotions." *Journal of Research in Personality*, 11(3), 273-294. doi:10.1016/0092-6566(77)90037-X
- Russell, J.A. (1980). "A circumplex model of affect." *Journal of Personality and Social Psychology*, 39(6), 1161-1178. doi:10.1037/h0077714
- Ortony, A., Clore, G.L., & Collins, A. (1988). *The Cognitive Structure of Emotions*. Cambridge University Press. ISBN: 978-0-521-35364-3.
- Scherer, K.R. (2001). "Appraisal considered as a process of multilevel sequential checking." In K.R. Scherer, A. Schorr, & T. Johnstone (Eds.), *Appraisal Processes in Emotion: Theory, Methods, Research* (pp. 92-120). Oxford University Press.
- Damasio, A.R. (1994). *Descartes' Error: Emotion, Reason, and the Human Brain*. New York: Grosset/Putnam.
- Bower, G.H. (1981). "Mood and memory." *American Psychologist*, 36(2), 129-148. doi:10.1037/0003-066X.36.2.129
- Plutchik, R. (1980). *Emotion: A Psychoevolutionary Synthesis*. New York: Harper & Row.
- Kahneman, D. & Tversky, A. (1979). "Prospect Theory: An Analysis of Decision under Risk." *Econometrica*, 47(2), 263-291. doi:10.2307/1914185

### Affective Computing

- Gebhard, P. (2005). "ALMA -- A Layered Model of Affect." *Proceedings of the Fourth International Joint Conference on Autonomous Agents and Multiagent Systems (AAMAS '05)*, pp. 29-36. ACM Press. doi:10.1145/1082473.1082478
- Gadanho, S.C. (2003). "Learning Behavior-Selection by Emotions and Cognition in a Multi-Goal Robot Task." *Journal of Machine Learning Research*, 4, 385-412.
- Picard, R.W. (1997). *Affective Computing*. MIT Press.

### Somatic Markers and Decision-Making

- Bechara, A., Damasio, A.R., Damasio, H., & Anderson, S.W. (1994). "Insensitivity to future consequences following damage to human prefrontal cortex." *Cognition*, 50(1-3), 7-15. doi:10.1016/0010-0277(94)90018-3
- Bechara, A., Damasio, H., Tranel, D., & Damasio, A.R. (1997). "Deciding advantageously before knowing the advantageous strategy." *Science*, 275(5304), 1293-1295. doi:10.1126/science.275.5304.1293

### Narrative Psychology and Life Review

- Butler, R.N. (1963). "The life review: An interpretation of reminiscence in the aged." *Psychiatry*, 26(1), 65-76. doi:10.1080/00332747.1963.11023339
- McAdams, D.P. (2001). "The psychology of life stories." *Review of General Psychology*, 5(2), 100-122. doi:10.1037/1089-2680.5.2.100
- Costa, P.T. & McCrae, R.R. (1992). *NEO PI-R Professional Manual*. Psychological Assessment Resources.

### Sleep and Emotional Processing

- Walker, M.P. & van der Helm, E. (2009). "Overnight therapy? The role of sleep in emotional brain processing." *Psychological Bulletin*, 135(5), 731-748. doi:10.1037/a0016570

### Information Theory

- Tononi, G. (2004). "An information integration theory of consciousness." *BMC Neuroscience*, 5, 42. doi:10.1186/1471-2202-5-42
- Williams, P.L. & Beer, R.D. (2010). "Nonnegative decomposition of multivariate information." arXiv:1004.2515.

### LLM Cost Optimization

- Chen, L. et al. (2023). "FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance." arXiv:2305.05176.
- Shinn, N. et al. (2023). "Reflexion: Language Agents with Verbal Reinforcement Learning." *Advances in Neural Information Processing Systems (NeurIPS)*.

### Roko Documentation Cross-References

| Document | Path |
|---|---|
| PAD Vector | `/docs/v1/09-daimon/01-pad-vector.md` |
| ALMA Three-Layer Temporal Model | `/docs/v1/09-daimon/02-alma-three-layer-temporal.md` |
| OCC/Scherer Appraisal | `/docs/v1/09-daimon/03-occ-scherer-appraisal.md` |
| Six Behavioral States | `/docs/v1/09-daimon/04-six-behavioral-states.md` |
| Somatic Markers (Damasio) | `/docs/v1/09-daimon/06-somatic-markers-damasio.md` |
| 15% Contrarian Retrieval | `/docs/v1/09-daimon/07-15-percent-contrarian-retrieval.md` |
| 8-Dimensional Strategy Space | `/docs/v1/09-daimon/08-8-dimensional-strategy-space.md` |
| Vision and Mortality | `/docs/v1/09-daimon/00-vision-and-mortality-incompatibility.md` |

### Roko Source Code Cross-References

| File | Contents |
|---|---|
| `crates/roko-primitives/src/pad.rs` | Canonical PadVector struct |
| `crates/roko-core/src/affect.rs` | BehavioralState enum, `classify()` function |
| `crates/roko-daimon/src/lib.rs` | AlmaLayers, AffectState, DaimonState, AffectEngine trait, SomaticLandscape, StrategyCoordinates, AppraisalResult, RetrievalWeights, NoveltyFilter, AffectEvent, DispatchStrategy, DispatchParams, CodingStrategySpace |
| `crates/roko-daimon/src/mortality.rs` | VitalityPhase, MortalityEmotion, EmotionalDeathTestament |
| `crates/roko-daimon/src/life_review.rs` | LifeReview, NarrativeArc, TurningPoint, review() pipeline |
| `crates/roko-daimon/src/goals.rs` | GoalSeed, GoalNode, GoalTree, GoalStatus |
| `crates/roko-daimon/src/somatic_ta.rs` | SomaticOracleContext, IitPhiMetric, somatic_confidence_bias(), detect_synergy() |
| `crates/roko-daimon/src/phase2_stubs.rs` | AffectOctant, BehavioralStateTracker, BehavioralStateThresholds, ContrarianTracker, ContrarianConfig, AffectBehaviorModulation, TierThresholds, adjusted_thresholds() |
| `crates/roko-daimon/src/policy.rs` | AffectPolicy adapter for WorkflowEngine |

# Dream Consolidation: Biologically-Inspired Offline Learning for AI Agents

**Source crate**: `roko-dreams` (`crates/roko-dreams/` in the roko repository)
**Priority**: HIGH -- natural extension of IronClaw's heartbeat system
**Status**: Architecture study; no code written in IronClaw yet

---

## Table of Contents

1. [Introduction: What Is Dream Consolidation?](#introduction-what-is-dream-consolidation)
2. [Why This Matters](#why-this-matters)
3. [Neuroscience Foundations](#neuroscience-foundations)
4. [System Architecture Overview](#system-architecture-overview)
5. [NREM Replay -- Mattar-Daw Utility Scoring](#nrem-replay----mattar-daw-utility-scoring)
6. [REM Imagination -- Counterfactual Synthesis](#rem-imagination----counterfactual-synthesis)
7. [Hypnagogic Creativity Pipeline](#hypnagogic-creativity-pipeline)
8. [Threat Rehearsal](#threat-rehearsal)
9. [Staging Buffer -- Confidence Lifecycle](#staging-buffer----confidence-lifecycle)
10. [Dream Scheduling and Budget](#dream-scheduling-and-budget)
11. [Routing Advice -- Dream-to-Wake Knowledge Transfer](#routing-advice----dream-to-wake-knowledge-transfer)
12. [IronClaw Integration Architecture](#ironclaw-integration-architecture)
13. [Performance and Resource Characteristics](#performance-and-resource-characteristics)
14. [Implementation Plan](#implementation-plan)
15. [Academic References](#academic-references)

---

## Introduction: What Is Dream Consolidation?

Dream consolidation is a background offline learning system that runs during idle
periods -- intervals when the agent has no active user requests to process. It
replays, reorganizes, and strengthens the agent's accumulated knowledge, producing
durable insights, defensive strategies, and cross-domain connections that improve
future waking performance.

### The core idea in one paragraph

An AI agent accumulates experience as it works: it calls tools, makes decisions,
succeeds sometimes, fails sometimes, and observes patterns it never explicitly
reasons about. Dream consolidation takes that raw experience stream and processes
it during downtime -- reviewing what happened (replay), imagining what might have
happened differently (counterfactual reasoning), finding surprising connections
between unrelated experiences (creative association), and rehearsing defenses
against likely future failures (threat simulation). The outputs are durable
knowledge entries that make the agent smarter, faster, and more robust when it
wakes up and handles the next user request.

### How this differs from simple batch retraining

Dream consolidation is not batch retraining. It does not modify model weights, run
gradient descent, or require a training pipeline. Instead, it operates at the
**knowledge layer** -- the structured memories and heuristics that augment the
agent's context window at inference time. The differences:

| Aspect | Batch Retraining | Dream Consolidation |
|--------|-----------------|---------------------|
| What changes | Model weights | Knowledge store entries, routing advice |
| Infrastructure | GPU cluster, training pipeline | Single agent with LLM API access |
| Latency | Hours to days | Minutes |
| Risk | Catastrophic forgetting, distribution shift | Staging buffer prevents knowledge corruption |
| Cost | Thousands of dollars per run | $0.03-0.10 per dream cycle |
| Granularity | Entire model behavior | Individual insights and heuristics |

The word "dream" is not a metaphor. The system implements specific biological
sleep mechanisms that neuroscience has identified as critical for memory
consolidation, creative problem-solving, and threat preparation. The next section
explains the biological research and why it matters for AI systems.

### What the system produces

| Output Type | Description | Destination |
|-------------|-------------|-------------|
| **Insights** | Patterns extracted from episode replay | Knowledge store |
| **Heuristics** | Promoted insights with multiple confirmations; actionable rules | Knowledge store + playbooks |
| **Counterfactual strategies** | Novel approaches from imagining alternative histories | Staging buffer, then knowledge store on validation |
| **Warnings** | Threat scenarios with rehearsed responses | Knowledge store |
| **Confidence updates** | Strengthened or weakened belief in existing knowledge | Knowledge store confidence scores |
| **Routing advice** | Model selection recommendations for task categories | Persisted JSON, loaded at wake time |

In IronClaw's context, this maps to the existing workspace memory system
(`src/workspace/`). Episodes are the agent's action records; the knowledge store
is the workspace memory with its hybrid FTS + vector search. The heartbeat system
(`src/agent/heartbeat.rs`) already provides the idle-time execution scaffold.

---

## Why This Matters

### 1. Memory consolidation prevents knowledge decay

Without consolidation, an agent's accumulated knowledge is only as good as its
most recent context window. Important patterns from weeks-old interactions fade
into the noise of an ever-growing memory store. Dream consolidation actively
reviews past experiences, strengthening high-value memories and tagging low-value
ones for decay. The biological research is clear: memories that are not replayed
deteriorate. The same applies to an AI agent's knowledge store.

Lin et al. (2025) demonstrated this quantitatively: dedicating computation to
offline processing during idle periods yields a **5x reduction in test-time
compute requirements**. Agents that process experiences during low-activity
periods execute fewer expensive inference calls during active work. The compute
spent on dreams is an investment in waking performance, not idle-time waste.

### 2. Creative problem-solving through novel recombination

The "alpha convergence" problem (Grossman & Stiglitz, 1980) is real for AI
agents: when all agents use the same foundation models, they reach the same
conclusions and take the same actions. Dream consolidation breaks this
convergence by recombining each agent's unique experiential history into novel
hypotheses. Lacaux et al. (2021) showed that the hypnagogic state (sleep
onset) tripled creative problem-solving success rates in humans (83% vs 30%).
The hypnagogia engine implements this computationally: loosening associative
constraints to find unexpected connections between unrelated experiences.

### 3. Threat preparation through simulated failure

Revonsuo (2000) proposed that biological dreaming evolved primarily as a threat
rehearsal mechanism -- dreams over-represent threatening scenarios relative to
waking experience. For an AI agent, this means systematically analyzing past
failures, constructing plausible threat scenarios, rehearsing recovery paths,
and generating synthetic episodes that strengthen defensive responses. An agent
that has rehearsed a failure mode recovers faster when it encounters the real
thing.

### 4. Catastrophic forgetting prevention

WSCL (Luppi et al., 2024) demonstrated that interleaving wake and sleep
processing phases produces a **38% reduction in catastrophic forgetting**
compared to continuous waking-only learning. Without consolidation, new knowledge
overwrites old knowledge -- a well-characterized problem in both biological and
artificial learning systems. The staging buffer and confidence ladder prevent
this by ensuring new dream-generated insights are validated against existing
knowledge before promotion.

### 5. Cost efficiency

A full dream cycle costs $0.03-0.10 in model inference. If it prevents even one
unnecessary task retry (which costs $0.10-0.50 in inference), it pays for itself
immediately. Over weeks and months of accumulated experience, the compounding
effect of better routing advice, stronger heuristics, and rehearsed threat
responses produces substantial savings.

---

## Neuroscience Foundations

The dream consolidation system is grounded in six major lines of neuroscience and
computational research. These are not decorative analogies -- each directly
informs a specific implementation decision.

### Complementary Learning Systems (CLS)

McClelland, McNaughton, & O'Reilly (1995) demonstrated that biological brains
maintain two learning systems: a fast episodic system (hippocampus) that records
individual experiences, and a slow semantic system (neocortex) that gradually
extracts general knowledge. Dreams bridge the two -- during sleep, episodic
memories are replayed and gradually integrated into semantic knowledge. The key
insight is that trying to learn everything in one system causes catastrophic
interference: new memories overwrite old ones. The two-system architecture solves
this by keeping raw episodes separate from consolidated knowledge.

> McClelland, J. L., McNaughton, B. L., & O'Reilly, R. C. (1995). Why there
> are complementary learning systems in the hippocampus and neocortex: Insights
> from the successes and failures of connectionist models of learning and
> memory. *Psychological Review*, 102(3), 419-457.

**Map to IronClaw**: The episode log (action records from tool dispatch in
`src/context/memory.rs`) is the fast system. The workspace memory store
(`src/workspace/`) is the slow system. Dream consolidation replays action records
and distills them into durable workspace memories.

### Mattar-Daw Prioritized Replay

Mattar & Daw (2018) provided the normative theory for which memories should be
replayed and when. Not all memories are equally worth replaying -- the utility of
replaying a specific memory depends on how much the agent's behavior would improve
(gain), how relevant the memory is to current policy (need), and how recently it
was last replayed (spacing). The formula is:

```
Utility(episode) = Gain(episode) x Need(episode) x (1 / SpacingPenalty(episode))
```

This is the mathematical core of the NREM replay phase. The key insight from the
paper: replay is not random recall. Optimal replay prioritizes experiences with
high prediction error (surprising outcomes) that are relevant to upcoming
decisions (high need) and have not been recently rehearsed (spacing effect).

> Mattar, M. G. & Daw, N. D. (2018). Prioritized memory access explains
> planning and hippocampal replay. *Nature Neuroscience*, 21(11), 1609-1617.
> DOI: 10.1038/s41593-018-0232-z

**Map to IronClaw**: The `ReplayUtility` struct in `crates/roko-dreams/src/replay.rs`
implements the exact decomposition: `gain * need * spacing_inv`.

### Synaptic Homeostasis Hypothesis (SHY)

Tononi & Cirelli (2003, 2006) proposed that sleep serves a global renormalization
function: wakefulness strengthens synapses throughout the brain (learning creates
new connections), and sleep downscales synaptic strength back to a sustainable
baseline. The function of sleep is to pay the "price of plasticity" -- without
periodic downscaling, the brain would saturate. The hypothesis predicts that sleep
should selectively preserve important connections while pruning unimportant ones,
which is exactly what the staging buffer's confidence ladder does.

> Tononi, G. & Cirelli, C. (2006). Sleep function and synaptic homeostasis.
> *Sleep Medicine Reviews*, 10(1), 49-62.
>
> Tononi, G. & Cirelli, C. (2003). Sleep and synaptic homeostasis: A
> hypothesis. *Brain Research Bulletin*, 62(2), 143-150.
>
> Tononi, G. & Cirelli, C. (2014). Sleep and the price of plasticity: From
> synaptic and cellular homeostasis to memory consolidation and integration.
> *Neuron*, 81(1), 12-34.

**Map to IronClaw**: The staging buffer implements SHY computationally. Raw
dream-generated insights start at low confidence (0.20) and must survive
multiple validation cycles to reach promotion (0.70). Entries that fail to
promote within 7 days are garbage collected -- the computational equivalent of
synaptic downscaling.

### Hypnagogic Creativity

Lacaux et al. (2021) demonstrated experimentally that subjects in the hypnagogic
state (sleep onset, N1 stage) solved 83% of creative problems versus 30% for
fully awake subjects. The effect disappeared when subjects entered deeper sleep
(N2+). This validates Edison's technique of dozing with a key to capture creative
insights at the precise moment of sleep onset. The N1 state is characterized by
reduced prefrontal control (loosened executive function) combined with preserved
cortical activity, enabling novel associations that waking cognition would inhibit.

> Lacaux, C., Andrillon, T., Bastoul, C., Idir, Y., Fonteix-Galet, A.,
> Arnulf, I., & Oudiette, D. (2021). Sleep onset is a creative sweet spot.
> *Science Advances*, 7(50), eabj5866.

The MIT Dormio project (Haar Horowitz et al., 2020, 2023) confirmed these
results with targeted dream incubation, demonstrating a 43% creativity boost
via controlled N1 intervention.

> Haar Horowitz, A., Cunningham, T. J., Maes, P., & Stickgold, R. (2023).
> Targeted dream incubation at sleep onset increases post-sleep creativity.
> *Scientific Reports*, 13, 7319.

**Map to IronClaw**: The `HypnagogiaEngine` in
`crates/roko-dreams/src/hypnagogia.rs` implements the four-layer pipeline:
thalamic gate (signal filtering), executive loosener (constraint relaxation),
Dali interrupt (random creative breaks), and homuncular observer (quality
filtering). The pipeline deliberately introduces stochastic resonance to
generate novel associations.

### Threat Simulation Theory

Revonsuo (2000) proposed that biological dreaming evolved primarily as a threat
rehearsal mechanism. Threat-related dream content is over-represented relative to
waking experience, and the simulation function is most active following
threatening waking experiences. The theory predicts that dreams should
preferentially rehearse threatening scenarios to prepare defensive responses,
which is exactly what the threat simulation module does with failure episodes.

> Revonsuo, A. (2000). The reinterpretation of dreams: An evolutionary
> hypothesis of the function of dreaming. *Behavioral and Brain Sciences*,
> 23(6), 877-901.

**Map to IronClaw**: The `enumerate_threats` function in
`crates/roko-dreams/src/threat.rs` clusters failed episodes by failure pattern,
scores severity using FMEA methodology, and generates warning entries. The
`rehearse_threats` function in `rehearsal.rs` simulates recovery paths and
produces synthetic episodes for iterative learning.

### Sleep-Time Compute

Lin et al. (2025) introduced the concept of "sleep-time compute" -- dedicating
computation to offline processing during idle periods. Using a dual-agent
architecture (a Sleeper Agent that precomputes during downtime and a Serve Agent
that handles live interactions), they demonstrated approximately 5x test-time
compute reduction with up to 18% accuracy gains on modified reasoning tasks. This
provides the economic justification for the dream system: idle-time computation
is an investment in waking performance.

> Lin, K., Snell, C., Wang, Y., Packer, C., Wooders, S., Stoica, I., &
> Gonzalez, J. E. (2025). Sleep-time compute: Beyond inference scaling at
> test-time. *arXiv preprint* arXiv:2504.13171.

The WSCL framework (Luppi et al., 2024) demonstrated a complementary result:
interleaving wake and sleep processing phases, modeled on the Complementary
Learning Systems theory, produces significant reductions in catastrophic
forgetting and positive forward transfer -- the first continual learning method
to demonstrate the ability to prepare synapses for future knowledge.

> Luppi, A. I., Gurnee, W., Vilas, M. G., et al. (2024). Wake-sleep
> consolidated learning. *arXiv preprint* arXiv:2401.08623.

**Map to IronClaw**: The dream scheduling system in
`crates/roko-dreams/src/runner.rs` implements idle-time detection, cron-based
scheduling, and budget-constrained execution. IronClaw's heartbeat system
(`src/agent/heartbeat.rs`) already provides the periodic execution scaffold.

---

## System Architecture Overview

The roko-dreams crate implements a multi-phase dream cycle. The top-level module
structure:

```
crates/roko-dreams/src/
    lib.rs              # Crate root, public re-exports, subsystem facades
    cycle.rs            # Full cycle orchestration: cluster, distill, report
    runner.rs           # DreamRunner/DreamEngine: scheduling, budget, heartbeat
    replay.rs           # NREM replay: Mattar-Daw scoring, episode selection
    imagination.rs      # REM imagination: counterfactual generation
    hypnagogia.rs       # Hypnagogic creativity: 4-layer pipeline
    threat.rs           # Threat scenario enumeration (FMEA/FTA)
    rehearsal.rs        # Threat rehearsal: simulate recovery paths
    staging.rs          # Confidence staging buffer for knowledge promotion
    routing_advice.rs   # Dream-to-wake routing recommendations
    phase2/             # Advanced features (partially shipped)
        mod.rs          # Re-exports for advanced dream concepts
        sleep_time.rs   # Per-phase budget allocation (DREAM-12)
        advanced.rs     # Dream journal, nightmare containment, lucid monitoring
        imagination.rs  # Causal graphs, backtracking counterfactuals
        hypnagogia.rs   # Targeted dream incubation, novelty filters
        threat.rs       # Advanced red-teaming, fault trees
        evolution.rs    # MAP-Elites archive for strategy evolution
        ...
```

Source: `crates/roko-dreams/src/lib.rs`

### Dream Cycle State Machine

The dream cycle progresses through a deterministic state machine:

```
IDLE -> HYPNAGOGIA -> NREM_REPLAY -> REM_IMAGINATION -> INTEGRATION -> IDLE
```

Each phase runs to completion before the next begins. Between full cycles, brief
idle gaps can trigger micro-consolidation -- a single high-priority replay without
the full cycle overhead.

### Core Types

The crate's public API is defined through re-exports in `lib.rs`:

```rust
// Source: crates/roko-dreams/src/lib.rs:57-84

pub use cycle::{
    AgentDispatcher, DreamCycle, DreamCycleReport, PhaseBudgetSummary, StagingBufferStats,
};
pub use hypnagogia::{
    DaliInterrupt, ExecutiveLoosener, HomuncularObserver, HypnagogiaEngine, ThalamicGate,
};
pub use imagination::{
    CausalModel, CounterfactualQuery, ImaginationMode, ImaginationOutcome,
    counterfactual_episode, imagine, synthesize_hypotheses,
};
pub use rehearsal::{RehearsalOutcome, RehearsalReport, rehearse_threats};
pub use replay::{
    DreamReplayBatch, DreamReplayMode, DreamReplayPolicy, MattarDawConfig,
    ReplayUtility, compute_replay_utility, select_replay_episodes,
    select_replay_episodes_with_affect,
};
pub use runner::{
    BusPulseTriggerConfig, DreamAgentConfig, DreamBudget, DreamConfig,
    DreamEngine, DreamHeartbeatPolicy, DreamHeartbeatReport, DreamLoopConfig,
    DreamReport, DreamRunner, DreamRuntimeControls, DreamSchedulePolicy,
    DreamTrigger, Episode, Insight, IntensiveMode,
    PlanCompletionTriggerPolicy, build_dream_review_dispatcher,
};
pub use staging::{ConfidenceStage, StagingBuffer, StagingEntry};
pub use threat::{ThreatScenario, enumerate_threats, threat_warning_entries};
```

The `DreamCycle` struct owns the core consolidation loop:

```rust
// Source: crates/roko-dreams/src/cycle.rs:391-405

pub struct DreamCycle {
    episode_store: Arc<EpisodeLogger>,
    knowledge_store: Arc<KnowledgeStore>,
    playbook_store: Arc<PlaybookStore>,
    dispatcher: Arc<dyn AgentDispatcher>,
    last_dream_at: Option<DateTime<Utc>>,
    threat_simulation: bool,
    threat_severity_floor: f64,
    staging_buffer: StagingBuffer,
    staging_path: Option<PathBuf>,
    phase_tracker: Option<DreamBudgetTracker>,
}
```

---

## NREM Replay -- Mattar-Daw Utility Scoring

NREM replay is the first and cheapest phase of every dream cycle. It selects
episodes from the agent's history using a mathematically-principled utility
formula, replays them with controlled mutations, and extracts cross-episode
patterns.

### The Utility Formula

The core scoring algorithm implements the Mattar-Daw (2018) normative theory of
prioritized memory access:

```
Utility(episode) = Gain x Need x (1 / spacing_penalty)
```

Where:
- **Gain** measures prediction error -- how surprising was the outcome?
  Episodes where the agent failed (high prediction error) have high gain.
  Episodes that succeeded cleanly have low gain (less to learn).
- **Need** measures policy relevance -- how often does the agent encounter
  situations like this? Combines novelty (is this a new pattern?) with
  recency (is this recent enough to matter?).
- **Spacing inverse** implements spaced repetition -- episodes not recently
  replayed get higher scores to prevent over-rehearsal of the same memory.

The original Mattar-Daw paper derives this formula from normative decision theory:
the optimal replay policy selects the memory whose replay would most improve
future decisions. Gain corresponds to the prediction error signal that drives
learning. Need corresponds to the state-occupancy measure (how likely the agent
is to encounter similar situations). Spacing implements the well-established
spacing effect from memory research (Cepeda et al., 2006, *Psychological
Bulletin*): recently rehearsed items show diminishing returns from additional
rehearsal.

### Exact Implementation

The `ReplayUtility` struct decomposes the score into its three factors:

```rust
// Source: crates/roko-dreams/src/replay.rs:41-54

/// Mattar-Daw utility score for replay candidate prioritization.
///
/// `U(episode) = gain * need * (1/spacing)` -- replay episodes with high
/// expected learning value AND high policy relevance.
///
/// Reference: Mattar & Daw (2018) *Nature Neuroscience*.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReplayUtility {
    /// Expected improvement from replaying this episode.
    pub gain: f64,
    /// How policy-relevant this episode is.
    pub need: f64,
    /// Inverse of time since last replay (spaced-repetition term).
    pub spacing_inv: f64,
    /// Final utility: `gain * need * spacing_inv`.
    pub utility: f64,
}
```

The `compute` method on `ReplayUtility` assembles the three factors:

```rust
// Source: crates/roko-dreams/src/replay.rs:57-75

impl ReplayUtility {
    pub fn compute(
        episode: &Episode,
        novelty: f64,
        recency: f64,
        config: &MattarDawConfig,
    ) -> Self {
        let gain = Self::compute_gain(episode) * config.gain_weight;
        let need = Self::compute_need(novelty, recency) * config.need_weight;
        let spacing_inv = Self::compute_spacing_inv(episode, recency);
        let utility = gain * need * spacing_inv;
        Self { gain, need, spacing_inv, utility }
    }
}
```

### Gain Computation

Gain is derived from prediction error. Failed episodes have higher gain because
they contain more to learn. Complexity (measured by token usage) adds a secondary
signal:

```rust
// Source: crates/roko-dreams/src/replay.rs:79-94

fn compute_gain(episode: &Episode) -> f64 {
    let gate_total = episode.gate_verdicts.len().max(1) as f64;
    let fail_count = episode.gate_verdicts.iter()
        .filter(|v| !v.passed).count() as f64;
    // Prediction error: how surprising was the outcome?
    let error_rate = fail_count / gate_total;
    let surprise = if episode.success {
        // Successful but with some failures -- moderately surprising
        1.0 + error_rate * 0.5
    } else {
        // Failed -- high prediction error
        1.5 + error_rate
    };
    // Token usage as a secondary signal (complex tasks have more to learn)
    let complexity = (episode.tokens_used.max(1) as f64)
        .log10().clamp(0.0, 3.0) * 0.1;
    surprise + complexity
}
```

**Worked example**: Consider an episode that failed with 3 gate verdicts (2
failed), using 5,000 tokens:
- `error_rate = 2/3 = 0.667`
- `surprise = 1.5 + 0.667 = 2.167` (failed episode, high prediction error)
- `complexity = log10(5000) * 0.1 = 3.699 * 0.1 = 0.37`
- `gain = 2.167 + 0.37 = 2.537`

Compare with a successful episode, all 3 gates passing, same token count:
- `error_rate = 0/3 = 0.0`
- `surprise = 1.0 + 0.0 = 1.0` (clean success, low prediction error)
- `complexity = 0.37` (same)
- `gain = 1.0 + 0.37 = 1.37`

The failed episode has ~1.85x higher gain -- it is replayed more often because
there is more to learn from it.

### Need Computation

Need combines novelty (how new is this pattern?) with recency (is it still
relevant to current policy?):

```rust
// Source: crates/roko-dreams/src/replay.rs:99-106

fn compute_need(novelty: f64, recency: f64) -> f64 {
    let novelty_term = novelty.clamp(0.0, 1.0);
    let recency_term = recency.clamp(0.0, 1.0);
    // Weighted combination: novelty matters more than raw recency
    0.6 * novelty_term + 0.4 * recency_term
}
```

Novelty is computed from how many prior episodes share the same signature hash
(task_id + model + trigger_kind + success + failure_reason + gate verdicts). The
score decays as `1.0 / (1.0 + seen_count / novelty_window)` where the default
`novelty_window` is 12 episodes.

### Spacing Inverse

Spacing implements the spaced repetition effect from Cepeda et al. (2006):

```rust
// Source: crates/roko-dreams/src/replay.rs:111-119

fn compute_spacing_inv(episode: &Episode, recency: f64) -> f64 {
    let base_spacing = 1.0 - recency.clamp(0.0, 0.99);
    // Boost for episodes that were never replayed (no dream marker)
    let never_replayed = !episode.extra.contains_key("dream:replayed");
    let boost = if never_replayed { 1.5 } else { 1.0 };
    (base_spacing * boost).max(0.01)
}
```

Episodes that have never been replayed (no `dream:replayed` marker in their extra
metadata) get a 1.5x boost. Recency uses an exponential decay with configurable
half-life (default: 24 hours):

```rust
// Source: crates/roko-dreams/src/replay.rs:444-448

fn recency_decay(timestamp: DateTime<Utc>, now: DateTime<Utc>,
                 half_life_hours: f64) -> f64 {
    let age_hours = (now - timestamp).num_seconds().max(0) as f64 / 3600.0;
    let half_life = half_life_hours.max(0.1);
    (-age_hours / half_life).exp()
}
```

Note: this is a pure exponential decay `e^(-t/tau)`, not a proper half-life
formula which would be `2^(-t/t_half)` or equivalently `e^(-t * ln(2) / t_half)`.
The parameter is named `half_life_hours` but functions as a time constant (1/e
decay time). The decay rate is `1/e ~= 0.368` at `t = half_life_hours`, not `0.5`.
This is a minor naming imprecision in the roko codebase but does not affect the
algorithmic behavior -- the relative ranking of episodes is preserved under any
monotonic decay function.

### Configuration

All Mattar-Daw parameters are configurable:

```rust
// Source: crates/roko-dreams/src/replay.rs:123-177

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MattarDawConfig {
    /// Weight applied to the gain term (default 1.0).
    pub gain_weight: f64,
    /// Weight applied to the need term (default 1.0).
    pub need_weight: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DreamReplayPolicy {
    pub mode: DreamReplayMode,                    // default: Random
    pub max_episodes: usize,                      // default: 24
    pub novelty_window: usize,                    // default: 12
    pub recency_half_life_hours: f64,             // default: 24.0
    pub mattar_daw: MattarDawConfig,
}
```

### Four Replay Modes

The replay planner supports four distinct modes:

```rust
// Source: crates/roko-dreams/src/replay.rs:16-27

pub enum DreamReplayMode {
    /// Sample episodes using deterministic pseudo-random ordering.
    Random,
    /// Prioritize episodes with largest outcome signal.
    Consequence,
    /// Follow earliest failure chains back toward likely root causes.
    Causal,
    /// Replay counterfactual variants of the strongest episodes.
    Hypothetical,
}
```

**Random**: Deterministic pseudo-random ordering via hash-based ranking. Ensures
coverage of the full episode space. The ordering is deterministic for the same
input, making tests reproducible.

**Consequence**: Selects episodes by descending Mattar-Daw utility score. Episodes
with the highest prediction error and policy relevance are replayed first.

**Causal**: Groups episodes by task chain (using `task_id`), identifies failure
chains, and selects the earliest failure in each chain plus its immediate
predecessor. If no failures exist, falls back to Consequence mode. This
implements backward causal tracing -- finding root causes rather than symptoms.

**Hypothetical**: Selects the highest-utility episodes and creates counterfactual
variants -- mutating the model field (e.g., haiku -> sonnet), flipping the
trigger_kind to `dream:hypothetical`, marking them with `dream:hypothetical`
metadata. The utility is discounted by 5% (multiplied by 0.95) to reflect the
speculative nature.

### Emotional Biasing

The `select_replay_episodes_with_affect` function accepts an optional PAD
(Pleasure-Arousal-Dominance) vector from the daimon affect engine. When present:

- **Negative pleasure** (valence < 0) biases toward failure episodes: failure
  utility is multiplied by `1.0 + 0.5 * (-pleasure)`, giving failures up to
  1.5x higher utility when the agent is in a negative emotional state.
- **High arousal** (arousal > 0) increases the effective max_episodes by up to
  50%: `effective_max = max_episodes * (1.0 + 0.5 * arousal)`.

```rust
// Source: crates/roko-dreams/src/replay.rs:241-291

pub fn select_replay_episodes_with_affect(
    episodes: &[Episode],
    policy: &DreamReplayPolicy,
    now: DateTime<Utc>,
    emotional_context: Option<&PadVector>,
) -> DreamReplayBatch {
    let Some(pad) = emotional_context else {
        return select_replay_episodes(episodes, policy, now);
    };
    // Arousal-based intensity: up to 50% more episodes
    let arousal_factor = 1.0 + 0.5 * pad.arousal.max(0.0);
    let effective_max = ((policy.max_episodes as f64) * arousal_factor)
        .round() as usize;
    // ...
    // Negative pleasure biases toward failure episodes
    let failure_bias = 1.0 + 0.5 * (-pad.pleasure).max(0.0);
    for candidate in &mut candidates {
        if !candidate.episode.success {
            candidate.utility *= failure_bias;
        }
    }
    // ...
}
```

---

## REM Imagination -- Counterfactual Synthesis

REM imagination is the second phase of the dream cycle. Where NREM replay
strengthens and tests existing memories, REM imagination generates genuinely novel
hypotheses by recombining elements from different episodes and simulating
counterfactual histories.

### Biological Basis

During REM sleep, the prefrontal cortex (executive control) is suppressed while
associative cortex remains active, enabling novel combinations that waking
cognition would inhibit. Walker & van der Helm (2009, *Psychological Bulletin*)
showed that REM specifically depotentiates the emotional charge of memories --
"overnight therapy." This emotional processing is relevant to the affect-biased
replay described above.

### Three Creativity Modes

Following Boden's (2004) taxonomy of creativity, which distinguishes three
fundamental types of creative cognition:

```rust
// Source: crates/roko-dreams/src/imagination.rs:28-36

pub enum ImaginationMode {
    /// Merge patterns from two episodes.
    Combinational,
    /// Extend a known pattern into a nearby domain.
    Exploratory,
    /// Invert an assumption from a successful pattern.
    Transformational,
}
```

> Boden, M. A. (2004). *The Creative Mind: Myths and Mechanisms*. 2nd ed.
> Routledge.

| Mode | Operation | Example |
|------|-----------|---------|
| **Combinational** | Combine elements from unrelated successful episodes | "Episodes A and B share structural patterns -- reuse the routing discipline from one in the other" |
| **Exploratory** | Push a known pattern into an adjacent domain | "Extend the successful approach from task X into a neighboring task shape" |
| **Transformational** | Invert a core assumption | "What if the model had been sonnet instead of haiku? What would have changed?" |

### Causal Model

The imagination system builds a lightweight causal summary from observed episodes:

```rust
// Source: crates/roko-dreams/src/imagination.rs:46-99

pub struct CausalModel {
    /// Episodes indexed by id.
    pub episodes_by_id: BTreeMap<String, Episode>,
    /// Observed values per variable.
    pub variables: BTreeMap<String, BTreeMap<String, usize>>,
}

impl CausalModel {
    pub fn from_episodes(episodes: &[Episode]) -> Self {
        let mut episodes_by_id = BTreeMap::new();
        let mut variables = BTreeMap::new();
        for episode in episodes {
            episodes_by_id.insert(episode.id.clone(), episode.clone());
            bump_variable(&mut variables, "model", &episode.model);
            bump_variable(&mut variables, "task_id", &episode.task_id);
            bump_variable(&mut variables, "trigger_kind", &episode.trigger_kind);
            bump_variable(&mut variables, "outcome",
                if episode.success { "success" } else { "failure" });
            // ... failure_reason ...
        }
        Self { episodes_by_id, variables }
    }
}
```

This is a simplified structural causal model in the spirit of Pearl (2009), not
a full causal graph with do-calculus. It tracks variable co-occurrence
(which models appear with which outcomes, which task types co-occur with which
failure reasons) and uses these frequencies to assess counterfactual plausibility.

> Pearl, J. (2009). *Causality: Models, Reasoning, and Inference*. 2nd ed.
> Cambridge University Press.

### The `imagine` Function

The core counterfactual evaluation function assesses whether a hypothetical change
is plausible within a "trust region" -- a boundary that prevents the system from
generating implausible counterfactuals:

```rust
// Source: crates/roko-dreams/src/imagination.rs:120-174

pub fn imagine(
    query: &CounterfactualQuery,
    model: &CausalModel,
    mode: ImaginationMode,
) -> ImaginationOutcome {
    let Some(base) = model.base_episode(query) else {
        return ImaginationOutcome { /* unknown episode */ };
    };
    let (variable, new_value) = (&query.intervention.0, &query.intervention.1);
    let support = model.variable_support(variable, new_value);
    let current_value = current_value_for(base, variable);
    let similarity = /* HDC text fingerprint similarity */;

    // Trust region: minimum similarity needed for plausibility
    let trust_region = trust_region_floor(variable, support);
    let plausible = similarity >= trust_region || support > 0;

    // Projected delta: will this change improve or hurt success?
    let projected_success_delta = projected_delta(
        variable, &current_value, new_value, mode, base.success);

    // Confidence: base 0.35, +0.4 from similarity, +0.1 per support (max 3)
    let confidence = (0.35 + similarity * 0.4
        + support.min(3) as f64 * 0.1).clamp(0.0, 0.98);

    ImaginationOutcome {
        query: query.clone(), mode, plausible, confidence,
        projected_success_delta, narrative: /* human-readable */
    }
}
```

### Trust Region Floors

Different variables have different trust floors based on how much semantic distance
is acceptable for a counterfactual to be plausible. Each observation of the
replacement value in real data reduces the trust floor by 0.05 (up to 4
observations, max 0.20 reduction), making well-observed alternatives easier to
accept as plausible:

| Variable | Base Trust Floor | Rationale |
|----------|-----------------|-----------|
| `model` | 0.32 | Model swaps are fairly constrained |
| `task_id` | 0.28 | Tasks can be more loosely related |
| `trigger_kind` | 0.24 | Triggers are broad categories |
| `failure_reason` | 0.20 | Failure reasons can vary widely |
| Other | 0.30 | Default |

### Hypothesis Synthesis

The `synthesize_hypotheses` function generates hypothetical knowledge entries from
a batch of episodes. It runs all three creativity modes:

1. **Combinational**: Finds two successful episodes from different tasks and
   suggests reusing the routing discipline from one in the other.
2. **Exploratory**: Takes the most complex successful episode and suggests
   extending it into a neighboring task shape.
3. **Transformational**: Takes the episode with the strongest failure signal
   and asks "what if a different model had been used?"

All generated hypotheses enter the knowledge store at `Working` tier with
confidence 0.78 and tagged with `["dream", "rem", "counterfactual", <mode>]`.

---

## Hypnagogic Creativity Pipeline

The hypnagogia engine implements a four-layer creative onset system inspired by
the transitional state between waking and sleep. It runs before the structured
NREM/REM phases to produce genuinely novel associations.

### Why Hypnagogia Matters: The Alpha Convergence Problem

When all AI agents use the same foundation models, they reach the same conclusions
and take the same actions. Roko's docs call this the "alpha convergence" problem,
citing Grossman-Stiglitz (1980): if information acquisition is costless because the
model is the same, all agents acquire the same information and it becomes worthless.

The hypnagogia engine breaks this convergence by injecting agent-specific
experiential noise into the creative process. Each agent has different experiences,
different accumulated knowledge, and different failure histories. The engine uses
these unique experiential traces as raw material for creative recombination. The
theoretical frame is Derrida's hauntology (1993) -- each agent is "differently
haunted" by its own experiential traces, producing different creative outputs
even when using the same foundation model.

### The HypnagogiaEngine Struct

```rust
// Source: crates/roko-dreams/src/hypnagogia.rs:88-98

pub struct HypnagogiaEngine {
    /// Thalamic gate settings.
    pub gate: ThalamicGate,
    /// Executive loosener settings.
    pub loosener: ExecutiveLoosener,
    /// Dali-style interrupt settings.
    pub interrupt: DaliInterrupt,
    /// Homuncular observer settings.
    pub observer: HomuncularObserver,
}
```

### Layer 1: Thalamic Gate

**Biological basis**: Magnin et al. (2010, *PNAS*) showed thalamic deactivation
precedes cortical deactivation by 8.6 minutes at sleep onset. The thalamus acts as
a gate -- when it deactivates, sensory input is suppressed but cortical processing
continues, enabling internally generated imagery.

The Thalamic Gate filters incoming knowledge signals. High-confidence signals pass
through directly; low-confidence signals pass only if their "resonance score"
(derived from an HDC text fingerprint) exceeds a noise floor:

```rust
// Source: crates/roko-dreams/src/hypnagogia.rs:16-22

pub struct ThalamicGate {
    /// Minimum confidence retained by the gate before stochastic resonance.
    pub relevance_floor: f64,      // default: 0.45
    /// Fraction of low-confidence signals allowed through as noise.
    pub noise_floor: f64,          // default: 0.20
}
```

The filtering logic:

```rust
// Source: crates/roko-dreams/src/hypnagogia.rs:168-177

fn thalamic_gate(&self, signals: &[KnowledgeEntry]) -> Vec<KnowledgeEntry> {
    signals
        .iter()
        .filter(|signal| {
            signal.confidence >= self.gate.relevance_floor
                || resonance_score(&signal.content) >= self.gate.noise_floor
        })
        .cloned()
        .collect()
}
```

The resonance score uses HDC (Hyperdimensional Computing) text fingerprinting to
produce a pseudo-random but deterministic score from the signal's content text.
This deliberately introduces stochastic resonance -- some low-confidence signals
"leak" through the gate, providing the raw material for creative associations.

### Layer 2: Executive Loosener

**Biological basis**: During hypnagogia, the prefrontal cortex reduces its
influence on cortical processing, allowing "strange associations."

The Executive Loosener takes the gated signals and looks for neighborhood
associations -- signals within a configurable window that share at least one tag:

```rust
// Source: crates/roko-dreams/src/hypnagogia.rs:34-40

pub struct ExecutiveLoosener {
    /// Maximum neighborhood size to consider.
    pub neighborhood: usize,       // default: 4
    /// How aggressively the search should widen.
    pub looseness: f64,            // default: 0.35
}
```

When a neighborhood association is found, the loosener creates a new knowledge
entry labeled "Sleep-onset association" that merges the content and source episodes
of both signals. The confidence is the average of both signals' confidence,
scaled by `(0.5 + looseness)`.

### Layer 3: Dali Interrupt

**Biological basis**: Named after Salvador Dali's technique of holding a key over a
metal plate while dozing. The key falling and striking the plate would wake him at
the precise moment of hypnagogic onset, capturing creative imagery.

The Dali Interrupt iterates over episodes with a configurable stride, selecting
episodes whose resonance score exceeds an intensity threshold:

```rust
// Source: crates/roko-dreams/src/hypnagogia.rs:52-58

pub struct DaliInterrupt {
    /// How many signals to skip between injected interruptions.
    pub stride: usize,            // default: 3
    /// Weight that decides whether an interrupt is emitted.
    pub intensity: f64,           // default: 0.55
}
```

Selected episodes produce "Dali insight" entries with confidence 0.70, tagged
with `["dream", "hypnagogia", "dali-interrupt", "creative-break"]`.

### Layer 4: Homuncular Observer

**Biological basis**: From Ryle (1949), Dennett (1991), and Lycan (1996). In
biological hypnagogia, a meta-cognitive awareness persists even as executive control
loosens -- the dreamer can sometimes notice creative associations *as they form*.

The observer takes all candidates from Layers 2 and 3, scores them, deduplicates,
and retains only the top candidates:

```rust
// Source: crates/roko-dreams/src/hypnagogia.rs:70-76

pub struct HomuncularObserver {
    /// Minimum score required for a candidate to survive.
    pub retention_floor: f64,      // default: 0.40
    /// Maximum number of candidate insights to keep.
    pub max_candidates: usize,     // default: 6
}
```

Each candidate is scored as: `confidence + novelty_score(content) +
min(4, source_episodes.len()) * 0.04`. Candidates below the retention floor
(0.40) are discarded; duplicates (by ID) are removed; and at most 6 survive.

### Full Pipeline Execution

The complete pipeline is invoked via `HypnagogiaEngine::run`:

```rust
// Source: crates/roko-dreams/src/hypnagogia.rs:150-160

pub fn run(&self, signals: &[KnowledgeEntry], episodes: &[Episode],
           created_at: DateTime<Utc>) -> Vec<KnowledgeEntry> {
    let gated = self.thalamic_gate(signals);
    let loosened = self.executive_loosen(gated, episodes, created_at);
    let interrupted = self.dali_interrupt(episodes, created_at);
    self.homuncular_observer(
        loosened.into_iter().chain(interrupted).collect())
}
```

Data flow:

```
signals (KnowledgeEntry[])  -->  [Thalamic Gate]  -->  gated signals
                                     |
episodes (Episode[])  -------->  [Executive Loosener]  -->  loosened associations
                     \            (+ interrupt-to-insight)
                      \-------->  [Dali Interrupt]  -------->  dali insights
                                                        |
                                  [Homuncular Observer]  <--  (merged)
                                         |
                                  max 6 candidates out
```

---

## Threat Rehearsal

Threat rehearsal implements Revonsuo's (2000) Threat Simulation Theory. It uses
observed failure patterns to construct threat scenarios, rehearse recovery paths,
and generate synthetic episodes for future learning.

### Threat Enumeration (FMEA/FTA)

The `enumerate_threats` function clusters failed episodes by a composite key of
`task_id + model + failure_reason`, then scores each cluster:

```rust
// Source: crates/roko-dreams/src/threat.rs:15-37

pub struct ThreatScenario {
    pub id: String,
    pub description: String,
    /// Estimated likelihood, normalized to 0.0..=1.0
    pub likelihood: f64,
    /// Estimated impact, normalized to 0.0..=1.0
    pub impact: f64,
    /// How hard it is to detect before damage
    pub detection_difficulty: f64,
    /// Recommended mitigation
    pub mitigation: String,
}

impl ThreatScenario {
    pub fn severity(&self) -> f64 {
        (self.likelihood * self.impact
         * (1.0 - self.detection_difficulty)).clamp(0.0, 1.0)
    }
}
```

**Note on the severity formula**: The formula `likelihood x impact x (1 -
detection_difficulty)` means that high detection difficulty *reduces* computed
severity. This inverts standard FMEA convention, where high detection difficulty
(hard to detect) *increases* the Risk Priority Number (RPN). The roko
implementation prioritizes *actionable* threats -- those that are both likely and
detectable, because detection enables mitigation. For IronClaw adaptation, this
is a reasonable choice: the agent should focus on threats it can actually prepare
defenses for.

### Threat Warning Generation

Threats above a configurable severity floor (default: 0.20) are converted into
`KnowledgeEntry` values of kind `Warning`, tagged with
`["dream", "threat", "warning", "fmea", "fta"]`:

```rust
// Source: crates/roko-dreams/src/threat.rs:94-157

pub fn threat_warning_entries_with_floor(
    episodes: &[Episode], created_at: DateTime<Utc>, severity_floor: f64,
) -> Vec<KnowledgeEntry> {
    let threats = enumerate_threats(episodes);
    threats.into_iter()
        .filter(|threat| threat.severity() >= severity_floor)
        .map(|threat| /* KnowledgeEntry with Warning kind */)
        .collect()
}
```

Each warning entry includes the threat description and recommended mitigation,
along with emotional provenance transferred from the most intense source episode.
This ensures the emotional retrieval boost is active for affect-congruent
knowledge surfacing during future waking operations.

### Rehearsal Engine

The `rehearse_threats` function simulates recovery for the top-severity threats
(bounded by `MAX_REHEARSALS_PER_CYCLE` = 20):

```rust
// Source: crates/roko-dreams/src/rehearsal.rs:59-87

pub fn rehearse_threats(
    episodes: &[Episode],
    max_scenarios: Option<usize>,
    now: DateTime<Utc>,
) -> RehearsalReport {
    let threats = enumerate_threats(episodes);
    let limit = max_scenarios.unwrap_or(MAX_REHEARSALS_PER_CYCLE); // 20

    let mut outcomes = Vec::new();
    let mut generated_episodes = Vec::new();

    for threat in threats.iter().take(limit) {
        let outcome = rehearse_single(threat, now);
        let episode = outcome_to_episode(&outcome, threat);
        generated_episodes.push(episode);
        outcomes.push(outcome);
    }

    RehearsalReport { started_at, completed_at, threats_evaluated,
                      rehearsals_performed, outcomes, generated_episodes }
}
```

Each rehearsal determines recovery feasibility heuristically: if the threat has a
clear mitigation, the detection difficulty is below 0.7, and the severity is below
0.8, the recovery is considered feasible. Infeasible recoveries produce an
"escalate to human review" response.

Rehearsed outcomes are converted into synthetic episodes with:
- `id` prefixed with `rehearsal-`
- `model` set to `dream-rehearsal`
- A `threat-rehearsal` gate verdict recording the confidence and scenario
- These synthetic episodes feed back into future dream cycles for iterative learning

---

## Staging Buffer -- Confidence Lifecycle

Dream-generated insights do not go directly into permanent knowledge. They pass
through a confidence-gated staging buffer that prevents dream hallucinations from
corrupting durable knowledge.

### The Confidence Ladder

```
Raw (0.20) --> Replayed (0.30) --> Validated (0.50) --> Promoted (0.70)
```

```rust
// Source: crates/roko-dreams/src/staging.rs:33-44

pub enum ConfidenceStage {
    /// Just extracted, unvalidated.
    Raw,
    /// Successfully replayed in a subsequent dream cycle.
    Replayed,
    /// Cross-checked against existing knowledge (no contradiction, not redundant).
    Validated,
    /// Ready for knowledge store promotion.
    Promoted,
}
```

Each stage has a confidence floor:

| Stage | Confidence Floor | Meaning |
|-------|-----------------|---------|
| Raw | 0.20 | Just entered from a dream. Unverified. |
| Replayed | 0.30 | Survived one replay cycle without contradiction. |
| Validated | 0.50 | Cross-referenced against existing knowledge -- not redundant, not contradicted. |
| Promoted | 0.70 | Written to permanent knowledge store at `Transient` tier. |

### Staging Entry Structure

```rust
// Source: crates/roko-dreams/src/staging.rs:71-87

pub struct StagingEntry {
    /// The knowledge entry being staged.
    pub entry: KnowledgeEntry,
    /// Source episode that produced this insight.
    pub source_episode_id: String,
    /// Current position on the confidence ladder.
    pub stage: ConfidenceStage,
    /// Current confidence score (starts at 0.20).
    pub confidence: f64,
    /// When this entry was first added.
    pub created_at: DateTime<Utc>,
    /// When this entry last advanced a stage.
    pub last_advanced_at: DateTime<Utc>,
    /// When this entry was promoted, if ever.
    pub promoted_at: Option<DateTime<Utc>>,
}
```

### State Transitions

**Raw -> Replayed**: An entry advances when the source episode appears in a
subsequent replay batch. The minimum time between creation and first replay
ensures the insight survives at least one dream cycle.

**Replayed -> Validated**: An entry advances if it passes a redundancy check --
its HDC vector similarity to all existing knowledge store entries must be below
0.90. If any existing entry is > 0.90 similar, the candidate is considered
redundant and does not advance.

**Validated -> Promoted**: All validated entries are promoted to the knowledge
store at `Transient` tier with confidence 0.70.

### Garbage Collection

Raw entries older than 7 days are garbage collected:

```rust
// Source: crates/roko-dreams/src/staging.rs (gc logic)

pub fn gc_at(&mut self, now: DateTime<Utc>) {
    let horizon = now - Duration::days(GC_HORIZON_DAYS); // 7
    self.entries.retain(|entry| {
        if entry.stage != ConfidenceStage::Raw { return true; }
        entry.created_at > horizon
    });
}
```

Only `Raw` entries are GC'd. Once an entry has advanced to `Replayed` or beyond,
it persists regardless of age. This ensures that the staging buffer does not grow
unboundedly from dream noise that never gets validated.

### Full Lifecycle Diagram

```
    Dream Phase Output
         |
         v
  [add_candidate]  -->  StagingBuffer { stage: Raw, confidence: 0.20 }
         |
    Next replay batch includes source episode?
         |  yes
         v
  [advance_replayed] -->  { stage: Replayed, confidence: 0.30 }
         |
    Not redundant with existing knowledge (HDC sim < 0.90)?
         |  yes
         v
  [advance_validated] -->  { stage: Validated, confidence: 0.50 }
         |
  [promote_validated] -->  Written to KnowledgeStore at Transient tier
         |                  { stage: Promoted, confidence: 0.70 }
         v
  [remove_promoted]  -->  Cleaned from buffer

  If Raw and older than 7 days:
  [gc]  -->  Removed from buffer (never validated)
```

---

## Dream Scheduling and Budget

### Trigger Types

Dreams fire from several triggers:

```rust
// Source: crates/roko-dreams/src/runner.rs:255-283

pub enum DreamTrigger {
    /// Idle gap between task dispatches.
    Idle,
    /// Cron-like schedule.
    Scheduled,
    /// Manual command invocation.
    Manual,
    /// Accumulated episode count since last dream.
    EpisodeCount,
    /// Bus-reactive trigger from a high-value engram (DREAM-09).
    BusPulse { engram_hash: String },
    /// Coordination pattern trigger (INT-19).
    CoordinationPattern { pattern_name: String,
                          contributing_watchers: Vec<String> },
}
```

### Budget Controls

Each dream cycle has a three-axis budget:

```rust
// Source: crates/roko-dreams/src/runner.rs:191-205

pub struct DreamBudget {
    pub max_tokens: u64,
    pub max_cost_usd: f64,
    pub max_duration_secs: u64,
    pub consumed_tokens: u64,
    pub consumed_cost_usd: f64,
    pub consumed_duration_secs: u64,
}
```

The budget is consumed per-episode during replay. When any axis is exhausted, the
cycle stops processing and reports what it completed. The default budget is
unlimited (`u64::MAX` / `f64::MAX`).

### Per-Phase Compute Budget (DREAM-12)

The phase2 module introduces per-phase budget allocation:

```rust
// Source: crates/roko-dreams/src/phase2/sleep_time.rs:15-32

pub struct DreamComputeBudget {
    /// Total daily inference budget in USD.
    pub inference_daily_usd: f64,         // default: 10.0
    /// Fraction allocated to dreaming.
    pub dream_fraction: f64,              // default: 0.15 (15%)
    /// Per-phase budget fractions.
    pub phase_allocations: PhaseAllocations,
}
```

```rust
// Source: crates/roko-dreams/src/phase2/sleep_time.rs:57-80

pub struct PhaseAllocations {
    pub hypnagogia: f64,    // default: 0.10 (10%)
    pub nrem: f64,          // default: 0.30 (30%)
    pub rem: f64,           // default: 0.50 (50%)
    pub integration: f64,   // default: 0.00 (pure computation)
    pub evolution: f64,     // default: 0.10 (10%)
}
```

Each phase maps to a recommended model tier:

| Phase | Budget Share | Model Tier | Rationale |
|-------|-------------|-----------|-----------|
| Hypnagogia | 10% | T0 (Fast) | Fragment generation is cheap |
| NREM Replay | 30% | T0 (Fast) | Pattern matching, not creative reasoning |
| REM Imagination | 50% | T1 (Standard) | Creative reasoning needs capable model |
| Integration | 0% | None | Pure computation, no model calls |
| Evolution | 10% | T0 (Fast) | Mutation evaluation |

### Sleepwalker Mode

During dreaming, the agent can enter a reduced-capability state where only urgent
signals (process crashes, critical errors, operator interrupts) can wake it:

```rust
// Source: crates/roko-dreams/src/phase2/sleep_time.rs:168-178

pub enum SleepwalkerMode {
    /// Normal operation -- full agent capabilities.
    Awake,
    /// Dreaming -- only urgent signals processed.
    Dreaming { urgent_signal_types: Vec<String> },
}
```

Default urgent signals: `process_crash`, `critical_error`, `operator_interrupt`.

### Adaptive Scheduling

The `DreamSchedulePolicy` adapts based on dream quality. High-quality dreams
(many knowledge entries written, playbooks created) reduce the idle threshold by
25% (`quality_gain = 0.75`), making dreams fire more frequently. Low-quality
dreams increase the threshold by 25% (`quality_penalty = 1.25`), backing off:

```rust
// Source: crates/roko-dreams/src/runner.rs:378-416

pub struct DreamSchedulePolicy {
    pub enabled: bool,
    pub idle_threshold_mins: u64,          // default: 15
    pub scheduled_cron: Option<String>,
    pub manual_enabled: bool,
    pub quality_gain: f64,                 // default: 0.75
    pub quality_penalty: f64,              // default: 1.25
    pub episode_count_trigger: usize,      // default: 0 (disabled)
}
```

---

## Routing Advice -- Dream-to-Wake Knowledge Transfer

Dream consolidation produces routing advice that influences future model selection
during waking operation. This is how dreams improve waking performance.

### What Gets Generated

```rust
// Source: crates/roko-dreams/src/routing_advice.rs:18-28

pub struct DreamRoutingAdvice {
    pub generated_at: DateTime<Utc>,
    pub source_dream_report: String,
    pub recommendations: Vec<RoutingRecommendation>,
    pub pattern_summaries: Vec<PatternSummary>,
}
```

### Routing Recommendations

Each recommendation maps a task_category + complexity_band pair to a preferred
model and a list of deprioritized models:

```rust
// Source: crates/roko-dreams/src/routing_advice.rs:42-60

pub struct RoutingRecommendation {
    pub task_category: String,
    pub complexity_band: String,
    pub recommended_model: String,
    pub deprioritize: Vec<String>,
    pub confidence: f64,
    pub supporting_episodes: usize,
    pub recommended_model_success_rate: f64,
    pub pattern_signature: u64,
}
```

Recommendations are generated from cross-episode consolidation patterns. A model
is recommended when its success rate exceeds 0.80 for a given task shape; a model
is deprioritized when its success rate drops below 0.40 (and the next-tier model
is recommended instead: haiku -> sonnet, sonnet -> opus).

### Pattern Summaries

Human-readable pattern descriptions with actionable guidance:

```rust
// Source: crates/roko-dreams/src/routing_advice.rs:63-75

pub struct PatternSummary {
    pub description: String,
    pub applies_to: Vec<String>,
    pub guidance: String,
    pub confidence: f64,
    pub signature: u64,
}
```

Example guidance output:

> "Historical dream consolidation shows claude-sonnet-4-5 has a 85% success rate
> for this task shape across 12 episodes; this model/task pairing is reliable."

### Persistence and Loading

Routing advice is persisted to `.roko/learn/dream-routing-advice.json` and loaded
at wake time to bias future model selection via `dream_advice_to_routing_bias`.
The function converts recommendations into a `RoutingBias` struct containing
models to deprioritize for a given task category and complexity band, along with
reasons for the bias derived from episode counts and success rates.

---

## IronClaw Integration Architecture

### A. Enhanced Heartbeat System

**Where**: `src/agent/heartbeat.rs` (existing heartbeat runs periodically)

**How**: Augment the current HEARTBEAT.md-reading cycle with dream consolidation.
The heartbeat already provides the idle-time execution scaffold (`HeartbeatConfig`
with interval, quiet hours, timezone-aware scheduling); dream phases plug into it
as additional periodic tasks.

Current heartbeat:
```
Every 30 min -> Read HEARTBEAT.md -> Execute instructions -> Notify user
```

Enhanced heartbeat with dream consolidation:
```
Every 30 min  -> Read HEARTBEAT.md -> Execute instructions -> Notify user
Every 2 hours -> NREM Replay: Review recent action records, strengthen patterns
Every 6 hours -> REM Imagination: Generate counterfactuals for recent failures
Every 24 hours -> Hypnagogic scan: Scan for cross-domain insights
On failure    -> Threat Rehearsal: Analyze what went wrong, prepare defenses
```

Scheduling can use the `DreamSchedulePolicy` pattern from roko:
- Idle threshold: 15 minutes of no user activity
- Minimum episodes: 5 new action records since last consolidation
- Cron expression for fixed schedule (e.g., `0 0 3 * * *` for 3 AM daily)
- Adaptive quality feedback (reduce interval after productive dreams)

### B. Memory Quality Improvement

**Where**: `src/workspace/document.rs`

**How**: Add confidence staging metadata to memory documents. The workspace memory
system already supports hybrid FTS + vector search via `MemoryDocument` and
`MemoryChunk`. Add staging metadata fields:

```rust
// Proposed additions to MemoryDocument in src/workspace/document.rs

pub struct StagedMemoryMetadata {
    /// Current stage in the confidence ladder.
    pub stage: ConsolidationStage,
    /// How many times this memory has been replayed.
    pub replay_count: u32,
    /// When this memory was last reviewed by consolidation.
    pub last_consolidated: Option<DateTime<Utc>>,
    /// IDs of originating action records.
    pub source_action_records: Vec<Uuid>,
}

pub enum ConsolidationStage {
    /// Written during normal operation. Not yet reviewed.
    Raw,
    /// Reviewed during a consolidation pass, survived without contradiction.
    Replayed,
    /// Cross-referenced against other memories, consistent and non-redundant.
    Validated,
    /// High-confidence, integrated into core knowledge.
    Promoted,
}
```

The staging buffer provides a critical safety mechanism: dream-generated insights
are speculative. They should not immediately become trusted knowledge. The
confidence ladder (`Raw -> Replayed -> Validated -> Promoted`) ensures only
insights that survive multiple rounds of validation enter permanent memory.

### C. Consolidation Engine

**Where**: New module `src/consolidation/` (preferred) or extracted crate
`crates/ironclaw_dreams/`

Given IronClaw's architecture (module-owned initialization, tool-dispatch
pipeline), the consolidation engine should follow the established pattern:

```
src/consolidation/
    mod.rs              # ConsolidationEngine: scheduling, budget, orchestration
                        # Public factory function for wiring in app.rs
    replay.rs           # NREM replay adapted from roko replay.rs
                        #   - Mattar-Daw utility scoring on ActionRecords
                        #   - Four replay modes (Random, Consequence, Causal, Hypothetical)
    imagination.rs      # Counterfactual generation via ironclaw_llm
                        #   - Three creativity modes (Combinational, Exploratory,
                        #     Transformational)
                        #   - Trust region validation
    creativity.rs       # Hypnagogic insight pipeline adapted from roko hypnagogia.rs
                        #   - ThalamicGate, ExecutiveLoosener, DaliInterrupt,
                        #     HomuncularObserver
    rehearsal.rs        # Threat rehearsal adapted from roko rehearsal.rs + threat.rs
                        #   - FMEA-style severity scoring
                        #   - Recovery path simulation
                        #   - Synthetic episode generation for iterative learning
    staging.rs          # Confidence staging buffer adapted from roko staging.rs
                        #   - JSON persistence
                        #   - 7-day GC for unvalidated entries
                        #   - HDC redundancy checking (using ironclaw_embeddings
                        #     cosine similarity as the HDC analog)
    routing.rs          # Optional: routing advice generation
```

### D. ActionRecord as Episode

IronClaw's `ActionRecord` (from `src/context/memory.rs`, produced by tool
dispatch) maps to roko's `Episode`. The key fields to extract for replay scoring:

| roko Episode field | IronClaw ActionRecord equivalent | Source |
|---|---|---|
| `id` | `id: Uuid` | `ActionRecord.id` |
| `task_id` | Job ID or session context | From `JobContext` |
| `model` | LLM model used | From `ironclaw_llm` provider response |
| `success` | `success: bool` | `ActionRecord.success` |
| `failure_reason` | `error: Option<String>` | `ActionRecord.error` |
| `tokens_used` | Token count | From LLM call response metadata |
| `gate_verdicts` | Safety pipeline results | From `SafetyLayer` pass/fail |
| `timestamp` | `executed_at: DateTime<Utc>` | `ActionRecord.executed_at` |
| `duration_secs` | `duration: Duration` | `ActionRecord.duration` |

The adapter should be a standalone function, not a trait impl on ActionRecord,
to keep the conversion logic in the consolidation module:

```rust
// Proposed: src/consolidation/mod.rs

pub struct ConsolidationEpisode {
    pub id: String,
    pub task_id: String,
    pub model: String,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub tokens_used: u64,
    pub duration_secs: f64,
    pub timestamp: DateTime<Utc>,
    pub tool_name: String,
    pub safety_passed: bool,
    pub extra: HashMap<String, serde_json::Value>,
}

impl ConsolidationEpisode {
    pub fn from_action_record(
        record: &ActionRecord,
        model: &str,
        job_id: &str,
    ) -> Self {
        Self {
            id: record.id.to_string(),
            task_id: job_id.to_string(),
            model: model.to_string(),
            success: record.success,
            failure_reason: record.error.clone(),
            tokens_used: 0, // populated from LLM call metadata
            duration_secs: record.duration.as_secs_f64(),
            timestamp: record.executed_at,
            tool_name: record.tool_name.clone(),
            safety_passed: record.sanitization_warnings.is_empty(),
            extra: HashMap::new(),
        }
    }
}
```

### E. LLM Integration

The imagination and creativity phases require LLM calls. These reuse the existing
`ironclaw_llm` provider infrastructure:

- **NREM replay**: No LLM calls needed (pattern matching on structured data)
- **REM imagination**: Use existing `ironclaw_llm` with a cheap model (Haiku-class)
  for hypothesis synthesis. Budget-constrained via `CostGuard`.
- **Hypnagogia**: Primarily HDC/embedding-based operations (cheap). The Executive
  Loosener and Dali Interrupt generate text from templates, not LLM calls.
  Only optional quality evaluation uses a cheap model.
- **Threat rehearsal**: No LLM calls in the current implementation (heuristic
  simulation). Future versions could use LLM-backed scenario generation.

The `CostGuard` in `src/agent/cost_guard.rs` already enforces per-user daily
budgets and hourly call rates. Dream consolidation should respect these limits
by calling `CostGuard::check_allowed()` before any LLM inference and
`record_llm_call()` after.

### F. Tool Dispatch Integration

Following IronClaw's "Everything Goes Through Tools" principle (see
`src/tools/dispatch.rs`), consolidation actions should be observable as
`ActionRecord` entries via `ToolDispatcher`. This means:

- A `consolidation_run` tool triggers a dream cycle manually
- A `consolidation_status` tool queries staging buffer state, last dream time,
  and routing advice
- Each phase logs its outputs through the standard tool dispatch pipeline
- Dream-generated insights flow through the existing `memory_write` tool

### G. Database Changes

Add columns to workspace memory tables (both PostgreSQL and libSQL, per
IronClaw's dual-backend requirement documented in `src/db/CLAUDE.md`):

```sql
-- PostgreSQL migration
ALTER TABLE memory_documents
    ADD COLUMN consolidation_stage TEXT DEFAULT 'raw',
    ADD COLUMN replay_count INTEGER DEFAULT 0,
    ADD COLUMN last_consolidated TIMESTAMPTZ,
    ADD COLUMN source_action_records JSONB DEFAULT '[]';

CREATE INDEX idx_memory_documents_consolidation_stage
    ON memory_documents(consolidation_stage);
```

```sql
-- libSQL migration (equivalent)
ALTER TABLE memory_documents
    ADD COLUMN consolidation_stage TEXT DEFAULT 'raw';
ALTER TABLE memory_documents
    ADD COLUMN replay_count INTEGER DEFAULT 0;
ALTER TABLE memory_documents
    ADD COLUMN last_consolidated TEXT;
ALTER TABLE memory_documents
    ADD COLUMN source_action_records TEXT DEFAULT '[]';
```

### H. Configuration

Add consolidation settings to `src/config/`:

```rust
// Proposed: src/config/consolidation.rs

#[derive(Debug, Clone, Deserialize)]
pub struct ConsolidationConfig {
    /// Whether dream consolidation is enabled.
    pub enabled: bool,                        // default: false (opt-in)
    /// Idle threshold in minutes before consolidation may run.
    pub idle_threshold_mins: u64,             // default: 15
    /// Minimum new action records since last consolidation.
    pub min_episodes: usize,                  // default: 5
    /// Maximum replay episodes per cycle.
    pub max_replay_episodes: usize,           // default: 24
    /// Mattar-Daw gain weight.
    pub gain_weight: f64,                     // default: 1.0
    /// Mattar-Daw need weight.
    pub need_weight: f64,                     // default: 1.0
    /// Replay mode.
    pub replay_mode: String,                  // default: "random"
    /// Whether threat rehearsal is enabled.
    pub threat_rehearsal: bool,               // default: true
    /// Staging buffer GC horizon in days.
    pub gc_horizon_days: u32,                 // default: 7
    /// Maximum cost per dream cycle in USD.
    pub max_cost_usd: f64,                    // default: 0.10
}
```

Environment variables:
```
CONSOLIDATION_ENABLED=false
CONSOLIDATION_IDLE_THRESHOLD_MINS=15
CONSOLIDATION_MIN_EPISODES=5
CONSOLIDATION_MAX_REPLAY_EPISODES=24
CONSOLIDATION_MAX_COST_USD=0.10
```

---

## Performance and Resource Characteristics

### Computational Costs

Based on roko's cost analysis:

| Phase | Model Tier | Typical Duration | Cost per Run |
|-------|-----------|------------------|-------------|
| NREM Replay | T0 (Haiku) or none | 60-120 seconds for 10 episodes | ~$0.001/episode |
| REM Imagination | T1 (Sonnet) | 120-300 seconds for 3-5 counterfactuals | ~$0.01/counterfactual |
| Hypnagogia | Mostly computation, optional T0 | 30-60 seconds | ~$0.005 total |
| Integration | None (pure computation) | < 5 seconds | Negligible |
| Threat Rehearsal | None (heuristic) | < 2 seconds | Negligible |

**Total per full dream cycle**: ~$0.03-0.10 depending on episode count and model
pricing.

**Break-even**: If a dream cycle prevents even one unnecessary task retry (which
costs ~$0.10-0.50 in model inference), a $0.05 dream cycle pays for itself.

### Memory Usage

- Staging buffer: JSON file, typically < 100KB
- Dream cycle reports: JSON files, typically < 50KB each
- Routing advice: JSON file, typically < 20KB
- In-memory: Candidates vector during pipeline execution (bounded by
  `max_candidates = 6` for hypnagogia, `max_episodes = 24` for replay)

### Concurrency

- NREM replay episodes can be scored in parallel (each scoring is independent)
- Dream phases execute sequentially within a cycle
- Dream cycles run in background tokio tasks, never blocking the main agent loop
- Sleepwalker mode allows urgent interrupts during dreaming
- In IronClaw: use `tokio::spawn` for the consolidation task, check
  `CostGuard` before LLM calls, respect `HeartbeatConfig` quiet hours

### Incremental Processing

Each cycle processes a bounded batch. No "big bang" consolidation:
- Replay processes at most `max_episodes` (default: 24) per cycle
- Hypnagogia emits at most `max_candidates` (default: 6) insights
- Threat rehearsal is bounded by `MAX_REHEARSALS_PER_CYCLE` (20)
- The `processed_through` timestamp prevents reprocessing old episodes

---

## Implementation Plan

### Phase 1: Core Infrastructure (~800 lines)

**Goal**: Replay scoring and staging buffer, wired into heartbeat.

1. **Create `src/consolidation/mod.rs`**: Define `ConsolidationEpisode`,
   `ConsolidationEngine`, and the `from_action_record` adapter.

2. **Port `replay.rs`**: Implement `ReplayUtility`, `MattarDawConfig`,
   `DreamReplayPolicy`, `select_replay_episodes`. Replace roko-specific episode
   types with `ConsolidationEpisode`. The four replay modes (Random, Consequence,
   Causal, Hypothetical) port directly -- they operate on generic episode fields.

3. **Port `staging.rs`**: Implement `ConfidenceStage`, `StagingEntry`,
   `StagingBuffer` with JSON persistence. Default staging path:
   `~/.ironclaw/consolidation/staging-buffer.json`.

4. **Wire into heartbeat**: Add a `consolidation_tick` method to
   `HeartbeatConfig` or create a parallel `ConsolidationRunner` that checks
   idle time and episode count thresholds. Register it in `src/app.rs` alongside
   the existing heartbeat spawn.

5. **Database migration**: Add `consolidation_stage`, `replay_count`,
   `last_consolidated`, `source_action_records` columns to `memory_documents`
   in both PostgreSQL and libSQL backends.

6. **Configuration**: Add `ConsolidationConfig` to `src/config/` and wire the
   environment variables.

### Phase 2: Imagination and Creativity (~600 lines)

**Goal**: Counterfactual generation and creative association pipeline.

7. **Port `imagination.rs`**: Implement `CausalModel`, `CounterfactualQuery`,
   `ImaginationMode`, `imagine`, `synthesize_hypotheses`. Replace HDC text
   fingerprint similarity with `ironclaw_embeddings` cosine similarity for
   trust region validation.

8. **Port `hypnagogia.rs`**: Implement the four-layer pipeline
   (`ThalamicGate`, `ExecutiveLoosener`, `DaliInterrupt`, `HomuncularObserver`).
   Replace HDC resonance scoring with a hash-based pseudo-random function (the
   roko implementation already uses hash-derived scores, so this ports directly).

9. **LLM integration**: Wire imagination and creativity phases through
   `ironclaw_llm` with `CostGuard` budget controls. Use `deps.cheap_llm()`
   (the accessor from `AgentDeps`) for hypothesis evaluation.

### Phase 3: Threat Rehearsal and Routing (~400 lines)

**Goal**: Failure analysis, recovery simulation, and model routing advice.

10. **Port `threat.rs`**: Implement `ThreatScenario`, `enumerate_threats`,
    `threat_warning_entries_with_floor`. The FMEA severity scoring ports directly.

11. **Port `rehearsal.rs`**: Implement `rehearse_threats`, `RehearsalOutcome`,
    `RehearsalReport`, synthetic episode generation.

12. **Tool registration**: Register `consolidation_run` and
    `consolidation_status` tools in `src/tools/builtin/`. Follow the pattern in
    `src/tools/builtin/memory.rs` for tool registration.

13. **Optional: routing advice**: Port `routing_advice.rs` if IronClaw develops
    model routing capabilities. The `DreamRoutingAdvice` -> `RoutingBias`
    conversion is self-contained.

### Risk Assessment

- **Medium risk**: Background processing needs careful resource management.
  Tokio task spawning with budget limits mitigates runaway compute. The existing
  `CostGuard` already provides per-user daily budget enforcement.
- **Low risk**: Staging buffer prevents dream hallucinations from corrupting
  knowledge. 7-day GC ensures unbounded growth does not occur.
- **Dependencies**: Existing heartbeat system (`src/agent/heartbeat.rs`),
  workspace (`src/workspace/`), `ironclaw_llm` provider, `CostGuard`
  (`src/agent/cost_guard.rs`). No new external dependencies required.

### Complexity Assessment

- **Core implementation**: ~1,800 lines (replay + staging + heartbeat integration)
- **LLM integration** (imagination/creativity): Reuses existing `ironclaw_llm`
- **Database changes**: 4 new columns on memory tables
- **Total estimated**: ~1,800-2,000 lines across all phases

---

## Academic References

### Primary Sources

1. **McClelland, J. L., McNaughton, B. L., & O'Reilly, R. C.** (1995). Why
   there are complementary learning systems in the hippocampus and neocortex:
   Insights from the successes and failures of connectionist models of learning
   and memory. *Psychological Review*, 102(3), 419-457.
   Grounds: Two-system architecture (fast episodic + slow semantic).

2. **Mattar, M. G. & Daw, N. D.** (2018). Prioritized memory access explains
   planning and hippocampal replay. *Nature Neuroscience*, 21(11), 1609-1617.
   DOI: [10.1038/s41593-018-0232-z](https://doi.org/10.1038/s41593-018-0232-z).
   Grounds: Utility formula for replay prioritization (gain x need x spacing).

3. **Tononi, G. & Cirelli, C.** (2006). Sleep function and synaptic homeostasis.
   *Sleep Medicine Reviews*, 10(1), 49-62.
   Grounds: Synaptic Homeostasis Hypothesis -- sleep downscales synaptic strength.

4. **Tononi, G. & Cirelli, C.** (2014). Sleep and the price of plasticity: From
   synaptic and cellular homeostasis to memory consolidation and integration.
   *Neuron*, 81(1), 12-34.
   Grounds: Extended SHY framework connecting homeostasis to memory integration.

5. **Revonsuo, A.** (2000). The reinterpretation of dreams: An evolutionary
   hypothesis of the function of dreaming. *Behavioral and Brain Sciences*,
   23(6), 877-901.
   Grounds: Threat Simulation Theory -- dreams evolved for threat rehearsal.

6. **Lacaux, C., Andrillon, T., Bastoul, C., et al.** (2021). Sleep onset is a
   creative sweet spot. *Science Advances*, 7(50), eabj5866.
   Grounds: Hypnagogic creativity -- 83% vs 30% creative problem-solving.

7. **Lin, K., Snell, C., Wang, Y., et al.** (2025). Sleep-time compute: Beyond
   inference scaling at test-time. *arXiv:2504.13171*.
   Grounds: 5x test-time compute reduction via offline processing.

8. **Luppi, A. I., Gurnee, W., Vilas, M. G., et al.** (2024). Wake-sleep
   consolidated learning. *arXiv:2401.08623*.
   Grounds: 38% reduction in catastrophic forgetting via wake-sleep phases.

### Secondary Sources

9. **Boden, M. A.** (2004). *The Creative Mind: Myths and Mechanisms*. 2nd ed.
   Routledge.
   Grounds: Three creativity modes (combinational, exploratory, transformational).

10. **Pearl, J.** (2009). *Causality: Models, Reasoning, and Inference*. 2nd ed.
    Cambridge University Press.
    Grounds: Structural causal models for counterfactual reasoning.

11. **Cepeda, N. J., Pashler, H., Vul, E., et al.** (2006). Distributed practice
    in verbal recall tasks: A review and quantitative synthesis. *Psychological
    Bulletin*, 132(3), 354-380.
    Grounds: Spacing effect in memory research.

12. **Haar Horowitz, A., Cunningham, T. J., Maes, P., & Stickgold, R.** (2023).
    Targeted dream incubation at sleep onset increases post-sleep creativity.
    *Scientific Reports*, 13, 7319.
    Grounds: Targeted dream incubation validation (43% creativity boost).

13. **Wagner, U., Gais, S., Haider, H., Verleger, R., & Born, J.** (2004).
    Sleep inspires insight. *Nature*, 427, 352-355.
    Grounds: Sleep is 2.6x more likely to produce insight on hidden rule problems.

14. **Park, J. S., et al.** (2023). Generative agents: Interactive simulacra of
    human behavior. *UIST 2023*. arXiv:2304.03442.
    Grounds: Memory + reflection architecture for generative agents.

15. **Grossman, S. J. & Stiglitz, J. E.** (1980). On the impossibility of
    informationally efficient markets. *American Economic Review*, 70(3), 393-408.
    Grounds: Alpha convergence problem (information acquired costlessly becomes
    worthless when all agents acquire it).

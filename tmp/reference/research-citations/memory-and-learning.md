# Memory and Learning — Consolidation, Dreams, Online Learning Citations

Part of the [Research Citations](README.md) collection.

---

## Dream Consolidation and Offline Learning

During biological sleep, the brain does not idle — it actively consolidates experiences from the day into long-term memories, prunes synaptic connections, generates creative recombinations, and rehearses learned sequences. The brain dedicates 25-33% of its runtime to this offline state. Computationally, this is a form of offline learning that occurs when the system has no external obligations. Roko agents enter a "Delta" consolidation phase during idle periods with three phases: NREM replay (prioritized experience replay), REM imagination (counterfactual hypothesis generation using a world model), and integration staging (tier promotion). The convergence between biological sleep research and ML offline learning is not coincidental — both face the same optimization problem of extracting generalizable knowledge from sparse, sequential experience without catastrophic interference.

**Documents using this section**: `tmp/02-dream-consolidation.md`, `tmp/13-cognitive-architecture.md`

---

### Replay and Prioritization

**Mattar, M.G. & Daw, N.D. (2018). Prioritized Memory Access Explains Planning and Hippocampal Replay. _Nature Neuroscience_, 21(11), 1609-1617.**
[DOI: 10.1038/s41593-018-0232-z](https://doi.org/10.1038/s41593-018-0232-z)

- **Concept**: Utility = gain × need for replay selection. Episodes are replayed in order of their utility for future decisions, not recency or reward magnitude. Unifies planning, learning, and consolidation as different consequences of prioritized replay.
- **Roko adaptation**: Foundational algorithm for selecting which episodes to replay during Delta consolidation. Dream cycle replays episodes ordered by prediction error magnitude.
- **Crate**: `roko-dreams` (replay module)
- **Roko source**: [`https://github.com/wpank/roko/blob/main/crates/roko-dreams/README.md`](https://github.com/wpank/roko/blob/main/crates/roko-dreams/README.md) (line 18), [`https://github.com/wpank/roko/blob/main/docs/v2/26-CROSS-CUTS.md`](https://github.com/wpank/roko/blob/main/docs/v2/26-CROSS-CUTS.md) (lines 413, 432, 819)
- **tmp/ cross-refs**: `02-dream-consolidation.md`, `13-cognitive-architecture.md`
- **IronClaw relevance**: IronClaw's heartbeat system runs periodic background cycles. Mattar-Daw prioritization would optimize which workspace memories are worth replaying during these cycles.

---

**Wilson, M.A. & McNaughton, B.L. (1994). Reactivation of Hippocampal Ensemble Memories During Sleep. _Science_, 265(5172), 676-679.**
[DOI: 10.1126/science.8036517](https://doi.org/10.1126/science.8036517)

- **Concept**: First demonstration that hippocampal place cells reactivate during sleep in the same sequential order experienced during waking behavior. Foundation for computational replay.
- **Roko adaptation**: Directly implements prioritized experience replay in the Dreams subsystem (NREM phase).
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/21-references/01-memory-consolidation.md`](https://github.com/wpank/roko/blob/main/docs/v1/21-references/01-memory-consolidation.md)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

**Schaul, T., Quan, J., Antonoglou, I., & Silver, D. (2016). Prioritized Experience Replay. _International Conference on Learning Representations (ICLR)_.**
[arXiv:1511.05952](https://arxiv.org/abs/1511.05952)

- **Concept**: Priority proportional to TD error magnitude. Prioritizing important transitions leads to more efficient learning, outperforming uniform replay on 41 out of 49 Atari games.
- **Roko adaptation**: Surprise-weighted replay candidate selection in the dream consolidation engine.
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/21-references/01-memory-consolidation.md`](https://github.com/wpank/roko/blob/main/docs/v1/21-references/01-memory-consolidation.md)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

**Jensen, K.T., Hennequin, G., & Mattar, M.G. (2024). A Recurrent Network Model of Planning Explains Hippocampal Replay and Human Behavior. _Nature Neuroscience_, 27, 1340-1348.**
[DOI: 10.1038/s41593-024-01675-7](https://doi.org/10.1038/s41593-024-01675-7)

- **Concept**: Recurrent network model unifying planning and replay under a single computational framework. Hippocampal replay emerges from goal-directed planning computations.
- **Roko adaptation**: Validates that replay and planning share computational substrate; the dream cycle serves both functions simultaneously.
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/10-dreams/02-nrem-replay.md`](https://github.com/wpank/roko/blob/main/docs/v1/10-dreams/02-nrem-replay.md) (line 1059)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

### Emotional Depotentiation

**Walker, M.P. & van der Helm, E. (2009). Overnight Therapy: The Role of Sleep in Emotional Brain Processing. _Psychological Bulletin_, 135(5), 731-748.**
[DOI: 10.1037/a0016570](https://doi.org/10.1037/a0016570)

- **Concept**: REM sleep depotentiates the emotional charge of memories while preserving their informational content. The phrase "overnight therapy" captures the finding that traumatic memories are processed during sleep without re-traumatization.
- **Roko adaptation**: Dream cycles reduce arousal on highly charged memories by 0.3-0.5 per cycle to prevent panic lock-in — a REM-phase operation.
- **Crate**: `roko-dreams`, `roko-daimon`
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v2/26-CROSS-CUTS.md`](https://github.com/wpank/roko/blob/main/docs/v2/26-CROSS-CUTS.md) (lines 455, 496), [`https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/18-affect-as-functor.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/07-agent-runtime/18-affect-as-functor.md) (line 954)
- **tmp/ cross-refs**: `02-dream-consolidation.md`, `03-affect-engine.md`

---

### Creativity and Counterfactuals

**Boden, M.A. (2004). _The Creative Mind: Myths and Mechanisms_. 2nd ed. London: Routledge. ISBN: 978-0415314534.**

- **Concept**: Three creativity modes: exploratory (moving within a conceptual space), combinational (making unfamiliar combinations of familiar ideas), transformational (changing the rules to generate previously impossible structures).
- **Roko adaptation**: REM phase implements combinational creativity via HDC recombination and transformational creativity via causal model intervention.
- **Crate**: `roko-dreams` (imagination module)
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/21-references/03-dreams-and-offline-learning.md`](https://github.com/wpank/roko/blob/main/docs/v1/21-references/03-dreams-and-offline-learning.md)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

**Pearl, J. (2009). _Causality: Models, Reasoning, and Inference_. 2nd ed. Cambridge University Press. ISBN: 978-0521895606.**

- **Concept**: Structural causal models enable counterfactual reasoning via the do-calculus. The intervention do(X=x) computes the distribution of Y after setting X to x, regardless of its natural causes. Three levels of causal hierarchy: association, intervention, counterfactual.
- **Roko adaptation**: REM dreaming generates counterfactuals by intervening on causal variables: "what would have happened if I had chosen differently?"
- **Crate**: `roko-dreams`
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/21-references/03-dreams-and-offline-learning.md`](https://github.com/wpank/roko/blob/main/docs/v1/21-references/03-dreams-and-offline-learning.md), [`https://github.com/wpank/roko/blob/main/docs/v2/26-CROSS-CUTS.md`](https://github.com/wpank/roko/blob/main/docs/v2/26-CROSS-CUTS.md) (line 455)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

### Sleep-Time Compute

**Lin, K., Snell, C., Wang, Y., Packer, C., Wooders, S., Stoica, I., & Gonzalez, J.E. (2025). Sleep-time Compute: Beyond Inference Scaling at Test-Time.**
[arXiv:2504.13171](https://arxiv.org/abs/2504.13171)

- **Concept**: Dual-agent architecture: a Sleeper Agent precomputes during downtime (answering anticipated questions, updating knowledge summaries), while a Serve Agent handles live interactions with pre-computed answers. Achieves approximately 5x test-time compute reduction on Stateful GSM-Symbolic benchmarks.
- **Roko adaptation**: Dream cycles are sleep-time compute — agents process experiences during low-activity periods to reduce latency and improve accuracy at inference time.
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/21-references/03-dreams-and-offline-learning.md`](https://github.com/wpank/roko/blob/main/docs/v1/21-references/03-dreams-and-offline-learning.md)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

**Ha, D. & Schmidhuber, J. (2018). World Models.**
[arXiv:1803.10122](https://arxiv.org/abs/1803.10122)

- **Concept**: A controller trained entirely inside a compressed representation of experience ("dreams") in a recurrent world model achieves competitive performance on RL benchmarks without interacting with the real environment during training.
- **Roko adaptation**: Dreaming multiplies learning from scarce experience. Idle-time knowledge recombination creates new hypotheses without real execution.
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/21-references/03-dreams-and-offline-learning.md`](https://github.com/wpank/roko/blob/main/docs/v1/21-references/03-dreams-and-offline-learning.md)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

**Hafner, D., Pasukonis, J., Ba, J., & Lillicrap, T. (2025). Mastering Diverse Control Tasks Through World Models. _Nature_, 640, 647-653.**
[DOI: 10.1038/s41586-025-08744-2](https://doi.org/10.1038/s41586-025-08744-2)

- **Concept**: DreamerV3: agents trained on imagined trajectories from a world model outperform specialized methods across 150+ tasks spanning domains from robotics to games. The first algorithm to collect diamonds in Minecraft from scratch without human data or dense rewards.
- **Roko adaptation**: REM-phase consolidation generates synthetic scenarios from the causal model, validating the dream-based learning approach at scale.
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/10-dreams/03-rem-imagination.md`](https://github.com/wpank/roko/blob/main/docs/v1/10-dreams/03-rem-imagination.md) (line 1123)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

### Hypnagogia

**Lacaux, C., Andrillon, T., Bastoul, C., Idir, Y., Fonteix-Galet, A., Arnulf, I., & Oudiette, D. (2021). Sleep Onset is a Creative Sweet Spot. _Science Advances_, 7(50), eabj5866.**
[DOI: 10.1126/sciadv.abj5866](https://doi.org/10.1126/sciadv.abj5866)

- **Concept**: 83% of subjects spending at least 15 seconds in N1 (sleep onset / hypnagogia) discovered hidden mathematical rules versus 30% staying awake — a 2.8x creative advantage. The effect vanishes completely in deeper sleep.
- **Roko adaptation**: Foundational for the hypnagogia engine, which solves the Alpha Convergence Problem (all agents converging to similar solutions through homogenized training).
- **Crate**: `roko-dreams` (hypnagogia module)
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/21-references/03-dreams-and-offline-learning.md`](https://github.com/wpank/roko/blob/main/docs/v1/21-references/03-dreams-and-offline-learning.md)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

**Horowitz, A.H., Cunningham, T.J., Mauss, I.B., & Stickgold, R. (2023). Targeted Dream Incubation at Sleep Onset Increases Post-Sleep Creative Problem Solving. _Scientific Reports_, 13, 5669.**
[DOI: 10.1038/s41598-023-31361-w](https://doi.org/10.1038/s41598-023-31361-w)

- **Concept**: Targeted dream incubation (TDI) at sleep onset — presenting problem-specific stimuli during N1 sleep — enhances post-sleep creative performance specifically on the incubated topics.
- **Roko adaptation**: Validates the targeted consolidation approach in the hypnagogia engine. High-priority tasks can be "incubated" during Delta transitions.
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/10-dreams/07-hypnagogia-engine.md`](https://github.com/wpank/roko/blob/main/docs/v1/10-dreams/07-hypnagogia-engine.md) (line 864)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

### Sleep-Inspired LLM Research (2025)

**NeuroDream (2025). SSRN:5377250.**

- **Concept**: Dream-phase consolidation in LLM agents achieves 38% forgetting reduction and 17.6% zero-shot transfer increase compared to agents without consolidation.
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/21-references/24-additions-2025.md`](https://github.com/wpank/roko/blob/main/docs/v1/21-references/24-additions-2025.md)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

**SleepGate (2025).**
[arXiv:2603.14517](https://arxiv.org/abs/2603.14517)

- **Concept**: Active forgetting via learned curation resolves proactive interference in LLM agents. Validates that strategic forgetting is a feature, not a bug.
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/21-references/24-additions-2025.md`](https://github.com/wpank/roko/blob/main/docs/v1/21-references/24-additions-2025.md)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

**LightMem (2025).**
[arXiv:2510.18866](https://arxiv.org/abs/2510.18866)

- **Concept**: Offline consolidation achieving 10.9% accuracy gain with 117x token reduction through selective memory compression.
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/21-references/24-additions-2025.md`](https://github.com/wpank/roko/blob/main/docs/v1/21-references/24-additions-2025.md)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

**Sawada, T., Gowrishankar, N., Bhatt, J., & Kim, J. (2024). Prefrontal Synaptic Regulation of Homeostatic Sleep Pressure. _Science_, 386(6721).**
[DOI: 10.1126/science.adl3043](https://doi.org/10.1126/science.adl3043)

- **Concept**: Prefrontal cortex synaptic mechanisms directly regulate homeostatic sleep pressure through SIK3-mediated synaptic downscaling.
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/10-dreams/04-consolidation-and-staging.md`](https://github.com/wpank/roko/blob/main/docs/v1/10-dreams/04-consolidation-and-staging.md) (line 286)

---

## Memory Consolidation and Forgetting

The neuroscience of memory consolidation reveals that learning is not a one-time event but a multi-stage process where experiences are initially stored in labile form, then gradually stabilized through offline processing. Critically, forgetting is not merely degradation but an active computational process that serves generalization. The distinction between episodic memory (individual events) and semantic memory (general knowledge) maps directly to the engineering distinction between logs and summaries. IronClaw's principle "LLM data is never deleted" reflects this — instead of deletion, data should be filtered, weighted, and aging should be tracked.

**Documents using this section**: `tmp/10-universal-engram.md`, `tmp/02-dream-consolidation.md`

---

**McClelland, J.L., McNaughton, B.L., & O'Reilly, R.C. (1995). Why There Are Complementary Learning Systems in the Hippocampus and Neocortex: Insights from the Successes and Failures of Connectionist Models of Learning and Memory. _Psychological Review_, 102(3), 419-457.**
[DOI: 10.1037/0033-295X.102.3.419](https://doi.org/10.1037/0033-295X.102.3.419)

- **Concept**: Complementary Learning Systems (CLS) theory: fast episodic memory (hippocampus) consolidates into slow semantic memory (neocortex) during offline periods. Two systems are necessary because fast learning causes catastrophic interference in distributed representations.
- **Roko adaptation**: NeuroStore's dual-store architecture. Episodic entries consolidate into insights and heuristics during Delta cycles. Tier progression: Working -> Consolidated -> Persistent.
- **Crate**: `roko-neuro`
- **tmp/ cross-refs**: `10-universal-engram.md`, `02-dream-consolidation.md`

---

**Richards, B.A. & Frankland, P.W. (2017). The Persistence and Transience of Memory. _Neuron_, 94(6), 1071-1084.**
[DOI: 10.1016/j.neuron.2017.04.037](https://doi.org/10.1016/j.neuron.2017.04.037)

- **Concept**: Active forgetting serves computational functions: neurogenesis eliminates outdated information, prevents overfitting, and enables generalization. Forgetting is mathematically equivalent to L1 regularization.
- **Roko adaptation**: Foundational for the entire demurrage architecture. Knowledge that is not actively retrieved decays on per-type schedules.
- **tmp/ cross-refs**: `10-universal-engram.md`

---

**Nader, K., Schafe, G.E., & LeDoux, J.E. (2000). Fear Memories Require Protein Synthesis in the Amygdala for Reconsolidation After Retrieval. _Nature_, 406, 722-726.**
[DOI: 10.1038/35021052](https://doi.org/10.1038/35021052)

- **Concept**: Retrieved memories become labile and can be updated (reconsolidation). Memory is not a static recording but a reconstructive, updatable process.
- **Roko adaptation**: Confidence-update-on-retrieval mechanism. When a knowledge entry is retrieved, its confidence can be updated based on current context.
- **tmp/ cross-refs**: `10-universal-engram.md`

---

**Roediger, H.L. & Karpicke, J.D. (2006). Test-Enhanced Learning: Taking Memory Tests Improves Long-Term Retention. _Psychological Science_, 17(3), 249-255.**
[DOI: 10.1111/j.1467-9280.2006.01693.x](https://doi.org/10.1111/j.1467-9280.2006.01693.x)

- **Concept**: Retrieval strengthens memory traces more than re-study (+200% recall vs passive review). The testing effect: retrieval is more beneficial than restudying.
- **Roko adaptation**: Retrieved entries decay slower. Strength-increment-on-positive-outcome mechanism.
- **tmp/ cross-refs**: `10-universal-engram.md`

---

**Tononi, G. & Cirelli, C. (2014). Sleep and the Price of Plasticity: From Synaptic and Cellular Homeostasis to Memory Consolidation and Integration. _Neuron_, 81(1), 12-34.**
[DOI: 10.1016/j.neuron.2013.12.025](https://doi.org/10.1016/j.neuron.2013.12.025)

- **Concept**: Synaptic Homeostasis Hypothesis (SHY): net synaptic weight increases during waking learning and must decrease during sleep through active downscaling of weak connections.
- **Roko adaptation**: Knowledge decay during idle periods. The system actively prunes weak knowledge entries during Delta cycles.
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

**Park, J.S., O'Brien, J.C., Cai, C.J., Morris, M.R., Liang, P., & Bernstein, M.S. (2023). Generative Agents: Interactive Simulacra of Human Behavior. _Proceedings of the ACM Symposium on User Interface Software and Technology (UIST)_.**
[arXiv:2304.03442](https://arxiv.org/abs/2304.03442)

- **Concept**: Four-factor retrieval (recency, importance, relevance, emotional congruence) produces emergent social behaviors in simulated human agents. Ablating any single factor causes behavioral degeneration.
- **Roko adaptation**: Roko extends to four factors with ablation studies validating each dimension.
- **tmp/ cross-refs**: `10-universal-engram.md`, `17-agent-patterns.md`

---

## Ebbinghaus Forgetting Curve

Hermann Ebbinghaus conducted the first experimental study of human memory in 1885, learning lists of nonsense syllables and measuring retention at varying delays. His quantitative findings established the forgetting curve (negative exponential decay), the spacing effect (distributed practice outperforms massed practice), and the testing effect (retrieval strengthens memory). These findings have been replicated hundreds of times and form the mathematical basis for modern spaced repetition systems.

**IronClaw relevance**: IronClaw's workspace memory could adopt Ebbinghaus-style decay to automatically age out stale knowledge entries, with retrieval strengthening the decay time constant.

---

**Ebbinghaus, H. (1885). _Uber das Gedachtnis: Untersuchungen zur experimentellen Psychologie_ [Memory: A Contribution to Experimental Psychology]. Leipzig: Duncker & Humblot. Translated by Ruger, H.A. & Bussenius, C.E. (1913). New York: Teachers College, Columbia University.**

- **Concept**: The forgetting curve follows negative exponential decay: `R = e^(-t/S)`. Retrieval strengthens the trace and slows subsequent decay. Spaced repetition optimizes retention.
- **Roko adaptation**: Directly implemented as the `Decay::Ebbinghaus` variant: `weight = exp(-age / (strength * scale_ms))`. Per-type half-lives: Episodes 48h, Insights 7d, Heuristics 14d, Warnings 30d.
- **Crate**: `roko-core` (`decay.rs`)
- **Roko source**: [`https://github.com/wpank/roko/blob/main/crates/roko-core/src/decay.rs`](https://github.com/wpank/roko/blob/main/crates/roko-core/src/decay.rs) (line 59), [`https://github.com/wpank/roko/blob/main/docs/v2/01-SIGNAL.md`](https://github.com/wpank/roko/blob/main/docs/v2/01-SIGNAL.md) (line 671)
- **tmp/ cross-refs**: `10-universal-engram.md`, `11-mathematical-primitives.md`

---

## Online Learning and Model Routing

The online learning problem of selecting the best action from multiple alternatives based on contextual features is formalized as the contextual bandit problem. In agent systems, this maps to model routing: given a task description (context), which LLM model (arm) produces the best output per token-dollar? The Roko CascadeRouter transitions through three stages: Static (fewer than 50 observations), Confidence (50-200), and UCB/LinUCB (over 200), with the v2 spec replacing LinUCB with EFE-based routing while retaining the bandit infrastructure as a learning stage.

**Documents using this section**: `tmp/07-online-learning.md`

---

**Li, L., Chu, W., Langford, J., & Schapire, R.E. (2010). A Contextual-Bandit Approach to Personalized News Article Recommendation. _Proceedings of the 19th International Conference on World Wide Web (WWW)_, pp. 661-670.**
[DOI: 10.1145/1772690.1772758](https://doi.org/10.1145/1772690.1772758)

- **Concept**: LinUCB contextual bandit: learns linear feature weights mapping context to expected reward per arm. Provides theoretical guarantees on cumulative regret with an 18-dimensional context vector.
- **Roko adaptation**: Stage 3 of CascadeRouter uses LinUCB with 18-dimensional feature vector built from task complexity, domain familiarity, token budget, model latency, and historical pass rates.
- **Crate**: `roko-learn` (`model_router.rs`)
- **tmp/ cross-refs**: `07-online-learning.md`

---

**Thompson, W.R. (1933). On the Likelihood that One Unknown Probability Exceeds Another in View of the Evidence of Two Samples. _Biometrika_, 25(3-4), 285-294.**
[DOI: 10.2307/2332286](https://doi.org/10.2307/2332286)

- **Concept**: Thompson Sampling: maintain a Beta distribution per arm and sample to select. Naturally balances exploration and exploitation without explicit UCB bonus computation.
- **Roko adaptation**: Discounted Thompson Sampling with drift detection. Beta distributions decay by gamma factor to handle non-stationary environments.
- **Crate**: `roko-learn`
- **tmp/ cross-refs**: `07-online-learning.md`

---

**Garivier, A. & Moulines, E. (2011). On Upper-Confidence Bound Policies for Switching Bandit Problems. _International Conference on Algorithmic Learning Theory (ALT)_, LNCS 6925, pp. 174-188.**
[DOI: 10.1007/978-3-642-24412-4_16](https://doi.org/10.1007/978-3-642-24412-4_16)

- **Concept**: Discounted UCB for non-stationary bandits with a forgetting mechanism for old observations. Discount factor gamma creates an effective observation window of 1/(1-gamma) samples.
- **Roko adaptation**: Discount factor gamma ensures old model performance data fades exponentially, preventing stale routing decisions when models are updated.
- **tmp/ cross-refs**: `07-online-learning.md`

---

**Chen, L., Zaharia, M., & Zou, J. (2023). FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance.**
[arXiv:2305.05176](https://arxiv.org/abs/2305.05176)

- **Concept**: 98% cost reduction via intelligent model routing using cascade from cheap to expensive models.
- **Roko adaptation**: T0/T1/T2 cascade architecture.
- **tmp/ cross-refs**: `07-online-learning.md`, `13-cognitive-architecture.md`

---

**Ong, I., Almahairi, A., & Manning, C.D. (2024). RouteLLM: Learning to Route LLMs with Preference Data.**
[arXiv:2406.18665](https://arxiv.org/abs/2406.18665)

- **Concept**: Preference-based routing between models using learned routing functions trained on human preference data.
- **Roko adaptation**: CascadeRouter training uses preference-based learning from gate verdicts (gate pass = positive preference signal).
- **tmp/ cross-refs**: `07-online-learning.md`

---

## Reinforcement Learning Foundations

**Andrychowicz, M., Wolski, F., Ray, A., Schneider, J., Fong, R., Welter, L., McGrew, B., Tobin, J., Abbeel, P., & Zaremba, W. (2017). Hindsight Experience Replay. _Advances in Neural Information Processing Systems (NeurIPS)_, 30.**
[arXiv:1707.01495](https://arxiv.org/abs/1707.01495)

- **Concept**: HER: re-label failed trajectories with the goals they actually achieved, converting failures into successful experiences for different goals. Enables learning in sparse-reward settings.
- **Roko adaptation**: Phase 2 of dream consolidation (Hindsight Relabeling). Failed trajectories decomposed into sub-goals; achieved sub-goals relabeled as positive episodes. Recovers learning from at least 45% of otherwise-discarded episodes.
- **tmp/ cross-refs**: `02-dream-consolidation.md`, `07-online-learning.md`

---

**Rescorla, R.A. & Wagner, A.R. (1972). A Theory of Pavlovian Conditioning: Variations in the Effectiveness of Reinforcement and Nonreinforcement. In _Classical Conditioning II: Current Research and Theory_, pp. 64-99. New York: Appleton-Century-Crofts.**

- **Concept**: Learning is driven by the discrepancy between expected and actual outcomes (prediction error). The delta rule: `delta_V = alpha * beta * (lambda - V)`.
- **Roko adaptation**: The simplest form of prediction error that drives T0 probes and CalibrationTracker updates.

---

**Doya, K. (2002). Metalearning and Neuromodulation. _Neural Networks_, 15(4-6), 495-506.**
[DOI: 10.1016/S0893-6080(02)00044-8](https://doi.org/10.1016/S0893-6080(02)00044-8)

- **Concept**: Different neuromodulators control different meta-parameters of learning: dopamine (learning rate/reward), serotonin (time horizon), norepinephrine (exploration rate), acetylcholine (memory consolidation speed).
- **Roko adaptation**: Maps to the 7-axis Score where different axes control different aspects of knowledge management and learning dynamics.

---

## Self-Learning Systems

The ability to improve from experience — without human intervention and beyond the initial training distribution — is the defining challenge of artificial general intelligence.

**Documents using this section**: `tmp/07-online-learning.md`, `tmp/17-agent-patterns.md`

---

**Shinn, N., Cassano, F., Labash, A., Gopalan, A., Narasimhan, K., & Yao, S. (2023). Reflexion: Language Agents with Verbal Reinforcement Learning. _Advances in Neural Information Processing Systems (NeurIPS)_, 36.**
[arXiv:2303.11366](https://arxiv.org/abs/2303.11366)

- **Concept**: Verbal reinforcement learning via stored self-reflection. Post-episode reflection stored as persistent context for subsequent attempts. +22% on AlfWorld, +20% on HotPotQA.
- **Roko adaptation**: Theta-frequency reflection cycle. Agent periodically reflects on recent work and stores findings in NeuroStore.
- **tmp/ cross-refs**: `07-online-learning.md`, `17-agent-patterns.md`

---

**Zhao, A., Huang, D., Xu, Q., Lin, M., Liu, Y.-J., & Huang, G. (2024). ExpeL: LLM Agents Are Experiential Learners.**
[arXiv:2308.10144](https://arxiv.org/abs/2308.10144)

- **Concept**: Cross-task experience extraction. Agents mine their trajectory histories for generalizable insights that transfer across task types.
- **Roko adaptation**: Insights evolve across task sessions in the NeuroStore, enabling double-loop learning.
- **tmp/ cross-refs**: `07-online-learning.md`

---

**Wang, G., Xie, Y., Jiang, Y., Mandlekar, A., Xiao, C., Zhu, Y., Fan, L., & Anandkumar, A. (2023). Voyager: An Open-Ended Embodied Agent with Large Language Models.**
[arXiv:2305.16291](https://arxiv.org/abs/2305.16291)

- **Concept**: Code-as-action skill library. Agents write and store executable code as reusable procedural skills. 3.3x more unique behaviors versus baselines.
- **Roko adaptation**: EvoSkills: self-evolving skill library where agents compose reusable procedural skills stored as Engrams.
- **tmp/ cross-refs**: `17-agent-patterns.md`

---

**SAMULE (2025). Self-Learning Agents Enhanced by Multi-level Reflection. _Proceedings of the Conference on Empirical Methods in Natural Language Processing (EMNLP)_.**
[arXiv:2509.20562](https://arxiv.org/abs/2509.20562)

- **Concept**: Multi-level reflection across trajectories substantially outperforms single-trajectory reflection. Error classification and clustering extract actionable insights from failure patterns.
- **Roko adaptation**: Validates Theta-frequency reflection operating across episodes, not just within single task runs.

---

*Navigation: [README](README.md) | [HDC and VSA](hdc-and-vsa.md) | [Affect and Cognition](affect-and-cognition.md) | [Verification and Safety](verification-and-safety.md) | [Agents and Orchestration](agents-and-orchestration.md) | [Blockchain and Economics](blockchain-and-economics.md) | [Context and Search](context-and-search.md) | [Math and Statistics](math-and-statistics.md)*

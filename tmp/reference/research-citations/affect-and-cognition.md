# Affect and Cognition — Neuroscience, Architecture, and Emotion Citations

Part of the [Research Citations](README.md) collection.

---

## Affect, Emotion, and Somatic Markers

Affective computing studies how emotions influence cognition, decision-making, and social behavior — and how artificial systems can represent and use emotional states. Five independent research lines converge on the conclusion that emotion-like states serve genuine computational functions, not decorative ones. Damasio's lesion studies prove that emotion is necessary for rational decision-making. Kahneman's dual-process theory shows that affect is System 1's decision substrate. Zhang et al. (2024) demonstrate empirically that emotional state changes approximately 50% of agent decisions. The Daimon implements a PAD (Pleasure-Arousal-Dominance) vector with the ALMA three-layer temporal model (emotion/mood/personality) and a somatic landscape (k-d tree over strategy space) to bias every decision the agent makes.

**Documents using this section**: `tmp/03-affect-engine.md`, `tmp/13-cognitive-architecture.md`

---

### PAD Model

**Mehrabian, A. (1996). Pleasure, Arousal, Dominance: A General Framework for Describing and Measuring Individual Differences in Temperament. _Current Psychology_, 14(4), 261-292.**
[DOI: 10.1007/BF02686918](https://doi.org/10.1007/BF02686918)

- **Concept**: Three continuous dimensions (Pleasure, Arousal, Dominance) capture more emotional variance than discrete emotion labels. The 8 PAD octants map to behavioral states.
- **Roko adaptation**: Daimon state vector. Every Signal carries an optional PAD stamp. The 8 octants map to: Exuberant, Dependent, Relaxed, Docile, Hostile, Anxious, Disdainful, Depressed.
- **Crate**: `roko-daimon`, `roko-core`
- **Roko source**: `docs/v2/26-CROSS-CUTS.md` (line 226), `docs/v2-depth/07-agent-runtime/18-affect-as-functor.md`, `docs/v2/05-AGENT.md` (line 969)
- **tmp/ cross-refs**: `03-affect-engine.md`, `13-cognitive-architecture.md`

---

**Russell, J.A. & Mehrabian, A. (1977). Evidence for a Three-Factor Theory of Emotions. _Journal of Research in Personality_, 11(3), 273-294.**
[DOI: 10.1016/0092-6566(77)90037-X](https://doi.org/10.1016/0092-6566(77)90037-X)

- **Concept**: Empirical validation of the three-dimensional affect model through factor analysis of emotional self-reports across multiple populations and cultures.
- **Roko source**: `docs/v1/21-references/02-affective-computing.md`
- **tmp/ cross-refs**: `03-affect-engine.md`

---

### Somatic Markers

**Damasio, A.R. (1994). _Descartes' Error: Emotion, Reason, and the Human Brain_. New York: Putnam. ISBN: 978-0380726479.**

- **Concept**: Somatic markers: patients without emotional capacity (vmPFC damage) make consistently worse decisions under uncertainty despite intact reasoning. Emotional signals from body-mapped memories guide decision-making before deliberation.
- **Roko adaptation**: SomaticLandscape implemented as a k-d tree over 8-dimensional strategy space. Past emotional experiences bias future decisions before analytical reasoning begins.
- **Crate**: `roko-daimon` (`somatic_ta.rs`), `roko-core`
- **Roko source**: `crates/roko-daimon/src/somatic_ta.rs` (line 15), `docs/v2/05-AGENT.md` (line 969)
- **tmp/ cross-refs**: `03-affect-engine.md`, `13-cognitive-architecture.md`

---

**Bechara, A., Damasio, H., & Damasio, A.R. (2000). Emotion, Decision Making and the Orbitofrontal Cortex. _Cerebral Cortex_, 10(3), 295-307.**
[DOI: 10.1093/cercor/10.3.295](https://doi.org/10.1093/cercor/10.3.295)

- **Concept**: Anticipatory skin conductance responses precede conscious awareness in the Iowa Gambling Task. Normal participants develop gut feelings about deck riskiness before they consciously understand the card structure — pre-cognitive affect guides decisions.
- **Roko adaptation**: SomaticLandscape provides fast heuristic feelings about strategy regions before analytical reasoning, matching the biological pre-cognitive affect mechanism.
- **Crate**: `roko-daimon`
- **tmp/ cross-refs**: `03-affect-engine.md`

---

### OCC and Scherer Appraisal

**Ortony, A., Clore, G.L., & Collins, A. (1988). _The Cognitive Structure of Emotions_. Cambridge: Cambridge University Press. ISBN: 978-0521386814.**

- **Concept**: OCC emotion model: cognitive appraisal-based emotion taxonomy organizing 22 basic emotions by their cognitive antecedent conditions. Emotions arise from evaluating events against goals (desirability), standards (praiseworthiness), and attitudes (appealingness).
- **Roko adaptation**: Complements PAD with structural-of-emotion framework. The 8-step appraisal pipeline is OCC-Scherer.
- **Crate**: `roko-daimon`
- **Roko source**: `docs/v1/21-references/02-affective-computing.md`, `docs/v2-depth/07-agent-runtime/18-affect-as-functor.md` (line 126)
- **tmp/ cross-refs**: `03-affect-engine.md`

---

**Scherer, K.R. (2001). Appraisal Considered as a Process of Multi-Level Sequential Checking. In K.R. Scherer, A. Schorr, & T. Johnstone (Eds.), _Appraisal Processes in Emotion: Theory, Methods, Research_, pp. 92-120. Oxford: Oxford University Press. ISBN: 978-0195130072.**

- **Concept**: Multi-level sequential checking through four appraisal criteria: relevance, implication, coping potential, normative significance.
- **Roko adaptation**: The 8-step appraisal pipeline (Classify, Ground, Scale, Compute, Decay, Apply, Persist, Emit) is OCC-Scherer.
- **Crate**: `roko-daimon`
- **tmp/ cross-refs**: `03-affect-engine.md`

---

### ALMA Temporal Model

**Gebhard, P. (2005). ALMA — A Layered Model of Affect. _Proceedings of the 4th International Joint Conference on Autonomous Agents and Multiagent Systems (AAMAS)_, pp. 29-36.**
[DOI: 10.1145/1082473.1082478](https://doi.org/10.1145/1082473.1082478)

- **Concept**: Three-layer temporal affect: emotion (seconds), mood (hours), personality (lifetime). Each layer operates at different timescale with different time constants, preventing short-term events from corrupting personality baselines.
- **Roko adaptation**: Three nested loops. Emotion: tau=0.1. Mood: tau=0.5, fires every 10 ticks. Personality: tau=0.9, fires every 100 ticks. Effective PAD = 0.5*emotion + 0.3*mood + 0.2*personality.
- **Crate**: `roko-daimon`
- **Roko source**: `docs/v2-depth/07-agent-runtime/18-affect-as-functor.md` (line 480)
- **tmp/ cross-refs**: `03-affect-engine.md`

---

### Mood-Congruent Memory

**Bower, G.H. (1981). Mood and Memory. _American Psychologist_, 36(2), 129-148.**
[DOI: 10.1037/0003-066X.36.2.129](https://doi.org/10.1037/0003-066X.36.2.129)

- **Concept**: Emotional states bias memory retrieval via associative network activation. Happy moods preferentially retrieve positive memories; the effect is stronger for autobiographical than semantic memories.
- **Roko adaptation**: Emotional factor (0.15 weight) in four-factor retrieval scoring. Mandatory 15% contrarian retrieval prevents echo chambers.
- **Crate**: `roko-daimon`, `roko-neuro`
- **Roko source**: `crates/roko-daimon/src/somatic_ta.rs` (line 20)
- **tmp/ cross-refs**: `03-affect-engine.md`

---

### Affect Validation

**Zhang, Y., Shi, Z., Wang, X., & Zhang, M. (2024). Self-Emotion Changes ~50% of Decisions. _Proceedings of SIGDIAL 2024_.**

- **Concept**: Empirical measurement in multi-turn agent dialogue showing affect is the primary driver of agent behavior, not a display layer. Approximately 50% of decisions changed based on emotional state.
- **Roko adaptation**: Validates that the Daimon is architectural, not decorative.
- **tmp/ cross-refs**: `03-affect-engine.md`

---

### Learned Helplessness

**Seligman, M.E.P. (1972). Learned Helplessness. _Annual Review of Medicine_, 23, 407-412.**
[DOI: 10.1146/annurev.me.23.020172.002203](https://doi.org/10.1146/annurev.me.23.020172.002203)

- **Concept**: Uncontrollable negative outcomes produce learned helplessness — resignation and passivity that persists even when control becomes available.
- **Roko adaptation**: Dominance < -0.3 for 200+ ticks triggers a behavioral state machine alert in the Daimon. Prolonged low dominance is an agent health warning.
- **Crate**: `roko-daimon`
- **tmp/ cross-refs**: `03-affect-engine.md`

---

## Prospect Theory and Loss Aversion

Kahneman and Tversky's Prospect Theory replaced Expected Utility Theory as the descriptive model of decision under risk. The key empirical finding — that losses hurt approximately twice as much as equivalent gains feel good — has been replicated across cultures, species, and domains.

---

**Kahneman, D. & Tversky, A. (1979). Prospect Theory: An Analysis of Decision under Risk. _Econometrica_, 47(2), 263-292.**
[DOI: 10.2307/1914185](https://doi.org/10.2307/1914185)

- **Concept**: Value function: concave for gains (v(x) = x^0.88), convex for losses (v(-x) = -2.25 * x^0.88). Losses hurt more than equivalent gains feel good; lambda ≈ 2.25.
- **Roko adaptation**: Daimon appraisal pipeline uses prospect theory with lambda=2.25. Gate failures produce 2x the affect delta of gate passes. `v(delta) = delta^0.88` for gains, `-2.25 * (-delta)^0.88` for losses.
- **Crate**: `roko-daimon`
- **Roko source**: `docs/v2/05-AGENT.md` (lines 985-993), `docs/v2-depth/07-agent-runtime/18-affect-as-functor.md` (lines 240, 261)
- **tmp/ cross-refs**: `03-affect-engine.md`

---

## Cognitive Architectures

Cognitive architectures are comprehensive models of the information processing structures underlying intelligent behavior. From symbolic architectures (SOAR, ACT-R) to subsumption architectures (Brooks) to hybrid systems (CLARION), each makes strong commitments about memory organization, action selection, and learning mechanisms.

**Documents using this section**: `tmp/13-cognitive-architecture.md`, `tmp/17-agent-patterns.md`

---

**Sumers, T.R., Yao, S., Narasimhan, K., & Griffiths, T.L. (2023). Cognitive Architectures for Language Agents (CoALA).**
[arXiv:2309.02427](https://arxiv.org/abs/2309.02427)

- **Concept**: 9-step cognitive pipeline for language agents establishing memory (semantic, episodic, procedural, working), action selection, and environment interaction as core components.
- **Roko adaptation**: Universal loop extends CoALA with Gate (verification) and Daimon (affect).
- **tmp/ cross-refs**: `13-cognitive-architecture.md`, `17-agent-patterns.md`

---

**Kahneman, D. (2011). _Thinking, Fast and Slow_. New York: Farrar, Straus and Giroux. ISBN: 978-0374275631.**

- **Concept**: System 1 (fast, intuitive, heuristic) / System 2 (slow, deliberate, analytical) dual-process theory of cognition.
- **Roko adaptation**: T0/T1/T2 cascade directly implements this: T0 is System 1 (under 50ms), T1 is fast System 2 (1-5 seconds), T2 is full System 2 (10-120 seconds).
- **tmp/ cross-refs**: `13-cognitive-architecture.md`, `07-online-learning.md`

---

**Anderson, J.R. (1993). _The Architecture of Cognition_. Harvard University Press. ISBN: 978-0674044258.**

- **Concept**: ACT-R: declarative/procedural memory distinction. Production rules as computational substrate for cognition. 50+ years of experimental validation.
- **Roko adaptation**: NeuroStore knowledge types map to ACT-R memory categories. Declarative = stored knowledge. Procedural = heuristics/playbook rules.
- **Roko source**: `docs/v2-depth/07-agent-runtime/dual-process-and-efe-routing.md` (line 100)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

**Laird, J.E., Newell, A., & Rosenbloom, P.S. (1987). SOAR: An Architecture for General Intelligence. _Artificial Intelligence_, 33(1), 1-64.**
[DOI: 10.1016/0004-3702(87)90050-6](https://doi.org/10.1016/0004-3702(87)90050-6)

- **Concept**: Problem solving + chunking. Propose-decide-apply-learn cycle. Impasse-driven elaboration: when the system cannot proceed, it subgoals to resolve the impasse and learns the solution as a chunk.
- **Roko adaptation**: Compose-act-verify-adapt. T0 failure triggers T1 escalation (impasse-driven). Chunk learning maps to heuristic promotion.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

**Sun, R. (2002). _Duality of the Mind: A Bottom-Up Approach to Cognition_. Mahwah, NJ: Erlbaum. ISBN: 978-0805838176. (CLARION architecture)**

- **Concept**: Dual-level architecture with explicit (rule-based, symbolic) and implicit (neural network, subsymbolic) processing. Both levels learn and interact.
- **Roko adaptation**: T0 reflex (implicit) vs T2 reasoning (explicit). NeuroStore + HDC + somatic markers provide both levels.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

**Baars, B.J. (1988). _A Cognitive Theory of Consciousness_. Cambridge: Cambridge University Press. ISBN: 978-0521427432. (Global Workspace Theory)**

- **Concept**: Consciousness as a broadcast mechanism. Multiple specialized processors compete for access to a shared global workspace; the "winner" broadcasts to all other processors.
- **Roko adaptation**: Broadcast on Bus = global workspace. CognitiveWorkspace VCG = competitive access. The context window is the "workspace" that multiple subsystems compete to fill.
- **Roko source**: `docs/v2/05-AGENT.md` (line 1625)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`, `09-budget-composition.md`

---

## Dual-Process Cognition

Dual-process theories distinguish between automatic, fast, low-effort processing (System 1 / Type 1) and controlled, slow, high-effort processing (System 2 / Type 2). In neuroscience, neural oscillation frequencies provide a complementary taxonomy: gamma oscillations (30-100 Hz) for fast attention, theta (4-8 Hz) for memory and navigation, delta (0.5-4 Hz) for deep sleep and consolidation.

**Documents using this section**: `tmp/13-cognitive-architecture.md`

---

**Buzsaki, G. (2006). _Rhythms of the Brain_. Oxford: Oxford University Press. ISBN: 978-0199828234.**

- **Concept**: Neural oscillation bands serve distinct cognitive functions: gamma (30-100 Hz, perception/attention), theta (4-8 Hz, memory/navigation/learning), delta (0.5-4 Hz, deep sleep/consolidation).
- **Roko adaptation**: Three cognitive speeds named after oscillation bands. Gamma (~5-15 seconds, reactive), Theta (~75 seconds, reflection), Delta (hours, consolidation).
- **Roko source**: `docs/v1/00-architecture/10-three-cognitive-speeds.md` (line 7)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

## Active Inference and Free Energy Principle

The Free Energy Principle (FEP), developed by Karl Friston, proposes that all self-organizing systems minimize variational free energy — a quantity closely related to surprise or prediction error. Active inference extends this to action: agents select actions that minimize expected free energy over future states.

**Documents using this section**: `tmp/07-online-learning.md`, `tmp/13-cognitive-architecture.md`

---

**Friston, K. (2006). A Free Energy Principle for the Brain. _Journal of Physiology-Paris_, 100(1-3), 70-87.**
[DOI: 10.1016/j.jphysparis.2006.10.001](https://doi.org/10.1016/j.jphysparis.2006.10.001)

- **Concept**: All self-organizing biological systems minimize variational free energy. The foundational principle for prediction-error-driven cognition.
- **Roko adaptation**: Core routing principle. Every tier selection (T0/T1/T2) and context selection decision is driven by EFE minimization.
- **Crate**: `roko-learn` (routing), `roko-orchestrator`, `roko-core`
- **tmp/ cross-refs**: `07-online-learning.md`, `13-cognitive-architecture.md`

---

**Friston, K., Rigoli, F., Ognibene, D., Mathys, C., Fitzgerald, T., & Pezzulo, G. (2015). Active Inference and Epistemic Value. _Cognitive Neuroscience_, 6(4), 187-214.**
[DOI: 10.1080/17588928.2015.1020053](https://doi.org/10.1080/17588928.2015.1020053)

- **Concept**: EFE = pragmatic_value + epistemic_value. Agents naturally seek information when uncertain and exploit knowledge when confident.
- **Roko adaptation**: High-epistemic-value knowledge prioritized when uncertain; high-pragmatic-value knowledge dominates when confident.
- **tmp/ cross-refs**: `07-online-learning.md`, `13-cognitive-architecture.md`

---

**Millidge, B., Tschantz, A., & Buckley, C.L. (2021). Whence the Expected Free Energy? _Neural Computation_, 33(2), 447-482.**
[DOI: 10.1162/neco_a_01354](https://doi.org/10.1162/neco_a_01354)

- **Concept**: Naive EFE extension into the future actually discourages exploration rather than encouraging it. Careful mathematical formulation is required to preserve the epistemic value term.
- **Roko adaptation**: Essential corrective for proper EFE implementation. Prevents the naive implementation trap.
- **tmp/ cross-refs**: `07-online-learning.md`

---

**Parr, T., Pezzulo, G., & Friston, K. (2022). _Active Inference: The Free Energy Principle in Mind, Brain, and Behavior_. Cambridge, MA: MIT Press. ISBN: 978-0262045353.**

- **Concept**: Complete mathematical framework for active inference in artificial agents, covering generative models, policy selection, and learning under the FEP.
- **Roko adaptation**: Primary implementation reference for the full EFE computation.
- **tmp/ cross-refs**: `07-online-learning.md`

---

**Shafiei, A., Jesawada, H., Friston, K., & Russo, G. (2025). Distributionally Robust Free Energy Principle for Decision-Making. _Nature Communications_, 17, 707.**
[DOI: 10.1038/s41467-025-67348-6](https://doi.org/10.1038/s41467-025-67348-6)

- **Concept**: DR-FREE: distributionally robust active inference. Agents complete tasks even when state-of-the-art models fail under training-environment ambiguity.
- **Roko adaptation**: Tier routing under model uncertainty.

---

**Koudahl, M.T., Beronov, B., de Vries, B., & Kouw, W.M. (2024). Active Inference for Self-Organizing Multi-LLM Systems.**
[arXiv:2412.10425](https://arxiv.org/abs/2412.10425)

- **Concept**: Active inference as a cognitive layer above LLM agents, dynamically adjusting prompts and retrieval through information-seeking behavior driven by EFE minimization.
- **Roko adaptation**: Validates active-inference-driven context assembly.

---

## Cognitive Energy and Fatigue

Cognitive resources — attention, working memory, self-control — are limited and depletable. This provides the theoretical basis for modeling and managing agent cognitive resource consumption.

**IronClaw relevance**: Maps directly to IronClaw's context window budget system and the progressive tool disclosure feature.

---

**Kahneman, D. (1973). _Attention and Effort_. Englewood Cliffs, NJ: Prentice-Hall. ISBN: 978-0130505187.**

- **Concept**: Attention as a scarce resource drawn from a limited "effort supply" that replenishes over time.
- **Roko adaptation**: The cognitive energy pool. Energy is Kahneman's "effort supply" made explicit and computable.
- **Roko source**: `docs/v1/00-architecture/29-cognitive-energy-model.md` (lines 16, 20)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

**Baumeister, R.F., Bratslavsky, E., Muraven, M., & Tice, D.M. (1998). Ego Depletion: Is the Active Self a Limited Resource? _Journal of Personality and Social Psychology_, 74(5), 1252-1265.**
[DOI: 10.1037/0022-3514.74.5.1252](https://doi.org/10.1037/0022-3514.74.5.1252)

- **Concept**: Limited self-regulatory resource. Performing a self-control task depletes a shared pool, degrading subsequent self-control performance.
- **Roko adaptation**: Sustained high-intensity computation degrades output quality. Output quality is proportional to energy_fraction^0.3.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

**Hockey, G.R.J. (2011). A Motivational Control Theory of Cognitive Fatigue. In _Cognitive Fatigue: Multidisciplinary Perspectives on Current Research and Future Applications_, pp. 167-187.**
[DOI: 10.1037/12343-008](https://doi.org/10.1037/12343-008)

- **Concept**: Three levels of compensatory control under fatigue: performance protection, strategy adjustment, goal disengagement.
- **Roko adaptation**: Fatigue penalty = performance protection. Energy zone degradation = strategy adjustment. Critical zone goal count limits = goal disengagement.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

## Collective Intelligence

Collective intelligence refers to the shared intelligence that emerges from collaboration among diverse agents.

**Documents using this section**: `tmp/15-orchestrator-swarm.md`, `tmp/13-cognitive-architecture.md`

---

**Woolley, A.W., Chabris, C.F., Pentland, A., Hashmi, N., & Malone, T.W. (2010). Evidence for a Collective Intelligence Factor in the Performance of Human Groups. _Science_, 330(6004), 686-688.**
[DOI: 10.1126/science.1193147](https://doi.org/10.1126/science.1193147)

- **Concept**: Groups exhibit a measurable collective intelligence factor (C-Factor) not predicted by average or maximum individual IQ. C-Factor correlates with social sensitivity, turn-taking equality, and female group membership.
- **Roko adaptation**: Five C-Factor process variables implemented as runtime metrics: turn-taking entropy, peer prediction accuracy, citation reciprocity, delivery rate, HDC diversity.
- **Crate**: `roko-core` (`cfactor.rs`)
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`, `13-cognitive-architecture.md`

---

**Surowiecki, J. (2004). _The Wisdom of Crowds_. New York: Doubleday. ISBN: 978-0385503860.**

- **Concept**: Four conditions for wise crowds: diversity of opinion, independence of agents, decentralization, and aggregation mechanism.
- **Roko adaptation**: WisdomGate checks four Surowiecki conditions before accepting group consensus as reliable.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

---

**Hawkins, J., Ahmad, S., Cui, Y., & Scheinkman, L. (2017). A Theory of How Columns in the Neocortex Enable Learning the Structure of the World. _Frontiers in Neural Circuits_, 11, 81.**
[DOI: 10.3389/fncir.2017.00081](https://doi.org/10.3389/fncir.2017.00081)

- **Concept**: Thousand Brains Theory: multiple cortical columns independently model the world and vote on perception via a consensus mechanism.
- **Roko adaptation**: Multi-agent estimation consensus mirrors cortical column voting architecture.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

---

## Yerkes-Dodson Law

The inverted-U relationship between arousal/pressure and performance was established experimentally in 1908 and has been replicated across many domains.

---

**Yerkes, R.M. & Dodson, J.D. (1908). The Relation of Strength of Stimulus to Rapidity of Habit-Formation. _Journal of Comparative Neurology and Psychology_, 18(5), 459-482.**
[DOI: 10.1002/cne.920180503](https://doi.org/10.1002/cne.920180503)

- **Concept**: Inverted-U relationship between arousal/pressure and performance. Performance peaks at moderate pressure and degrades at both extremes.
- **Roko adaptation**: Conductor uses Yerkes-Dodson to adjust intervention aggressiveness. Modeled as Gaussian: `exp(-((pressure - optimal)^2) / (2 * width^2))` with default optimal=0.5, width=0.25.
- **Crate**: `roko-conductor` (`yerkes_dodson.rs`)
- **Roko source**: `crates/roko-conductor/src/yerkes_dodson.rs`
- **tmp/ cross-refs**: `06-conductor-anomaly.md`

---

## Cybernetics and Systems Theory

Cybernetics — the science of control and communication in animals and machines — provides the foundational framework for understanding purposive behavior in complex systems.

**Documents using this section**: `tmp/06-conductor-anomaly.md`, `tmp/13-cognitive-architecture.md`, `tmp/15-orchestrator-swarm.md`

---

**Wiener, N. (1948). _Cybernetics: Or Control and Communication in the Animal and the Machine_. Cambridge, MA: MIT Press. ISBN: 978-0262730099.**

- **Concept**: Feedback-based control as the foundation of purposive behavior in both machines and organisms.
- **Roko adaptation**: The fundamental cognitive loop (sense-assess-compose-act-verify-react) is a cybernetic feedback system.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

**Ashby, W.R. (1956). _An Introduction to Cybernetics_. London: Chapman & Hall.**

- **Concept**: Law of Requisite Variety: a controller's complexity (variety) must be at least equal to the system's complexity for effective regulation.
- **Roko adaptation**: The gate system must have sufficient verification variety (7 rungs) to match the variety of failure modes in LLM-generated output.
- **tmp/ cross-refs**: `05-gate-verification.md`, `13-cognitive-architecture.md`

---

**Beer, S. (1972). _Brain of the Firm_. London: Allen Lane. ISBN: 978-0713901375.**

- **Concept**: Viable System Model (VSM): five recursively nested subsystems (S1 implementation, S2 coordination, S3 control, S4 intelligence, S5 policy).
- **Roko adaptation**: S1-5 mapping to conductor hierarchy. L1 TurnConductor (S1), L2 TaskConductor, L3 PlanConductor, L4 FleetConductor (S5).
- **Crate**: `roko-conductor` (`federation.rs`)
- **Roko source**: `crates/roko-conductor/src/federation.rs` (line 4)
- **tmp/ cross-refs**: `06-conductor-anomaly.md`, `13-cognitive-architecture.md`, `15-orchestrator-swarm.md`

---

**Boyd, J. (1987). A Discourse on Winning and Losing. Unpublished briefing. U.S. Air Force.**

- **Concept**: OODA Loop (Observe-Orient-Decide-Act): the standard military decision cycle emphasizing tempo and adaptability.
- **Roko adaptation**: Roko's cognitive loop extends OODA with Gate (verification) and Policy (safety constraints) phases.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

**Conant, R.C. & Ashby, W.R. (1970). Every Good Regulator of a System Must Be a Model of That System. _International Journal of Systems Science_, 1(2), 89-97.**
[DOI: 10.1080/00207727008920220](https://doi.org/10.1080/00207727008920220)

- **Concept**: Good Regulator Theorem: effective control requires an internal model of the system being controlled.
- **Roko adaptation**: The world model requirement — agents must maintain internal models of their environment.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

**Argyris, C. & Schon, D.A. (1978). _Organizational Learning: A Theory of Action Perspective_. Reading, MA: Addison-Wesley. ISBN: 978-0201001754.**

- **Concept**: Triple-loop learning: single-loop (fix errors within assumptions), double-loop (change the assumptions/strategies), triple-loop (change how you learn).
- **Roko adaptation**: Maps to cognitive speeds. Gamma = fix errors (single-loop). Theta = change strategy (double-loop). Delta = change how you learn (triple-loop).
- **tmp/ cross-refs**: `13-cognitive-architecture.md`, `07-online-learning.md`

---

## IIT and Consciousness Metrics

**Tononi, G. (2004). An Information Integration Theory of Consciousness. _BMC Neuroscience_, 5, 42.**
[DOI: 10.1186/1471-2202-5-42](https://doi.org/10.1186/1471-2202-5-42)

- **Concept**: Integrated Information Theory (IIT): Phi measures how much a system is "more than the sum of its parts" via irreducible causal interactions.
- **Roko adaptation**: IitPhiMetric computes Phi over N subsystems from a mutual information matrix, estimating irreducible information integration as a system health indicator.
- **Crate**: `roko-daimon` (`somatic_ta.rs`)
- **Roko source**: `crates/roko-daimon/src/somatic_ta.rs` (lines 8, 27-28)
- **tmp/ cross-refs**: `03-affect-engine.md`

---

## Emergent Goals and Intrinsic Motivation

**Documents using this section**: `tmp/13-cognitive-architecture.md`

---

**Colas, C., Karch, T., Sigaud, O., & Oudeyer, P.-Y. (2022). Autotelic Agents with Intrinsically Motivated Goal Exploration Processes. _Journal of Machine Learning Research_, 23, 1-41.**
[JMLR link](https://jmlr.org/papers/v23/21-0345.html)

- **Concept**: Autotelic agents set their own goals based on intrinsic motivation (curiosity, competence progress, surprise). Goal representation and goal-conditioned policy learning enable open-ended development.
- **Roko adaptation**: Goal emergence engine synthesizes goals from affect (what the agent wants), knowledge (what it knows), and experience (what it has done).
- **Roko source**: `docs/v2/05-AGENT.md` (lines 1318, 1334)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

**Schmidhuber, J. (2010). Formal Theory of Creativity, Fun, and Intrinsic Motivation (1990-2010). _IEEE Transactions on Autonomous Mental Development_, 2(3), 230-247.**
[DOI: 10.1109/TAMD.2010.2056368](https://doi.org/10.1109/TAMD.2010.2056368)

- **Concept**: Intrinsic motivation from compression progress. Curiosity as a drive to compress experience into more efficient representations.
- **Roko source**: `docs/v2/05-AGENT.md` (line 1318)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

**Vygotsky, L.S. (1978). _Mind in Society: The Development of Higher Psychological Processes_. Cambridge, MA: Harvard University Press. ISBN: 978-0674576292.**

- **Concept**: Zone of Proximal Development (ZPD): the distance between what a learner can do independently and what they can do under guidance. Optimal learning occurs at the ZPD boundary.
- **Roko adaptation**: ZPD score peaks when goals are challenging enough to learn from but achievable enough to avoid frustration.
- **Roko source**: `docs/v2/05-AGENT.md` (lines 1334, 1629)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

## Philosophy of Agency

---

**Jonas, H. (1966). _The Phenomenon of Life: Toward a Philosophical Biology_. Evanston, IL: Northwestern University Press. ISBN: 978-0810110038.**

- **Concept**: Needful freedom: metabolism as freedom-through-necessity. Life maintains itself through active engagement with its environment.
- **Roko adaptation**: Economic burn rate. Agents must consume resources to operate, creating natural prioritization pressure.

---

**Derrida, J. (1993). _Spectres de Marx_. Paris: Galilee.**

- **Concept**: Hauntology: each entity is "differently haunted" by its past. The present is constituted by traces of the past.
- **Roko adaptation**: Each agent is differently haunted by its experiential traces. This solves the Alpha Convergence Problem.
- **tmp/ cross-refs**: `02-dream-consolidation.md`

---

**Popper, K. (1972). _Objective Knowledge: An Evolutionary Approach_. Oxford: Clarendon Press. ISBN: 978-0198750246.**

- **Concept**: Knowledge evolves through conjecture and refutation. Falsification is the engine of progress.
- **Roko adaptation**: AntiKnowledge type: explicitly stored knowledge about what does NOT work.
- **tmp/ cross-refs**: `10-universal-engram.md`

---

**Maturana, H.R. & Varela, F.J. (1980). _Autopoiesis and Cognition: The Realization of the Living_. Dordrecht: Reidel. ISBN: 978-9027710161.**

- **Concept**: Autopoietic systems produce the components that maintain them.
- **Roko adaptation**: Agents produce knowledge that sustains their own operation. The NeuroStore is autopoietic.

---

**Heidegger, M. (1927). _Sein und Zeit_ [Being and Time]. Max Niemeyer Verlag.**

- **Concept**: Being-toward-death as the structure of authentic temporality. Finite horizons create urgency that shapes prioritization.
- **Roko adaptation**: Agents under resource constraints experience temporality that shapes prioritization.

---

**Whitehead, A.N. (1929). _Process and Reality: An Essay in Cosmology_. New York: Macmillan.**

- **Concept**: Process philosophy: reality as ongoing process of experience, not static substance. Each event (actual occasion) builds on prior events.
- **Roko adaptation**: The Engram DAG is a Whiteheadian actual occasion chain.

---

**Varela, F.J., Thompson, E., & Rosch, E. (1991). _The Embodied Mind: Cognitive Science and Human Experience_. Cambridge, MA: MIT Press. ISBN: 978-0262720212.**

- **Concept**: Enactive cognition: cognition as enaction (bringing forth a world through interaction), not passive information processing.
- **Roko adaptation**: Agents don't passively process information; they actively construct their cognitive world through tool use and environment modification.

---

*Navigation: [README](README.md) | [HDC and VSA](hdc-and-vsa.md) | [Memory and Learning](memory-and-learning.md) | [Verification and Safety](verification-and-safety.md) | [Agents and Orchestration](agents-and-orchestration.md) | [Blockchain and Economics](blockchain-and-economics.md) | [Context and Search](context-and-search.md) | [Math and Statistics](math-and-statistics.md)*

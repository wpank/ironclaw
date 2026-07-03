# Agents and Orchestration — Agent Patterns, Swarm, DAG Execution Citations

Part of the [Research Citations](README.md) collection.

---

## Agent Harnesses and Tool Use

The empirical study of how LLM agents interact with their computational environment — tools, file systems, code executors, web browsers — has produced clear design principles: purpose-built interfaces outperform general interfaces, structured error feedback dramatically affects performance, and the harness design dominates model selection in determining overall performance.

**Documents using this section**: `tmp/17-agent-patterns.md`

---

**Yang, J., Jimenez, C.E., Wettig, A., Lieret, K., Yao, S., Narasimhan, K., & Press, O. (2024). SWE-agent: Agent-Computer Interfaces Enable Automated Software Engineering. _Advances in Neural Information Processing Systems (NeurIPS)_, 37.**
[arXiv:2405.15793](https://arxiv.org/abs/2405.15793)

- **Concept**: Agent-Computer Interfaces (ACI) purpose-built for LLM agents dramatically outperform general shell interfaces. How the agent interacts with its environment matters as much as the model's reasoning.
- **Roko adaptation**: Influences tool permissions, structured error digests, and role-specific feedback formatting.
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/21-references/14-agent-harnesses-and-tool-use.md`](https://github.com/wpank/roko/blob/main/docs/v1/21-references/14-agent-harnesses-and-tool-use.md)
- **tmp/ cross-refs**: `17-agent-patterns.md`

---

**Jimenez, C.E., Yang, J., Wettig, A., Yao, S., Pei, K., Press, O., & Narasimhan, K. (2024). SWE-bench: Can Language Models Resolve Real-World GitHub Issues? _International Conference on Learning Representations (ICLR)_.**
[arXiv:2310.06770](https://arxiv.org/abs/2310.06770)

- **Concept**: Benchmark of 2,294 real GitHub issues from 12 popular Python repositories. The gold standard for evaluating coding agents on realistic software engineering tasks.
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/21-references/14-agent-harnesses-and-tool-use.md`](https://github.com/wpank/roko/blob/main/docs/v1/21-references/14-agent-harnesses-and-tool-use.md)
- **tmp/ cross-refs**: `17-agent-patterns.md`

---

**Anthropic (2024). Building Effective Agents. Anthropic Engineering Blog.**

- **Concept**: Composition over complexity: keep individual agents simple and focused, compose through a controller rather than building monolithic agents.
- **Roko adaptation**: Each agent role does one thing; the orchestrator composes them into pipelines.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`, `17-agent-patterns.md`

---

**Hu, S., Lu, C., & Clune, J. (2025). Automated Design of Agentic Systems (ADAS). _International Conference on Learning Representations (ICLR)_.**
[arXiv:2408.08435](https://arxiv.org/abs/2408.08435)

- **Concept**: Meta-agent that searches the space of agent architectures, discovering novel building blocks and compositions. "Meta Agent Search" discovers agents that outperform human-designed baselines.
- **Roko adaptation**: Roko provides the composable trait system (6 Synapse traits) that ADAS-style search operates over.

---

**Lee, H., Chen, M., Gupta, A., & Hashimoto, T. (2026). Meta-Harness: End-to-End Optimization of Model Harnesses.**
[arXiv:2603.28052](https://arxiv.org/abs/2603.28052)

- **Concept**: "The scaffold IS the product." 6x performance gap from scaffold changes alone. +7.7 points text classification, +4.7 on IMO math, at 4x fewer tokens.
- **Roko adaptation**: Foundational for Roko's harness engineering approach. The agent framework matters more than any individual model.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`, `17-agent-patterns.md`

---

**Khattab, O., Singhvi, A., et al. (2024). DSPy: Compiling Declarative Language Model Calls into Self-Improving Pipelines. _International Conference on Learning Representations (ICLR)_.**
[arXiv:2310.03714](https://arxiv.org/abs/2310.03714)

- **Concept**: Replaces hand-crafted prompts with declarative signatures; DSPy compiler optimizes prompts, few-shot examples, and fine-tuning data automatically through program synthesis.
- **Roko adaptation**: Influences prompt budget allocation and optimization approach.

---

## Stigmergic Coordination

Stigmergy is coordination through environmental modification — agents leave traces in their environment that influence the behavior of other agents, enabling complex collective behavior without direct communication.

**Documents using this section**: `tmp/15-orchestrator-swarm.md`, `tmp/13-cognitive-architecture.md`

---

**Grasse, P.-P. (1959). La reconstruction du nid et les coordinations interindividuelles chez Bellicositermes natalensis et Cubitermes sp. La theorie de la stigmergie. _Insectes Sociaux_, 6(1), 41-80.**
[DOI: 10.1007/BF02223791](https://doi.org/10.1007/BF02223791)

- **Concept**: Coined "stigmergy" (stigma = mark, ergon = work). Termites coordinate construction without direct communication by responding to the current state of the environment.
- **Roko adaptation**: Foundational for the Agent Mesh pheromone field. Agents deposit coordination signals that decay over time and are reinforced by confirmation.
- **Crate**: `roko-core` (pheromone types), Bus system
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/11-stigmergy-as-bus.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/11-stigmergy-as-bus.md), [`https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/12-pheromone-mechanics-and-interference.md`](https://github.com/wpank/roko/blob/main/docs/v2-depth/11-memory/12-pheromone-mechanics-and-interference.md)
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`, `13-cognitive-architecture.md`

---

**Dorigo, M. & Gambardella, L.M. (1997). Ant Colony System: A Cooperative Learning Approach to the Traveling Salesman Problem. _IEEE Transactions on Evolutionary Computation_, 1(1), 53-66.**
[DOI: 10.1109/4235.585892](https://doi.org/10.1109/4235.585892)

- **Concept**: Ant Colony Optimization: pheromone deposit/evaporation cycle. Ants deposit pheromone proportional to route quality; evaporation prevents stagnation.
- **Roko adaptation**: Confirmation-based half-life extension. Seven pheromone kinds with distinct half-lives: Threat=2h, Opportunity=4h, Wisdom=24h, Alpha=1h, Pattern=12h, Anomaly=6h, Consensus=8h.
- **Crate**: `roko-core`
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

---

**Parunak, H.V.D., Brueckner, S., & Sauter, J. (2002). Digital Pheromone Mechanisms for Coordination of Unmanned Vehicles. _Proceedings of the International Conference on Autonomous Agents and Multiagent Systems (AAMAS)_.**

- **Concept**: Time-decaying digital signals enable emergent coordination in artificial systems without central coordination. Demonstrates scalability to hundreds of vehicles.
- **Roko adaptation**: Pheromone Engram decay and reinforcement mechanism.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

---

**Simard, S.W., Perry, D.A., Jones, M.D., Myrold, D.D., Durall, D.M., & Molina, R. (1997). Net Transfer of Carbon Between Ectomycorrhizal Tree Species in the Field. _Nature_, 388, 579-582.**
[DOI: 10.1038/41557](https://doi.org/10.1038/41557)

- **Concept**: Underground fungal networks share carbon, nutrients, and defense signals between trees without direct communication — the "wood wide web."
- **Roko adaptation**: Agent Mesh topology mirrors mycorrhizal "underground relay" architecture for knowledge sharing without direct agent communication.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

---

## FIPA Agent Lifecycle

The Foundation for Intelligent Physical Agents (FIPA) developed the most complete formal standards for agent lifecycle semantics.

**IronClaw relevance**: IronClaw's job state machine (Pending -> InProgress -> Completed -> Submitted -> Accepted) could be enriched with FIPA lifecycle semantics, particularly the SUSPENDED and WAITING states for long-running background jobs.

---

**FIPA (2002). Agent Management Specification. FIPA00023. Foundation for Intelligent Physical Agents.**

- **Concept**: Six formal states: INITIATED, ACTIVE, SUSPENDED, WAITING, TRANSIT, DELETED. State transitions are precisely defined with pre- and post-conditions.
- **Roko adaptation**: Roko maps FIPA states into cloud-native provisioning pipeline: Created (INITIATED), Provisioning, Active (ACTIVE), Paused (SUSPENDED), Draining (WAITING), Migrating (TRANSIT), Deleted (DELETED).
- **Crate**: `roko-agent` (`lifecycle.rs`)
- **Roko source**: [`https://github.com/wpank/roko/blob/main/docs/v1/17-lifecycle/01-agent-creation.md`](https://github.com/wpank/roko/blob/main/docs/v1/17-lifecycle/01-agent-creation.md) (lines 359-382)
- **tmp/ cross-refs**: `17-agent-patterns.md`

---

## Biological Analogues

Biological systems have solved optimization problems that engineering is still working on: efficient distributed coordination (ant colonies, fungal networks), adaptive response to environmental change (immune system, epigenetics), and emergent group intelligence (beehives, cortical columns).

**Documents using this section**: `tmp/15-orchestrator-swarm.md`

---

**Pirolli, P. & Card, S.K. (1999). Information Foraging. _Psychological Review_, 106(4), 643-675.**
[DOI: 10.1037/0033-295X.106.4.643](https://doi.org/10.1037/0033-295X.106.4.643)

- **Concept**: Adapts optimal foraging theory to information seeking. "Information scent" guides information gathering behavior.
- **Roko adaptation**: HDC similarity scores serve as information scent to guide knowledge retrieval from NeuroStore.

---

**Turing, A.M. (1952). The Chemical Basis of Morphogenesis. _Philosophical Transactions of the Royal Society of London, Series B_, 237(641), 37-72.**
[DOI: 10.1098/rstb.1952.0012](https://doi.org/10.1098/rstb.1952.0012)

- **Concept**: Reaction-diffusion systems with an activator and inhibitor generate spatial patterns from homogeneous initial conditions through symmetry breaking.
- **Roko adaptation**: Pheromone emission (activation) and decay (inhibition) creates emergent specialization patterns in agent collectives through reaction-diffusion dynamics.

---

**Holldobler, B. & Wilson, E.O. (2008). _The Superorganism: The Beauty, Elegance, and Strangeness of Insect Societies_. New York: W.W. Norton. ISBN: 978-0393067040.**

- **Concept**: Insect colonies function as superorganisms where collective intelligence emerges from simple local rules.
- **Roko adaptation**: When C-Factor > 1.0, the agent collective is operating as a superorganism with emergent capabilities exceeding any individual.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

---

**Nealson, K.H., Platt, T., & Hastings, J.W. (1970). Cellular Control of the Synthesis and Activity of the Bacterial Luminescent System. _Journal of Bacteriology_, 104(1), 313-322.**
[DOI: 10.1128/jb.104.1.313-322.1970](https://doi.org/10.1128/jb.104.1.313-322.1970)

- **Concept**: First description of quorum sensing in Vibrio fischeri: individual bacteria produce signaling molecules; when concentration exceeds a threshold (quorum), collective behavior is triggered.
- **Roko adaptation**: Quorum-based triggering in agent collectives: decisions require a quorum of confirming signals.

---

## Evolutionary and Generational Dynamics

Evolutionary biology provides concepts for understanding how knowledge populations change over time under selection pressure.

---

**Ray, T.S. (1991). An Approach to the Synthesis of Life. In _Artificial Life II_, Vol. XI, pp. 371-408. Redwood City, CA: Addison-Wesley.**

- **Concept**: Tierra: digital evolution requires a reaper mechanism for complex evolution to occur. 300+ genotypes emerged only when entities had finite lifespans and resource competition.
- **Roko adaptation**: Knowledge decay necessity — without resource pressure (demurrage), knowledge accumulates indefinitely and becomes useless signal.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

**Baldwin, J.M. (1896). A New Factor in Evolution. _American Naturalist_, 30(354), 441-451.**
[DOI: 10.1086/276408](https://doi.org/10.1086/276408)

- **Concept**: The Baldwin Effect: behavior learned by individuals can eventually become instinctual across generations under selection pressure.
- **Roko adaptation**: Heuristics learned at Working tier are promoted to Persistent tier when proven reliable.
- **tmp/ cross-refs**: `10-universal-engram.md`

---

**Price, G.R. (1970). Selection and Covariance. _Nature_, 227, 520-521.**
[DOI: 10.1038/227520a0](https://doi.org/10.1038/227520a0)

- **Concept**: The Price equation: universal selection equation relating fitness change to covariance between character and fitness.
- **Roko adaptation**: Knowledge persistence = covariance with Gate success.

---

**Lenski, R.E., Ofria, C., Pennock, R.T., & Adami, C. (2003). The Evolutionary Origin of Complex Features. _Nature_, 423(6936), 139-144.**
[DOI: 10.1038/nature01568](https://doi.org/10.1038/nature01568)

- **Concept**: Long-Term Evolution Experiment with digital organisms: complex features require generational turnover. Novel functions emerge only from successive mutations across many generations.
- **Roko adaptation**: Lossy knowledge compression through tier promotion (Episode -> Insight -> Heuristic -> Playbook) produces generalization across generations.

---

**Shuvaev, S., Starosta, S., Bhatt, D., Bhatt, A., Bhatt, P., & Bhatt, V. (2024). Encoding Innate Ability Through a Genomic Bottleneck. _Proceedings of the National Academy of Sciences_, 121(39), e2403306121.**
[DOI: 10.1073/pnas.2403306121](https://doi.org/10.1073/pnas.2403306121)

- **Concept**: The genome is approximately 1000x smaller than the information needed for brain connectivity, yet organisms have innate behaviors. The genomic bottleneck forces compression that becomes the regularizer enabling generalization.
- **Roko adaptation**: Tier promotion forces generalization. Knowledge compressed during Delta consolidation generalizes better than the raw episodes.
- **tmp/ cross-refs**: `10-universal-engram.md`

---

## Temporal Knowledge and Causal Reasoning

Knowledge is not timeless — facts that were true yesterday may be false today.

---

**Allen, J.F. (1983). Maintaining Knowledge about Temporal Intervals. _Communications of the ACM_, 26(11), 832-843.**
[DOI: 10.1145/182.358434](https://doi.org/10.1145/182.358434)

- **Concept**: 13 mutually exclusive and exhaustive temporal interval relations. Complete vocabulary for reasoning about temporal relationships between events.
- **Roko adaptation**: Temporal knowledge layer implements Allen's interval algebra for reasoning about knowledge validity windows.

---

**Kowalski, R. & Sergot, M. (1986). A Logic-Based Calculus of Events. _New Generation Computing_, 4(1), 67-95.**
[DOI: 10.1007/BF03037383](https://doi.org/10.1007/BF03037383)

- **Concept**: Event calculus: logical formalism for tracking fluent changes (what was true when) over time, using events as the primitive that initiate and terminate fluents.
- **Roko adaptation**: Tracks "what was true when?" for causal reasoning about agent behavior history.

---

## Additional Notable References

**Arbesman, S. (2012). _The Half-Life of Facts: Why Everything We Know Has an Expiration Date_. New York: Current/Penguin. ISBN: 978-1591844969.**

- **Concept**: Per-domain factual decay rates. Different types of knowledge expire at measurable, distinct rates. Medical facts have a half-life of 45 years; physics facts, centuries.
- **Roko adaptation**: Per-type half-life calibration in the NeuroStore.
- **tmp/ cross-refs**: `10-universal-engram.md`

---

**Dohare, S., Sutton, R.S., & Kumar, S. (2024). Loss of Plasticity in Deep Continual Learning. _Nature_, 632, 768-774.**
[DOI: 10.1038/s41586-024-07711-7](https://doi.org/10.1038/s41586-024-07711-7)

- **Concept**: 90% of units become "dead" (non-updating) in continual learning systems. Periodic replacement of dead units outperforms continuous adaptation for preventing plasticity loss.
- **Roko adaptation**: Validates knowledge decay and the Curator's pruning cycle. Knowledge stores require active maintenance to prevent calcification.

---

**Sculley, D., Holt, G., Golovin, D., Davydov, E., et al. (2015). Hidden Technical Debt in Machine Learning Systems. _Advances in Neural Information Processing Systems (NeurIPS)_, 28.**

- **Concept**: Technical debt compounds silently in long-running ML systems. Only a small fraction of code in an ML system is the model itself.
- **Roko adaptation**: Controlled knowledge decay prevents silent accumulation of stale heuristics — the cognitive analog of ML technical debt.

---

**Maxwell, J.C. (1868). On Governors. _Proceedings of the Royal Society of London_, 16, 270-283.**
[DOI: 10.1098/rspl.1867.0055](https://doi.org/10.1098/rspl.1867.0055)

- **Concept**: First mathematical analysis of feedback control. The centrifugal governor prevents steam engine oscillation through proportional damping.
- **Roko adaptation**: Foundational for the adaptive clock control system.

---

**Sterling, P. (2012). Allostasis: A Model of Predictive Regulation. _Physiology and Behavior_, 106(1), 86-93.**
[DOI: 10.1016/j.physbeh.2011.06.004](https://doi.org/10.1016/j.physbeh.2011.06.004)

- **Concept**: Allostasis: predictive regulation that anticipates needs rather than reacting to deviations from a fixed setpoint (homeostasis). More efficient than reactive correction.
- **Roko adaptation**: Predictive foraging — agents anticipate which knowledge they will need rather than retrieving reactively.

---

**Kauffman, S.A. (1993). _The Origins of Order: Self-Organization and Selection in Evolution_. Oxford: Oxford University Press. ISBN: 978-0195079517.**

- **Concept**: Self-organized criticality. Systems near the "edge of chaos" optimize adaptability — too much order stagnates, too much chaos dissolves structure.
- **Roko adaptation**: Adaptive clock targets edge of chaos for optimal balance of stability and adaptability.

---

*Navigation: [README](README.md) | [HDC and VSA](hdc-and-vsa.md) | [Memory and Learning](memory-and-learning.md) | [Affect and Cognition](affect-and-cognition.md) | [Verification and Safety](verification-and-safety.md) | [Blockchain and Economics](blockchain-and-economics.md) | [Context and Search](context-and-search.md) | [Math and Statistics](math-and-statistics.md)*

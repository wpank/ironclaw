# Context and Search — Prompt Composition, Code Intelligence, Attention Citations

Part of the [Research Citations](README.md) collection.

---

## Context Engineering

Context engineering addresses how to select, arrange, and compress information within an LLM's context window to maximize response quality. The LLM does not see "the world" — it sees only what is in its context. The key empirical findings: LLMs attend more to the beginning and end of context (U-shaped attention), irrelevant context actively degrades performance (not just dilutes attention), and retrieved content that does not support the query is worse than no retrieval at all.

**Documents using this section**: `tmp/09-budget-composition.md`

---

**Liu, N.F., Lin, K., Hewitt, J., Paranjape, A., Scott, M., Sabba, F., & Manning, C.D. (2024). Lost in the Middle: How Language Models Use Long Contexts. _Transactions of the Association for Computational Linguistics_, 12, 157-173.**
[arXiv:2307.03172](https://arxiv.org/abs/2307.03172)

- **Concept**: U-shaped attention: LLMs attend more to content at the beginning and end of the context window, with significant degradation for content in the middle.
- **Roko adaptation**: Highest-priority context placed at start and end of composed prompt. Safety and task context always at boundaries.
- **tmp/ cross-refs**: `09-budget-composition.md`

---

**Zhang, Q., Gao, J., Zhang, X., Liu, S., Liu, B., Lim, E., & Shao, Z. (2026). ACE: Agentic Context Engineering — Evolving Contexts for Self-Improving Language Models. _International Conference on Learning Representations (ICLR)_.**
[arXiv:2510.04618](https://arxiv.org/abs/2510.04618)

- **Concept**: Generator-Reflector-Curator cycle for evolving context playbooks. Achieves +10.6% on agent benchmarks, +8.6% on finance benchmarks.
- **Roko adaptation**: Compose-verify-persist cycle in context assembly pipeline.
- **tmp/ cross-refs**: `09-budget-composition.md`

---

**Lewis, P., Perez, E., Piktus, A., Petroni, F., Karpukhin, V., Goyal, N., Kuttler, H., Lewis, M., Yih, W., Rocktaschel, T., Riedel, S., & Kiela, D. (2020). Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks. _Advances in Neural Information Processing Systems (NeurIPS)_, 33.**
[arXiv:2005.11401](https://arxiv.org/abs/2005.11401)

- **Concept**: Retrieval-Augmented Generation (RAG): retrieve relevant documents before generation to ground responses in factual knowledge, reducing hallucination.
- **Roko adaptation**: Per-tick context assembly retrieves relevant knowledge before LLM call.
- **tmp/ cross-refs**: `09-budget-composition.md`

---

**Shi, F., Chen, X., Misra, K., Scales, N., Dohan, D., Chi, E., Salimans, T., & Zhou, D. (2023). Large Language Models Can Be Easily Distracted by Irrelevant Context. _Proceedings of the International Conference on Machine Learning (ICML)_.**
[arXiv:2302.00093](https://arxiv.org/abs/2302.00093)

- **Concept**: Irrelevant context actively degrades LLM performance — not merely dilutes attention but actively misleads reasoning. Filtering is not optional.
- **Captured adaptation**: Quality filtering is mandatory. Budget allocation should prevent irrelevant subsystems from dominating the prompt.
- **tmp/ cross-refs**: `09-budget-composition.md`

---

**Joren, H., Yoran, O., Berant, J., & Glass, J. (2025). Sufficient Context: A New Lens on Retrieval Augmented Generation Systems. _International Conference on Learning Representations (ICLR)_.**
[arXiv:2411.06037](https://arxiv.org/abs/2411.06037)

- **Concept**: Tests whether retrieved snippets alone could plausibly answer the query. Uncovers new RAG failure modes and lifts selective accuracy by 2-10 points.
- **Roko adaptation**: Validates context quality assessment: including wrong context is worse than no context.
- **Roko source**: `docs/v2-depth/02-block/distributed-and-affect-composition.md` (line 31)
- **tmp/ cross-refs**: `09-budget-composition.md`

---

## Information Theory and Signal Processing

Shannon's mathematical theory of communication provides the quantitative vocabulary for information, uncertainty, and compression that underlies much of the cognitive architecture. Predictive processing extends these ideas to cognition: brains (and agents) are hierarchical prediction machines that minimize the information-theoretic surprise of their sensory inputs.

---

**Shannon, C.E. (1948). A Mathematical Theory of Communication. _Bell System Technical Journal_, 27, 379-423 & 623-656.**
[DOI: 10.1002/j.1538-7305.1948.tb01338.x](https://doi.org/10.1002/j.1538-7305.1948.tb01338.x)

- **Concept**: Information entropy H = -sum(p log p), mutual information, channel capacity, source coding theorem. The foundational paper of information theory.
- **Roko adaptation**: Used throughout for measuring knowledge value, information flow, and communication efficiency.

---

**Clark, A. (2013). Whatever Next? Predictive Brains, Situated Agents, and the Future of Cognitive Science. _Behavioral and Brain Sciences_, 36(3), 181-204.**
[DOI: 10.1017/S0140525X12000477](https://doi.org/10.1017/S0140525X12000477)

- **Concept**: Predictive processing: brains as hierarchical prediction machines that minimize prediction error. Perception, action, and attention are unified as consequences of this minimization.
- **Roko adaptation**: Foundational for prediction-error-driven T0/T1/T2 tier routing.

---

**Itti, L. & Baldi, P. (2005). Bayesian Surprise Attracts Human Attention. _Advances in Neural Information Processing Systems (NeurIPS)_, 18.**

- **Concept**: Surprise = KL(posterior || prior). Formally identical to the epistemic EFE component. Human attention is captured by high-surprise stimuli.
- **Roko adaptation**: Active inference agents naturally seek knowledge with the highest Bayesian surprise in context selection scoring.
- **Roko source**: `docs/v2-depth/02-block/active-inference-context-selection.md` (line 32)

---

**Landauer, R. (1961). Irreversibility and Heat Generation in the Computing Process. _IBM Journal of Research and Development_, 5(3), 183-191.**
[DOI: 10.1147/rd.53.0183](https://doi.org/10.1147/rd.53.0183)

- **Concept**: Landauer's principle: erasing one bit of information generates a minimum of kT ln 2 joules of heat. Information erasure is thermodynamically irreversible.
- **Roko adaptation**: Theoretical foundation for the claim that knowledge decay has real computational cost — forgetting is not free.

---

*Navigation: [README](README.md) | [HDC and VSA](hdc-and-vsa.md) | [Memory and Learning](memory-and-learning.md) | [Affect and Cognition](affect-and-cognition.md) | [Verification and Safety](verification-and-safety.md) | [Agents and Orchestration](agents-and-orchestration.md) | [Blockchain and Economics](blockchain-and-economics.md) | [Math and Statistics](math-and-statistics.md)*

# Context And Search Citations

Research context for prompt composition, retrieval, attention placement, and
information-theoretic framing.

## Selected References

| Reference | Contribution | Local relevance |
|---|---|---|
| Liu et al. (2024), "Lost in the Middle" | Long-context models often use boundary information better than middle content. | Motivation for measuring placement strategies instead of assuming longer prompts help. |
| Lewis et al. (2020), RAG | Retrieval-augmented generation for knowledge-intensive tasks. | Baseline framing for workspace memory and code retrieval. |
| Shi et al. (2023), distracted by irrelevant context | Irrelevant context can harm model output. | Supports precision-focused context selection. |
| Sufficient Context (2025) | Retrieval should be judged by whether context is sufficient for the task. | Useful for fixture design and context-quality metrics. |
| Shannon (1948), information theory | Communication, entropy, channel capacity. | Analogy for context budget and compression tradeoffs. |
| Clark (2013), predictive processing | Situated prediction and error minimization. | Background for active-inference-inspired context scoring. |
| Itti and Baldi (2005), Bayesian surprise | Attention directed by surprise. | Useful as a heuristic for selecting informative context. |
| Landauer (1961) | Irreversibility and information cost. | Cautionary analogy for treating context as a finite resource. |

## Use In IronClaw

- Prefer measured context sufficiency over fixed token quotas.
- Keep safety and task-critical context explicit.
- Treat VCG, active inference, and information theory as scoring inspiration
  unless the implementation meets their assumptions.

Navigation: [README](README.md) | [Blockchain and Economics](blockchain-and-economics.md) |
[Agents and Orchestration](agents-and-orchestration.md)

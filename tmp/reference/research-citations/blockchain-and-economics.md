# Blockchain And Economics Citations

Research context for allocation, demurrage, protocol standards, reputation, and
marketplace sketches. Economics terms should be treated as analogies unless the
implementation satisfies the formal assumptions.

## Selected References

| Reference | Contribution | Local relevance |
|---|---|---|
| Vickrey (1961), Clarke (1971), Groves (1973) | VCG mechanism design. | Useful allocation diagnostic; do not claim truthfulness without matching assumptions. |
| Simon (1971), Sims (2003) | Attention as scarce resource. | Context-window budget framing. |
| Nemhauser, Wolsey, and Fisher (1978) | Greedy approximation for submodular maximization. | Candidate baseline for context selection. |
| Duetting et al. (2024) | Mechanism design for LLMs. | Modern bridge between auctions and model systems. |
| Ostrom (1990) | Commons governance. | Useful for shared memory and reputation governance. |
| Gesell (1916) | Demurrage economics. | Analogy for retrieval-weight decay, not physical deletion. |
| Anthropic MCP spec; Google A2A spec | Tool and agent protocol context. | Compare with IronClaw MCP/channel boundaries. |
| ERC-8004 and x402 | Identity and payment protocol ideas. | Future-facing trust/payment references. |
| Kelly (1956), Peters (2019) | Bet sizing and ergodicity. | Budget allocation analogy; verify before use. |
| Hayek (1945), Lo (2004), Taleb (2012) | Distributed knowledge, adaptive markets, antifragility. | Reputation and feedback-loop metaphors. |

## Use In IronClaw

- Local approvals, sandboxing, and auth remain hard gates regardless of
  reputation or market signals.
- Demurrage is safest as a retrieval-weighting idea, not data deletion.
- Chain or payment integrations need separate security and threat-model review.

Navigation: [README](README.md) | [Context and Search](context-and-search.md) |
[Math and Statistics](math-and-statistics.md)

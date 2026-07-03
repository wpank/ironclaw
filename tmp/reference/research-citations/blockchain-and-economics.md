# Blockchain and Economics — Reputation, Tokens, Mechanism Design Citations

Part of the [Research Citations](README.md) collection.

---

## VCG Auction and Mechanism Design

Mechanism design is the branch of economics and game theory concerned with designing rules for collective decision-making that produce desirable outcomes even when participants act strategically. The Vickrey-Clarke-Groves (VCG) mechanism is the canonical incentive-compatible design: it guarantees truthful revelation as a dominant strategy. In Roko, this is applied to the cognitive resource allocation problem — how should competing cognitive subsystems share the finite context-window budget?

**Documents using this section**: `tmp/09-budget-composition.md`, `tmp/13-cognitive-architecture.md`

---

**Vickrey, W. (1961). Counterspeculation, Auctions, and Competitive Sealed Tenders. _Journal of Finance_, 16(1), 8-37.**
[DOI: 10.1111/j.1540-6261.1961.tb02789.x](https://doi.org/10.1111/j.1540-6261.1961.tb02789.x)

- **Concept**: Second-price sealed-bid auction. Truthful bidding is a dominant strategy because the winner pays the second-highest price.
- **Roko adaptation**: Eight bidder subsystems (Task, Code, Episode, Neuro, Safety, Research, Tool, Heuristic) compete for context window tokens. VCG ensures each subsystem bids its true valuation.
- **Crate**: `roko-compose`
- **Roko source**: `docs/v2-depth/02-block/vcg-attention-auction.md`
- **tmp/ cross-refs**: `09-budget-composition.md`, `13-cognitive-architecture.md`

---

**Clarke, E.H. (1971). Multipart Pricing of Public Goods. _Public Choice_, 11(1), 17-33.**
[DOI: 10.1007/BF01726210](https://doi.org/10.1007/BF01726210)

- **Concept**: Extends Vickrey's truthful mechanism to multi-item public good allocation. Participants pay the Clarke tax: the externality their participation imposes on others.
- **tmp/ cross-refs**: `09-budget-composition.md`

---

**Groves, T. (1973). Incentives in Teams. _Econometrica_, 41(4), 617-631.**
[DOI: 10.2307/1914085](https://doi.org/10.2307/1914085)

- **Concept**: Generalizes Clarke's mechanism to team settings with arbitrary payoff functions. Proves that truthful revelation of private information is incentive-compatible in team contexts.
- **Roko adaptation**: Subsystems truthfully reveal their valuation for context tokens to maximize collective output.
- **tmp/ cross-refs**: `09-budget-composition.md`

---

**Simon, H.A. (1971). Designing Organizations for an Information-Rich World. In M. Greenberger (Ed.), _Computers, Communications, and the Public Interest_, pp. 37-72. Baltimore: Johns Hopkins Press.**

- **Concept**: "A wealth of information creates a poverty of attention." Attention is the binding constraint in information-rich environments, not information itself.
- **Captured adaptation**: The context window is the attention constraint. Prompt composition allocates that scarce resource with density scoring and optional VCG-style diagnostics.
- **Roko source**: `docs/v2-depth/02-block/vcg-attention-auction.md` (line 13)
- **tmp/ cross-refs**: `09-budget-composition.md`

---

**Sims, C.A. (2003). Implications of Rational Inattention. _Journal of Monetary Economics_, 50(3), 665-690.**
[DOI: 10.1016/S0304-3932(03)00029-1](https://doi.org/10.1016/S0304-3932(03)00029-1)

- **Concept**: Finite-capacity agents optimally ignore some information — rational inattention. The channel capacity constraint forces selective processing.
- **Captured adaptation**: Agents must be selectively inattentive because they cannot process all available information within the context window; the allocator should make that tradeoff explicit and measurable.
- **tmp/ cross-refs**: `09-budget-composition.md`

---

**Nemhauser, G.L., Wolsey, L.A., & Fisher, M.L. (1978). An Analysis of Approximations for Maximizing Submodular Set Functions — I. _Mathematical Programming_, 14(1), 265-294.**
[DOI: 10.1007/BF01588971](https://doi.org/10.1007/BF01588971)

- **Concept**: Greedy algorithm provides (1-1/e) ≈ 0.632 approximation for maximizing monotone submodular functions subject to cardinality constraints.
- **Roko adaptation**: Context selection is submodular (diminishing returns from adding more of the same type). Greedy knapsack for context window allocation is near-optimal.
- **tmp/ cross-refs**: `09-budget-composition.md`

---

**Duetting, P., Kolumbus, Y., Mirrokni, V., Tardos, E., & Talgam-Cohen, I. (2024). Mechanism Design for Large Language Models. _Proceedings of the ACM Web Conference (WWW Best Paper)_.**
[arXiv:2310.10826](https://arxiv.org/abs/2310.10826)

- **Concept**: Token-by-token mechanism design for multi-LLM output generation. Extends VCG theory to the sequential token generation setting.
- **Roko source**: `docs/v2-depth/02-block/vcg-attention-auction.md` (line 405)
- **tmp/ cross-refs**: `09-budget-composition.md`

---

**Ostrom, E. (1990). _Governing the Commons: The Evolution of Institutions for Collective Action_. Cambridge: Cambridge University Press. ISBN: 978-0521405997.**

- **Concept**: Governing shared resources without central authority. Eight design principles for institutional governance of commons that prevent both the Tragedy of the Commons and over-centralization.
- **Roko adaptation**: Knowledge commons governance in the Agent Mesh. Pheromone field as a commons resource with emergent governance.
- **tmp/ cross-refs**: `09-budget-composition.md`

---

## Gesellian Demurrage

Silvio Gesell, a German-Argentine merchant and self-taught economist, proposed in 1916 that money should carry a holding charge (demurrage) that makes hoarding costly. This concept provides the economic metaphor for knowledge decay.

**IronClaw relevance**: IronClaw's "LLM data is never deleted" principle is compatible with demurrage if applied to retrieval weighting rather than physical deletion — data persists but its weight in retrieval scoring decays.

---

**Gesell, S. (1916). _Die Naturliche Wirtschaftsordnung_ [The Natural Economic Order]. Selbstverlag, Les Hauts Geneveys. Translated by Philip Pye, 1929.**

- **Concept**: Demurrage: money that decays over time to encourage circulation and prevent hoarding. The demurrage charge is the cost of storing purchasing power.
- **Roko adaptation**: Economic metaphor for knowledge decay. KORAI token 1% annual demurrage mirrors knowledge entry decay. `Demurrage` trait: `balance *= (1 - rate)^elapsed_hours`.
- **Crate**: `roko-core` (`demurrage.rs`), `roko-chain` (`korai_token.rs`)
- **Roko source**: `crates/roko-core/src/demurrage.rs`, `crates/roko-chain/src/korai_token.rs`
- **tmp/ cross-refs**: `08-chain-reputation.md`, `10-universal-engram.md`

---

## Protocol Standards and Blockchain

The AI agent protocol stack is rapidly standardizing around three layers: model-tool interaction (MCP), agent-agent communication (A2A), and blockchain-based identity and payment.

**Documents using this section**: `tmp/21-mcp-editor-integration.md`, `tmp/24-smart-contracts.md`, `tmp/08-chain-reputation.md`

---

**Anthropic (2024). Model Context Protocol (MCP) Specification. Version 2024-11-05. [https://spec.modelcontextprotocol.io/]**

- **Concept**: Standardized protocol for model-tool interaction via JSON-RPC 2.0. Defines Resources (data), Tools (executables), and Prompts (templates) as the three primitive types.
- **Roko adaptation**: `roko-agent` implements MCP client for tool dispatch. `roko-mcp` provides the protocol implementation.
- **Crate**: `roko-agent`, `roko-mcp`
- **IronClaw relevance**: IronClaw implements MCP client in `src/tools/mcp/`. See `src/tools/mcp/client.rs`, `src/tools/mcp/factory.rs`.
- **tmp/ cross-refs**: `21-mcp-editor-integration.md`

---

**Google (2025). Agent-to-Agent (A2A) Protocol Specification. Version 0.2.1. [https://google.github.io/A2A/specification/]**

- **Concept**: Standardized agent-to-agent communication protocol for multi-agent interoperability. Defines Agent Cards for capability advertisement and task negotiation.
- **Roko adaptation**: Informs the Agent Mesh wire protocol design.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

---

**Bryan, M. (2024). ERC-8004: Agent Identity. Ethereum Improvement Proposals. [https://eips.ethereum.org/EIPS/eip-8004]**

- **Concept**: Soulbound NFT with capabilities. Non-transferable agent identity token standard on Ethereum.
- **tmp/ cross-refs**: `24-smart-contracts.md`, `08-chain-reputation.md`

---

**Cloudflare / Linux Foundation (2025). x402 Protocol. [https://x402.org]**

- **Concept**: HTTP 402 micropayments. Sub-second USDC settlement enabling self-funding agents via standard HTTP.
- **tmp/ cross-refs**: `24-smart-contracts.md`, `08-chain-reputation.md`

---

## Market Microstructure

Market microstructure studies the mechanics of how markets clear — how prices form, how orders execute, how information flows.

**Documents using this section**: `tmp/08-chain-reputation.md`

---

**Peters, O. (2019). The Ergodicity Problem in Economics. _Nature Physics_, 15, 1216-1221.**
[DOI: 10.1038/s41567-019-0732-0](https://doi.org/10.1038/s41567-019-0732-0)

- **Concept**: The ensemble average of wealth growth differs from the time average in multiplicative processes. Log-wealth maximization is optimal for individual agents in non-ergodic settings.
- **Roko adaptation**: Kelly criterion for routing budget allocation. Agents maximize time-average (not ensemble-average) returns.
- **tmp/ cross-refs**: `08-chain-reputation.md`

---

**Kelly, J.L. Jr. (1956). A New Interpretation of Information Rate. _Bell System Technical Journal_, 35(4), 917-926.**
[DOI: 10.1002/j.1538-7305.1956.tb03809.x](https://doi.org/10.1002/j.1538-7305.1956.tb03809.x)

- **Concept**: Kelly criterion: optimal bet sizing that maximizes the logarithm of wealth. The Kelly fraction (edge/odds) maximizes the geometric growth rate of capital over time.
- **Roko adaptation**: Position sizing in Route decisions and attention budget allocation.
- **tmp/ cross-refs**: `08-chain-reputation.md`

---

**Lo, A.W. (2004). The Adaptive Markets Hypothesis: Market Efficiency from an Evolutionary Perspective. _Journal of Portfolio Management_, 30(5), 15-29.**
[DOI: 10.3905/jpm.2004.442611](https://doi.org/10.3905/jpm.2004.442611)

- **Concept**: Markets are adaptively efficient via evolutionary dynamics. Strategies that worked before may stop working; agents must continuously adapt.
- **tmp/ cross-refs**: `08-chain-reputation.md`

---

**Charnov, E.L. (1976). Optimal Foraging, the Marginal Value Theorem. _Theoretical Population Biology_, 9(2), 129-136.**
[DOI: 10.1016/0040-5809(76)90040-X](https://doi.org/10.1016/0040-5809(76)90040-X)

- **Concept**: Optimal foraging: leave a resource patch when its marginal return falls below the average return available in the environment.
- **Roko adaptation**: Task switching decision — when should the agent stop working on a depleting task and switch to a more promising one?
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

**Taleb, N.N. (2012). _Antifragile: Things That Gain from Disorder_. New York: Random House. ISBN: 978-1400067824.**

- **Concept**: Antifragility: systems with convex response to stressors improve under volatility, beyond mere robustness.
- **Roko adaptation**: Antifragility as design principle. Gate failures should make the system stronger through learning.

---

**Hayek, F.A. (1945). The Use of Knowledge in Society. _American Economic Review_, 35(4), 519-530.**
[JSTOR: 1809376](https://www.jstor.org/stable/1809376)

- **Concept**: Distributed knowledge aggregation. The price system as a distributed information processing mechanism encoding local knowledge into a global signal without any central processor possessing the whole.
- **Roko adaptation**: Pheromone field as a price-like aggregation system for distributed agent knowledge.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

---

**Dawkins, R. (1976). _The Selfish Gene_. Oxford: Oxford University Press. ISBN: 978-0192860927.**

- **Concept**: Memes: units of cultural transmission that replicate, mutate, and are selected in the cultural environment, analogous to genes in the biological environment.
- **Roko adaptation**: Signals are agent-ecosystem memes — units of knowledge that replicate, mutate, and are selected by gate verdict.

---

**Fisher, R.A. (1930). _The Genetical Theory of Natural Selection_. Oxford: Clarendon Press.**

- **Concept**: Fundamental theorem: rate of improvement equals variance in fitness. Diversity drives improvement; homogeneity prevents adaptation.

---

*Navigation: [README](README.md) | [HDC and VSA](hdc-and-vsa.md) | [Memory and Learning](memory-and-learning.md) | [Affect and Cognition](affect-and-cognition.md) | [Verification and Safety](verification-and-safety.md) | [Agents and Orchestration](agents-and-orchestration.md) | [Context and Search](context-and-search.md) | [Math and Statistics](math-and-statistics.md)*

# Academic and Technical References

[Back to overview](./README.md)

All 23 citations used across this document set, with annotations explaining their relevance.

---

[1] J. R. Douceur, "The Sybil Attack," *Proc. 1st International Workshop on Peer-to-Peer Systems (IPTPS)*, 2002. The foundational paper defining the Sybil attack, where a single entity creates multiple pseudonymous identities to subvert a reputation system. Douceur proved that Sybil attacks are always possible in distributed systems without a trusted authority that can certify identities, which directly motivates the soulbound passport approach.

[2] E. G. Weyl, P. Ohlhaver, V. Buterin, "Decentralized Society: Finding Web3's Soul," SSRN, May 2022. [https://papers.ssrn.com/sol3/papers.cfm?abstract_id=4105763](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=4105763). The paper proposing soulbound tokens (SBTs) as non-transferable tokens representing commitments, credentials, and affiliations, encoding the trust networks of the real economy. Section 2 introduces the "Soul" construct as an anchor for reputation and identity.

[3] S. J. Roberts, "Exponential smoothing: The state of the art — Part II," *International Journal of Forecasting*, vol. 20, no. 1, pp. 1-4, 2004. Standard reference for exponential moving average (EMA) / exponential smoothing methods in time-series analysis. The EMA formula `R_new = alpha * F + (1 - alpha) * R_old` is a first-order exponential smoothing filter. The "Part II" survey covers adaptive alpha variants analogous to the job-count-dependent alpha in CHAIN-03.

[4] J. Duchi, E. Hazan, Y. Singer, "Adaptive Subgradient Methods for Online Learning and Stochastic Optimization," *JMLR*, vol. 12, pp. 2121-2159, 2011. The concept of adaptive learning rates (varying step sizes based on accumulated gradient history, as in AdaGrad) which inspired the adaptive alpha approach in the reputation system. The analogy: job count plays the role of accumulated gradient magnitude — agents with more jobs have effectively "seen more data" and need a smaller update rate.

[5] E. Rutherford, "Radioactive Substances and their Radiations," 1913. The half-life decay model `N(t) = N_0 * 0.5^(t / t_half)` originates from nuclear physics but is widely applied in pharmacokinetics, signal processing, and here in reputation decay. The key insight is that half-life gives an intuitive, parameter-free description of the decay rate: "after 30 days, the deviation from neutral halves."

[6] L. Page, S. Brin, R. Motwani, T. Winograd, "The PageRank Citation Ranking: Bringing Order to the Web," Stanford InfoLab Technical Report, 1998. The original PageRank paper describing iterative power iteration with damping/teleportation for computing importance scores on directed graphs. TraceRank adapts this to quality-weighted payment edges: instead of hyperlinks (binary, unweighted), TraceRank edges are weighted by `payment_amount * quality_score`.

[7] S. D. Kamvar, M. T. Schlosser, H. Garcia-Molina, "The EigenTrust Algorithm for Reputation Management in P2P Networks," *Proc. 12th International World Wide Web Conference (WWW '03)*, Budapest, 2003. [https://nlp.stanford.edu/pubs/eigentrust.pdf](https://nlp.stanford.edu/pubs/eigentrust.pdf). Formalized eigenvector-based trust propagation in peer-to-peer networks, computing a global trust value for each peer based on transaction history. The paper also analyzes collusion attacks on EigenTrust (Section 6), showing that coordinated malicious peers can amplify each other's trust — motivating the mutual-ratio-based collusion detection in CHAIN-03. TraceRank is an adaptation of EigenTrust with quality-weighted edges.

[8] O. Perron, "Zur Theorie der Matrices," *Mathematische Annalen*, vol. 64, pp. 248-263, 1907; G. Frobenius, "Ueber Matrizen aus nicht negativen Elementen," *Sitzungsberichte der Preussischen Akademie der Wissenschaften*, pp. 456-477, 1912. The Perron-Frobenius theorem guarantees that a positive stochastic matrix has a unique largest eigenvalue (= 1) with a corresponding positive eigenvector (the stationary distribution) — the mathematical foundation for PageRank/TraceRank convergence. The key conditions are positivity (guaranteed by teleportation), irreducibility (guaranteed by teleportation making all states reachable), and aperiodicity (guaranteed by teleportation breaking cycles).

[9] C. Bron, J. Kerbosch, "Algorithm 457: Finding All Cliques of an Undirected Graph," *Communications of the ACM*, vol. 16, no. 9, pp. 575-577, September 1973. The original algorithm for enumerating all maximal cliques in an undirected graph. The pivoting optimization in the version used here (choosing the pivot vertex that maximizes the number of candidates eliminated) reduces the worst-case complexity from O(2^n) to O(3^(n/3)), which is asymptotically optimal for maximal clique enumeration (Moon & Moser, 1965 [21]).

[10] M. Mitzenmacher, "The Power of Two Choices in Randomized Load Balancing," *IEEE Transactions on Parallel and Distributed Systems*, vol. 12, no. 10, pp. 1094-1104, 2001. The theoretical foundation for the RandomVRF hiring model: sampling two random candidates and picking the better one achieves O(log log N) maximum load, an exponential improvement over purely random assignment's O(log N / log log N). The original result is due to Azar, Broder, Karlin, and Upfal (1994).

[11] W. Vickrey, "Counterspeculation, Auctions, and Competitive Sealed Tenders," *Journal of Finance*, vol. 16, no. 1, pp. 8-37, 1961. The foundational paper on second-price sealed-bid auctions, proving that truthful bidding is the dominant strategy when the winner pays the second-highest bid (i.e., their payment is independent of their own bid). Vickrey received the Nobel Prize in Economics (1996) for this work and the broader theory of incentives in mechanism design.

[12] E. H. Clarke, "Multipart Pricing of Public Goods," *Public Choice*, vol. 11, pp. 17-33, 1971; T. Groves, "Incentives in Teams," *Econometrica*, vol. 41, no. 4, pp. 617-631, 1973. The generalization of Vickrey's second-price mechanism to multi-item settings, forming the VCG (Vickrey-Clarke-Groves) mechanism design framework. The roko BlindAuction's reputation-adjusted price is a single-item VCG variant where the "welfare" being maximized includes both price and reputation quality.

[13] S. Gesell, *Die Naturliche Wirtschaftsordnung durch Freiland und Freigeld* (The Natural Economic Order), 1916. Proposed Freigeld ("free money") bearing a demurrage charge (~5% annually) to discourage hoarding and incentivize circulation. Gesell's insight was that money should behave like perishable goods to maintain economic velocity. Keynes acknowledged the soundness of this insight in *The General Theory of Employment, Interest and Money* (1936), chapter 23. The Worgl Schilling experiment (1932-33) demonstrated the effect empirically: demurrage-bearing local currency achieved dramatically higher velocity during the Great Depression.

[14] R. Fielding et al., "Hypertext Transfer Protocol — HTTP/1.1," RFC 2616, IETF, June 1999. Defined the HTTP 402 "Payment Required" status code, noting it was "reserved for future use." The roko X402 protocol provides the concrete implementation that the RFC anticipated but did not specify. RFC 2616 was later superseded by RFC 7231 (2014), which maintains the 402 reservation unchanged.

[15] Coinbase, "x402: An Open Protocol for HTTP-Native Payments," 2025. [https://docs.cdp.coinbase.com/x402/core-concepts/http-402](https://docs.cdp.coinbase.com/x402/core-concepts/http-402). The production x402 protocol specification enabling machine-to-machine payments via HTTP 402 with stablecoin support across EVM and Solana networks. Coinbase's implementation uses USDC/ERC-3009; the roko implementation uses KORAI/ERC-3009.

[16] P. Becker, "ERC-3009: Transfer With Authorization," Ethereum Improvement Proposals, 2020. [https://github.com/ethereum/EIPs/issues/3010](https://github.com/ethereum/EIPs/issues/3010). The standard for gasless ERC-20 transfers via off-chain signed authorizations (`transferWithAuthorization`), using non-sequential random nonces for concurrency. Implemented natively by USDC (Circle's stablecoin), MATIC, and other tokens. The random nonce design (vs sequential counters) is key for X402: it allows multiple concurrent payment authorizations without coordination between the payer's sessions.

[17] Raiden Network, "Payment Channels and State Channels," [https://raiden.network/101.html](https://raiden.network/101.html). State channels enable off-chain transactions with on-chain dispute resolution, reducing gas costs to 2 transactions per session (open + close). The concept was independently developed as the Lightning Network for Bitcoin (Poon & Dryja, 2016) and the Raiden Network for Ethereum. The strictly increasing nonce mechanism in the roko state channel implementation mirrors Lightning's revocation mechanism and Raiden's balance proof update mechanism.

[18] NEAR Protocol, "NEP-171: Non-Fungible Token Standard," [https://nomicon.io/Standards/Tokens/NonFungibleToken/Core](https://nomicon.io/Standards/Tokens/NonFungibleToken/Core). NEAR's NFT standard (analogous to ERC-721), which provides the foundation for soulbound passport tokens. The NEAR community's discussion of a dedicated soulbound token extension (disabling `nft_transfer` and `nft_transfer_call`) is ongoing at [https://gov.near.org/t/discussion-of-soulbound-token-standard/31223](https://gov.near.org/t/discussion-of-soulbound-token-standard/31223).

[19] NEAR Protocol, "NEP-141: Fungible Token Standard," [https://nomicon.io/Standards/Tokens/FungibleToken/Core](https://nomicon.io/Standards/Tokens/FungibleToken/Core). NEAR's fungible token standard (analogous to ERC-20), which serves as the foundation for the NEAR-native KORAI token implementation. The `ft_transfer` and `ft_transfer_call` interface is analogous to ERC-20's `transfer` and `transferFrom`, but adapted for NEAR's async execution model.

[20] S. Nakamoto, "Bitcoin: A Peer-to-Peer Electronic Cash System," 2008. [https://bitcoin.org/bitcoin.pdf](https://bitcoin.org/bitcoin.pdf). The KORAI emission schedule with halvings is directly inspired by Bitcoin's block reward halving schedule (every 210,000 blocks, approximately 4 years). The terminal rate floor in KORAI (1 KORAI/block) addresses the "incentive cliff" that Bitcoin faces post-2140 when block rewards approach zero, by ensuring perpetual low-level issuance.

[21] J. Moon, L. Moser, "On Cliques in Graphs," *Israel Journal of Mathematics*, vol. 3, pp. 23-28, 1965. Proved the tight bound on the maximum number of maximal cliques in a graph: at most 3^(n/3) for n vertices. This establishes the theoretical worst-case for Bron-Kerbosch and confirms that the O(3^(n/3)) pivoting variant is asymptotically optimal for maximal clique enumeration.

[22] D. Chaum, E. van Heyst, "Group Signatures," *Advances in Cryptology — EUROCRYPT '91*, Lecture Notes in Computer Science, vol. 547, pp. 257-265, 1991. The TEE attestation field in the passport (`tee_attestation`) is conceptually related to Chaum's group signature schemes, which allow members of a group to sign messages anonymously while remaining accountable to a group manager. TEE attestation provides a hardware-rooted guarantee that the agent is running the attested software environment.

[23] A. Back, "Hashcash — A Denial of Service Counter-Measure," 2002. [http://hashcash.org/papers/hashcash.pdf](http://hashcash.org/papers/hashcash.pdf). The concept of proof-of-work as an anti-spam mechanism predates Bitcoin. The KORAI staking requirement for tier promotion serves a similar function to proof-of-work: it imposes a real cost on adversaries who want to create high-tier passports without legitimate reputation.

---

## Navigation

- [README — Overview and architecture](./README.md)
- [Passport System](./passport-system.md)
- [Reputation Scoring](./reputation-scoring.md)
- [Bounty Marketplace](./bounty-marketplace.md)
- [Token Economics](./token-economics.md)
- [NEAR Implementation](./near-implementation.md)
- [Benchmarking](./benchmarking.md)

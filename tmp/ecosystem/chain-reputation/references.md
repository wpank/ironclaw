# References

[Back to overview](./README.md)

These references justify the concepts used in this folder. They are not source-code dependencies, and the design should remain understandable without separate code access. Before implementing against any named protocol, verify the current standard and threat model in the owning project.

1. J. R. Douceur, "The Sybil Attack," IPTPS, 2002. Defines the Sybil problem: one actor can present many identities unless a system introduces a trusted identity authority or meaningful cost.

2. E. G. Weyl, P. Ohlhaver, V. Buterin, "Decentralized Society: Finding Web3's Soul," SSRN, 2022. Introduces soulbound tokens as non-transferable credentials and affiliation records.

3. S. J. Roberts, "Exponential smoothing: The state of the art - Part II," International Journal of Forecasting, 2004. Background for EMA-style score updates.

4. J. Duchi, E. Hazan, Y. Singer, "Adaptive Subgradient Methods for Online Learning and Stochastic Optimization," JMLR, 2011. Useful analogy for adaptive learning rates; the reputation alpha schedule is a policy heuristic, not AdaGrad.

5. E. Rutherford, "Radioactive Substances and their Radiations," 1913. Historical source for half-life decay, reused here as an intuitive time-decay model.

6. L. Page, S. Brin, R. Motwani, T. Winograd, "The PageRank Citation Ranking: Bringing Order to the Web," Stanford InfoLab, 1998. Basis for graph trust propagation.

7. S. D. Kamvar, M. T. Schlosser, H. Garcia-Molina, "The EigenTrust Algorithm for Reputation Management in P2P Networks," WWW, 2003. Reputation propagation and collusion considerations in peer-to-peer networks.

8. O. Perron, "Zur Theorie der Matrices," 1907; G. Frobenius, "Ueber Matrizen aus nicht negativen Elementen," 1912. Mathematical basis for stationary distributions when the required matrix conditions hold.

9. C. Bron, J. Kerbosch, "Algorithm 457: Finding All Cliques of an Undirected Graph," Communications of the ACM, 1973. Clique enumeration algorithm used for suspicious collaboration graphs.

10. M. Mitzenmacher, "The Power of Two Choices in Randomized Load Balancing," IEEE TPDS, 2001. Background for random two-candidate assignment. Applies under random sampling assumptions.

11. W. Vickrey, "Counterspeculation, Auctions, and Competitive Sealed Tenders," Journal of Finance, 1961. Basis for second-price sealed-bid auctions.

12. E. H. Clarke, "Multipart Pricing of Public Goods," 1971; T. Groves, "Incentives in Teams," Econometrica, 1973. General mechanism-design background. Reputation-adjusted marketplace scoring needs its own review before claiming VCG properties.

13. S. Gesell, The Natural Economic Order, 1916. Historical demurrage-currency argument. Use as economic context, not proof that a token design will increase useful activity.

14. R. Fielding et al., HTTP/1.1 RFC 2616, 1999; later HTTP semantics RFCs. HTTP 402 remains a reserved/payment-related status code rather than a complete payment protocol.

15. HTTP 402 and x402-style payment protocol documentation. Useful references for the request/payment/retry pattern; NEAR requires its own authorization design.

16. ERC-3009, "Transfer With Authorization." EVM token authorization model useful for x402-style flows on compatible chains.

17. Raiden Network and Lightning Network materials on payment channels. Background for off-chain signed balance updates with on-chain dispute resolution.

18. NEAR Protocol, NEP-171 Non-Fungible Token Standard. Basis for a NEAR passport representation with transfer methods disabled by policy.

19. NEAR Protocol, NEP-141 Fungible Token Standard. Basis for NEAR token settlement if KORAI is implemented as a fungible token.

20. S. Nakamoto, "Bitcoin: A Peer-to-Peer Electronic Cash System," 2008. Background for halving-style emission schedules. Any application-token emission schedule should be separately simulated.

21. J. Moon, L. Moser, "On Cliques in Graphs," Israel Journal of Mathematics, 1965. Worst-case bound for the number of maximal cliques.

22. D. Chaum, E. van Heyst, "Group Signatures," EUROCRYPT, 1991. Related accountability/anonymity background. TEE attestation is a different mechanism and should not be described as the same guarantee.

23. A. Back, "Hashcash - A Denial of Service Counter-Measure," 2002. Background for making abuse costly. Staking is cost-imposition by capital lockup, not proof-of-work.

## Navigation

- [README](./README.md)
- [Passport System](./passport-system.md)
- [Reputation Scoring](./reputation-scoring.md)
- [Bounty Marketplace](./bounty-marketplace.md)
- [Token Economics](./token-economics.md)
- [NEAR Implementation](./near-implementation.md)
- [Benchmarking](./benchmarking.md)

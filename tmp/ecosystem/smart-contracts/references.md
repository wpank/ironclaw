# References

> Academic and technical citations for the Roko/IronClaw smart contract
> architecture documentation. Covers revm documentation, NEAR SDK docs,
> Solidity security patterns, formal verification, smart contract auditing
> methodologies, EVM vs NEAR comparisons, and cross-chain bridges.

Navigation: [README](./README.md) | [Solidity Contracts](./solidity-contracts.md) | [EVM Simulator](./evm-simulator.md) | [NEAR Contracts](./near-contracts.md) | [Benchmarks](./benchmarking.md) | [IronClaw Integration](./ironclaw-integration.md) | **References**

---

## Primary References (Cited in Documents)

These references are cited by number `[N]` throughout the document suite.

[1] Szabo, N. (1994). "Smart Contracts: Formalizing and Securing
Relationships on Public Networks." *First Monday*, 2(9).
https://ojphi.org/ojs/index.php/fm/article/view/548

> The foundational paper on smart contracts. Szabo introduced the concept
> of embedding contractual clauses in hardware and software to make breach
> of contract expensive. Referenced in [README](./README.md) for the
> definition of smart contracts.

[2] Solidity Documentation. Ethereum Foundation (2024).
https://docs.soliditylang.org/

> Official documentation for the Solidity programming language. The Roko
> contract suite targets Solidity `^0.8.26`. Referenced in
> [README](./README.md) and [solidity-contracts.md](./solidity-contracts.md)
> for language specification.

[3] Wood, G. (2014). "Ethereum: A Secure Decentralised Generalised
Transaction Ledger." Ethereum Yellow Paper.
https://ethereum.github.io/yellowpaper/paper.pdf

> The formal specification of the Ethereum Virtual Machine, including opcode
> definitions, gas costs, and state transition function. Referenced in
> [README](./README.md) for EVM gas cost tables.

[4] NEAR Protocol Documentation. NEAR Foundation (2024).
https://docs.near.org/

> Official documentation for NEAR Protocol, covering the account model,
> storage staking, gas model, and cross-contract calls. Referenced
> throughout [README](./README.md) and
> [near-contracts.md](./near-contracts.md).

[5] ERC-8004: "The Universal Standard for AI Agent Identity." Ethereum
Improvement Proposals (2024).
https://www.chainup.com/blog/erc-8004-ai-agent-identity-standard/

> The proposed standard for AI agent identity on Ethereum, establishing
> three on-chain registries for Identity, Reputation, and Validation.
> Referenced in [README](./README.md) and
> [solidity-contracts.md](./solidity-contracts.md) for the IdentityRegistry
> design.

[6] ERC-5192: "Minimal Soulbound NFTs." Ethereum Improvement Proposals
(2023). Breuer, T.; Fietkau, J.
https://eips.ethereum.org/EIPS/eip-5192

> Defines the `locked()` function for non-transferable NFTs. The
> IdentityRegistry implements this interface to make passports soulbound.
> Referenced in [solidity-contracts.md](./solidity-contracts.md).

[7] "Exponential Moving Averages at Scale: Building Smart Time-Decay
Systems." Open Data Science Conference (2023).
https://odsc.medium.com/exponential-moving-averages-at-scale-building-smart-time-decay-systems-45d7509cd6ae

> Discussion of EMA-based reputation systems with time decay. The
> WorkerRegistry uses alpha=0.2 EMA with halving-based decay. Referenced
> in [README](./README.md) for reputation model design.

[8] Foundry: Ethereum Development Framework. Paradigm (2024).
https://www.getfoundry.sh/

> The Solidity build and test framework used by the Roko contract suite.
> Referenced in [solidity-contracts.md](./solidity-contracts.md) for the
> build system configuration.

[9] revm: Rust implementation of the Ethereum Virtual Machine. bluealloy
(2024).
https://github.com/bluealloy/revm

> The Rust EVM implementation that mirage-rs uses for contract simulation.
> Provides deterministic execution, configurable spec versions, and the
> Inspector framework for tracing. Referenced in
> [evm-simulator.md](./evm-simulator.md).

[10] NEAR Protocol Storage Staking. NEAR Foundation (2024).
https://docs.near.org/protocol/storage/storage-staking

> Documentation of NEAR's storage staking model: 10^19 yoctoNEAR per byte,
> refundable when storage is freed. Referenced in
> [benchmarking.md](./benchmarking.md) and
> [near-contracts.md](./near-contracts.md).

[11] Aurora: Ethereum Compatibility on NEAR. Aurora Labs (2024).
https://docs.near.org/aurora/what-is

> Aurora runs an EVM as a NEAR smart contract, providing an alternative
> deployment path for Solidity contracts with NEAR's finality and fees.
> Referenced in [near-contracts.md](./near-contracts.md).

[12] near-sdk-rs: Rust library for writing NEAR smart contracts. NEAR
Foundation (2024).
https://github.com/near/near-sdk-rs

> The Rust SDK for NEAR smart contracts, providing `#[near]` macros,
> collection types (`UnorderedMap`, `LookupMap`, `TreeMap`), and promise
> APIs. All NEAR contract ports use near-sdk-rs 5.x. Referenced in
> [near-contracts.md](./near-contracts.md).

[13] NEP-141: Fungible Token Standard. NEAR Enhancement Proposals (2022).
https://nomicon.io/Standards/Tokens/FungibleToken/Core

> The NEAR equivalent of ERC-20. Key difference: `ft_transfer_call()`
> atomically transfers and calls a receiver, replacing the `approve` +
> `transferFrom` pattern. Referenced in
> [near-contracts.md](./near-contracts.md) and
> [benchmarking.md](./benchmarking.md).

[14] NEP-171: Non-Fungible Token Standard. NEAR Enhancement Proposals
(2022).
https://nomicon.io/Standards/Tokens/NonFungibleToken/Core

> The NEAR equivalent of ERC-721. Referenced in
> [near-contracts.md](./near-contracts.md) for the soulbound passport
> comparison.

[15] Nakamoto, S. (2008). "Bitcoin: A Peer-to-Peer Electronic Cash
System."
https://bitcoin.org/bitcoin.pdf

> The original blockchain paper. Referenced for historical context in
> blockchain-based coordination systems.

[16] Buterin, V. (2014). "A Next-Generation Smart Contract and
Decentralized Application Platform." Ethereum Whitepaper.
https://ethereum.org/en/whitepaper/

> The Ethereum whitepaper describing smart contracts as a general-purpose
> computation layer on blockchain. Referenced for the EVM design rationale.

[17] NEAR Nightshade Sharding Design. NEAR Foundation (2023).
https://near.org/blog/near-launches-nightshade-sharding-paving-the-way-for-mass-adoption

> Documentation of NEAR's Nightshade sharding protocol, which enables
> parallel transaction processing and sub-second finality. Referenced in
> [benchmarking.md](./benchmarking.md) for throughput comparison.

[18] Roko Contract Repository. Pank, W. (2024).
https://github.com/wpank/roko

> The source repository for all 13 Solidity contracts, mirage-rs simulator,
> roko-chain-watcher, and roko-chain integration crate. Referenced
> throughout all documents.

---

## revm Documentation and Resources

[19] revm Book: Architecture and Usage Guide (2024).
https://bluealloy.github.io/revm/

> Comprehensive guide to revm's architecture, including the Database trait,
> Inspector framework, and EVM builder pattern. Essential reading for
> understanding mirage-rs's simulation layer.

[20] revm: Inspector trait documentation. bluealloy (2024).
https://github.com/bluealloy/revm/blob/main/crates/interpreter/src/inspector.rs

> Source code for the Inspector trait that enables call tracing, storage
> monitoring, and gas profiling in mirage-rs's trace analysis.

[21] EIP-2929: "Gas cost increases for state access opcodes." Buterin, V.;
Swende, M. (2020).
https://eips.ethereum.org/EIPS/eip-2929

> Introduces warm/cold access pricing for SLOAD and CALL opcodes.
> mirage-rs's gas estimation must account for EIP-2929 to produce
> accurate results.

[22] EIP-2930: "Optional access lists." Buterin, V.; Swende, M. (2021).
https://eips.ethereum.org/EIPS/eip-2930

> Access lists reduce gas costs by pre-declaring which storage slots a
> transaction will touch. Relevant for optimizing mirage-rs simulation
> of complex multi-contract calls.

---

## NEAR SDK Documentation

[23] near-sdk-rs API Documentation (2024).
https://docs.rs/near-sdk/latest/near_sdk/

> API reference for near-sdk-rs 5.x, including collection types,
> environment functions, promise APIs, and contract macros.

[24] NEAR Smart Contract Development Guide. NEAR Foundation (2024).
https://docs.near.org/build/smart-contracts/what-is

> Tutorial-style guide for building NEAR smart contracts in Rust,
> covering storage management, cross-contract calls, and testing.

[25] near-workspaces-rs: Testing framework for NEAR contracts (2024).
https://github.com/near/near-workspaces-rs

> The integration testing framework used for NEAR contract ports. Provides
> sandbox node management, contract deployment, and transaction simulation.

[26] NEP-297: Events Standard. NEAR Enhancement Proposals (2022).
https://nomicon.io/Standards/EventsFormat

> The JSON-based event standard for NEAR contracts, used as the equivalent
> of Solidity events. Enables indexer-friendly structured logging.

[27] NEAR JSON-RPC API Documentation. NEAR Foundation (2024).
https://docs.near.org/api/rpc/introduction

> RPC API specification for NEAR nodes, covering transaction broadcast,
> state queries, and access key management. Used by the IronClaw chain
> tool for all NEAR interactions.

[28] near-jsonrpc-client-rs: Rust client for NEAR RPC (2024).
https://github.com/near/near-jsonrpc-client-rs

> The Rust RPC client library used by IronClaw's chain tool for NEAR
> blockchain interactions.

---

## Solidity Security Patterns

[29] OpenZeppelin Contracts: Secure smart contract library (2024).
https://github.com/OpenZeppelin/openzeppelin-contracts

> The Roko contract suite uses OpenZeppelin's ERC20 implementation and
> IERC20 interface. The contracts intentionally do not use OpenZeppelin's
> `AccessControl` or `ReentrancyGuard`, preferring lighter-weight
> alternatives.

[30] Atzei, N.; Bartoletti, M.; Cimoli, T. (2017). "A Survey of Attacks
on Ethereum Smart Contracts (SoK)." *Proceedings of the 6th International
Conference on Principles of Security and Trust*, pp. 164-186.
https://doi.org/10.1007/978-3-662-54455-6_8

> Systematic taxonomy of smart contract vulnerabilities including
> reentrancy, integer overflow, and transaction ordering dependence.
> Relevant to the security analysis in
> [ironclaw-integration.md](./ironclaw-integration.md).

[31] Wohrer, M.; Zdun, U. (2018). "Smart Contracts: Security Patterns
and Best Practices." *2018 International Workshop on Blockchain Oriented
Software Engineering (IWBOSE)*, pp. 2-8.
https://doi.org/10.1109/IWBOSE.2018.8327565

> Catalog of security patterns for smart contracts: checks-effects-
> interactions, pull-over-push, emergency stop, access restriction.
> The Roko contracts follow checks-effects-interactions but do not use
> explicit reentrancy guards.

[32] Chen, T.; Li, X.; Luo, X.; Zhang, X. (2017). "Under-optimized Smart
Contracts Devour Your Money." *2017 IEEE 24th International Conference on
Software Analysis, Evolution and Reengineering (SANER)*.
https://doi.org/10.1109/SANER.2017.7884650

> Analysis of gas optimization patterns in Solidity. Relevant to the
> gas benchmarks in [benchmarking.md](./benchmarking.md).

[33] Trail of Bits: "Building Secure Smart Contracts." (2024).
https://github.com/crytic/building-secure-contracts

> Comprehensive guide to smart contract security from Trail of Bits,
> covering testing, fuzzing, and formal verification. Foundry's fuzz
> testing (256 runs in the Roko config) follows this methodology.

---

## Formal Verification and Auditing

[34] Hildenbrandt, E.; Saxena, M.; Rodrigues, N.; Zhu, X.; Daian, P.;
Guth, D.; Moore, B.; Park, D.; Zhang, Y.; Stefanescu, A.; Rosu, G.
(2018). "KEVM: A Complete Formal Semantics of the Ethereum Virtual
Machine." *2018 IEEE 31st Computer Security Foundations Symposium (CSF)*,
pp. 204-217.
https://doi.org/10.1109/CSF.2018.00022

> Formal semantics of the EVM in the K framework. KEVM enables formal
> verification of EVM bytecode, which could be applied to the Roko
> contract suite to prove invariants like "bounty funds cannot be lost."

[35] Bhargavan, K.; Delignat-Lavaud, A.; Fournet, C.; Gollamudi, A.;
Gonthier, G.; Kobeissi, N.; Kulatova, N.; Rastogi, A.; Sibut-Pinote,
T.; Swamy, N.; Zanella-Beguelin, S. (2016). "Formal Verification of
Smart Contracts: Short Paper." *Proceedings of the 2016 ACM Workshop on
Programming Languages and Analysis for Security*, pp. 91-96.
https://doi.org/10.1145/2993600.2993611

> Early work on formal verification of smart contracts using F*. Proposes
> translating Solidity to F* for automated verification.

[36] Luu, L.; Chu, D.H.; Olickel, H.; Saxena, P.; Hobor, A. (2016).
"Making Smart Contracts Smarter." *Proceedings of the 2016 ACM SIGSAC
Conference on Computer and Communications Security*, pp. 254-269.
https://doi.org/10.1145/2976749.2978309

> Introduces Oyente, one of the first symbolic execution tools for
> Ethereum contracts. Detects reentrancy, timestamp dependence, and
> mishandled exceptions.

[37] Slither: Solidity Static Analysis Framework. Trail of Bits (2024).
https://github.com/crytic/slither

> Static analysis tool for Solidity that detects common vulnerabilities.
> Recommended for pre-audit scanning of the Roko contract suite.

[38] Echidna: Smart Contract Fuzzer. Trail of Bits (2024).
https://github.com/crytic/echidna

> Property-based fuzzer for Solidity contracts. Can be used alongside
> Foundry's built-in fuzzing (configured with 256 runs in `foundry.toml`)
> for deeper coverage.

---

## EVM vs NEAR Comparisons

[39] NEAR vs Ethereum: A Technical Comparison. NEAR Foundation (2023).
https://docs.near.org/concepts/web3/near-and-eth

> Official comparison of NEAR and Ethereum architectures, covering
> consensus, account models, and developer experience.

[40] Doomslug: NEAR's Block Production Consensus. NEAR Foundation (2020).
https://near.org/blog/doomslug-comparison

> Technical description of NEAR's block production algorithm, which
> achieves ~1-second block times with deterministic finality in 2-3
> seconds. Referenced in [benchmarking.md](./benchmarking.md).

[41] EIP-1967: Proxy Storage Slots. Santiago Palladino (2019).
https://eips.ethereum.org/EIPS/eip-1967

> Standardized storage slots for transparent proxy contracts. NEAR's
> native upgrade model (deploy to same account) eliminates the need
> for proxy patterns.

---

## Cross-Chain Bridges

[42] Rainbow Bridge: NEAR-Ethereum Trustless Bridge. NEAR Foundation
(2024).
https://docs.near.org/concepts/web3/bridges

> The trustless bridge between NEAR and Ethereum that enables asset
> transfers. Relevant for the Aurora deployment path described in
> [near-contracts.md](./near-contracts.md).

[43] Robinson, D.; Konstantopoulos, G. (2020). "Ethereum is a Dark
Forest." *Paradigm Research*.
https://www.paradigm.xyz/2020/08/ethereum-is-a-dark-forest

> Description of front-running and MEV on Ethereum. Relevant to the
> security considerations in
> [ironclaw-integration.md](./ironclaw-integration.md) regarding
> simulation result confidentiality.

---

## Agent Identity and Coordination Standards

[44] Weiss, G. (1999). *Multiagent Systems: A Modern Approach to
Distributed Artificial Intelligence*. MIT Press.
https://mitpress.mit.edu/9780262731317/

> Foundational text on multi-agent systems covering coordination,
> negotiation, and reputation. The Roko contract suite implements many
> of the patterns described here in on-chain form.

[45] ERC-4337: "Account Abstraction Using an Alt Mempool." Ethereum
Improvement Proposals (2023).
https://eips.ethereum.org/EIPS/eip-4337

> Account abstraction for Ethereum, enabling smart contract wallets that
> could simplify agent key management. Future integration possibility
> for IronClaw's chain tool.

[46] Model Context Protocol (MCP) Specification. Anthropic (2024).
https://spec.modelcontextprotocol.io/

> The agent card URI field in the IdentityRegistry stores an MCP-compliant
> agent descriptor URL, enabling agent-to-agent discovery. Referenced in
> [solidity-contracts.md](./solidity-contracts.md).

---

## Roko Source References

All Roko source code is available at
[https://github.com/wpank/roko](https://github.com/wpank/roko). Key
paths referenced in this documentation:

| Path | Description | Referenced In |
|------|-------------|---------------|
| [`contracts/src/`](https://github.com/wpank/roko/blob/main/contracts/src/) | All 13 Solidity contracts | [solidity-contracts.md](./solidity-contracts.md) |
| [`contracts/foundry.toml`](https://github.com/wpank/roko/blob/main/contracts/foundry.toml) | Foundry build configuration | [solidity-contracts.md](./solidity-contracts.md), [evm-simulator.md](./evm-simulator.md) |
| [`contracts/test/`](https://github.com/wpank/roko/blob/main/contracts/test/) | Foundry test files | [solidity-contracts.md](./solidity-contracts.md) |
| [`apps/mirage-rs/`](https://github.com/wpank/roko/blob/main/apps/mirage-rs/) | EVM simulator | [evm-simulator.md](./evm-simulator.md) |
| [`apps/mirage-rs/src/main.rs`](https://github.com/wpank/roko/blob/main/apps/mirage-rs/src/main.rs) | Simulator entry point + ERC-8004 bootstrap | [evm-simulator.md](./evm-simulator.md) |
| [`apps/mirage-rs/src/precompiles/hdc.rs`](https://github.com/wpank/roko/blob/main/apps/mirage-rs/src/precompiles/hdc.rs) | HDC precompile at 0xA0C | [evm-simulator.md](./evm-simulator.md) |
| [`apps/mirage-rs/src/chain/knowledge.rs`](https://github.com/wpank/roko/blob/main/apps/mirage-rs/src/chain/knowledge.rs) | Knowledge layer integration | [evm-simulator.md](./evm-simulator.md) |
| [`apps/roko-chain-watcher/`](https://github.com/wpank/roko/blob/main/apps/roko-chain-watcher/) | Chain watcher agent | [evm-simulator.md](./evm-simulator.md) |
| [`apps/roko-chain-watcher/src/reactions.rs`](https://github.com/wpank/roko/blob/main/apps/roko-chain-watcher/src/reactions.rs) | 9 reaction rules | [evm-simulator.md](./evm-simulator.md) |
| [`apps/roko-chain-watcher/src/block_observer.rs`](https://github.com/wpank/roko/blob/main/apps/roko-chain-watcher/src/block_observer.rs) | Block analysis rules | [evm-simulator.md](./evm-simulator.md) |
| [`crates/roko-chain/`](https://github.com/wpank/roko/blob/main/crates/roko-chain/) | Rust-side chain integration | [evm-simulator.md](./evm-simulator.md) |
| [`crates/roko-chain/src/client.rs`](https://github.com/wpank/roko/blob/main/crates/roko-chain/src/client.rs) | ChainClient trait | [evm-simulator.md](./evm-simulator.md) |
| [`crates/roko-chain/src/agent_registry.rs`](https://github.com/wpank/roko/blob/main/crates/roko-chain/src/agent_registry.rs) | Rust AgentRegistry | [evm-simulator.md](./evm-simulator.md) |
| [`crates/roko-chain/src/block_watcher.rs`](https://github.com/wpank/roko/blob/main/crates/roko-chain/src/block_watcher.rs) | Block watcher | [evm-simulator.md](./evm-simulator.md) |
| [`contracts/script/Deploy.s.sol`](https://github.com/wpank/roko/blob/main/contracts/script/Deploy.s.sol) | Deployment script | [solidity-contracts.md](./solidity-contracts.md) |

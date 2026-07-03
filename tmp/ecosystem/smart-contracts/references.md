# References

Status: curated references for the NEAR-native smart contract notes in this
folder. Links were checked for relevance while revising this folder on
2026-07-03.

Navigation: [README](./README.md) |
[NEAR Contracts](./near-contracts.md) |
[IronClaw Integration](./ironclaw-integration.md) | **References**

## NEAR Protocol and Runtime

[1] NEAR Protocol Documentation.
https://docs.near.org/

Primary entry point for NEAR account model, transactions, gas, contract
development, and RPC documentation.

[2] NEAR storage staking documentation.
https://docs.near.org/protocol/storage/storage-staking

Explains storage staking and the network storage price. Use the live protocol
config or RPC data for production estimates rather than hardcoding assumptions
in contract code.

[3] NEAR access keys.
https://docs.near.org/protocol/accounts-contracts/access-keys

Describes full-access keys and function-call access keys. The IronClaw
integration should use function-call keys for routine agent transactions.

[4] NEAR JSON-RPC API.
https://docs.near.org/api/rpc/introduction

Reference for programmatic chain access. Wrap RPC calls behind a small local
client so request-method changes do not spread across the codebase.

## Contract SDK and Standards

[5] `near-sdk-rs` API documentation.
https://docs.rs/near-sdk/latest/near_sdk/

Rust SDK for NEAR contracts, including contract macros, environment helpers,
collections, promises, and testing utilities.

[6] `near-sdk-rs` repository.
https://github.com/near/near-sdk-rs

Source repository for the Rust SDK and examples.

[7] NEP-141 fungible token standard.
https://github.com/near/NEPs/blob/master/neps/nep-0141.md

Defines NEAR fungible token interfaces, including `ft_transfer_call` and the
receiver callback pattern used for bonds and bounties.

[8] NEP-145 storage management standard.
https://github.com/near/NEPs/blob/master/neps/nep-0145.md

Relevant when token contracts or application contracts require users to fund
their storage footprint explicitly.

[9] NEP-171 non-fungible token standard.
https://github.com/near/NEPs/blob/master/neps/nep-0171.md

Useful if a future identity/passport contract needs an NFT-compatible public
interface. Do not expose transfer methods for soulbound credentials.

[10] NEP-297 events standard.
https://github.com/near/NEPs/blob/master/neps/nep-0297.md

Defines the `EVENT_JSON:` log convention used by indexers and recommended for
contract events in these examples.

## Testing and Tooling

[11] `near-workspaces-rs`.
https://github.com/near/near-workspaces-rs

Rust library for sandbox-based NEAR contract tests. Use it for lifecycle tests,
simulation-before-execution checks, and gas/storage measurements.

[12] `near-jsonrpc-client-rs`.
https://github.com/near/near-jsonrpc-client-rs

Rust client for NEAR RPC. Suitable for an IronClaw chain tool wrapper, while
keeping RPC details isolated from product logic.

[13] NEAR CLI.
https://github.com/near/near-cli-rs

Command-line tooling for account, contract, and transaction workflows. Useful
for testnet deployment and manual verification.

## Security and Operations

[14] Trail of Bits, "Building Secure Smart Contracts."
https://github.com/crytic/building-secure-contracts

General smart contract security guidance. Apply the testing and review mindset,
while adapting examples to NEAR's asynchronous promise model.

[15] Wohrer, M.; Zdun, U. (2018). "Smart Contracts: Security Patterns and Best
Practices."
https://doi.org/10.1109/IWBOSE.2018.8327565

Catalog of common security patterns such as access restriction, pull payments,
and emergency stops. Relevant to payout, withdrawal, and admin paths.

[16] NEAR smart contract security introduction.
https://docs.near.org/smart-contracts/security/introduction

NEAR-specific overview of contract security concepts, including callbacks,
front-running, Sybil resistance, reentrancy, account ownership, randomness, and
small-deposit attacks.

[17] NEAR cross-contract callback security.
https://docs.near.org/smart-contracts/security/callbacks

Guidance for handling asynchronous cross-contract calls and callbacks. This is
directly relevant to bounty payout and reputation-update flows.

## Agent Interoperability

[18] Model Context Protocol specification.
https://modelcontextprotocol.io/specification

Potential format for an agent descriptor URI referenced by an identity
registry. The contract should store only a URI/hash, not the descriptor body.

[19] NEAR Enhancement Proposals index.
https://near.github.io/NEPs/

Canonical index for NEAR protocol and standard proposals. Prefer this index
when adding a new token, event, account, or storage standard reference.

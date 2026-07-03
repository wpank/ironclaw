# Token Economics and HTTP 402 Payments

[Back to overview](./README.md)

This design can use an application token for rewards, stake, and marketplace settlement, but IronClaw should not depend on a new token for the first implementation slice. Start with accounting interfaces that can later be backed by native NEAR, a NEP-141 token, or a hosted payment adapter.

## Token Policy

Candidate token roles:

- pay bounties and tool-call fees,
- stake into passport tiers or reputation domains,
- fund dispute bonds,
- reward validation and knowledge contributions,
- collect marketplace fees.

Keep those roles separate in code. A balance, a stake, an escrow deposit, and a dispute bond have different withdrawal and slash rules.

## Demurrage

Demurrage is a holding cost applied to idle balances:

```text
effective_balance = stored_balance * (1 - annual_rate)^(elapsed / seconds_per_year)
```

An example policy might use a low annual rate such as 1%. Treat any demurrage rate as an economic experiment, not a guarantee that circulation improves. Validate against simulated holder behavior and actual marketplace volume before enabling it for real funds.

```rust
pub fn effective_balance(stored: u128, elapsed_secs: u64, annual_rate_bps: u32) -> u128 {
    let years = elapsed_secs as f64 / 31_557_600.0;
    let rate = annual_rate_bps as f64 / 10_000.0;
    (stored as f64 * (1.0 - rate).powf(years)).floor() as u128
}
```

Contract implementation should avoid floating point. Use fixed-point math, cap loops, and materialize decay on transfer, stake, or withdrawal.

Open policy questions:

- Does demurrage apply to escrowed funds?
- Does it apply to staked funds?
- Who receives demurrage: nobody, treasury, validators, or fee rebates?
- How is total supply reported: stored supply or effective supply?

Answer these before contract design. Each answer changes accounting invariants.

## Emission Schedule

A halving schedule is easy to explain, but it can overpay early participants or starve later incentives. Use it only as a candidate:

```rust
pub struct EmissionPolicy {
    pub initial_rate_per_epoch: u128,
    pub epoch_blocks: u64,
    pub min_rate_per_epoch: u128,
    pub max_supply: u128,
}

pub fn emission_rate(policy: &EmissionPolicy, epoch: u64) -> u128 {
    let halvings = epoch.min(64);
    let decayed = policy.initial_rate_per_epoch >> halvings;
    decayed.max(policy.min_rate_per_epoch)
}
```

Validation targets:

- projected supply under realistic block times,
- inflation paid to useful work versus passive farming,
- treasury solvency under low marketplace volume,
- stake thresholds as a percentage of circulating supply,
- attack cost for Sybil registration and collusion.

## HTTP 402 Payment Flow

HTTP 402 is useful as an interaction pattern: request, quote, signed payment authorization, retry.

```text
1. Client calls a priced endpoint.
2. Server replies 402 with amount, asset, recipient, nonce, expiry, and reason.
3. Client signs or submits a payment authorization.
4. Client retries with payment proof.
5. Server verifies proof and serves the response.
```

Architecture-level structs:

```rust
pub struct PaymentQuote {
    pub recipient: String,
    pub asset: String,
    pub amount: u128,
    pub nonce: [u8; 32],
    pub expires_at_unix: u64,
    pub reason: String,
}

pub struct PaymentProof {
    pub quote_nonce: [u8; 32],
    pub payer: String,
    pub signature_or_tx: Vec<u8>,
}
```

On EVM, ERC-3009-style authorizations are one reference pattern. On NEAR, use NEAR-native primitives instead: function-call access keys with strict allowance, NEP-141 `ft_transfer_call`, signed intents if available, or a payment adapter that settles and returns a verifiable receipt. Do not copy EVM payment assumptions into NEAR without a protocol-specific threat model.

## Verification Rules

Before serving paid work, verify:

- quote nonce is known and unused,
- quote has not expired,
- amount and asset match,
- recipient matches,
- payer is authorized for the request,
- signature, transaction, or adapter receipt is valid,
- the quote is bound to the endpoint, method, and request hash.

Nonce storage must be durable enough to prevent replay across process restarts.

## State Channels

State channels can reduce on-chain settlement frequency for repeated interactions, but "two on-chain transactions" is only the cooperative happy path. Disputes, top-ups, challenge responses, and timeout closes add transactions.

Use channels only after direct payment flows work:

```rust
pub struct ChannelState {
    pub channel_id: [u8; 32],
    pub party_a: String,
    pub party_b: String,
    pub total_deposit: u128,
    pub balance_a: u128,
    pub balance_b: u128,
    pub nonce: u64,
    pub challenge_deadline: Option<u64>,
}
```

Invariant:

```rust
fn balances_conserve(state: &ChannelState) -> bool {
    state.balance_a + state.balance_b == state.total_deposit
}
```

Strictly increasing nonces are required so the settlement contract can reject stale balance proofs.

## IronClaw Integration

For MCP or tool calls, the payment layer should be a wrapper around outbound requests:

1. Send the request through existing outbound policy.
2. If the server returns a payment challenge, evaluate cost and approval policy.
3. Sign or submit payment using a scoped key or facilitator.
4. Retry with proof.
5. Record the payment event for reputation, budgeting, and audit.

The payment path must not bypass IronClaw bearer tokens, OAuth, outbound allowlists, approvals, or sandbox policy.

## Navigation

- [Passport System](./passport-system.md)
- [Reputation Scoring](./reputation-scoring.md)
- [Bounty Marketplace](./bounty-marketplace.md)
- [NEAR Implementation](./near-implementation.md)
- [Benchmarking](./benchmarking.md)
- [References](./references.md)

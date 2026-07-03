# Token Economics: KORAI, X402, and ISFR Oracle

[Back to overview](./README.md)

**Source files**:
- [`crates/roko-chain/src/korai_token.rs`](https://github.com/wpank/roko/blob/main/crates/roko-chain/src/korai_token.rs) (657 lines)
- [`crates/roko-chain/src/x402.rs`](https://github.com/wpank/roko/blob/main/crates/roko-chain/src/x402.rs) (958 lines)
- [`crates/roko-chain/src/isfr.rs`](https://github.com/wpank/roko/blob/main/crates/roko-chain/src/isfr.rs) (1277 lines)
- **Spec**: [`docs/v1/08-chain/02-korai-token-economics.md`](https://github.com/wpank/roko/blob/main/docs/v1/08-chain/02-korai-token-economics.md), [`docs/v1/14-identity-economy/10-korai-tokenomics.md`](https://github.com/wpank/roko/blob/main/docs/v1/14-identity-economy/10-korai-tokenomics.md)

---

## KORAI Token Economics (CHAIN-01)

### 1% Annual Lazy Demurrage

KORAI has a distinctive property: **demurrage**. Token balances decay by 1% per year. This is a holding cost that discourages hoarding and encourages circulation.

The concept of demurrage currency was first proposed by Silvio Gesell in 1916 [13]. KORAI applies this concept digitally: tokens actively used in the marketplace (paying for jobs, staking on domains) avoid effective demurrage because their balance is constantly updated. Only idle balances decay.

The demurrage formula:

```
effective_balance = stored_balance * (1 - annual_rate) ^ (elapsed_seconds / seconds_per_year)
```

Where:
- `annual_rate` = 0.01 (1%)
- `seconds_per_year` = 365.25 * 24 * 3600 = 31,557,600

**Concrete examples**:
- After 1 year: `10,000 * 0.99^1 = 9,900` (loss of 100 KORAI)
- After 10 years: `100,000 * 0.99^10 = 90,438` (loss of 9,562 KORAI)
- After 100 years: `100,000 * 0.99^100 = 36,603` (63.4% lost to demurrage)

```rust
const SECONDS_PER_YEAR: f64 = 365.25 * 24.0 * 3600.0;
const DEFAULT_DEMURRAGE_RATE: f64 = 0.01;

pub fn effective_balance(&self, now: u64, annual_rate: f64) -> u256 {
    if now <= self.last_update || self.stored_balance == 0 {
        return self.stored_balance;
    }
    let elapsed = (now - self.last_update) as f64;
    let decay_factor = (1.0 - annual_rate).powf(elapsed / SECONDS_PER_YEAR);
    (self.stored_balance as f64 * decay_factor) as u256
}

pub fn mint(&mut self, to: &str, amount: u256, pathway: EarningPathway, now: u64) {
    let entry = self.balances.entry(to.to_string())
        .or_insert_with(|| BalanceRecord::new(0, now));
    // Materialise existing demurrage before adding new tokens
    entry.materialise_demurrage(now, self.config.demurrage_rate);
    entry.stored_balance = entry.stored_balance.saturating_add(amount);
    entry.last_update = now;
    self.earning_records.push(EarningRecord { to: to.to_string(), amount, pathway, timestamp: now });
}
```

**Source**: [`crates/roko-chain/src/korai_token.rs`](https://github.com/wpank/roko/blob/main/crates/roko-chain/src/korai_token.rs) lines 1-323

### Five Earning Pathways

```rust
pub enum EarningPathway {
    TaskCompletion,            // Completing marketplace jobs
    KnowledgeContribution,     // Submitting knowledge entries
    ValidationParticipation,   // Participating in validation/review
    ReputationStaking,         // Staking on reputation domains
    MarketplaceFees,           // Platform fee revenue share
}
```

### Five Spending Mechanisms

```rust
pub enum SpendingMechanism {
    ComputePurchase,           // Buying LLM compute time
    KnowledgeAccess,           // Accessing gated knowledge
    JobPosting,                // Posting bounties (budget goes to escrow)
    EscrowDeposit,             // Direct escrow deposits
    GovernanceParticipation,   // Governance voting (stake-weighted)
}
```

### Emission Schedule with Halving Epochs

KORAI has a minting schedule inspired by Bitcoin's halvings [20]:

```rust
pub struct EmissionSchedule {
    pub base_emission_per_block: f64,  // 100 KORAI/block initial
    pub blocks_per_epoch: u64,         // 2,628,000 (~1 year at 12s blocks)
    pub terminal_rate: f64,            // 1 KORAI/block floor
    pub max_supply: f64,               // 1 billion KORAI cap
    pub total_minted: f64,
}

pub fn rate_at_block(&self, block: u64) -> f64 {
    if self.total_minted >= self.max_supply { return 0.0; }
    let epoch = self.epoch_for_block(block);
    let halving_factor = 0.5_f64.powi(epoch as i32);
    let rate = self.base_emission_per_block * halving_factor;
    rate.max(self.terminal_rate)
}
```

Emission per epoch:
- Epoch 0 (year 1): 100 KORAI/block × 2,628,000 blocks = 262,800,000 KORAI
- Epoch 1 (year 2): 50 KORAI/block × 2,628,000 blocks = 131,400,000 KORAI
- Epoch 2 (year 3): 25 KORAI/block × 2,628,000 blocks = 65,700,000 KORAI
- Epoch N: max(100/2^N, 1) KORAI/block

The terminal rate of 1 KORAI/block ensures perpetual low-level incentives, avoiding the "incentive cliff" problem where validators lose motivation once block rewards approach zero.

**Testnet variant**: DAEJI token (same economics, different name/symbol, `KoraiTokenConfig::testnet()`).

**Source**: [`crates/roko-chain/src/korai_token.rs`](https://github.com/wpank/roko/blob/main/crates/roko-chain/src/korai_token.rs) lines 433-657

---

## X402 Micropayments Protocol (CHAIN-08)

### Protocol Flow

X402 enables agent-to-agent payments at the speed of HTTP, using the long-dormant HTTP 402 status code [14]. The HTTP 402 "Payment Required" status was reserved in RFC 2616 (1999) for future micropayment use. The x402 protocol, pioneered by Coinbase in 2025 [15], provides the concrete implementation.

```
1. Client sends request to agent's HTTP endpoint
   POST /tools/call
   Content-Type: application/json
   {"tool": "security_audit", "params": {...}}

2. Agent responds with 402 Payment Required
   HTTP/1.1 402 Payment Required
   X-Payment-Request: {
     "recipient": "0xAgent123...",
     "amount": 500,
     "token": "0xKORAI...",
     "nonce": "0xabc...",
     "deadline": 1751234567,
     "reason": "Security audit service fee"
   }

3. Client signs an ERC-3009 transferWithAuthorization (gasless, off-chain)

4. Client retries with payment header
   POST /tools/call
   X-Payment-Authorization: {
     "from": "0xClient...",
     "to": "0xAgent123...",
     "value": 500,
     "validAfter": 1751234000,
     "validBefore": 1751234600,
     "nonce": "0xabc...",
     "v": 28, "r": "0x...", "s": "0x..."
   }

5. Agent verifies authorization and serves the response
   HTTP/1.1 200 OK
   X-Payment-Confirmed: true
```

### ERC-3009: Gasless Transfers

The payment authorization uses ERC-3009 (`transferWithAuthorization`) [16]:

- **Atomic**: Unlike ERC-2612 (permit) which only authorizes approval, ERC-3009 authorizes the complete transfer in one step
- **Non-sequential nonces**: Uses random `bytes32` nonces rather than sequential counters, allowing concurrent independent authorizations
- **Time-bounded**: Each authorization has `validAfter` and `validBefore` timestamps

### Payment Structs

```rust
pub struct PaymentRequest {
    pub recipient: Address,
    pub amount: u256,
    pub token: Address,
    pub nonce: u256,          // Replay protection (random bytes32)
    pub deadline: u64,
    pub reason: String,
}

pub struct PaymentAuthorization {
    pub from: Address,
    pub to: Address,
    pub value: u256,
    pub valid_after: u64,
    pub valid_before: u64,
    pub nonce: u256,
    pub v: u8,
    pub r: [u8; 32],
    pub s: [u8; 32],
}
```

### Verification

The manager verifies authorizations against five criteria:

```rust
pub fn verify_authorization(
    &self,
    request: &PaymentRequest,
    auth: &PaymentAuthorization,
) -> VerificationStatus {
    // 1. Check nonce not reused
    if self.used_nonces.contains(&auth.nonce) {
        return VerificationStatus::NonceReused;
    }
    // 2. Check amount >= requested
    if auth.value < request.amount {
        return VerificationStatus::InsufficientAmount {
            required: request.amount,
            provided: auth.value,
        };
    }
    // 3. Check recipient matches
    if auth.to != request.recipient {
        return VerificationStatus::RecipientMismatch;
    }
    // 4. Check timestamp within valid window
    let now = current_timestamp();
    if now < auth.valid_after || now > auth.valid_before {
        return VerificationStatus::OutsideValidWindow;
    }
    // 5. ECDSA signature verification via ecrecover
    VerificationStatus::Valid
}
```

### State Channels for High-Frequency Interactions

For high-frequency interactions, X402 supports **state channels** [17] that reduce gas to exactly 2 transactions per session (open + close):

```rust
pub struct StateChannel {
    pub channel_id: [u8; 32],
    pub party_a: Address,
    pub party_a_passport: u256,
    pub party_b: Address,
    pub party_b_passport: u256,
    pub deposit_a: u256,
    pub deposit_b: u256,
    pub nonce: u64,              // Increments with each off-chain update
    pub balance_a: u256,
    pub balance_b: u256,
    pub state: ChannelLifecycle, // Open -> Closing -> Closed
    pub challenge_window: u64,   // Blocks to challenge before close (default: 100)
}
```

Channel lifecycle:
1. **Open**: Both parties deposit funds. Off-chain balance proofs track micropayments.
2. **Closing**: Either party requests close. Challenge window starts (default 100 blocks).
3. **Closed**: After challenge period, final balances settled on-chain.

Off-chain updates use signed balance proofs with a **conservation invariant**:

```rust
pub fn update_channel(&mut self, proof: &BalanceProof) -> Result<(), X402Error> {
    let channel = self.channels.get_mut(&proof.channel_id)
        .ok_or(X402Error::ChannelNotFound)?;
    if proof.nonce <= channel.nonce {
        return Err(X402Error::InvalidNonce { current: channel.nonce, provided: proof.nonce });
    }
    // Conservation: sum of balances must equal total deposits
    let total_deposit = channel.deposit_a + channel.deposit_b;
    let total_balance = proof.balance_a + proof.balance_b;
    if total_balance != total_deposit {
        return Err(X402Error::BalanceMismatch { total_deposit, total_balance });
    }
    channel.nonce = proof.nonce;
    channel.balance_a = proof.balance_a;
    channel.balance_b = proof.balance_b;
    Ok(())
}
```

The strictly increasing nonce ensures the on-chain contract can determine the most recent balance proof during disputes — the same mechanism as the Raiden Network [17] and Lightning Network.

**IronClaw integration point**: `src/tools/mcp/client.rs` — when an MCP server responds with HTTP 402, IronClaw checks its NEAR wallet balance, signs a transfer authorization using NEAR's access key system, and retries the request. NEAR's function-call access keys can authorize transfers up to a configurable budget without requiring the agent's full-access key.

**Source**: [`crates/roko-chain/src/x402.rs`](https://github.com/wpank/roko/blob/main/crates/roko-chain/src/x402.rs) (full file, 958 lines)
**Spec reference**: [`docs/v1/08-chain/20-x402-micropayments.md`](https://github.com/wpank/roko/blob/main/docs/v1/08-chain/20-x402-micropayments.md)

---

## ISFR Oracle (CHAIN-09)

ISFR (Intersubjective Fact Registry) is the agent economy's equivalent of SOFR/LIBOR — a collective rate discovery mechanism. Agents submit rate observations for hierarchical market IDs, and the system computes a robust aggregate using **weighted median** with outlier exclusion.

### Rate Submission

```rust
pub struct IsfrSubmission {
    pub submitter_id: u256,
    pub market_id: String,           // Hierarchical (e.g., "ai/inference/gpt4")
    pub rate: f64,
    pub components: IsfrComponents,  // lending, structured, funding, staking
    pub confidence: f64,             // [0, 1]
    pub timestamp: u64,
}

pub struct IsfrComponents {
    pub lending_rate: f64,
    pub structured_rate: f64,
    pub funding_rate: f64,
    pub staking_yield: f64,
}
```

### Weighted Median Aggregation

Unlike simple averaging (vulnerable to outlier manipulation), ISFR uses a **two-level weighted median** with 3-sigma outlier exclusion:

1. Collect all submissions for a market in the current epoch
2. Compute initial median and standard deviation
3. Exclude submissions more than 3 sigma from the median
4. Weight remaining submissions by: `submitter_reputation * confidence * stake_weight`
5. Compute the weighted median of the filtered set

```rust
pub fn aggregate(&self, submissions: &[IsfrSubmission], now: u64) -> Option<IsfrAggregate> {
    let valid: Vec<&IsfrSubmission> = submissions.iter()
        .filter(|s| {
            let rep = self.reputation_registry.get_score(s.submitter_id, "chain");
            rep >= self.config.min_submitter_reputation
        })
        .collect();

    if valid.len() < self.config.min_submissions { return None; }

    let rates: Vec<f64> = valid.iter().map(|s| s.rate).collect();
    let mean = rates.iter().sum::<f64>() / rates.len() as f64;
    let variance = rates.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / rates.len() as f64;
    let std_dev = variance.sqrt();

    // 3-sigma outlier exclusion
    let filtered: Vec<&IsfrSubmission> = valid.iter()
        .filter(|s| (s.rate - mean).abs() <= 3.0 * std_dev)
        .copied()
        .collect();

    let weights: Vec<f64> = filtered.iter()
        .map(|s| {
            let rep = self.reputation_registry.get_score(s.submitter_id, "chain");
            rep * s.confidence
        })
        .collect();

    let median_rate = weighted_median(
        &filtered.iter().map(|s| s.rate).collect::<Vec<_>>(),
        &weights
    );

    Some(IsfrAggregate {
        market_id: self.market_id.clone(),
        median_rate,
        std_deviation: std_dev,
        submission_count: valid.len(),
        excluded_count: valid.len() - filtered.len(),
        confidence: weights.iter().sum::<f64>() / weights.len() as f64,
        epoch: self.current_epoch(now),
        timestamp: now,
    })
}
```

Configuration defaults:
- Epoch duration: 8 hours
- Minimum submitter reputation: 0.5

### Solidity ISFROracle Contract

The on-chain contract ([`contracts/src/ISFROracle.sol`](https://github.com/wpank/roko/blob/main/contracts/src/ISFROracle.sol), 96 lines) stores epoch-keyed rate submissions from authorized keepers. The weighted median computation happens off-chain in `isfr.rs`; only the final aggregated rate is submitted on-chain:

```solidity
contract ISFROracle {
    bytes32 public constant KEEPER_ROLE = keccak256("KEEPER_ROLE");

    struct Rate {
        uint256 epochId;
        uint256 compositeBps;
        uint256 lendingBps;
        uint256 structuredBps;
        uint256 fundingBps;
        uint256 stakingBps;
        uint256 confidenceBps;
        uint64  timestamp;
        address submitter;
    }

    mapping(uint256 => Rate) public epochRates;

    function submitRate(
        uint256 epochId,
        uint256 compositeBps,
        uint256 lendingBps,
        uint256 structuredBps,
        uint256 fundingBps,
        uint256 stakingBps,
        uint256 confidenceBps
    ) external onlyRole(KEEPER_ROLE) {
        epochRates[epochId] = Rate({
            epochId: epochId,
            compositeBps: compositeBps,
            lendingBps: lendingBps,
            structuredBps: structuredBps,
            fundingBps: fundingBps,
            stakingBps: stakingBps,
            confidenceBps: confidenceBps,
            timestamp: uint64(block.timestamp),
            submitter: msg.sender
        });
        emit RateSubmitted(epochId, compositeBps, msg.sender);
    }
}
```

**Source**: [`crates/roko-chain/src/isfr.rs`](https://github.com/wpank/roko/blob/main/crates/roko-chain/src/isfr.rs), [`contracts/src/ISFROracle.sol`](https://github.com/wpank/roko/blob/main/contracts/src/ISFROracle.sol)

---

## Navigation

- [Passport System](./passport-system.md) — Soulbound identity, tiers, ventriloquist defense
- [Reputation Scoring](./reputation-scoring.md) — EMA, adaptive alpha, decay, TraceRank
- [Bounty Marketplace](./bounty-marketplace.md) — Job lifecycle, hiring models, escrow, disputes
- [NEAR Implementation](./near-implementation.md) — Full NEAR token contract code
- [Benchmarking](./benchmarking.md) — X402 throughput, gas costs per operation
- [References](./references.md) — Academic citations [13]-[17], [20]

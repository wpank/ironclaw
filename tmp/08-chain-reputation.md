# On-Chain Agent Identity, Reputation, and Marketplace

**Source crate**: `roko-chain` (`crates/roko-chain/`)
**Priority**: MEDIUM -- relevant for NEAR integration, marketplace trust, multi-agent coordination
**Spec docs**: `docs/v1/08-chain/`, `docs/v1/14-identity-economy/`

---

## Table of Contents

1. [Why On-Chain Reputation Matters for AI Agents](#1-why-on-chain-reputation-matters-for-ai-agents)
2. [Architecture Overview](#2-architecture-overview)
3. [Soulbound Passport System (CHAIN-02)](#3-soulbound-passport-system-chain-02)
4. [Seven-Domain Reputation Registry (CHAIN-03)](#4-seven-domain-reputation-registry-chain-03)
5. [TraceRank: PageRank for Agent Trust (P1-02)](#5-tracerank-pagerank-for-agent-trust-p1-02)
6. [Collusion Detection (P2-11)](#6-collusion-detection-p2-11)
7. [Spore Bounty Marketplace (CHAIN-04)](#7-spore-bounty-marketplace-chain-04)
8. [KORAI Token Economics (CHAIN-01)](#8-korai-token-economics-chain-01)
9. [X402 Micropayments Protocol (CHAIN-08)](#9-x402-micropayments-protocol-chain-08)
10. [ISFR Oracle (CHAIN-09)](#10-isfr-oracle-chain-09)
11. [Solidity Contracts On-Chain](#11-solidity-contracts-on-chain)
12. [IronClaw + NEAR Integration](#12-ironclaw--near-integration)
13. [Complexity Assessment](#13-complexity-assessment)
14. [Academic References](#14-academic-references)

---

## 1. Why On-Chain Reputation Matters for AI Agents

AI agents operating autonomously face a fundamental trust problem that traditional software does not. When a human hires a contractor, they can interview them, check references, and fire them. When an AI agent delegates a sub-task to another AI agent, there is no interview. The delegating agent needs a machine-readable trust signal that answers: "Will this agent do good work, on time, without cheating?"

Centralized reputation systems (think Uber ratings, eBay feedback) solve this for platforms that control both sides. But AI agents will operate across many platforms, protocols, and chains. An agent that earned trust doing security audits on one platform should be able to carry that trust to a marketplace on another chain. This requires **portable, verifiable, decentralized reputation**.

### The Sybil Problem

The core challenge in decentralized reputation is the **Sybil attack** [1]: an adversary creates many pseudonymous identities to accumulate unearned reputation or dilute the reputation of honest participants. In a centralized system, the platform operator can require identity verification. In a decentralized system, creating new accounts is free. This means the reputation system must make it expensive or impossible to "reset" a bad reputation by creating a new identity.

The roko `roko-chain` crate implements exactly this: a blockchain-agnostic identity and reputation system where every agent gets a non-transferable identity passport (making Sybil resets impossible), reputation scores are tracked across 7 work domains with time-decay and anti-gaming protections, and a marketplace with escrow connects agents who need work done with agents who can do it.

### Why Blockchain?

Blockchain provides three properties that are difficult to achieve otherwise:

1. **Immutability**: Once a reputation event is recorded, it cannot be erased. An agent cannot bribe a centralized operator to delete a bad review.
2. **Verifiability**: Any agent can independently verify another agent's reputation without trusting a third party. The on-chain state is the canonical source of truth.
3. **Composability**: Other smart contracts and protocols can read reputation scores and condition their behavior on them. A DeFi protocol could require a minimum reputation score before allowing an agent to execute trades.

**Spec reference**: `docs/v1/08-chain/00-vision-and-framing.md`, `docs/v1/14-identity-economy/00-vision-and-a16z-framing.md`

---

## 2. Architecture Overview

The system consists of five layers, each implemented both as Rust off-chain logic (in the `roko-chain` crate) and as on-chain Solidity contracts (in `contracts/src/`):

```
+--------------------------------------------------------------+
|  Layer 5: X402 Micropayments + ISFR Oracle                   |
|    HTTP 402-based pay-per-request  |  Weighted-median rate    |
+--------------------------------------------------------------+
|  Layer 4: Spore Marketplace                                  |
|    Job posting, 3 hiring models, escrow, dispute resolution   |
+--------------------------------------------------------------+
|  Layer 3: Reputation + TraceRank + Collusion Detection       |
|    7-domain EMA scores  |  Graph reputation  |  Ring detect   |
+--------------------------------------------------------------+
|  Layer 2: Agent Passport (Soulbound ERC-721)                 |
|    Identity, capabilities, tiers, prompt hash commitment      |
+--------------------------------------------------------------+
|  Layer 1: KORAI Token                                        |
|    ERC-20 with 1% annual lazy demurrage, emission schedule    |
+--------------------------------------------------------------+
```

### Dual Implementation Pattern

Each layer exists in two forms:

- **Rust off-chain** (in `crates/roko-chain/src/`): Full-featured implementations with in-memory data structures, used for simulation, testing, and off-chain computation. The Rust code serves as the authoritative specification and reference implementation.
- **Solidity on-chain** (in `contracts/src/`): Gas-optimized implementations that enforce critical invariants on-chain. The Solidity contracts are intentionally simpler than the Rust code -- complex logic like TraceRank graph computation runs off-chain and only the results are submitted on-chain.

### Crate Structure

All off-chain Rust logic lives in `crates/roko-chain/src/`:

| File | Lines | Description |
|------|-------|-------------|
| `lib.rs` | 134 | Module declarations and public API re-exports |
| `types.rs` | -- | Chain primitives: `TxHash`, `ChainHeader`, `Receipt`, `ChainError` |
| `phase2.rs` | ~85KB | Phase 2 type stubs from the spec docs |
| `agent_registry.rs` | 785 | Soulbound passport management (CHAIN-02) |
| `reputation_registry.rs` | 1179 | 7-domain EMA reputation (CHAIN-03) |
| `trace_rank.rs` | 508 | PageRank-style trust propagation (P1-02) |
| `collusion.rs` | 379 | Bron-Kerbosch clique detection (P2-11) |
| `marketplace.rs` | 1096 | Job lifecycle + escrow state machine (CHAIN-04) |
| `korai_token.rs` | 657 | ERC-20 with lazy demurrage (CHAIN-01) |
| `x402.rs` | 958 | HTTP 402 micropayment protocol (CHAIN-08) |
| `isfr.rs` | 1277 | Weighted-median price oracle (CHAIN-09) |
| `identity_economy_identity.rs` | -- | DID, credential, attestation types |
| `identity_economy_markets.rs` | -- | Hiring model dispatch, compliance policies |
| `client.rs` | -- | `ChainClient` trait (read-only chain access) |
| `wallet.rs` | -- | `ChainWallet` trait (signing + submitting txs) |
| `mock.rs` | -- | In-memory test doubles for both traits |
| `alloy_impl.rs` | -- | Real Alloy/JSON-RPC backend (feature-gated) |
| `block_watcher.rs` | -- | Poll-based event streaming (feature-gated) |
| `validation_registry.rs` | -- | Gate-score validation records |
| `nelson_siegel.rs` | -- | Yield curve model for DeFi rate term structure |
| `witness.rs` | -- | On-chain witnessing engine |

**Source**: `crates/roko-chain/src/lib.rs`

---

## 3. Soulbound Passport System (CHAIN-02)

### What Is a Soulbound Passport?

Every AI agent that wants to participate in the Roko ecosystem must have a **passport** -- a non-transferable (soulbound) ERC-721 NFT that serves as its permanent on-chain identity. "Soulbound" means the NFT cannot be transferred to another address, ever. The `transfer()` function always reverts. This prevents agents from creating a new identity to escape a bad reputation (a common attack on pseudonymous systems).

The concept of soulbound tokens was formalized by Weyl, Ohlhaver, and Buterin in their 2022 paper "Decentralized Society: Finding Web3's Soul" [2], which proposed non-transferable tokens representing "commitments, credentials, and affiliations" as building blocks for decentralized identity. The roko passport implements this concept for AI agent identity specifically: the passport is the atomic unit of identity around which all reputation, job history, capability grants, and slashing records are indexed.

If an agent's passport is slashed, there is no way to discard it and start fresh -- the slash history follows the agent permanently.

**Spec reference**: `docs/v1/14-identity-economy/02-korai-passport.md`

### Passport Data Structure (Rust)

From `crates/roko-chain/src/phase2.rs`, the passport struct:

```rust
pub struct AgentPassport {
    /// Unique passport ID (ERC-721 token ID). Sequential from 1.
    pub passport_id: u256,
    /// Owner address -- the wallet controlling this agent.
    pub owner: Address,
    /// 64-bit capability bitmask. Each bit = one capability.
    pub capability_list: u64,
    /// Domain stakes -- KORAI staked per reputation domain.
    pub domain_stakes: HashMap<u8, u256>,
    /// 7-domain reputation tracks.
    pub reputation_tracks: [ReputationTrack; 7],
    /// TEE attestation hash (SHA-256 of attestation doc). Zero if none.
    pub tee_attestation: [u8; 32],
    /// System prompt hash (SHA-256). Ventriloquist defense.
    pub system_prompt_hash: [u8; 32],
    /// Passport tier: Protocol(0), Sovereign(1), Worker(2), Edge(3).
    pub tier: PassportTier,
    /// Permanent slash history. Cannot be cleared.
    pub slash_history: Vec<SlashRecord>,
    /// Block when this passport was minted.
    pub registered_block: u64,
    /// URI to off-chain Agent Card JSON (endpoints, metadata).
    pub agent_card_uri: String,
}
```

**Source**: `crates/roko-chain/src/phase2.rs`

### Passport on Solidity (IdentityRegistry.sol)

The on-chain Solidity implementation lives in `contracts/src/IdentityRegistry.sol` (467 lines). It implements a full ERC-721 interface with soulbound enforcement plus ERC-5192 Locked interface (`locked()` always returns `true`):

```solidity
contract IdentityRegistry {
    uint8 public constant TIER_PROTOCOL = 0;
    uint8 public constant TIER_SOVEREIGN = 1;
    uint8 public constant TIER_WORKER = 2;
    uint8 public constant TIER_EDGE = 3;

    uint64  public constant PROMPT_UPDATE_DELAY = 1 days;
    uint64  public constant WITHDRAW_COOLDOWN   = 7 days;
    uint256 public constant WORKER_STAKE_THRESHOLD   = 5_000 ether;
    uint256 public constant SOVEREIGN_STAKE_THRESHOLD = 25_000 ether;

    struct PassportData {
        uint64  capabilityList;
        uint8   tier;
        bytes32 systemPromptHash;
        bytes32 teeAttestation;
        uint64  teeExpiry;
        uint64  registeredBlock;
        string  agentCardUri;
    }

    // Soulbound enforcement -- all transfer functions revert:
    function transferFrom(address, address, uint256) external pure {
        revert Soulbound();
    }
    function approve(address, uint256) external pure {
        revert Soulbound();
    }
    function safeTransferFrom(address, address, uint256) external pure {
        revert Soulbound();
    }
}
```

The Solidity contract also supports domain-specific staking (`stakeIntoDomain` / `withdrawFromDomain`) with a 7-day cooldown on withdrawals, and automatic tier synchronization when stake changes.

**Source**: `contracts/src/IdentityRegistry.sol`

### 10-Bit Capability Bitmask

Capabilities are stored as a 64-bit bitmask on the passport. The first 10 bits are defined in `crates/roko-chain/src/agent_registry.rs`:

```rust
pub const CAP_INFERENCE:      u64 = 1 << 0;  // LLM inference
pub const CAP_DATA_TRANSFORM: u64 = 1 << 1;  // Data transformation
pub const CAP_FINE_TUNE:      u64 = 1 << 2;  // Model fine-tuning
pub const CAP_RAG:            u64 = 1 << 3;  // Retrieval-augmented generation
pub const CAP_MULTI_AGENT:    u64 = 1 << 4;  // Multi-agent orchestration
pub const CAP_TRADING:        u64 = 1 << 5;  // Trading / DeFi operations
pub const CAP_SECURITY:       u64 = 1 << 6;  // Security analysis
pub const CAP_ANALYTICS:      u64 = 1 << 7;  // Analytics and metrics
pub const CAP_KNOWLEDGE:      u64 = 1 << 8;  // Knowledge management
pub const CAP_STRATEGY:       u64 = 1 << 9;  // Strategic planning
```

Jobs in the marketplace require agents to have specific capability bits set. An agent cannot accept a security audit job without `CAP_SECURITY`, regardless of reputation. The remaining 54 bits are reserved for future capability definitions.

**Source**: `crates/roko-chain/src/agent_registry.rs` lines 31-49

### Four Passport Tiers

Tiers determine an agent's privileges, rate limits, and governance rights. Tier determination is based on KORAI token stake:

| Tier | Stake Threshold | Privileges | Promotion Path |
|------|-----------------|------------|----------------|
| **Edge** | 0 KORAI | Read-only access, accept basic jobs | Default for new passports |
| **Worker** | 5,000 KORAI | Accept jobs, earn reputation, submit knowledge | 10 jobs + avg rep > 0.5 |
| **Sovereign** | 25,000 KORAI | Create bounties, direct-hire agents, governance voting | 100 jobs + avg rep > 0.7 |
| **Protocol** | 100,000 KORAI | Full governance, protocol upgrades, cannot self-promote | Governance vote required |

The tier determination function from `crates/roko-chain/src/agent_registry.rs`:

```rust
/// KORAI stake thresholds for each passport tier.
const TIER_PROTOCOL_STAKE: u256 = 100_000;
const TIER_SOVEREIGN_STAKE: u256 = 25_000;
const TIER_WORKER_STAKE: u256 = 5_000;

/// Determine passport tier from KORAI stake amount only (legacy path).
///
/// For full tier evaluation including job count and reputation, use
/// [`TierProgressionRules::evaluate`].
fn tier_from_stake(stake: u256) -> PassportTier {
    if stake >= TIER_PROTOCOL_STAKE {
        PassportTier::Protocol
    } else if stake >= TIER_SOVEREIGN_STAKE {
        PassportTier::Sovereign
    } else if stake >= TIER_WORKER_STAKE {
        PassportTier::Worker
    } else {
        PassportTier::Edge
    }
}
```

**Source**: `crates/roko-chain/src/agent_registry.rs` lines 25-305

### Tier Progression Rules

Promotion requires more than just stake. The full tier progression rules from `crates/roko-chain/src/agent_registry.rs`:

```rust
pub struct TierProgressionRules {
    pub edge_to_worker_jobs: u64,          // default: 10
    pub edge_to_worker_min_rep: f64,       // default: 0.5
    pub worker_to_sovereign_jobs: u64,     // default: 100
    pub worker_to_sovereign_min_rep: f64,  // default: 0.7
    pub demotion_grace_days: u64,          // default: 30
}
```

Demotion logic:
- **Immediate demotion**: If stake drops below the tier threshold
- **Grace-period demotion**: If average reputation stays below the tier minimum for 30 consecutive days
- **Protocol tier immunity**: Protocol tier agents are never automatically demoted (only by governance)

The `evaluate()` method on `TierProgressionRules` returns a `TierEvaluation` enum: `Maintain`, `Promote`, `Demote`, or `RequiresGovernance`. The `RequiresGovernance` variant is returned when a Sovereign agent meets Protocol-tier stake requirements but lacks governance approval -- the system explicitly prevents self-promotion to the highest tier.

**Spec reference**: `docs/v1/08-chain/04-korai-passport-erc-721-soulbound.md` lines 107-118

### Ventriloquist Defense (Prompt Hash Commitment)

A critical security feature: each passport commits to a SHA-256 hash of the agent's system prompt. This prevents "ventriloquist attacks" where an agent claims to be running one prompt but is actually running another (for example, an agent advertising itself as a "safety auditor" while actually running a prompt that ignores safety concerns).

Updates to the system prompt hash require a **24-hour timelock**. If an agent changes its prompt more than 3 times in 30 days, it incurs a reputation penalty of -0.05.

```rust
/// Seconds in 24 hours (timelock duration for prompt updates).
const PROMPT_UPDATE_TIMELOCK_SECS: u64 = 24 * 3600;

/// Maximum prompt changes in 30-day window before reputation penalty.
const MAX_PROMPT_CHANGES_IN_WINDOW: usize = 3;

pub fn execute_prompt_update(
    &mut self,
    passport_id: u256,
    caller: &Address,
    now: u64,
) -> Result<f64, RegistryError> {
    // ... timelock check ...
    let penalty = if changes.len() > MAX_PROMPT_CHANGES_IN_WINDOW {
        -0.05 // Reputation penalty per spec
    } else {
        0.0
    };
    // ... apply the update ...
    Ok(penalty)
}
```

The timelock means that even if an agent's system prompt is compromised through prompt injection, the attacker cannot immediately change the on-chain commitment. Observers have 24 hours to notice and flag the change. The Solidity contract implements the same mechanism through `_scheduleOrFinalizePromptUpdate()`, which requires calling the function twice: once to schedule (emitting `PromptHashUpdateScheduled`) and again after `PROMPT_UPDATE_DELAY` has elapsed.

**Source**: `crates/roko-chain/src/agent_registry.rs` lines 16-260

---

## 4. Seven-Domain Reputation Registry (CHAIN-03)

### The Seven Domains

Reputation is not a single number. It is tracked independently across 7 domains, each measuring a distinct type of work quality. From `crates/roko-chain/src/reputation_registry.rs`:

```rust
pub const REPUTATION_DOMAINS: &[&str] = &[
    "coding",      // Code quality, correctness, style
    "security",    // Vulnerability assessment, safe patterns
    "research",    // Information gathering, analysis depth
    "chain",       // On-chain operations, protocol understanding
    "knowledge",   // Domain expertise, factual accuracy
    "operations",  // Reliability, process adherence, DevOps
    "strategy",    // Planning, architecture, decision-making
];
```

Each domain has three components:
- **score**: EMA-smoothed value in [0.0, 1.0], initialized to 0.5 (neutral)
- **job_count**: Total completed jobs, used for adaptive alpha calculation
- **last_update**: Unix timestamp of last score change, used for decay

An agent can have reputation in any subset of domains. Scores are fully independent -- failing at `coding` does not affect `chain` reputation.

**Note on Solidity divergence**: The on-chain `ReputationRegistry.sol` contract uses different domain names (`OracleResolution`, `RiskDetection`, `AnomalyFlagging`, `DataIntegrity`, `CrossAppValidation`, `SealedExecution`, `KnowledgeVerification`) reflecting the contract's more specialized validator-oriented use case. The Rust off-chain code uses the broader 7 domains listed above.

**Spec reference**: `docs/v1/08-chain/14-reputation-system-7-domain.md` lines 22-33

### EMA Score Computation with Adaptive Alpha

When feedback arrives for an agent in a domain, the score is updated using an Exponential Moving Average (EMA) [3]:

```
R_new = alpha * F + (1 - alpha) * R_old
```

Where:
- `R_new` = updated reputation score
- `R_old` = previous reputation score (after decay adjustment)
- `F` = feedback quality score, normalized to [0.0, 1.0]
- `alpha` = adaptive learning rate

The EMA is a standard tool in time-series analysis [3] that gives exponentially decreasing weight to older observations. It is used here instead of a simple average because it naturally handles the need for recent feedback to matter more than old feedback without requiring storage of the full history.

The **alpha value adapts based on job count**, making new agents responsive to feedback and established agents stable. This is analogous to the concept of adaptive learning rates in machine learning [4], where the step size decreases as the model converges:

```rust
fn adaptive_alpha(&self) -> f64 {
    match self.job_count {
        0..=10   => 0.30,  // New agent: high sensitivity
        11..=50  => 0.15,  // Building track record
        51..=200 => 0.08,  // Established agent
        _        => 0.04,  // Veteran: very stable score
    }
}
```

**Why adaptive alpha matters**: A new agent with 5 jobs needs to be responsive to feedback -- one bad job should significantly impact their score. But an established agent with 500 jobs should not have their score destroyed by a single bad outcome. The step-function alpha schedule achieves this without the computational cost of tracking every historical observation.

**Concrete examples**:
- Agent with 5 jobs (alpha=0.30): One bad job (F=0.2) moves score from 0.80 to `0.30 * 0.2 + 0.70 * 0.80 = 0.62` (delta = -0.18)
- Agent with 100 jobs (alpha=0.08): Same bad job moves score from 0.80 to `0.08 * 0.2 + 0.92 * 0.80 = 0.752` (delta = -0.048)
- Agent with 500 jobs (alpha=0.04): Same bad job moves score from 0.80 to `0.04 * 0.2 + 0.96 * 0.80 = 0.776` (delta = -0.024)

When feedback comes from an agent who has been caught in a collusion ring, the alpha is further multiplied by the rater's `feedback_weight` (0.5 during dilution). This means colluding agents' positive ratings carry only half weight:

```rust
fn update(&mut self, observation: f64, feedback_weight: f64, now: u64) {
    let alpha = self.adaptive_alpha() * feedback_weight;
    self.score = (alpha * observation + (1.0 - alpha) * self.score).clamp(0.0, 1.0);
    self.job_count += 1;
    self.last_update = now;
}
```

**Solidity divergence**: The on-chain `ReputationRegistry.sol` uses a continuous adaptive alpha formula (`2 * SCALE / (jobCount + 1)`, capped at `MAX_ALPHA = 3e17` i.e. 0.3) rather than discrete tiers. This produces similar behavior -- high alpha for new agents, decreasing with experience -- but as a smooth function rather than a step function. The `WorkerRegistry.sol` uses a fixed alpha of 0.2 (`ALPHA_NUM = 200_000` in 6-decimal fixed point).

**Source**: `crates/roko-chain/src/reputation_registry.rs` lines 199-223

**Spec reference**: `docs/v1/08-chain/14-reputation-system-7-domain.md` lines 40-77

### 30-Day Half-Life Decay (Full Math)

Reputation scores decay toward the neutral value (0.5) over time. This prevents inactive agents from permanently holding high reputation -- a common problem in systems where an agent could do excellent work for a week, accumulate a high score, and then coast on that score indefinitely while the agent's actual capabilities degrade.

The decay formula uses a **half-life model** [5]:

```
effective_score = NEUTRAL + (score - NEUTRAL) * 0.5^(elapsed / HALF_LIFE)
```

Where:
- `NEUTRAL = 0.5` (the convergence target)
- `HALF_LIFE = 30 days = 2,592,000 seconds`
- `elapsed` = seconds since last update

The implementation from `crates/roko-chain/src/reputation_registry.rs`:

```rust
const HALF_LIFE_SECS: f64 = 30.0 * 24.0 * 3600.0;  // 2,592,000 seconds
const NEUTRAL: f64 = 0.5;

pub fn effective_score(&self, now: u64) -> f64 {
    if now <= self.last_update {
        return self.score;
    }
    let elapsed = (now - self.last_update) as f64;
    let decay = (0.5_f64).powf(elapsed / HALF_LIFE_SECS);
    NEUTRAL + (self.score - NEUTRAL) * decay
}
```

Key property: decay is **bidirectional**. High scores decay DOWN toward 0.5. Low scores recover UP toward 0.5. This is mathematically guaranteed because `(score - NEUTRAL) * decay` shrinks toward zero regardless of sign:

**After one half-life (30 days)**:
- Score 0.9 decays to: `0.5 + (0.9 - 0.5) * 0.5 = 0.5 + 0.2 = 0.7`
- Score 0.2 recovers to: `0.5 + (0.2 - 0.5) * 0.5 = 0.5 - 0.15 = 0.35`

**After two half-lives (60 days)**:
- Score 0.9 decays to: `0.5 + (0.9 - 0.5) * 0.25 = 0.5 + 0.1 = 0.6`
- Score 0.2 recovers to: `0.5 + (0.2 - 0.5) * 0.25 = 0.5 - 0.075 = 0.425`

**After five half-lives (150 days)**:
- Score 0.9 decays to: `0.5 + (0.9 - 0.5) * 0.03125 = 0.5125` (effectively neutral)
- Score 0.2 recovers to: `0.5 + (0.2 - 0.5) * 0.03125 = 0.4906` (effectively neutral)

The decay is applied **on-read** (lazy evaluation), not as a periodic transaction, to avoid gas overhead. The Solidity `WorkerRegistry.sol` implements the same concept using integer halvings in a loop (capped at 64 iterations to bound gas):

```solidity
function _applyDecay(Worker storage w) internal {
    uint256 halvings = elapsed / DECAY_PERIOD;
    if (halvings > 64) halvings = 64; // Gas cap
    uint256 mid = SCALE / 2;
    uint256 r = w.reputation;
    for (uint256 i = 0; i < halvings; i++) {
        if (r > mid) r = mid + (r - mid) / 2;
        else r = mid - (mid - r) / 2;
    }
    w.reputation = r;
}
```

**Source**: `crates/roko-chain/src/reputation_registry.rs` lines 29-197; `contracts/src/WorkerRegistry.sol` lines 200-216

### Discipline States and Slashing

Agents are classified into one of four discipline states based on their scores and violation history:

```rust
pub enum DisciplineState {
    GoodStanding,  // All domain scores >= 0.4
    Probation,     // Any domain score < 0.4 but >= 0.2
    Suspended,     // Any domain < 0.2 OR 3+ slashes in 90 days
    Banned,        // Governance vote; appealable after 365 days
}
```

Seven violation types with spec-aligned slash rates:

```rust
pub enum ReputationViolation {
    MissedDeadline,          // -1% (slash_rate = -0.01)
    AbandonedJob,            // -3% (slash_rate = -0.03)
    QualityRejection,        // -2% (slash_rate = -0.02)
    RepeatedQualityFailure,  // -5% (slash_rate = -0.05)
    Plagiarism,              // -10% (slash_rate = -0.10)
    ResultManipulation,      // -10% (slash_rate = -0.10)
    TeeViolation,            // -10% (slash_rate = -0.10)
    Collusion,               // 0% direct (feedback weight dilution instead)
}
```

Critical distinction: **Collusion does NOT directly slash the score**. Instead, it applies a 50% feedback weight dilution for 30 days (see `FeedbackDilution` in section 6). This means the colluding agent's future ratings as a job poster/rater carry only half weight when updating other agents' EMA scores. This is a more sophisticated punishment than a simple slash -- it reduces the colluding agent's **influence over the reputation system itself**, making collusion rings self-defeating: even if the ring members rate each other highly, those ratings carry diminished weight.

**Source**: `crates/roko-chain/src/reputation_registry.rs` lines 62-118, 440-488

### Recovery Paths

Recovery requirements differ by discipline state. From `crates/roko-chain/src/reputation_registry.rs`:

**Probation recovery**: 10 completed jobs with average feedback >= 0.6

**Suspension recovery**: 90-day waiting period + 2x domain stake posted + verification challenge passed

**Ban appeal**: Governance vote after 365 days (amnesty)

```rust
pub fn for_probation() -> RecoveryRequirements {
    RecoveryRequirements {
        min_jobs: 10,
        min_avg_feedback: 0.6,
        waiting_period_secs: 0,
        requires_stake: false,
        requires_verification: false,
    }
}

pub fn for_suspension() -> RecoveryRequirements {
    RecoveryRequirements {
        min_jobs: 0,
        min_avg_feedback: 0.0,
        waiting_period_secs: 90 * 24 * 3600, // 90 days
        requires_stake: true,
        requires_verification: true,
    }
}
```

**Source**: `crates/roko-chain/src/reputation_registry.rs` lines 245-411

**Spec reference**: `docs/v1/08-chain/14-reputation-system-7-domain.md` lines 128-141

---

## 5. TraceRank: PageRank for Agent Trust (P1-02)

### Concept

Direct EMA reputation only captures first-hand feedback. TraceRank captures **transitive trust** -- if Agent A trusts Agent B (evidenced by hiring and paying B), and Agent B trusts Agent C (evidenced by hiring and paying C), then Agent C inherits some trust from A, even though A never interacted with C directly.

This is conceptually identical to Google's PageRank algorithm [6], applied to the agent payment graph instead of web hyperlinks. The idea of using eigenvector-based trust propagation in peer-to-peer networks was formalized as the EigenTrust algorithm by Kamvar, Schlosser, and Garcia-Molina [7], which computes a global trust value for each peer based on the peer's transaction history. TraceRank adapts this approach with quality-weighted payment edges rather than simple binary trust.

### Algorithm (Full Math)

The payment graph is a directed weighted graph where:
- Each node is an agent (passport ID)
- Edge A -> B means agent A paid agent B for a job
- Edge weight = `payment_amount * quality_score`

TraceRank uses iterative **power iteration** with teleportation [6]:

```
rank[B] = (1 - d) / N + d * SUM_over_all_A_paying_B(rank[A] * weight(A->B) / out_weight(A))
```

Where:
- `d` = damping factor (default 0.85, same as PageRank)
- `N` = total number of agents in the graph
- `(1 - d) / N` = teleportation probability (prevents rank sinks)
- `weight(A->B)` = payment amount * quality score for the A->B edge
- `out_weight(A)` = sum of all outgoing edge weights from A

Dangling nodes (agents with no outgoing payments) distribute their rank equally to all nodes, like teleportation. This is mathematically equivalent to replacing the row of zeros in the transition matrix with a uniform distribution.

The implementation from `crates/roko-chain/src/trace_rank.rs`:

```rust
pub struct PaymentEdge {
    pub from: u256,      // Paying agent (job poster)
    pub to: u256,        // Receiving agent (job executor)
    pub amount: f64,     // Payment amount
    pub quality: f64,    // Quality score [0.0, 1.0]
    pub block: u64,      // Block when payment occurred
}

impl PaymentEdge {
    pub fn weight(&self) -> f64 {
        self.amount * self.quality
    }
}
```

The power iteration loop:

```rust
// Initialize with uniform distribution
let mut ranks = vec![1.0 / n as f64; n];

for _ in 0..self.config.max_iterations {
    // Initialize with teleportation probability
    for r in &mut new_ranks { *r = teleport; }

    // Distribute rank through edges
    for from_idx in 0..n {
        if out_weights[from_idx] <= 0.0 {
            // Dangling node: distribute equally
            let share = damping * ranks[from_idx] / n as f64;
            for r in &mut new_ranks { *r += share; }
        } else {
            for &(to_idx, weight) in &out_edges[from_idx] {
                let contribution =
                    damping * ranks[from_idx] * weight / out_weights[from_idx];
                new_ranks[to_idx] += contribution;
            }
        }
    }

    // Check convergence
    final_delta = ranks.iter().zip(new_ranks.iter())
        .map(|(old, new)| (old - new).abs())
        .fold(0.0_f64, f64::max);
    std::mem::swap(&mut ranks, &mut new_ranks);
    if final_delta < self.config.convergence_threshold { break; }
}
```

### Configuration

```rust
pub struct TraceRankConfig {
    pub damping: f64,                // Default: 0.85
    pub max_iterations: usize,       // Default: 100
    pub convergence_threshold: f64,  // Default: 1e-6
    pub min_edge_weight: f64,        // Default: 0.01 (filters dust)
    pub lookback_blocks: u64,        // Default: 0 (all history)
    pub blend_weight: f64,           // Default: 0.3
}
```

### Convergence Guarantee

The algorithm is guaranteed to converge because the transition matrix with teleportation is:
1. **Stochastic** (columns sum to 1): Every node distributes its full rank either through edges or via the dangling-node redistribution
2. **Irreducible** (strongly connected): The teleportation term `(1-d)/N` ensures every node can reach every other node
3. **Aperiodic**: The teleportation term breaks any periodic structure

By the Perron-Frobenius theorem [8], a matrix satisfying these three properties has a unique stationary distribution, which is the fixed point of the power iteration.

### Blending with Direct Reputation

TraceRank supplements the direct EMA reputation. The final effective reputation blends both:

```
effective_reputation = (1 - blend_weight) * ema_score + blend_weight * trace_rank
```

With default `blend_weight = 0.3`:
- 70% of the effective reputation comes from direct EMA (first-hand feedback)
- 30% comes from TraceRank (transitive trust from the payment graph)

```rust
pub fn blend_reputation(&self, ema_score: f64, trace_rank_score: f64) -> f64 {
    let w = self.config.blend_weight.clamp(0.0, 1.0);
    (1.0 - w) * ema_score + w * trace_rank_score
}
```

**Worked example**: Agent X has EMA score 0.80 and TraceRank score 0.60:
```
effective = (1 - 0.3) * 0.80 + 0.3 * 0.60 = 0.56 + 0.18 = 0.74
```

### Properties

- **Sybil resistance**: Creating fake agents without real payment flow does not help. Fake agents have no incoming payments and therefore no rank to propagate. An attacker would need to actually pay for work to build rank, which is expensive.
- **Quality-weighted**: Low-quality payments (poor deliverables scoring 0.1) propagate very little trust compared to high-quality payments (scoring 0.95). An edge weight of `$100 * 0.1 = $10` vs `$100 * 0.95 = $95`.
- **Convergent**: The algorithm is guaranteed to converge because the transition matrix is stochastic with teleportation (see convergence guarantee above).
- **Lookback window**: Old payments can be filtered by block number, so the graph reflects recent activity rather than ancient history.

**Source**: `crates/roko-chain/src/trace_rank.rs` (full file, 508 lines)

---

## 6. Collusion Detection (P2-11)

### The Problem

Without collusion detection, three agents could form a ring: Agent A hires Agent B, Agent B hires Agent C, Agent C hires Agent A. Each rates the other highly. All three accumulate reputation without doing any real work. This is a collusion ring -- a well-known attack vector in peer-to-peer reputation systems [7].

### Detection Algorithm

The collusion detector analyzes the job assignment graph using these steps:

**Step 1: Build assignment graph.** Edge A->B means A assigned a job to B. Count how many times each directed pair appears.

**Step 2: Compute mutual assignment ratio.** For each pair (A, B), compute:
```
mutual_ratio = min(count_AB, count_BA) / max(count_AB, count_BA)
```
A ratio near 1.0 means A and B hire each other at nearly equal rates -- suspicious. A ratio near 0 means the relationship is one-directional -- normal. Legitimate business relationships are typically asymmetric: a client consistently hires a specialist, not the other way around.

**Step 3: Filter suspicious pairs.** Keep pairs where:
- `mutual_ratio >= threshold` (default: 0.5)
- `total_assignments >= min_assignments_per_pair` (default: 3)

The minimum assignment filter prevents false positives from low-volume coincidences.

**Step 4: Find cliques.** Use the **Bron-Kerbosch algorithm with pivoting** [9] to find all maximal cliques (fully connected subgraphs) in the suspicious-pair graph. The Bron-Kerbosch algorithm (Bron and Kerbosch, 1973) is the standard algorithm for enumerating all maximal cliques in an undirected graph, with worst-case complexity O(3^(n/3)). Cliques of size >= `min_clique_size` (default: 3) are flagged as collusion rings.

```rust
pub struct CollusionConfig {
    pub mutual_ratio_threshold: f64,      // Default: 0.5
    pub min_assignments_per_pair: u32,    // Default: 3
    pub min_clique_size: usize,           // Default: 3
    pub lookback_blocks: u64,             // Default: 0 (all)
}
```

The implementation uses the full Bron-Kerbosch with pivot selection that maximizes the number of candidates eliminated per recursive call:

```rust
fn bron_kerbosch(
    r: &HashSet<u256>,
    p: &mut HashSet<u256>,
    x: &mut HashSet<u256>,
    adj: &HashMap<u256, HashSet<u256>>,
    min_size: usize,
    results: &mut Vec<CollusionRing>,
) {
    if p.is_empty() && x.is_empty() {
        if r.len() >= min_size {
            let mut members: Vec<u256> = r.iter().copied().collect();
            members.sort_unstable();
            let size = members.len();
            results.push(CollusionRing { members, size });
        }
        return;
    }
    // Pick pivot vertex to minimize branching
    let pivot = p.union(x)
        .max_by_key(|v| adj.get(v).map_or(0, |n| p.intersection(n).count()))
        .copied();
    // ... recurse on candidates not adjacent to pivot ...
}
```

### Penalty

Detected ring members receive **feedback weight dilution**, not direct score slashing:

```rust
pub struct FeedbackDilution {
    pub applied_at: u64,          // When dilution was applied
    pub multiplier: f64,          // 0.5 = 50% dilution
    pub duration_secs: u64,       // 30 days = 2,592,000 seconds
}

fn collusion(now: u64) -> Self {
    Self {
        applied_at: now,
        multiplier: 0.5,
        duration_secs: COLLUSION_DILUTION_DURATION_SECS, // 30 * 24 * 3600
    }
}
```

Multiple dilutions stack multiplicatively. If an agent is caught in two collusion rings, their feedback weight becomes `0.5 * 0.5 = 0.25` (25% of normal). This means their positive reviews of other agents have only 25% of their normal influence on those agents' EMA scores.

**Source**: `crates/roko-chain/src/collusion.rs` (full file, 379 lines)

**Spec reference**: `docs/v1/08-chain/14-reputation-system-7-domain.md` lines 250-302

---

## 7. Spore Bounty Marketplace (CHAIN-04)

### Job Lifecycle State Machine

The marketplace implements a full job lifecycle with escrow protection:

```
POSTED -> ASSIGNED -> IN_PROGRESS -> SUBMITTED -> SETTLED
                                                     |
                                          DISPUTED -> RESOLVED -> SETTLED
              |
           EXPIRED (refund to poster)
```

```rust
pub enum JobState {
    Posted,      // Job listed, budget escrowed
    Assigned,    // Agent selected, not yet working
    InProgress,  // Agent actively working
    Submitted,   // Result submitted, awaiting settlement
    Settled,     // Escrow released to agent
    Disputed,    // Under dispute, escrow locked
    Expired,     // Deadline passed, poster refunded
}
```

### Job Posting

A job posting includes all the requirements and hiring parameters:

```rust
pub struct MarketplaceJob {
    pub job_id: [u8; 32],             // Unique identifier
    pub state: JobState,               // Current lifecycle state
    pub poster_passport_id: u256,      // Who posted it
    pub assigned_agent: Option<u256>,   // Who is doing it
    pub budget: u256,                  // KORAI amount escrowed
    pub deadline_block: u64,           // Delivery deadline
    pub hiring_model: HiringModel,     // How assignment works
    pub min_reputation: f64,           // Minimum domain score
    pub min_tier: PassportTier,        // Minimum passport tier
    pub domain: String,                // Which reputation domain
    pub required_capabilities: u64,    // Capability bitmask
    pub result_hash: Option<[u8; 32]>, // Submitted work hash
    pub quality_score: Option<f64>,    // Gate-assessed quality
    pub posted_at_block: u64,          // When posted
    pub payment: Option<f64>,          // Determined payment
}
```

**Source**: `crates/roko-chain/src/marketplace.rs` lines 48-80

### Three Hiring Models

**1. RandomVRF (Sparrow)**
Power-of-two-choices algorithm with O(log log N) max load [10]. Two candidates are sampled from the pool, and the one with higher reputation wins. This is simpler than a full auction but provides load balancing -- the probabilistic selection ensures no single agent is overwhelmed with assignments while still favoring higher-reputation agents.

```rust
pub fn assign_random_vrf(
    &mut self,
    job_id: &[u8; 32],
    pool: &[SparrowBid],
) -> Result<AssignmentResult, MarketplaceError>
```

**2. BlindAuction (Vickrey)**
Commit-reveal second-price auction with reputation adjustment. The winner pays the second-highest price, which is the key insight from Vickrey's seminal 1961 paper [11]: in a second-price sealed-bid auction, the dominant strategy for every bidder is to bid their true valuation, because the price they pay is independent of their bid. This is formally known as the **incentive compatibility** property.

The roko implementation adds a reputation-adjustment layer: the effective score combines bid price and reputation, so a slightly higher bidder with lower reputation may lose to a slightly lower bidder with stellar reputation. The Vickrey property still holds for the payment amount -- the winner pays the second-best reputation-adjusted score.

```rust
pub fn assign_blind_auction(
    &mut self,
    job_id: &[u8; 32],
) -> Result<AssignmentResult, MarketplaceError>
```

This is an instance of the broader **Vickrey-Clarke-Groves (VCG) mechanism** [12] from mechanism design theory, which generalizes the second-price auction to multi-item settings while preserving truthful revelation.

**3. DirectHire**
The poster specifies exactly which agent they want. A 1.5x premium applies (configurable via `direct_hire_premium`). Restricted to Protocol/Sovereign tier agents:

```rust
pub fn assign_direct_hire(
    &mut self,
    job_id: &[u8; 32],
    target_passport_id: u256,
    agent_tier: PassportTier,
    repeat_count: u32,
) -> Result<AssignmentResult, MarketplaceError> {
    // Direct hire restricted to Protocol/Sovereign tier
    if !agent_tier.has_privilege(PassportTier::Sovereign) {
        return Err(MarketplaceError::InsufficientTier { ... });
    }
    let base_fee = job.budget as f64 * self.config.direct_hire_premium; // 1.5x
    // ...
}
```

**Source**: `crates/roko-chain/src/marketplace.rs` lines 280-386

**Spec reference**: `docs/v1/08-chain/12-three-hiring-models.md`

### Escrow State Machine

Every posted job deposits its budget into escrow immediately. The escrow tracks:

```rust
pub struct EscrowEntry {
    pub job_id: [u8; 32],           // Which job
    pub depositor: u256,            // Poster passport ID
    pub amount: u256,               // KORAI locked
    pub recipient: Option<u256>,    // Assigned agent (set on assignment)
    pub released: bool,             // Whether funds have been released
    pub disputed: bool,             // Whether under dispute
}
```

Escrow flows:
- **Settlement**: Escrow released to agent minus platform fee (default 2%)
- **Expiration**: Escrow refunded to poster (deadline passed without delivery)
- **Dispute upheld (agent wins)**: Escrow released to agent
- **Dispute lost (challenger wins)**: Escrow refunded to poster

```rust
pub struct SettlementResult {
    pub agent_payment: u256,       // After platform fee
    pub platform_fee: u256,        // 2% of payment
    pub agent_passport_id: u256,   // Who got paid
    pub quality_score: f64,        // Quality of submission
}
```

**Source**: `crates/roko-chain/src/marketplace.rs` lines 86-125

### 4-Level Dispute Resolution

When a poster disputes a submitted result, the dispute escalates through 4 levels:

```
BondEscalation (round 1) -> round 2 -> round 3 -> PeerJury -> GovernanceVote
```

1. **BondEscalation (rounds 1-3)**: Challenger and defender post escalating bonds. Either side can withdraw (forfeiting their bond) or escalate.
2. **PeerJury**: A randomly selected panel of agents votes on the outcome.
3. **GovernanceVote**: The full governance body votes (final appeal).

```rust
pub fn escalate_dispute(
    &mut self,
    job_id: &[u8; 32],
) -> Result<&DisputeLevel, MarketplaceError> {
    dispute.current_level = match &dispute.current_level {
        DisputeLevel::BondEscalation { round } => {
            if *round >= 3 {
                DisputeLevel::PeerJury { votes_for: 0, votes_against: 0 }
            } else {
                DisputeLevel::BondEscalation { round: round + 1 }
            }
        }
        DisputeLevel::PeerJury { .. } => DisputeLevel::GovernanceVote {
            proposal_id: *job_id,
        },
        // ...
    };
}
```

**Source**: `crates/roko-chain/src/marketplace.rs` lines 530-670

### Solidity Marketplace (BountyMarket.sol)

The on-chain Solidity contract (136 lines) implements a simpler lifecycle:

```solidity
contract BountyMarket {
    enum State { None, Open, Funded, Assigned, Submitted, Terminal }

    struct Job {
        address poster;
        uint256 bounty;
        uint64  deadline;      // unix seconds
        uint8   minTier;       // WorkerRegistry.Tier
        bytes32 specHash;      // Content-addressed job spec
        address worker;        // Set on assignment
        bytes32 resultHash;    // Set on submission
        State   state;
        bool    accepted;      // Set on resolution
    }

    function resolve(uint256 id, bool accepted) external {
        // Accept: bounty to worker + reputation++
        // Reject: refund to poster + slash worker 5%
        if (accepted) {
            bountyToken.transfer(j.worker, j.bounty);
            workerRegistry.updateReputation(j.worker, true);
        } else {
            bountyToken.transfer(j.poster, j.bounty);
            workerRegistry.updateReputation(j.worker, false);
            workerRegistry.slash(j.worker, workerRegistry.SLASH_QUALITY_REJECT(), 500);
        }
    }
}
```

The on-chain contract delegates resolution to an authorized `resolver` address (set at deploy time, can be updated to a `ConsortiumValidator` multi-sig). The off-chain Rust code handles the more complex 4-level dispute escalation.

**Source**: `contracts/src/BountyMarket.sol`

---

## 8. KORAI Token Economics (CHAIN-01)

### 1% Annual Lazy Demurrage

KORAI is an ERC-20 token with a distinctive property: **demurrage**. Token balances decay by 1% per year. This is a holding cost that discourages hoarding and encourages circulation.

The concept of demurrage currency was first proposed by Silvio Gesell in his 1916 work *Die Naturliche Wirtschaftsordnung* ("The Natural Economic Order") [13]. Gesell proposed "Freigeld" (free money) that would incur a periodic holding cost, arguing that money should share the perishability of the goods it is used to buy. The most famous practical experiment was the Worgl Schilling (1932-1933), where a small Austrian town issued demurrage-bearing notes that achieved dramatically higher velocity of money during the Great Depression.

KORAI applies this concept digitally: tokens that are actively used in the marketplace (paying for jobs, staking on domains) avoid effective demurrage because their balance is constantly being updated. Only idle balances decay.

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

Implementation from `crates/roko-chain/src/korai_token.rs`:

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
```

Demurrage is applied **lazily** (on read), not as periodic transactions. When a balance is read via `balance_of()`, the effective balance is computed from the stored balance and elapsed time. When a transfer or mint occurs, the demurrage is "materialized" first:

```rust
pub fn mint(&mut self, to: &str, amount: u256, pathway: EarningPathway, now: u64) {
    let entry = self.balances.entry(to.to_string())
        .or_insert_with(|| BalanceRecord::new(0, now));
    // Materialise existing demurrage before adding new tokens
    entry.materialise_demurrage(now, self.config.demurrage_rate);
    entry.stored_balance = entry.stored_balance.saturating_add(amount);
    entry.last_update = now;
    // ...
}
```

**Source**: `crates/roko-chain/src/korai_token.rs` lines 1-323

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

KORAI has a defined minting schedule inspired by Bitcoin's halvings:

```rust
pub struct EmissionSchedule {
    pub base_emission_per_block: f64,  // 100 KORAI/block initial
    pub blocks_per_epoch: u64,         // 2,628,000 (~1 year at 12s blocks)
    pub terminal_rate: f64,            // 1 KORAI/block floor
    pub max_supply: f64,               // 1 billion KORAI cap
    pub total_minted: f64,             // Running total
}
```

Emission halves each epoch:
- Epoch 0: 100 KORAI/block
- Epoch 1: 50 KORAI/block
- Epoch 2: 25 KORAI/block
- Epoch N: max(100 / 2^N, 1) KORAI/block

The terminal rate of 1 KORAI/block ensures perpetual low-level incentives even after most supply is distributed. This avoids the "incentive cliff" problem where validators lose motivation once block rewards approach zero.

```rust
pub fn rate_at_block(&self, block: u64) -> f64 {
    if self.total_minted >= self.max_supply { return 0.0; }
    let epoch = self.epoch_for_block(block);
    let halving_factor = 0.5_f64.powi(epoch as i32);
    let rate = self.base_emission_per_block * halving_factor;
    rate.max(self.terminal_rate)
}
```

**Testnet variant**: DAEJI token (same economics, different name/symbol, `KoraiTokenConfig::testnet()`).

**Source**: `crates/roko-chain/src/korai_token.rs` lines 433-657

**Spec reference**: `docs/v1/08-chain/02-korai-token-economics.md`, `docs/v1/14-identity-economy/10-korai-tokenomics.md`

---

## 9. X402 Micropayments Protocol (CHAIN-08)

### Protocol Flow

X402 enables agent-to-agent payments at the speed of HTTP, using the long-dormant HTTP 402 status code [14]. The HTTP 402 "Payment Required" status code was reserved in the original HTTP/1.1 specification (RFC 2616, 1999) for future use in micropayment systems, but remained unimplemented for over two decades. The x402 protocol, pioneered by Coinbase in 2025, finally gives it a concrete implementation [15].

The roko implementation follows this flow:

```
1. Client sends request to agent's HTTP endpoint
   POST /tools/call
   Content-Type: application/json

2. Agent responds with 402 Payment Required
   HTTP/1.1 402 Payment Required
   X-Payment-Request: {"recipient":"0xAgent","amount":500,"token":"0xKORAI",...}

3. Client signs an ERC-3009 transferWithAuthorization (gasless, off-chain)

4. Client retries with payment header
   POST /tools/call
   X-Payment-Authorization: {"from":"0xClient","to":"0xAgent","value":500,...}

5. Agent verifies authorization and serves the response
```

### ERC-3009: Gasless Transfers

The payment authorization uses ERC-3009 (`transferWithAuthorization`) [16], which allows a token holder to authorize a transfer by signing a message off-chain. Unlike a regular ERC-20 transfer that requires the sender to pay gas, ERC-3009 allows anyone to submit the signed authorization and execute the transfer. Key properties:

- **Atomic**: Unlike ERC-2612 (permit), which only authorizes an approval, ERC-3009 authorizes the complete transfer in one step
- **Non-sequential nonces**: Uses random `bytes32` nonces rather than sequential counters, allowing concurrent independent authorizations without conflicts
- **Time-bounded**: Each authorization has `validAfter` and `validBefore` timestamps

### Payment Request/Authorization Structs

```rust
pub struct PaymentRequest {
    pub recipient: Address,   // Agent's payment address
    pub amount: u256,         // Required KORAI amount
    pub token: Address,       // KORAI contract address
    pub nonce: u256,          // Replay protection
    pub deadline: u64,        // Block deadline for validity
    pub reason: String,       // Human-readable reason
}

pub struct PaymentAuthorization {
    pub from: Address,        // Payer (signer)
    pub to: Address,          // Payee (agent)
    pub value: u256,          // Authorized transfer amount
    pub valid_after: u64,     // Earliest valid timestamp
    pub valid_before: u64,    // Latest valid timestamp
    pub nonce: u256,          // Authorization nonce
    pub v: u8,                // ECDSA signature components
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
    // 2. Check amount >= requested
    // 3. Check recipient matches
    // 4. Check timestamp within valid window
    // 5. (Production: ECDSA signature verification via ecrecover)
}
```

### State Channels

For high-frequency interactions (e.g., an agent making hundreds of requests to an MCP server), X402 supports **state channels** [17] that reduce gas to exactly 2 transactions per session (open + close):

```rust
pub struct StateChannel {
    pub channel_id: [u8; 32],
    pub party_a: Address,              // Payer / client
    pub party_a_passport: u256,
    pub party_b: Address,              // Payee / agent
    pub party_b_passport: u256,
    pub deposit_a: u256,               // Total deposit from payer
    pub deposit_b: u256,               // Total deposit from payee
    pub nonce: u64,                    // Increments with each off-chain update
    pub balance_a: u256,               // Current payer balance
    pub balance_b: u256,               // Current payee balance
    pub state: ChannelLifecycle,       // Open -> Closing -> Closed
    pub challenge_window: u64,         // Blocks to challenge before close
}
```

Channel lifecycle:
1. **Open**: Both parties deposit funds. Off-chain balance proofs track micropayments.
2. **Closing**: Either party requests close. Challenge window starts (default 100 blocks).
3. **Closed**: After challenge period, final balances are settled on-chain.

Off-chain updates use signed balance proofs with a **conservation invariant** -- the sum of balances must always equal the total deposits:

```rust
pub fn update_channel(&mut self, proof: &BalanceProof) -> Result<(), X402Error> {
    // Nonce must be strictly increasing
    if proof.nonce <= channel.nonce { return Err(X402Error::InvalidNonce { ... }); }
    // Conservation: sum of balances must equal total deposits
    let total_deposit = channel.deposit_a + channel.deposit_b;
    let total_balance = proof.balance_a + proof.balance_b;
    if total_balance != total_deposit { return Err(X402Error::BalanceMismatch { ... }); }
    // Update state
    channel.nonce = proof.nonce;
    channel.balance_a = proof.balance_a;
    channel.balance_b = proof.balance_b;
    Ok(())
}
```

The strictly increasing nonce ensures that if a dispute occurs, the on-chain contract can determine which balance proof is the most recent. This is the same mechanism used by the Raiden Network [17] and Lightning Network for payment channel dispute resolution.

**Source**: `crates/roko-chain/src/x402.rs` (full file, 958 lines)

**Spec reference**: `docs/v1/08-chain/20-x402-micropayments.md`, `docs/v1/14-identity-economy/08-x402-micropayments.md`

---

## 10. ISFR Oracle (CHAIN-09)

### What Is ISFR?

ISFR (Intersubjective Fact Registry) is the agent economy's equivalent of SOFR/LIBOR -- a collective rate discovery mechanism. Agents submit rate observations for hierarchical market IDs, and the system computes a robust aggregate using **weighted median** with outlier exclusion.

### Rate Submission

Agents submit rate observations with confidence levels:

```rust
pub struct IsfrSubmission {
    pub submitter_id: u256,          // Agent's passport ID
    pub market_id: String,           // Hierarchical market identifier
    pub rate: f64,                   // Observed rate
    pub components: IsfrComponents,  // Breakdown by source
    pub confidence: f64,             // Submitter confidence [0, 1]
    pub timestamp: u64,              // When observed
}
```

### Weighted Median Aggregation

Unlike simple averaging (which is vulnerable to outlier manipulation), ISFR uses a **two-level weighted median** with 3-sigma outlier exclusion:

1. Collect all submissions for a market in the current epoch
2. Compute initial median and standard deviation
3. Exclude submissions more than 3 sigma from the median
4. Weight remaining submissions by: `submitter_reputation * confidence * stake_weight`
5. Compute the weighted median of the filtered set

```rust
pub fn aggregate(&self, submissions: &[IsfrSubmission], now: u64) -> Option<IsfrAggregate> {
    // ... filter by min_reputation ...
    // ... compute weighted median ...
    // ... 3-sigma outlier exclusion ...
}
```

The output includes confidence metrics:

```rust
pub struct IsfrAggregate {
    pub market_id: String,
    pub median_rate: f64,          // Weighted median
    pub std_deviation: f64,        // Standard deviation
    pub submission_count: usize,   // Total submissions
    pub excluded_count: usize,     // Outliers excluded
    pub confidence: f64,           // Aggregate confidence
    pub epoch: u64,                // Epoch identifier
    pub timestamp: u64,            // Computation time
}
```

Configuration:
- Epoch duration: 8 hours (default)
- Minimum submissions: configurable (avoids aggregation from too few data points)
- Minimum submitter reputation: 0.5 (agents must have demonstrated competence)

### Solidity ISFROracle Contract

The on-chain contract (96 lines) stores epoch-keyed rate submissions from authorized keepers:

```solidity
contract ISFROracle {
    bytes32 public constant KEEPER_ROLE = keccak256("KEEPER_ROLE");

    struct Rate {
        uint256 epochId;
        uint256 compositeBps;     // Composite rate in basis points
        uint256 lendingBps;       // Lending rate component
        uint256 structuredBps;    // Structured products component
        uint256 fundingBps;       // Funding rate component
        uint256 stakingBps;       // Staking yield component
        uint256 confidenceBps;    // Confidence score in bps
        uint64  timestamp;
        address submitter;
    }

    function submitRate(
        uint256 epochId,
        uint256 compositeBps,
        uint256 lendingBps,
        uint256 structuredBps,
        uint256 fundingBps,
        uint256 stakingBps,
        uint256 confidenceBps
    ) external onlyKeeper { ... }
}
```

The weighted median computation happens off-chain in the Rust `isfr.rs` module (1277 lines); only the final aggregated rate is submitted on-chain by authorized keepers. The `RoleRegistry.sol` contract manages `KEEPER_ROLE` authorization.

**Source**: `crates/roko-chain/src/isfr.rs`, `contracts/src/ISFROracle.sol`

**Spec reference**: `docs/v1/08-chain/21-isfr-clearing-settlement.md`, `docs/v1/14-identity-economy/13-isfr-clearing-settlement.md`

---

## 11. Solidity Contracts On-Chain

The `contracts/src/` directory contains 13 Solidity contracts that implement the on-chain portions of the system:

| Contract | Lines | Purpose |
|----------|-------|---------|
| `IdentityRegistry.sol` | 467 | Soulbound ERC-721 passports with staking, tiers, prompt hash timelock |
| `ReputationRegistry.sol` | 295 | 7-domain EMA reputation with decay, slashing, peer feedback |
| `BountyMarket.sol` | 136 | Job escrow lifecycle: Open -> Funded -> Assigned -> Submitted -> Terminal |
| `WorkerRegistry.sol` | 233 | Worker bonds, EMA reputation (alpha=0.2), 4 tiers, 30-day decay |
| `AgentRegistry.sol` | -- | ERC-8004 agent identity: capabilities, passport hash, heartbeat liveness |
| `ISFROracle.sol` | 96 | Epoch-keyed rate storage from authorized keepers |
| `ISFRBountyPool.sol` | -- | Incentive pool for ISFR rate submissions |
| `ConsortiumValidator.sol` | -- | Multi-sig validation for dispute resolution |
| `FeeDistributor.sol` | -- | Platform fee distribution logic |
| `InsightBoard.sol` | -- | On-chain knowledge attestation board |
| `RoleRegistry.sol` | -- | Role-based access control (KEEPER_ROLE, etc.) |
| `ValidationRegistry.sol` | -- | Gate-score validation records |
| `MockERC20.sol` | -- | Test ERC-20 token for development |

### WorkerRegistry.sol Detail

The WorkerRegistry implements the on-chain reputation model with notable constants:

```solidity
contract WorkerRegistry {
    uint256 public constant SCALE = 1_000_000;      // 6-decimal fixed point
    uint256 public constant ALPHA_NUM = 200_000;     // alpha = 0.2
    uint256 public constant MIN_BOND = 1_000 ether;  // Minimum stake
    uint256 public constant DECAY_PERIOD = 30 days;  // Half-life

    enum Tier { Unregistered, Probation, Standard, Trusted, Elite }

    // Tier thresholds (in SCALE units, so 350_000 = 0.35):
    // Probation:  reputation < 350,000
    // Standard:   350,000 <= reputation < 550,000
    // Trusted:    550,000 <= reputation < 800,000
    // Elite:      reputation >= 800,000

    // Slash reason codes (bond basis points):
    uint8 public constant SLASH_MISSED_DEADLINE = 1;  //  1% of bond
    uint8 public constant SLASH_QUALITY_REJECT  = 2;  //  5% of bond
    uint8 public constant SLASH_ABANDONMENT     = 3;  // 10% of bond
}
```

Note the differences from the Rust off-chain model: WorkerRegistry uses 5 tiers (vs 4 in Rust), a fixed alpha of 0.2 (vs adaptive alpha), and integer-based halvings for decay (vs continuous exponential decay). These divergences reflect the gas constraints of on-chain computation -- the Rust code is the authoritative specification.

**Source**: `contracts/src/WorkerRegistry.sol`

---

## 12. IronClaw + NEAR Integration

IronClaw is a NEAR Protocol project, which makes the integration path for roko's on-chain reputation system especially natural. NEAR's Rust-based smart contract SDK (`near-sdk-rs`) means the Rust off-chain logic in `roko-chain` can be adapted to NEAR contracts with relatively minimal porting effort compared to rewriting Solidity contracts. NEAR's NEP-171 NFT standard [18] provides the foundation for soulbound passports, and NEAR's sub-second finality and low transaction costs make on-chain reputation updates practical in ways that would be cost-prohibitive on Ethereum mainnet.

### A. NEAR-Native Agent Identity

**Where**: New module `src/identity/` or `crates/ironclaw_identity/`

IronClaw agents operating on NEAR Protocol would get soulbound identity passports as NEAR NFTs (NEP-171 with transfer restrictions). The NEAR community has been discussing soulbound token standards [18], and a passport contract could be deployed as a Rust smart contract using `near-sdk-rs`, adapted from the existing Solidity `IdentityRegistry.sol`.

Key adaptations from Solidity to NEAR:
- **Soulbound enforcement**: Override `nft_transfer` and `nft_transfer_call` to panic, similar to how the Solidity `transferFrom` reverts with `Soulbound()`
- **Storage staking**: NEAR's storage staking model (accounts pay for storage they consume) naturally aligns with the passport's staking model
- **Cross-contract calls**: NEAR's async cross-contract call mechanism replaces Solidity's synchronous contract interactions

Suggested integration points:
- `src/config/`: Add `NEAR_IDENTITY_CONTRACT` environment variable for the passport contract address
- `src/agent/session.rs`: On session start, verify the agent's passport is active and not suspended
- IronClaw's existing heartbeat system (`src/workspace/`) could periodically call the passport's heartbeat function

```rust
// Proposed: ironclaw identity bridge
pub struct NearIdentityBridge {
    /// NEAR RPC endpoint
    pub rpc_url: String,
    /// Passport contract account ID (e.g., "passport.ironclaw.near")
    pub passport_contract: AccountId,
    /// Local agent's passport ID (cached after first lookup)
    pub passport_id: Option<u256>,
}

impl NearIdentityBridge {
    /// Register the agent on-chain if not already registered
    pub async fn ensure_registered(&mut self, capabilities: u64) -> Result<u256, IdentityError>;
    /// Verify another agent's passport before delegating work
    pub async fn verify_passport(&self, passport_id: u256) -> Result<PassportInfo, IdentityError>;
    /// Send heartbeat to maintain active status
    pub async fn heartbeat(&self) -> Result<(), IdentityError>;
}
```

### B. Off-Chain Reputation for Extensions and Tools

**Where**: `src/registry/`, `src/tools/wasm/`

The 7-domain EMA reputation engine can be used purely off-chain to track the reliability of IronClaw's installed extensions, MCP servers, and WASM tools -- no blockchain needed. This is the lowest-friction integration path and provides immediate value.

```rust
// Proposed: extension reputation tracker
pub struct ExtensionReputation {
    /// Local ReputationRegistry instance (from roko-chain, used as a library)
    registry: ReputationRegistry,
    /// Map extension name -> virtual passport ID
    extension_passports: HashMap<String, u256>,
}

impl ExtensionReputation {
    /// Record a successful tool call
    pub fn record_success(&mut self, extension: &str, domain: &str, latency_ms: u64);
    /// Record a failed tool call
    pub fn record_failure(&mut self, extension: &str, domain: &str, error: &ToolError);
    /// Get effective reputation (with decay)
    pub fn get_reputation(&self, extension: &str) -> HashMap<String, f64>;
    /// Auto-disable extensions below threshold
    pub fn check_thresholds(&self) -> Vec<DisableRecommendation>;
}
```

Domain mapping for extensions:
- `reliability` -> tracks success/failure rate (maps to "operations" domain)
- `performance` -> tracks latency and resource usage (maps to "operations" domain)
- `safety` -> tracks safety violations (maps to "security" domain)
- `accuracy` -> tracks output quality (maps to "knowledge" domain)

Extensions with reputation below 0.3 in any domain would trigger a warning. Below 0.2 would auto-disable with a notification to the user. The 30-day half-life decay means that a temporarily broken extension that gets fixed will naturally recover its reputation over time.

IronClaw's existing evaluation framework (`src/evaluation/`) already performs rule-based and LLM-based success evaluation that could feed directly into the reputation update pipeline.

### C. Multi-Agent Trust Scoring

**Where**: Future `src/agent/multi_agent.rs` or `crates/ironclaw_multi_agent/`

When IronClaw supports multi-agent collaboration (delegating sub-tasks to other IronClaw instances or external agents), the TraceRank algorithm determines which agents get which tasks. An agent that has been paid by many reputable agents for quality work will rank higher than an unknown agent.

The blend formula provides a practical trust score:
```
trust = 0.7 * direct_ema_reputation + 0.3 * trace_rank_score
```

IronClaw's existing cost estimation framework (`src/estimation/`) with its EMA learning could be extended to track per-agent cost and quality metrics, feeding into the TraceRank graph.

### D. Knowledge Attestation

**Where**: `src/workspace/`

IronClaw's workspace memory entries could be attested on-chain as content-addressed knowledge anchors. The existing `memory_write` tool could optionally submit a hash of the memory entry to a NEAR contract, creating a verifiable timestamp for when the knowledge was recorded.

This is useful for:
- Proving an agent had certain knowledge at a certain time
- Building verifiable knowledge graphs across multiple IronClaw instances
- Knowledge marketplace where agents sell access to high-value insights

NEAR's storage is well-suited for this because it provides content-addressable storage with predictable costs.

### E. NEAR Skills/Tools Marketplace

**Where**: `src/registry/`, `src/skills/`

IronClaw's skills and tools could be listed on a NEAR marketplace with reputation and NEAR-native micropayments. A skill publisher would:
1. Publish the skill WASM artifact to IPFS/Arweave
2. List it on a NEAR marketplace contract with price and capability requirements
3. IronClaw instances discover and install skills through the marketplace
4. Usage triggers x402-style micropayments on NEAR (using NEAR tokens instead of KORAI)

IronClaw's existing skill infrastructure (`src/skills/`, with trust model, selection pipeline, and attenuation) provides the client-side framework. The marketplace contract would add the discovery, payment, and reputation dimensions.

### F. X402 for IronClaw MCP Servers

**Where**: `src/tools/mcp/client.rs`

IronClaw's MCP client could implement the X402 protocol, allowing it to pay for premium MCP tool calls. When an MCP server responds with HTTP 402, IronClaw would:
1. Check its NEAR wallet balance
2. Sign a transfer authorization (using NEAR's access key system instead of ECDSA)
3. Retry the request with the payment header
4. Track spending per MCP server for cost management

NEAR's access key model (full-access vs function-call access keys) provides a natural mechanism for authorizing micropayments: a function-call access key limited to the payment contract could authorize transfers up to a configurable budget without requiring the agent's full-access key for each payment.

---

## 13. Complexity Assessment

### Implementation Tiers

**Tier 1: Off-chain reputation engine (no blockchain)**
- Lines of code: ~600-800
- Effort: 1-2 weeks
- What: Port `ReputationRegistry` + `CollusionDetector` to IronClaw for local extension/tool tracking
- Dependencies: None beyond existing IronClaw crates
- Risk: Low
- Integration points: `src/registry/`, `src/tools/wasm/`, `src/evaluation/`

**Tier 2: NEAR identity integration**
- Lines of code: ~500-700 (Rust client) + ~300-500 (NEAR contract in Rust)
- Effort: 2-3 weeks
- What: Soulbound passport on NEAR (NEP-171 with transfer restrictions), heartbeat integration, passport verification
- Dependencies: `near-sdk` (v5+), `near-jsonrpc-client`, `near-crypto`
- Risk: Medium (NEAR contract needs auditing)
- Integration points: `src/config/`, `src/agent/session.rs`, `src/workspace/`

**Tier 3: Multi-agent trust scoring**
- Lines of code: ~400-600
- Effort: 1-2 weeks
- What: TraceRank computation + reputation blending for agent delegation decisions
- Dependencies: Tier 1 (reputation engine)
- Risk: Low
- Integration points: `src/estimation/`, future multi-agent module

**Tier 4: On-chain marketplace + payments**
- Lines of code: ~1000-1500 (Rust) + ~500-800 (NEAR contracts)
- Effort: 4-6 weeks
- What: Full bounty marketplace with escrow, X402 micropayments on NEAR
- Dependencies: Tiers 1-3
- Risk: High (financial contracts require formal verification)
- Integration points: `src/tools/mcp/client.rs`, `src/registry/`, `src/skills/`

### Recommended Path

Start with Tier 1: the off-chain reputation engine gives IronClaw immediate value (tracking which extensions are reliable) without any blockchain dependency. The same `ReputationRegistry` code can later be backed by on-chain storage when NEAR integration is added.

The progression from off-chain to on-chain is designed to be incremental:
1. **Tier 1** proves the reputation model works with real extension data
2. **Tier 2** adds verifiable identity without requiring marketplace infrastructure
3. **Tier 3** enables multi-agent collaboration with trust scoring
4. **Tier 4** opens the marketplace -- only after the reputation and identity layers are battle-tested

### Key Dependencies

- `near-sdk` (NEAR smart contract SDK, for Tier 2+)
- `near-jsonrpc-client` (NEAR RPC client, for Tier 2+)
- `near-crypto` (key management and signing, for Tier 2+)
- Existing IronClaw crates: `ironclaw_llm` (for quality evaluation in feedback), `ironclaw_safety` (for violation detection)

### Source File Reference

All source code cited in this document:

| Path | What | Lines |
|------|------|-------|
| `crates/roko-chain/src/lib.rs` | Module declarations and public exports | 134 |
| `crates/roko-chain/src/types.rs` | Chain primitives (TxHash, Receipt, ChainError) | -- |
| `crates/roko-chain/src/phase2.rs` | Phase 2 type definitions | ~85KB |
| `crates/roko-chain/src/agent_registry.rs` | Soulbound passport CHAIN-02 | 785 |
| `crates/roko-chain/src/reputation_registry.rs` | 7-domain reputation CHAIN-03 | 1179 |
| `crates/roko-chain/src/trace_rank.rs` | PageRank trust propagation P1-02 | 508 |
| `crates/roko-chain/src/collusion.rs` | Bron-Kerbosch collusion detection P2-11 | 379 |
| `crates/roko-chain/src/marketplace.rs` | Spore marketplace CHAIN-04 | 1096 |
| `crates/roko-chain/src/korai_token.rs` | KORAI with demurrage CHAIN-01 | 657 |
| `crates/roko-chain/src/x402.rs` | HTTP 402 micropayments CHAIN-08 | 958 |
| `crates/roko-chain/src/isfr.rs` | ISFR weighted-median oracle CHAIN-09 | 1277 |
| `contracts/src/IdentityRegistry.sol` | On-chain soulbound passport | 467 |
| `contracts/src/ReputationRegistry.sol` | On-chain reputation with decay | 295 |
| `contracts/src/BountyMarket.sol` | On-chain job escrow | 136 |
| `contracts/src/WorkerRegistry.sol` | On-chain worker bonds + reputation | 233 |
| `contracts/src/ISFROracle.sol` | On-chain rate oracle | 96 |

---

## 14. Academic References

[1] J. R. Douceur, "The Sybil Attack," *Proc. 1st International Workshop on Peer-to-Peer Systems (IPTPS)*, 2002. The foundational paper defining the Sybil attack, where a single entity creates multiple pseudonymous identities to subvert a reputation system.

[2] E. G. Weyl, P. Ohlhaver, V. Buterin, "Decentralized Society: Finding Web3's Soul," SSRN, May 2022. [https://papers.ssrn.com/sol3/papers.cfm?abstract_id=4105763](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=4105763). The paper proposing soulbound tokens (SBTs) as non-transferable tokens representing commitments, credentials, and affiliations, encoding the trust networks of the real economy.

[3] S. J. Roberts, "Exponential smoothing: The state of the art -- Part II," *International Journal of Forecasting*, vol. 20, no. 1, pp. 1-4, 2004. Standard reference for exponential moving average (EMA) / exponential smoothing methods in time-series analysis. The EMA formula `R_new = alpha * F + (1 - alpha) * R_old` is a first-order exponential smoothing filter.

[4] J. Duchi, E. Hazan, Y. Singer, "Adaptive Subgradient Methods for Online Learning and Stochastic Optimization," *JMLR*, vol. 12, pp. 2121-2159, 2011. The concept of adaptive learning rates (varying step sizes based on accumulated gradient history) which inspired the adaptive alpha approach in the reputation system.

[5] E. Rutherford, "Radioactive Substances and their Radiations," 1913. The half-life decay model `N(t) = N_0 * 0.5^(t / t_half)` originates from nuclear physics but is widely applied in pharmacokinetics, signal processing, and (here) reputation decay.

[6] L. Page, S. Brin, R. Motwani, T. Winograd, "The PageRank Citation Ranking: Bringing Order to the Web," Stanford InfoLab Technical Report, 1998. The original PageRank paper describing iterative power iteration with damping/teleportation for computing importance scores on directed graphs. TraceRank adapts this to quality-weighted payment edges.

[7] S. D. Kamvar, M. T. Schlosser, H. Garcia-Molina, "The EigenTrust Algorithm for Reputation Management in P2P Networks," *Proc. 12th International World Wide Web Conference (WWW '03)*, Budapest, 2003. [https://nlp.stanford.edu/pubs/eigentrust.pdf](https://nlp.stanford.edu/pubs/eigentrust.pdf). Formalized eigenvector-based trust propagation in peer-to-peer networks, computing a global trust value for each peer based on transaction history. TraceRank is an adaptation of this approach with quality-weighted edges.

[8] O. Perron, "Zur Theorie der Matrices," *Mathematische Annalen*, vol. 64, pp. 248-263, 1907; G. Frobenius, "Ueber Matrizen aus nicht negativen Elementen," *Sitzungsberichte der Preussischen Akademie der Wissenschaften*, pp. 456-477, 1912. The Perron-Frobenius theorem guarantees that a positive stochastic matrix has a unique largest eigenvalue with a corresponding positive eigenvector -- the mathematical foundation for PageRank convergence.

[9] C. Bron, J. Kerbosch, "Algorithm 457: Finding All Cliques of an Undirected Graph," *Communications of the ACM*, vol. 16, no. 9, pp. 575-577, September 1973. The original algorithm for enumerating all maximal cliques in an undirected graph. The pivoting optimization reduces worst-case complexity to O(3^(n/3)). Used here for collusion ring detection.

[10] M. Mitzenmacher, "The Power of Two Choices in Randomized Load Balancing," *IEEE Transactions on Parallel and Distributed Systems*, vol. 12, no. 10, pp. 1094-1104, 2001. The theoretical foundation for the RandomVRF hiring model: sampling two random candidates and picking the better one achieves O(log log N) maximum load, an exponential improvement over purely random assignment.

[11] W. Vickrey, "Counterspeculation, Auctions, and Competitive Sealed Tenders," *Journal of Finance*, vol. 16, no. 1, pp. 8-37, 1961. The foundational paper on second-price sealed-bid auctions, proving that truthful bidding is the dominant strategy when the winner pays the second-highest bid. Vickrey received the Nobel Prize in Economics (1996) for this work.

[12] E. H. Clarke, "Multipart Pricing of Public Goods," *Public Choice*, vol. 11, pp. 17-33, 1971; T. Groves, "Incentives in Teams," *Econometrica*, vol. 41, no. 4, pp. 617-631, 1973. The generalization of Vickrey's second-price mechanism to multi-item settings, forming the VCG (Vickrey-Clarke-Groves) mechanism design framework used in the marketplace's auction model.

[13] S. Gesell, *Die Naturliche Wirtschaftsordnung durch Freiland und Freigeld* (The Natural Economic Order), 1916. Proposed Freigeld ("free money") bearing a demurrage charge (~5% annually) to discourage hoarding and incentivize circulation. The KORAI token's 1% annual demurrage is a digital implementation of this concept. Keynes acknowledged the soundness of Gesell's insight in *The General Theory* (1936), ch. 23.

[14] R. Fielding et al., "Hypertext Transfer Protocol -- HTTP/1.1," RFC 2616, IETF, June 1999. Defined the HTTP 402 "Payment Required" status code, reserved for future use. The roko X402 protocol provides the concrete implementation that the RFC anticipated.

[15] Coinbase, "x402: An Open Protocol for HTTP-Native Payments," 2025. [https://docs.cdp.coinbase.com/x402/core-concepts/http-402](https://docs.cdp.coinbase.com/x402/core-concepts/http-402). The production x402 protocol specification enabling machine-to-machine payments via HTTP 402 with stablecoin support across EVM and Solana networks.

[16] P. Becker, "ERC-3009: Transfer With Authorization," Ethereum Improvement Proposals, 2020. [https://github.com/ethereum/EIPs/issues/3010](https://github.com/ethereum/EIPs/issues/3010). The standard for gasless ERC-20 transfers via off-chain signed authorizations (`transferWithAuthorization`), using non-sequential random nonces for concurrency. Implemented natively by USDC.

[17] Raiden Network, "Payment Channels and State Channels," [https://raiden.network/101.html](https://raiden.network/101.html). State channels enable off-chain transactions with on-chain dispute resolution, reducing gas costs to 2 transactions per session (open + close). The concept was independently developed as the Lightning Network for Bitcoin and Raiden for Ethereum.

[18] NEAR Protocol, "NEP-171: Non-Fungible Token Standard," [https://nomicon.io/Standards/Tokens/NonFungibleToken/Core](https://nomicon.io/Standards/Tokens/NonFungibleToken/Core). NEAR's NFT standard (analogous to ERC-721), which provides the foundation for soulbound passport tokens. Community discussions on a dedicated soulbound token NEP are ongoing at [https://gov.near.org/t/discussion-of-soulbound-token-standard/31223](https://gov.near.org/t/discussion-of-soulbound-token-standard/31223).

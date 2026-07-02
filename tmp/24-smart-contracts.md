# 24 -- On-Chain Smart Contract Architecture for AI Agent Infrastructure

> Roko's Solidity contract suite provides the economic and identity substrate
> for autonomous AI agents: verifiable identity, trustless work coordination,
> stake-backed reputation, knowledge curation, bounty escrow, validation
> committees, oracle data feeds, and fee distribution. This document is a
> complete walkthrough of every contract, the EVM simulator that hosts them,
> the chain-watching agent that reacts to on-chain events, and a concrete
> analysis of what a NEAR port would look like for the IronClaw project.

**Source repository**: `/Users/will/dev/nunchi/roko/roko/`

---

## Table of Contents

1. [Why AI Agents Need On-Chain Infrastructure](#1-why-ai-agents-need-on-chain-infrastructure)
2. [Contract Suite Overview](#2-contract-suite-overview)
3. [Build System and Dependencies](#3-build-system-and-dependencies)
4. [Contract-by-Contract Reference](#4-contract-by-contract-reference)
   - 4.1 [MockERC20 -- Test Token](#41-mockerc20----test-token)
   - 4.2 [RoleRegistry -- Access Control](#42-roleregistry----access-control)
   - 4.3 [AgentRegistry -- Lightweight Agent Identity](#43-agentregistry----lightweight-agent-identity)
   - 4.4 [IdentityRegistry -- ERC-8004 Soulbound Passports](#44-identityregistry----erc-8004-soulbound-passports)
   - 4.5 [WorkerRegistry -- Stake, Reputation, Tiers](#45-workerregistry----stake-reputation-tiers)
   - 4.6 [ReputationRegistry -- Domain-Specific Reputation](#46-reputationregistry----domain-specific-reputation)
   - 4.7 [BountyMarket -- Programmable Escrow](#47-bountymarket----programmable-escrow)
   - 4.8 [ConsortiumValidator -- 2-of-3 Validation Committee](#48-consortiumvalidator----2-of-3-validation-committee)
   - 4.9 [ValidationRegistry -- Work Proofs and Attestations](#49-validationregistry----work-proofs-and-attestations)
   - 4.10 [InsightBoard -- On-Chain Knowledge with Pheromone Curation](#410-insightboard----on-chain-knowledge-with-pheromone-curation)
   - 4.11 [ISFROracle -- Interest Rate Oracle](#411-isfroracle----interest-rate-oracle)
   - 4.12 [ISFRBountyPool -- Oracle Keeper Rewards](#412-isfrbountypool----oracle-keeper-rewards)
   - 4.13 [FeeDistributor -- Multi-Party Fee Splitting](#413-feedistributor----multi-party-fee-splitting)
5. [Deployment Script and Contract Wiring](#5-deployment-script-and-contract-wiring)
6. [Contract Dependency Graph](#6-contract-dependency-graph)
7. [The EVM Simulator: mirage-rs](#7-the-evm-simulator-mirage-rs)
   - 7.1 [What mirage-rs Does](#71-what-mirage-rs-does)
   - 7.2 [ERC-8004 Bootstrap and ForkState](#72-erc-8004-bootstrap-and-forkstate)
   - 7.3 [HDC Precompile at 0xA0C](#73-hdc-precompile-at-0xa0c)
   - 7.4 [Knowledge Layer Integration](#74-knowledge-layer-integration)
8. [The Chain Watcher: roko-chain-watcher](#8-the-chain-watcher-roko-chain-watcher)
   - 8.1 [Architecture](#81-architecture)
   - 8.2 [Reaction Rules](#82-reaction-rules)
   - 8.3 [Block Observer](#83-block-observer)
9. [Rust-Side Contract Integration: roko-chain](#9-rust-side-contract-integration-roko-chain)
10. [Test Suite](#10-test-suite)
11. [Porting to NEAR](#11-porting-to-near)
    - 11.1 [Account Model Differences](#111-account-model-differences)
    - 11.2 [Storage Staking Economics](#112-storage-staking-economics)
    - 11.3 [Gas Budget Constraints](#113-gas-budget-constraints)
    - 11.4 [The Aurora Shortcut](#114-the-aurora-shortcut)
    - 11.5 [Native NEAR Contract Sketches](#115-native-near-contract-sketches)
    - 11.6 [Key Porting Challenges](#116-key-porting-challenges)
12. [IronClaw Integration Plan](#12-ironclaw-integration-plan)
    - 12.1 [What IronClaw Needs from On-Chain Infrastructure](#121-what-ironclaw-needs-from-on-chain-infrastructure)
    - 12.2 [Phase 1: Agent Identity on NEAR](#122-phase-1-agent-identity-on-near)
    - 12.3 [Phase 2: Reputation and Work Coordination](#123-phase-2-reputation-and-work-coordination)
    - 12.4 [Phase 3: Knowledge Layer](#124-phase-3-knowledge-layer)
    - 12.5 [Integration Architecture](#125-integration-architecture)
13. [Security Considerations](#13-security-considerations)
14. [References](#14-references)

---

## 1. Why AI Agents Need On-Chain Infrastructure

Traditional AI agents run in isolated processes with no way to prove their
identity, no mechanism for trustless economic coordination, and no verifiable
track record. When agents need to collaborate -- delegating tasks, sharing
knowledge, resolving disputes -- there is no neutral substrate that all
parties can trust.

On-chain infrastructure solves three fundamental problems:

**Verifiable Identity.** An agent's identity is anchored to a cryptographic
key pair and recorded in an immutable registry. Its capabilities, system
prompt hash, and TEE attestation are all publicly auditable. No central
authority can fabricate or revoke an identity without on-chain evidence.
Roko implements this through ERC-8004 soulbound passports
(`IdentityRegistry.sol`) and a lighter-weight heartbeat-based registry
(`AgentRegistry.sol`). The ERC-8004 standard -- proposed specifically for
AI agent identity [1] -- establishes three on-chain registries for Identity,
Reputation, and Validation, making each agent's identity a non-transferable
on-chain credential analogous to the ERC-5192 Minimal Soulbound NFT
interface [2] but extended with capability bitmasks, TEE attestations, and
domain staking.

**Trustless Coordination.** When agent A posts a bounty and agent B claims
it, neither party needs to trust the other. The bounty funds are locked in
escrow (`BountyMarket.sol`), a committee of validators evaluates the work
(`ConsortiumValidator.sol`), and the outcome triggers an automatic
settlement -- bounty to the worker if accepted, refund to the poster if
rejected, with reputation updates in both cases. No intermediary can steal
the funds or bias the outcome.

**Economic Incentives.** Reputation is not a badge -- it is an economic
signal backed by staked tokens. Workers bond tokens to register
(`WorkerRegistry.sol`, minimum 1,000 DAEJI). Poor performance triggers
slashing (5% of bond for rejected work). Reputation decays over time via
exponential moving average (EMA), so agents cannot rest on past performance
[3]. The fee distribution contract (`FeeDistributor.sol`) splits payments
across validators (40%), data providers (30%), the performing agent (20%),
and protocol treasury (10%).

These three properties -- identity, coordination, incentives -- compose into
an agent economy where autonomous software can participate in markets,
build reputations, and be held accountable, all without human intermediaries.

---

## 2. Contract Suite Overview

The contract suite contains 13 Solidity files in
`/Users/will/dev/nunchi/roko/roko/contracts/src/`:

| # | Contract | Purpose | Dependencies |
|---|----------|---------|--------------|
| 1 | `MockERC20.sol` | Test token (DAEJI) with open mint | OpenZeppelin ERC20 |
| 2 | `RoleRegistry.sol` | RBAC for ISFR contracts | None |
| 3 | `AgentRegistry.sol` | Lightweight agent identity + heartbeat | None |
| 4 | `IdentityRegistry.sol` | ERC-8004 soulbound passport + domain staking | IERC20Minimal (local) |
| 5 | `WorkerRegistry.sol` | Stake bonds + EMA reputation + tiers | OZ IERC20 |
| 6 | `ReputationRegistry.sol` | Multi-domain reputation with adaptive EMA | IdentityRegistry |
| 7 | `BountyMarket.sol` | 4-state programmable escrow | OZ IERC20, WorkerRegistry |
| 8 | `ConsortiumValidator.sol` | 2-of-3 validation committee | WorkerRegistry, BountyMarket |
| 9 | `ValidationRegistry.sol` | Work proofs + validator attestations | IdentityRegistry |
| 10 | `InsightBoard.sol` | On-chain knowledge with pheromone curation | OZ IERC20 |
| 11 | `ISFROracle.sol` | Interest rate oracle with epoch submissions | RoleRegistry |
| 12 | `ISFRBountyPool.sol` | Keeper reward pool for oracle submissions | RoleRegistry, OZ IERC20 |
| 13 | `FeeDistributor.sol` | Multi-party fee splitting (4 buckets) | OZ IERC20 |

All contracts target Solidity `^0.8.26` and use OpenZeppelin for ERC20 and
access-control primitives. Note that `IdentityRegistry` defines its own
minimal `IERC20Minimal` interface inline (only `transfer` and `transferFrom`)
rather than importing the full OpenZeppelin `IERC20`.

---

## 3. Build System and Dependencies

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/foundry.toml`

The project uses [Foundry](https://www.getfoundry.sh/) (forge) for
compilation and testing [4]. The Foundry configuration targets the Shanghai
EVM specification to match mirage-rs's `MIRAGE_EVM_SPEC`, which supports
`PUSH0` but avoids Cancun-only opcodes (`MCOPY`, `TSTORE`, `TLOAD`):

```toml
[profile.default]
src = "src"
out = "out"
libs = ["lib"]
solc = "0.8.26"
evm_version = "shanghai"
optimizer = true
optimizer_runs = 200
via_ir = false
auto_detect_solc = false
broadcast = "broadcast"
bytecode_hash = "none"
cbor_metadata = false

[rpc_endpoints]
mirage = "${ROKO_MIRAGE_URL}"
local = "http://127.0.0.1:8545"

[fuzz]
runs = 256
```

The `bytecode_hash = "none"` and `cbor_metadata = false` settings skip the
bytecode hash suffix so that `CREATE2` addresses are deterministic across
builds -- important for the ERC-8004 bootstrap contracts that mirage-rs
deploys at well-known addresses.

**Remappings** (`/Users/will/dev/nunchi/roko/roko/contracts/remappings.txt`):

```
forge-std/=lib/forge-std/src/
@openzeppelin/contracts/=lib/openzeppelin-contracts/contracts/
```

The contract suite depends on:
- `forge-std` for testing infrastructure (`Test`, `console2`, `Script`)
- `@openzeppelin/contracts` for `ERC20`, `IERC20`

---

## 4. Contract-by-Contract Reference

### 4.1 MockERC20 -- Test Token

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/MockERC20.sol`

A minimal ERC20 token named "DAEJI" with an open `mint()` function. Used as
the settlement token across all contracts in the demo environment.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MockERC20 is ERC20 {
    uint8 private immutable _decimals;

    constructor(string memory name_, string memory symbol_, uint8 decimals_)
        ERC20(name_, symbol_)
    {
        _decimals = decimals_;
    }

    function decimals() public view override returns (uint8) {
        return _decimals;
    }

    /// Anyone can mint -- test-environment only.
    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}
```

**Storage**: Inherits OpenZeppelin's ERC20 storage (balances, allowances,
totalSupply). Adds a single immutable `_decimals`.

**Security**: The open `mint()` is intentional for testing. The contract
NatSpec explicitly warns "Never deploy to mainnet."

---

### 4.2 RoleRegistry -- Access Control

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/RoleRegistry.sol`

A lightweight RBAC contract that maps `bytes32 role => address => bool`.
Used by the ISFR oracle contracts (ISFROracle and ISFRBountyPool) for
keeper and oracle role management.

```solidity
contract RoleRegistry {
    address public admin;
    mapping(bytes32 => mapping(address => bool)) private _roles;

    event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender);
    event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender);
    event AdminTransferred(address indexed oldAdmin, address indexed newAdmin);

    error NotAdmin();

    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }

    constructor(address admin_) {
        admin = admin_;
    }

    function grantRole(bytes32 role, address account) external onlyAdmin { ... }
    function revokeRole(bytes32 role, address account) external onlyAdmin { ... }
    function hasRole(bytes32 role, address account) external view returns (bool) { ... }
    function transferAdmin(address newAdmin) external onlyAdmin { ... }
}
```

**Design choices**: Unlike OpenZeppelin's `AccessControl`, this is a
standalone mapping with a single admin -- no role hierarchy, no role-admin
concept. This simplicity is appropriate for the ISFR subsystem where only
two roles are needed (`KEEPER_ROLE` and `ORACLE_ROLE`). The constructor
takes an explicit `admin_` address rather than defaulting to `msg.sender`,
enabling deployment by a factory that installs a different admin.

---

### 4.3 AgentRegistry -- Lightweight Agent Identity

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/AgentRegistry.sol`

Tracks `address -> Agent` with capabilities, passport hash, and heartbeat
liveness. A complementary on-chain counterpart to the `0xA09` precompile
exposed by mirage-rs. Agents call `register()` once, then `heartbeat()`
periodically; `isActive()` returns true within a 200-block liveness window.

```solidity
contract AgentRegistry {
    struct Agent {
        string capabilities;
        bytes32 passportHash;
        uint64 registeredAt;
        uint64 lastHeartbeat;
        bool exists;
    }

    uint64 public constant LIVENESS_WINDOW = 200;

    mapping(address => Agent) private _agents;
    address[] private _registered;

    event AgentRegistered(address indexed agent, bytes32 passportHash, string capabilities);
    event AgentHeartbeat(address indexed agent, uint64 blockNumber);
    event AgentCapabilitiesUpdated(address indexed agent, string capabilities);

    error AlreadyRegistered();
    error NotRegistered();

    function register(string calldata capabilities, bytes32 passportHash) external { ... }
    function heartbeat() external { ... }
    function updateCapabilities(string calldata capabilities) external { ... }
    function isActive(address agent) external view returns (bool) { ... }
    function getAgent(address agent) external view returns (Agent memory) { ... }
    function registeredCount() external view returns (uint256) { ... }
    function registeredAt(uint256 index) external view returns (address) { ... }
}
```

**Key behavior**: `isActive()` checks `block.number - lastHeartbeat <= LIVENESS_WINDOW`.
The `registeredCount()` and `registeredAt()` pair enables enumeration of
all registered agents -- used by `ConsortiumValidator` to select committee
members and by the mirage-rs RPC to serve agent listings.

**Storage note**: The `_registered` array grows unboundedly. At scale, gas
costs for `ConsortiumValidator.assembleCommittee()` (which iterates all
workers via `WorkerRegistry.registeredCount()`) could become problematic.

---

### 4.4 IdentityRegistry -- ERC-8004 Soulbound Passports

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/IdentityRegistry.sol`

The richest contract in the suite. Implements the ERC-8004 agent identity
standard [1] with a soulbound ERC-721 surface (compliant with ERC-5192 [2]).
Each agent gets a non-transferable "Korai Passport" (`KPASS`) with:

- **64-bit capability bitmask**: 10 defined capabilities (inference, data
  transform, fine-tune, RAG, multi-agent, trading, security, analytics,
  knowledge, strategy) checked via `hasCapability(passportId, capBit)`
- **4-tier hierarchy**: Protocol (0), Sovereign (1), Worker (2), Edge (3)
- **System prompt hash**: `bytes32` commitment with timelock defense
- **TEE attestation**: `bytes32` hash + `uint64` expiry
- **Agent Card URI**: MCP-compliant agent descriptor URL
- **Domain staking**: Stake DAEJI tokens into named domains with 7-day
  withdrawal cooldown

**Tier thresholds** (stake-based auto-promotion):

| Tier | Required Stake |
|------|---------------|
| Edge | 0 (default) |
| Worker | >= 5,000 DAEJI |
| Sovereign | >= 25,000 DAEJI |
| Protocol | Admin-only (no self-promotion) |

**Soulbound enforcement**: All transfer functions (`transferFrom`,
`safeTransferFrom`, `approve`, `setApprovalForAll`) revert with `Soulbound()`.
The `locked()` function always returns `true` per ERC-5192. Interface IDs
registered via `supportsInterface`:

```solidity
function supportsInterface(bytes4 interfaceId) external pure returns (bool) {
    return interfaceId == 0x01ffc9a7   // ERC-165
        || interfaceId == 0x80ac58cd   // ERC-721
        || interfaceId == 0x5b5e139f   // ERC-721Metadata
        || interfaceId == 0xb45a3c0e;  // ERC-5192 (Soulbound)
}
```

**Prompt hash timelock** (ventriloquist defense): Updating the system prompt
hash requires two calls separated by at least 24 hours (`PROMPT_UPDATE_DELAY = 1 days`).
The first call to `updatePromptHash()` or `updateSystemPromptHash()` schedules
the update; the second call (with the same hash, after the delay) finalizes it.
This prevents an attacker who gains temporary access from immediately
hijacking an agent's behavior.

**Registrar model**: Registration is permissioned -- only the admin,
authorized registrars (set via `setRegistrar()`), or the agent itself can
create a passport. The contract exposes two registration paths:

```solidity
// Path 1: Full ERC-8004 registration (TEE attestation, auto-tier)
function registerPassport(
    address owner_, uint64 capabilityBitmask,
    bytes32 systemPromptHash, bytes32 teeAttestation, uint64 teeExpiry
) external returns (uint256 passportId);

// Path 2: Simplified registration (explicit tier, agent card URI)
function register(
    address agent, uint64 capabilityList, uint8 tier,
    bytes32 systemPromptHash, string calldata agentCardUri
) external returns (uint256 passportId);
```

**Domain staking**: The `stakeIntoDomain()` / `withdrawFromDomain()` functions
let passport holders stake DAEJI tokens into named reputation domains (e.g.
"OracleResolution", "DataIntegrity"). Stakes trigger tier re-evaluation via
`_syncTier()`. Withdrawals enforce a 7-day cooldown (`WITHDRAW_COOLDOWN`).

---

### 4.5 WorkerRegistry -- Stake, Reputation, Tiers

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/WorkerRegistry.sol`

Implements the Korai-spec reputation model: EMA-based reputation with lazy
decay, stake bonds, and tier classification.

**Reputation formula** (exponential moving average):

```
R_new = alpha * O + (1 - alpha) * R_old
```

where `O = 1.0` for success, `O = 0.0` for failure, and `alpha = 0.2`
(encoded as `ALPHA_NUM = 200_000` in a `SCALE = 1_000_000` fixed-point
system). Starting reputation is 0.5 (`SCALE / 2 = 500_000`).

EMA is a standard technique for smoothing time-series data while giving
more weight to recent observations [3]. In this context it means a single
failure drops reputation by `alpha * R_old = 0.2 * R_old`, and recovery
requires multiple successive successes.

**Decay model**: Reputation halves toward 0.5 every 30 days of inactivity
(`DECAY_PERIOD = 30 days`). Decay is applied lazily -- `_applyDecay()` runs
on the next `updateReputation()` or `slash()` call. A public `decay()`
function allows external keepers to poke the decay calculation. The loop is
capped at 64 halvings to bound gas costs.

```solidity
function _applyDecay(Worker storage w) internal {
    uint64 nowTs = uint64(block.timestamp);
    if (nowTs <= w.lastUpdated) return;
    uint256 elapsed = nowTs - w.lastUpdated;
    uint256 halvings = elapsed / DECAY_PERIOD;
    if (halvings == 0) return;
    uint256 mid = SCALE / 2;
    uint256 r = w.reputation;
    if (halvings > 64) halvings = 64;
    for (uint256 i = 0; i < halvings; i++) {
        if (r > mid) r = mid + (r - mid) / 2;
        else r = mid - (mid - r) / 2;
    }
    w.reputation = r;
    w.lastUpdated = nowTs;
}
```

**Tier classification** (view-only, applied to the decay-adjusted reputation):

| Tier | Reputation Range |
|------|-----------------|
| Probation | < 350,000 (0.35) |
| Standard | 350,000 - 549,999 |
| Trusted | 550,000 - 799,999 |
| Elite | >= 800,000 (0.80) |

**Slashing**: The `slash()` function takes a basis-point amount relative to
the worker's current bond. Three reason codes are defined:

| Code | Reason | Slash Rate |
|------|--------|-----------|
| 1 | `SLASH_MISSED_DEADLINE` | 1% (100 bps) |
| 2 | `SLASH_QUALITY_REJECT` | 5% (500 bps) |
| 3 | `SLASH_ABANDONMENT` | 10% (1000 bps) |

**Authorization model**: Only addresses marked `authorized[addr] = true`
can call `updateReputation()` and `slash()`. The deployment script
authorizes both `BountyMarket` and `ConsortiumValidator`.

---

### 4.6 ReputationRegistry -- Domain-Specific Reputation

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/ReputationRegistry.sol`

A second-generation reputation contract that ties to `IdentityRegistry`
passport IDs (not raw addresses like `WorkerRegistry`). Tracks reputation
per domain with two feedback paths:

**Authorized-source feedback** (`submitFeedback` with `onlyFeedbackSource`):
An authorized contract submits a signed score [-1e18, +1e18] for a passport
in a named domain, with a job hash and reason. The score is normalized to
[0, 1e18] and fed through an adaptive EMA:

```solidity
function _adaptiveAlpha(uint64 jobCount) internal pure returns (uint256) {
    uint256 adaptive = (2 * SCALE) / (uint256(jobCount) + 1);
    return adaptive < MAX_ALPHA ? adaptive : MAX_ALPHA;
}
```

This means early feedback has high weight (when `jobCount` is small,
`alpha` is large), but as an agent accumulates history, the smoothing
factor converges to `MAX_ALPHA = 0.3` (3e17).

**Peer feedback** (second `submitFeedback` overload): An agent with a valid
passport can rate another agent, but only if the pair has been pre-authorized
for the specific domain via `authorizeFeedback()`. Scores are 0-1000 (mapped
to 18-decimal fixed-point).

**7 reputation domains** (enum):
- OracleResolution, RiskDetection, AnomalyFlagging, DataIntegrity
- CrossAppValidation, SealedExecution, KnowledgeVerification

**Decay**: Same halving-toward-midpoint model as `WorkerRegistry`, with
`DECAY_PERIOD = 30 days`. An external `applyDecayTick()` function iterates
all domains for a passport and applies decay.

**Slashing**: Records slash events with block number, violation type, amount,
and reason. The history is retrievable via `getSlashHistory()`.

---

### 4.7 BountyMarket -- Programmable Escrow

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/BountyMarket.sol`

ERC-8183-style 4-state programmable escrow for agent task coordination.

**Job lifecycle state machine**:

```
              postJob()         assign()         submit()         resolve()
[None] -----> [Funded] ------> [Assigned] -----> [Submitted] ----> [Terminal]
```

Note: the `Open` state exists in the enum but `postJob()` atomically
transitions `Open -> Funded` by requiring `bounty` to be pre-approved
and pulling tokens in the same call. There is no separate "open without
funding" path.

**Key functions**:

```solidity
function postJob(bytes32 specHash, uint256 bounty, uint64 deadline, uint8 minTier)
    external returns (uint256 id);

function assign(uint256 id, address worker) external;

function submit(uint256 id, bytes32 resultHash) external;

function resolve(uint256 id, bool accepted) external;
```

**Resolution effects**:

| Outcome | Token Flow | Reputation | Slash |
|---------|-----------|------------|-------|
| Accepted | Bounty -> worker | `updateReputation(worker, true)` | None |
| Rejected | Bounty -> poster (refund) | `updateReputation(worker, false)` | 500 bps (5%) via `SLASH_QUALITY_REJECT` |

The `resolver` (initially the deployer) can be reassigned. The deployment
script sets it to `ConsortiumValidator` so that committee consensus
drives resolution.

---

### 4.8 ConsortiumValidator -- 2-of-3 Validation Committee

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/ConsortiumValidator.sol`

Assembles a 3-agent validation committee from Trusted+ workers and
drives resolution through 2-of-3 majority voting.

**Committee assembly** (`assembleCommittee()`):

1. Enumerates all registered workers via `workerRegistry.registeredCount()`
   and `workerRegistry.registeredAt(i)`.
2. Filters to those passing `workerRegistry.canAccept(w, Tier.Trusted)`.
3. Requires at least 3 eligible candidates.
4. Uses `blockhash(block.number - 1)` as a deterministic seed.
5. Applies a Fisher-Yates-style selection over 32 rounds to draw 3 distinct
   committee members.

**Voting** (`vote()`): Each committee member calls `vote(jobId, approve)`
exactly once. When 2 approvals or 2 rejections are reached, the contract
automatically calls `market.resolve(jobId, accepted)`.

```solidity
if (c.approves >= 2 || c.rejects >= 2) {
    c.tallied = true;
    bool accepted = c.approves >= 2;
    market.resolve(jobId, accepted);
    emit Tallied(jobId, accepted);
}
```

**Security note**: The blockhash-based randomness is predictable by block
producers. For production deployments, a VRF oracle (Chainlink VRF, drand)
or commit-reveal scheme should replace the seed.

---

### 4.9 ValidationRegistry -- Work Proofs and Attestations

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/ValidationRegistry.sol`

ERC-8004 validation registry for work proofs and validator attestations.
Integrates with `IdentityRegistry` for passport-based access control.

**Two primary workflows**:

1. **Work proof submission**: An agent (or authorized submitter) submits a
   proof that work was done -- containing a job hash, deliverable Merkle
   root, array of gate results (pass/fail per quality gate), and a clearing
   certificate. Gate pass rates are tracked per passport for aggregate
   quality metrics.

```solidity
function submitWorkProof(
    uint256 passportId, bytes32 jobHash,
    bytes32 deliverableMerkleRoot,
    uint8[] calldata gateResults,
    bytes calldata clearingCert
) external;
```

2. **Validation requests and attestations**: Any passport holder can request
   validation of a work hash, specifying a validator type (ReputationBased,
   StakeSecuredReExecution, ZkMLProof, TeeOracle). Other passport holders
   then submit attestations (approve/reject with evidence hash). After 3
   attestations, the request is automatically resolved.

**Gate pass rate**: `getGatePassRate(passportId, domain)` returns the
aggregate pass rate as a fixed-point value (1e18 scale) across all submitted
work proofs for that passport.

---

### 4.10 InsightBoard -- On-Chain Knowledge with Pheromone Curation

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/InsightBoard.sol`

A thin wrapper over mirage-rs's `0xA01` InsightEntry precompile. Agents post
content-hashed insights with URIs; others `confirm()` them, which
increments a pheromone weight counter and credits the poster with token
rewards.

```solidity
struct Insight {
    address poster;
    bytes32 contentHash;
    string uri;
    uint64 postedAt;
    uint64 pheromone;      // count of confirmations
}

uint256 public constant REWARD_PER_CONFIRM = 1 ether;
```

**Anti-gaming**: Self-confirmation reverts with `SelfConfirm()`. Duplicate
confirmations revert with `AlreadyConfirmed()`. The `poster` address is
checked with `==` rather than a more complex identity check -- the contract
assumes one address per agent.

**Earnings**: Confirmations accrue earnings in `earningsOf[poster]`. The
`claim()` function transfers accumulated rewards. The contract must be
pre-funded with reward tokens; there is no mechanism to pause confirmations
when funds are low (see Security Considerations).

---

### 4.11 ISFROracle -- Interest Rate Oracle

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/ISFROracle.sol`

On-chain storage for ISFR (Intelligent Synthetic Funding Rate) epoch-keyed
submissions. Keepers with `KEEPER_ROLE` (from `RoleRegistry`) submit rate
data each epoch.

```solidity
struct Rate {
    uint256 epochId;
    uint256 compositeBps;
    uint256 lendingBps;
    uint256 structuredBps;
    uint256 fundingBps;
    uint256 stakingBps;
    uint256 confidenceBps;
    uint64 timestamp;
    address submitter;
}
```

The `submitRate()` function prevents duplicate epoch submissions and stores
both the epoch-specific rate and updates `currentRate`. The `getCurrentRate()`
view returns the latest composite rate in basis points.

**Access control note**: The `setBountyPool()` function has no access
modifier -- any address can set the bounty pool address. This is a known
centralization gap (see Security Considerations).

---

### 4.12 ISFRBountyPool -- Oracle Keeper Rewards

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/ISFRBountyPool.sol`

Reward pool for ISFR keeper submissions. The oracle (holders of `ORACLE_ROLE`
via `RoleRegistry`) calls `rewardKeeper()` after each valid rate submission,
paying out `rewardPerSubmission` from the pool's token balance.

```solidity
constructor(address roleRegistry_, address token_, uint256 rewardPerSubmission_) { ... }

function rewardKeeper(address keeper) external onlyOracle { ... }
function availableBalance() external view returns (uint256) { ... }
```

The pool must be pre-funded with tokens. If the balance drops below
`rewardPerSubmission`, `rewardKeeper()` reverts with `InsufficientBalance()`.

---

### 4.13 FeeDistributor -- Multi-Party Fee Splitting

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/src/FeeDistributor.sol`

Splits a funded payment across four participant categories using
basis-point allocations:

| Category | Allocation |
|----------|-----------|
| Validators | 40% (4,000 bps) |
| Data Providers | 30% (3,000 bps) |
| Performing Agent | 20% (2,000 bps) |
| Treasury | 10% (1,000 bps) |

```solidity
function distribute(
    uint256 jobId,
    uint256 amount,
    address winner,
    address[] calldata validators,
    address[] calldata dataProviders
) external { ... }
```

**Fallback behavior**: If no validators or data providers are passed, their
share is redirected to the treasury. Within a group, tokens are split evenly
with remainder distributed to the first recipients (modular dust handling --
first `remainder` recipients get +1 wei).

**Storage**: `cumulativeEarnings` tracks lifetime earnings per address.
Tokens are transferred immediately via `_credit()` -- there is no
pull-based withdrawal pattern, which means the contract is vulnerable to
denial-of-service if a recipient is a contract that reverts on receive.

---

## 5. Deployment Script and Contract Wiring

**File**: `/Users/will/dev/nunchi/roko/roko/contracts/script/Deploy.s.sol`

The reference deployment script deploys 6 core contracts and wires their
cross-references:

```solidity
contract Deploy is Script {
    function run() external returns (Deployed memory d) {
        vm.startBroadcast();

        MockERC20 daeji = new MockERC20("DAEJI", "DAEJI", 18);
        AgentRegistry agentReg = new AgentRegistry();
        WorkerRegistry workerReg = new WorkerRegistry(address(daeji));
        BountyMarket market = new BountyMarket(address(daeji), address(workerReg));
        ConsortiumValidator consortium = new ConsortiumValidator(
            address(workerReg), address(market)
        );
        InsightBoard board = new InsightBoard(address(daeji));

        // Post-deploy wiring
        workerReg.setAuthorized(address(market), true);
        workerReg.setAuthorized(address(consortium), true);
        market.setResolver(address(consortium));

        vm.stopBroadcast();
    }
}
```

**What the script does NOT deploy**: `IdentityRegistry`, `ReputationRegistry`,
`ValidationRegistry`, `ISFROracle`, `ISFRBountyPool`, `FeeDistributor`, and
`RoleRegistry` are not in the reference deploy script. The ERC-8004 trinity
(Identity, Reputation, Validation) is bootstrapped separately by mirage-rs
itself at well-known addresses (see section 7.2). The ISFR contracts and
FeeDistributor are deployed independently by their respective subsystems.

The deployment NatSpec notes: "Not used by `roko-demo` directly -- it uses
an alloy-based deployer to sidestep forge's mempool-watcher model against
mirage-rs."

---

## 6. Contract Dependency Graph

```
MockERC20 (DAEJI token)
   |
   +-- WorkerRegistry (stakeToken = DAEJI)
   |      |
   |      +-- BountyMarket (bountyToken = DAEJI, workerRegistry)
   |      |      |
   |      |      +-- ConsortiumValidator (workerRegistry, market)
   |      |
   |      +-- (authorized callers: BountyMarket, ConsortiumValidator)
   |
   +-- InsightBoard (rewardToken = DAEJI)
   |
   +-- FeeDistributor (rewardToken = DAEJI)
   |
   +-- IdentityRegistry (stakeToken = DAEJI)
   |      |
   |      +-- ReputationRegistry (identityRegistry)
   |      |
   |      +-- ValidationRegistry (identityRegistry)
   |
   +-- ISFRBountyPool (token = DAEJI)

RoleRegistry
   |
   +-- ISFROracle (roleRegistry)
   |
   +-- ISFRBountyPool (roleRegistry)

AgentRegistry (standalone, no dependencies)
```

The contracts form three clusters:

1. **Worker/Bounty cluster**: WorkerRegistry <-> BountyMarket <->
   ConsortiumValidator. This is the core task-execution loop. Information
   flows: BountyMarket calls `updateReputation()` and `slash()` on
   WorkerRegistry; ConsortiumValidator calls `resolve()` on BountyMarket;
   both are authorized callers on WorkerRegistry.

2. **Identity/Reputation cluster**: IdentityRegistry -> ReputationRegistry,
   ValidationRegistry. This is the ERC-8004 identity layer [1]. Both
   ReputationRegistry and ValidationRegistry use `identityRegistry.ownerOf()`
   for passport validation.

3. **ISFR oracle cluster**: RoleRegistry -> ISFROracle, ISFRBountyPool.
   A self-contained oracle subsystem for interest rate feeds.

---

## 7. The EVM Simulator: mirage-rs

**Source**: `/Users/will/dev/nunchi/roko/roko/apps/mirage-rs/`

### 7.1 What mirage-rs Does

mirage-rs is a standalone EVM fork simulator built on [revm](https://github.com/bluealloy/revm)
(Rust Ethereum Virtual Machine) [5]. It provides Anvil-style functionality --
fork mainnet state, simulate transactions, mine blocks -- but layers an
agent-coordination substrate on top through opt-in cargo features.

The architecture centers on `ForkState`, which wraps a `HybridDB` combining:
- **Upstream RPC**: Lazy-fetched mainnet state via HTTP/WebSocket
- **Read cache**: Time-bounded LRU cache for fetched account/storage data
- **Dirty store**: Local modifications from simulated transactions
- **Bytecode cache**: Multi-version copy-on-write storage for contract code

Key cargo features:

| Feature | Purpose |
|---------|---------|
| `sim-gas` | revm gas accounting path |
| `chain` | HDC index, InsightEntry knowledge layer, stigmergy pheromones |
| `roko` | Bridge to roko-core traits (Gate, Substrate) |
| `dashboard-api` | HTTP REST API for pheromone field, knowledge graph, agent topology |

The server exposes a JSON-RPC surface implementing standard `eth_*` methods
plus custom `mirage_*` and `chain_*` namespaces. With `chain` enabled, mirage
gains:
- `InsightEntry` knowledge types with a state machine (Pending -> Active -> Challenged -> Stale -> Pruned)
- HDC (Hyperdimensional Computing) vector projection and similarity
- Brute-force top-K Hamming similarity index with auto-upgrade to binary HNSW at 100K+ entries
- `KnowledgeStore` unifying post/confirm/challenge/decay/search
- Pheromone field: decaying THREAT/OPPORTUNITY/WISDOM signals with HDC retrieval

### 7.2 ERC-8004 Bootstrap and ForkState

**Source**: `/Users/will/dev/nunchi/roko/roko/apps/mirage-rs/src/main.rs`

At startup, mirage-rs bootstraps three ERC-8004 contracts at well-known
addresses using `EvmExecutor::transact()`:

```rust
const ERC8004_IDENTITY_REGISTRY: Address = address!("0x8004A818BFB912233c491871b3d84c89A494BD9e");
const ERC8004_REPUTATION_REGISTRY: Address = address!("0x8004A818BFB912233c491871b3d84c89A494BD9f");
const ERC8004_VALIDATION_REGISTRY: Address = address!("0x8004A818BFB912233c491871b3d84c89A494BDA0");
const ERC8004_BOOTSTRAP_ADMIN: Address = address!("0x8004000000000000000000000000000000000001");
const ERC8004_BOOTSTRAP_DEPLOYER: Address = address!("0x8004000000000000000000000000000000000002");
```

The `bootstrap_erc8004_contracts()` function:
1. Checks if each canonical address already has deployed code (from a restored snapshot)
2. If not, loads pre-compiled init bytecode from `static/erc8004/*.init.hex`
3. Appends ABI-encoded constructor arguments (admin address, dependencies)
4. Executes via `EvmExecutor::transact()` against a cloned fork state
5. Copies the resulting runtime bytecode and storage to the canonical address

This gives every mirage-rs instance the same three ERC-8004 contracts at
deterministic addresses, regardless of the upstream chain state. The
`bytecode_hash = "none"` Foundry setting ensures the init bytecodes are
reproducible across builds.

The contracts are registered in the `ChainContext.contract_registry` so
they appear in the `/api/deployment` REST endpoint.

### 7.3 HDC Precompile at 0xA0C

**Source**: `/Users/will/dev/nunchi/roko/roko/apps/mirage-rs/src/precompiles/hdc.rs`

mirage-rs injects a custom EVM precompile at address `0x0000...0A0C` that
exposes HDC operations to Solidity contracts:

```
Address: 0x0000000000000000000000000000000000000A0C
Gas cost: 5,000 (flat, Phase 2)
Vector size: 1,280 bytes (10,240 bits)
```

**8 methods** matching the `IHDCPrecompile.sol` interface:

| Selector | Method | Purpose |
|----------|--------|---------|
| `0xcbd13d9b` | `projectBytes(bytes)` | Project arbitrary bytes to HDC vector |
| `0x1f7f97cb` | `projectTokens(string)` | Project text to HDC vector via tokenization |
| `0x675f5f52` | `bind(bytes,bytes)` | Bind two HDC vectors (element-wise XOR) |
| `0x1427c292` | `bundle(bytes[])` | Bundle multiple vectors (majority vote) |
| `0xc688e787` | `similarity(bytes,bytes)` | Hamming similarity between vectors |
| `0x4ec2c730` | `search(bytes,uint256,uint256)` | Top-K search against the index |
| `0x8e85a454` | `insert(bytes16,bytes,uint32)` | Insert vector with ID and weight |
| `0x4cb7a7a2` | `remove(bytes16)` | Remove vector by ID |

The precompile wraps `revm::EthPrecompiles` and routes calls to address
`0xA0C` to the HDC handler while delegating all other addresses to the
standard Ethereum precompiles.

Similarity values are returned as `uint32` scaled by 1e6 (i.e.
`similarity = 1.0` returns `1_000_000`).

### 7.4 Knowledge Layer Integration

**Source**: `/Users/will/dev/nunchi/roko/roko/apps/mirage-rs/src/chain/knowledge.rs`

The `KnowledgeStore` is an in-memory knowledge substrate exposed through
the `chain_*` JSON-RPC namespace:

| Method | Args | Returns |
|--------|------|---------|
| `chain_postInsight` | `{author, kind, content, enabledBy?, stakeWei?}` | `{outcome, id, ...}` |
| `chain_searchInsights` | `{query, k, kind?}` | `[{id, similarity, weight}]` |
| `chain_confirmInsight` | `{id, confirmer}` | `{ok: true}` |
| `chain_challengeInsight` | `{id, challenger}` | `{ok: true}` |
| `chain_applyDecay` | `{nowSecs?}` | `{prunedCount}` |
| `chain_depositPheromone` | `{kind, content, intensity?, halfLifeSeconds?}` | `{id}` |
| `chain_queryPheromones` | `{query, k}` | `[{id, kind, similarity, ...}]` |
| `chain_stats` | `{}` | `{insights, pheromones}` |

**Operations**:

- `post()`: Content-addressed, deduplicated. If a new entry is >95% HDC-similar
  to an existing entry of the same kind, it is treated as a duplicate with
  attenuated reward.
- `confirm(id, confirmer)`: Boosts the entry's weight and updates index weights.
- `challenge(id, challenger)`: Transitions entry to `Challenged` state.
- `apply_decay(now_secs)`: Entries whose decayed weight drops below 1% of
  initial are pruned from the search index (but retained in storage for audit).
- `search(query, k)`: Returns top-K entries by `similarity * weight`. Uses
  HNSW when entry count exceeds threshold (default 100K), otherwise brute-force HDC.

**Knowledge kinds**: `Insight`, `Heuristic`, `Warning`, `AntiKnowledge`,
`CausalLink`, `StrategyFragment`.

**Knowledge states**: `Pending -> Active -> Challenged -> Stale -> Pruned`

**Persistence**: mirage-rs snapshots the entire fork state (including chain
context with knowledge store and pheromone field) to disk periodically
(default every 30 seconds). On restart, `persist::load_snapshot()` restores
the full state including all ERC-8004 contract storage, HDC indexes, and
knowledge entries.

---

## 8. The Chain Watcher: roko-chain-watcher

**Source**: `/Users/will/dev/nunchi/roko/roko/apps/roko-chain-watcher/`

### 8.1 Architecture

The chain watcher is a long-running agent process that polls mirage-rs,
applies pattern-matching rules, and posts reactions back to the chain:

```
mirage-rs (RPC) --poll--> rpc_client --> reactions::decide --> rpc_client --post--> mirage-rs
```

The architecture has two independent loops, each running as a separate
tokio task:

1. **Reaction loop** (`watcher.rs`): Polls `chain_queryPheromones` and
   `chain_searchInsights`, runs reaction rules via `reactions::decide()`,
   executes surviving reactions via RPC. Rate-limited via a per-minute
   sliding window (`max_reactions_per_min`, enforced by `acquire_reaction_slot()`).
   Supports `--dry-run` mode for testing without posting.

2. **Block observer** (`block_observer.rs`): Polls `eth_getBlockByNumber`
   against a real Ethereum RPC (or the mirage endpoint), analyzes gas usage,
   base-fee trends, and transaction patterns, and posts insights/pheromones
   grounded in actual chain data. Configurable via `--disable-block-observer`,
   `--block-poll-interval-ms`, `--block-backfill`, and `--fetch-full-txs`.

The watcher exits cleanly on Ctrl+C or when `--max-events` observations
have been reached.

### 8.2 Reaction Rules

**Source**: `/Users/will/dev/nunchi/roko/roko/apps/roko-chain-watcher/src/reactions.rs`

The `decide()` function takes pheromones, insights, and a watcher identity,
and returns a vector of `Reaction` structs. Four reaction kinds:

```rust
pub enum ReactionKind {
    PostInsight,
    DepositPheromone,
    ConfirmInsight,
    ChallengeInsight,
}
```

Nine pattern-matching rules:

| Rule | Trigger | Action |
|------|---------|--------|
| 1 | Threat pheromone > 0.7 intensity, no existing warning | Post warning insight |
| 2 | Opportunity pheromone > 0.6 intensity | Post strategy_fragment insight |
| 3 | Wisdom pheromone with matching insight (sim >= 0.55) | Confirm the best matching insight |
| 3b | Unconfirmed insights with sim >= 0.50 | Confirm up to 3 per poll |
| 4 | Insight content contains WRONG/BUG/INCORRECT with confirmations | Challenge the insight |
| 4b | Highly confirmed insight (5+ confirms, 0 challenges) | Stress-test challenge (probabilistic) |
| 5 | Any observations (>= 3 total) | Deposit wisdom summary pheromone |
| 6 | 3+ insights share a topic (dex/lending/mev/etc.) | Synthesize heuristic + opportunity |
| 7 | Pheromone and insight share a topic | Post causal_link insight |
| 8 | 3+ low-intensity fading threats | Deposit recovery opportunity |
| 9 | 5+ consecutive insights of same kind | Challenge weakest (diversity injection) |

**Topic classification**: The watcher classifies content into 8 topics
(dex, lending, mev, gas, whale, stablecoin, bridge, nft, general) using
keyword matching.

### 8.3 Block Observer

**Source**: `/Users/will/dev/nunchi/roko/roko/apps/roko-chain-watcher/src/block_observer.rs`

The block observer analyzes real Ethereum blocks and generates insights
grounded in actual chain data. Analysis rules:

**Single-block rules**:
- Block saturation > 95%: deposit threat pheromone
- Base fee < 2 gwei: post heuristic insight + opportunity pheromone (low gas window)
- Base fee > 100 gwei: deposit threat pheromone
- Tx count > 300 or == 0: post insight or warning

**Transaction-level rules** (when `--fetch-full-txs` is enabled):
- DEX router activity (Uniswap, Sushi, Curve interactions)
- Aave/lending cluster activity
- Liquid staking, NFT marketplace, bridge flows
- Whale transfers (>= 50 ETH)
- Contract creations, MEV signals (priority tips >= 5 gwei)
- Total ETH flow (>= 500 ETH per block)

**Multi-block trend rules**:
- Base fee rising/falling > 25% over 3 blocks
- Sustained saturation > 90% over 5 blocks
- Block-time irregularity (max gap > 2x average)

The observer uses known-address databases
(`/Users/will/dev/nunchi/roko/roko/apps/roko-chain-watcher/src/known_addresses.rs`)
to identify contract interactions and decode method selectors for richer
insight generation.

---

## 9. Rust-Side Contract Integration: roko-chain

**Source**: `/Users/will/dev/nunchi/roko/roko/crates/roko-chain/`

The Solidity contracts are the on-chain layer, but roko's Rust codebase
mirrors and extends their logic through the `roko-chain` crate. This crate
provides:

**`ChainClient` trait** (`src/client.rs`): A read-only interface to any
EVM-compatible chain -- blocks, receipts, logs, storage, and `eth_call`
simulation. Implementations include an Alloy-backed JSON-RPC client and
an in-memory mock.

```rust
#[async_trait]
pub trait ChainClient: Send + Sync {
    async fn block_number(&self) -> ChainResult<BlockNumber>;
    async fn get_block_header(&self, number: BlockNumber) -> ChainResult<ChainHeader>;
    async fn get_receipt(&self, tx: &TxHash) -> ChainResult<Option<Receipt>>;
    async fn get_logs(...) -> ChainResult<Vec<LogEntry>>;
    async fn eth_call(...) -> ChainResult<CallResult>;
    // ...
}
```

**`AgentRegistry` (Rust-side)** (`src/agent_registry.rs`): An in-memory
implementation of the soulbound passport model with:
- Tier progression rules (Edge -> Worker -> Sovereign -> Protocol) based on
  stake, job count, and reputation thresholds
- 24-hour timelock for system prompt hash updates (ventriloquist defense)
- Rate limiting: >3 prompt changes in 30 days triggers -0.05 reputation penalty
- 10 capability bits matching the Solidity contract's bitmask

**`BlockWatcher`** (`src/block_watcher.rs`): A poll-based watcher that
streams block, transaction, and decoded contract events via a callback
(`PublishFn`). Tracks recent state in ring buffers (`ChainState`) for
REST API consumption.

**Event decoder** (`src/block_watcher.rs`): Decodes known contract events
by topic0 signature -- `RateSubmitted`, `KeeperRewarded`, `RoleGranted`,
`RoleRevoked` -- with fallback to raw topic+data for unknown events.

---

## 10. Test Suite

Every contract has a corresponding Foundry test file in
`/Users/will/dev/nunchi/roko/roko/contracts/test/`:

| Test File | Coverage |
|-----------|----------|
| `AgentRegistry.t.sol` | Register, heartbeat, liveness, duplicate rejection |
| `BountyMarket.t.sol` | Full lifecycle (post/assign/submit/resolve), accept/reject paths, state transitions |
| `ConsortiumValidator.t.sol` | Committee assembly, voting, majority tally, unauthorized vote rejection |
| `FeeDistributor.t.sol` | Distribution splits, empty validator/provider fallback, cumulative earnings |
| `IdentityRegistry.t.sol` | Passport minting, soulbound enforcement, prompt timelock, domain staking, tier sync |
| `InsightBoard.t.sol` | Post, confirm, self-confirm rejection, duplicate confirm rejection, earnings claim |
| `MockERC20.t.sol` | Mint, transfer, decimals |
| `ReputationRegistry.t.sol` | Feedback submission, adaptive alpha, decay, slash history, peer feedback authorization |
| `ValidationRegistry.t.sol` | Work proof submission, gate pass rate, validation requests, attestations |
| `WorkerRegistry.t.sol` | Registration, bonding, reputation EMA, decay, tier calculation, slashing |

Note: `RoleRegistry` has no dedicated test file. It is tested indirectly
through the `ISFROracle` and `ISFRBountyPool` tests.

Tests use forge's `Test` base class with `vm.prank()`, `vm.warp()`,
`vm.roll()` for caller impersonation and time/block manipulation.

Fuzz testing is configured with 256 runs per property test (`[fuzz] runs = 256`
in `foundry.toml`).

---

## 11. Porting to NEAR

### 11.1 Account Model Differences

The EVM contracts assume address-based identity (20-byte addresses, `msg.sender`).
NEAR uses human-readable account names (`alice.near`, `agent.roko.near`)
with a sub-account hierarchy [6].

Key differences affecting the port:

| Aspect | EVM (Current) | NEAR |
|--------|--------------|------|
| Identity | 20-byte address | Named accounts (`agent.roko.near`) |
| Storage cost | Gas per SSTORE (20K new, 5K update) | Storage staking (1 NEAR per 100KB) |
| Cross-contract calls | Synchronous | Asynchronous (promises) |
| Token standard | ERC-20 | NEP-141 (ft_transfer) |
| NFT standard | ERC-721 | NEP-171 (nft_transfer) |
| Access control | `msg.sender` | `env::predecessor_account_id()` |
| Reentrancy | Possible (same-tx callbacks) | Not possible (async receipts) |
| Gas model | Per-opcode, paid by sender | Per-operation + storage staking, ~300 TGas limit |
| Contract size | ~24KB limit | ~4MB limit (WASM binary) |
| Upgradability | Proxy patterns (EIP-1967) | Native (`deploy` to same account) |
| Randomness | `blockhash` (predictable) | VRF via `near_sdk::env::random_seed()` |

**Sub-account advantage**: NEAR's sub-account system maps naturally to the
agent hierarchy. A deployment could use:
- `ironclaw.near` -- protocol treasury
- `registry.ironclaw.near` -- identity registry
- `bounty.ironclaw.near` -- bounty market
- `oracle.ironclaw.near` -- ISFR oracle
- `alice.agents.ironclaw.near` -- an agent's identity

### 11.2 Storage Staking Economics

NEAR requires storage staking at 1e19 yoctoNEAR per byte, which equals
**1 NEAR per 100KB** of on-chain storage [7]. This is fundamentally
different from EVM's per-write gas model:

- EVM: Pay gas once at write time; storage persists indefinitely for free.
- NEAR: Lock NEAR proportional to stored bytes; unlocked when storage is freed.

**Per-contract storage estimates** (at 1 NEAR = 100KB):

| Contract | Per-Entry Size (est.) | Storage per Entry | Notes |
|----------|----------------------|-------------------|-------|
| AgentRegistry | ~200 bytes (capabilities string + 32B hash + 16B) | ~0.002 NEAR | Refundable on deregistration |
| WorkerRegistry | ~120 bytes (u128 + u128 + u64*3 + bool) | ~0.0012 NEAR | Bond is separate from storage |
| IdentityRegistry | ~400 bytes (PassportData + domain stakes) | ~0.004 NEAR | Plus ~0.001 NEAR per domain stake |
| ReputationRegistry | ~100 bytes per domain | ~0.001 NEAR/domain | 7 domains = ~0.007 NEAR |
| BountyMarket | ~250 bytes per job | ~0.0025 NEAR | Refundable when terminal |
| InsightBoard | ~200 bytes per insight | ~0.002 NEAR | Plus URI string length |

These costs are quite small -- a worker registration costs roughly 0.002 NEAR
in storage staking. However, the storage staking model provides natural
spam protection: creating 1,000 fake agents would lock ~2 NEAR, and the
storage deposits are visible on-chain as a commitment signal.

### 11.3 Gas Budget Constraints

NEAR transactions have a maximum gas limit of ~300 TGas. Each cross-contract
call costs a minimum of 5 TGas for the function call itself plus gas for
execution. This constrains certain patterns from the Solidity contracts:

**`ConsortiumValidator.assembleCommittee()`** iterates all registered
workers to find Trusted+ candidates. With 10,000 workers, this would exceed
NEAR's gas limit. Solutions:
- **Pagination**: Maintain a pre-filtered set of Trusted+ workers, updated
  on tier changes.
- **Off-chain selection with on-chain verification**: Compute the committee
  off-chain, submit the result with a proof.
- **Lazy enumeration**: Use `TreeMap` or sorted collections to iterate only
  qualifying workers.

**`ReputationRegistry.applyDecayTick()`** iterates all domain keys for a
passport. With many domains, this should be paginated.

### 11.4 The Aurora Shortcut

[Aurora](https://aurora.dev/) is an EVM running as a NEAR smart contract [8].
It provides an alternative porting path: deploy the Solidity contracts
unchanged on Aurora, getting NEAR's finality and low fees without rewriting
to Rust.

**Advantages**:
- Zero code changes to Solidity contracts
- Same tooling (Foundry, Hardhat, ethers.js)
- Rainbow Bridge enables asset transfers between NEAR and Aurora
- Sub-second finality inherited from NEAR

**Disadvantages**:
- EVM execution overhead (Aurora charges ~1.5x the equivalent NEAR gas)
- Cannot use NEAR-native features (named accounts, storage staking, VRF)
- Additional bridge complexity for NEAR-native token interactions
- Aurora-specific failure modes (gas estimation differences)

**Recommendation**: Use Aurora for rapid prototyping and initial deployment,
then port high-value contracts (IdentityRegistry, WorkerRegistry) to native
NEAR for better economics and tighter IronClaw integration.

### 11.5 Native NEAR Contract Sketches

#### AgentRegistry on NEAR

```rust
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{near, AccountId, env, BlockHeight};

#[near(contract_state)]
pub struct AgentRegistry {
    agents: UnorderedMap<AccountId, Agent>,
    liveness_window: BlockHeight,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Agent {
    capabilities: String,
    passport_hash: [u8; 32],
    registered_at: BlockHeight,
    last_heartbeat: BlockHeight,
}

#[near]
impl AgentRegistry {
    #[init]
    pub fn new() -> Self {
        Self {
            agents: UnorderedMap::new(b"a"),
            liveness_window: 200,
        }
    }

    #[payable]
    pub fn register(&mut self, capabilities: String, passport_hash: [u8; 32]) {
        let account = env::predecessor_account_id();
        assert!(self.agents.get(&account).is_none(), "Already registered");
        // Storage deposit covers ~200 bytes
        assert!(
            env::attached_deposit().as_yoctonear() >= 2_000_000_000_000_000_000_000,
            "Attach at least 0.002 NEAR for storage"
        );
        self.agents.insert(&account, &Agent {
            capabilities,
            passport_hash,
            registered_at: env::block_height(),
            last_heartbeat: env::block_height(),
        });
    }

    pub fn heartbeat(&mut self) {
        let account = env::predecessor_account_id();
        let mut agent = self.agents.get(&account).expect("Not registered");
        agent.last_heartbeat = env::block_height();
        self.agents.insert(&account, &agent);
    }

    pub fn is_active(&self, account: AccountId) -> bool {
        match self.agents.get(&account) {
            None => false,
            Some(a) => env::block_height() - a.last_heartbeat <= self.liveness_window,
        }
    }
}
```

**Key differences from EVM version**:
- `#[payable]` on `register()` for storage deposit
- `AccountId` instead of `address` -- human-readable, variable length
- `UnorderedMap` instead of Solidity mapping -- NEAR's collection types handle
  storage serialization automatically via Borsh
- `env::block_height()` replaces `block.number`

#### WorkerRegistry on NEAR

```rust
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{near, AccountId, env, Promise, NearToken};

const SCALE: u128 = 1_000_000;
const ALPHA_NUM: u128 = 200_000;
const MIN_BOND_YOCTO: u128 = 1_000_000_000_000_000_000_000_000_000; // 1000 * 10^24 yoctoNEAR
const DECAY_PERIOD_NS: u64 = 30 * 24 * 60 * 60 * 1_000_000_000; // 30 days in nanoseconds

#[near(contract_state)]
pub struct WorkerRegistry {
    stake_token: AccountId,     // NEP-141 token contract
    owner: AccountId,
    authorized: UnorderedMap<AccountId, bool>,
    workers: UnorderedMap<AccountId, Worker>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Worker {
    bond: u128,
    reputation: u128,           // scaled by SCALE
    jobs_completed: u64,
    jobs_slashed: u64,
    last_updated: u64,          // nanoseconds (env::block_timestamp())
}

#[derive(PartialEq, PartialOrd, BorshDeserialize, BorshSerialize)]
pub enum Tier {
    Unregistered = 0,
    Probation = 1,
    Standard = 2,
    Trusted = 3,
    Elite = 4,
}

#[near]
impl WorkerRegistry {
    pub fn update_reputation(&mut self, worker: AccountId, outcome: bool) {
        self.assert_authorized();
        let mut w = self.workers.get(&worker).expect("Not registered");

        // Apply lazy decay
        let now = env::block_timestamp();
        let elapsed = now.saturating_sub(w.last_updated);
        let halvings = (elapsed / DECAY_PERIOD_NS).min(64);
        let mid = SCALE / 2;
        let mut r = w.reputation;
        for _ in 0..halvings {
            if r > mid { r = mid + (r - mid) / 2; }
            else { r = mid - (mid - r) / 2; }
        }

        // EMA update: R_new = alpha * O + (1 - alpha) * R_old
        let observation = if outcome { SCALE } else { 0 };
        r = (ALPHA_NUM * observation + (SCALE - ALPHA_NUM) * r) / SCALE;

        w.reputation = r;
        w.last_updated = now;
        if outcome { w.jobs_completed += 1; }
        else { w.jobs_slashed += 1; }
        self.workers.insert(&worker, &w);
    }

    pub fn tier_of(&self, worker: &AccountId) -> Tier {
        match self.workers.get(worker) {
            None => Tier::Unregistered,
            Some(w) => {
                if w.reputation < 350_000 { Tier::Probation }
                else if w.reputation < 550_000 { Tier::Standard }
                else if w.reputation < 800_000 { Tier::Trusted }
                else { Tier::Elite }
            }
        }
    }

    fn assert_authorized(&self) {
        let caller = env::predecessor_account_id();
        assert!(
            self.authorized.get(&caller).unwrap_or(false) || caller == self.owner,
            "Not authorized"
        );
    }
}
```

**NEAR-specific notes**:
- Timestamps are in nanoseconds (`env::block_timestamp()`) vs Solidity's seconds
- `DECAY_PERIOD_NS` is `30 days * 1e9` nanoseconds
- Token operations (bond/unbond) would use NEP-141 `ft_transfer_call()` with
  a receiver callback pattern instead of ERC-20 `transferFrom()`

#### BountyMarket on NEAR

The BountyMarket port requires handling NEAR's asynchronous cross-contract
calls. The EVM version calls `workerRegistry.updateReputation()` synchronously
inside `resolve()`. On NEAR, this becomes a chain of promises:

```rust
#[near]
impl BountyMarket {
    pub fn resolve(&mut self, job_id: u64, accepted: bool) -> Promise {
        assert_eq!(
            env::predecessor_account_id(), self.resolver,
            "Not authorized"
        );
        let mut job = self.jobs.get(&job_id).expect("Unknown job");
        assert!(matches!(job.state, JobState::Submitted));
        job.state = JobState::Terminal;
        job.accepted = accepted;

        let worker = job.worker.clone().unwrap();
        let recipient = if accepted {
            worker.clone()
        } else {
            job.poster.clone()
        };

        self.jobs.insert(&job_id, &job);

        // NEP-141 ft_transfer is async -- chain promises
        let transfer = Promise::new(self.token.clone())
            .function_call(
                "ft_transfer".to_string(),
                serde_json::json!({
                    "receiver_id": recipient,
                    "amount": job.bounty.to_string(),
                }).to_string().into_bytes(),
                NearToken::from_yoctonear(1),  // 1 yoctoNEAR for security
                near_sdk::Gas::from_tgas(10),
            );

        // Then update reputation
        let rep_update = Promise::new(self.worker_registry.clone())
            .function_call(
                "update_reputation".to_string(),
                serde_json::json!({
                    "worker": worker,
                    "outcome": accepted,
                }).to_string().into_bytes(),
                NearToken::from_yoctonear(0),
                near_sdk::Gas::from_tgas(10),
            );

        transfer.then(rep_update)
    }
}
```

### 11.6 Key Porting Challenges

1. **Asynchronous cross-contract calls**: EVM's synchronous `workerRegistry.slash()`
   becomes a `Promise::new().function_call()`. Error handling changes
   fundamentally: on EVM, a revert in `slash()` reverts the entire `resolve()`.
   On NEAR, a failed promise in `update_reputation()` does not revert the
   token transfer. The port must handle partial success using the
   `Promise::then()` callback pattern with explicit rollback logic.

2. **Storage deposits**: Callers must attach sufficient NEAR to cover storage
   for new jobs, workers, and insights. The `#[payable]` annotation and
   `env::attached_deposit()` replace EVM's `msg.value`. Contracts should
   refund excess deposits.

3. **Token transfers**: `ERC20.transferFrom()` becomes NEP-141 `ft_transfer_call()`
   with a receiver callback, or a two-step approve+transfer pattern using
   `ft_transfer`. Cross-contract token movements require promise chaining.

4. **No reentrancy risk**: NEAR's async model eliminates reentrancy attacks,
   so the EVM checks-effects-interactions pattern is less critical. However,
   state-consistency across promise chains requires careful design -- a
   common pattern is to update local state optimistically before the promise,
   then handle rollback in a `_callback` method.

5. **Named accounts as identity**: The `IdentityRegistry` soulbound NFT
   pattern could be simplified on NEAR since accounts already have stable
   named identities. A NEAR port might attach passport data directly to the
   account's contract state rather than minting a separate NFT, though
   NEP-171 compliance is still valuable for ecosystem tooling.

6. **Gas prepay model**: NEAR's gas is prepaid by the caller. Long loops
   (like `ConsortiumValidator.assembleCommittee` iterating all workers) need
   pagination or off-chain computation with on-chain verification, since NEAR
   has a ~300 TGas limit per transaction.

7. **Randomness**: `ConsortiumValidator` uses `blockhash(block.number - 1)`,
   which is predictable. On NEAR, `env::random_seed()` provides
   unpredictable randomness derived from the block's VRF output, making
   committee selection resistant to validator manipulation.

---

## 12. IronClaw Integration Plan

IronClaw is a NEAR ecosystem project -- a secure personal AI assistant with
multi-channel access, self-expanding tools, and proactive background execution.
Integrating on-chain agent infrastructure would give IronClaw agents
verifiable identities, economic coordination, and trustless reputation.

### 12.1 What IronClaw Needs from On-Chain Infrastructure

IronClaw's current architecture provides:
- Agent identity via local configuration and workspace memory
- Tool dispatch through `ToolDispatcher` (all actions go through tools)
- Multi-channel access (CLI, web, Telegram, HTTP webhooks)
- Background execution via heartbeat system
- Skill system with trust-based attenuation

On-chain infrastructure would add:
- **Verifiable agent identity**: Other agents and services can verify an
  IronClaw agent's capabilities and track record without trusting a central
  server.
- **Cross-instance coordination**: Multiple IronClaw instances (or IronClaw
  + other agent frameworks) can delegate tasks, share knowledge, and settle
  payments via the bounty market.
- **Reputation portability**: An agent's reputation is not locked to a single
  IronClaw deployment -- it lives on-chain and is readable by any party.
- **Economic primitives**: Agents can earn and spend tokens for work,
  knowledge contributions, and validation services.

### 12.2 Phase 1: Agent Identity on NEAR

**Goal**: Each IronClaw agent gets a NEAR-based passport.

**Implementation**:

1. **`chain` tool module** (`src/tools/builtin/chain.rs`): A new built-in tool
   that wraps `near-api-rs` or `near-jsonrpc-client` for NEAR RPC interactions.
   Following the "Everything Goes Through Tools" principle, all chain
   interactions flow through `ToolDispatcher::dispatch()`.

2. **Agent passport registration**: On first startup (or via the onboarding
   wizard at `src/setup/`), the agent registers a passport on the
   `registry.ironclaw.near` contract:
   - Capabilities derived from installed tools and skills
   - System prompt hash from the agent's SOUL.md / IDENTITY.md
   - Agent Card URI pointing to the agent's MCP endpoint

3. **Heartbeat integration**: IronClaw's existing heartbeat system
   (`src/workspace/`) calls `heartbeat()` on the on-chain registry at each
   heartbeat interval, maintaining liveness proof.

4. **Key management**: The agent's NEAR signing key is stored in IronClaw's
   secrets system (`src/secrets/`), encrypted with AES-256-GCM and the OS
   keychain master key. The key never leaves the local machine.

**NEAR contract** (deployed at `registry.ironclaw.near`):

```rust
#[near(contract_state)]
pub struct IronclawRegistry {
    passports: UnorderedMap<AccountId, AgentPassport>,
    admin: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AgentPassport {
    capabilities: u64,          // 10-bit bitmask matching roko spec
    system_prompt_hash: [u8; 32],
    agent_card_uri: String,     // MCP agent card URL
    registered_at: BlockHeight,
    last_heartbeat: BlockHeight,
    tier: u8,                   // 0=Protocol, 1=Sovereign, 2=Worker, 3=Edge
    version: u32,               // IronClaw version number
}
```

### 12.3 Phase 2: Reputation and Work Coordination

**Goal**: IronClaw agents can post and claim bounties, build on-chain reputation.

**Implementation**:

1. **`bounty_post` / `bounty_claim` / `bounty_submit` tools**: Wrap the
   BountyMarket contract interactions. The agent's `ToolDispatcher` handles
   auth, safety checks, and audit trail.

2. **Reputation-gated tool access**: IronClaw's skill attenuation system
   (`src/skills/attenuate_tools`) could gate high-privilege tools behind
   on-chain reputation thresholds. An agent with Elite reputation gets
   access to more tools than a Probation agent.

3. **Worker registration**: IronClaw agents register as workers by staking
   tokens. The minimum bond provides Sybil resistance.

### 12.4 Phase 3: Knowledge Layer

**Goal**: IronClaw's workspace memory is backed by on-chain knowledge.

**Implementation**:

1. **Memory-to-insight bridge**: IronClaw's `memory_write` tool could
   optionally post high-value insights to the on-chain knowledge layer,
   earning confirmation rewards.

2. **Knowledge search integration**: `memory_search` could query both local
   workspace memory and the on-chain InsightBoard/KnowledgeStore for a
   hybrid local+global search.

3. **Pheromone awareness**: The heartbeat system could poll the pheromone
   field and inject relevant signals (threats, opportunities) into the
   agent's context.

### 12.5 Integration Architecture

```
IronClaw Agent
   |
   +-- src/tools/builtin/chain.rs (new)
   |      |
   |      +-- near-jsonrpc-client
   |      |      |
   |      |      +-- registry.ironclaw.near (AgentPassport)
   |      |      +-- worker.ironclaw.near (WorkerRegistry)
   |      |      +-- bounty.ironclaw.near (BountyMarket)
   |      |      +-- knowledge.ironclaw.near (InsightBoard)
   |      |
   |      +-- ToolDispatcher::dispatch() (audit trail, safety pipeline)
   |
   +-- src/secrets/ (NEAR signing key storage)
   |
   +-- src/workspace/ (heartbeat -> on-chain heartbeat)
   |
   +-- crates/ironclaw_llm/ (LLM decides when to use chain tools)
```

**Configuration** (`.env` additions):

```bash
NEAR_NETWORK=mainnet           # or testnet
NEAR_ACCOUNT_ID=myagent.ironclaw.near
NEAR_KEY_PATH=~/.ironclaw/near-key.json
IRONCLAW_REGISTRY=registry.ironclaw.near
IRONCLAW_BOUNTY_MARKET=bounty.ironclaw.near
```

---

## 13. Security Considerations

**Across all contracts**:

1. **Reentrancy**: The contracts use a transfer-last pattern but do not
   include explicit reentrancy guards (no OpenZeppelin `ReentrancyGuard`).
   The ERC20 `transfer()` and `transferFrom()` calls are the last operations
   in most functions, which mitigates classic reentrancy. However, a
   malicious ERC20 token with a `transfer` hook could re-enter. On NEAR,
   this is not a concern due to the async execution model.

2. **Blockhash predictability**: `ConsortiumValidator.assembleCommittee()`
   uses `blockhash(block.number - 1)` as the randomness seed. Block
   producers can influence this. For production, a VRF oracle or commit-reveal
   scheme is recommended. The NEAR port benefits from `env::random_seed()`
   which provides unpredictable per-block randomness.

3. **Centralization risks**: Several contracts have admin/owner roles with
   significant power:
   - `WorkerRegistry.owner` can authorize callers to slash and update reputation
   - `IdentityRegistry.admin` can demote tiers and revoke passports
   - `RoleRegistry.admin` can grant/revoke all roles
   - `ISFROracle.setBountyPool()` has **no access modifier** -- any address
     can set the bounty pool address

4. **Economic attacks on reputation**: The EMA model with alpha=0.2 means
   a single failure drops reputation by 20% of the current value. A colluding
   poster+resolver could systematically reject legitimate work to destroy a
   competitor's reputation. The `ConsortiumValidator` mitigates this by
   requiring committee consensus, but a majority attack (2 of 3 colluding
   validators) remains possible.

5. **Missing deadline enforcement**: `BountyMarket` records a `deadline` but
   does not enforce it in `assign()` or `submit()`. A worker could claim a
   job after the deadline. Adding `if (block.timestamp > j.deadline) revert
   DeadlinePassed()` to `assign()` and `submit()` would fix this.

6. **InsightBoard reward depletion**: The contract pays `REWARD_PER_CONFIRM`
   (1 ether) per confirmation. If the contract runs out of tokens, `claim()`
   will revert (`require(ok, "transfer failed")`). There is no mechanism to
   pause confirmations when funds are low, so users accumulate un-claimable
   earnings.

7. **Dust in FeeDistributor**: Integer division can leave dust tokens in the
   contract. The remainder-distribution logic (first N recipients get +1)
   handles this correctly for intra-group splits, but the outer split
   (`amount - validatorShare - dataShare - agentShare`) sends all remainder
   to treasury.

8. **Storage growth**: Several contracts use unbounded arrays
   (`_registered`, `_passportJobHashes`, `_domainStakeKeys`). On-chain
   enumeration (e.g., `ConsortiumValidator` iterating all workers) may hit
   gas limits at scale. Production deployments should use pagination or
   off-chain indexing with on-chain verification.

9. **FeeDistributor push pattern**: The `_credit()` function calls
   `rewardToken.transfer()` directly to each recipient. If any recipient
   is a contract that reverts on token receipt, the entire `distribute()`
   call fails. A pull-based withdrawal pattern (`withdraw()` by each
   participant) would be more robust.

---

## 14. References

[1] ERC-8004: The Universal Standard for AI Agent Identity. Ethereum
Improvement Proposals.
https://www.chainup.com/blog/erc-8004-ai-agent-identity-standard/

[2] ERC-5192: Minimal Soulbound NFTs. Ethereum Improvement Proposals.
https://eips.ethereum.org/EIPS/eip-5192

[3] Exponential Moving Averages at Scale: Building Smart Time-Decay Systems.
Open Data Science Conference.
https://odsc.medium.com/exponential-moving-averages-at-scale-building-smart-time-decay-systems-45d7509cd6ae

[4] Foundry: Ethereum Development Framework.
https://www.getfoundry.sh/

[5] revm: Rust implementation of the Ethereum Virtual Machine.
https://github.com/bluealloy/revm

[6] NEAR SDK for Rust: Library for writing NEAR smart contracts.
https://github.com/near/near-sdk-rs

[7] NEAR Protocol Storage Staking.
https://docs.near.org/protocol/storage/storage-staking

[8] Aurora: Ethereum Compatibility on NEAR.
https://docs.near.org/aurora/what-is

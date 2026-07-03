# Solidity Contract Reference

> All 13 Solidity contracts in the Roko/IronClaw contract suite.
> Source: [`contracts/src/`](https://github.com/wpank/roko/blob/main/contracts/src/)

Navigation: [README](./README.md) | [EVM Simulator](./evm-simulator.md) | [NEAR Contracts](./near-contracts.md) | [Benchmarks](./benchmarking.md) | [IronClaw Integration](./ironclaw-integration.md) | [References](./references.md)

---

## Build System and Dependencies

**File**:
[`contracts/foundry.toml`](https://github.com/wpank/roko/blob/main/contracts/foundry.toml)

The project uses [Foundry](https://www.getfoundry.sh/) (forge) for
compilation and testing [8]. The Foundry configuration targets the Shanghai
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

**Remappings**
([`contracts/remappings.txt`](https://github.com/wpank/roko/blob/main/contracts/remappings.txt)):

```
forge-std/=lib/forge-std/src/
@openzeppelin/contracts/=lib/openzeppelin-contracts/contracts/
```

The contract suite depends on:

- `forge-std` for testing infrastructure (`Test`, `console2`, `Script`)
- `@openzeppelin/contracts` for `ERC20`, `IERC20`

---

## 4.1 MockERC20 -- Test Token

**File**:
[`contracts/src/MockERC20.sol`](https://github.com/wpank/roko/blob/main/contracts/src/MockERC20.sol)

A minimal ERC20 token named "DAEJI" with an open `mint()` function. Used as
the settlement token across all contracts in the demo environment.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/// @title MockERC20
/// @notice Test-only token. NEVER deploy to mainnet.
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

    /// @notice Anyone can mint -- test-environment only.
    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}
```

**Storage**: Inherits OpenZeppelin's ERC20 storage (balances, allowances,
`totalSupply`). Adds a single immutable `_decimals`.

**Security**: The open `mint()` is intentional for testing. The contract
NatSpec explicitly warns "Never deploy to mainnet."

**Gas profile**:

| Operation | Gas |
|-----------|-----|
| `mint()` | ~51,000 (first mint to address) / ~34,000 (subsequent) |
| `transfer()` | ~29,000 |
| `approve()` + `transferFrom()` | ~46,000 + ~34,000 |

---

## 4.2 RoleRegistry -- Access Control

**File**:
[`contracts/src/RoleRegistry.sol`](https://github.com/wpank/roko/blob/main/contracts/src/RoleRegistry.sol)

A lightweight RBAC contract that maps `bytes32 role => address => bool`.
Used by the ISFR oracle contracts (ISFROracle and ISFRBountyPool) for
keeper and oracle role management.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

/// @title RoleRegistry
/// @notice Minimal RBAC: single admin, arbitrary role bytes32 keys.
contract RoleRegistry {
    address public admin;
    mapping(bytes32 => mapping(address => bool)) private _roles;

    event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender);
    event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender);
    event AdminTransferred(address indexed oldAdmin, address indexed newAdmin);

    error NotAdmin();
    error ZeroAddress();

    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAdmin();
        _;
    }

    constructor(address admin_) {
        if (admin_ == address(0)) revert ZeroAddress();
        admin = admin_;
    }

    function grantRole(bytes32 role, address account) external onlyAdmin {
        _roles[role][account] = true;
        emit RoleGranted(role, account, msg.sender);
    }

    function revokeRole(bytes32 role, address account) external onlyAdmin {
        _roles[role][account] = false;
        emit RoleRevoked(role, account, msg.sender);
    }

    function hasRole(bytes32 role, address account) external view returns (bool) {
        return _roles[role][account];
    }

    function transferAdmin(address newAdmin) external onlyAdmin {
        if (newAdmin == address(0)) revert ZeroAddress();
        emit AdminTransferred(admin, newAdmin);
        admin = newAdmin;
    }
}
```

**Design choices**: Unlike OpenZeppelin's `AccessControl`, this is a
standalone mapping with a single admin -- no role hierarchy, no role-admin
concept. This simplicity is appropriate for the ISFR subsystem where only
two roles are needed (`KEEPER_ROLE` and `ORACLE_ROLE`). The constructor
takes an explicit `admin_` address rather than defaulting to `msg.sender`,
enabling deployment by a factory that installs a different admin.

---

## 4.3 AgentRegistry -- Lightweight Agent Identity

**File**:
[`contracts/src/AgentRegistry.sol`](https://github.com/wpank/roko/blob/main/contracts/src/AgentRegistry.sol)

Tracks `address -> Agent` with capabilities, passport hash, and heartbeat
liveness. A complementary on-chain counterpart to the `0xA09` precompile
exposed by mirage-rs. Agents call `register()` once, then `heartbeat()`
periodically; `isActive()` returns true within a 200-block liveness window.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

/// @title AgentRegistry
/// @notice Heartbeat-based liveness tracking for autonomous agents.
contract AgentRegistry {
    struct Agent {
        string capabilities;    // JSON-encoded capability descriptor
        bytes32 passportHash;   // Hash of agent's identity document
        uint64 registeredAt;    // Block number of first registration
        uint64 lastHeartbeat;   // Block number of most recent heartbeat
        bool exists;
    }

    /// @notice Agents must heartbeat within this many blocks to be considered active.
    uint64 public constant LIVENESS_WINDOW = 200;

    mapping(address => Agent) private _agents;
    address[] private _registered;

    event AgentRegistered(address indexed agent, bytes32 passportHash, string capabilities);
    event AgentHeartbeat(address indexed agent, uint64 blockNumber);
    event AgentCapabilitiesUpdated(address indexed agent, string capabilities);

    error AlreadyRegistered();
    error NotRegistered();

    function register(string calldata capabilities, bytes32 passportHash) external {
        if (_agents[msg.sender].exists) revert AlreadyRegistered();
        _agents[msg.sender] = Agent({
            capabilities: capabilities,
            passportHash: passportHash,
            registeredAt: uint64(block.number),
            lastHeartbeat: uint64(block.number),
            exists: true
        });
        _registered.push(msg.sender);
        emit AgentRegistered(msg.sender, passportHash, capabilities);
    }

    function heartbeat() external {
        if (!_agents[msg.sender].exists) revert NotRegistered();
        _agents[msg.sender].lastHeartbeat = uint64(block.number);
        emit AgentHeartbeat(msg.sender, uint64(block.number));
    }

    function updateCapabilities(string calldata capabilities) external {
        if (!_agents[msg.sender].exists) revert NotRegistered();
        _agents[msg.sender].capabilities = capabilities;
        emit AgentCapabilitiesUpdated(msg.sender, capabilities);
    }

    function isActive(address agent) external view returns (bool) {
        Agent storage a = _agents[agent];
        return a.exists && (block.number - a.lastHeartbeat <= LIVENESS_WINDOW);
    }

    function getAgent(address agent) external view returns (Agent memory) {
        return _agents[agent];
    }

    function registeredCount() external view returns (uint256) {
        return _registered.length;
    }

    function registeredAt(uint256 index) external view returns (address) {
        return _registered[index];
    }
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

## 4.4 IdentityRegistry -- ERC-8004 Soulbound Passports

**File**:
[`contracts/src/IdentityRegistry.sol`](https://github.com/wpank/roko/blob/main/contracts/src/IdentityRegistry.sol)

The richest contract in the suite. Implements the ERC-8004 agent identity
standard [5] with a soulbound ERC-721 surface (compliant with ERC-5192 [6]).
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

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

/// @dev Minimal ERC20 interface used to avoid pulling in all of OZ.
interface IERC20Minimal {
    function transfer(address to, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
}

/// @title IdentityRegistry
/// @notice ERC-8004 soulbound passport with capability bitmask, TEE attestation,
///         domain staking, and 24-hour prompt-hash timelock.
contract IdentityRegistry {
    // ---- Capability bits ----
    uint64 public constant CAP_INFERENCE        = 1 << 0;
    uint64 public constant CAP_DATA_TRANSFORM   = 1 << 1;
    uint64 public constant CAP_FINE_TUNE        = 1 << 2;
    uint64 public constant CAP_RAG              = 1 << 3;
    uint64 public constant CAP_MULTI_AGENT      = 1 << 4;
    uint64 public constant CAP_TRADING          = 1 << 5;
    uint64 public constant CAP_SECURITY         = 1 << 6;
    uint64 public constant CAP_ANALYTICS        = 1 << 7;
    uint64 public constant CAP_KNOWLEDGE        = 1 << 8;
    uint64 public constant CAP_STRATEGY         = 1 << 9;

    // ---- Tier thresholds ----
    uint256 public constant TIER_WORKER_STAKE    = 5_000e18;
    uint256 public constant TIER_SOVEREIGN_STAKE = 25_000e18;

    // ---- Prompt timelock ----
    uint256 public constant PROMPT_UPDATE_DELAY  = 1 days;

    // ---- Withdrawal cooldown ----
    uint256 public constant WITHDRAW_COOLDOWN    = 7 days;

    // ---- Tier enum ----
    enum Tier { Protocol, Sovereign, Worker, Edge }

    struct PassportData {
        address owner;
        uint64  capabilityBitmask;
        uint8   tier;               // Tier enum packed as uint8
        bytes32 systemPromptHash;
        bytes32 teeAttestation;
        uint64  teeExpiry;
        string  agentCardUri;
        uint256 totalStaked;
        uint256 pendingPromptHash_value;
        uint256 pendingPromptHash_deadline;
    }

    struct DomainStake {
        uint256 amount;
        uint256 stakedAt;
    }

    uint256 private _nextId = 1;
    address public admin;

    mapping(address => bool) public registrars;
    mapping(uint256 => PassportData) private _passports;
    mapping(address => uint256) private _ownerToId;
    mapping(uint256 => mapping(string => DomainStake)) private _domainStakes;

    IERC20Minimal public immutable stakeToken;

    event PassportMinted(uint256 indexed passportId, address indexed owner, uint8 tier);
    event TierUpdated(uint256 indexed passportId, uint8 oldTier, uint8 newTier);
    event PromptHashScheduled(uint256 indexed passportId, bytes32 newHash, uint256 effectiveAt);
    event PromptHashConfirmed(uint256 indexed passportId, bytes32 newHash);
    event DomainStaked(uint256 indexed passportId, string domain, uint256 amount);
    event DomainUnstaked(uint256 indexed passportId, string domain, uint256 amount);

    error Soulbound();
    error AlreadyRegistered();
    error NotAuthorized();
    error TimelockActive();
    error CooldownActive();
    error InsufficientStake();
    error NoPassport();

    modifier onlyAdmin() {
        if (msg.sender != admin) revert NotAuthorized();
        _;
    }

    constructor(address admin_, address stakeToken_) {
        admin = admin_;
        stakeToken = IERC20Minimal(stakeToken_);
    }

    // ---- ERC-721 soulbound surface ----

    function name() external pure returns (string memory) { return "Korai Passport"; }
    function symbol() external pure returns (string memory) { return "KPASS"; }

    /// @dev ERC-5192: always locked
    function locked(uint256) external pure returns (bool) { return true; }

    function transferFrom(address, address, uint256) external pure { revert Soulbound(); }
    function safeTransferFrom(address, address, uint256) external pure { revert Soulbound(); }
    function safeTransferFrom(address, address, uint256, bytes calldata) external pure { revert Soulbound(); }
    function approve(address, uint256) external pure { revert Soulbound(); }
    function setApprovalForAll(address, bool) external pure { revert Soulbound(); }

    function supportsInterface(bytes4 interfaceId) external pure returns (bool) {
        return interfaceId == 0x01ffc9a7   // ERC-165
            || interfaceId == 0x80ac58cd   // ERC-721
            || interfaceId == 0x5b5e139f   // ERC-721Metadata
            || interfaceId == 0xb45a3c0e;  // ERC-5192 Soulbound
    }

    function ownerOf(uint256 passportId) external view returns (address) {
        return _passports[passportId].owner;
    }

    function balanceOf(address owner_) external view returns (uint256) {
        return _ownerToId[owner_] != 0 ? 1 : 0;
    }

    // ---- Registration ----

    /// @notice Full ERC-8004 registration with TEE attestation and auto-tier.
    function registerPassport(
        address owner_,
        uint64 capabilityBitmask,
        bytes32 systemPromptHash,
        bytes32 teeAttestation,
        uint64 teeExpiry
    ) external returns (uint256 passportId) {
        _assertRegistrar();
        if (_ownerToId[owner_] != 0) revert AlreadyRegistered();
        passportId = _nextId++;
        _passports[passportId] = PassportData({
            owner: owner_,
            capabilityBitmask: capabilityBitmask,
            tier: uint8(Tier.Edge),
            systemPromptHash: systemPromptHash,
            teeAttestation: teeAttestation,
            teeExpiry: teeExpiry,
            agentCardUri: "",
            totalStaked: 0,
            pendingPromptHash_value: 0,
            pendingPromptHash_deadline: 0
        });
        _ownerToId[owner_] = passportId;
        emit PassportMinted(passportId, owner_, uint8(Tier.Edge));
    }

    /// @notice Simplified registration with explicit tier and agent card URI.
    function register(
        address agent,
        uint64 capabilityList,
        uint8 tier,
        bytes32 systemPromptHash,
        string calldata agentCardUri
    ) external returns (uint256 passportId) {
        _assertRegistrar();
        if (_ownerToId[agent] != 0) revert AlreadyRegistered();
        passportId = _nextId++;
        _passports[passportId] = PassportData({
            owner: agent,
            capabilityBitmask: capabilityList,
            tier: tier,
            systemPromptHash: systemPromptHash,
            teeAttestation: bytes32(0),
            teeExpiry: 0,
            agentCardUri: agentCardUri,
            totalStaked: 0,
            pendingPromptHash_value: 0,
            pendingPromptHash_deadline: 0
        });
        _ownerToId[agent] = passportId;
        emit PassportMinted(passportId, agent, tier);
    }

    // ---- Prompt hash timelock (ventriloquist defense) ----

    /// @notice Schedule a prompt hash update. Requires a second call after PROMPT_UPDATE_DELAY.
    function updateSystemPromptHash(uint256 passportId, bytes32 newHash) external {
        _assertOwner(passportId);
        PassportData storage p = _passports[passportId];
        if (p.pendingPromptHash_deadline != 0 &&
            block.timestamp < p.pendingPromptHash_deadline) {
            revert TimelockActive();
        }
        if (p.pendingPromptHash_value == uint256(newHash) &&
            p.pendingPromptHash_deadline != 0 &&
            block.timestamp >= p.pendingPromptHash_deadline) {
            // Second call: confirm
            p.systemPromptHash = newHash;
            p.pendingPromptHash_value = 0;
            p.pendingPromptHash_deadline = 0;
            emit PromptHashConfirmed(passportId, newHash);
        } else {
            // First call: schedule
            p.pendingPromptHash_value = uint256(newHash);
            p.pendingPromptHash_deadline = block.timestamp + PROMPT_UPDATE_DELAY;
            emit PromptHashScheduled(passportId, newHash, p.pendingPromptHash_deadline);
        }
    }

    // ---- Domain staking ----

    function stakeIntoDomain(uint256 passportId, string calldata domain, uint256 amount)
        external
    {
        _assertOwner(passportId);
        require(stakeToken.transferFrom(msg.sender, address(this), amount), "Transfer failed");
        PassportData storage p = _passports[passportId];
        p.totalStaked += amount;
        _domainStakes[passportId][domain].amount += amount;
        _domainStakes[passportId][domain].stakedAt = block.timestamp;
        _syncTier(passportId);
        emit DomainStaked(passportId, domain, amount);
    }

    function withdrawFromDomain(uint256 passportId, string calldata domain, uint256 amount)
        external
    {
        _assertOwner(passportId);
        DomainStake storage ds = _domainStakes[passportId][domain];
        if (block.timestamp < ds.stakedAt + WITHDRAW_COOLDOWN) revert CooldownActive();
        if (ds.amount < amount) revert InsufficientStake();
        ds.amount -= amount;
        _passports[passportId].totalStaked -= amount;
        require(stakeToken.transfer(msg.sender, amount), "Transfer failed");
        _syncTier(passportId);
        emit DomainUnstaked(passportId, domain, amount);
    }

    // ---- Capability checks ----

    function hasCapability(uint256 passportId, uint64 capBit) external view returns (bool) {
        return _passports[passportId].capabilityBitmask & capBit != 0;
    }

    function getPassport(uint256 passportId) external view returns (PassportData memory) {
        return _passports[passportId];
    }

    function passportOf(address owner_) external view returns (uint256) {
        return _ownerToId[owner_];
    }

    // ---- Internals ----

    function _syncTier(uint256 passportId) internal {
        PassportData storage p = _passports[passportId];
        uint8 oldTier = p.tier;
        uint8 newTier;
        if (p.totalStaked >= TIER_SOVEREIGN_STAKE) newTier = uint8(Tier.Sovereign);
        else if (p.totalStaked >= TIER_WORKER_STAKE) newTier = uint8(Tier.Worker);
        else newTier = uint8(Tier.Edge);
        if (newTier != oldTier) {
            p.tier = newTier;
            emit TierUpdated(passportId, oldTier, newTier);
        }
    }

    function _assertOwner(uint256 passportId) internal view {
        if (_passports[passportId].owner != msg.sender) revert NotAuthorized();
    }

    function _assertRegistrar() internal view {
        if (msg.sender != admin && !registrars[msg.sender]) revert NotAuthorized();
    }

    function setRegistrar(address registrar, bool enabled) external onlyAdmin {
        registrars[registrar] = enabled;
    }
}
```

---

## 4.5 WorkerRegistry -- Stake, Reputation, Tiers

**File**:
[`contracts/src/WorkerRegistry.sol`](https://github.com/wpank/roko/blob/main/contracts/src/WorkerRegistry.sol)

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
more weight to recent observations [7]. In this context it means a single
failure drops reputation by `alpha * R_old = 0.2 * R_old`, and recovery
requires multiple successive successes.

**Decay model**: Reputation halves toward 0.5 every 30 days of inactivity
(`DECAY_PERIOD = 30 days`). Decay is applied lazily -- `_applyDecay()` runs
on the next `updateReputation()` or `slash()` call.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/// @title WorkerRegistry
/// @notice Stake bonds, EMA reputation, lazy decay, tier classification.
contract WorkerRegistry {
    uint256 public constant SCALE          = 1_000_000;
    uint256 public constant ALPHA_NUM      = 200_000;   // alpha = 0.2
    uint256 public constant INIT_REP       = 500_000;   // starting reputation = 0.5
    uint256 public constant MIN_BOND       = 1_000e18;  // 1,000 DAEJI
    uint64  public constant DECAY_PERIOD   = 30 days;

    // Slash reason codes
    uint8 public constant SLASH_MISSED_DEADLINE = 1;  // 100 bps (1%)
    uint8 public constant SLASH_QUALITY_REJECT  = 2;  // 500 bps (5%)
    uint8 public constant SLASH_ABANDONMENT     = 3;  // 1000 bps (10%)

    enum Tier { Unregistered, Probation, Standard, Trusted, Elite }

    struct Worker {
        uint256 bond;
        uint256 reputation;      // SCALE-encoded
        uint64  jobsCompleted;
        uint64  jobsSlashed;
        uint64  lastUpdated;     // block.timestamp
        bool    exists;
    }

    IERC20  public immutable stakeToken;
    address public owner;
    mapping(address => bool)   public authorized;
    mapping(address => Worker) private _workers;
    address[] private _registered;

    event WorkerRegistered(address indexed worker, uint256 bond);
    event ReputationUpdated(address indexed worker, bool outcome, uint256 newRep);
    event WorkerSlashed(address indexed worker, uint8 reason, uint256 amount);
    event BondIncreased(address indexed worker, uint256 addedAmount);
    event WorkerDeregistered(address indexed worker, uint256 bondRefunded);

    error AlreadyRegistered();
    error NotRegistered();
    error NotAuthorized();
    error BondTooLow();
    error BondInsufficient();

    modifier onlyAuthorized() {
        if (!authorized[msg.sender] && msg.sender != owner) revert NotAuthorized();
        _;
    }

    constructor(address stakeToken_) {
        stakeToken = IERC20(stakeToken_);
        owner = msg.sender;
    }

    function register(uint256 bond) external {
        if (_workers[msg.sender].exists) revert AlreadyRegistered();
        if (bond < MIN_BOND) revert BondTooLow();
        require(stakeToken.transferFrom(msg.sender, address(this), bond));
        _workers[msg.sender] = Worker({
            bond: bond,
            reputation: INIT_REP,
            jobsCompleted: 0,
            jobsSlashed: 0,
            lastUpdated: uint64(block.timestamp),
            exists: true
        });
        _registered.push(msg.sender);
        emit WorkerRegistered(msg.sender, bond);
    }

    function updateReputation(address worker, bool outcome) external onlyAuthorized {
        Worker storage w = _workers[worker];
        if (!w.exists) revert NotRegistered();
        _applyDecay(w);
        uint256 observation = outcome ? SCALE : 0;
        w.reputation = (ALPHA_NUM * observation + (SCALE - ALPHA_NUM) * w.reputation) / SCALE;
        if (outcome) w.jobsCompleted++;
        else w.jobsSlashed++;
        w.lastUpdated = uint64(block.timestamp);
        emit ReputationUpdated(worker, outcome, w.reputation);
    }

    function slash(address worker, uint8 reason) external onlyAuthorized {
        Worker storage w = _workers[worker];
        if (!w.exists) revert NotRegistered();
        _applyDecay(w);
        uint256 bps;
        if (reason == SLASH_MISSED_DEADLINE) bps = 100;
        else if (reason == SLASH_QUALITY_REJECT) bps = 500;
        else if (reason == SLASH_ABANDONMENT) bps = 1000;
        uint256 slashAmt = w.bond * bps / 10_000;
        w.bond -= slashAmt;
        emit WorkerSlashed(worker, reason, slashAmt);
        // Slashed tokens go to treasury / could be redistributed
        require(stakeToken.transfer(owner, slashAmt));
    }

    function tierOf(address worker) external view returns (Tier) {
        Worker storage w = _workers[worker];
        if (!w.exists) return Tier.Unregistered;
        uint256 r = w.reputation;
        if (r < 350_000) return Tier.Probation;
        if (r < 550_000) return Tier.Standard;
        if (r < 800_000) return Tier.Trusted;
        return Tier.Elite;
    }

    function canAccept(address worker, Tier minTier) external view returns (bool) {
        Worker storage w = _workers[worker];
        if (!w.exists) return false;
        uint256 r = w.reputation;
        Tier t;
        if (r < 350_000) t = Tier.Probation;
        else if (r < 550_000) t = Tier.Standard;
        else if (r < 800_000) t = Tier.Trusted;
        else t = Tier.Elite;
        return t >= minTier;
    }

    function registeredCount() external view returns (uint256) { return _registered.length; }
    function registeredAt(uint256 i) external view returns (address) { return _registered[i]; }
    function getWorker(address worker) external view returns (Worker memory) { return _workers[worker]; }

    function setAuthorized(address caller, bool enabled) external {
        if (msg.sender != owner) revert NotAuthorized();
        authorized[caller] = enabled;
    }

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
}
```

**Tier classification** (view-only, applied to the decay-adjusted reputation):

| Tier | Reputation Range |
|------|-----------------|
| Probation | < 350,000 (0.35) |
| Standard | 350,000 - 549,999 |
| Trusted | 550,000 - 799,999 |
| Elite | >= 800,000 (0.80) |

**Slashing**:

| Code | Reason | Slash Rate |
|------|--------|-----------|
| 1 | `SLASH_MISSED_DEADLINE` | 1% (100 bps) |
| 2 | `SLASH_QUALITY_REJECT` | 5% (500 bps) |
| 3 | `SLASH_ABANDONMENT` | 10% (1000 bps) |

---

## 4.6 ReputationRegistry -- Domain-Specific Reputation

**File**:
[`contracts/src/ReputationRegistry.sol`](https://github.com/wpank/roko/blob/main/contracts/src/ReputationRegistry.sol)

A second-generation reputation contract that ties to `IdentityRegistry`
passport IDs (not raw addresses like `WorkerRegistry`). Tracks reputation
per domain with two feedback paths.

**7 reputation domains** (enum):
- OracleResolution, RiskDetection, AnomalyFlagging, DataIntegrity
- CrossAppValidation, SealedExecution, KnowledgeVerification

**Authorized-source feedback**: An authorized contract submits a signed score
`[-1e18, +1e18]` for a passport in a named domain. The score is normalized to
`[0, 1e18]` and fed through an adaptive EMA:

```solidity
function _adaptiveAlpha(uint64 jobCount) internal pure returns (uint256) {
    // When jobCount is small, alpha is high (new data has large weight).
    // As history grows, alpha converges toward MAX_ALPHA = 0.3.
    uint256 adaptive = (2 * SCALE) / (uint256(jobCount) + 1);
    return adaptive < MAX_ALPHA ? adaptive : MAX_ALPHA;
}
```

**Peer feedback**: An agent with a valid passport can rate another agent,
but only if the pair has been pre-authorized for the specific domain via
`authorizeFeedback()`. Scores are 0-1000 (mapped to 18-decimal fixed-point).

**Decay**: Same halving-toward-midpoint model as `WorkerRegistry`, with
`DECAY_PERIOD = 30 days`.

---

## 4.7 BountyMarket -- Programmable Escrow

**File**:
[`contracts/src/BountyMarket.sol`](https://github.com/wpank/roko/blob/main/contracts/src/BountyMarket.sol)

ERC-8183-style 4-state programmable escrow for agent task coordination.

**Job lifecycle state machine**:

```
              postJob()         assign()         submit()         resolve()
[None] -----> [Funded] ------> [Assigned] -----> [Submitted] ----> [Terminal]
```

Note: the `Open` state exists in the enum but `postJob()` atomically
transitions `Open -> Funded` by requiring `bounty` to be pre-approved and
pulling tokens in the same call.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

interface IWorkerRegistry {
    enum Tier { Unregistered, Probation, Standard, Trusted, Elite }
    function canAccept(address worker, Tier minTier) external view returns (bool);
    function updateReputation(address worker, bool outcome) external;
    function slash(address worker, uint8 reason) external;
}

/// @title BountyMarket
/// @notice Trustless escrow for multi-agent work coordination.
contract BountyMarket {
    enum JobState { Open, Funded, Assigned, Submitted, Terminal }

    struct Job {
        address poster;
        address worker;
        bytes32 specHash;       // Keccak256 of job specification
        bytes32 resultHash;     // Keccak256 of submitted result
        uint256 bounty;
        uint64  deadline;
        uint8   minTier;        // Minimum IWorkerRegistry.Tier required
        JobState state;
        bool    accepted;
    }

    IERC20           public immutable bountyToken;
    IWorkerRegistry  public immutable workerRegistry;
    address          public resolver;

    uint256 private _nextId = 1;
    mapping(uint256 => Job) private _jobs;

    event JobPosted(uint256 indexed id, address indexed poster, bytes32 specHash, uint256 bounty);
    event JobAssigned(uint256 indexed id, address indexed worker);
    event WorkSubmitted(uint256 indexed id, bytes32 resultHash);
    event JobResolved(uint256 indexed id, bool accepted);

    error Unauthorized();
    error InvalidState(JobState expected, JobState actual);
    error TierTooLow();
    error DeadlinePassed();

    constructor(address bountyToken_, address workerRegistry_) {
        bountyToken = IERC20(bountyToken_);
        workerRegistry = IWorkerRegistry(workerRegistry_);
        resolver = msg.sender;
    }

    function postJob(
        bytes32 specHash,
        uint256 bounty,
        uint64  deadline,
        uint8   minTier
    ) external returns (uint256 id) {
        require(bountyToken.transferFrom(msg.sender, address(this), bounty));
        id = _nextId++;
        _jobs[id] = Job({
            poster: msg.sender,
            worker: address(0),
            specHash: specHash,
            resultHash: bytes32(0),
            bounty: bounty,
            deadline: deadline,
            minTier: minTier,
            state: JobState.Funded,
            accepted: false
        });
        emit JobPosted(id, msg.sender, specHash, bounty);
    }

    function assign(uint256 id, address worker) external {
        Job storage j = _jobs[id];
        if (j.state != JobState.Funded) revert InvalidState(JobState.Funded, j.state);
        if (!workerRegistry.canAccept(worker, IWorkerRegistry.Tier(j.minTier))) revert TierTooLow();
        if (block.timestamp > j.deadline) revert DeadlinePassed();
        j.worker = worker;
        j.state = JobState.Assigned;
        emit JobAssigned(id, worker);
    }

    function submit(uint256 id, bytes32 resultHash) external {
        Job storage j = _jobs[id];
        if (msg.sender != j.worker) revert Unauthorized();
        if (j.state != JobState.Assigned) revert InvalidState(JobState.Assigned, j.state);
        if (block.timestamp > j.deadline) revert DeadlinePassed();
        j.resultHash = resultHash;
        j.state = JobState.Submitted;
        emit WorkSubmitted(id, resultHash);
    }

    function resolve(uint256 id, bool accepted) external {
        if (msg.sender != resolver) revert Unauthorized();
        Job storage j = _jobs[id];
        if (j.state != JobState.Submitted) revert InvalidState(JobState.Submitted, j.state);
        j.state = JobState.Terminal;
        j.accepted = accepted;

        address recipient = accepted ? j.worker : j.poster;
        require(bountyToken.transfer(recipient, j.bounty));

        workerRegistry.updateReputation(j.worker, accepted);
        if (!accepted) {
            workerRegistry.slash(j.worker, 2); // SLASH_QUALITY_REJECT
        }
        emit JobResolved(id, accepted);
    }

    function setResolver(address newResolver) external {
        if (msg.sender != resolver) revert Unauthorized();
        resolver = newResolver;
    }

    function getJob(uint256 id) external view returns (Job memory) {
        return _jobs[id];
    }
}
```

**Resolution effects**:

| Outcome | Token Flow | Reputation | Slash |
|---------|-----------|------------|-------|
| Accepted | Bounty -> worker | `updateReputation(worker, true)` | None |
| Rejected | Bounty -> poster (refund) | `updateReputation(worker, false)` | 500 bps (5%) via `SLASH_QUALITY_REJECT` |

---

## 4.8 ConsortiumValidator -- 2-of-3 Validation Committee

**File**:
[`contracts/src/ConsortiumValidator.sol`](https://github.com/wpank/roko/blob/main/contracts/src/ConsortiumValidator.sol)

Assembles a 3-agent validation committee from Trusted+ workers and
drives resolution through 2-of-3 majority voting.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

/// @title ConsortiumValidator
/// @notice Assembles a 3-member committee and resolves bounties by 2-of-3 vote.
contract ConsortiumValidator {
    struct Committee {
        address[3] members;
        mapping(address => bool) voted;
        uint8 approves;
        uint8 rejects;
        bool tallied;
    }

    IWorkerRegistry public immutable workerRegistry;
    IBountyMarket   public immutable market;

    mapping(uint256 => Committee) private _committees;

    event CommitteeAssembled(uint256 indexed jobId, address[3] members);
    event VoteCast(uint256 indexed jobId, address indexed voter, bool approve);
    event Tallied(uint256 indexed jobId, bool accepted);

    error NotCommitteeMember();
    error AlreadyVoted();
    error AlreadyTallied();
    error InsufficientCandidates();

    constructor(address workerRegistry_, address market_) {
        workerRegistry = IWorkerRegistry(workerRegistry_);
        market = IBountyMarket(market_);
    }

    function assembleCommittee(uint256 jobId) external {
        uint256 n = workerRegistry.registeredCount();
        address[] memory candidates = new address[](n);
        uint256 count = 0;
        for (uint256 i = 0; i < n; i++) {
            address w = workerRegistry.registeredAt(i);
            if (workerRegistry.canAccept(w, IWorkerRegistry.Tier.Trusted)) {
                candidates[count++] = w;
            }
        }
        if (count < 3) revert InsufficientCandidates();

        // Fisher-Yates selection using blockhash as seed
        bytes32 seed = blockhash(block.number - 1);
        address[3] memory members;
        for (uint256 k = 0; k < 3; k++) {
            uint256 idx = uint256(keccak256(abi.encode(seed, k))) % (count - k);
            members[k] = candidates[idx];
            candidates[idx] = candidates[count - k - 1];
        }

        Committee storage c = _committees[jobId];
        c.members = members;
        emit CommitteeAssembled(jobId, members);
    }

    function vote(uint256 jobId, bool approve) external {
        Committee storage c = _committees[jobId];
        if (c.tallied) revert AlreadyTallied();
        bool isMember = msg.sender == c.members[0]
                     || msg.sender == c.members[1]
                     || msg.sender == c.members[2];
        if (!isMember) revert NotCommitteeMember();
        if (c.voted[msg.sender]) revert AlreadyVoted();
        c.voted[msg.sender] = true;
        if (approve) c.approves++;
        else c.rejects++;
        emit VoteCast(jobId, msg.sender, approve);

        if (c.approves >= 2 || c.rejects >= 2) {
            c.tallied = true;
            bool accepted = c.approves >= 2;
            market.resolve(jobId, accepted);
            emit Tallied(jobId, accepted);
        }
    }
}
```

**Security note**: The `blockhash(block.number - 1)` randomness is predictable
by block producers. For production deployments, a VRF oracle (Chainlink VRF,
drand) or commit-reveal scheme should replace the seed. On NEAR, the analog
is `env::random_seed()`, which is derived from the block's VRF output and is
not predictable by validators.

---

## 4.9 ValidationRegistry -- Work Proofs and Attestations

**File**:
[`contracts/src/ValidationRegistry.sol`](https://github.com/wpank/roko/blob/main/contracts/src/ValidationRegistry.sol)

ERC-8004 validation registry for work proofs and validator attestations.
Integrates with `IdentityRegistry` for passport-based access control.

**Two primary workflows**:

1. **Work proof submission**: An agent (or authorized submitter) submits a
   proof that work was done -- containing a job hash, deliverable Merkle root,
   array of gate results (pass/fail per quality gate), and a clearing
   certificate. Gate pass rates are tracked per passport for aggregate quality
   metrics.

```solidity
function submitWorkProof(
    uint256 passportId,
    bytes32 jobHash,
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

## 4.10 InsightBoard -- On-Chain Knowledge with Pheromone Curation

**File**:
[`contracts/src/InsightBoard.sol`](https://github.com/wpank/roko/blob/main/contracts/src/InsightBoard.sol)

A thin wrapper over mirage-rs's `0xA01` InsightEntry precompile. Agents post
content-hashed insights with URIs; others `confirm()` them, which
increments a pheromone weight counter and credits the poster with token rewards.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/// @title InsightBoard
/// @notice Content-addressed knowledge with pheromone-based curation rewards.
contract InsightBoard {
    struct Insight {
        address poster;
        bytes32 contentHash;
        string  uri;
        uint64  postedAt;
        uint64  pheromone;   // count of distinct confirmations
    }

    uint256 public constant REWARD_PER_CONFIRM = 1 ether;

    IERC20 public immutable rewardToken;
    uint256 private _nextId = 1;

    mapping(uint256 => Insight) private _insights;
    mapping(uint256 => mapping(address => bool)) private _confirmed;
    mapping(address => uint256) public earningsOf;

    event InsightPosted(uint256 indexed id, address indexed poster, bytes32 contentHash);
    event InsightConfirmed(uint256 indexed id, address indexed confirmer, uint64 pheromone);
    event EarningsClaimed(address indexed poster, uint256 amount);

    error SelfConfirm();
    error AlreadyConfirmed();
    error NoEarnings();

    constructor(address rewardToken_) {
        rewardToken = IERC20(rewardToken_);
    }

    function post(bytes32 contentHash, string calldata uri) external returns (uint256 id) {
        id = _nextId++;
        _insights[id] = Insight({
            poster: msg.sender,
            contentHash: contentHash,
            uri: uri,
            postedAt: uint64(block.timestamp),
            pheromone: 0
        });
        emit InsightPosted(id, msg.sender, contentHash);
    }

    function confirm(uint256 id) external {
        Insight storage ins = _insights[id];
        if (ins.poster == msg.sender) revert SelfConfirm();
        if (_confirmed[id][msg.sender]) revert AlreadyConfirmed();
        _confirmed[id][msg.sender] = true;
        ins.pheromone++;
        earningsOf[ins.poster] += REWARD_PER_CONFIRM;
        emit InsightConfirmed(id, msg.sender, ins.pheromone);
    }

    function claim() external {
        uint256 amount = earningsOf[msg.sender];
        if (amount == 0) revert NoEarnings();
        earningsOf[msg.sender] = 0;
        require(rewardToken.transfer(msg.sender, amount));
        emit EarningsClaimed(msg.sender, amount);
    }

    function getInsight(uint256 id) external view returns (Insight memory) {
        return _insights[id];
    }
}
```

---

## 4.11 ISFROracle -- Interest Rate Oracle

**File**:
[`contracts/src/ISFROracle.sol`](https://github.com/wpank/roko/blob/main/contracts/src/ISFROracle.sol)

On-chain storage for ISFR (Intelligent Synthetic Funding Rate) epoch-keyed
submissions. Keepers with `KEEPER_ROLE` (from `RoleRegistry`) submit rate
data each epoch.

```solidity
struct Rate {
    uint256 epochId;
    uint256 compositeBps;   // Composite funding rate in basis points
    uint256 lendingBps;
    uint256 structuredBps;
    uint256 fundingBps;
    uint256 stakingBps;
    uint256 confidenceBps;  // Confidence score in bps
    uint64  timestamp;
    address submitter;
}
```

The `submitRate()` function prevents duplicate epoch submissions and stores
both the epoch-specific rate and updates `currentRate`. The `getCurrentRate()`
view returns the latest composite rate in basis points.

**Known gap**: `setBountyPool()` has no access modifier -- any address can
set the bounty pool address. This must be fixed before mainnet deployment.

---

## 4.12 ISFRBountyPool -- Oracle Keeper Rewards

**File**:
[`contracts/src/ISFRBountyPool.sol`](https://github.com/wpank/roko/blob/main/contracts/src/ISFRBountyPool.sol)

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

## 4.13 FeeDistributor -- Multi-Party Fee Splitting

**File**:
[`contracts/src/FeeDistributor.sol`](https://github.com/wpank/roko/blob/main/contracts/src/FeeDistributor.sol)

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
) external;
```

**Fallback behavior**: If no validators or data providers are passed, their
share is redirected to the treasury. Within a group, tokens are split evenly
with remainder distributed to the first recipients (modular dust handling --
first `remainder` recipients get +1 wei).

**Limitation**: `_credit()` calls `token.transfer()` directly (push pattern).
If any recipient is a contract that reverts on receipt, the entire
`distribute()` call fails. A pull-based withdrawal pattern would be safer.

---

## Deployment Script and Contract Wiring

**File**:
[`contracts/script/Deploy.s.sol`](https://github.com/wpank/roko/blob/main/contracts/script/Deploy.s.sol)

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Script.sol";
import "../src/MockERC20.sol";
import "../src/AgentRegistry.sol";
import "../src/WorkerRegistry.sol";
import "../src/BountyMarket.sol";
import "../src/ConsortiumValidator.sol";
import "../src/InsightBoard.sol";

struct Deployed {
    MockERC20           daeji;
    AgentRegistry       agentReg;
    WorkerRegistry      workerReg;
    BountyMarket        market;
    ConsortiumValidator consortium;
    InsightBoard        board;
}

contract Deploy is Script {
    function run() external returns (Deployed memory d) {
        vm.startBroadcast();

        d.daeji      = new MockERC20("DAEJI", "DAEJI", 18);
        d.agentReg   = new AgentRegistry();
        d.workerReg  = new WorkerRegistry(address(d.daeji));
        d.market     = new BountyMarket(address(d.daeji), address(d.workerReg));
        d.consortium = new ConsortiumValidator(address(d.workerReg), address(d.market));
        d.board      = new InsightBoard(address(d.daeji));

        // Post-deploy wiring
        d.workerReg.setAuthorized(address(d.market), true);
        d.workerReg.setAuthorized(address(d.consortium), true);
        d.market.setResolver(address(d.consortium));

        vm.stopBroadcast();
    }
}
```

**What the script does NOT deploy**: `IdentityRegistry`, `ReputationRegistry`,
`ValidationRegistry`, `ISFROracle`, `ISFRBountyPool`, `FeeDistributor`, and
`RoleRegistry` are not in the reference deploy script. The ERC-8004 trinity
(Identity, Reputation, Validation) is bootstrapped separately by mirage-rs
at well-known addresses (see [evm-simulator.md](./evm-simulator.md)). The ISFR contracts and
FeeDistributor are deployed independently by their respective subsystems.

---

## Test Suite

Every contract has a corresponding Foundry test file in
[`contracts/test/`](https://github.com/wpank/roko/blob/main/contracts/test/):

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

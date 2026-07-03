# Reputation And Contract Blueprints

This document gives a self-contained implementation sketch for the reputation
and contract concepts in docs 08 and 24. The recommended IronClaw path is to
start off-chain, then add NEAR contracts only after the scoring semantics are
stable.

## Off-Chain Reputation First

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ReputationDomain {
    CodeQuality,
    Reliability,
    Accuracy,
    Creativity,
    Collaboration,
    Security,
    Efficiency,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReputationVector {
    pub scores: HashMap<ReputationDomain, f64>,
    pub observations: u64,
    pub updated_at_ms: i64,
}

impl ReputationVector {
    pub fn new(now_ms: i64) -> Self {
        let mut scores = HashMap::new();
        for domain in [
            ReputationDomain::CodeQuality,
            ReputationDomain::Reliability,
            ReputationDomain::Accuracy,
            ReputationDomain::Creativity,
            ReputationDomain::Collaboration,
            ReputationDomain::Security,
            ReputationDomain::Efficiency,
        ] {
            scores.insert(domain, 0.5);
        }
        Self { scores, observations: 0, updated_at_ms: now_ms }
    }

    pub fn observe(&mut self, domain: ReputationDomain, outcome: f64, now_ms: i64) {
        let old = *self.scores.get(&domain).unwrap_or(&0.5);
        let alpha = if self.observations < 10 {
            0.30
        } else if self.observations < 100 {
            0.12
        } else {
            0.04
        };
        let updated = alpha * outcome.clamp(0.0, 1.0) + (1.0 - alpha) * old;
        self.scores.insert(domain, updated);
        self.observations += 1;
        self.updated_at_ms = now_ms;
    }

    pub fn trust_score(&self) -> f64 {
        let weights = [
            (ReputationDomain::Reliability, 0.22),
            (ReputationDomain::Security, 0.20),
            (ReputationDomain::Accuracy, 0.18),
            (ReputationDomain::CodeQuality, 0.16),
            (ReputationDomain::Efficiency, 0.10),
            (ReputationDomain::Collaboration, 0.08),
            (ReputationDomain::Creativity, 0.06),
        ];
        weights.iter()
            .map(|(domain, weight)| self.scores.get(domain).copied().unwrap_or(0.5) * weight)
            .sum()
    }
}
```

IronClaw use cases:

- Extension reputation: success rate, denied actions, latency, user disables.
- MCP server reputation: tool success, timeout rate, approval denial rate.
- Model/provider reputation: quality pass, cost, latency, reliability.

## Solidity Soulbound Identity Sketch

This is the EVM-shaped concept from the captured contract suite. For IronClaw,
it is primarily a design reference unless EVM deployment becomes a requirement.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

contract AgentPassport {
    struct Passport {
        address owner;
        bytes32 metadataHash;
        uint64 issuedAt;
        uint64 revokedAt;
        uint32 capabilityMask;
    }

    mapping(uint256 => Passport) public passports;
    mapping(address => uint256) public passportOf;
    uint256 public nextId = 1;
    address public admin;

    event Issued(uint256 indexed tokenId, address indexed owner, bytes32 metadataHash);
    event Revoked(uint256 indexed tokenId, string reason);
    event CapabilityUpdated(uint256 indexed tokenId, uint32 capabilityMask);

    modifier onlyAdmin() {
        require(msg.sender == admin, "not admin");
        _;
    }

    constructor() {
        admin = msg.sender;
    }

    function issue(address owner, bytes32 metadataHash, uint32 capabilityMask)
        external
        onlyAdmin
        returns (uint256 tokenId)
    {
        require(owner != address(0), "zero owner");
        require(passportOf[owner] == 0, "already issued");
        tokenId = nextId++;
        passports[tokenId] = Passport({
            owner: owner,
            metadataHash: metadataHash,
            issuedAt: uint64(block.timestamp),
            revokedAt: 0,
            capabilityMask: capabilityMask
        });
        passportOf[owner] = tokenId;
        emit Issued(tokenId, owner, metadataHash);
    }

    function revoke(uint256 tokenId, string calldata reason) external onlyAdmin {
        Passport storage p = passports[tokenId];
        require(p.owner != address(0), "unknown");
        require(p.revokedAt == 0, "revoked");
        p.revokedAt = uint64(block.timestamp);
        emit Revoked(tokenId, reason);
    }

    function updateCapabilities(uint256 tokenId, uint32 capabilityMask) external onlyAdmin {
        Passport storage p = passports[tokenId];
        require(p.owner != address(0), "unknown");
        require(p.revokedAt == 0, "revoked");
        p.capabilityMask = capabilityMask;
        emit CapabilityUpdated(tokenId, capabilityMask);
    }

    function transferFrom(address, address, uint256) external pure {
        revert("soulbound");
    }

    function safeTransferFrom(address, address, uint256) external pure {
        revert("soulbound");
    }
}
```

## NEAR-Oriented Shape

For IronClaw, NEAR is a more natural target than EVM. The contract shape should
store compact, auditable records and leave rich evidence in IronClaw storage.

```rust
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};
use near_sdk::collections::LookupMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AgentRecord {
    owner: AccountId,
    metadata_hash: [u8; 32],
    capability_mask: u32,
    revoked: bool,
    trust_score_bps: u16,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    Agents,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AgentRegistry {
    admin: AccountId,
    agents: LookupMap<AccountId, AgentRecord>,
}

#[near_bindgen]
impl AgentRegistry {
    #[init]
    pub fn new(admin: AccountId) -> Self {
        Self { admin, agents: LookupMap::new(StorageKey::Agents) }
    }

    pub fn register_agent(
        &mut self,
        owner: AccountId,
        metadata_hash: Vec<u8>,
        capability_mask: u32,
    ) {
        self.assert_admin();
        assert_eq!(metadata_hash.len(), 32, "metadata hash must be 32 bytes");
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&metadata_hash);
        let record = AgentRecord {
            owner: owner.clone(),
            metadata_hash: hash,
            capability_mask,
            revoked: false,
            trust_score_bps: 5_000,
        };
        self.agents.insert(&owner, &record);
    }

    pub fn update_trust(&mut self, owner: AccountId, trust_score_bps: u16) {
        self.assert_admin();
        assert!(trust_score_bps <= 10_000, "trust score out of range");
        let mut record = self.agents.get(&owner).expect("unknown agent");
        record.trust_score_bps = trust_score_bps;
        self.agents.insert(&owner, &record);
    }

    pub fn revoke(&mut self, owner: AccountId) {
        self.assert_admin();
        let mut record = self.agents.get(&owner).expect("unknown agent");
        record.revoked = true;
        self.agents.insert(&owner, &record);
    }

    fn assert_admin(&self) {
        assert_eq!(env::predecessor_account_id(), self.admin, "not admin");
    }
}
```

Benchmarking:

- Registration gas/storage per agent.
- Trust update gas/storage per observation batch.
- Off-chain verification latency for fetching evidence by `metadata_hash`.
- Dispute workflow latency.

Safety:

- Never write private conversation content on chain.
- Store hashes and compact scores on chain; keep evidence in IronClaw storage.
- Treat on-chain state as public, immutable, and adversarially inspectable.


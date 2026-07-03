# Reputation And Contract Blueprints

Keep reputation local-first. Chain publishing is optional, separately flagged,
and never required for core runtime trust decisions.

## Off-Chain Ledger

Owner: trust/reputation module plus `src/db/`.

```rust
pub struct ReputationEvent {
    pub reputation_event_id: String,
    pub subject_id: String,
    pub domain: ReputationDomain,
    pub delta: f64,
    pub evidence_ref: String,
    pub timestamp_ms: i64,
}

pub enum ReputationDomain {
    Tool,
    Extension,
    Provider,
    Agent,
    Workflow,
    Memory,
    UserVisibleOutcome,
}
```

Rules:

- Persist evidence refs, not raw prompts or secrets.
- Bound score deltas and apply half-life decay.
- Run contract tests against PostgreSQL and libSQL.
- Gate selection changes behind `experimental.local_reputation`, default `off`.

## Optional NEAR Adapter

Use only after local reputation is useful and stable.

Flag: `experimental.near_reputation_bridge`, default `off`.

Adapter constraints:

- Publish hashes or attestations, not raw evidence.
- Treat chain write failure as non-fatal.
- Never block local runtime trust decisions on chain availability.
- Keep gas/storage estimates in rollout artifacts.

## Required Tests

- Failed tool outcome updates local score through the event path.
- Feature flag off leaves selection unchanged.
- Forged or missing evidence is rejected.
- PostgreSQL and libSQL produce the same score projection.
- Optional bridge failure degrades to local-only mode.

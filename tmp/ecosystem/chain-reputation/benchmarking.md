# Benchmarking and Validation Plan

[Back to overview](./README.md)

This file defines how to measure the design. It intentionally avoids presenting unmeasured gas, latency, throughput, security, or economic numbers as facts.

## Reporting Format

Every benchmark result should include:

- code revision and contract hash,
- environment: local simulation, NEAR localnet, testnet, or mainnet,
- dataset size and generator seed,
- hardware for off-chain runs,
- protocol parameters: alpha schedule, decay half-life, damping, thresholds,
- p50/p95/p99 latency where relevant,
- failure and retry behavior,
- raw output or artifact path.

## Reputation Convergence

Analytical baseline for fixed alpha:

```text
score_n = feedback - (feedback - score_0) * (1 - alpha)^n
```

Use it to validate the implementation, then run adaptive-alpha simulations.

Test matrix:

| Scenario | What to measure |
|----------|-----------------|
| new agent, consistent high feedback | jobs to reach policy thresholds |
| new agent, one early bad job | recovery speed |
| mature agent, isolated bad job | maximum score movement |
| inactive high-score agent | decay toward neutral |
| inactive low-score agent | recovery toward neutral |

Acceptance target: observed scores match the reference model within fixed-point tolerance, and policy thresholds behave as expected in caller-level marketplace tests.

## TraceRank

Measure on generated and replayed payment graphs:

| Graph | Purpose |
|-------|---------|
| empty graph | neutral/default behavior |
| star graph | influence concentration |
| chain graph | trust propagation depth |
| dense legitimate team | false collusion pressure |
| adversarial ring | detector interaction |
| sparse 10k-node graph | convergence and memory use |

Report:

- iterations to threshold,
- total runtime,
- memory usage,
- rank stability across seeds,
- sensitivity to damping and dust filters.

Target: convergence under the configured max iteration count for expected graph sizes. Do not publish a fixed "20-40 iterations" claim until measured on IronClaw-shaped data.

## Collusion Detection

Create labeled synthetic datasets instead of guessing true/false positive rates.

Variables:

- agent count,
- job count,
- ring size,
- mutual assignment ratio,
- legitimate repeated collaboration rate,
- lookback window,
- minimum assignments per pair.

Metrics:

- precision and recall by ring size,
- false positives on legitimate teams,
- runtime and memory,
- number of fragmented clique findings,
- manual-review queue size.

Initial acceptance target: detector output is useful as review evidence and feedback-weight dilution input. It should not be the only trigger for irreversible slashing.

## Marketplace and Payments

Measure complete caller paths:

| Path | Required checks |
|------|-----------------|
| post job | storage charged, job visible, escrow locked |
| assign | capability and reputation gates enforced |
| submit | only assigned agent can submit |
| accept | feedback recorded, escrow released once |
| reject | refund/slash policy applied exactly once |
| dispute | escrow locked, reputation delayed |
| expire | refund without reputation update |
| HTTP 402 retry | quote bound to request, nonce not replayable |

For NEAR, report localnet and testnet values separately:

- receipt count,
- gas burnt,
- attached deposit,
- storage delta,
- callback latency,
- failed-callback recovery behavior.

## Storage

Compute storage cost from measured bytes:

```text
cost = storage_delta_bytes * env::storage_byte_cost
```

Do not hardcode fiat estimates. If a cost table is useful, include the NEAR price date, source, and exchange rate as a separate assumption.

Records to measure:

- passport registration,
- prompt hash update,
- first reputation domain track,
- additional domain track,
- job post,
- result submission,
- dispute record,
- payment nonce.

## Security Validation

Minimum adversarial tests:

- transfer and approval of a passport always fail,
- one account cannot register two active passports,
- escrow cannot pay twice,
- callback failure cannot lose funds,
- stale payment nonce cannot be replayed,
- expired payment quote cannot be reused,
- direct hire counts toward collusion evidence,
- reputation cannot be updated by unauthorized callers,
- disputed jobs cannot update reputation before final resolution.

Financial contracts need independent review before real funds. "Audited" should name the reviewer, scope, date, and commit hash.

## Economic Validation

Demurrage, emissions, stake thresholds, and fees require simulation:

| Parameter | Validation question |
|-----------|---------------------|
| annual demurrage | does it improve useful circulation or only punish idle users? |
| emission schedule | does it fund useful work without runaway supply? |
| stake thresholds | what is the cost to create many high-tier identities? |
| marketplace fee | does it cover operations without pushing work off-platform? |
| dispute bond | is it high enough to deter spam and low enough for valid appeals? |

Use sensitivity analysis. Report ranges, not single confident numbers.

## Implementation Tiers

Planning estimates only:

| Tier | Slice | Risk | Validation gate |
|------|-------|------|-----------------|
| 1 | off-chain reputation for IronClaw tools/extensions | low-medium | local tests and replayed event data |
| 2 | NEAR passport identity bridge | medium | localnet/testnet contract tests |
| 3 | TraceRank-assisted delegation | medium | decision-quality evaluation |
| 4 | marketplace escrow and payments | high | contract review, adversarial tests, staged rollout |

Recommended path: build Tier 1 first. It validates the model without introducing funds, contracts, or token economics.

## Navigation

- [Passport System](./passport-system.md)
- [Reputation Scoring](./reputation-scoring.md)
- [Bounty Marketplace](./bounty-marketplace.md)
- [Token Economics](./token-economics.md)
- [NEAR Implementation](./near-implementation.md)
- [References](./references.md)

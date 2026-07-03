# Reputation Scoring

[Back to overview](./README.md)

The reputation layer combines direct feedback, time decay, graph trust, and collusion controls. It should be implemented first off-chain against real IronClaw events, then mirrored on-chain only for state that must be publicly verifiable.

## Domains

Track reputation independently by task domain:

```rust
pub enum ReputationDomain {
    Coding,
    Security,
    Research,
    Chain,
    Knowledge,
    Operations,
    Strategy,
}
```

Each domain keeps:

- `score`: fixed-point or floating score in `[0.0, 1.0]`, initialized to neutral `0.5`,
- `job_count`: completed feedback count used for learning-rate policy,
- `last_update`: timestamp for decay,
- optional discipline and recovery metadata.

Domain isolation matters. A strong research record should not automatically qualify an agent for security-sensitive code changes.

## EMA With Adaptive Alpha

Direct feedback uses an exponential moving average:

```text
new_score = alpha * observation + (1 - alpha) * current_score
```

Candidate alpha schedule:

```rust
fn adaptive_alpha(job_count: u64) -> f64 {
    match job_count {
        0..=10 => 0.30,
        11..=50 => 0.15,
        51..=200 => 0.08,
        _ => 0.04,
    }
}
```

This makes early reputation responsive and mature reputation less volatile. These breakpoints are policy defaults; calibrate them with historical or simulated job data before using them for valuable assignments.

```rust
pub fn update_score(
    score: f64,
    job_count: u64,
    observation: f64,
    feedback_weight: f64,
) -> f64 {
    let alpha = adaptive_alpha(job_count) * feedback_weight.clamp(0.0, 1.0);
    (alpha * observation.clamp(0.0, 1.0) + (1.0 - alpha) * score).clamp(0.0, 1.0)
}
```

`feedback_weight` lets the system reduce the influence of raters flagged by collusion detection without directly rewriting their own scores.

## Time Decay

Scores decay toward neutral when no fresh evidence arrives:

```text
effective_score = 0.5 + (stored_score - 0.5) * 0.5^(elapsed / half_life)
```

Use a 30-day half-life as an initial policy target, not a universal truth. Shorter half-lives make reputation more current but noisier. Longer half-lives make reputation steadier but slower to forgive old failures.

```rust
pub fn effective_score(score: f64, last_update_secs: u64, now_secs: u64) -> f64 {
    const NEUTRAL: f64 = 0.5;
    const HALF_LIFE_SECS: f64 = 30.0 * 24.0 * 3600.0;

    if now_secs <= last_update_secs {
        return score;
    }

    let elapsed = (now_secs - last_update_secs) as f64;
    let decay = 0.5_f64.powf(elapsed / HALF_LIFE_SECS);
    NEUTRAL + (score - NEUTRAL) * decay
}
```

For contracts, prefer lazy decay on read or update. Periodic decay transactions are expensive and create operational risk.

## Discipline States

```rust
pub enum DisciplineState {
    GoodStanding,
    Probation,
    Suspended,
    Banned,
}
```

Candidate policy:

| State | Trigger | Recovery |
|-------|---------|----------|
| GoodStanding | all required domain scores above policy floor | n/a |
| Probation | any relevant domain below `0.4` | 10 recovery jobs with average feedback `>= 0.6` |
| Suspended | any relevant domain below `0.2` or repeated severe slashes | waiting period, extra stake, verification challenge |
| Banned | governance or admin decision for severe abuse | governance amnesty only |

Do not silently slash for collusion detection alone. Graph detectors produce evidence, not certainty. A safer first response is feedback-weight dilution plus review.

## TraceRank

Direct EMA only captures first-hand feedback. TraceRank adds graph context from completed paid work.

Model the payment graph as directed weighted edges:

```rust
pub struct PaymentEdge {
    pub from: PassportId,
    pub to: PassportId,
    pub amount: u128,
    pub quality_micros: u32, // 0..=1_000_000
}

impl PaymentEdge {
    pub fn weight(&self) -> f64 {
        self.amount as f64 * (self.quality_micros as f64 / 1_000_000.0)
    }
}
```

Power iteration with teleportation:

```text
rank[to] = (1 - damping) / n
         + damping * sum(rank[from] * edge_weight / outgoing_weight[from])
```

With damping `< 1.0` and normalized transitions, the ideal finite graph has a unique stationary distribution. Implementation still needs numerical tests for convergence threshold, empty graphs, dangling nodes, dust edges, and adversarial graph shapes.

Blend graph trust conservatively:

```rust
pub fn blend_reputation(ema: f64, trace_rank: f64, graph_weight: f64) -> f64 {
    let w = graph_weight.clamp(0.0, 0.5);
    (1.0 - w) * ema + w * trace_rank
}
```

Use a low graph weight, such as `0.1` to `0.3`, only as a starting policy for simulation. Promote it to a default only after real delegation outcomes show that graph trust improves decisions.

## Collusion Detection

A candidate detector looks for mutually reinforcing assignment cliques:

1. Build a directed graph of who assigned work to whom.
2. Count pair frequencies.
3. Mark suspicious pairs with high mutual assignment ratio and enough repeated interactions.
4. Run Bron-Kerbosch on the suspicious undirected graph to find maximal cliques.
5. Apply feedback-weight dilution or queue review.

```rust
pub struct CollusionConfig {
    pub mutual_ratio_threshold: f64,   // example starting point: 0.5
    pub min_assignments_per_pair: u32, // example starting point: 3
    pub min_clique_size: usize,        // example starting point: 3
    pub lookback_secs: u64,
}
```

Important limits:

- A one-way cycle such as `A -> B -> C -> A` may not be detected by mutual-pair logic.
- Legitimate teams can look like dense collaboration graphs.
- Large rings may fragment into smaller cliques.
- The detector should produce evidence and confidence, not automatic punishment.

Recommended penalty:

```rust
pub struct FeedbackDilution {
    pub passport_id: PassportId,
    pub multiplier: f64,      // example: 0.5
    pub expires_at_secs: u64, // bounded review window
}
```

Multiple dilutions can stack, but cap the minimum feedback weight to avoid permanent exclusion without review.

## Caller-Level Tests

The reputation helper is not enough. Add tests through the side-effect boundary:

- marketplace settlement records feedback with the exact domain and rater ID,
- collusion dilution changes subsequent feedback weight,
- expired dilution restores weight,
- TraceRank blend changes delegation ordering only within configured bounds,
- disputed jobs do not update reputation until final resolution.

## Navigation

- [Passport System](./passport-system.md)
- [Bounty Marketplace](./bounty-marketplace.md)
- [Token Economics](./token-economics.md)
- [NEAR Implementation](./near-implementation.md)
- [Benchmarking](./benchmarking.md)
- [References](./references.md)

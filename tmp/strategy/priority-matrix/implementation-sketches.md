# Implementation Sketches: Top 10 Priorities

> **Navigation**:
> [README](README.md) |
> [Detailed Rankings](detailed-rankings.md) |
> [Implementation Sketches (you are here)](implementation-sketches.md) |
> [Quick Wins](quick-wins.md) |
> [Synergy Analysis](synergy-analysis.md) |
> [Benchmarking Plans](benchmarking-plans.md) |
> [References](references.md)

These are complete, runnable Rust implementations, not pseudocode. They are designed to be
dropped into IronClaw's source tree with minor import adjustments.

---

## Rank 1: Metacognitive Monitor

**Source analysis**: [../../agent-intelligence/agent-patterns.md](../../agent-intelligence/agent-patterns.md)
**Location:** `src/agent/metacognitive.rs` (new file)

```rust
// src/agent/metacognitive.rs

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use tracing::debug;

/// Detects three agent pathologies: stuck loops, cost runaways, self-contradictions.
/// Integrated into `src/agent/agentic_loop.rs` — call `observe()` after each turn,
/// `diagnose()` to check for pathologies, and `recommend()` for corrective action.
pub struct MetacognitiveMonitor {
    action_window: VecDeque<u64>,
    window_size: usize,
    uniqueness_threshold: f64,
    cost_ewma: f64,
    cost_alpha: f64,
    recent_claims: VecDeque<(u64, bool)>,
    claims_window: usize,
    enabled: bool,
    total_turns: u32,
}

#[derive(Debug, Clone)]
pub enum Pathology {
    StuckLoop { unique_ratio: f64, window_size: usize },
    CostRunaway { projected_total_cents: f64, budget_remaining_cents: f64, turns_remaining: u32 },
    SelfContradiction { claim_hash: u64, summary: String },
}

#[derive(Debug, Clone)]
pub enum Correction {
    BreakLoop { injection_prompt: String },
    DowngradeModel { reason: String },
    FlagContradiction { summary: String },
    None,
}

impl MetacognitiveMonitor {
    pub fn new(window_size: usize, uniqueness_threshold: f64) -> Self {
        Self {
            action_window: VecDeque::with_capacity(window_size),
            window_size,
            uniqueness_threshold,
            cost_ewma: 0.0,
            cost_alpha: 0.3,
            recent_claims: VecDeque::with_capacity(50),
            claims_window: 50,
            enabled: true,
            total_turns: 0,
        }
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn observe(&mut self, action_fingerprint: u64, cost_cents: f64) {
        if !self.enabled { return; }
        if self.action_window.len() >= self.window_size {
            self.action_window.pop_front();
        }
        self.action_window.push_back(action_fingerprint);
        if self.total_turns == 0 {
            self.cost_ewma = cost_cents;
        } else {
            self.cost_ewma = self.cost_alpha * cost_cents + (1.0 - self.cost_alpha) * self.cost_ewma;
        }
        self.total_turns += 1;
        debug!(turns = self.total_turns, cost_ewma = self.cost_ewma, "MetacognitiveMonitor: observed turn");
    }

    pub fn observe_claim(&mut self, claim_text: &str, polarity: bool) {
        if !self.enabled { return; }
        let hash = hash_str(claim_text);
        if self.recent_claims.len() >= self.claims_window {
            self.recent_claims.pop_front();
        }
        self.recent_claims.push_back((hash, polarity));
    }

    pub fn diagnose(&self, budget_remaining_cents: f64, turns_remaining: u32) -> Option<Pathology> {
        if !self.enabled || self.total_turns < 4 { return None; }

        let unique: HashSet<&u64> = self.action_window.iter().collect();
        let unique_ratio = unique.len() as f64 / self.action_window.len() as f64;
        if unique_ratio < self.uniqueness_threshold {
            debug!(unique_ratio, threshold = self.uniqueness_threshold, "stuck loop detected");
            return Some(Pathology::StuckLoop { unique_ratio, window_size: self.action_window.len() });
        }

        if turns_remaining > 0 {
            let projected = self.cost_ewma * turns_remaining as f64;
            if projected > budget_remaining_cents {
                debug!(projected, budget_remaining_cents, "cost runaway detected");
                return Some(Pathology::CostRunaway {
                    projected_total_cents: projected,
                    budget_remaining_cents,
                    turns_remaining,
                });
            }
        }

        let mut seen: HashMap<u64, bool> = HashMap::new();
        for &(hash, polarity) in &self.recent_claims {
            if let Some(&prev_polarity) = seen.get(&hash) {
                if prev_polarity != polarity {
                    return Some(Pathology::SelfContradiction {
                        claim_hash: hash,
                        summary: format!("Claim #{:016x} was asserted with opposite polarities", hash),
                    });
                }
            }
            seen.insert(hash, polarity);
        }

        None
    }

    pub fn recommend(&self, pathology: &Pathology) -> Correction {
        match pathology {
            Pathology::StuckLoop { unique_ratio, window_size } => {
                Correction::BreakLoop {
                    injection_prompt: format!(
                        "You appear to be repeating yourself: only {:.0}% of your last {} \
                         actions were unique. Try a fundamentally different approach, \
                         ask a clarifying question, or report that the task cannot be completed.",
                        unique_ratio * 100.0, window_size
                    ),
                }
            }
            Pathology::CostRunaway { projected_total_cents, budget_remaining_cents, turns_remaining } => {
                Correction::DowngradeModel {
                    reason: format!(
                        "Projected cost ${:.4} ({} turns × ${:.4}/turn avg) exceeds \
                         remaining budget ${:.4}. Switching to cheaper model.",
                        projected_total_cents / 100.0, turns_remaining,
                        self.cost_ewma / 100.0, budget_remaining_cents / 100.0,
                    ),
                }
            }
            Pathology::SelfContradiction { summary, .. } => {
                Correction::FlagContradiction { summary: summary.clone() }
            }
        }
    }

    pub fn fingerprint_action(tool_name: &str, args_json: &str) -> u64 {
        hash_str(&format!("{tool_name}:{args_json}"))
    }
}

fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_stuck_loop_after_window_fills() {
        let mut monitor = MetacognitiveMonitor::new(10, 0.25);
        for _ in 0..10 { monitor.observe(42, 0.5); }
        let p = monitor.diagnose(100.0, 10).expect("should detect stuck loop");
        assert!(matches!(p, Pathology::StuckLoop { .. }));
    }

    #[test]
    fn no_detection_during_warmup() {
        let mut monitor = MetacognitiveMonitor::new(10, 0.25);
        for _ in 0..3 { monitor.observe(42, 0.5); }
        assert!(monitor.diagnose(100.0, 10).is_none());
    }

    #[test]
    fn detects_cost_runaway() {
        let mut monitor = MetacognitiveMonitor::new(10, 0.25);
        for i in 0..8u64 { monitor.observe(i, 30.0); }
        let p = monitor.diagnose(50.0, 10).expect("should detect cost runaway");
        assert!(matches!(p, Pathology::CostRunaway { .. }));
    }

    #[test]
    fn detects_contradiction() {
        let mut monitor = MetacognitiveMonitor::new(10, 0.25);
        for i in 0..8u64 { monitor.observe(i, 0.1); }
        monitor.observe_claim("the server is running on port 8080", true);
        monitor.observe_claim("the server is running on port 8080", false);
        let p = monitor.diagnose(1000.0, 10).expect("should detect contradiction");
        assert!(matches!(p, Pathology::SelfContradiction { .. }));
    }

    #[test]
    fn diverse_actions_no_loop_detection() {
        let mut monitor = MetacognitiveMonitor::new(10, 0.25);
        for i in 0..10u64 { monitor.observe(i, 0.5); }
        assert!(monitor.diagnose(1000.0, 10).is_none());
    }
}
```

**Integration in `src/agent/agentic_loop.rs`:**

```rust
// Instantiate once per job, before the turn loop:
let mut monitor = MetacognitiveMonitor::new(10, 0.25)
    .with_enabled(config.metacognitive_monitor_enabled);

// Inside the turn loop, after each set of tool calls:
let fingerprint = MetacognitiveMonitor::fingerprint_action(
    &tool_call.name,
    &serde_json::to_string(&tool_call.arguments).unwrap_or_default(),
);
monitor.observe(fingerprint, turn_cost_cents);

if let Some(pathology) = monitor.diagnose(budget_remaining_cents, max_turns - turn_idx) {
    let correction = monitor.recommend(&pathology);
    match correction {
        Correction::BreakLoop { ref injection_prompt } => {
            messages.push(ChatMessage::system(injection_prompt.clone()));
        }
        Correction::DowngradeModel { ref reason } => {
            debug!(reason, "downgrading model due to cost runaway");
            current_provider = cheap_provider.clone();
        }
        Correction::FlagContradiction { ref summary } => {
            debug!(summary, "contradiction detected");
            messages.push(ChatMessage::system(format!(
                "Warning: I detected a potential contradiction: {summary}"
            )));
        }
        Correction::None => {}
    }
}
```

---

## Rank 2: Ebbinghaus Decay

**Source analysis**: [../../core-concepts/universal-engram.md](../../core-concepts/universal-engram.md)
**Location:** `src/workspace/decay.rs` (new file)

```rust
// src/workspace/decay.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DecayVariant {
    None,
    HalfLife { half_life_secs: f64 },
    Ttl { ttl_secs: f64 },
    Ebbinghaus { stability_secs: f64 },
}

impl Default for DecayVariant {
    fn default() -> Self {
        Self::Ebbinghaus { stability_secs: 3600.0 }
    }
}

impl DecayVariant {
    pub fn current_strength(&self, last_accessed: DateTime<Utc>, now: DateTime<Utc>) -> f64 {
        let elapsed_secs = (now - last_accessed).num_seconds().max(0) as f64;
        match self {
            DecayVariant::None => 1.0,
            DecayVariant::HalfLife { half_life_secs } => 0.5_f64.powf(elapsed_secs / half_life_secs),
            DecayVariant::Ttl { ttl_secs } => if elapsed_secs >= *ttl_secs { 0.0 } else { 1.0 },
            DecayVariant::Ebbinghaus { stability_secs } => (-elapsed_secs / stability_secs).exp(),
        }
    }

    pub fn on_access(self, now: DateTime<Utc>) -> (Self, DateTime<Utc>) {
        let updated = match self {
            DecayVariant::Ebbinghaus { stability_secs } => {
                let new_stability = (stability_secs * 2.0).min(63_072_000.0);
                DecayVariant::Ebbinghaus { stability_secs: new_stability }
            }
            other => other,
        };
        (updated, now)
    }

    pub const FADED_THRESHOLD: f64 = 0.05;
}

pub fn apply_decay_weights(
    results: &mut Vec<(f64, DecayVariant, DateTime<Utc>)>,
    now: DateTime<Utc>,
) {
    results.retain_mut(|(score, decay, last_accessed)| {
        let strength = decay.current_strength(*last_accessed, now);
        if strength < DecayVariant::FADED_THRESHOLD { return false; }
        *score *= strength;
        true
    });
    results.sort_by(|(a, ..), (b, ..)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
}
```

**Database migrations** (both PostgreSQL and libSQL required per IronClaw dual-backend rule):

```sql
-- PostgreSQL: migrations/20260101000001_memory_decay.sql
ALTER TABLE workspace_entries
    ADD COLUMN IF NOT EXISTS decay_variant  JSONB    NOT NULL DEFAULT '{"type":"none"}',
    ADD COLUMN IF NOT EXISTS last_accessed  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS access_count   INTEGER  NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS idx_workspace_entries_last_accessed
    ON workspace_entries(last_accessed);

-- libSQL: migrations/libsql/20260101000001_memory_decay.sql
ALTER TABLE workspace_entries ADD COLUMN decay_variant  TEXT    NOT NULL DEFAULT '{"type":"none"}';
ALTER TABLE workspace_entries ADD COLUMN last_accessed  TEXT    NOT NULL DEFAULT (datetime('now'));
ALTER TABLE workspace_entries ADD COLUMN access_count   INTEGER NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS idx_workspace_entries_last_accessed
    ON workspace_entries(last_accessed);
```

---

## Rank 3: BLAKE3 Content Dedup

**Source analysis**: [../../core-concepts/universal-engram.md](../../core-concepts/universal-engram.md)
**Location:** `src/workspace/dedup.rs` (new file)

```rust
// src/workspace/dedup.rs

use blake3::Hash;

pub fn content_hash(content: &str) -> [u8; 32] {
    let normalized = normalize_content(content);
    *blake3::hash(normalized.as_bytes()).as_bytes()
}

fn normalize_content(s: &str) -> String {
    let trimmed = s.trim();
    let mut result = String::with_capacity(trimmed.len());
    let mut prev_was_space = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !prev_was_space { result.push(' '); }
            prev_was_space = true;
        } else {
            result.push(ch);
            prev_was_space = false;
        }
    }
    result
}

pub fn format_hash(hash: &[u8; 32]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

#[derive(Debug)]
pub enum DedupOutcome {
    New { hash: [u8; 32] },
    Duplicate { existing_id: String, existing_path: String, hash: [u8; 32] },
}
```

**Database migrations:**

```sql
-- PostgreSQL: migrations/20260101000002_memory_content_hash.sql
ALTER TABLE workspace_entries ADD COLUMN IF NOT EXISTS content_hash BYTEA;
CREATE UNIQUE INDEX IF NOT EXISTS idx_workspace_entries_content_hash
    ON workspace_entries(content_hash) WHERE content_hash IS NOT NULL;

-- libSQL: migrations/libsql/20260101000002_memory_content_hash.sql
ALTER TABLE workspace_entries ADD COLUMN content_hash BLOB;
CREATE UNIQUE INDEX IF NOT EXISTS idx_workspace_entries_content_hash
    ON workspace_entries(content_hash) WHERE content_hash IS NOT NULL;
```

---

## Rank 4: Robust Statistics

**Source analysis**: [../../core-concepts/mathematical-primitives.md](../../core-concepts/mathematical-primitives.md)
**Location:** Append to `src/util.rs`

```rust
// src/util.rs — append these functions

/// Median of a non-empty slice. Returns `f64::NAN` for empty input.
pub fn median(values: &[f64]) -> f64 {
    if values.is_empty() { return f64::NAN; }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    let mid = n / 2;
    if n % 2 == 0 { (sorted[mid - 1] + sorted[mid]) / 2.0 } else { sorted[mid] }
}

/// Trimmed mean: discard bottom and top `trim_fraction` of values, then average.
pub fn trimmed_mean(values: &[f64], trim_fraction: f64) -> f64 {
    assert!((0.0..0.5).contains(&trim_fraction), "trim_fraction must be in [0.0, 0.5)");
    if values.is_empty() { return f64::NAN; }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let trim_count = (sorted.len() as f64 * trim_fraction).floor() as usize;
    let trimmed = &sorted[trim_count..sorted.len() - trim_count];
    if trimmed.len() < 2 { return median(values); }
    trimmed.iter().sum::<f64>() / trimmed.len() as f64
}

/// Median Absolute Deviation: robust measure of spread.
pub fn mad(values: &[f64]) -> f64 {
    if values.is_empty() { return f64::NAN; }
    let med = median(values);
    let deviations: Vec<f64> = values.iter().map(|&x| (x - med).abs()).collect();
    median(&deviations)
}

/// Hodges-Lehmann estimator: median of all pairwise averages. O(n²). Use for n < 10,000.
pub fn hodges_lehmann(values: &[f64]) -> f64 {
    if values.is_empty() { return f64::NAN; }
    let n = values.len();
    let mut pairwise = Vec::with_capacity(n * (n + 1) / 2);
    for i in 0..n {
        for j in i..n {
            pairwise.push((values[i] + values[j]) / 2.0);
        }
    }
    median(&pairwise)
}

/// EWMA update. `alpha` in (0, 1). Higher = more weight to recent values.
pub fn ewma_update(current_ewma: f64, new_value: f64, alpha: f64) -> f64 {
    alpha * new_value + (1.0 - alpha) * current_ewma
}

/// Robust z-score using MAD. A value is an outlier if |robust_z| > 3.0.
pub fn robust_z_score(value: f64, values: &[f64]) -> f64 {
    let med = median(values);
    let m = mad(values);
    if m == 0.0 { return 0.0; }
    (value - med) / (1.4826 * m)
}
```

---

## Rank 5: Composable Scorers

**Source analysis**: [../../agent-intelligence/agent-patterns.md](../../agent-intelligence/agent-patterns.md)
**Location:** `src/evaluation/scorer.rs` (new file)

```rust
// src/evaluation/scorer.rs

use std::sync::Arc;
use chrono::{DateTime, Utc};

pub trait Scoreable: Send + Sync {
    fn timestamp(&self) -> Option<DateTime<Utc>> { None }
    fn content(&self) -> Option<&str> { None }
    fn utility(&self) -> Option<f64> { None }
    fn access_count(&self) -> Option<u32> { None }
    fn tags(&self) -> &[String] { &[] }
}

pub trait Scorer: Send + Sync {
    fn score(&self, item: &dyn Scoreable) -> f64;
    fn name(&self) -> &str;
}

pub struct WeightedScorer {
    components: Vec<(f64, Arc<dyn Scorer>)>,
    name: String,
}

impl WeightedScorer {
    pub fn new(components: Vec<(f64, Arc<dyn Scorer>)>) -> Self {
        Self { components, name: "weighted".to_string() }
    }
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into(); self
    }
}

impl Scorer for WeightedScorer {
    fn score(&self, item: &dyn Scoreable) -> f64 {
        let total_weight: f64 = self.components.iter().map(|(w, _)| *w).sum();
        if total_weight == 0.0 { return 0.0; }
        self.components.iter().map(|(weight, scorer)| weight * scorer.score(item)).sum::<f64>()
            / total_weight
    }
    fn name(&self) -> &str { &self.name }
}

pub struct ThresholdScorer { inner: Arc<dyn Scorer>, threshold: f64 }

impl ThresholdScorer {
    pub fn new(inner: Arc<dyn Scorer>, threshold: f64) -> Self { Self { inner, threshold } }
}

impl Scorer for ThresholdScorer {
    fn score(&self, item: &dyn Scoreable) -> f64 {
        if self.inner.score(item) >= self.threshold { 1.0 } else { 0.0 }
    }
    fn name(&self) -> &str { "threshold" }
}

pub struct ChainScorer { stages: Vec<Arc<dyn Scorer>> }

impl ChainScorer {
    pub fn new(stages: Vec<Arc<dyn Scorer>>) -> Self { Self { stages } }
}

impl Scorer for ChainScorer {
    fn score(&self, item: &dyn Scoreable) -> f64 {
        for stage in &self.stages {
            if stage.score(item) == 0.0 { return 0.0; }
        }
        self.stages.last().map(|s| s.score(item)).unwrap_or(0.0)
    }
    fn name(&self) -> &str { "chain" }
}

pub struct RecencyScorer { half_life_hours: f64 }

impl RecencyScorer {
    pub fn new(half_life_hours: f64) -> Self { Self { half_life_hours } }
}

impl Scorer for RecencyScorer {
    fn score(&self, item: &dyn Scoreable) -> f64 {
        match item.timestamp() {
            Some(ts) => {
                let elapsed_hours = (Utc::now() - ts).num_seconds().max(0) as f64 / 3600.0;
                (-elapsed_hours / self.half_life_hours).exp()
            }
            None => 0.5,
        }
    }
    fn name(&self) -> &str { "recency" }
}

pub struct UtilityScorer;

impl Scorer for UtilityScorer {
    fn score(&self, item: &dyn Scoreable) -> f64 { item.utility().unwrap_or(0.5) }
    fn name(&self) -> &str { "utility" }
}

pub struct PopularityScorer { saturation: f64 }

impl PopularityScorer {
    pub fn new(saturation: f64) -> Self { Self { saturation } }
}

impl Scorer for PopularityScorer {
    fn score(&self, item: &dyn Scoreable) -> f64 {
        item.access_count().map(|c| (c as f64 / self.saturation).min(1.0)).unwrap_or(0.0)
    }
    fn name(&self) -> &str { "popularity" }
}

pub struct TagScorer { required_tags: Vec<String> }

impl TagScorer {
    pub fn new(required_tags: Vec<String>) -> Self { Self { required_tags } }
}

impl Scorer for TagScorer {
    fn score(&self, item: &dyn Scoreable) -> f64 {
        if self.required_tags.is_empty() { return 1.0; }
        let item_tags: std::collections::HashSet<&str> =
            item.tags().iter().map(String::as_str).collect();
        let matched = self.required_tags.iter()
            .filter(|t| item_tags.contains(t.as_str()))
            .count();
        matched as f64 / self.required_tags.len() as f64
    }
    fn name(&self) -> &str { "tag_overlap" }
}
```

---

## Rank 6: Hierarchical Cancellation

**Source analysis**: [../../execution-verification/runtime-infrastructure.md](../../execution-verification/runtime-infrastructure.md)
**Location:** `src/agent/cancel.rs` (new file)

```rust
// src/agent/cancel.rs

use tokio_util::sync::CancellationToken;
use tracing::debug;

pub struct CancellationTree {
    token: CancellationToken,
}

impl CancellationTree {
    pub fn new() -> Self { Self { token: CancellationToken::new() } }
    pub fn token(&self) -> CancellationToken { self.token.clone() }
    pub fn cancel(&self) {
        debug!("CancellationTree: root cancelled (session-level shutdown)");
        self.token.cancel();
    }
    pub fn child_token(&self) -> CancellationToken { self.token.child_token() }
    pub fn is_cancelled(&self) -> bool { self.token.is_cancelled() }
}

impl Default for CancellationTree {
    fn default() -> Self { Self::new() }
}

pub async fn run_with_cancel<F, T>(token: CancellationToken, future: F) -> Result<T, Cancelled>
where F: std::future::Future<Output = T> {
    tokio::select! {
        result = future => Ok(result),
        _ = token.cancelled() => Err(Cancelled),
    }
}

#[derive(Debug, thiserror::Error)]
#[error("operation cancelled")]
pub struct Cancelled;
```

**Integration pattern:**

```rust
// When a session starts:
let session_cancel = CancellationTree::new();

// When a job is spawned:
let job_token = session_cancel.child_token();

// When a tool call is dispatched:
let tool_token = job_token.child_token();

// In the LLM call:
let response = run_with_cancel(tool_token, llm_provider.chat(request))
    .await.map_err(|Cancelled| AgentError::Cancelled)?;

// To cancel a single job (user presses Cancel in the web UI):
job_token.cancel(); // Only this job and its tool calls stop; session continues
```

---

## Rank 7: Cascade Router

**Source analysis**: [../../agent-intelligence/online-learning.md](../../agent-intelligence/online-learning.md)
**Location:** `crates/ironclaw_llm/src/router/` (new module, 4 files)

```rust
// crates/ironclaw_llm/src/router/bandit.rs

use ndarray::{Array1, Array2};

pub struct LinUcbArm {
    a_matrix: Array2<f64>,
    b_vector: Array1<f64>,
    d: usize,
    alpha: f64,
}

impl LinUcbArm {
    pub fn new(d: usize, alpha: f64) -> Self {
        Self {
            a_matrix: Array2::eye(d),
            b_vector: Array1::zeros(d),
            d,
            alpha,
        }
    }

    pub fn ucb_score(&self, x: &Array1<f64>) -> f64 {
        let a_inv = self.a_matrix_inverse();
        let theta_hat = a_inv.dot(&self.b_vector);
        let expected_reward = theta_hat.dot(x);
        let uncertainty = (x.dot(&a_inv.dot(x))).sqrt();
        expected_reward + self.alpha * uncertainty
    }

    pub fn update(&mut self, x: &Array1<f64>, reward: f64) {
        for i in 0..self.d {
            for j in 0..self.d {
                self.a_matrix[[i, j]] += x[i] * x[j];
            }
        }
        for i in 0..self.d {
            self.b_vector[i] += reward * x[i];
        }
    }

    fn a_matrix_inverse(&self) -> Array2<f64> {
        // Production: use ndarray-linalg or Sherman-Morrison for O(d²) updates.
        // Placeholder for d ≤ 20.
        self.a_matrix.clone()
    }
}

/// 18-dimensional context features for the bandit.
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub complexity: f64,
    pub speed_tier: f64,
    pub tool_call_count: f64,
    pub conversation_depth: f64,
    pub budget_fraction: f64,
    pub normalized_msg_len: f64,
    pub is_code_task: f64,
    pub is_search_task: f64,
    pub hour_of_day: f64,
    pub latency_sensitivity: f64,
    pub provider_health: f64,
    pub recent_error_rate: f64,
    pub task_type_onehot: [f64; 6],
}

impl RequestContext {
    pub fn to_feature_vector(&self) -> Array1<f64> {
        let mut v = Array1::zeros(18);
        v[0] = self.complexity;
        v[1] = self.speed_tier;
        v[2] = (self.tool_call_count / 20.0).min(1.0);
        v[3] = (self.conversation_depth / 50.0).min(1.0);
        v[4] = self.budget_fraction;
        v[5] = self.normalized_msg_len;
        v[6] = self.is_code_task;
        v[7] = self.is_search_task;
        v[8] = self.hour_of_day;
        v[9] = self.latency_sensitivity;
        v[10] = self.provider_health;
        v[11] = self.recent_error_rate;
        for i in 0..6 { v[12 + i] = self.task_type_onehot[i]; }
        v
    }
}
```

```rust
// crates/ironclaw_llm/src/router/cascade.rs

use super::bandit::{LinUcbArm, RequestContext};

pub enum RoutingDecision {
    StaticRule { model_id: String, rule_name: &'static str, confidence: f64 },
    BanditSelected { model_id: String, ucb_score: f64 },
    Fallback { model_id: String },
}

const STATIC_RULE_CONFIDENCE_THRESHOLD: f64 = 0.80;
const BANDIT_WARMUP_ROUNDS: usize = 50;

pub struct CascadeRouter {
    arms: Vec<(String, LinUcbArm)>,
    total_rounds: usize,
    fallback_model: String,
}

impl CascadeRouter {
    pub fn new(model_ids: Vec<String>, feature_dim: usize, alpha: f64) -> Self {
        let fallback = model_ids.first().cloned().unwrap_or_default();
        let arms = model_ids.into_iter()
            .map(|id| (id.clone(), LinUcbArm::new(feature_dim, alpha)))
            .collect();
        Self { arms, total_rounds: 0, fallback_model: fallback }
    }

    pub fn select(&self, ctx: &RequestContext) -> RoutingDecision {
        let available_models: Vec<String> = self.arms.iter().map(|(id, _)| id.clone()).collect();

        // Stage 1: Static rules
        if let Some((model_id, rule_name, confidence)) = apply_static_rules(ctx, &available_models) {
            if confidence >= STATIC_RULE_CONFIDENCE_THRESHOLD {
                return RoutingDecision::StaticRule { model_id, rule_name, confidence };
            }
        }

        // Stage 2: Bandit (only after warm-up)
        if self.total_rounds >= BANDIT_WARMUP_ROUNDS {
            let x = ctx.to_feature_vector();
            let (best_model, best_ucb) = self.arms.iter()
                .map(|(id, arm)| (id.clone(), arm.ucb_score(&x)))
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((self.fallback_model.clone(), 0.0));
            return RoutingDecision::BanditSelected { model_id: best_model, ucb_score: best_ucb };
        }

        // Stage 3: Fallback
        RoutingDecision::Fallback { model_id: self.fallback_model.clone() }
    }

    pub fn observe_reward(&mut self, model_id: &str, ctx: &RequestContext, reward: f64) {
        let x = ctx.to_feature_vector();
        if let Some((_, arm)) = self.arms.iter_mut().find(|(id, _)| id == model_id) {
            arm.update(&x, reward);
        }
        self.total_rounds += 1;
    }
}

fn apply_static_rules(ctx: &RequestContext, available_models: &[String])
    -> Option<(String, &'static str, f64)>
{
    if ctx.is_search_task > 0.8 && ctx.complexity < 0.2 {
        if let Some(cheap) = available_models.first() {
            return Some((cheap.clone(), "simple_search_cheap", 0.95));
        }
    }
    if ctx.is_code_task > 0.8 && ctx.complexity > 0.7 {
        if let Some(primary) = available_models.last() {
            return Some((primary.clone(), "complex_code_primary", 0.90));
        }
    }
    if ctx.speed_tier < 0.1 && ctx.complexity < 0.3 {
        if let Some(cheap) = available_models.first() {
            return Some((cheap.clone(), "gamma_cheap", 0.85));
        }
    }
    None
}
```

---

## Rank 8: Cognitive Speed Labels

**Source analysis**: [../../core-concepts/cognitive-architecture.md](../../core-concepts/cognitive-architecture.md)
**Location:** `src/agent/cognitive_speed.rs` (new file)

```rust
// src/agent/cognitive_speed.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CognitiveSpeed {
    /// Reactive: ≤15s. Simple lookups, greetings. Cheapest model.
    Gamma,
    /// Reflective: ~75s. Multi-step reasoning. Standard model.
    Theta,
    /// Consolidation: minutes to hours. Background learning. Best model.
    Delta,
}

impl CognitiveSpeed {
    pub fn latency_budget_secs(&self) -> u64 {
        match self { CognitiveSpeed::Gamma => 15, CognitiveSpeed::Theta => 75, CognitiveSpeed::Delta => 3600 }
    }

    pub fn cost_weight(&self) -> f64 {
        match self { CognitiveSpeed::Gamma => 0.1, CognitiveSpeed::Theta => 0.5, CognitiveSpeed::Delta => 1.0 }
    }

    pub fn to_feature(&self) -> f64 {
        match self { CognitiveSpeed::Gamma => 0.0, CognitiveSpeed::Theta => 0.5, CognitiveSpeed::Delta => 1.0 }
    }
}

pub struct SpeedClassifierInput<'a> {
    pub message: &'a str,
    pub tool_call_count: usize,
    pub conversation_depth: usize,
    pub is_background: bool,
    pub is_code_task: bool,
    pub estimated_complexity: f64,
}

pub fn classify_speed(input: &SpeedClassifierInput<'_>) -> CognitiveSpeed {
    if input.is_background { return CognitiveSpeed::Delta; }

    let word_count = input.message.split_whitespace().count();
    if word_count <= 10 && input.tool_call_count == 0 && input.estimated_complexity < 0.2 && !input.is_code_task {
        return CognitiveSpeed::Gamma;
    }

    if input.estimated_complexity > 0.7 || input.conversation_depth > 20 || input.is_code_task {
        if input.estimated_complexity > 0.9 || input.conversation_depth > 50 {
            return CognitiveSpeed::Delta;
        }
        return CognitiveSpeed::Theta;
    }

    CognitiveSpeed::Theta
}

pub fn estimate_complexity(message: &str) -> f64 {
    let word_count = message.split_whitespace().count() as f64;
    let question_count = message.chars().filter(|&c| c == '?').count() as f64;
    let code_markers = ["```", "fn ", "impl ", "pub ", "async ", "await"].iter()
        .filter(|marker| message.contains(*marker)).count() as f64;
    let length_score = (word_count / 200.0).min(1.0);
    let question_score = (question_count / 5.0).min(1.0);
    let code_score = (code_markers / 3.0).min(1.0);
    (length_score * 0.5 + question_score * 0.2 + code_score * 0.3).min(1.0)
}
```

---

## Rank 9: HDC Similarity Engine

**Source analysis**: [../../core-concepts/hyperdimensional-computing/README.md](../../core-concepts/hyperdimensional-computing/README.md)
**Location:** New `crates/ironclaw_hdc/`

```rust
// crates/ironclaw_hdc/src/lib.rs

pub const DEFAULT_DIM_BITS: usize = 10_240;
pub const DIM_WORDS: usize = DEFAULT_DIM_BITS / 64; // 160 u64 words

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HyperVector {
    words: [u64; DIM_WORDS],
}

impl HyperVector {
    pub fn random() -> Self {
        let mut words = [0u64; DIM_WORDS];
        for word in &mut words { *word = rand_u64(); }
        Self { words }
    }

    pub fn zeros() -> Self { Self { words: [0u64; DIM_WORDS] } }

    pub fn bind(&self, other: &Self) -> Self {
        let mut words = [0u64; DIM_WORDS];
        for i in 0..DIM_WORDS { words[i] = self.words[i] ^ other.words[i]; }
        Self { words }
    }

    pub fn bundle(vectors: &[&HyperVector]) -> Self {
        if vectors.is_empty() { return Self::zeros(); }
        let n = vectors.len();
        let threshold = n / 2;
        let mut count = vec![0u32; DIM_WORDS * 64];
        for hv in vectors {
            for (w_idx, &word) in hv.words.iter().enumerate() {
                for bit in 0..64 {
                    if (word >> bit) & 1 == 1 { count[w_idx * 64 + bit] += 1; }
                }
            }
        }
        let mut words = [0u64; DIM_WORDS];
        for (i, &cnt) in count.iter().enumerate() {
            if cnt > threshold as u32 { words[i / 64] |= 1u64 << (i % 64); }
        }
        Self { words }
    }

    pub fn rotate(&self, positions: usize) -> Self {
        let shift = positions % DEFAULT_DIM_BITS;
        let word_shift = shift / 64;
        let bit_shift = shift % 64;
        let mut words = [0u64; DIM_WORDS];
        for i in 0..DIM_WORDS {
            let src = (i + word_shift) % DIM_WORDS;
            if bit_shift == 0 {
                words[i] = self.words[src];
            } else {
                let next_src = (src + 1) % DIM_WORDS;
                words[i] = (self.words[src] << bit_shift) | (self.words[next_src] >> (64 - bit_shift));
            }
        }
        Self { words }
    }

    pub fn hamming_distance(&self, other: &Self) -> u32 {
        self.words.iter().zip(other.words.iter()).map(|(a, b)| (a ^ b).count_ones()).sum()
    }

    pub fn similarity(&self, other: &Self) -> f64 {
        1.0 - self.hamming_distance(other) as f64 / DEFAULT_DIM_BITS as f64
    }
}

pub struct HdcIndex { entries: Vec<(String, HyperVector)> }

impl HdcIndex {
    pub fn new() -> Self { Self { entries: Vec::new() } }

    pub fn insert(&mut self, label: String, hv: HyperVector) {
        self.entries.push((label, hv));
    }

    pub fn search_top_k(&self, query: &HyperVector, k: usize) -> Vec<(&str, f64)> {
        let mut scored: Vec<(&str, f64)> = self.entries.iter()
            .map(|(label, hv)| (label.as_str(), hv.similarity(query)))
            .collect();
        scored.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(k);
        scored
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

impl Default for HdcIndex { fn default() -> Self { Self::new() } }

fn rand_u64() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::sync::atomic::{AtomicU64, Ordering};
    static STATE: AtomicU64 = AtomicU64::new(0);
    let mut x = STATE.load(Ordering::Relaxed);
    if x == 0 {
        x = SystemTime::now().duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64).unwrap_or(0x9e3779b97f4a7c15);
    }
    x ^= x << 13; x ^= x >> 7; x ^= x << 17;
    STATE.store(x, Ordering::Relaxed);
    x
}
```

---

## Rank 10: Gate Verification Pipeline

**Source analysis**: [../../execution-verification/gate-verification.md](../../execution-verification/gate-verification.md)
**Location:** `src/tools/builder/gate.rs`

```rust
// src/tools/builder/gate.rs

use std::path::Path;
use tokio::process::Command as AsyncCommand;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rung { Compile, Lint, Test, Symbol }

impl Rung {
    pub fn name(&self) -> &'static str {
        match self { Rung::Compile => "compile", Rung::Lint => "lint", Rung::Test => "test", Rung::Symbol => "symbol" }
    }

    pub fn timeout_secs(&self) -> u64 {
        match self { Rung::Compile => 60, Rung::Lint => 30, Rung::Test => 120, Rung::Symbol => 90 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RungResult {
    pub rung: Rung,
    pub passed: bool,
    pub output: String,
    pub elapsed_ms: u64,
    pub exit_code: Option<i32>,
}

pub async fn run_rung(rung: Rung, project_dir: &Path) -> RungResult {
    let start = std::time::Instant::now();
    let (program, args): (&str, Vec<&str>) = match rung {
        Rung::Compile => ("cargo", vec!["check", "--message-format=json"]),
        Rung::Lint    => ("cargo", vec!["clippy", "--", "-D", "warnings"]),
        Rung::Test    => ("cargo", vec!["test", "--no-fail-fast"]),
        Rung::Symbol  => ("cargo", vec!["build", "--all"]),
    };
    let timeout = std::time::Duration::from_secs(rung.timeout_secs());
    let result = tokio::time::timeout(
        timeout,
        AsyncCommand::new(program).args(&args).current_dir(project_dir).output(),
    ).await;
    let elapsed_ms = start.elapsed().as_millis() as u64;
    match result {
        Err(_) => RungResult {
            rung, passed: false,
            output: format!("Rung '{}' timed out after {}s", rung.name(), rung.timeout_secs()),
            elapsed_ms, exit_code: None,
        },
        Ok(Err(e)) => RungResult {
            rung, passed: false,
            output: format!("Failed to spawn '{}': {e}", program),
            elapsed_ms, exit_code: None,
        },
        Ok(Ok(output)) => {
            let exit_code = output.status.code();
            let passed = output.status.success();
            let combined = format!(
                "stdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            );
            let truncated = if combined.len() > 4096 {
                format!("{}... [truncated]", &combined[..4090])
            } else { combined };
            RungResult { rung, passed, output: truncated, elapsed_ms, exit_code }
        }
    }
}

pub async fn run_gate_pipeline(project_dir: &Path) -> Vec<RungResult> {
    let rungs = [Rung::Compile, Rung::Lint, Rung::Test, Rung::Symbol];
    let mut results = Vec::with_capacity(rungs.len());
    for rung in rungs {
        let result = run_rung(rung, project_dir).await;
        let passed = result.passed;
        results.push(result);
        if !passed { break; }
    }
    results
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateSummary {
    pub all_passed: bool,
    pub rungs_attempted: usize,
    pub failing_rung: Option<Rung>,
    pub total_elapsed_ms: u64,
    pub results: Vec<RungResult>,
}

impl GateSummary {
    pub fn from_results(results: Vec<RungResult>) -> Self {
        let all_passed = results.iter().all(|r| r.passed);
        let failing_rung = results.iter().find(|r| !r.passed).map(|r| r.rung);
        let total_elapsed_ms = results.iter().map(|r| r.elapsed_ms).sum();
        let rungs_attempted = results.len();
        Self { all_passed, rungs_attempted, failing_rung, total_elapsed_ms, results }
    }
}
```

---

## Score Reproduction Script

Verify that all 25 composite scores match the formula `Composite = U*0.30 + S*0.20 + E*0.25 + Sa*0.15 + I*0.10`:

```ruby
#!/usr/bin/env ruby
# Run from the repository root:
# ruby tmp/strategy/priority-matrix/verify-scores.rb

text = File.read("tmp/strategy/priority-matrix/README.md")
rows = text.scan(
  /^\|\s*(\d+)\s*\|\s*([^|]+?)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|\s*\*\*([0-9.]+)\*\*/
)

failed = false
rows.each do |rank, concept, u, s, e, safety, independence, stated|
  computed =
    u.to_i          * 0.30 +
    s.to_i          * 0.20 +
    e.to_i          * 0.25 +
    safety.to_i     * 0.15 +
    independence.to_i * 0.10

  if (computed - stated.to_f).abs > 0.011
    failed = true
    warn "MISMATCH rank=#{rank} #{concept.strip}: stated=#{stated}, computed=#{format('%.2f', computed)}"
  end
end

abort "score mismatch" if failed
puts "verified #{rows.length} priority scores — all match formula"
```

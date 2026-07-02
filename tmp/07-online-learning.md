# Online Learning: Bandit-Based Model Routing

**Source crate**: `roko-learn` (`crates/roko-learn/src/`)
**Priority**: HIGH -- immediate LLM cost savings via intelligent model selection
**Roko docs**: `docs/v2/07-LEARNING.md` (Loop 2: Strategy Routing), `docs/v1/05-learning/13-8-missing-feedback-loops.md` (reward weights derivation)
**Plan**: `plans/P19-cascade-router-acp/tasks.toml` (production wiring)

---

## Table of Contents

1. [What This Document Covers](#1-what-this-document-covers)
2. [The Problem: Static Model Selection Is Wasteful](#2-the-problem-static-model-selection-is-wasteful)
3. [Background: From Multi-Armed Bandits to Contextual Bandits](#3-background-from-multi-armed-bandits-to-contextual-bandits)
4. [The LinUCB Algorithm -- Full Mathematical Derivation](#4-the-linucb-algorithm----full-mathematical-derivation)
5. [The 18-Dimensional Context Vector](#5-the-18-dimensional-context-vector)
6. [The 3-Stage Cascade Router](#6-the-3-stage-cascade-router)
7. [Pareto Frontier Computation](#7-pareto-frontier-computation)
8. [Provider Health Circuit Breaker](#8-provider-health-circuit-breaker)
9. [Reward Signal Composition](#9-reward-signal-composition)
10. [EWMA Anomaly Detection](#10-ewma-anomaly-detection)
11. [Bayesian Confidence (Beta-Binomial)](#11-bayesian-confidence-beta-binomial)
12. [Episode Logging and Routing Audit Trail](#12-episode-logging-and-routing-audit-trail)
13. [Latency Tracking](#13-latency-tracking)
14. [Active Inference Tier Selection](#14-active-inference-tier-selection)
15. [Worked Example: Learning to Route Different Query Types](#15-worked-example-learning-to-route-different-query-types)
16. [IronClaw Integration Plan](#16-ironclaw-integration-plan)
17. [References](#17-references)

---

## 1. What This Document Covers

This document describes the **online learning** system in roko's `roko-learn` crate -- a production implementation of contextual bandits for intelligent LLM model routing. "Online learning" here means a system that learns from each individual decision it makes, updating its internal model after every observation, rather than training on a fixed dataset and deploying a frozen model.

The system solves a concrete problem: given a request that needs an LLM response, which model should serve it? The answer depends on the request's complexity, the task category, the agent's role, cost constraints, latency budgets, and observed historical performance. A static mapping (always use `claude-sonnet-4-6`) wastes money on simple tasks and delivers poor quality on hard ones. The online learning system learns, from real production traffic, which models perform best for which types of requests.

This document covers the full stack:

- **Mathematical foundations**: LinUCB contextual bandits, UCB1, Thompson sampling, Pareto optimization
- **Roko implementation**: Verified code citations from `crates/roko-learn/src/`
- **IronClaw integration plan**: How to port this into IronClaw's `crates/ironclaw_llm/` provider chain

---

## 2. The Problem: Static Model Selection Is Wasteful

Every LLM request must pick a model. Today, IronClaw's choice is static: the config says `LLM_MODEL=claude-sonnet-4-6` and every request goes there regardless of whether the task is a trivial greeting, a complex multi-step code refactor, or a simple translation. This is wasteful in two directions:

**Overspending on simple tasks.** A greeting like "What time is it in Tokyo?" costs the same as a 50-turn code refactoring session because both use the same expensive model. A cheaper model (GPT-4o-mini, Claude Haiku) handles the greeting equally well at 5-10x lower cost.

**Underperforming on hard tasks.** Conversely, when the config points to a cheap model for cost savings, complex tasks suffer. A code architecture question that needs Claude Opus gets Claude Haiku and produces garbage that the user must retry -- wasting both money and time.

**The scale of the opportunity.** In roko's production deployment, analysis of routing logs shows that approximately 60% of tasks are "simple" (mechanical transformations, lookups, formatting) while only 15% are genuinely complex (architectural decisions, multi-file refactors). A system that routes the simple 60% to a model that costs 10x less and reserves the expensive model for the hard 15% can reduce total LLM spend by 40-50% with no quality degradation.

The static routing in IronClaw's `SmartRoutingProvider` (`crates/ironclaw_llm/src/smart_routing.rs`) is a step in the right direction -- it scores prompt complexity across 13 dimensions and routes to cheap vs primary models. But its thresholds are hand-tuned and fixed. It cannot learn that a particular user's "code review" requests are actually simple rubber-stamp approvals, or that a certain project's "translations" are actually complex domain-specific localizations. Online learning closes this gap.

---

## 3. Background: From Multi-Armed Bandits to Contextual Bandits

### 3.1 The Multi-Armed Bandit Problem

The multi-armed bandit (MAB) problem is named after a gambler facing a row of slot machines ("one-armed bandits"), each with an unknown payout distribution. The gambler must decide which machine to play at each step to maximize total reward over time. The fundamental tension is between **exploitation** (playing the machine with the highest observed average payout) and **exploration** (trying other machines to learn whether they might be better).

The classic result by Auer, Cesa-Bianchi, and Fischer (2002) [1] established the UCB1 algorithm, which achieves logarithmic cumulative regret -- meaning the cost of learning converges to zero relative to the total number of decisions. For each arm `a`, UCB1 computes:

```
UCB(a) = mean_reward(a) + C * sqrt( ln(total_pulls) / pulls(a) )
```

The first term exploits (prefer arms with high observed rewards). The second term explores (prefer arms with few observations, since `sqrt(ln(N)/n)` is large when `n` is small). The constant `C` controls the exploration-exploitation tradeoff.

Roko implements this in `crates/roko-learn/src/bandits.rs`:

```rust
// crates/roko-learn/src/bandits.rs

/// Statistics for a single arm of a UcbBandit.
pub struct BanditArm {
    pub name: String,
    pub pulls: u64,
    pub total_reward: f64,
}

impl BanditArm {
    pub fn mean_reward(&self) -> f64 {
        if self.pulls == 0 { 0.0 }
        else { self.total_reward / (self.pulls as f64) }
    }
}
```

The `UcbBandit` uses `parking_lot::RwLock` for arm stats and `AtomicU64` for the pull counter, so `select` only acquires a shared read lock while `update` acquires an exclusive write lock.

### 3.2 Why Context Matters: From UCB1 to LinUCB

Plain UCB1 treats all requests identically -- it learns a single "best model" globally. But the best model depends on the request. A greeting is best served by a cheap fast model; a security audit needs the most capable model available. The mapping from request characteristics to optimal model is what a **contextual bandit** learns.

In a contextual bandit, the learner observes a **context vector** `x` before choosing an arm. The expected reward of arm `a` given context `x` is modeled as a function `f(x, a)`. LinUCB (Li et al., 2010) [2] assumes this function is linear:

```
E[reward | x, a] = theta_a^T * x
```

where `theta_a` is a weight vector specific to arm `a`, learned via ridge regression from observed `(context, reward)` pairs. This is the algorithm roko uses for model routing.

### 3.3 Thompson Sampling Alternative

Roko also implements **Thompson sampling** (Thompson, 1933 [3]), a Bayesian alternative to UCB. Instead of computing confidence bounds, Thompson sampling maintains a posterior distribution over each arm's reward rate and samples from it:

```rust
// crates/roko-learn/src/model_router.rs

pub struct ThompsonArm {
    pub slug: String,
    pub alpha: f64,   // Beta prior: success count + 1
    pub beta: f64,    // Beta prior: failure count + 1
    pub sum_reward: f64,
    pub sum_reward_sq: f64,
    pub observations: u64,
    pub discount: f64,  // For non-stationarity
}

impl ThompsonArm {
    pub fn update(&mut self, reward: f64, success: bool) {
        // Discount prior to handle non-stationarity
        self.alpha = 1.0 + self.discount * (self.alpha - 1.0);
        self.beta = 1.0 + self.discount * (self.beta - 1.0);
        if success { self.alpha += 1.0; } else { self.beta += 1.0; }
        self.sum_reward += reward;
        self.sum_reward_sq += reward * reward;
        self.observations += 1;
    }
}
```

The `discount` factor (configurable, default 0.99 in `roko-core/src/config/routing.rs`) gently decays the prior before each update, preventing the system from becoming too confident in stale observations. This handles **non-stationarity** -- model quality changes over time as providers update their models.

The routing algorithm is selectable at configuration time:

```rust
// crates/roko-core/src/config/routing.rs

pub enum RoutingAlgorithm {
    LinUcb,     // Contextual bandit with upper-confidence bounds (default)
    Thompson,   // Discounted Thompson sampling for non-stationary routing
}
```

### 3.4 Track-and-Stop for Best-Arm Identification

For tool format selection (a different decision domain from model routing), roko implements the **Track-and-Stop** algorithm (Garivier and Kaufmann, 2016 [4]). Unlike UCB1 which minimizes cumulative regret, Track-and-Stop minimizes the number of samples needed to identify the best arm with probability >= 1 - delta. Once the best format for a `(model, role, tool_count, complexity)` key is identified with sufficient confidence, exploration stops permanently.

This is implemented in `crates/roko-learn/src/bandits.rs` as `TrackAndStopBandit`, implementing the `FormatBandit` trait.

---

## 4. The LinUCB Algorithm -- Full Mathematical Derivation

### 4.1 Setup

We have `K` arms (LLM models), a `d`-dimensional context vector `x`, and a per-arm weight vector `theta_a` (unknown). The expected reward of arm `a` given context `x` is:

```
E[r_t | x_t, a_t = a] = theta_a^T * x_t
```

### 4.2 Ridge Regression Estimator

After observing `n` interactions with arm `a` (contexts `x_1, ..., x_n` and rewards `r_1, ..., r_n`), we estimate `theta_a` via ridge regression:

```
theta_hat_a = A_a^{-1} * b_a
```

where:

```
A_a = I_d + sum_{t: a_t=a} x_t * x_t^T     (d x d matrix)
b_a = sum_{t: a_t=a} r_t * x_t               (d x 1 vector)
```

The `I_d` term is the ridge regularizer (identity matrix), which prevents singularity when few observations are available and provides implicit Bayesian regularization toward a zero prior.

### 4.3 Upper Confidence Bound

The UCB score for arm `a` given context `x` is:

```
score(a) = theta_hat_a^T * x  +  alpha * sqrt(x^T * A_a^{-1} * x)
             \_____________/       \____________________________/
              exploitation                  exploration
```

The first term is the estimated reward (exploitation). The second term is the uncertainty bonus (exploration). `sqrt(x^T * A_a^{-1} * x)` is large when the context `x` lies in a direction where arm `a` has few observations, encouraging exploration in under-sampled regions of the context space.

### 4.4 Why This Works (Intuition)

The matrix `A_a^{-1}` is the inverse of the regularized covariance matrix of contexts seen by arm `a`. When arm `a` has been tried many times in contexts similar to the current `x`, `A_a` is large in the direction of `x`, so `x^T * A_a^{-1} * x` is small and the uncertainty bonus shrinks. When arm `a` has rarely been tried in contexts like `x`, the bonus is large and the algorithm explores.

This is fundamentally an online ridge regression with confidence ellipsoids -- a direct application of the theory developed by Abbasi-Yadkori et al. (2011) [5] for linear bandits.

### 4.5 Implementation: Per-Arm State

In roko, each arm maintains its own `A` matrix and `b` vector:

```rust
// crates/roko-learn/src/model_router.rs

pub struct ArmState {
    pub slug: String,
    pub a_matrix: Vec<Vec<f64>>,  // A matrix (d x d), stored row-major
    pub b_vector: Vec<f64>,       // b vector (d x 1)
    pub observations: u64,
    pub reward_stats: MultiObjectiveStats,
    pub ewc: EwcRegularizer,      // Elastic regularizer protecting consolidated weights
}

impl ArmState {
    fn new(slug: impl Into<String>, dim: usize) -> Self {
        let mut a = vec![vec![0.0; dim]; dim];
        for (i, row) in a.iter_mut().enumerate() {
            row[i] = 1.0;  // Initialize A = I_d (identity matrix)
        }
        Self {
            slug: slug.into(),
            a_matrix: a,
            b_vector: vec![0.0; dim],
            observations: 0,
            reward_stats: MultiObjectiveStats::default(),
            ewc: EwcRegularizer::new(dim),
        }
    }
}
```

### 4.6 Computing A^{-1}: Cholesky Decomposition

The 18x18 matrix inverse is computed via Cholesky decomposition, implemented inline without external linear algebra dependencies:

```rust
// crates/roko-learn/src/model_router.rs

fn cholesky_inverse(a: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = a.len();
    // Step 1: Cholesky decomposition A = L * L^T
    let mut l = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let mut s: f64 = a[i][j];
            for k in 0..j { s -= l[i][k] * l[j][k]; }
            if i == j {
                if s <= 0.0 { return None; }  // Not positive definite
                l[i][j] = s.sqrt();
            } else {
                l[i][j] = s / l[j][j];
            }
        }
    }
    // Step 2: Invert L (lower triangular)
    let mut l_inv = vec![vec![0.0; n]; n];
    for i in 0..n {
        l_inv[i][i] = 1.0 / l[i][i];
        for j in (0..i).rev() {
            let mut s = 0.0;
            for k in j..i { s += l[i][k] * l_inv[k][j]; }
            l_inv[i][j] = -s / l[i][i];
        }
    }
    // Step 3: A^{-1} = L^{-T} * L^{-1}
    let mut inv = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let mut s = 0.0;
            for k in i..n { s += l_inv[k][i] * l_inv[k][j]; }
            inv[i][j] = s;
            inv[j][i] = s;  // Symmetric
        }
    }
    Some(inv)
}
```

No external linear algebra library is needed. For a fixed 18-dimensional context vector, this inline Cholesky decomposition is O(d^3) = O(5832) -- trivial compared to an LLM API call.

### 4.7 Alpha Decay: Shifting from Exploration to Exploitation

The exploration parameter `alpha` decays exponentially from 1.0 (aggressive exploration) toward 0.05 (mostly exploitation) as observations accumulate:

```
alpha = 0.05 + 0.95 * exp(-observations / 60)
```

| Observations | alpha | Behavior |
|:-------------|:------|:---------|
| 0 | 1.0 | Full exploration |
| 50 | 0.47 | Balanced |
| 120 | 0.18 | Mostly exploitation |
| 200 | 0.09 | Near-exploitation |
| 500 | ~0.05 | Exploitation with residual exploration |

This decay schedule was tuned empirically. The residual 0.05 floor ensures the system never stops exploring entirely, which is important because model quality is non-stationary (providers update their models).

### 4.8 The LinUCBRouter

The router wraps all per-arm state behind a `parking_lot::RwLock`:

```rust
// crates/roko-learn/src/model_router.rs

pub struct LinUCBRouter {
    state: RwLock<RouterState>,
    persist_path: Option<PathBuf>,
    static_table: HashMap<ModelTier, String>,
}

struct RouterState {
    arms: Vec<ArmState>,
    total_observations: u64,
}
```

Thread safety is important: multiple async tasks route requests concurrently. Read-side routing (`select`) takes a shared read lock; write-side updates (`observe_reward`) take an exclusive write lock. Since routing is far more frequent than observation, this asymmetric locking minimizes contention.

---

## 5. The 18-Dimensional Context Vector

The context vector `x` encodes everything the router needs to make a good decision. From the module doc comment in `crates/roko-learn/src/model_router.rs`:

| Dimensions | Feature | Encoding | Range |
|:-----------|:--------|:---------|:------|
| 1-8 | Task category | One-hot for `TaskCategory` variants (Implementation, Testing, Review, Planning, Debugging, Documentation, Refactoring, Analysis) | {0, 1} |
| 9 | Complexity band | Scalar: Simple=0.0, Standard=0.5, Complex=1.0 | [0, 1] |
| 10 | Iteration | Normalized: `min(iteration / 10, 1.0)` | [0, 1] |
| 11-14 | Agent role | Hashed to 4-dimensional float vector | [0, 1] |
| 15 | Crate familiarity | `success_count / total_count` for the crate being modified | [0, 1] |
| 16 | Has prior failure | Binary: 0.0 or 1.0 | {0, 1} |
| 17 | Bias term | Always 1.0 (intercept) | {1} |
| 18 | Cache affinity | 1.0 when the candidate matches the previous model | {0, 1} |

The `RoutingContext` struct encodes these features:

```rust
// crates/roko-learn/src/cascade_router.rs (via model_router.rs RoutingContext)

pub struct RoutingContext {
    pub task_category: TaskCategory,
    pub complexity: TaskComplexityBand,
    pub iteration: u32,
    pub role: AgentRole,
    pub crate_familiarity: f64,
    pub has_prior_failure: bool,
    pub conductor_load: f64,
    pub active_agents: usize,
    pub ready_queue_depth: usize,
    pub max_queue_wait_hours: f64,
    pub daimon_policy: DaimonPolicy,
    pub thinking_level: Option<u8>,
    pub temperament: Option<roko_core::temperament::Temperament>,
    pub previous_model: Option<String>,
    pub plan_context_tokens: Option<u64>,
    pub tier_thresholds: Option<roko_core::agent::TaskRequirements>,
}
```

**Why 18 dimensions?** This is small enough that the Cholesky inverse is trivially cheap (< 1ms), the ridge regression converges with modest data (50-200 observations per arm), and the matrix can be stored inline as `Vec<Vec<f64>>` without a linear algebra library. Larger context vectors would slow convergence without meaningfully improving routing quality.

**Cache affinity** (dimension 18) is notable: when the candidate being scored is the same model that handled the previous request, the affinity feature is 1.0. If the bandit learns a positive weight for this feature, it creates a preference for keeping the same model across turns -- useful because it avoids cold cache penalties with some providers.

---

## 6. The 3-Stage Cascade Router

The cascade router is the central orchestrator that wraps the LinUCB bandit inside a staged selection process. It addresses the **cold start problem**: with zero observations, the bandit has no data to learn from, so the system needs a fallback strategy.

### 6.1 Stage Definitions

From `crates/roko-learn/src/cascade_router.rs`:

| Stage | Name | Observations | Strategy |
|:------|:-----|:-------------|:---------|
| 1 | Static | < 50 | Hardcoded role -> model table |
| 2 | Confidence | 50 - 200 | Empirical pass rates + confidence intervals |
| 3 | UCB | > 200 | Full LinUCB contextual bandit |

```rust
// crates/roko-learn/src/cascade_router.rs

pub struct CascadeRouter {
    linucb: LinUCBRouter,
    confidence_stats: Mutex<HashMap<String, ModelStats>>,
    pareto_frontier: Mutex<ParetoFrontierState>,
    role_table: Mutex<HashMap<AgentRole, String>>,
    model_slugs: Vec<String>,
    tier_map: HashMap<String, ModelTier>,
    stage_tracking: Mutex<StageTracking>,
    free_tier_shadow_runner: Option<Arc<dyn ShadowModelRunner>>,
}
```

### 6.2 Stage 1: Static Mapping (Cold Start)

With fewer than 50 observations, the router uses a hardcoded mapping from agent role to model. For example, an `Implementer` agent might default to `claude-sonnet-4-6`, while a `Reviewer` might get `gpt-4o`. This provides reasonable defaults until enough data accumulates.

The mapping is configurable via `roko.toml`:

```toml
[routing]
algorithm = "linucb"
fast_task_model = "claude-haiku-4-5"
standard_task_model = "claude-sonnet-4-6"
complex_task_model = "claude-opus-4-6"
```

### 6.3 Stage 2: Confidence-Based Selection

Between 50 and 200 observations, the router has enough data to compute empirical pass rates but not enough for the bandit's confidence bounds to be tight. It tracks per-model statistics:

```rust
// crates/roko-learn/src/cascade/types.rs

pub struct ModelStats {
    pub successes: u64,
    pub failures: u64,
}
```

The router selects the model with the highest lower confidence bound on pass rate. This is equivalent to a Wilson score interval -- preferring models where we are confident the true pass rate is high, rather than models with a high but uncertain pass rate (which could be an artifact of small sample size).

### 6.4 Stage 3: Full LinUCB

With more than 200 observations, the contextual bandit takes over. The alpha decay schedule ensures the transition is smooth: at 200 observations, alpha is approximately 0.09 (mostly exploitation with residual exploration).

### 6.5 Stage Transitions

Stage transitions are tracked and logged:

```rust
// crates/roko-learn/src/cascade_router.rs

pub fn check_stage_transition(&self) -> Option<StageTransition> {
    let obs = self.total_observations();
    let next = stage_for_observations(obs);
    let mut tracking = self.stage_tracking.lock();
    if next == tracking.current { return None; }
    let transition = StageTransition {
        from: tracking.current,
        to: next,
        observations: obs,
        timestamp: Utc::now(),
    };
    tracking.current = next;
    tracking.transitions.push(transition.clone());
    tracing::info!(
        from = %transition.from, to = %transition.to,
        observations = transition.observations,
        "cascade router stage transition"
    );
    Some(transition)
}
```

### 6.6 CascadeModel Output

The router returns a `CascadeModel` containing not just a primary model but also a fallback chain and a context-overflow fallback:

```rust
// crates/roko-learn/src/cascade/types.rs

pub struct CascadeModel {
    pub primary: ModelSpec,
    pub fallback_chain: Vec<ModelSpec>,
    pub context_overflow_fallback: Option<ModelSpec>,
    pub latency_sla_ms: u64,
    pub stage: CascadeStage,
}

impl CascadeModel {
    pub fn model_for_attempt(&self, attempt: usize) -> Option<&ModelSpec> {
        match attempt {
            0 => Some(&self.primary),
            _ => self.fallback_chain.get(attempt - 1),
        }
    }
}
```

This means a single routing decision produces a complete execution plan: "try Sonnet first, fall back to GPT-4o on error, and if the context overflows, try Gemini 2.5 Pro (which has a 1M token window)."

---

## 7. Pareto Frontier Computation

Not all models are worth considering. A model that is worse than another model on **every** dimension (quality, cost, latency, reliability) is **dominated** and should be excluded from the candidate set. The set of non-dominated models forms the **Pareto frontier** (Deb, 2001 [6]).

### 7.1 4-Objective Dominance

From `crates/roko-learn/src/pareto.rs`:

```rust
// crates/roko-learn/src/pareto.rs

pub struct ModelObservation {
    pub pass_rate: f64,          // Higher is better
    pub cost_per_success: f64,   // Lower is better
    pub avg_latency_ms: f64,     // Lower is better
    pub reliability: f64,        // Higher is better (non-error fraction)
    pub observations: u64,
}

pub fn compute_pareto_frontier(stats: &HashMap<String, ModelObservation>) -> Vec<String> {
    let mut frontier = Vec::new();
    for (slug_a, obs_a) in stats {
        let dominated = stats.iter().any(|(slug_b, obs_b)| {
            if slug_b == slug_a { return false; }
            let quality_ok = obs_b.pass_rate >= obs_a.pass_rate;
            let cost_ok = obs_b.cost_per_success <= obs_a.cost_per_success;
            let latency_ok = obs_b.avg_latency_ms <= obs_a.avg_latency_ms;
            let reliability_ok = obs_b.reliability >= obs_a.reliability;
            let all_geq = quality_ok && cost_ok && latency_ok && reliability_ok;
            let any_strictly_better = obs_b.pass_rate > obs_a.pass_rate
                || obs_b.cost_per_success < obs_a.cost_per_success
                || obs_b.avg_latency_ms < obs_a.avg_latency_ms
                || obs_b.reliability > obs_a.reliability;
            all_geq && any_strictly_better
        });
        if !dominated { frontier.push(slug_a.clone()); }
    }
    frontier.sort();
    frontier
}
```

A model is Pareto-dominated when another model is at least as good on ALL four objectives and strictly better on at least one. Models on the frontier represent genuine tradeoffs (e.g., Model A is cheaper but slower; Model B is faster but more expensive).

### 7.2 Weighted Scalarization

For ranking within the frontier, roko uses weighted scalarization:

```rust
// crates/roko-learn/src/pareto.rs

pub struct ParetoWeights {
    pub quality: f64,
    pub cost: f64,
    pub latency: f64,
    pub reliability: f64,
}

pub fn scalarize(obs: &ModelObservation, weights: &ParetoWeights) -> f64 {
    let total_weight = (weights.quality + weights.cost
        + weights.latency + weights.reliability).max(f64::EPSILON);
    let cost_normalized = 1.0 - (obs.cost_per_success / 100.0).clamp(0.0, 1.0);
    let latency_normalized = 1.0 - (obs.avg_latency_ms / 60_000.0).clamp(0.0, 1.0);
    (weights.quality * obs.pass_rate
        + weights.cost * cost_normalized
        + weights.latency * latency_normalized
        + weights.reliability * obs.reliability) / total_weight
}
```

Cost and latency are inverted (1 - normalized) so that higher scores always mean "better." The normalization caps (100.0 for cost, 60000.0 for latency) are chosen to represent a reasonable maximum; values beyond these are clamped.

### 7.3 Multi-Objective ParetoFrontier

For generalized multi-objective optimization, roko provides the `ParetoFrontier` struct:

```rust
// crates/roko-learn/src/pareto.rs

pub struct ParetoSolution {
    pub values: Vec<f64>,
    pub model_id: String,
}

pub struct ParetoFrontier {
    pub objectives: Vec<String>,
    pub solutions: Vec<ParetoSolution>,
}

pub fn is_dominated(a: &ParetoSolution, b: &ParetoSolution) -> bool {
    let len = a.values.len().min(b.values.len());
    if len == 0 { return false; }
    let mut all_geq = true;
    let mut any_gt = false;
    for i in 0..len {
        if a.values[i] < b.values[i] { all_geq = false; break; }
        if a.values[i] > b.values[i] { any_gt = true; }
    }
    all_geq && any_gt
}
```

All objectives in `ParetoSolution` are assumed to be maximized. Callers that want to minimize a metric (e.g., cost) negate it before constructing solutions.

### 7.4 Integration with the Cascade

The Pareto frontier is recomputed periodically (controlled by `PARETO_RECOMPUTE_INTERVAL`) and cached in `CascadeRouter.pareto_frontier`. During UCB selection, dominated models receive a penalty that discourages (but does not entirely prevent) their selection.

---

## 8. Provider Health Circuit Breaker

Before considering a model for routing, the system must verify that its provider is healthy. A provider experiencing an outage should be temporarily removed from the candidate set, not selected and then retried.

### 8.1 Three-State Machine

From `crates/roko-learn/src/provider_health.rs`:

```
Healthy (Closed) --[3 consecutive failures]--> Unhealthy (Open)
       ^                                              |
       |                                       [cooldown expires]
       |                                              v
       +---[record_success]-------- Probing (HalfOpen)
                                   [record_failure]--> Unhealthy (timer reset)
```

```rust
// crates/roko-learn/src/provider_health.rs

pub enum CircuitState { Closed, Open, HalfOpen }

pub enum ErrorClass {
    RateLimit, AuthFailure, Timeout,
    ServerError, ContentPolicy, ContextOverflow, Unknown,
}

pub struct ProviderHealth {
    pub provider_id: String,
    pub state: CircuitState,
    pub consecutive_failures: u32,
    pub total_requests: u64,
    pub total_failures: u64,
    pub last_failure_at: Option<i64>,
    pub cooldown_until: Option<i64>,
    pub failure_window: VecDeque<FailureRecord>,
}
```

### 8.2 Error-Class-Dependent Cooldown

Different failure types trigger different cooldown durations. A rate limit (429 response) warrants a shorter cooldown than a server error (5xx), because rate limits are usually transient. Auth failures may warrant a longer cooldown because they typically require configuration changes.

### 8.3 Availability Check

```rust
// crates/roko-learn/src/provider_health.rs

impl ProviderHealth {
    pub fn is_available(&mut self, now_ms: i64) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(until) = self.cooldown_until {
                    if now_ms >= until {
                        self.state = CircuitState::HalfOpen;
                        return true;  // Allow one probe request
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn record_success(&mut self) {
        self.total_requests = self.total_requests.saturating_add(1);
        self.consecutive_failures = 0;
        self.cooldown_until = None;
        if self.state == CircuitState::HalfOpen || self.state == CircuitState::Open {
            self.state = CircuitState::Closed;
        }
    }

    pub fn record_failure(&mut self, error: ErrorClass, now_ms: i64) {
        self.total_requests = self.total_requests.saturating_add(1);
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        self.total_failures = self.total_failures.saturating_add(1);
        self.last_failure_at = Some(now_ms);
        // Trip to Open after 3 consecutive failures
        if self.consecutive_failures >= 3 {
            self.state = CircuitState::Open;
            self.cooldown_until = Some(now_ms + self.cooldown_ms(error));
        }
    }
}
```

The circuit breaker pattern was popularized by Nygard (2007) [7] and is now standard in distributed systems. The key insight is **fail fast**: when a provider is down, immediately return an error (or use a fallback) rather than waiting for each request to time out.

Recovery timestamps use `std::time::Instant` (immune to wall-clock adjustments, not serializable) for the in-memory runtime, while persisted snapshots use unix milliseconds.

The rolling failure window retains the last 20 failures, enabling trend analysis (e.g., "this provider has been flapping for the last hour").

---

## 9. Reward Signal Composition

The reward fed to the bandit must capture what "good routing" means. Roko uses a weighted combination of three objectives: quality, cost, and latency.

### 9.1 Reward Weights

From `crates/roko-core/src/config/routing.rs`:

```rust
// crates/roko-core/src/config/routing.rs

pub struct RewardWeights {
    pub quality: f64,   // Default: 0.5
    pub cost: f64,      // Default: 0.3
    pub latency: f64,   // Default: 0.2
    pub knowledge_bias: Option<f64>,
}
```

The defaults reflect a priority ordering: quality > cost > latency. This is appropriate for a coding agent where a wrong answer wastes more time (and money, via retries) than a slightly more expensive correct answer.

Weights can be overridden per task complexity tier:

```rust
// crates/roko-core/src/config/routing.rs

pub struct RoutingRewardWeightsConfig {
    pub default: RewardWeights,
    pub mechanical: Option<RewardWeights>,    // Simple tasks: favor cost
    pub focused: Option<RewardWeights>,       // Standard tasks: balanced
    pub integrative: Option<RewardWeights>,   // Complex tasks: favor quality
    pub architectural: Option<RewardWeights>, // Critical tasks: strongly favor quality
}
```

### 9.2 Scalarized Reward Computation

```rust
// crates/roko-learn/src/model_router.rs

pub fn compute_routing_reward_with_weights(
    pass_rate: f64,
    normalized_cost: f64,
    normalized_duration: f64,
    weights: &RewardWeights,
) -> f64 {
    let pr = pass_rate.clamp(0.0, 1.0);
    let nc = normalized_cost.clamp(0.0, 1.0);
    let nd = normalized_duration.clamp(0.0, 1.0);
    (1.0 - nd).mul_add(
        weights.latency,
        pr.mul_add(weights.quality, (1.0 - nc) * weights.cost),
    )
}
```

Breaking this down with default weights (0.5, 0.3, 0.2):

```
reward = 0.5 * pass_rate + 0.3 * (1 - normalized_cost) + 0.2 * (1 - normalized_duration)
```

Both cost and duration are inverted: lower cost and lower latency produce higher reward.

### 9.3 Latency-SLA-Normalized Variant

The v2 reward function normalizes observed latency against a per-model latency SLA:

```rust
// crates/roko-learn/src/model_router.rs

pub fn compute_routing_reward_v2(
    pass_rate: f64,
    normalized_cost: f64,
    observed_latency_ms: f64,
    latency_sla_ms: f64,
) -> f64 {
    let normalized_duration = if latency_sla_ms > 0.0 {
        (observed_latency_ms / latency_sla_ms).min(1.0)
    } else {
        1.0
    };
    compute_routing_reward_with_weights(
        pass_rate, normalized_cost, normalized_duration,
        &RewardWeights::default(),
    )
}
```

This means a model that takes 2 seconds when its SLA is 5 seconds gets `normalized_duration = 0.4` (rewarded for being under budget), while a model that takes 6 seconds on a 5-second SLA gets `normalized_duration = 1.0` (maximum latency penalty, capped).

### 9.4 Multi-Objective Stats

Each arm also maintains independent tracking of quality, cost, and latency for Pareto analysis:

```rust
// crates/roko-learn/src/model_router.rs

pub struct MultiObjectiveStats {
    pub quality_sum: f64,
    pub quality_sq_sum: f64,
    pub cost_sum: f64,
    pub cost_sq_sum: f64,
    pub latency_sum: f64,
    pub latency_sq_sum: f64,
    pub observations: u64,
}
```

The squared sums enable variance computation for confidence intervals without storing individual observations.

---

## 10. EWMA Anomaly Detection

The anomaly detector runs alongside the routing pipeline, watching for three patterns that indicate something has gone wrong.

### 10.1 Detector Design

From `crates/roko-learn/src/anomaly.rs`:

```rust
// crates/roko-learn/src/anomaly.rs

pub struct AnomalyDetector {
    prompt_hash_window: VecDeque<u64>,  // Last 20 prompt hashes
    cost_ewma: EwmaState,
    quality_history: VecDeque<f64>,     // Last 50 quality scores
    session_cost_usd: f64,
    session_start_ms: i64,
}
```

### 10.2 Prompt Loop Detection

A prompt loop occurs when the agent keeps sending the same (or very similar) prompt to the LLM, usually because it is stuck in a retry loop. The detector hashes each prompt and checks whether the same hash appears 5 or more times in the last 20 prompts:

```rust
// crates/roko-learn/src/anomaly.rs

pub fn check_prompt(&mut self, prompt_hash: u64) -> Option<Anomaly> {
    self.prompt_hash_window.push_back(prompt_hash);
    if self.prompt_hash_window.len() > 20 { self.prompt_hash_window.pop_front(); }
    let repeated_count = self.prompt_hash_window.iter()
        .filter(|&&hash| hash == prompt_hash).count();
    if repeated_count >= 5 { Some(Anomaly::PromptLoop { repeated_count }) }
    else { None }
}
```

### 10.3 Cost Spike Detection (EWMA)

Exponentially Weighted Moving Average (EWMA) tracks the rolling mean and variance of cost per request. A new observation that is more than 3 standard deviations above the mean (z-score > 3.0) is flagged as a cost spike.

```rust
// crates/roko-learn/src/anomaly.rs

pub struct EwmaState {
    pub mean: f64,
    pub variance: f64,
    alpha: f64,  // Smoothing factor, default 0.2
}

pub fn check_cost(&mut self, cost_usd: f64) -> Option<Anomaly> {
    let z_score = self.cost_ewma.z_score(cost_usd);
    self.cost_ewma.update(cost_usd);  // Update AFTER comparison
    self.session_cost_usd += cost_usd;
    if z_score > 3.0 { Some(Anomaly::CostSpike { z_score }) }
    else { None }
}
```

The EWMA update formula is:

```
mean_{t+1} = alpha * x_t + (1 - alpha) * mean_t
variance_{t+1} = alpha * (x_t - mean_{t+1})^2 + (1 - alpha) * variance_t
```

The z-score is computed **before** the update, so a sudden spike is detected against the pre-spike baseline rather than being immediately folded in.

### 10.4 Quality Degradation Detection

The detector compares the average quality of the most recent 5 observations against the 10 observations before them. If the average drops by more than 0.15 and the recent average falls below 0.5, it flags sustained quality degradation.

---

## 11. Bayesian Confidence (Beta-Binomial)

For Stage 2 of the cascade and for standalone confidence tracking, roko provides a Bayesian updater using the conjugate Beta-Binomial model.

From `crates/roko-learn/src/bayesian_confidence.rs`:

```rust
// crates/roko-learn/src/bayesian_confidence.rs

pub struct BayesianConfidenceUpdater {
    pub alpha: f64,       // Pseudo-count of successes + prior
    pub beta: f64,        // Pseudo-count of failures + prior
    pub observations: u64,
    pub label: Option<String>,
}

impl BayesianConfidenceUpdater {
    pub fn uniform() -> Self {
        Self { alpha: 1.0, beta: 1.0, observations: 0, label: None }
    }

    pub fn confidence(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)  // Posterior mean
    }

    pub fn with_informative_prior(prior_confidence: f64, strength: f64) -> Self {
        let p = prior_confidence.clamp(0.01, 0.99);
        let s = strength.max(0.1);
        Self { alpha: p * s, beta: (1.0 - p) * s, observations: 0, label: None }
    }
}
```

**Why Beta-Binomial?** The Beta distribution is the conjugate prior for the Bernoulli likelihood (success/failure observations). This means the posterior after observing `s` successes and `f` failures from a Beta(a, b) prior is simply Beta(a + s, b + f) -- no numerical integration or MCMC needed. The posterior mean `alpha / (alpha + beta)` gives the expected success probability, and the posterior variance `alpha * beta / ((alpha + beta)^2 * (alpha + beta + 1))` quantifies uncertainty.

The informative prior constructor lets you encode prior beliefs: `with_informative_prior(0.8, 10.0)` creates a Beta(8.0, 2.0) prior centered at 0.8 with the weight of 10 pseudo-observations.

---

## 12. Episode Logging and Routing Audit Trail

Every routing decision is logged to an append-only JSONL file for offline analysis and debugging.

### 12.1 Routing Decision Log

From `crates/roko-learn/src/routing_log.rs`:

```rust
// crates/roko-learn/src/routing_log.rs

pub struct RoutingDecisionLog {
    pub timestamp: String,
    pub trace_id: String,
    pub task_id: String,
    pub requested_model: String,
    pub role: String,
    pub task_complexity: String,
    pub task_category: String,
    pub selected_provider: String,
    pub selected_model: String,
    pub routing_stage: String,
    pub routing_reason: String,
    pub candidates: Vec<CandidateEntry>,
    pub outcome_success: Option<bool>,
    pub outcome_cost_usd: Option<f64>,
    pub outcome_latency_ms: Option<u64>,
}

pub struct CandidateEntry {
    pub model: String,
    pub provider: String,
    pub score: f64,
    pub disqualified: Option<String>,
}
```

Each log entry captures the full decision context: what was requested, what was selected, why, what alternatives were considered (with their scores), and (backfilled after task completion) what the outcome was.

### 12.2 Candidate Scoring in the Log

The cascade router also produces `CascadeCandidateScore` entries for observability:

```rust
// crates/roko-learn/src/cascade/types.rs

pub struct CascadeCandidateScore {
    pub slug: String,
    pub score: f64,
    pub selected: bool,
    pub on_pareto_frontier: bool,
}
```

The `on_pareto_frontier` flag lets operators quickly see whether a selected model is Pareto-optimal or whether the bandit chose a dominated model (which should only happen during exploration).

---

## 13. Latency Tracking

From `crates/roko-learn/src/latency.rs`:

```rust
// crates/roko-learn/src/latency.rs

pub struct LatencyStats {
    pub model_slug: String,
    pub provider_id: String,
    pub ttft_ema_ms: f64,            // Time to first token (EMA)
    pub total_latency_ema_ms: f64,   // Total response time (EMA)
    pub tokens_per_second_ema: f64,  // Output throughput (EMA)
    pub observations: u64,
    pub recent_latencies: VecDeque<f64>,  // Last 100 for percentiles
}

impl LatencyStats {
    pub fn record(&mut self, ttft_ms: f64, total_ms: f64, output_tokens: u64) {
        let alpha = 0.1;
        self.ttft_ema_ms = alpha * ttft_ms + (1.0 - alpha) * self.ttft_ema_ms;
        self.total_latency_ema_ms = alpha * total_ms + (1.0 - alpha) * self.total_latency_ema_ms;
        if total_ms > 0.0 && output_tokens > 0 {
            let tps = output_tokens as f64 / (total_ms / 1000.0);
            self.tokens_per_second_ema = alpha * tps + (1.0 - alpha) * self.tokens_per_second_ema;
        }
        self.observations += 1;
        self.recent_latencies.push_back(total_ms);
        if self.recent_latencies.len() > 100 { self.recent_latencies.pop_front(); }
    }
}
```

Three metrics are tracked per (model, provider) pair:

- **TTFT** (time to first token): How quickly the model starts responding. Critical for interactive use.
- **Total latency**: End-to-end response time including all tokens.
- **Throughput** (tokens/second): Output generation speed.

All three use EMA with alpha=0.1 (recent observations weighted 10%, history weighted 90%), creating a smooth rolling estimate that adapts to changing conditions.

The `recent_latencies` buffer (last 100 observations) enables percentile computation:

```rust
pub fn p50_ms(&self) -> f64 { self.percentile(0.50) }
pub fn p95_ms(&self) -> f64 { self.percentile(0.95) }
pub fn p99_ms(&self) -> f64 { self.percentile(0.99) }

pub fn adaptive_timeout_ms(&self) -> u64 {
    if self.observations < 10 { return 120_000; }
    let timeout = (self.p95_ms() * 2.0) as u64;
    timeout.clamp(5_000, 300_000)
}
```

The adaptive timeout (2x the p95 latency, clamped to 5-300 seconds) prevents both overly aggressive timeouts (which cause spurious retries) and overly generous timeouts (which waste time waiting for a hung provider).

---

## 14. Active Inference Tier Selection

Roko includes an experimental (not yet wired to runtime) active inference module for tier routing, inspired by Friston's free energy principle (Friston, Kilner, and Harrison, 2006 [8]).

From `crates/roko-learn/src/active_inference.rs`:

```rust
// crates/roko-learn/src/active_inference.rs

pub struct BeliefState {
    pub probabilities: Vec<f64>,  // Flattened 3 x 3 x 10 = 90 latent states
    pub updates: u64,
}

pub fn select_tier(belief: &BeliefState, requirements: &TaskRequirements) -> ModelTier {
    let task_difficulty = task_difficulty(requirements);
    if task_difficulty >= 2 { return ModelTier::Premium; }
    let tiers = [ModelTier::Fast, ModelTier::Standard, ModelTier::Premium];
    tiers.into_iter()
        .min_by(|left, right| {
            expected_free_energy(belief, *left, task_difficulty)
                .partial_cmp(&expected_free_energy(belief, *right, task_difficulty))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(ModelTier::Standard)
}
```

The belief state is a factorized probability distribution over `3 x 3 x 10 = 90` latent states (3 difficulty levels x 3 skill levels x 10 confidence levels). The tier selector minimizes **expected free energy** -- a quantity that balances exploitation (preferring tiers that are expected to succeed) with epistemic foraging (preferring tiers that would resolve uncertainty in the belief state).

For hard tasks (`task_difficulty >= 2`), the system short-circuits to `ModelTier::Premium` without computing free energy.

---

## 15. Worked Example: Learning to Route Different Query Types

To make the system's behavior concrete, consider an IronClaw deployment with three models configured:

- **Haiku** (cheap, fast, moderate quality): $0.25/M input, $1.25/M output
- **Sonnet** (balanced): $3/M input, $15/M output
- **Opus** (expensive, slow, highest quality): $15/M input, $75/M output

### 15.1 Cold Start (Observations 0-50): Static Stage

The router has no data. It uses the static role table:

```
Request: "What time is it in Tokyo?"
Context: category=Implementation, complexity=Simple, role=Implementer
Stage: Static (0 observations)
Selected: Sonnet (default for Implementer role)
```

Every request goes to Sonnet regardless of complexity. The system is learning nothing yet, but it is accumulating observations that will fuel Stages 2 and 3.

### 15.2 Building Confidence (Observations 50-200): Confidence Stage

After 50 observations, the router has per-model stats:

```
Haiku:  30 tasks, 25 successes, 5 failures  -> pass_rate = 0.83
Sonnet: 15 tasks, 14 successes, 1 failure   -> pass_rate = 0.93
Opus:    5 tasks,  5 successes, 0 failures  -> pass_rate = 1.00
```

But Opus has only 5 observations -- its high pass rate has low confidence. The Wilson lower bound for Opus at 95% confidence is about 0.57, while Sonnet's lower bound is 0.71. The router prefers Sonnet over Opus for most tasks because it is more confident in Sonnet's quality.

```
Request: "Refactor the auth module to use async/await"
Context: category=Refactoring, complexity=Complex
Stage: Confidence (80 observations)
Selected: Sonnet (highest confident pass rate for complex tasks)
```

### 15.3 Contextual Learning (Observations 200+): UCB Stage

At 200+ observations, the bandit has learned context-dependent patterns. The 18D context vector now matters. For example:

**Simple greeting routed to Haiku:**

```
Request: "Hi, what's the weather?"
Context vector: [0,0,0,0,0,1,0,0, 0.0, 0.1, ...] (category=Documentation, complexity=Simple)
Bandit scores:
  Haiku:  exploitation=0.91, exploration=0.04, total=0.95
  Sonnet: exploitation=0.93, exploration=0.02, total=0.95
  Opus:   exploitation=0.95, exploration=0.01, total=0.96
Alpha is 0.09, so exploration bonuses are small.
But Haiku's reward includes cost savings: reward = 0.5*0.95 + 0.3*0.97 + 0.2*0.90 = 0.95
     Sonnet's cost-penalized reward:       reward = 0.5*0.97 + 0.3*0.70 + 0.2*0.85 = 0.87
Selected: Haiku (higher composite reward because cost weight favors it for simple tasks)
```

**Complex code refactor routed to Opus:**

```
Request: "Redesign the database layer to support both PostgreSQL and libSQL"
Context vector: [0,0,0,0,0,0,1,0, 1.0, 0.3, ...] (category=Refactoring, complexity=Complex)
Bandit scores:
  Haiku:  exploitation=0.35, exploration=0.06, total=0.41
  Sonnet: exploitation=0.72, exploration=0.04, total=0.76
  Opus:   exploitation=0.88, exploration=0.03, total=0.91
The bandit has learned that complex refactoring tasks fail often with Haiku (low exploitation score).
Selected: Opus (highest score despite highest cost, because quality dominates for complex tasks)
```

### 15.4 The Learning Signal

After each task completes, the bandit receives a reward:

```
Task completed: "Redesign the database layer..."
  Selected: Opus
  Outcome: success, cost=$0.12, latency=8200ms, SLA=15000ms
  Reward = 0.5 * 1.0 + 0.3 * (1 - 0.12/75*1e6) + 0.2 * (1 - 8200/15000)
         = 0.5 + 0.3 * ~1.0 + 0.2 * 0.45
         = 0.5 + 0.30 + 0.09 = 0.89
```

This reward is used to update Opus's arm state: `A_opus = A_opus + x * x^T`, `b_opus = b_opus + 0.89 * x`. The next time a similar context vector appears, Opus's exploitation score will reflect this positive outcome.

### 15.5 Convergence

After approximately 500 tasks, the router has converged to a stable policy:

| Task Type | Typical Model | Cost Reduction vs Always-Sonnet |
|:----------|:-------------|:-------------------------------|
| Greetings, time queries | Haiku | ~90% |
| Simple code edits | Haiku or Sonnet | ~50% |
| Standard implementation | Sonnet | 0% (baseline) |
| Code review | Sonnet | 0% |
| Complex refactoring | Opus | -400% (more expensive, but higher success rate means fewer retries) |
| Architecture decisions | Opus | -400% |

The net effect is approximately 40% cost reduction with equivalent or better task success rates.

---

## 16. IronClaw Integration Plan

### 16.1 Relationship to Existing SmartRoutingProvider

IronClaw already has a complexity-based routing system in `crates/ironclaw_llm/src/smart_routing.rs`. The `SmartRoutingProvider` scores prompts across 13 dimensions (reasoning words, token estimate, code indicators, multi-step, domain-specific, ambiguity, creativity, precision, context dependency, tool likelihood, safety requirements, conversation history, output format complexity) and routes to cheap vs primary models based on fixed score thresholds.

The bandit-based system **replaces the fixed thresholds** with learned thresholds while keeping the complexity scorer as a feature input. The 13 scoring dimensions from SmartRoutingProvider become part of the context vector, and the bandit learns the optimal threshold for each dimension rather than using hand-tuned cutoffs.

### 16.2 Module Layout

New module: `crates/ironclaw_llm/src/routing/`

```
crates/ironclaw_llm/src/routing/
    mod.rs              # CascadeRouter, RoutingContext, CascadeModel public API
    linucb.rs           # LinUCB contextual bandit (port from roko-learn/model_router.rs)
    features.rs         # RoutingContext -> feature vector encoding
    cascade.rs          # 3-stage cascade logic (port from roko-learn/cascade_router.rs)
    pareto.rs           # Pareto frontier computation (port from roko-learn/pareto.rs)
    health.rs           # Provider health circuit breaker (port from roko-learn/provider_health.rs)
    anomaly.rs          # EWMA anomaly detection (port from roko-learn/anomaly.rs)
    reward.rs           # Reward signal composition
    latency.rs          # Rolling latency tracking (port from roko-learn/latency.rs)
    episode.rs          # Episode logging (adapted from roko-learn/routing_log.rs)
    rules.rs            # Static routing rules for Stage 1
    persist.rs          # JSON-based state persistence
```

### 16.3 IronClaw RoutingContext

Adapted for IronClaw's domain (no agent roles or crate familiarity -- IronClaw is a single-agent system):

```rust
/// Context for routing decisions in IronClaw.
pub struct RoutingContext {
    // Encoded in the feature vector:
    pub task_type: TaskType,            // Chat, CodeGen, Analysis, Search, ToolUse
    pub estimated_complexity: f32,      // From SmartRoutingProvider's 13D scorer
    pub conversation_turn: u32,        // Turn number in conversation
    pub tool_count: u32,               // Number of tools in the active registry
    pub error_rate_recent: f32,        // Recent error rate for this session
    pub channel: ChannelType,          // CLI, Web, Telegram, etc.
    pub sensitivity: Sensitivity,      // From safety layer analysis
    pub budget_remaining: f32,         // Cost budget remaining fraction

    // Used by cascade logic but not in feature vector:
    pub user_tier: UserTier,           // From user profile
    pub session_quality_so_far: f32,   // Running session quality
    pub previous_model: Option<String>, // For cache affinity
}
```

IronClaw's feature vector would be smaller than roko's 18D (likely 12-14D) because IronClaw is single-agent (no role hashing) and doesn't have crate familiarity tracking. The `SmartRoutingProvider`'s 13 complexity dimensions could be compressed into a single score or kept as raw features, depending on convergence speed in testing.

### 16.4 CascadeRouter as an LlmProvider Decorator

The cascade router integrates as a decorator in IronClaw's existing provider chain, following the same pattern as `RetryProvider`, `CircuitBreakerProvider`, and `SmartRoutingProvider`:

```rust
/// Cascade-routing LLM provider decorator.
///
/// Wraps multiple LlmProvider instances and routes requests to the best
/// model based on contextual bandit learning.
pub struct CascadeRoutingProvider {
    /// Available providers, keyed by model slug.
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    /// The cascade router that makes selection decisions.
    router: Arc<CascadeRouter>,
    /// Fallback: statically configured default provider.
    default_provider: Arc<dyn LlmProvider>,
}

#[async_trait]
impl LlmProvider for CascadeRoutingProvider {
    fn model_name(&self) -> &str {
        self.default_provider.model_name()
    }

    fn cost_per_token(&self) -> (Decimal, Decimal) {
        self.default_provider.cost_per_token()
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        let context = RoutingContext::from_request(&request);

        // Router selects model + fallback chain
        let cascade = match self.router.select_model_safe(&context) {
            Some(c) => c,
            None => return self.default_provider.complete(request).await,
        };

        // Try primary, fall back on error
        let mut last_error = None;
        for attempt in 0..cascade.max_attempts() {
            let model = match cascade.model_for_attempt(attempt) {
                Some(m) => m,
                None => break,
            };
            let provider = match self.providers.get(&model.slug) {
                Some(p) => p,
                None => continue,
            };
            match provider.complete(request.clone()).await {
                Ok(response) => {
                    self.router.observe_success(&cascade, attempt, &response);
                    return Ok(response);
                }
                Err(e) => {
                    self.router.observe_failure(&cascade, attempt, &e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or(LlmError::RequestFailed {
            message: "all cascade candidates failed".into(),
        }))
    }

    // complete_with_tools follows the same pattern
}
```

### 16.5 Placement in the Provider Chain

The cascade router sits **outside** the per-provider retry and circuit breaker layers but **inside** the response cache:

```
CascadeRoutingProvider          <-- NEW: selects which provider to use
  |-- Provider A (e.g., Anthropic Claude Sonnet)
  |     |-- RetryProvider
  |     |-- CircuitBreakerProvider
  |-- Provider B (e.g., OpenAI GPT-4o-mini)
  |     |-- RetryProvider
  |     |-- CircuitBreakerProvider
  |-- Provider C (e.g., NEAR AI)
        |-- RetryProvider
        |-- CircuitBreakerProvider
```

Each individual provider retains its own retry and circuit breaker logic. The cascade router operates at a higher level, choosing among providers and learning from outcomes.

### 16.6 State Persistence

Router state persists to `~/.ironclaw/learn/`:

```
~/.ironclaw/learn/
  cascade-router.json    # LinUCB arm states, confidence stats, stage tracking
  provider-health.json   # Circuit breaker states
  latency-registry.json  # Per-model/provider latency stats
  routing-decisions.jsonl # Append-only routing audit trail
```

All persistence is debounced (batch writes every 100ms) and uses atomic write-rename for crash safety, following roko's pattern.

### 16.7 Database Schema for Episode Logging

```sql
CREATE TABLE model_routing_episodes (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    session_id TEXT NOT NULL,
    -- Decision context
    context_vector JSONB NOT NULL,
    routing_stage TEXT NOT NULL,        -- "static", "confidence", "ucb"
    routing_reason TEXT,
    -- Selection
    requested_model TEXT,
    selected_model TEXT NOT NULL,
    selected_provider TEXT NOT NULL,
    candidates JSONB,
    -- Outcome (backfilled after task completion)
    reward REAL,
    success BOOLEAN,
    latency_ms INTEGER,
    input_tokens INTEGER,
    output_tokens INTEGER,
    cost_usd REAL,
    -- Anomaly flags
    anomaly_type TEXT,
    anomaly_score REAL
);

CREATE INDEX idx_routing_episodes_timestamp ON model_routing_episodes (timestamp);
CREATE INDEX idx_routing_episodes_model ON model_routing_episodes (selected_model);
CREATE INDEX idx_routing_episodes_session ON model_routing_episodes (session_id);
```

Both PostgreSQL and libSQL backends must be supported per IronClaw's dual-backend requirement (`src/db/CLAUDE.md`).

### 16.8 Graceful Degradation

If the router encounters any error (corrupted state, computation failure, empty candidate set), it falls back to the current behavior: use the statically configured model. The router is **additive** -- it can only improve routing, never make it worse than the status quo.

```rust
pub fn select_model_safe(&self, context: &RoutingContext) -> Option<CascadeModel> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        self.select_model(context)
    })) {
        Ok(cascade) => Some(cascade),
        Err(_) => {
            tracing::error!("cascade router panic; falling back to default model");
            None
        }
    }
}
```

### 16.9 Complexity Assessment

| Component | Estimated LOC | Complexity | Source Reference |
|:----------|:-------------|:-----------|:----------------|
| LinUCB core | 400-500 | Medium | `roko-learn/src/model_router.rs` |
| Cascade router | 500-600 | Medium-High | `roko-learn/src/cascade_router.rs` + `cascade/` |
| Feature encoding | 200-250 | Low | `roko-learn/src/model_router.rs` (RoutingContext) |
| Pareto frontier | 150-200 | Low | `roko-learn/src/pareto.rs` |
| Provider health | 300-400 | Medium | `roko-learn/src/provider_health.rs` |
| Anomaly detection | 200-250 | Low | `roko-learn/src/anomaly.rs` |
| Latency tracking | 200-250 | Low | `roko-learn/src/latency.rs` |
| Reward computation | 100-150 | Low | `roko-learn/src/model_router.rs` |
| Episode logging | 200-250 | Low | `roko-learn/src/routing_log.rs` |
| LLM integration | 400-500 | Medium | Wiring into `ironclaw_llm` provider chain |
| **Total** | **2700-3350** | | |

### 16.10 Dependencies

- `parking_lot` -- for RwLock/Mutex (IronClaw already uses this)
- `serde` + `serde_json` -- for state persistence (already used)
- `chrono` -- for timestamps (already used)
- `rand` -- for Thompson sampling beta distribution sampling (already used)

No external linear algebra library is needed. The 18x18 (or smaller) matrix operations (Cholesky decomposition, matrix-vector multiply) are implemented inline, as roko demonstrates this is simpler and more predictable than pulling in `nalgebra` for such a small dimension.

### 16.11 Risk Assessment

- **Low risk**: Graceful fallback to current behavior if the bandit is uncertain or encounters errors. The system is strictly additive.
- **Medium risk**: Feature encoding must match IronClaw's task taxonomy, not roko's. IronClaw has no `TaskCategory` enum with roko's 8 variants; a mapping from SmartRoutingProvider's `Tier` to a smaller category set is needed.
- **Low risk**: All state is optional -- the system works without any persisted learning state (it just starts from scratch with the static stage).
- **Low risk**: IronClaw already has `CircuitBreakerProvider` (`crates/ironclaw_llm/src/circuit_breaker.rs`) with a compatible Closed/Open/HalfOpen state machine. The routing-layer health tracker can delegate to or complement the existing per-provider circuit breaker.
- **Observation**: Roko's experience shows convergence within 200-500 observations per task type. A moderately active IronClaw deployment (50-100 requests/day) would reach Stage 3 within 2-5 days.

---

## 17. References

[1] Auer, P., Cesa-Bianchi, N., and Fischer, P. (2002). "Finite-time Analysis of the Multiarmed Bandit Problem." *Machine Learning*, 47, 235-256. DOI: [10.1023/A:1013689704352](https://doi.org/10.1023/A:1013689704352). Introduced the UCB1 algorithm achieving logarithmic cumulative regret for rewards in [0, 1].

[2] Li, L., Chu, W., Langford, J., and Schapire, R. E. (2010). "A Contextual-Bandit Approach to Personalized News Article Recommendation." In *Proceedings of the 19th International Conference on World Wide Web (WWW '10)*, pp. 661-670. DOI: [10.1145/1772690.1772758](https://doi.org/10.1145/1772690.1772758). arXiv: [1003.0146](https://arxiv.org/abs/1003.0146). Introduced the LinUCB algorithm with disjoint linear models per arm. The algorithm boosted Yahoo! News click-through rates by 12.5%.

[3] Thompson, W. R. (1933). "On the Likelihood that One Unknown Probability Exceeds Another in View of the Evidence of Two Samples." *Biometrika*, 25(3/4), 285-294. DOI: [10.1093/biomet/25.3-4.285](https://doi.org/10.1093/biomet/25.3-4.285). Original paper introducing Thompson sampling (probability matching) for sequential decision-making.

[4] Garivier, A. and Kaufmann, E. (2016). "Optimal Best Arm Identification with Fixed Confidence." In *Proceedings of the 29th Annual Conference on Learning Theory (COLT)*. arXiv: [1602.04589](https://arxiv.org/abs/1602.04589). Introduced the Track-and-Stop algorithm for best-arm identification with asymptotically optimal sample complexity.

[5] Abbasi-Yadkori, Y., Pal, D., and Szepesvari, C. (2011). "Improved Algorithms for Linear Stochastic Bandits." In *Advances in Neural Information Processing Systems (NeurIPS)*, 24. Established the theoretical foundation for confidence ellipsoids in linear bandits that underpin LinUCB's exploration bonus.

[6] Deb, K. (2001). *Multi-Objective Optimization Using Evolutionary Algorithms.* Wiley, Chichester. Foundational reference for Pareto optimality and multi-objective optimization, defining dominated and non-dominated solutions.

[7] Nygard, M. T. (2007). *Release It! Design and Deploy Production-Ready Software.* Pragmatic Bookshelf. Popularized the circuit breaker pattern for resilient distributed systems. The pattern was later implemented in Netflix's Hystrix (2012) and is now standard practice.

[8] Friston, K., Kilner, J., and Harrison, L. (2006). "A Free Energy Principle for the Brain." *Journal of Physiology-Paris*, 100(1-3), 70-87. Introduced the free energy principle, the theoretical basis for active inference, which roko adapts for tier selection by minimizing expected free energy over a belief state.

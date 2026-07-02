# Conductor: Anomaly Detection & Circuit Breaking

**Source crate**: `roko-conductor` (`crates/roko-conductor/src/`)
**Total source**: ~10,100 lines of Rust across 24 files (roughly 50% implementation, 50% tests)
**Priority**: MEDIUM -- production-grade system health monitoring for LLM providers
**Roko README**: `crates/roko-conductor/README.md`

---

## Table of Contents

1. [Why Anomaly Detection Matters for Production AI Systems](#1-why-anomaly-detection-matters-for-production-ai-systems)
2. [What a Conductor Does for an AI Agent](#2-what-a-conductor-does-for-an-ai-agent)
3. [Architecture Overview](#3-architecture-overview)
4. [The React Trait and Engram Signal Model](#4-the-react-trait-and-engram-signal-model)
5. [The 10 Watchers -- Complete Reference](#5-the-10-watchers----complete-reference)
6. [Severity Classification and Intervention Policies](#6-severity-classification-and-intervention-policies)
7. [Circuit Breaker and Predictive Tripping](#7-circuit-breaker-and-predictive-tripping)
8. [Holt Exponential Smoothing -- Full Mathematical Treatment](#8-holt-exponential-smoothing----full-mathematical-treatment)
9. [Compound Pattern Detection (CEP)](#9-compound-pattern-detection-cep)
10. [Adaptive Threshold Learning](#10-adaptive-threshold-learning)
11. [Thompson Sampling via BanditPolicy](#11-thompson-sampling-via-banditpolicy)
12. [Yerkes-Dodson Pressure Framework](#12-yerkes-dodson-pressure-framework)
13. [Federation: 4-Level Conductor Hierarchy](#13-federation-4-level-conductor-hierarchy)
14. [Self-Healing with Oscillation Detection](#14-self-healing-with-oscillation-detection)
15. [Diagnosis Engine](#15-diagnosis-engine)
16. [Health Monitor](#16-health-monitor)
17. [Stuck Detection and Meta-Cognition](#17-stuck-detection-and-meta-cognition)
18. [Routing Bias and Provider Health](#18-routing-bias-and-provider-health)
19. [Cognitive Signals](#19-cognitive-signals)
20. [IronClaw Integration Plan](#20-ironclaw-integration-plan)
21. [Complexity Assessment](#21-complexity-assessment)
22. [References](#22-references)

---

## 1. Why Anomaly Detection Matters for Production AI Systems

Production AI systems that interact with LLM providers face a class of failure modes that traditional monitoring does not handle. LLM calls are expensive ($0.01--$1.00+ per request), non-deterministic, and subject to provider-side degradation that manifests gradually rather than as a binary up/down signal. A system that does not detect these anomalies will:

- **Burn money on ghost turns**: An agent can enter a loop where it produces output that looks active but makes zero progress -- no file changes, no test improvements, no meaningful work. Each turn costs real dollars. Without detection, the system burns through budgets on wasted computation.

  *Real-world example*: An agent is asked to fix a compile error. It reads the error, generates a plan, reads the file, generates another plan, reads the file again -- consuming 3 turns at $0.50 each without ever writing a fix. The GhostTurnWatcher detects this after 3 consecutive empty turns and triggers a restart with a different strategy.

- **Thrash between providers**: If provider A degrades and the system switches to provider B, B may become overloaded, causing the system to switch back to A, creating an oscillation loop that never stabilizes.

  *Real-world example*: OpenAI returns 429 rate-limit errors, so the system switches to Anthropic. Anthropic's latency spikes under increased load, so the system switches back to OpenAI, which is still rate-limited. The self-healing oscillation detector breaks this cycle after 5 alternations by entering a cooldown period.

- **Miss compound failures**: Individual metrics may stay within bounds while the system is failing. Latency is fine, error rate is fine, but latency is up AND error rate is creeping AND costs are rising -- together, these signal a degradation that no single metric captures.

  *Real-world example*: Cost is at 85% of budget (below the overrun threshold), time is at 75% of timeout (below the alert threshold), and context window is at 70% utilization (below the pressure threshold). Individually, each is fine. But the CEP pattern detector recognizes that all three resource watchers are trending upward simultaneously and fires a `total_resource_exhaustion` compound pattern at Critical severity.

- **React too late**: Traditional circuit breakers trip after N failures. By that point, the system has already consumed the budget for N failed attempts. Predictive circuit breaking trips before the Nth failure by forecasting the trend.

  *Real-world example*: After 1 success and 2 consecutive failures, the Holt forecaster projects that the error rate will exceed 0.5 on the next step. The circuit trips proactively, saving the $2-5 cost of a third failed attempt that was almost certainly going to fail.

The conductor addresses all of these. It is a **purely reactive** layer: it reads signal streams, produces intervention decisions, and has no side effects. The orchestrator feeds it data; the conductor tells the orchestrator what to do.

> **Roko design principle** (from `crates/roko-conductor/src/lib.rs`, lines 14-15):
> "Every watcher is a pure function: `&[Engram] -> Vec<Engram>`. Watchers have no side effects."

---

## 2. What a Conductor Does for an AI Agent

An AI agent -- whether IronClaw's single-user personal assistant or roko's multi-agent development platform -- executes in a loop: receive input, call an LLM, execute tools, evaluate the result, repeat. The **conductor** is a supervisory layer that observes this loop from outside and decides when to intervene.

Think of it as an orchestra conductor: it does not play any instrument (no side effects), but it watches all the players (watchers), detects when the performance is going off-track (anomaly detection), and signals corrections (intervention decisions). Specifically:

1. **Signal collection**: After each agent turn, the orchestrator packages runtime observations (token usage, cost, timing, compile results, test results, file changes) into a stream of `Engram` signals.

2. **Anomaly detection**: Ten specialized watchers scan the signal stream for specific failure patterns -- loops, regressions, resource exhaustion, stuck behavior.

3. **Severity classification**: Each detected anomaly is classified as Info (log it), Warning (restart the current approach), or Critical (abort the plan entirely).

4. **Intervention policy**: A policy layer merges all watcher outputs into a single `ConductorDecision` -- Continue, Restart, or Fail.

5. **Predictive circuit breaking**: A per-plan circuit breaker uses Holt exponential smoothing to forecast error rates and trip proactively before the budget is wasted.

6. **Compound pattern detection**: A CEP-inspired detector identifies multi-signal anomalies that no individual watcher can see (e.g., quality AND resource AND progress all degrading simultaneously).

7. **Adaptive learning**: The system learns from intervention outcomes -- if a restart worked, it lowers the threshold for future interventions; if it did not help, it raises it.

8. **Routing guidance**: The conductor emits `RoutingBias` signals telling the orchestrator which providers to deprioritize and whether to prefer cheaper models.

---

## 3. Architecture Overview

The conductor sits between the orchestrator and the running agents. After each agent turn, the orchestrator feeds a stream of `Engram` signals to the conductor. The conductor runs all watchers, applies the intervention policy, and returns a `ConductorDecision` that the orchestrator acts on.

```
                              +-----------------------------+
                              |       Orchestrator          |
                              |                             |
                              |   feed signal stream        |
                              |          |                  |
                              |          v                  |
                              |    +--------------+         |
                              |    |  Conductor    |         |
                              |    |              |         |
                              |    |  +---------+ |         |
                              |    |  | Watcher1| |         |
                              |    |  | Watcher2| |         |
                              |    |  | ...     | |         |
                              |    |  |Watcher10| |         |
                              |    |  +---------+ |         |
                              |    |       |      |         |
                              |    |       v      |         |
                              |    |  Policy      |         |
                              |    |  + CB        |         |
                              |    |  + CEP       |         |
                              |    |  + Learning  |         |
                              |    +------+-------+         |
                              |           |                 |
                              |           v                 |
                              |    ConductorDecision        |
                              |    + CognitiveSignals       |
                              |    + RoutingBias            |
                              +-----------------------------+
```

The `Conductor` struct from `crates/roko-conductor/src/conductor.rs` (lines 60-78) holds all the state:

```rust
// Source: crates/roko-conductor/src/conductor.rs, lines 60-78
pub struct Conductor {
    /// The individual watchers, stored as boxed `React` impls.
    watchers: Vec<Box<dyn React>>,
    /// Intervention escalation policy.
    policy: Box<dyn InterventionPolicy>,
    /// Per-plan circuit breaker.
    circuit_breaker: CircuitBreaker,
    /// Most recent routing bias derived from the live signal stream.
    routing_bias: Mutex<RoutingBias>,
    /// Per-provider health tracker for routing decisions (COND-09).
    provider_health: Option<Arc<ProviderHealthTracker>>,
    /// Adaptive threshold learner (COND-03).
    threshold_learner: Mutex<ThresholdLearner>,
    /// CEP-inspired compound pattern detector (COND-07).
    pattern_detector: Mutex<PatternDetector>,
    /// Most recently detected compound patterns from the last evaluate() call.
    last_compound_patterns: Mutex<Vec<CompoundPattern>>,
}
```

The default constructor registers all 10 watchers:

```rust
// Source: crates/roko-conductor/src/conductor.rs, lines 95-108
fn default_watchers() -> Vec<Box<dyn React>> {
    vec![
        Box::new(GhostTurnWatcher::default()),
        Box::new(ReviewLoopWatcher::default()),
        Box::new(IterationLoopWatcher::default()),
        Box::new(TestFailureBudgetWatcher::default()),
        Box::new(CompileFailRepeatWatcher::default()),
        Box::new(ContextWindowPressureWatcher::default()),
        Box::new(SpecDriftWatcher::default()),
        Box::new(CostOverrunWatcher::default()),
        Box::new(TimeOverrunWatcher::new()),
        Box::new(StuckPatternWatcher::default()),
    ]
}
```

A test in the same file confirms the count:

```rust
// Source: crates/roko-conductor/src/conductor.rs, line 859-862
#[test]
fn watcher_count() {
    let c = Conductor::default();
    assert_eq!(c.watchers.len(), 10);
}
```

---

## 4. The React Trait and Engram Signal Model

Every watcher implements the `React` trait from `roko-core`. This trait defines a single method `decide` that takes a signal stream and returns intervention signals:

```rust
// From roko-core (re-exported in roko-conductor)
pub trait React: Send + Sync {
    /// Examine the signal stream and produce intervention signals.
    fn decide(&self, stream: &[Engram], ctx: &Context) -> Vec<Engram>;

    /// Human-readable name of this reactor.
    fn name(&self) -> &str;
}
```

`Engram` is roko's universal signal type. An Engram has:
- A `Kind` (e.g., `AgentOutput`, `GateVerdict`, `PlanPhase`, `Metric`, `TokenUsage`, `CompileDiagnostic`, or `Custom(String)`)
- A `Body` (text, JSON, bytes, or empty)
- A set of key-value `tags` for metadata

Watchers scan the stream for specific `Kind` values and emit `Kind::Custom("conductor.intervention")` signals when anomalies are detected. These intervention signals carry tags for `watcher` (name), `severity` ("info", "warning", "critical"), and watcher-specific metadata.

The conductor's `collect_watcher_outputs` function (lines 547-570) runs every watcher and converts their raw `Engram` outputs into structured `WatcherOutput` values:

```rust
// Source: crates/roko-conductor/src/conductor.rs, lines 547-570
fn collect_watcher_outputs(
    watchers: &[Box<dyn React>],
    stream: &[Engram],
    ctx: &Context,
) -> Vec<WatcherOutput> {
    let mut outputs = Vec::new();
    for watcher in watchers {
        let signals = watcher.decide(stream, ctx);
        for s in &signals {
            let severity = match s.tag("severity") {
                Some("critical") => Severity::Critical,
                Some("warning") => Severity::Warning,
                _ => Severity::Info,
            };
            let watcher_name = s.tag("watcher").unwrap_or_else(|| watcher.name());
            let description = match &s.body {
                Body::Text(t) => t.clone(),
                _ => format!("intervention from {watcher_name}"),
            };
            outputs.push(WatcherOutput::new(watcher_name, severity, description));
        }
    }
    outputs
}
```

---

## 5. The 10 Watchers -- Complete Reference

Each watcher is a pure function with no side effects. They scan `&[Engram]` for specific signal patterns and emit intervention signals when anomalies are detected. All watchers have configurable thresholds.

### 5.1 GhostTurnWatcher (Progress Family)

**File**: `crates/roko-conductor/src/watchers/ghost_turn.rs` (300 lines)
**Signal kind scanned**: `Kind::Custom("conductor.ghost_turn")`
**Default threshold**: 3 consecutive ghost turns (`MAX_GHOST_TURNS`)
**Severity**: Warning

Detects agent turns that consume tokens but produce zero meaningful output. A "ghost turn" is one where `output_meaningful == false` AND `net_new_changes == 0`. The watcher counts consecutive ghost turns from the end of the stream. Any non-ghost-turn signal breaks the chain.

```rust
// Source: crates/roko-conductor/src/watchers/ghost_turn.rs, lines 19-32
#[derive(Debug, Clone, Deserialize)]
struct GhostTurnEvent {
    plan_id: String,
    task: String,
    role: String,
    model: String,
    cost_usd: f64,
    duration_ms: u64,
    changed_files_before: Vec<String>,
    changed_files_after: Vec<String>,
    net_new_changes: usize,
    output_meaningful: bool,
    wasted_cost: bool,
}
```

The intervention signal includes the model name, cost, duration, and file change counts so the orchestrator can decide whether to switch models.

*Real-world example*: An agent is working on a feature implementation. It produces three consecutive turns where it reads files, discusses the approach, and reads more files, but never writes any changes. Total cost: $1.50 in LLM calls with zero progress. The GhostTurnWatcher fires at Warning severity after the third empty turn. The orchestrator restarts the agent with a more directive prompt: "Write the implementation to src/feature.rs."

### 5.2 ReviewLoopWatcher (Progress Family)

**File**: `crates/roko-conductor/src/watchers/review_loop.rs` (228 lines)
**Signal kind scanned**: `Kind::PlanPhase` with `event == "ReviewRejected"`
**Default threshold**: 3 rejections (`MAX_REVIEW_CYCLES`)
**Severity**: Warning

Counts consecutive review rejections for the same plan. A `ReviewApproved`, `DocRevisionDone`, or `MergeSucceeded` event resets the counter. When the agent repeatedly fails review without advancing, something is fundamentally wrong with its approach and a restart is warranted.

*Real-world example*: An agent submits code for review, the reviewer rejects it for not handling edge cases. The agent adds edge case handling but introduces a regression. The reviewer rejects again. The agent fixes the regression but now violates a style guide rule. After 3 rejections, the ReviewLoopWatcher fires, signaling the orchestrator to restart with a fresh approach -- perhaps using a stronger model or a different decomposition of the task.

### 5.3 IterationLoopWatcher (Progress Family)

**File**: `crates/roko-conductor/src/watchers/iteration_loop.rs` (210 lines)
**Signal kind scanned**: `Kind::PlanPhase` with `event == "GateFailed"`
**Default threshold**: 3 gate failures (`MAX_IMPLEMENTER_ATTEMPTS`)
**Severity**: **Critical** (triggers plan failure, not just restart)

Counts gate failures for a plan. A `GatePassed`, `ImplementationDone`, `ReviewApproved`, `DocRevisionDone`, `MergeSucceeded`, or `VerifyPassed` event resets the counter. This is the only watcher that fires at Critical severity by default -- repeated gate failures mean the agent is fundamentally unable to complete the task with its current approach.

*Real-world example*: An agent attempts to implement a complex borrow-checker fix in Rust. Gate 1: compile fails due to lifetime issues. Gate 2: the fix introduces a different lifetime error. Gate 3: the fix compiles but tests fail with a use-after-free. After 3 gate failures, the IterationLoopWatcher fires at Critical severity. The conductor aborts the plan entirely, rather than just restarting -- the problem likely requires architectural rethinking, not more iterations.

### 5.4 TestFailureBudgetWatcher (Quality Family)

**File**: `crates/roko-conductor/src/watchers/test_failure_budget.rs` (201 lines)
**Signal kind scanned**: `Kind::GateVerdict` with structured test counts
**Default threshold**: 1 additional failure (`MIN_FAILURE_INCREASE`)
**Severity**: Warning

Tracks the baseline test failure count from the first `GateVerdict` signal for each plan, then compares subsequent verdicts. If the latest failure count exceeds the baseline by `min_failure_increase`, the watcher fires. This detects regressions introduced by the agent.

The watcher parses structured JSON from gate verdicts:
```json
{
    "plan_id": "plan-1",
    "gate": "test",
    "test_count": {
        "passed": 9,
        "failed": 3,
        "ignored": 0,
        "total": 12
    }
}
```

*Real-world example*: The codebase starts with 2 failing tests. The agent is asked to fix a bug. After the fix, there are now 3 failing tests -- the agent introduced a regression. The TestFailureBudgetWatcher detects the increase from 2 to 3 and fires, prompting a restart. This prevents the "fix one thing, break another" cycle that can destroy a codebase.

### 5.5 CompileFailRepeatWatcher (Quality Family)

**File**: `crates/roko-conductor/src/watchers/compile_fail_repeat.rs` (207 lines)
**Signal kind scanned**: `Kind::CompileDiagnostic`
**Default threshold**: 3 identical failures (`MAX_IDENTICAL_COMPILE_FAILURES`)
**Severity**: Warning

Detects when the same compile error appears consecutively in the stream. The watcher extracts a normalized "diagnostic key" from each `CompileDiagnostic` signal and checks whether the last N diagnostics share the same key. Non-compile signals between compile diagnostics are filtered out, so interleaved `AgentOutput` signals do not break the chain.

*Real-world example*: An agent is trying to fix `error[E0308]: mismatched types`. It changes the return type, gets the same error. It adds a type annotation, gets the same error. It tries a `.into()` call, gets the same error. After 3 identical `E0308` errors, the CompileFailRepeatWatcher fires. The orchestrator escalates to a stronger model that can reason more deeply about the type system.

### 5.6 ContextWindowPressureWatcher (Resource Family)

**File**: `crates/roko-conductor/src/watchers/context_window_pressure.rs` (382 lines)
**Signal kind scanned**: `Kind::TokenUsage`
**Default threshold**: 80% utilization (`MAX_CONTEXT_USAGE_RATIO`)
**Severity**: Warning
**Gated**: Only active when `conductor.context_pressure_enabled = true` in config

This watcher is more sophisticated than the others. It uses a lookback window of 3 (`PRESSURE_LOOKBACK`) to take the maximum utilization over recent signals, preventing noisy alternating high/low usage from firing repeatedly. It resolves context window sizes via three sources:

1. Precomputed map from `ModelProfile.context_window` config entries (passed at construction)
2. Hardcoded Anthropic models: Opus = 1,000,000 tokens; Sonnet/Haiku = 200,000 tokens
3. Falls back to `None` for unknown models (inert, does not fire)

```rust
// Source: crates/roko-conductor/src/watchers/context_window_pressure.rs, lines 190-206
fn context_window_tokens(&self, model: &str) -> Option<u64> {
    let model_lower = model.to_ascii_lowercase();
    // First: check configured model profiles.
    if let Some(&ctx) = self.configured_windows.get(&model_lower) {
        return Some(ctx);
    }
    // Fallback: hardcoded Anthropic models.
    if model_lower.contains("opus") {
        Some(OPUS_CONTEXT_WINDOW_TOKENS)   // 1_000_000
    } else if model_lower.contains("haiku") || model_lower.contains("sonnet") {
        Some(SMALL_CONTEXT_WINDOW_TOKENS)  // 200_000
    } else {
        None
    }
}
```

*Real-world example*: An agent is working on a large codebase and has read 15 files into its context. Token usage hits 85% of the 200K Sonnet context window. The ContextWindowPressureWatcher fires, emitting a `CognitiveSignal::InjectContext` signal that suggests trimming conversation history before the next turn, preventing a context overflow error.

### 5.7 SpecDriftWatcher (Quality Family)

**File**: `crates/roko-conductor/src/watchers/spec_drift.rs` (263 lines)
**Signal kind scanned**: `Kind::Metric` with `name == "spec_drift"`
**Default threshold**: 25% drift (`MAX_SPEC_DRIFT_RATIO`)
**Severity**: Warning

Monitors the ratio of files changed outside the task's declared scope. If a task declared it would write `["src/lib.rs"]` but actually changed `["src/lib.rs", "src/main.rs"]`, the drift ratio is 0.5 (50%). The watcher uses the most recent spec drift metric signal.

The watcher supports two input formats: a simple tag-based value or a structured JSON body with `plan_id`, `task_id`, `write_files`, `changed_files`, `unexpected_files`, and `drift_ratio`.

*Real-world example*: An agent is tasked with adding a new field to a struct in `src/models.rs`. Instead, it modifies `src/models.rs`, `src/api.rs`, `src/database.rs`, and `tests/integration.rs`. The drift ratio is 0.75 (3 out of 4 files are outside scope). The SpecDriftWatcher fires, alerting the orchestrator that the agent is making changes far beyond its mandate -- a sign that it may be misunderstanding the task.

### 5.8 CostOverrunWatcher (Resource Family)

**File**: `crates/roko-conductor/src/watchers/cost_overrun.rs` (170 lines)
**Signal kind scanned**: `Kind::Metric` with `name == "plan_cost"` and `name == "plan_budget"`
**Default threshold**: $10.00 fallback budget (`DEFAULT_BUDGET`)
**Severity**: Warning

Compares the most recent `plan_cost` metric against the most recent `plan_budget` metric. If no budget metric exists, falls back to a configurable default. Fires when cost exceeds budget.

*Real-world example*: A plan has a $5.00 budget. After 6 turns of unsuccessful debugging, the accumulated cost reaches $5.20. The CostOverrunWatcher fires, and the conductor emits a `Cooldown` signal suggesting the orchestrator switch to a cheaper model tier or reduce the scope of the remaining work.

### 5.9 TimeOverrunWatcher (Resource Family)

**File**: `crates/roko-conductor/src/watchers/time_overrun.rs` (199 lines)
**Signal kind scanned**: `Kind::Custom("conductor.agent_output")`
**Default threshold**: 80% of timeout (`ALERT_THRESHOLD`)
**Severity**: Warning

Checks the most recent task timing signal. If `duration_ms > timeout_secs * 1000 * 0.80`, the watcher fires. This gives the orchestrator an early warning before a task actually times out, allowing it to switch strategies proactively.

*Real-world example*: A task has a 5-minute timeout. After 4 minutes (80% of timeout), the agent is still working on the first of three subtasks. The TimeOverrunWatcher fires, giving the orchestrator 60 seconds to either switch to a faster approach, skip lower-priority subtasks, or gracefully wind down the current attempt.

### 5.10 StuckPatternWatcher (Progress Family)

**File**: `crates/roko-conductor/src/watchers/stuck_pattern.rs` (232 lines)
**Signal kind scanned**: `Kind::AgentOutput` and `Kind::AgentMessage`
**Default threshold**: 4 identical actions (`MAX_IDENTICAL_ACTIONS`)
**Severity**: Warning

Walks backward through the signal stream, counting consecutive action signals with identical body text. Non-action signals (e.g., `GateVerdict`) are skipped without breaking the chain. Different action kinds (`AgentOutput` vs `AgentMessage`) are counted together if their body text matches.

*Real-world example*: An agent gets stuck in a loop where it keeps running `cargo test`, seeing the same failure, and running `cargo test` again without making any changes in between. After 4 identical "running cargo test" actions, the StuckPatternWatcher fires. The conductor emits an `Explore` signal telling the orchestrator to try a different approach -- perhaps reading the test source code first, or trying a different fix strategy.

### Summary Table

| # | Watcher | Family | Signal Kind | Default Threshold | Severity |
|---|---------|--------|------------|-------------------|----------|
| 1 | **GhostTurn** | Progress | `conductor.ghost_turn` | 3 consecutive | Warning |
| 2 | **ReviewLoop** | Progress | `PlanPhase/ReviewRejected` | 3 rejections | Warning |
| 3 | **IterationLoop** | Progress | `PlanPhase/GateFailed` | 3 gate failures | **Critical** |
| 4 | **TestFailureBudget** | Quality | `GateVerdict` | 1 regression | Warning |
| 5 | **CompileFailRepeat** | Quality | `CompileDiagnostic` | 3 identical | Warning |
| 6 | **ContextWindowPressure** | Resource | `TokenUsage` | 80% utilization | Warning |
| 7 | **SpecDrift** | Quality | `Metric/spec_drift` | 25% drift | Warning |
| 8 | **CostOverrun** | Resource | `Metric/plan_cost` | budget exceeded | Warning |
| 9 | **TimeOverrun** | Resource | `conductor.agent_output` | 80% of timeout | Warning |
| 10 | **StuckPattern** | Progress | `AgentOutput/AgentMessage` | 4 identical | Warning |

All thresholds are configurable via `[conductor.watchers.*]` in `roko.toml`. The `configured_watchers` function in `conductor.rs` (lines 110-192) reads these overrides.

---

## 6. Severity Classification and Intervention Policies

**File**: `crates/roko-conductor/src/interventions.rs` (464 lines)

### 6.1 Three-Level Severity Model

Roko uses three severity levels that map directly to `ConductorDecision` variants:

```rust
// Source: crates/roko-conductor/src/interventions.rs, lines 31-38
pub enum Severity {
    /// Informational -- logged, no action taken.
    Info = 0,
    /// Warning -- triggers a restart of the current phase.
    Warning = 1,
    /// Critical -- triggers terminal failure.
    Critical = 2,
}
```

The mapping is:
- `Severity::Info` -> `ConductorDecision::Continue` (log but proceed)
- `Severity::Warning` -> `ConductorDecision::Restart` (restart the current phase with a new approach)
- `Severity::Critical` -> `ConductorDecision::Fail` (abort the plan entirely)

> **Roko design reference**: The three-level model is documented in `crates/roko-conductor/src/interventions.rs` (doc comment, line 4): "Roko's conductor uses a simplified 3-level intervention model (SS 11.2: Continue / Restart / Fail)."

### 6.2 WatcherOutput

Each watcher's raw `Engram` output is converted to a structured `WatcherOutput`:

```rust
// Source: crates/roko-conductor/src/interventions.rs, lines 58-67
pub struct WatcherOutput {
    /// Name of the watcher that produced this finding.
    pub watcher: String,
    /// Severity of the detected anomaly.
    pub severity: Severity,
    /// Human-readable description.
    pub description: String,
    /// Optional metric value.
    pub metric: Option<f64>,
}
```

### 6.3 InterventionPolicy Trait

The policy trait maps a batch of watcher outputs to a single decision:

```rust
// Source: crates/roko-conductor/src/interventions.rs, lines 106-112
pub trait InterventionPolicy: Send + Sync {
    fn evaluate(&self, outputs: &[WatcherOutput], ctx: &Context) -> ConductorDecision;
    fn name(&self) -> &str;
}
```

Two implementations exist:

**WorstSeverityPolicy** (default): Takes the maximum severity across all watcher outputs and uses that. If any watcher fires Critical, the decision is Fail. If any fires Warning, the decision is Restart. Otherwise, Continue.

```rust
// Source: crates/roko-conductor/src/interventions.rs, lines 118-128
impl InterventionPolicy for WorstSeverityPolicy {
    fn evaluate(&self, outputs: &[WatcherOutput], _ctx: &Context) -> ConductorDecision {
        let worst = outputs.iter().max_by_key(|o| o.severity);
        worst.map_or_else(ConductorDecision::cont, WatcherOutput::to_decision)
    }
    fn name(&self) -> &str { "worst-severity" }
}
```

**BanditPolicy** (learned): Blends Thompson Sampling with worst-severity at a 65/35 ratio after a 50-observation warmup. See section 11 for details.

---

## 7. Circuit Breaker and Predictive Tripping

**File**: `crates/roko-conductor/src/circuit_breaker.rs` (699 lines)

### 7.1 Background: The Circuit Breaker Pattern

The circuit breaker pattern was introduced by Michael Nygard in *Release It! Design and Deploy Production-Ready Software* (Nygard, 2007) [1]. Originally inspired by electrical circuit breakers that prevent overloads from causing fires, the software pattern prevents a system from repeatedly calling a failing service. The canonical state machine has three states:

- **Closed** (normal operation): requests flow through; failures are counted
- **Open** (tripped): requests are immediately rejected without calling the downstream service
- **Half-Open** (testing recovery): a limited number of requests are allowed through to test if the service has recovered

Netflix's Hystrix library popularized the pattern for microservices, though it entered maintenance mode in 2018 and was succeeded by Resilience4j [2].

### 7.2 Roko's Simplification: Per-Plan Failure Budget

Roko simplifies the three-state model to a count-based approach with Holt forecasting layered on top. The circuit breaker tracks failures per plan using `DashMap` for lock-free concurrent access. The default threshold is 2 failures (`MAX_PLAN_FAILURES`). When a plan accumulates enough failures, the circuit "trips" and the plan is aborted on the next evaluation.

```rust
// Source: crates/roko-conductor/src/circuit_breaker.rs, lines 147-161
pub struct CircuitBreaker {
    /// Maximum failures before tripping.
    max_failures: u32,
    /// Per-plan failure records.
    records: DashMap<String, FailureRecord>,
    /// Per-plan Holt forecasters for predictive tripping (COND-08).
    forecasters: DashMap<String, HoltForecaster>,
    /// Per-plan total evaluation count for error rate.
    eval_counts: DashMap<String, (u32, u32)>,
    /// Whether predictive mode is enabled.
    predictive: bool,
    /// Trip threshold for the forecasted error rate (default: 0.5).
    forecast_trip_threshold: f64,
}
```

The circuit breaker is checked first in `evaluate_full`, before any watchers run:

```rust
// Source: crates/roko-conductor/src/conductor.rs, lines 315-326
if let Some(ref pid) = plan_id {
    if self.circuit_breaker.is_tripped(pid) {
        self.update_routing_bias(stream, &[]);
        return ConductorDecision::fail(
            "circuit-breaker",
            roko_core::FailureKind::MaxIterations,
        )
        .with_signals(vec![CognitiveSignal::Shutdown {
            reason: "circuit breaker tripped".into(),
        }]);
    }
}
```

### 7.3 State Persistence

The circuit breaker supports snapshot/restore for persistence across restarts:

```rust
// Source: crates/roko-conductor/src/circuit_breaker.rs, lines 126-133
pub struct CircuitBreakerState {
    pub max_failures: u32,
    pub records: HashMap<String, FailureRecord>,
}
```

`snapshot_state()` captures the current state; `from_state()` rebuilds a circuit breaker from a saved snapshot. This ensures that circuit breaker state survives process restarts.

### 7.4 Predictive Mode (COND-08)

When predictive mode is enabled via `with_predictive(trip_threshold)`, the circuit breaker also tracks a `HoltForecaster` per plan. Each success is fed as 0.0 and each failure as 1.0. The forecaster projects the error rate forward.

Two levels of proactive signaling:

```rust
// Source: crates/roko-conductor/src/circuit_breaker.rs, lines 108-123
pub enum ProactiveTripSignal {
    /// Forecast at horizon 3 exceeds threshold -- early warning.
    Warning {
        plan_id: String,
        forecast_h3: f64,
    },
    /// Forecast at horizon 1 exceeds threshold -- proactive trip.
    ProactiveTrip {
        plan_id: String,
        forecast_h1: f64,
    },
}
```

- **Warning** (`forecast(3) >= threshold`): The error rate trend suggests the circuit will trip within 3 steps. The conductor emits a `CognitiveSignal::Cooldown { factor: 1.5 }` to slow down.
- **ProactiveTrip** (`forecast(1) >= threshold`): The error rate will exceed the threshold on the very next step. The conductor emits a `CognitiveSignal::Shutdown` and trips proactively, avoiding the cost of the final failure.

The count-based trip is always preserved as a fallback:

```rust
// Source: crates/roko-conductor/src/circuit_breaker.rs, lines 214-243
pub fn record_failure(&self, plan_id: &str, reason: impl Into<String>, now_ms: i64) -> bool {
    // ... increment failure count ...
    // Count-based trip check (fallback always active).
    if count >= self.max_failures {
        return true;
    }
    // Predictive trip: if forecast(1) exceeds threshold, proactively trip.
    if self.predictive {
        if let Some(f) = self.forecasters.get(plan_id) {
            if f.observation_count() >= 2 && f.forecast(1) >= self.forecast_trip_threshold {
                return true;
            }
        }
    }
    false
}
```

---

## 8. Holt Exponential Smoothing -- Full Mathematical Treatment

**File**: `crates/roko-conductor/src/circuit_breaker.rs`, lines 32-104

### 8.1 Background

Holt's method (also called double exponential smoothing or Holt's linear trend method) was introduced by Charles C. Holt in 1957 [3]. It extends simple exponential smoothing by adding a trend component. Simple exponential smoothing tracks only the level (where the series is now). Holt's method also tracks the slope (where the series is going). This makes it suitable for forecasting error rates that are trending upward.

The method is widely used in production forecasting due to its simplicity and effectiveness. It requires minimal historical data (as few as 2 observations to produce a meaningful trend), has only 2 tunable parameters, and is computationally trivial -- a constant-time update per observation [4].

### 8.2 The Three Equations

The Holt method consists of two update equations and one forecast equation:

```
Level(t) = alpha * observation(t) + (1 - alpha) * (Level(t-1) + Trend(t-1))
Trend(t) = beta  * (Level(t) - Level(t-1)) + (1 - beta) * Trend(t-1)
Forecast(t+h) = Level(t) + h * Trend(t)
```

Where:
- `alpha` (level smoothing factor, default 0.3): Controls how quickly the level responds to new observations. Higher alpha = more responsive to recent data, lower alpha = more stable.
- `beta` (trend smoothing factor, default 0.1): Controls how quickly the trend responds to level changes. Lower beta = smoother trend estimate, less sensitive to noise.
- `h`: Forecast horizon (number of steps ahead to predict).

The level equation is a weighted average of the new observation and the one-step-ahead forecast from the previous period (`Level(t-1) + Trend(t-1)`). The trend equation is a weighted average of the observed slope (`Level(t) - Level(t-1)`) and the previous trend estimate.

### 8.3 The Implementation

```rust
// Source: crates/roko-conductor/src/circuit_breaker.rs, lines 44-104
pub struct HoltForecaster {
    pub level: f64,
    pub trend: f64,
    pub alpha: f64,   // default 0.3
    pub beta: f64,    // default 0.1
    pub observations: u32,
}

impl HoltForecaster {
    pub fn update(&mut self, observation: f64) {
        if self.observations == 0 {
            // First observation: initialize level, trend starts at 0.
            self.level = observation;
            self.trend = 0.0;
        } else {
            let prev_level = self.level;
            // Level equation: blend new observation with predicted level.
            self.level = self.alpha * observation
                       + (1.0 - self.alpha) * (self.level + self.trend);
            // Trend equation: blend observed slope with previous trend.
            self.trend = self.beta * (self.level - prev_level)
                       + (1.0 - self.beta) * self.trend;
        }
        self.observations += 1;
    }

    pub fn forecast(&self, horizon: usize) -> f64 {
        self.level + (horizon as f64) * self.trend
    }
}
```

**Note on initialization**: The first observation sets the level directly and the trend to 0. This is the standard initialization approach for Holt's method when only a single data point is available. An alternative (used when batch data is available) is to set the initial trend to the difference between the first two observations, but roko uses streaming updates.

### 8.4 Worked Example

Consider a plan where the circuit breaker observes this sequence of outcomes (1.0 = failure, 0.0 = success):

```
Step 1: observation = 0.0 (success)
  level = 0.0, trend = 0.0
  forecast(1) = 0.0, forecast(3) = 0.0

Step 2: observation = 1.0 (failure)
  level = 0.3 * 1.0 + 0.7 * (0.0 + 0.0) = 0.3
  trend = 0.1 * (0.3 - 0.0) + 0.9 * 0.0  = 0.03
  forecast(1) = 0.3 + 1 * 0.03 = 0.33
  forecast(3) = 0.3 + 3 * 0.03 = 0.39

Step 3: observation = 1.0 (failure)
  level = 0.3 * 1.0 + 0.7 * (0.3 + 0.03) = 0.300 + 0.231 = 0.531
  trend = 0.1 * (0.531 - 0.3) + 0.9 * 0.03 = 0.0231 + 0.027 = 0.0501
  forecast(1) = 0.531 + 1 * 0.0501 = 0.5811
  forecast(3) = 0.531 + 3 * 0.0501 = 0.6813
  -- forecast(1) > 0.5 threshold: PROACTIVE TRIP
```

After just 3 observations (1 success + 2 failures), the forecaster projects that the error rate will exceed 0.5 on the next step. The circuit trips proactively, saving the cost of a third failure. The key insight: the positive trend (0.0501) amplifies the level, causing the forecast to cross the threshold even though the level alone (0.531) is only slightly above 0.5.

### 8.5 Why Not Thompson Sampling Here?

Thompson Sampling is used for threshold learning (section 11), not for error rate forecasting. The distinction is:
- **Holt smoothing**: "What will the error rate be in N steps?" (time-series forecasting with trend detection)
- **Thompson Sampling**: "Which intervention action (Continue, Restart, Abort) has the highest expected reward?" (exploration/exploitation tradeoff in a multi-armed bandit setting)

Holt is appropriate for the circuit breaker because error rates are sequential and correlated -- two failures in a row make a third failure more likely. Thompson Sampling is appropriate for the intervention policy because different situations may call for different actions, and the system needs to explore suboptimal actions occasionally to learn their true reward.

> **Roko doc reference**: The decision to use f64 for EMA/Holt computations is documented in `docs/v2-depth/05-execution-engine/resilience-and-numerics.md` (section 4.6): "For adaptive gate thresholds that accumulate over thousands of gate evaluations, f64 is mandatory."

### 8.6 Limitations of Holt's Method

Holt's linear trend method does not model seasonality (periodic patterns). If provider error rates exhibited daily or weekly cycles, Holt-Winters triple exponential smoothing (adding a seasonal component) would be more appropriate [5]. However, for the conductor's use case -- short-horizon forecasting of error rate trends within a single plan execution -- linear trend is sufficient. Plans typically complete within minutes to hours, too short for seasonal effects to manifest.

---

## 9. Compound Pattern Detection (CEP)

**File**: `crates/roko-conductor/src/pattern_detector.rs` (326 lines)

### 9.1 Background: Complex Event Processing

Complex Event Processing (CEP) was pioneered by David Luckham at Stanford and described in *The Power of Events: An Introduction to Complex Event Processing in Distributed Enterprise Systems* (Luckham, 2002) [6]. CEP detects meaningful patterns across streams of events by composing simple events into complex events using operators like conjunction (AND), sequence (A then B), and negation (A without B).

### 9.2 What CEP Brings to the Conductor

CEP-inspired compound pattern detection identifies multi-signal anomalies that no single watcher can see. Individual metrics may all stay within bounds while the system is failing in a correlated way. The pattern detector groups watchers into three families and detects three composition patterns.

### 9.3 Watcher Family Classification

```rust
// Source: crates/roko-conductor/src/pattern_detector.rs, lines 20-27
pub enum WatcherFamily {
    /// Cost, time, and context window pressure.
    Resource,
    /// Compile, test, and spec drift watchers.
    Quality,
    /// Ghost turn, iteration loop, stuck pattern, and review loop.
    Progress,
}
```

The default family mapping (lines 60-76):

| Watcher | Family |
|---------|--------|
| `cost-overrun` | Resource |
| `time-overrun` | Resource |
| `context-window-pressure` | Resource |
| `compile-fail-repeat` | Quality |
| `test-failure-budget` | Quality |
| `spec-drift` | Quality |
| `ghost-turn` | Progress |
| `iteration-loop` | Progress |
| `stuck-pattern` | Progress |
| `review-loop` | Progress |

### 9.4 Three Composition Patterns

**1. Family Conjunction**: When 2+ watchers in the same family fire at Warning+ severity in the same evaluation cycle, a compound pattern is detected. This is a CEP conjunction operator applied within each family.

```
cost-overrun (Warning) + time-overrun (Warning)
=> CompoundPattern { name: "resource_exhaustion", severity: Critical }

compile-fail-repeat (Warning) + test-failure-budget (Warning)
=> CompoundPattern { name: "quality_degradation", severity: Critical }

ghost-turn (Warning) + stuck-pattern (Warning)
=> CompoundPattern { name: "progress_stall", severity: Critical }
```

*Real-world example*: The agent is consuming tokens (cost rising) AND taking too long (time rising), but neither metric has individually crossed its threshold. The family conjunction detects the correlated degradation and escalates to Critical -- the system is clearly failing even though individual metrics look borderline acceptable.

**2. Total Resource Exhaustion**: When ALL three resource watchers fire simultaneously, a special "total_resource_exhaustion" pattern is emitted in addition to the family-level pattern:

```rust
// Source: crates/roko-conductor/src/pattern_detector.rs, lines 151-161
let resource_watchers = ["cost-overrun", "time-overrun", "context-window-pressure"];
let all_resource_fired = resource_watchers.iter().all(|w| fired_watchers.contains_key(w));
if all_resource_fired {
    patterns.push(CompoundPattern {
        pattern_name: "total_resource_exhaustion".to_string(),
        contributing_watchers: resource_watchers.iter().map(|s| s.to_string()).collect(),
        escalated_severity: Severity::Critical,
    });
}
```

**3. Progressive Degradation Sequence**: When ghost-turn, iteration-loop, AND stuck-pattern all have non-zero consecutive fire counts (tracked via the history buffer), a "progressive_degradation" pattern fires. This detects the common failure cascade where an agent first starts producing empty turns, then starts looping through gate failures, then gets stuck repeating the same action. This is a CEP sequence operator detecting a temporal ordering of failure modes.

*Real-world example*: Turn 1-3: agent produces ghost turns (thinking without acting). Turn 4-6: agent starts producing code but fails every gate check. Turn 7-9: agent gets stuck running the same test over and over. The progressive degradation detector recognizes this cascade and fires at Critical severity -- the agent has exhausted all its strategies.

### 9.5 Temporal Hysteresis

The pattern detector maintains per-watcher consecutive fire counts. A watcher must fire for N consecutive evaluation cycles (configurable via `hysteresis_window`, default 2) before it "passes hysteresis." If a watcher stops firing, its count resets to 0.

```rust
// Source: crates/roko-conductor/src/pattern_detector.rs, lines 97-113
pub fn record(&mut self, outputs: &[WatcherOutput]) -> Vec<CompoundPattern> {
    let fired_watchers: HashMap<&str, &WatcherOutput> = outputs
        .iter()
        .filter(|o| o.severity >= Severity::Warning)
        .map(|o| (o.watcher.as_str(), o))
        .collect();

    // Increment or reset consecutive counts.
    let all_watchers: Vec<String> = self.family_map.keys().cloned().collect();
    for watcher in &all_watchers {
        if fired_watchers.contains_key(watcher.as_str()) {
            *self.consecutive_fires.entry(watcher.clone()).or_default() += 1;
        } else {
            self.consecutive_fires.insert(watcher.clone(), 0);
        }
    }
    // ... pattern detection follows ...
}
```

### 9.6 How Compound Patterns Affect the Conductor Decision

When compound patterns are detected, the conductor in `evaluate_full` (lines 336-368) takes two actions:

1. **Emits cognitive signals** based on the pattern type:
   - `resource_exhaustion` / `total_resource_exhaustion`: `Cooldown { factor: 2.0 }` + `Reprioritize`
   - `quality_degradation`: `Escalate { to_tier: 3 }` (switch to a stronger model)
   - `progress_stall` / `progressive_degradation`: `Explore { budget_multiplier: 2.0 }` (try alternative approaches)

2. **Overrides the policy decision**: If any compound pattern has `Critical` severity and the policy returned `Continue`, the conductor forces a `Restart` decision.

---

## 10. Adaptive Threshold Learning

**File**: `crates/roko-conductor/src/threshold_learner.rs` (399 lines)

### 10.1 The Problem

Static thresholds have two failure modes:
- **Too tight**: Too many false alarms cause unnecessary restarts, wasting time and money switching providers or restarting tasks that would have succeeded.
- **Too loose**: Real issues are missed, leading to degraded user experience and wasted budget on failing plans.

The optimal threshold is different for each watcher and changes over time as the system encounters different workloads.

### 10.2 EMA-Based Learning

The `ThresholdLearner` uses Exponential Moving Average (EMA) to adjust thresholds based on intervention outcomes. After each intervention, the orchestrator records whether the intervention improved the outcome (the task succeeded after the restart/switch).

```rust
// Source: crates/roko-conductor/src/threshold_learner.rs, lines 31-40
pub struct AdaptiveThreshold {
    /// Current EMA of the optimal intervention boundary.
    pub ema: f64,
    /// Total observations for this watcher.
    pub observations: u64,
    /// Interventions that were effective (task succeeded after).
    pub effective_count: u64,
    /// Interventions that were ineffective (task still failed after).
    pub ineffective_count: u64,
}
```

The update rule:
- **Effective intervention**: Lower the threshold by 0.05 (intervene earlier next time, since intervening worked)
- **Ineffective intervention**: Raise the threshold by 0.05 (intervene later, since intervening did not help)
- Apply EMA smoothing with alpha = 0.1

```rust
// Source: crates/roko-conductor/src/threshold_learner.rs, lines 58-79
fn update(&mut self, alpha: f64, effective: bool) {
    self.observations += 1;
    if effective {
        self.effective_count += 1;
    } else {
        self.ineffective_count += 1;
    }
    let target = if effective {
        (self.ema - 0.05).max(0.1)  // lower = intervene earlier
    } else {
        (self.ema + 0.05).min(1.0)  // higher = intervene later
    };
    if self.observations == 1 {
        self.ema = target;
    } else {
        self.ema = alpha.mul_add(target, (1.0 - alpha) * self.ema);
    }
}
```

The EMA update formula written mathematically:

```
EMA(t) = alpha * target(t) + (1 - alpha) * EMA(t-1)
```

where `target(t) = EMA(t-1) - 0.05` (if effective) or `EMA(t-1) + 0.05` (if ineffective), clamped to [0.1, 1.0].

### 10.3 Warmup Period

The learner requires 10 observations (`WARMUP_OBSERVATIONS`) before its adaptive thresholds override the static defaults. During warmup, `restart_threshold()` returns the default (0.7) and `fail_threshold()` returns 0.9. This ensures the system does not make drastic threshold changes based on insufficient data.

### 10.4 Persistence

Thresholds persist to `.roko/learn/conductor-thresholds.json` via atomic write (write to `.tmp`, then rename). This ensures the system remembers what it learned across restarts and does not lose calibration data.

### 10.5 Constants

| Constant | Value | Meaning |
|----------|-------|---------|
| `DEFAULT_ALPHA` | 0.1 | EMA smoothing factor |
| `DEFAULT_RESTART_THRESHOLD` | 0.7 | Default restart threshold before warmup |
| `DEFAULT_FAIL_THRESHOLD` | 0.9 | Default fail threshold before warmup |
| `WARMUP_OBSERVATIONS` | 10 | Minimum observations before adaptive mode |
| `MAX_HISTORY` | 100 | Ring buffer size for intervention history |

---

## 11. Thompson Sampling via BanditPolicy

**File**: `crates/roko-conductor/src/interventions.rs`, lines 130-282

### 11.1 Background: Thompson Sampling

Thompson Sampling is a Bayesian approach to the multi-armed bandit problem, first described by William R. Thompson in 1933 [7]. The multi-armed bandit problem models the exploration-exploitation tradeoff: given multiple actions with unknown rewards, how should a system balance trying new actions (exploration) to learn their rewards versus repeating the best-known action (exploitation) to maximize cumulative reward?

Thompson Sampling maintains a posterior probability distribution over the reward for each action (typically a Beta distribution for binary rewards). At each decision point, it samples from each action's posterior and selects the action with the highest sample. Actions with high uncertainty will occasionally produce high samples, ensuring they are explored. Actions with well-established high rewards will consistently produce high samples, ensuring they are exploited. Agrawal and Goyal (2012) proved that Thompson Sampling achieves logarithmic expected regret for the stochastic multi-armed bandit problem, matching the theoretical lower bound [8].

### 11.2 What Thompson Sampling Does in the Conductor

In the conductor's context, the "arms" are the possible intervention actions (Continue, Restart, SwitchModel, Abort, InjectHint). Thompson Sampling maintains a probability distribution over the expected reward for each action and samples from these distributions to decide which action to take.

The `BanditPolicy` uses the `ConductorBandit` from `roko-learn` to make learned intervention decisions.

### 11.3 Warmup and Blending

During warmup (fewer than 50 total observations), the bandit policy delegates entirely to `WorstSeverityPolicy`. After warmup, it blends the bandit's recommendation with the static policy at a 65/35 ratio:

```rust
// Source: crates/roko-conductor/src/interventions.rs, lines 133-136
const BANDIT_WARMUP_THRESHOLD: u64 = 50;
const BANDIT_BLEND_WEIGHT: f64 = 0.65;
```

The blending algorithm:
1. Compute the static (worst-severity) decision
2. Ask the bandit for its recommendation
3. If they agree, use that decision
4. If they disagree, use the bandit's recommendation 65% of the time (determined by a deterministic blend using the context timestamp to avoid non-determinism in tests)

```rust
// Source: crates/roko-conductor/src/interventions.rs, lines 265-276
if static_decision.label() == bandit_decision.label() {
    return static_decision;
}
let blend_value = (ctx.now_ms as f64 * 0.001).fract();
if blend_value < BANDIT_BLEND_WEIGHT {
    bandit_decision
} else {
    static_decision
}
```

**Note on the blend mechanism**: Rather than using a random number generator (which would make tests non-deterministic), the blend uses the fractional part of `ctx.now_ms * 0.001`. This produces a pseudo-random value in [0, 1) that varies naturally across evaluations (since timestamps differ) while being deterministic for any given timestamp.

### 11.4 State Encoding

The bandit receives a `ConductorState` encoding the current system state:

```rust
// Source: crates/roko-conductor/src/interventions.rs, lines 198-232
fn state_from_outputs(outputs: &[WatcherOutput]) -> ConductorState {
    let worst_severity = outputs.iter().map(|o| o.severity).max();
    let consecutive_failures = outputs
        .iter()
        .filter(|o| o.severity >= Severity::Warning)
        .count() as u32;
    let error_pattern = /* mapped from watcher names */;
    ConductorState {
        iteration: consecutive_failures.max(1),
        consecutive_failures,
        error_pattern,
        elapsed_ms: 0,
        cost_so_far_usd: 0.0,
        model_tier: "standard".to_string(),
        task_complexity: match worst_severity {
            Some(Severity::Critical) => "architectural",
            Some(Severity::Warning) => "focused",
            _ => "mechanical",
        },
    }
}
```

The error pattern mapping:

| Watcher | ErrorPattern |
|---------|-------------|
| `compile-fail-repeat` | `Compile` |
| `test-failure-budget` | `Test` |
| `iteration-loop` / `stuck-pattern` | `LoopDetected` |
| `context-window-pressure` | `ContextOverflow` |
| `time-overrun` | `Timeout` |
| `cost-overrun` | `RateLimit` |

---

## 12. Yerkes-Dodson Pressure Framework

**File**: `crates/roko-conductor/src/yerkes_dodson.rs` (248 lines)

### 12.1 Background: The Yerkes-Dodson Law

The Yerkes-Dodson law is an empirical observation first reported by Robert M. Yerkes and John D. Dodson in 1908 in "The Relation of Strength of Stimulus to Rapidity of Habit-Formation" [9]. The study found that moderate stimulus intensity produced optimal learning in mice -- too little stimulus produced no motivation, while too much produced anxiety that interfered with performance. Later work, particularly by Donald Hebb (1955), generalized this into the "inverted-U" relationship between arousal and performance [10].

### 12.2 Application to AI Systems

Roko applies this framework to AI system pressure -- moderate deadlines and constraints produce the best output; too little pressure leads to unfocused exploration; too much pressure leads to rushed, failing work. Concretely:

- **Low pressure** (0.0-0.2): The agent has plenty of budget, time, and no failures. Interventions should be light -- let the agent explore different approaches.
- **Optimal pressure** (0.5): The agent is working within reasonable constraints. Peak performance, minimal intervention needed.
- **High pressure** (0.8-1.0): Budget nearly exhausted, time running out, failures mounting. Interventions should be aggressive -- switch models, reduce scope, or abort.

### 12.3 The Gaussian Performance Curve

The curve is modeled as a Gaussian:

```
performance(pressure) = exp(-((pressure - optimal)^2) / (2 * width^2))
```

This produces the characteristic inverted-U shape. At the optimal pressure point, performance is 1.0 (maximum). As pressure deviates from optimal in either direction, performance decays exponentially according to the width parameter.

### 12.4 The Implementation

```rust
// Source: crates/roko-conductor/src/yerkes_dodson.rs, lines 20-28
pub struct YerkesDodson {
    /// Current pressure level (0.0 = no pressure, 1.0 = maximum).
    pub pressure: f64,
    /// Pressure level at which performance peaks (default: 0.5).
    pub optimal: f64,
    /// Width of the Gaussian curve (default: 0.25).
    pub width: f64,
}
```

Key methods:

```rust
// Source: crates/roko-conductor/src/yerkes_dodson.rs, lines 61-74
pub fn performance_multiplier(&self) -> f64 {
    let diff = self.pressure - self.optimal;
    let exponent = -(diff * diff) / (2.0 * self.width * self.width);
    exponent.exp()
}

pub fn intervention_aggressiveness(&self) -> f64 {
    1.0 - self.performance_multiplier()
}
```

At the optimal pressure (0.5 by default):
- `performance_multiplier()` = 1.0 (peak performance)
- `intervention_aggressiveness()` = 0.0 (no intervention needed)

Far from optimal (pressure = 0.0 or 1.0):
- `performance_multiplier()` drops toward ~0.135 (with width = 0.25)
- `intervention_aggressiveness()` rises toward ~0.865

### 12.5 Worked Example: Performance at Various Pressure Levels

With defaults (optimal = 0.5, width = 0.25):

| Pressure | Exponent | Performance | Aggressiveness |
|----------|----------|-------------|----------------|
| 0.0 | -((0.5)^2)/(2*0.0625) = -2.0 | e^(-2.0) = 0.135 | 0.865 |
| 0.25 | -((0.25)^2)/(0.125) = -0.5 | e^(-0.5) = 0.607 | 0.393 |
| 0.5 | 0 | 1.0 | 0.0 |
| 0.75 | -0.5 | 0.607 | 0.393 |
| 1.0 | -2.0 | 0.135 | 0.865 |

The curve is symmetric around the optimal point, confirming that both under-pressure and over-pressure degrade performance equally.

### 12.6 Pressure Computation

Pressure is derived from four weighted signals:

```rust
// Source: crates/roko-conductor/src/yerkes_dodson.rs, lines 100-112
pub fn compute_pressure(
    cost_pressure: f64,     // fraction of budget consumed
    time_pressure: f64,     // fraction of time budget consumed
    failure_rate: f64,      // recent gate failure rate
    stuck_signals: f64,     // number of stuck/loop watcher triggers
) -> f64 {
    let raw = cost_pressure * 0.25
        + time_pressure * 0.25
        + failure_rate * 0.30
        + stuck_signals * 0.20;
    raw.clamp(0.0, 1.0)
}
```

The weights reflect that failure rate (0.30) is the strongest pressure signal, followed equally by cost (0.25) and time (0.25), with stuck signals (0.20) having less weight.

### 12.7 Danger Zone

The `is_danger_zone()` method returns `true` when performance drops below 50% (i.e., pressure is more than ~0.42 units from optimal with default width). The `pressure_delta()` method returns a signed value indicating how to adjust pressure to move toward optimal (positive = increase pressure, negative = decrease pressure).

### 12.8 Curve Width

The `width` parameter controls how sharp the performance drop-off is. A narrow width (0.1) means performance degrades rapidly as pressure moves away from optimal. A wide width (0.4) means performance degrades more gradually. The default (0.25) is a moderate curve.

The constructor guards against division by zero: `width: width.max(0.001)`.

---

## 13. Federation: 4-Level Conductor Hierarchy

**File**: `crates/roko-conductor/src/federation.rs` (289 lines)

### 13.1 Background: Beer's Viable System Model

Roko's conductor federation is inspired by Stafford Beer's Viable System Model (VSM), articulated in *Brain of the Firm* (Beer, 1972) [11]. The VSM identifies five interacting subsystems (S1-S5) that every viable organization needs: operations, coordination, control, intelligence, and policy. Beer's key insight is that these subsystems operate at different temporal frequencies and organizational scopes, forming a recursive hierarchy [12].

### 13.2 The Four Levels

Roko adapts Beer's model into a four-level conductor hierarchy where each level operates at a different temporal frequency and scope:

| Level | Name | Scope | Frequency | Description |
|-------|------|-------|-----------|-------------|
| **L1** | `TurnConductor` | Per-turn | Gamma (every agent turn) | Stuck detection + meta-cognition |
| **L2** | `Conductor` (task) | Per-task | Beta (every evaluation) | The 10 watchers + policy |
| **L3** | `PlanConductor` | Per-plan | Delta (after each task) | Aggregates task decisions |
| **L4** | `FleetConductor` | Per-fleet | Alpha (cross-agent) | Fleet-wide coordination (stub) |

### 13.3 L1: TurnConductor

Wraps the `StuckDetector` and `MetaCognitionHook` from `stuck_detection.rs`. Evaluates after every single agent turn for stuck conditions.

```rust
// Source: crates/roko-conductor/src/federation.rs, lines 23-30
pub struct TurnConductor {
    pub stuck_detector: StuckDetector,
    pub meta_cognition: MetaCognitionHook,
    /// Sensitivity multiplier (adjustable by L2). Default 1.0.
    pub sensitivity: f64,
}
```

The sensitivity parameter allows the L2 task conductor to tune L1's aggressiveness. Higher sensitivity = lower confidence threshold for firing (intervene more aggressively). The threshold formula:

```rust
let threshold = 0.6 / self.sensitivity.max(0.1);
```

At default sensitivity (1.0), the threshold is 0.6. At sensitivity 2.0, the threshold drops to 0.3, making the detector more trigger-happy. Sensitivity is clamped to [0.1, 10.0].

### 13.4 L3: PlanConductor

Aggregates task-level decisions into plan-level decisions. Tracks:
- Accumulated task decisions and outcomes
- Remaining plan budget
- Task failure count vs. threshold

```rust
// Source: crates/roko-conductor/src/federation.rs, lines 78-89
pub struct PlanConductor {
    pub task_decisions: Vec<TaskDecisionRecord>,
    pub plan_budget_remaining: f64,
    pub task_failure_count: usize,
    pub max_plan_failures: usize,  // default 2
    pub l2_adjustments: HashMap<String, f64>,
}
```

The L3 conductor cascades parameters down to L2:
- **High failure rate** (> 50% of completed tasks failed): Lower quality watcher thresholds by 30% (set `compile-fail-repeat` and `test-failure-budget` multipliers to 0.7)
- **Low budget** (< $1.00 remaining): Tighten cost thresholds (set `cost-overrun` multiplier to 0.5)

### 13.5 L4: FleetConductor

Currently a stub that always returns `ConductorDecision::cont()`. Designed for Phase 2+ multi-agent coordination where fleet-wide budget and resource allocation need centralized oversight.

```rust
// Source: crates/roko-conductor/src/federation.rs, lines 203-225
pub struct FleetConductor {
    pub active_agents: usize,
    pub fleet_budget_remaining: f64,
}
impl FleetConductor {
    pub fn evaluate(&self) -> ConductorDecision {
        ConductorDecision::cont()
    }
}
```

---

## 14. Self-Healing with Oscillation Detection

**File**: `crates/roko-conductor/src/self_healing.rs` (379 lines)

### 14.1 The Oscillation Problem

Without oscillation detection, the conductor can enter pathological states:
- Watcher fires -> restart -> watcher stops -> next turn watcher fires again -> restart -> ...
- Provider A degrades -> switch to B -> B overwhelmed -> switch back to A -> loop

This oscillation wastes resources and makes no forward progress. The self-healing subsystem detects and breaks these loops.

### 14.2 Three Recovery Strategies

```rust
// Source: crates/roko-conductor/src/self_healing.rs, lines 54-68
pub enum HealingAction {
    /// Reset a specific watcher that is oscillating.
    ResetWatcher(String),
    /// Suppress interventions from a watcher for N ticks.
    CooldownWatcher { watcher: String, ticks: u64 },
    /// Auto-restart the conductor with fresh state.
    AutoRestart,
    /// No healing action needed.
    None,
}
```

### 14.3 Policy Configuration

```rust
// Source: crates/roko-conductor/src/self_healing.rs, lines 12-23
pub struct SelfHealingPolicy {
    /// Maximum oscillation count before watcher reset. Default: 5.
    pub max_oscillations: u32,
    /// Ticks to suppress after reset. Default: 10.
    pub cooldown_ticks: u64,
    /// Consecutive failures before full conductor reset. Default: 3.
    pub auto_restart_threshold: u32,
}
```

### 14.4 Oscillation Detection Algorithm

The `SelfHealingState` tracks per-watcher state across ticks:

1. **State tracking**: For each watcher, record whether it was firing on the previous tick.
2. **Oscillation counting**: If the state changes (firing -> not firing, or vice versa), increment the oscillation counter. If the state is stable (same as last tick), decay the counter by 1.
3. **Threshold check**: When the oscillation counter reaches `max_oscillations`, the watcher is reset and enters cooldown for `cooldown_ticks` ticks.
4. **Cooldown suppression**: During cooldown, observations for that watcher are ignored -- it cannot trigger interventions.

```rust
// Source: crates/roko-conductor/src/self_healing.rs, lines 81-126
pub fn observe_watcher(
    &mut self, watcher: &str, firing: bool, policy: &SelfHealingPolicy
) -> HealingAction {
    if self.is_in_cooldown(watcher) {
        return HealingAction::None;
    }
    let was_firing = self.watcher_states.iter()
        .find(|(name, _)| name == watcher)
        .map(|(_, was)| *was);
    // Update current state...
    if let Some(was) = was_firing {
        if was != firing {
            // State changed -- oscillation detected
            let count = self.increment_oscillation(watcher);
            if count >= policy.max_oscillations {
                self.total_interventions += 1;
                self.enter_cooldown(watcher, policy.cooldown_ticks);
                return HealingAction::ResetWatcher(watcher.to_string());
            }
        } else {
            // Stable -- decay oscillation count
            self.decay_oscillation(watcher);
        }
    }
    HealingAction::None
}
```

*Real-world example*: The GhostTurnWatcher fires on tick 1, causing a restart. On tick 2, the restart produces good output, so the watcher does not fire. On tick 3, the agent falls back into empty output, so the watcher fires again. This on-off-on-off pattern continues. After 5 oscillations, the self-healer resets the GhostTurnWatcher and suppresses it for 10 ticks, giving the agent a chance to stabilize without constant restarts.

### 14.5 Auto-Restart

If the conductor itself fails consecutively (e.g., every plan it restarts still fails), the `record_failure` method counts consecutive failures. At `auto_restart_threshold` (default 3), it returns `HealingAction::AutoRestart`, signaling the orchestrator to reset the conductor to fresh state.

---

## 15. Diagnosis Engine

**File**: `crates/roko-conductor/src/diagnosis.rs` (936 lines)

### 15.1 Purpose

The diagnosis engine is a pattern-matching system that classifies error output text into known categories and suggests remediation actions. It is a pure function: given an error string, it returns ranked matches with confidence scores.

### 15.2 Error Categories

The engine recognizes 20 error categories:

| Category | Examples | Suggested Action |
|----------|----------|-----------------|
| `CompileError` | `error[E0277]` | RetryWithContext |
| `TypeMismatch` | `error[E0308]` | RetryWithContext |
| `BorrowCheckerError` | `error[E0502]`, `E0382`, `E0505` | RestartAgent |
| `LifetimeError` | `error[E0106]`, `E0621` | RestartAgent |
| `ImportError` | `error[E0432]`, `E0433` | AutoFix |
| `TestFailure` | `test result: FAILED` | RetryWithContext |
| `ClippyWarning` | `warning:`, `clippy::` | WarnAndContinue |
| `GitConflict` | `<<<<<<<`, `CONFLICT (content)` | MergeResolution |
| `DependencyError` | `no matching package named` | RetryWithContext |
| `MissingFile` | `No such file or directory` | RetryWithContext |
| `PermissionDenied` | `Permission denied` | AbortPlan |
| `NetworkError` | `Connection refused`, DNS failure | BackoffRetry |
| `TimeoutError` | `timed out` | BackoffRetry |
| `OomError` | `out of memory`, `SIGKILL` | AbortPlan |
| `DiskFull` | `No space left on device` | AbortPlan |
| `LlmRateLimit` | `rate limit`, `429 Too Many Requests` | BackoffRetry |
| `LlmContextOverflow` | `context_length_exceeded` | ReduceContext |
| `LlmRefusal` | `content_filter`, `I cannot` | SwitchModel |
| `ProcessCrash` | `Segmentation fault`, `SIGABRT` | RestartAgent |
| `LoopDetected` | `LOOP DETECTED` | RestartAgent |

The engine ships with 36 built-in patterns (the `built_in_patterns()` function).

### 15.3 Confidence Scoring

Confidence is computed from three factors:
1. **Coverage ratio**: How much of the error output the needle covers (longer match relative to total = higher confidence)
2. **Specificity bonus**: Longer needles get a small boost (0.5% per character, capped at 15%)
3. **Exact match bonus**: Case-sensitive patterns with hyphens in the name get an additional 2% boost

This ensures that `error[E0308]` (specific type mismatch) scores higher than `error[E` (generic compile error) when both match.

---

## 16. Health Monitor

**File**: `crates/roko-conductor/src/health.rs` (641 lines)

### 16.1 System-Level Health Checks

The `HealthMonitor` runs four composable health checks against a `SystemSnapshot`:

| Check | What It Monitors | Degraded When |
|-------|-----------------|---------------|
| `terminal_liveness` | Agent heartbeat freshness | Heartbeat stale > 60s |
| `agent_status` | Active vs. expected agents | Some agents missing |
| `spec_drift` | Plan spec hash change | Spec changed since agents started |
| `coverage_trend` | Test coverage trajectory | Coverage declining > 2% |

The overall status is the worst of all checks:

```rust
// Source: crates/roko-conductor/src/health.rs, lines ~194-200
pub fn overall_status(&self, snapshot: &SystemSnapshot) -> HealthStatus {
    self.check_all(snapshot)
        .iter()
        .map(|c| c.status)
        .max()
        .unwrap_or(HealthStatus::Healthy)
}
```

Three health levels: `Healthy` < `Degraded` < `Critical`.

---

## 17. Stuck Detection and Meta-Cognition

**File**: `crates/roko-conductor/src/stuck_detection.rs` (2,004 lines)

### 17.1 StuckDetector

The `StuckDetector` analyzes `ActivityEntry` records (hash of output, gate failure count, test delta, iteration number) using configurable heuristics:

```rust
pub struct StuckThresholds {
    pub max_identical_hashes: usize,      // default 5
    pub min_test_progress: i64,           // default -2
    pub max_iterations_without_progress: usize,  // default 4
    pub max_gate_failures_in_window: usize,      // default 3
    pub window_size: usize,              // default 5
    pub empty_output_threshold: usize,   // default 3
}
```

Twelve `StuckKind` classifications: `OutputLoop`, `NoProgress`, `GateLoop`, `CompileLoop`, `EmptyOutput`, `ExcessiveRetries`, `ReviewLoop`, `IterationLoop`, `SilenceTimeout`, `CompileFailThreshold`, `TaskStall`, `ContextPressure`.

### 17.2 MetaCognitionHook

The meta-cognition hook provides a self-assessment mechanism. When a stuck condition is detected, it can recommend actions like `SelfAssess` (ask the agent to reflect on its approach) or `AlternativeStrategy` (try a different approach). It maintains a `CooldownFilter` that prevents repeated assessments from firing too frequently.

### 17.3 Relationship to L1 TurnConductor

The `TurnConductor` (federation L1) wraps `StuckDetector` + `MetaCognitionHook` and evaluates after every agent turn. The sensitivity parameter from L2 adjusts the confidence threshold.

---

## 18. Routing Bias and Provider Health

**File**: `crates/roko-conductor/src/conductor.rs`, lines 34-42 and 604-685

### 18.1 RoutingBias

After each evaluation, the conductor derives a `RoutingBias` that tells the orchestrator's router how to adjust provider selection:

```rust
// Source: crates/roko-conductor/src/conductor.rs, lines 34-42
pub struct RoutingBias {
    /// Model slugs that should be deprioritized for the next routing decision.
    pub deprioritize: Vec<String>,
    /// Whether routing should bias toward cheaper tiers.
    pub prefer_cheaper: bool,
    /// Human-readable reason for the bias.
    pub reason: String,
}
```

The derivation logic:
- **Load pressure** (any resource watcher at Warning+): Set `prefer_cheaper = true`
- **Recent failure** (any quality/progress watcher at Warning+): Extract the model slug from the most recent signal and add it to `deprioritize`

### 18.2 Provider Health Integration (COND-09)

The conductor optionally integrates with `ProviderHealthTracker` from `roko-learn`. When a provider is unhealthy, the conductor emits an `Escalate { to_tier: 2 }` cognitive signal:

```rust
// Source: crates/roko-conductor/src/conductor.rs, lines 399-409
if let Some(ref tracker) = self.provider_health {
    if let Some(provider) = extract_provider(stream) {
        if !tracker.is_healthy(&provider) {
            signals.push(CognitiveSignal::Escalate { to_tier: 2 });
        }
    }
}
```

---

## 19. Cognitive Signals

**File**: `crates/roko-conductor/src/conductor.rs`, lines 482-543

The `evaluate_full` method returns not just a decision but also `CognitiveSignal`s -- sub-critical modulations that hint at adjustments even when the primary decision is `Continue`. Signal types:

| Signal | Trigger | Effect |
|--------|---------|--------|
| `InjectContext` | Context pressure | Suggest trimming conversation history |
| `Cooldown { factor }` | Cost or time pressure | Extend budgets by the given factor |
| `Escalate { to_tier }` | Quality issues (without stuck) | Switch to a stronger model |
| `Explore { budget_multiplier }` | Stuck patterns | Try alternative approaches with extra budget |
| `Reprioritize { reason }` | 2+ resource watchers firing | Reorder the task queue |
| `Shutdown { reason }` | Circuit breaker tripped | Terminate the plan |

The derivation checks which watcher families are active at Warning+ level and emits the appropriate combination of signals:

```rust
// Source: crates/roko-conductor/src/conductor.rs, lines 521-523
// Quality issues without being stuck -> Escalate to stronger model.
if has_quality_issue && !has_stuck {
    signals.push(CognitiveSignal::Escalate { to_tier: 2 });
}
```

---

## 20. IronClaw Integration Plan

### Phase 1: LLM Provider Health Monitoring

**Where**: `crates/ironclaw_llm/`
**What**: Track per-provider health metrics and use Holt forecasting to detect degradation trends before they cause failures.
**Why**: IronClaw supports multiple LLM backends (OpenAI, Anthropic, Bedrock, NEAR AI, Ollama). When a provider degrades, the system should automatically route to a healthier alternative.

**Step 1: Create `crates/ironclaw_llm/src/health.rs`**

```rust
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

/// Per-provider health state.
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Holt forecaster for latency trend projection.
    latency_forecaster: HoltForecaster,
    /// Holt forecaster for error rate trend projection.
    error_rate_forecaster: HoltForecaster,
    /// Sliding window of recent request outcomes (true = success).
    recent_outcomes: Vec<bool>,
    /// Maximum window size for recent outcomes.
    max_window: usize,
    /// Whether this provider's circuit is tripped.
    tripped: bool,
    /// Number of consecutive failures.
    consecutive_failures: u32,
    /// Maximum consecutive failures before tripping (default: 3).
    max_failures: u32,
}

impl Default for ProviderHealth {
    fn default() -> Self {
        Self {
            latency_forecaster: HoltForecaster::default(),
            error_rate_forecaster: HoltForecaster::default(),
            recent_outcomes: Vec::new(),
            max_window: 20,
            tripped: false,
            consecutive_failures: 0,
            max_failures: 3,
        }
    }
}

/// Holt double exponential smoothing forecaster.
///
/// Adapted from roko-conductor's HoltForecaster.
/// Two equations updated on each observation:
///   level(t)   = alpha * obs(t) + (1 - alpha) * (level(t-1) + trend(t-1))
///   trend(t)   = beta  * (level(t) - level(t-1)) + (1 - beta) * trend(t-1)
///   forecast(h) = level(t) + h * trend(t)
#[derive(Debug, Clone)]
pub struct HoltForecaster {
    level: f64,
    trend: f64,
    alpha: f64,
    beta: f64,
    observations: u32,
}

impl Default for HoltForecaster {
    fn default() -> Self {
        Self { level: 0.0, trend: 0.0, alpha: 0.3, beta: 0.1, observations: 0 }
    }
}

impl HoltForecaster {
    pub fn update(&mut self, observation: f64) {
        if self.observations == 0 {
            self.level = observation;
            self.trend = 0.0;
        } else {
            let prev_level = self.level;
            self.level = self.alpha * observation
                + (1.0 - self.alpha) * (self.level + self.trend);
            self.trend = self.beta * (self.level - prev_level)
                + (1.0 - self.beta) * self.trend;
        }
        self.observations += 1;
    }

    pub fn forecast(&self, horizon: usize) -> f64 {
        self.level + (horizon as f64) * self.trend
    }
}

/// Tracks health across all configured LLM providers.
pub struct ProviderHealthTracker {
    providers: Mutex<HashMap<String, ProviderHealth>>,
}

impl ProviderHealthTracker {
    pub fn new() -> Self {
        Self { providers: Mutex::new(HashMap::new()) }
    }

    /// Record a completed LLM request.
    pub fn record_request(
        &self,
        provider: &str,
        latency_ms: u64,
        success: bool,
    ) {
        let mut providers = self.providers.lock();
        let health = providers
            .entry(provider.to_string())
            .or_default();

        // Update latency forecaster.
        health.latency_forecaster.update(latency_ms as f64);

        // Update error rate forecaster (1.0 = failure, 0.0 = success).
        health.error_rate_forecaster.update(if success { 0.0 } else { 1.0 });

        // Track consecutive failures.
        if success {
            health.consecutive_failures = 0;
            health.tripped = false;
        } else {
            health.consecutive_failures += 1;
            if health.consecutive_failures >= health.max_failures {
                health.tripped = true;
            }
        }

        // Maintain sliding window.
        health.recent_outcomes.push(success);
        if health.recent_outcomes.len() > health.max_window {
            health.recent_outcomes.remove(0);
        }
    }

    /// Check if a provider is healthy (not tripped, error rate trend < 0.5).
    pub fn is_healthy(&self, provider: &str) -> bool {
        let providers = self.providers.lock();
        match providers.get(provider) {
            None => true, // Unknown provider assumed healthy.
            Some(h) => {
                if h.tripped { return false; }
                // Check if error rate is trending toward 0.5.
                if h.error_rate_forecaster.observations >= 2
                    && h.error_rate_forecaster.forecast(1) >= 0.5
                {
                    return false;
                }
                true
            }
        }
    }

    /// Get the provider with the best health for routing.
    pub fn best_provider(&self, candidates: &[String]) -> Option<String> {
        let providers = self.providers.lock();
        candidates.iter()
            .filter(|p| {
                providers.get(p.as_str())
                    .map_or(true, |h| !h.tripped)
            })
            .min_by(|a, b| {
                let a_rate = providers.get(a.as_str())
                    .map_or(0.0, |h| h.error_rate_forecaster.forecast(1));
                let b_rate = providers.get(b.as_str())
                    .map_or(0.0, |h| h.error_rate_forecaster.forecast(1));
                a_rate.partial_cmp(&b_rate).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }
}
```

**Step 2: Wire into the LLM call path**

In `crates/ironclaw_llm/src/provider.rs` (or wherever LLM calls are dispatched), add health recording:

```rust
// In the LLM dispatch function, after each request completes:
let start = std::time::Instant::now();
let result = provider.complete(request).await;
let latency_ms = start.elapsed().as_millis() as u64;

health_tracker.record_request(
    provider.name(),
    latency_ms,
    result.is_ok(),
);

// Before selecting a provider:
if !health_tracker.is_healthy(provider.name()) {
    // Try the next provider in the fallback chain.
}
```

**Step 3: Add to `src/app.rs` startup**

```rust
let health_tracker = Arc::new(ProviderHealthTracker::new());
// Pass to LlmProvider construction.
```

### Phase 2: Tool Execution Monitoring

**Where**: `src/tools/dispatch.rs`
**What**: Apply circuit breaker pattern to tool execution. If an MCP tool or shell command starts returning identical errors, circuit-break it before burning through the retry budget.

**Step 1: Create `src/tools/health.rs`**

```rust
use std::collections::HashMap;
use parking_lot::Mutex;

/// Per-tool execution health tracker.
pub struct ToolHealthMonitor {
    /// Per-tool consecutive failure count.
    failures: Mutex<HashMap<String, u32>>,
    /// Maximum failures before tripping (default: 3).
    max_failures: u32,
    /// Tripped tools.
    tripped: Mutex<HashMap<String, bool>>,
}

impl ToolHealthMonitor {
    pub fn new(max_failures: u32) -> Self {
        Self {
            failures: Mutex::new(HashMap::new()),
            max_failures,
            tripped: Mutex::new(HashMap::new()),
        }
    }

    pub fn should_execute(&self, tool_name: &str) -> bool {
        !self.tripped.lock().get(tool_name).copied().unwrap_or(false)
    }

    pub fn record_success(&self, tool_name: &str) {
        self.failures.lock().remove(tool_name);
        self.tripped.lock().remove(tool_name);
    }

    pub fn record_failure(&self, tool_name: &str) -> bool {
        let mut failures = self.failures.lock();
        let count = failures.entry(tool_name.to_string()).or_default();
        *count += 1;
        if *count >= self.max_failures {
            self.tripped.lock().insert(tool_name.to_string(), true);
            true  // tripped
        } else {
            false
        }
    }

    pub fn reset(&self, tool_name: &str) {
        self.failures.lock().remove(tool_name);
        self.tripped.lock().remove(tool_name);
    }
}
```

**Step 2: Integrate in `src/tools/dispatch.rs`**

```rust
// In ToolDispatcher::dispatch():
if !self.health_monitor.should_execute(&tool_name) {
    return Err(ToolError::CircuitBroken {
        tool: tool_name,
        reason: "too many consecutive failures".into(),
    });
}

let result = tool.execute(params).await;
match &result {
    Ok(_) => self.health_monitor.record_success(&tool_name),
    Err(_) => {
        if self.health_monitor.record_failure(&tool_name) {
            tracing::warn!(tool = %tool_name, "tool circuit breaker tripped");
        }
    }
}
```

### Phase 3: System Health Dashboard

**Where**: `src/channels/web/` -- expose via SSE to the web UI
**What**: Use the `HealthMonitor` concept with `SystemSnapshot` to stream health status.

```rust
// In the SSE health endpoint handler:
pub async fn health_stream(state: &AppState) -> impl Stream<Item = Event> {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    async_stream::stream! {
        loop {
            interval.tick().await;
            let snapshot = SystemSnapshot {
                active_sessions: state.active_session_count(),
                provider_health: state.provider_health_summary(),
                tool_health: state.tool_health_summary(),
                memory_usage_mb: get_memory_usage_mb(),
            };
            let json = serde_json::to_string(&snapshot).unwrap_or_default();
            yield Event::default().data(json).event("health");
        }
    }
}
```

### Phase 4: Cost Control with Yerkes-Dodson

**Where**: `src/estimation/`
**What**: Use the Yerkes-Dodson pressure framework to dynamically adjust agent behavior based on resource pressure.

```rust
/// Compute pressure from current session state and adjust behavior.
pub fn compute_session_pressure(
    cost_spent: f64,
    budget: f64,
    elapsed_secs: f64,
    deadline_secs: f64,
    recent_failure_rate: f64,
    stuck_count: f64,
) -> PressureAssessment {
    let pressure = YerkesDodson::compute_pressure(
        cost_spent / budget.max(0.01),
        elapsed_secs / deadline_secs.max(1.0),
        recent_failure_rate,
        stuck_count,
    );

    let mut yd = YerkesDodson::default();
    yd.set_pressure(pressure);

    PressureAssessment {
        pressure,
        performance: yd.performance_multiplier(),
        aggressiveness: yd.intervention_aggressiveness(),
        in_danger_zone: yd.is_danger_zone(),
        recommended_action: if yd.is_danger_zone() {
            if pressure > 0.7 {
                PressureAction::SwitchToCheaperModel
            } else {
                PressureAction::IncreaseUrgency
            }
        } else {
            PressureAction::Continue
        },
    }
}
```

### Implementation Priority

| Phase | Effort | Impact | Dependencies |
|-------|--------|--------|-------------|
| Phase 1: Provider Health | 2-3 days | High | None |
| Phase 2: Tool Health | 1-2 days | Medium | None |
| Phase 3: Health Dashboard | 1-2 days | Low | Phase 1, 2 |
| Phase 4: Pressure Framework | 1 day | Medium | None |

All phases are independently implementable. Phase 1 has the highest impact because provider degradation is the most common source of wasted budget in production.

---

## 21. Complexity Assessment

### Lines of Code by Component

| Component | LOC (actual) | Source Files |
|-----------|-------------|-------------|
| Core conductor + evaluate | 989 | `conductor.rs` |
| 10 watchers | ~2,220 | `watchers/*.rs` (11 files) |
| Circuit breaker + Holt | 699 | `circuit_breaker.rs` |
| Interventions + BanditPolicy | 464 | `interventions.rs` |
| Pattern detector (CEP) | 326 | `pattern_detector.rs` |
| Threshold learner | 399 | `threshold_learner.rs` |
| Yerkes-Dodson | 248 | `yerkes_dodson.rs` |
| Federation | 289 | `federation.rs` |
| Self-healing | 379 | `self_healing.rs` |
| Diagnosis engine | 936 | `diagnosis.rs` |
| Health monitor | 641 | `health.rs` |
| Stuck detection | 2,004 | `stuck_detection.rs` |
| State machine | 218 | `state_machine.rs` |
| Module root | 89 | `lib.rs` |
| **Total roko-conductor** | **~10,100** | **24 files** |

### Risk Assessment

- **Risk**: Low -- purely additive monitoring layer with no side effects
- **Dependencies**: `roko-core` (for `Engram`, `React`, `ConductorDecision`), `roko-learn` (for `ConductorBandit`, `ProviderHealthTracker`, `AgentEfficiencyEvent`)
- **Threading**: All concurrent access uses `DashMap` (lock-free) or `parking_lot::Mutex` (fast, non-async)
- **Persistence**: Threshold learner and circuit breaker state persist to JSON files with atomic writes
- **Testing**: Extensive test coverage in every module (the source files are roughly 50% tests by line count)

### Roko Documentation References

| Topic | File |
|-------|------|
| Conductor overview | `crates/roko-conductor/README.md` |
| Resilience algebra + circuit breaker state machine | `docs/v2-depth/05-execution-engine/resilience-and-numerics.md` |
| Orchestrator integration status | `docs/v2/27-ORCHESTRATOR.md` (lines 511-515, 660-678) |
| Numerical precision (f64 for EMA) | `docs/v2-depth/05-execution-engine/resilience-and-numerics.md` (section 4.6) |
| Thompson sampling precision | `docs/v2-depth/05-execution-engine/resilience-and-numerics.md` (section 4.1) |
| Provider health and Pareto routing | `docs/v2-depth/10-learning-loops/provider-health-and-pareto.md` |
| Drift and stability | `docs/v2-depth/10-learning-loops/drift-and-stability.md` |
| Graceful degradation levels | `docs/v2-depth/05-execution-engine/resilience-and-numerics.md` (section 3) |

---

## 22. References

[1] M. Nygard, *Release It! Design and Deploy Production-Ready Software*. Pragmatic Bookshelf, 2007. Second edition, 2018. Introduced the circuit breaker pattern for software systems.

[2] Netflix Hystrix entered maintenance mode in November 2018 (final release: v1.5.18). Resilience4j is the successor library for JVM-based circuit breakers. See: [Hystrix Status](https://github.com/Netflix/Hystrix#hystrix-status).

[3] C. C. Holt, "Forecasting Seasonals and Trends by Exponentially Weighted Moving Averages," *International Journal of Forecasting*, vol. 20, no. 1, pp. 5-10, 2004. (Original ONR Research Memorandum No. 52, Carnegie Institute of Technology, 1957.) Introduced double exponential smoothing with level and trend components.

[4] P. R. Winters, "Forecasting Sales by Exponentially Weighted Moving Averages," *Management Science*, vol. 6, no. 3, pp. 324-342, 1960. Extended Holt's method to include a seasonal component (Holt-Winters triple exponential smoothing).

[5] R. J. Hyndman and G. Athanasopoulos, *Forecasting: Principles and Practice*, 3rd ed., OTexts, 2021. Available online at [otexts.com/fpp3](https://otexts.com/fpp3/). Chapter 8 covers exponential smoothing methods comprehensively.

[6] D. C. Luckham, *The Power of Events: An Introduction to Complex Event Processing in Distributed Enterprise Systems*. Addison-Wesley, 2002. Established the foundations of CEP, including event pattern languages, causal event hierarchies, and composition operators (conjunction, sequence, negation).

[7] W. R. Thompson, "On the Likelihood that One Unknown Probability Exceeds Another in View of the Evidence of Two Samples," *Biometrika*, vol. 25, no. 3-4, pp. 285-294, 1933. The original Thompson Sampling paper.

[8] S. Agrawal and N. Goyal, "Analysis of Thompson Sampling for the Multi-armed Bandit Problem," *Proceedings of the 25th Annual Conference on Learning Theory (COLT)*, JMLR: W&CP vol. 23, pp. 39.1-39.26, 2012. First proof that Thompson Sampling achieves logarithmic expected regret. Available at [proceedings.mlr.press/v23/agrawal12](https://proceedings.mlr.press/v23/agrawal12.html).

[9] R. M. Yerkes and J. D. Dodson, "The Relation of Strength of Stimulus to Rapidity of Habit-Formation," *Journal of Comparative Neurology and Psychology*, vol. 18, no. 5, pp. 459-482, 1908. The original study demonstrating the inverted-U relationship between stimulus intensity and performance.

[10] D. O. Hebb, "Drives and the C.N.S. (Conceptual Nervous System)," *Psychological Review*, vol. 62, no. 4, pp. 243-254, 1955. Generalized the Yerkes-Dodson finding into the inverted-U curve relating arousal to performance. Sometimes cited as "Hebb's curve."

[11] S. Beer, *Brain of the Firm*. Allen Lane, The Penguin Press, 1972. Second edition, Wiley, 1981. Introduced the Viable System Model (VSM) with its five recursive subsystems.

[12] S. Beer, *Diagnosing the System for Organizations*. Wiley, 1985. Practical guide to applying the VSM, including the recursive structure where viable systems contain viable subsystems operating at different temporal frequencies.

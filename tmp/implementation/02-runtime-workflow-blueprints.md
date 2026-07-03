# Runtime Workflow Blueprints

This document provides clean IronClaw-native sketches for workflow execution,
verification gates, event streams, and conductor-style anomaly detection.

## Cell And Graph Runtime

```rust
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeOutput {
    pub value: Value,
    pub tokens: u64,
    pub cost_microusd: u64,
    pub duration_ms: u64,
}

#[derive(Clone, Debug)]
pub struct CellContext {
    pub run_id: String,
    pub node_id: String,
    pub budget: BudgetTracker,
}

#[async_trait]
pub trait Cell: Send + Sync {
    fn kind(&self) -> &'static str;
    fn estimated_cost_microusd(&self) -> Option<u64> { None }
    async fn execute(&self, input: Vec<NodeOutput>, context: CellContext) -> anyhow::Result<NodeOutput>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EdgeCondition {
    Always,
    OnSuccess,
    OnFailure,
    JsonEquals { path: String, value: Value },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub cell_kind: String,
    pub config: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub condition: EdgeCondition,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphSpec {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Clone, Debug)]
pub struct BudgetLimits {
    pub max_tokens: Option<u64>,
    pub max_cost_microusd: Option<u64>,
    pub max_wall_time: Option<Duration>,
}

#[derive(Clone, Debug)]
pub struct BudgetTracker {
    started: Instant,
    limits: BudgetLimits,
    tokens: u64,
    cost_microusd: u64,
}

impl BudgetTracker {
    pub fn new(limits: BudgetLimits) -> Self {
        Self { started: Instant::now(), limits, tokens: 0, cost_microusd: 0 }
    }

    pub fn record(&mut self, output: &NodeOutput) {
        self.tokens += output.tokens;
        self.cost_microusd += output.cost_microusd;
    }

    pub fn check(&self) -> anyhow::Result<()> {
        if let Some(max) = self.limits.max_tokens {
            anyhow::ensure!(self.tokens <= max, "token budget exceeded: {}/{}", self.tokens, max);
        }
        if let Some(max) = self.limits.max_cost_microusd {
            anyhow::ensure!(self.cost_microusd <= max, "cost budget exceeded: {}/{} microusd", self.cost_microusd, max);
        }
        if let Some(max) = self.limits.max_wall_time {
            anyhow::ensure!(self.started.elapsed() <= max, "wall-time budget exceeded");
        }
        Ok(())
    }
}
```

IronClaw integration rule: a `ToolCell` must dispatch through
`src/tools/dispatch.rs::ToolDispatcher`, not call tools directly. That preserves
approval, audit, safety, and channel neutrality.

## Progressive Gate Contract

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GateInput {
    pub artifact_path: String,
    pub diff_summary: String,
    pub changed_files: Vec<String>,
    pub declared_scope: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GateVerdict {
    pub gate: String,
    pub passed: bool,
    pub confidence: f64,
    pub duration_ms: u64,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub remediation: Option<String>,
}

#[async_trait]
pub trait VerificationGate: Send + Sync {
    fn name(&self) -> &'static str;
    fn rung(&self) -> u8;
    async fn verify(&self, input: &GateInput) -> anyhow::Result<GateVerdict>;
}

pub struct GatePipeline {
    gates: Vec<Box<dyn VerificationGate>>,
}

impl GatePipeline {
    pub async fn run_until_failure(&self, input: &GateInput, max_rung: u8) -> Vec<GateVerdict> {
        let mut verdicts = Vec::new();
        for gate in self.gates.iter().filter(|g| g.rung() <= max_rung) {
            let verdict = gate.verify(input).await.unwrap_or_else(|error| GateVerdict {
                gate: gate.name().to_string(),
                passed: false,
                confidence: 1.0,
                duration_ms: 0,
                stdout_tail: String::new(),
                stderr_tail: error.to_string(),
                remediation: Some("Gate failed before producing a normal verdict".into()),
            });
            let passed = verdict.passed;
            verdicts.push(verdict);
            if !passed {
                break;
            }
        }
        verdicts
    }
}
```

Recommended initial rungs for IronClaw:

| Rung | Gate | Caller-level test target |
|---|---|---|
| 0 | Format/schema validation | Tool output handler or web handler |
| 1 | Compile/check | Generated-code caller |
| 2 | Lint/clippy | Same caller, not only gate helper |
| 3 | Unit test subset | Caller that triggered generation |
| 4 | Scope drift check | ToolDispatcher or effect adapter boundary |

## Conductor Watcher Core

```rust
#[derive(Clone, Debug)]
pub struct Observation {
    pub name: &'static str,
    pub value: f64,
    pub timestamp_ms: i64,
}

#[derive(Clone, Debug)]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

#[derive(Clone, Debug)]
pub struct Intervention {
    pub watcher: &'static str,
    pub severity: Severity,
    pub reason: String,
    pub suggested_action: SuggestedAction,
}

#[derive(Clone, Debug)]
pub enum SuggestedAction {
    Continue,
    Cooldown,
    SwitchProvider,
    ReduceModelTier,
    RestartTask,
    FailFast,
}

pub trait Watcher {
    fn name(&self) -> &'static str;
    fn observe(&mut self, obs: &Observation) -> Option<Intervention>;
}

#[derive(Clone, Debug)]
pub struct HoltForecaster {
    alpha: f64,
    beta: f64,
    level: Option<f64>,
    trend: f64,
}

impl HoltForecaster {
    pub fn new(alpha: f64, beta: f64) -> Self {
        Self { alpha, beta, level: None, trend: 0.0 }
    }

    pub fn update(&mut self, value: f64) {
        match self.level {
            None => self.level = Some(value),
            Some(prev_level) => {
                let level = self.alpha * value + (1.0 - self.alpha) * (prev_level + self.trend);
                self.trend = self.beta * (level - prev_level) + (1.0 - self.beta) * self.trend;
                self.level = Some(level);
            }
        }
    }

    pub fn forecast(&self, steps: f64) -> Option<f64> {
        self.level.map(|level| level + steps * self.trend)
    }
}

pub struct LatencyWatcher {
    forecaster: HoltForecaster,
    threshold_ms: f64,
}

impl Watcher for LatencyWatcher {
    fn name(&self) -> &'static str { "latency" }

    fn observe(&mut self, obs: &Observation) -> Option<Intervention> {
        if obs.name != "llm_latency_ms" {
            return None;
        }
        self.forecaster.update(obs.value);
        let forecast = self.forecaster.forecast(1.0)?;
        if forecast >= self.threshold_ms {
            Some(Intervention {
                watcher: self.name(),
                severity: Severity::Warning,
                reason: format!("forecast latency {:.0}ms exceeds {:.0}ms", forecast, self.threshold_ms),
                suggested_action: SuggestedAction::SwitchProvider,
            })
        } else {
            None
        }
    }
}
```

## Event Bus Shape

```rust
use tokio::sync::broadcast;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RuntimeEvent {
    TurnStarted { session_id: String, turn_id: String },
    ToolDispatched { tool: String, call_id: String },
    ToolCompleted { tool: String, call_id: String, duration_ms: u64 },
    LlmCallCompleted { provider: String, tokens: u64, cost_microusd: u64, latency_ms: u64 },
    GateCompleted { gate: String, passed: bool, confidence: f64 },
    InterventionRaised { watcher: String, severity: String, reason: String },
}

#[derive(Clone)]
pub struct RuntimeEventBus {
    tx: broadcast::Sender<RuntimeEvent>,
}

impl RuntimeEventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn publish(&self, event: RuntimeEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RuntimeEvent> {
        self.tx.subscribe()
    }
}
```

The first practical use is observability: emit from LLM calls, tool dispatch,
cost guard, and gate completion. Later, the same bus can drive TUI, web SSE,
benchmarks, and conductor watchers.

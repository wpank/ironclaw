# 23 -- Control Plane & API Server Architecture

> How roko separates the control plane from the agent runtime, exposes 100+ HTTP API routes through a centralized server, aggregates data from a fleet of per-agent sidecars, streams events over WebSocket and SSE, and bridges agents to a relay bus. A deep reference for IronClaw adoption.

---

## 1. Why AI Agent Systems Need a Control Plane

An AI agent is a loop: observe, reason, act. A control plane is everything around that loop that makes it observable, controllable, and composable. These are separate concerns for the same reason a Kubernetes control plane is separate from the containers it manages [1]. The separation of the control plane from the data plane is a foundational principle in distributed systems architecture, applied across network switches, service meshes, and container orchestrators [2].

**Observability.** Without a control plane, the only way to know what an agent is doing is to read its logs. A control plane publishes structured events (agent spawned, task started, gate passed, cost incurred) so dashboards, CI systems, and other agents can react in real time. The agent itself does not need to know who is watching. This follows the observer pattern [3], where subjects notify registered observers without coupling to their implementations.

**Control.** Agents should not manage their own lifecycle. Starting, stopping, pausing, restarting, and scaling agents are operational concerns. A control plane provides an API surface for these operations, separating the "what should happen" (the agent's goals) from the "how it runs" (process management, resource allocation, credential injection). This mirrors the separation of concerns in Kubernetes between the API server (desired state) and the kubelet (actual state reconciliation) [1].

**Aggregation.** A single agent is useful. A fleet of agents -- each with different capabilities, models, and specializations -- is a system. Someone has to maintain the roster, merge their outputs, track which ones are healthy, and present a unified view. That is the control plane's job. The aggregation pattern follows the API gateway design [4], where a single entry point serves as the reverse proxy to multiple backend services.

**Credential isolation.** Agents should never hold API keys directly. The control plane owns secrets and proxies inference requests, so a compromised agent cannot exfiltrate credentials. This is the same principle as a service mesh sidecar holding TLS certificates rather than the application process [5]. The principle of least privilege dictates that each component should have access only to the credentials it needs, and only for the duration it needs them.

**Learning and adaptation.** When the system learns from its own performance (model routing, gate threshold tuning, cost optimization), that learning state is global -- it crosses agent boundaries. The control plane is the natural home for this shared state.

Roko implements this separation through two crates: `roko-serve` (the centralized control plane) and `roko-agent-server` (the per-agent sidecar). Together they form a hub-and-spoke architecture where the serve process is the single entry point for all external consumers, and each agent gets its own lightweight HTTP server that reports back to the hub.

---

## 2. Architecture Overview

```
                    External consumers
                    (dashboard, CI, scripts)
                           |
                           v
              +---------------------------+
              |  roko-serve  (port 6677)  |
              |  REST + SSE + WebSocket   |
              |  100+ HTTP API routes     |
              |  AuthMiddleware           |
              |  SecretScrubber           |
              |  EventBus (broadcast)     |
              |  StateHub (projection)    |
              |  RateLimiter (governor)   |
              +-----+------+------+------+
                    |      |      |
          +---------+      |      +----------+
          |                |                 |
          v                v                 v
   +-----------+    +-----------+     +-----------+
   | Agent A   |    | Agent B   |     | Agent C   |
   | sidecar   |    | sidecar   |     | sidecar   |
   | :rand_port|    | :rand_port|     | :rand_port|
   +-----------+    +-----------+     +-----------+
        |                |                 |
        +--------+-------+---------+------+
                 |                  |
                 v                  v
          +-------------+   +--------------+
          | agent-relay  |   | LLM providers|
          | (pub/sub bus)|   | (via gateway)|
          +--------------+   +--------------+
```

**File paths (all relative to roko repo root):**

| Component | Path |
|-----------|------|
| Control plane crate | `crates/roko-serve/` |
| Per-agent sidecar crate | `crates/roko-agent-server/` |
| Relay protocol | `crates/agent-relay/` |
| Control plane entry point | `crates/roko-serve/src/lib.rs` |
| Route definitions | `crates/roko-serve/src/routes/mod.rs` |
| AppState (shared state) | `crates/roko-serve/src/state.rs` |
| Event types | `crates/roko-serve/src/events.rs` |
| Event bus | `crates/roko-serve/src/event_bus.rs` |
| Agent sidecar entry | `crates/roko-agent-server/src/lib.rs` |
| Agent sidecar state | `crates/roko-agent-server/src/state.rs` |

---

## 3. The Control Plane: `roko-serve`

### 3.1 AppState -- The Shared Kernel

Every axum handler receives `State<Arc<AppState>>`. The `AppState` struct is defined in `crates/roko-serve/src/state.rs` and holds all shared runtime state:

```rust
// crates/roko-serve/src/state.rs (condensed from actual source)
pub struct AppState {
    /// Project working directory.
    pub workdir: PathBuf,

    /// `.roko/` directory layout helper.
    pub layout: RokoLayout,

    /// Lazily initialized `.roko/engrams.jsonl` writer.
    pub signal_store: SignalStore,

    /// Cancellation token for graceful shutdown.
    pub cancel: CancelToken,

    /// Monotonic timestamp when the server state was created.
    pub started_at: Instant,

    /// Prometheus-compatible metric registry.
    pub metrics: Arc<MetricRegistry>,

    /// Process lifecycle manager.
    pub supervisor: Arc<ProcessSupervisor>,

    /// Affect engine used to stamp PAD vectors onto persisted episodes.
    pub affect_engine: Mutex<DaimonState>,

    /// Event bus for streaming server events to clients.
    pub event_bus: EventBus<ServerEvent>,

    /// Unified state hub for dashboard snapshot + event streaming.
    pub state_hub: roko_runtime::SharedStateHub,

    /// RuntimeEvent SSE adapter for workflow event streaming.
    pub sse_adapter: Arc<SseAdapter>,

    /// Event subscriptions loaded at startup.
    pub subscriptions: SubscriptionRegistry,

    /// Runtime bridge to CLI operations (run_once, status, dashboard).
    pub runtime: Arc<dyn CliRuntime>,

    /// Shared model-call gateway service used by HTTP inference adapters.
    pub model_call_service: Arc<ModelCallService>,

    /// Full `roko.toml` schema configuration with lock-free reads.
    pub roko_config: ArcSwap<RokoConfig>,

    /// In-memory provider health tracker exposed via serve APIs.
    pub provider_health: ProviderHealthTracker,

    /// In-memory provider latency stats exposed via serve APIs.
    pub latency_registry: LatencyRegistry,

    /// Active one-shot runs.
    pub active_runs: RwLock<HashMap<String, RunHandle>>,

    /// Active plan executions.
    pub active_plans: RwLock<HashMap<String, PlanHandle>>,

    /// Active generic operations.
    pub operations: RwLock<HashMap<String, OperationHandle>>,

    /// Agent template registry.
    pub templates: RwLock<TemplateRegistry>,

    /// Cloud deploy backend (Railway, CLI, manual).
    pub deploy_backend: Arc<dyn DeployBackend>,

    /// Active cloud deployments.
    pub deployments: RwLock<HashMap<String, Deployment>>,

    /// Secret scrubber for redacting API-key / token patterns from responses.
    pub scrubber: Arc<LogScrubber>,

    /// Shared HTTP client for aggregator fan-out.
    pub http_client: reqwest::Client,

    /// Discovery registry for local and chain-discovered agents.
    pub discovered_agents: RwLock<HashMap<String, DiscoveredAgent>>,

    /// Short-lived aggregator cache keyed by route + query signature.
    pub aggregator_cache: RwLock<HashMap<String, CachedJsonValue>>,

    /// Ring buffer of recent heartbeat payloads.
    pub heartbeats: RwLock<VecDeque<HeartbeatPayload>>,

    /// Optional alloy JSON-RPC client for on-chain reads.
    pub chain_client: Option<Arc<AlloyChainClient>>,

    /// Optional alloy wallet for on-chain writes.
    pub chain_wallet: Option<Arc<AlloyChainWallet>>,

    /// Agent relay URL for relay proxy.
    pub agent_relay_url: Option<String>,

    // ... additional fields for bench runs, matrix runs, feeds, etc.
}
```

Key design choices:
- **`ArcSwap<RokoConfig>`** for hot-reloadable configuration without restarts. `ArcSwap` provides atomic pointer swaps, allowing concurrent readers with zero contention [6].
- **`EventBus<ServerEvent>`** backed by `tokio::sync::broadcast` with a replay ring buffer for reconnection support (see section 3.3).
- **`SharedStateHub`** as a projection engine for the dashboard -- accumulates structured events and lets clients subscribe to filtered delta streams.
- **`Arc<ProcessSupervisor>`** from `roko-runtime` to spawn and manage agent processes.
- **`RwLock<HashMap<...>>`** pattern for concurrent access to mutable collections (plans, runs, agents). The lock acquisition order is documented in the source to prevent deadlocks: `active_runs` before `active_plans` before `operations` before `templates`, etc.
- **`RwLock<VecDeque<HeartbeatPayload>>`** for heartbeats, implementing a bounded ring buffer with `HEARTBEAT_RING_CAPACITY` eviction, not an unbounded HashMap.
- **`CancelToken`** for coordinated graceful shutdown, exposed through `GET /ready` which returns 503 once cancellation is triggered.
- **`Arc<LogScrubber>`** for response-side secret redaction across all API routes.

### 3.2 Event Types -- The ServerEvent Enum

All events flow through a single tagged union defined in `crates/roko-serve/src/events.rs`. There are **60+ event variants** (106 `match` arms across both `ExecutionEvent` and `ServerEvent`) covering every aspect of system operation:

```rust
// crates/roko-serve/src/events.rs (condensed from actual source)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    // Plan lifecycle
    PlanStarted { plan_id: String },
    PlanCompleted { plan_id: String, success: bool },

    // Agent lifecycle
    AgentSpawned { agent_id: String, role: String, #[serde(default)] model: String },
    AgentStarted { agent_id: String },
    AgentStopped { agent_id: String, reason: String },

    // Agent output (streamed, sanitized)
    AgentOutput {
        agent_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        run_id: Option<String>,
        content: String,
        #[serde(default)]
        done: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        metadata: Option<Value>,
    },

    // Raw trace (unsanitized, opt-in subscription)
    AgentTrace {
        agent_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        run_id: Option<String>,
        content: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<Value>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reasoning: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        usage: Option<Value>,
        #[serde(default)]
        done: bool,
    },

    // Gate verification
    GateResult { plan_id: String, task_id: String, gate: String,
                 #[serde(default)] rung: u32, passed: bool },

    // Execution progress
    Execution { plan_id: String, event: ExecutionEvent },
    PhaseTransition { plan_id: String, from: String, to: String },
    Episode { plan_id: String, task_id: String, passed: bool },
    EfficiencyEvent { plan_id: String, task_id: String, metric: String, value: f64 },

    // Inference gateway
    InferenceStarted { request_id: String, model: String,
                       #[serde(default)] agent_id: String,
                       #[serde(default)] auto_routed: bool },
    InferenceCompleted { request_id: String, model: String,
                         #[serde(default)] agent_id: String,
                         input_tokens: u64, output_tokens: u64,
                         cost_usd: f64, duration_ms: u64 },
    InferenceFailed { request_id: String, model: String,
                      #[serde(default)] agent_id: String, error: String },

    // Somatic markers (affect engine)
    SomaticMarkerFired { plan_id: String, task_id: String,
                         valence: f64, intensity: f64,
                         source_episodes: Vec<ContentHash>,
                         strategy_param: String },

    // One-shot runs
    RunStarted { run_id: String, #[serde(rename = "prompt_preview")] prompt: String },
    RunCompleted { run_id: String, success: bool },

    // Operations
    OperationStarted { op_id: String, kind: String },
    OperationCompleted { op_id: String, kind: String, success: bool },

    // Cloud deployments
    DeploymentCreated { id: String, name: String },
    DeploymentReady { id: String, url: String },
    DeploymentFailed { id: String, reason: String },
    DeploymentTornDown { id: String },

    // Marketplace jobs
    JobCreated { job: Value },
    JobUpdated { job: Value },
    JobTransitioned { job_id: String, from: String, to: String,
                      assigned_to: Option<String> },
    JobPostedToCandidate { job_id: String, agent_id: String, reward: String },
    JobSubmitted { job_id: String, agent_id: String },
    JobEvaluated { job_id: String, accepted: bool, feedback: String },
    JobStateChanged { job_id: String, from: String, to: String },

    // Job execution lifecycle
    JobExecutionStarted { job_id: String, job_type: String, agent_id: String },
    JobProgress { job_id: String, percent: u8, message: String },
    JobAgentOutput { job_id: String, agent_id: String, content: String, done: bool },
    ChainTriageResult { job_id: String, event_count: usize,
                        anomaly_count: usize, summary: String },

    // Worker tasks
    WorkerTaskStarted { deployment_id: String, task_id: String },
    WorkerTaskCompleted { deployment_id: String, task_id: String, success: bool },

    // Dashboard-facing task events
    TaskStarted { plan_id: String, task_id: String, description: String },
    TaskCompleted { plan_id: String, task_id: String, success: bool },
    TaskFailed { plan_id: String, task_id: String, error: String },

    // Benchmark runs
    #[serde(rename = "BenchRunStarted")]
    BenchRunStarted { bench_id: String, suite_id: String, total_tasks: usize },
    #[serde(rename = "BenchTaskStarted")]
    BenchTaskStarted { bench_id: String, task_id: String, task_name: String,
                       task_index: usize, total_tasks: usize },
    #[serde(rename = "BenchTaskCompleted")]
    BenchTaskCompleted { bench_id: String, task_id: String, result: Value },
    #[serde(rename = "BenchProgress")]
    BenchProgress { bench_id: String, completed: usize, total: usize, cost_so_far: f64 },
    #[serde(rename = "BenchRunCompleted")]
    BenchRunCompleted { bench_id: String, summary: Value },
    #[serde(rename = "BenchLearningEvent")]
    BenchLearningEvent { bench_id: String, task_id: String,
                         playbooks_created: u32, anti_patterns_created: u32,
                         total_playbooks: u32, total_anti_patterns: u32 },
    #[serde(rename = "BenchGateVerdict")]
    BenchGateVerdict { bench_id: String, task_id: String, gate: String,
                       passed: bool, message: Option<String>, duration_ms: u64 },
    #[serde(rename = "BenchTokenVelocity")]
    BenchTokenVelocity { bench_id: String, task_id: String,
                         tokens_per_second: f64, tokens_in: u64,
                         tokens_out: u64, duration_ms: u64 },
    #[serde(rename = "BenchAgentOutput")]
    BenchAgentOutput { bench_id: String, task_id: String, agent_id: String,
                       content: String, done: bool,
                       tool_calls: Option<Vec<Value>>,
                       reasoning: Option<String> },
    #[serde(rename = "BenchRegressionReport")]
    BenchRegressionReport { bench_id: String, has_regressions: bool,
                            report: Value },

    // SWE-bench
    #[serde(rename = "SweRunStarted")]
    SweRunStarted { run_id: String, dataset: String, total_instances: usize },
    #[serde(rename = "SweInstanceCompleted")]
    SweInstanceCompleted { run_id: String, instance_id: String,
                           resolved: bool, duration_ms: u64 },
    #[serde(rename = "SweRunCompleted")]
    SweRunCompleted { run_id: String, resolved: u32, total: u32, pass_rate: f64 },

    // Matrix (multi-lane) bench
    #[serde(rename = "MatrixRunStarted")]
    MatrixRunStarted { matrix_id: String, suite_id: String,
                       lane_ids: Vec<String>, total_lanes: usize },
    #[serde(rename = "MatrixLaneCompleted")]
    MatrixLaneCompleted { matrix_id: String, lane_id: String,
                          pass_rate: f64, cost_usd: f64 },
    #[serde(rename = "MatrixRunCompleted")]
    MatrixRunCompleted { matrix_id: String, summary: Vec<Value> },

    // Chain events
    ChainBlock { number: u64, hash: String, parent_hash: String,
                 timestamp: u64, gas_used: u64, gas_limit: u64,
                 tx_count: u32, base_fee_per_gas: Option<u64> },
    ChainTx { block_number: u64, tx_hash: String, from: String,
              to: Option<String>, value_wei: String, gas_used: u64,
              method_sig: Option<String>, success: bool },
    ChainContractEvent { block_number: u64, tx_hash: String, log_index: u32,
                         contract: String, event_name: String,
                         decoded: Value },

    // Feed data
    FeedTick { agent_id: String, feed_id: String, topic: String,
               payload: Value, timestamp_ms: i64 },
    FeedAgentOnline { agent_id: String, name: String, feed_count: usize },
    FeedAgentOffline { agent_id: String },

    // Heartbeats
    HeartbeatReceived { sender_id: String, active_tasks: usize, active_agents: usize },
    Heartbeat { agent_id: String, block_number: Option<u64> },

    // ISFR (DeFi reference rate)
    IsfrRateComputed { composite_bps: u64, lending_bps: u64, structured_bps: u64,
                       funding_bps: u64, staking_bps: u64, confidence_bps: u64,
                       source_count: usize, timestamp_ms: i64 },
    IsfrSourceHealthChanged { source_id: String, health: String,
                              last_rate_bps: Option<u64> },
    IsfrKeeperStateChanged { running: bool },

    // Vision loop
    VisionLoopIteration { run_id: String, iteration: u32, score: f64, notes: String },
    VisionLoopCompleted { run_id: String, iterations: u32,
                          best_score: f64, stop_reason: String },

    // Configuration
    ConfigReloaded { applied_sections: Vec<String>, restart_required: Vec<String> },
    StrategyReloaded { goals_count: usize, tactics_count: usize },

    // Webhook signals
    WebhookReceived { signal: Engram },

    // Lifecycle
    ServerShutdown,
    Error { message: String },
}
```

Most variants use `#[serde(tag = "type", rename_all = "snake_case")]`, so they serialize as `{"type": "agent_output", "agent_id": "...", ...}`. The bench-related variants use explicit `#[serde(rename = "BenchRunStarted")]` to preserve PascalCase for dashboard compatibility.

There is also a nested `ExecutionEvent` enum with its own variants (`PlanStarted`, `TaskStarted`, `TaskPhaseChanged`, `GateResult`, `TaskCompleted`, `PlanCompleted`, `ReplanTriggered`, `WatcherAlert`) that gets wrapped inside the `Execution` variant of `ServerEvent`.

### 3.3 EventBus

The `EventBus` in `crates/roko-serve/src/event_bus.rs` is a thin wrapper around `roko_runtime::event_bus::EventBus`. It combines `tokio::sync::broadcast` with a replay ring buffer for reconnection support:

```rust
// crates/roko-serve/src/event_bus.rs (actual source)
pub use roko_runtime::event_bus::Envelope;

pub type Receiver<E> = tokio::sync::broadcast::Receiver<Envelope<E>>;

#[derive(Clone)]
pub struct EventBus<E: Clone + Send + Sync + 'static> {
    inner: roko_runtime::event_bus::EventBus<E>,
}

impl<E: Clone + Send + Sync + 'static> EventBus<E> {
    /// Create a new event bus with the given replay capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: roko_runtime::event_bus::EventBus::new(capacity),
        }
    }

    /// Publish an event to all live subscribers and record it for replay.
    /// Returns the sequence number assigned to the event.
    pub fn publish(&self, event: E) -> u64 {
        self.inner.emit(event)
    }

    /// Subscribe to live events.
    pub fn subscribe(&self) -> Receiver<E> {
        self.inner.subscribe()
    }

    /// Return a snapshot of events published after `after_seq`.
    pub fn replay_from(&self, after_seq: u64) -> Vec<Envelope<E>> {
        self.inner.replay_from(after_seq)
    }
}
```

This is more than simple fire-and-forget broadcasting. Key differences from a bare `broadcast::Sender`:

- **Sequence numbers**: every published event gets a monotonically increasing `u64` sequence number, enabling cursor-based reconnection.
- **Replay ring**: the bus maintains a bounded ring buffer of recent events. Clients that reconnect with a `Last-Event-ID` can replay missed events from the ring without re-fetching from the database.
- **Envelope wrapping**: events are wrapped in an `Envelope<E>` containing `seq: u64`, `ts_millis: i64`, and `payload: E`.
- **SSE replay cap**: the SSE handler caps replays at 256 events to bound memory pressure from reconnecting clients.

The broadcast channel has a configurable capacity; lagged receivers get `RecvError::Lagged(skipped)` with the count of missed messages.

---

## 4. HTTP API Routes (100+ Endpoints)

The route tree is assembled in `crates/roko-serve/src/routes/mod.rs` via `build_router()`. All API routes are nested under `/api/` with auth and secret-scrubbing middleware applied. Routes are organized into 40+ functional modules, each with its own `routes()` function returning a `Router<Arc<AppState>>`. A global rate limiter (governor, 100 req/s) and a 4 MiB request body cap apply to all routes.

### 4.1 Route Registration

```rust
// crates/roko-serve/src/routes/mod.rs (condensed from actual source)
pub fn build_router(
    state: Arc<AppState>,
    cors_origins: &[String],
    api_auth: ServeAuthConfig,
) -> Router {
    // Wire up SSE adapter and state hub bridge
    state.sse_adapter.set_state_hub_consumer(
        crate::dashboard_event_bridge(&state)
    );
    state.sse_adapter.start_runtime_event_subscription();

    let api = Router::new()
        .merge(crate::openapi::routes())  // OpenAPI spec
        .merge(status::routes())          // Health, metrics, dashboard, gates, episodes
        .merge(jobs::routes())            // Marketplace job management
        .merge(heartbeats::routes())      // Heartbeat ingestion + network stats
        .merge(plans::routes())           // Plan CRUD, execution, estimation
        .merge(prds::routes())            // PRD CRUD
        .merge(run::routes())             // One-shot runs
        .merge(runs::routes())            // Dashboard runs
        .merge(research::routes())        // Research queries
        .merge(subscriptions::routes())   // Subscription CRUD
        .merge(templates::routes())       // Template CRUD and deployment
        .merge(aggregator::routes())      // Fleet aggregation from discovered agents
        .merge(agents::routes())          // Agent management, registration, tokens
        .merge(learning::routes())        // Learning data (efficiency, cascade, experiments)
        .merge(config::routes())          // Configuration read/write/reload
        .merge(deployments::routes())     // Cloud deployment management
        .merge(diagnosis::routes())       // Diagnosis/debug
        .merge(integrations::routes())    // Integration management
        .merge(projections::routes())     // StateHub projections
        .merge(neuro::routes())           // Knowledge store queries
        .merge(dream::routes())           // Dream consolidation cycle
        .merge(event_ingest::routes())    // Event ingestion
        .merge(gateway::routes())         // Inference gateway
        .merge(chain::routes())           // On-chain proxy
        .merge(connectors::routes())      // Connector management
        .merge(feeds::routes())           // Feed management
        .merge(isfr::routes())            // ISFR (DeFi reference rate)
        .merge(auth::routes())            // Auth management
        .merge(secrets::routes())         // Secret management
        .merge(vision_loop::routes())     // Vision loop runs
        .merge(team::routes())            // Team management
        .merge(bench::routes())           // Benchmark runs
        .merge(swe_bench::routes())       // SWE-bench runs
        .merge(workflows::routes())       // Workflow read model + streaming
        .merge(workspaces::routes())      // Workspace management
        .merge(shared_runs::auth_routes())     // Shared run management
        .merge(webhooks::authenticated_routes()) // Authenticated webhooks
        .nest("/providers", providers::router())     // Provider inventory
        .nest("/models", providers::models_router()) // Model list
        .nest("/routing", providers::routing_router()) // Routing decisions
        .merge(sse::routes())             // SSE event stream
        .merge(rpc_proxy::routes())       // RPC proxy
        .route("/workflow/events", get(workflow_sse_handler));

    let api = if api_auth.enabled {
        api.layer(axum::middleware::from_fn(middleware::require_scope))
            .layer(axum::middleware::from_fn_with_state(
                Arc::clone(&state),
                middleware::require_api_key,
            ))
    } else {
        api
    };

    // Secret-scrubbing layer: redacts API keys / tokens from JSON responses.
    let scrubber = Arc::clone(&state.scrubber);
    let api = api.layer(axum::middleware::from_fn_with_state(
        scrubber,
        middleware::scrub_secrets,
    ));

    // WebSocket routes -- separate from /api/ prefix, may require auth
    let ws = if api_auth.enabled {
        ws::routes().layer(axum::middleware::from_fn_with_state(
            Arc::clone(&state),
            middleware::require_api_key,
        ))
    } else {
        ws::routes()
    };

    // Public routes (outside /api/ -- no auth required)
    let router = Router::new()
        .route("/health", get(top_level_health))
        .route("/ready", get(top_level_ready))
        .route("/metrics", get(metrics::metrics_handler))
        .merge(webhooks::public_routes())
        .merge(shared_runs::public_routes())
        .nest("/api", api)
        .merge(ws)
        .merge(relay_proxy::routes())
        .fallback(crate::embedded::serve_embedded);

    // Global rate limiter + body cap + tracing + CORS
    let rate_limiter = build_global_rate_limiter(DEFAULT_GLOBAL_RATE_PER_SEC);
    router
        .layer(DefaultBodyLimit::max(DEFAULT_REQUEST_BODY_LIMIT_BYTES))
        .layer(axum::middleware::from_fn_with_state(
            rate_limiter, rate_limit_middleware,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
```

### 4.2 Route Catalog by Category

#### Health & Status

```
GET  /health                          # Top-level liveness (public, no auth)
GET  /ready                           # Readiness probe (returns 503 during shutdown)
GET  /metrics                         # Prometheus-format scraping (public)
GET  /api/health                      # Detailed health (version, uptime, counts)
GET  /api/status                      # Dashboard session status
GET  /api/relay/health                # Relay health check
GET  /api/retention                   # Data retention status
GET  /api/parity                      # Data parity check
GET  /api/statehub/snapshot           # StateHub ring buffer snapshot
GET  /api/statehub/events             # StateHub event stream
```

#### Metrics

```
GET  /api/metrics                     # Full metrics
GET  /api/metrics/summary             # Active plans, c-factor, experiments
GET  /api/metrics/success_rate        # Pass rate over time
GET  /api/metrics/engagement          # Agent engagement metrics
GET  /api/metrics/c_factor            # Composite c-factor breakdown
GET  /api/metrics/model_efficiency    # Per-model cost/speed/quality
GET  /api/metrics/gate_rate           # Gate pass rate trends
GET  /api/metrics/experiments         # Experiment metrics
GET  /api/metrics/feedback_latency    # Feedback loop latency
GET  /api/metrics/velocity            # Development velocity
GET  /api/metrics/coverage            # Test coverage tracking
GET  /api/metrics/prometheus          # Prometheus-format (alias)
```

#### Plans

```
GET  /api/plans                       # List all plans
POST /api/plans                       # Create a new plan
GET  /api/plans/{id}                  # Get a specific plan
GET  /api/plans/{id}/tasks            # List tasks for a plan
POST /api/plans/{id}/execute          # Execute a plan (background)
GET  /api/plans/{id}/status           # Plan execution status
POST /api/plans/{id}/pause            # Pause execution
POST /api/plans/{id}/resume           # Resume execution
GET  /api/plans/{id}/gates            # Gate verdicts for plan
GET  /api/plans/{id}/costs            # Cost breakdown
GET  /api/plans/{id}/reviews          # List reviews
POST /api/plans/{id}/tasks/{tid}/review  # Submit task review
GET  /api/plans/{id}/tasks/{tid}/diff # Task diff
POST /api/plans/{id}/chat             # Chat about a plan
POST /api/plans/{id}/estimate         # Estimate plan cost/time
POST /api/plans/generate              # Generate plan from description
```

#### Runs

```
POST /api/run                         # Start a one-shot run
GET  /api/run/{id}/status             # Run status
GET  /api/runs                        # List dashboard runs
POST /api/runs                        # Start a dashboard run
```

#### Agent Management

```
GET  /api/managed-agents              # List all managed + discovered agents
POST /api/agents/register             # Register an agent
POST /api/agents/create               # Create (spawn) an agent
GET  /api/agents/{id}                 # Get agent details
GET  /api/agents/{id}/profile         # Agent profile
GET  /api/agents/{id}/config          # Agent configuration
POST /api/agents/{id}/stop            # Stop an agent
POST /api/agents/{id}/start           # Start an agent
POST /api/agents/{id}/restart         # Restart an agent
GET  /api/agents/{id}/episodes        # Agent episodes
GET  /api/agents/{id}/logs            # Proxy agent logs
POST /api/agents/{id}/message         # Send message to agent
GET  /api/agents/{id}/token           # Token status
POST /api/agents/{id}/token           # Issue agent token
```

#### Fleet Aggregation (20+ routes)

The aggregator module (`crates/roko-serve/src/routes/aggregator.rs`) provides routes that query all discovered agent sidecars and merge responses. The actual route set is significantly larger than a basic agent list:

```
GET  /api/agents                      # Aggregated agent list
GET  /api/agents/topology             # Agent topology graph
GET  /api/agents/{id}/stats           # Aggregated agent stats
GET  /api/agents/{id}/skills          # Aggregated agent skills
GET  /api/agents/{id}/heartbeat       # Agent heartbeat data
GET  /api/agents/{id}/trace           # Agent trace data
GET  /api/predictions/sessions        # Prediction sessions list
GET  /api/predictions/sessions/{id}   # Prediction session detail
GET  /api/predictions/claims          # Prediction claims list
GET  /api/predictions/calibration/{agent_id}  # Calibration analysis
GET  /api/knowledge/entries           # Knowledge entries
GET  /api/knowledge/edges             # Knowledge graph edges
GET  /api/knowledge/search            # Knowledge search
GET  /api/knowledge/kinds             # Knowledge entry kinds
GET  /api/tasks                       # Aggregated task queue
GET  /api/tasks/stats                 # Task statistics
GET  /api/tasks/{id}                  # Individual task detail
GET  /api/ws                          # Multiplexed WebSocket stream
```

#### Benchmarks (18+ routes)

```
GET  /api/bench/provider-status       # LLM provider availability
POST /api/bench/run                   # Start a bench run
POST /api/bench/runs                  # Start a bench run (alias)
GET  /api/bench/run/{id}              # Get bench run details
GET  /api/bench/runs/{id}             # Get bench run details (alias)
GET  /api/bench/run/{id}/status       # Bench run status
DEL  /api/bench/run/{id}              # Delete a bench run
DEL  /api/bench/runs/{id}             # Delete (alias)
POST /api/bench/runs/{id}/cancel      # Cancel a bench run
GET  /api/bench/runs                  # List bench runs
GET  /api/bench/runs/compare          # Compare bench runs
GET  /api/bench/cost-summary          # Cost summary across runs
GET  /api/bench/suites                # List bench suites
GET  /api/bench/suites/{id}           # Get suite details
POST /api/bench/suites                # Upload a suite
GET  /api/bench/models                # List available models
GET  /api/bench/pareto                # Pareto frontier analysis
GET  /api/bench/export/{id}           # Export bench run
GET  /api/bench/events                # Bench events SSE stream
```

#### Remaining Categories

| Category | Routes | Key Endpoints |
|----------|--------|--------------|
| Inference Gateway | 5 | complete, stats, models, batch submit/status |
| Learning & Adaptation | 16+ | Efficiency, costs, provider outcomes, retries, cascade, experiments, thresholds |
| Gates & Episodes | 5 | Gate pass rates, history, per-gate history, episodes, signals |
| Configuration | 4 | Config read/write/reload + raw TOML |
| Subscriptions | 7 | CRUD + enable/disable + catalog |
| Templates | 4 | CRUD + deploy |
| Deployments | 6 | CRUD + logs + task proxy + callback |
| Jobs | 11 | CRUD + state machine (open -> assigned -> in_progress -> submitted -> completed) |
| Webhooks | 3 | GitHub, Slack (verified), generic (authenticated) |
| Heartbeats | 3 | POST /api/heartbeats, GET /api/heartbeats, GET /api/network/stats |
| Secrets | 4 | List + set + delete + test |
| Knowledge (Neuro) | 2 | POST query + GET query |
| Research | 1 | POST query |
| Chain | 7 | Agents, bounties, status, blocks, transactions, events, watcher |
| Connectors | 4 | CRUD + health |
| Feeds | 7 | CRUD + catalog + runtime status |
| ISFR | 4 | Status, current rate, history, sources |
| Projections | 3 | Catalog, get, stream |
| Workflows | 7 | List, latest, stream, by-id, tasks, stream-by-id, WebSocket |
| Dreams | 2 | Run + journal |
| PRDs | ~4 | CRUD |
| SWE-bench | 3+ | Run, instances, status |
| Vision Loop | 2+ | Start + status |
| Teams | 2+ | Management |
| Workspaces | 2+ | Management |
| Providers | 3 | List, health, test |
| Models | 1 | List |
| Routing | 1 | Explain routing decision |
| SSE | 2 | GET /api/events, GET /api/sse |
| WebSocket | 2 | GET /ws, GET /roko-ws |
| Relay Proxy | 4 | HTTP catch-all + 2 WebSocket proxies + root |
| OpenAPI | 1+ | OpenAPI spec |

---

## 5. Real-Time Streaming

### 5.1 Server-Sent Events (SSE)

The SSE endpoints at `GET /api/events` and `GET /api/sse` provide persistent HTTP connections that stream events. SSE is ideal for this use case because it works over standard HTTP, survives proxy buffering with proper headers, and supports automatic reconnection via `Last-Event-ID` [7].

```rust
// crates/roko-serve/src/routes/sse.rs (actual source)
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/events", get(sse_handler))
        .route("/sse", get(sse_handler))
}

async fn sse_handler(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let last_event_id = headers
        .get("Last-Event-ID")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);

    // Cap replay to 256 events to prevent memory pressure
    let replay = state
        .state_hub
        .replay_from(last_event_id)
        .into_iter()
        .take(256)
        .map(|envelope| {
            let data = serde_json::to_string(&envelope.payload).unwrap_or_default();
            Ok::<_, Infallible>(Event::default()
                .data(data)
                .id(envelope.seq.to_string()))
        });

    let live = stream::unfold(
        state.state_hub.subscribe_events(),
        |mut rx| async move {
            loop {
                match rx.recv().await {
                    Ok(envelope) => {
                        let data = serde_json::to_string(&envelope.payload)
                            .unwrap_or_default();
                        let event = Event::default()
                            .data(data)
                            .id(envelope.seq.to_string());
                        return Some((Ok(event), rx));
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(n, "SSE client lagged, skipped events");
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => return None,
                }
            }
        },
    );

    // Keep-alive every 8 seconds to survive aggressive proxy timeouts
    let sse = Sse::new(stream::iter(replay).chain(live)).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(8))
            .text("keepalive"),
    );

    (sse_response_headers(), sse)
}
```

Key implementation details:

- **Replay on reconnect**: clients send `Last-Event-ID` header, and the server replays from the state hub's ring buffer (capped at 256 events).
- **Monotonic event IDs**: each SSE frame carries an `id:` field from the event bus sequence number, enabling reliable reconnection.
- **Anti-buffering headers**: the response includes `X-Accel-Buffering: no`, `Cache-Control: no-cache, no-store, no-transform, must-revalidate`, and `Connection: keep-alive` to disable HTTP/2 proxy buffering (Railway, Nginx, Cloudflare).
- **8-second keep-alive**: shorter than the default 15s to survive aggressive proxy timeouts (Railway 30s, Nginx 60s).

### 5.2 WebSocket Streaming

The WebSocket endpoints at `GET /ws` and `GET /roko-ws` provide bidirectional communication with client-side filtering and cursor-based resume:

```rust
// crates/roko-serve/src/routes/ws.rs (actual source)
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/ws", get(ws_upgrade))
        .route("/roko-ws", get(ws_upgrade))
}

/// Max message: 1 MiB, max frame: 256 KiB.
pub(crate) const WS_MAX_MESSAGE_SIZE: usize = 1024 * 1024;
pub(crate) const WS_MAX_FRAME_SIZE: usize = 256 * 1024;

pub(crate) fn apply_ws_size_limits(ws: WebSocketUpgrade) -> WebSocketUpgrade {
    ws.max_message_size(WS_MAX_MESSAGE_SIZE)
        .max_frame_size(WS_MAX_FRAME_SIZE)
}
```

The WebSocket handler supports a rich client control protocol:

```json
{
    "subscribe": ["projection:gate_pipeline", "topic:agent.*"],
    "cursor": 42,
    "back_pressure": "at_most_once"
}
```

- **Subscription filtering**: clients send `subscribe` arrays with pattern strings that filter which events they receive. Patterns support `projection:<name>`, `topic:<pattern>` (with `*` wildcard), `engram-stream:<name>`, or plain substring matching against event type tags.
- **Cursor-based replay**: clients can resume from a specific sequence number, replaying missed events from the event bus ring buffer.
- **Back-pressure modes**: `at_most_once` (default, drop on transport failure), `coalesce` (planned), `resume_required` (planned).
- **Lag tolerance**: lagged clients get warnings throttled to one log per 5 seconds with cumulative skip counts.

### 5.3 Projection Streaming

Beyond the raw event stream, roko provides **projection streaming** -- server-computed views of system state that clients subscribe to with SSE. This follows the CQRS (Command Query Responsibility Segregation) pattern [8], where the write side (events) is separated from the read side (projections optimized for specific client views).

```rust
// crates/roko-serve/src/routes/projections.rs (actual source)
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/projections/catalog", get(projections_catalog))
        .route("/projections/{name}", get(get_projection))
        .route("/projections/{name}/stream", get(stream_projection))
}

async fn stream_projection(
    Path(name): Path<String>,
    Query(query): Query<ProjectionQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let projections = RuntimeProjectionSet::load(&state).await?;
    let initial_state = projections.project(&name, &query)?;
    let initial = Event::default()
        .event("state")
        .id(projections.cursor.to_string())
        .data(projections.state_frame(&name, initial_state).to_string());

    let delta_stream = stream::unfold(
        state.state_hub.subscribe_events(),
        move |mut rx| {
            async move {
                loop {
                    match rx.recv().await {
                        Ok(envelope) => {
                            if !projection_accepts_event(&name, &query, &envelope.payload) {
                                continue; // skip irrelevant events
                            }
                            let event = Event::default()
                                .event("delta")
                                .id(envelope.seq.to_string())
                                .data(projection_delta_frame(
                                    &name, envelope.seq, &envelope.payload
                                ).to_string());
                            return Some((Ok(event), rx));
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!(projection = %name, skipped, "projection stream lagged");
                        }
                        Err(broadcast::error::RecvError::Closed) => return None,
                    }
                }
            }
        },
    );

    Ok(Sse::new(stream::once(async { Ok(initial) }).chain(delta_stream))
        .keep_alive(KeepAlive::default()))
}
```

This is conceptually similar to event sourcing with CQRS read models [8][9]: the client gets an initial snapshot (`event: state`), then a stream of deltas (`event: delta`), each with a monotonically increasing sequence number. The client can reconnect at any cursor position. The projection engine filters events server-side, so clients only receive updates relevant to their subscribed view.

### 5.4 Workflow WebSocket

Workflow state (PRD -> plan -> tasks -> execution) gets its own WebSocket endpoint with periodic push:

```rust
// crates/roko-serve/src/routes/workflows.rs (actual source)
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/workflows", get(list_workflows))
        .route("/workflows/latest", get(get_latest_workflow))
        .route("/workflows/latest/stream", get(stream_latest_workflow))
        .route("/workflows/{id}", get(get_workflow))
        .route("/workflows/{id}/tasks", get(get_workflow_tasks))
        .route("/workflows/{id}/stream", get(stream_workflow))
        .route("/workflow/ws", get(workflow_ws_upgrade))
}
```

---

## 6. Fleet Aggregation

### 6.1 The Problem

When multiple agents are running (each with its own sidecar HTTP server on a random port), the dashboard needs a single endpoint to query the combined state. Fetching from each sidecar individually would require the client to know all sidecar addresses and handle partial failures. This is the classic API gateway aggregation problem [4], where a single entry point shields consumers from the topology of the backend fleet.

### 6.2 The Solution: Aggregator Routes

The aggregator module (`crates/roko-serve/src/routes/aggregator.rs`) provides fan-out-fan-in routes that query all discovered agent sidecars and merge their responses. The module includes TTL-based caching to avoid hammering sidecars:

```rust
// crates/roko-serve/src/routes/aggregator.rs (actual source)
const AGENT_LIST_TTL: Duration = Duration::from_secs(30);
const AGENT_STATS_TTL: Duration = Duration::from_secs(5);
const PREDICTIONS_TTL: Duration = Duration::from_secs(10);
const KNOWLEDGE_TTL: Duration = Duration::from_secs(30);
const TASKS_TTL: Duration = Duration::from_secs(30);
const STREAM_DISCOVERY_REFRESH: Duration = Duration::from_secs(10);
const STREAM_RECONNECT_DELAY: Duration = Duration::from_secs(2);

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/agents", get(list_agents))
        .route("/agents/topology", get(agent_topology))
        .route("/agents/{id}/stats", get(agent_stats))
        .route("/agents/{id}/skills", get(agent_skills))
        .route("/agents/{id}/heartbeat", get(agent_heartbeat))
        .route("/agents/{id}/trace", get(agent_trace))
        .route("/predictions/sessions", get(list_prediction_sessions))
        .route("/predictions/sessions/{id}", get(get_prediction_session))
        .route("/predictions/claims", get(list_prediction_claims))
        .route("/predictions/calibration/{agent_id}", get(prediction_calibration))
        .route("/knowledge/entries", get(list_knowledge_entries))
        .route("/knowledge/edges", get(list_knowledge_edges))
        .route("/knowledge/search", get(search_knowledge))
        .route("/knowledge/kinds", get(list_knowledge_kinds))
        .route("/tasks", get(list_tasks))
        .route("/tasks/stats", get(task_stats))
        .route("/tasks/{id}", get(get_task))
        .route("/ws", get(ws_upgrade))
}
```

### 6.3 Multiplexed WebSocket Streaming

The most sophisticated aggregation is the multiplexed WebSocket at `GET /api/ws` (registered via the aggregator). It connects to every discovered agent's WebSocket endpoint and fans all their events into a single client-facing stream. This implements the fan-in pattern [10], where multiple upstream sources converge into a single downstream channel.

The discovery loop periodically checks for new or removed agents (every 10 seconds) and adjusts WebSocket connections accordingly. Each upstream message is tagged with `_source_agent` so the client knows which agent produced it. A new agent that starts up will appear in the multiplexed stream within 12 seconds (10s discovery + 2s reconnect delay).

---

## 7. Per-Agent Sidecar: `roko-agent-server`

### 7.1 Why a Sidecar

Each agent process gets its own HTTP server rather than sharing the control plane's port. This follows the sidecar proxy pattern [5], where auxiliary functionality runs alongside the primary service in a separate process. The benefits:

- **Process isolation.** If an agent crashes, its sidecar dies with it, but other agents and the control plane are unaffected.
- **Capability-scoped routes.** Each sidecar only exposes the routes that the agent actually supports (messaging, predictions, research, tasks).
- **Independent auth.** Sidecars can require their own bearer tokens, separate from the control plane's auth.
- **Direct agent communication.** Other agents or services can call the sidecar directly for low-latency messaging without going through the control plane.

### 7.2 Feature-Gated Routes

The sidecar uses a `FeatureFlags` struct to conditionally register routes:

```rust
// crates/roko-agent-server/src/lib.rs (actual source)
#[derive(Debug, Clone, Copy, Default)]
struct FeatureFlags {
    messaging: bool,
    predictions: bool,
    research: bool,
    tasks: bool,
}

impl AgentServer {
    fn protected_router(&self) -> Router<Arc<AgentState>> {
        let mut router = Router::new()
            .route("/stats", axum::routing::get(features::health::stats))
            .merge(features::logs::router());

        if self.features.messaging {
            router = router.merge(features::messaging::router());
        }
        if self.features.predictions {
            router = router.merge(features::predictions::router());
        }
        if self.features.research {
            router = router.merge(features::research::router());
        }
        if self.features.tasks {
            router = router.merge(features::tasks::router());
        }
        router
    }
}
```

The builder API makes enabling features ergonomic:

```rust
// Usage (from agent bootstrap code, actual API)
let server = AgentServer::builder()
    .agent_id("analyst-1")
    .bind("0.0.0.0:0")                 // Random port
    .messaging()                        // Enable messaging routes
    .predictions()                      // Enable predictions routes
    .auth(BearerAuth::new("token"))     // Require auth on protected routes
    .serve_url("http://localhost:6677") // Control plane for heartbeats
    .build()?;

server.serve().await?;
```

### 7.3 Sidecar Route Inventory

Each sidecar exposes a subset of these routes:

**Always present (public):**
```
GET  /health          # {"status": "ok", "agent_id": "...", "uptime_s": N}
GET  /capabilities    # Feature manifest, routes list, skills map
```

**Always present (protected):**
```
GET  /stats           # Runtime statistics
GET  /logs            # Agent log tail
```

**Feature: messaging**
```
POST /message         # Send a message to the agent
```

**Feature: predictions**
```
GET  /predictions           # List predictions
POST /predictions           # Create prediction
GET  /predictions/{id}      # Get prediction
GET  /predictions/residuals # Prediction residual analysis
```

**Feature: research**
```
POST /research        # Trigger research query
```

**Feature: tasks**
```
GET  /tasks                 # List queued tasks
POST /tasks/{id}/accept     # Accept a task
POST /tasks/{id}/complete   # Complete a task
```

### 7.4 Capabilities Manifest

The `GET /capabilities` endpoint returns a machine-readable manifest that the control plane uses for aggregation:

```json
{
    "features": ["custom-skill"],
    "routes": ["/health", "/capabilities", "/stats", "/logs"],
    "skills": {
        "custom-skill": {
            "enabled": true,
            "config": {}
        }
    }
}
```

The manifest includes:
- **`features`**: List of enabled feature names (only live capabilities -- reserved names like "messaging" are filtered out if the feature flag is off)
- **`routes`**: List of all registered route paths
- **`skills`**: Map of custom skill names to their config

### 7.5 Capability Normalization

The sidecar builder's `normalize_capabilities` function ensures that if a feature capability is listed but the feature flag is off, the capability is removed. This prevents overclaiming:

```rust
// crates/roko-agent-server/src/lib.rs (actual source)
fn capability_is_live(value: &str, features: FeatureFlags) -> bool {
    match value {
        "messaging" => features.messaging,
        "predictions" => features.predictions,
        "research" => features.research,
        "tasks" => features.tasks,
        _ => true, // Custom capabilities always pass
    }
}
```

### 7.6 Agent Registration

When a sidecar starts, it can optionally register itself with the control plane or with the agent-relay. The registration system publishes an ERC-8004 Agent Card:

```rust
// crates/roko-agent-server/src/registration.rs (actual source)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentCard {
    /// Human-readable agent name.
    pub name: String,
    /// Advertised capabilities.
    pub capabilities: Vec<String>,
    /// Endpoint map used for discovery.
    pub endpoints: AgentEndpoints,
    /// Domain tags used for off-chain filtering.
    pub domain_tags: Vec<String>,
    /// Card schema/version.
    pub version: String,
}
```

### 7.7 Sidecar Auth

Each sidecar has independent bearer token auth:

```rust
// crates/roko-agent-server/src/lib.rs (actual source)
pub fn router(&self) -> Router {
    let public = Router::new().merge(features::health::router());
    let protected = self.protected_router();

    let protected = if let Some(auth) = self.auth.clone() {
        protected.layer(middleware::from_fn_with_state(
            auth,
            auth::bearer::require_bearer_auth,
        ))
    } else {
        protected
    };

    public
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::clone(&self.state))
}
```

Health and capabilities are always public. Stats, logs, messaging, etc. require auth when configured.

---

## 8. Heartbeat / Health Monitoring

### 8.1 Sidecar -> Control Plane Heartbeats

Each agent sidecar periodically POSTs a heartbeat payload to the control plane. This follows the heartbeat pattern used by distributed systems like Kubernetes, ZooKeeper, and Cassandra [11] for failure detection and liveness monitoring.

```rust
// crates/roko-agent-server/src/lib.rs (actual source)
async fn heartbeat_loop(
    state: Arc<AgentState>,
    url: String,           // e.g., "http://localhost:6677/api/heartbeats"
    interval_secs: u64,
) {
    let client = reqwest::Client::new();
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let payload = HeartbeatPayload {
            sender_id: state.agent_id().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            active_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            active_agents: 1,
            frequency: 1.0 / interval_secs as f64,
            metrics: HashMap::new(),
        };

        match client.post(&url).json(&payload).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::trace!(agent_id = state.agent_id(), "heartbeat sent");
            }
            Ok(resp) => {
                tracing::debug!(status = %resp.status(), "heartbeat rejected");
            }
            Err(err) => {
                tracing::debug!(error = %err, "heartbeat failed (control plane unreachable?)");
            }
        }
    }
}
```

The default interval is 30 seconds (from `roko_core::DEFAULT_HEARTBEAT_INTERVAL_SECS`). The heartbeat URL is constructed by appending `/api/heartbeats` to the configured `serve_url`. `MissedTickBehavior::Skip` ensures that if the system is under load, heartbeat ticks are skipped rather than queued up.

### 8.2 Control Plane Heartbeat Ingestion

The control plane receives heartbeats and stores them in a bounded ring buffer:

```rust
// crates/roko-serve/src/routes/heartbeats.rs (actual source)
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/heartbeats", post(receive_heartbeat).get(list_heartbeats))
        .route("/network/stats", get(network_stats))
}

async fn receive_heartbeat(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<HeartbeatPayload>,
) -> axum::http::StatusCode {
    state.event_bus.publish(ServerEvent::HeartbeatReceived {
        sender_id: payload.sender_id.clone(),
        active_tasks: payload.active_tasks,
        active_agents: payload.active_agents,
    });

    let mut ring = state.heartbeats.write().await;
    if ring.len() >= HEARTBEAT_RING_CAPACITY {
        ring.pop_front();
    }
    ring.push_back(payload);

    axum::http::StatusCode::ACCEPTED  // 202, not 200
}
```

Key details:
- Returns **202 ACCEPTED** (not 200 OK), signaling that the heartbeat was received and will be processed.
- Uses a **`VecDeque` ring buffer** with `HEARTBEAT_RING_CAPACITY` eviction, not a HashMap keyed by agent ID. This preserves heartbeat history for time-series analysis via `GET /api/heartbeats` and `GET /api/network/stats`.
- The `network_stats` endpoint aggregates heartbeat data by sender to compute per-agent statistics (heartbeat count, last seen, average active tasks).

---

## 9. Relay Bridge

### 9.1 What the Relay Is

The agent-relay is a separate service (in `crates/agent-relay/`) that provides:
- **Presence**: agents announce themselves and their capabilities via a `Hello` frame
- **Card hosting**: each agent's metadata (AgentCard) is stored at a REST endpoint with a stable URI
- **Pub/sub messaging**: agents can subscribe to topics and publish typed messages
- **Feed distribution**: real-time data feeds from specialized agents

### 9.2 Relay Client Connection

Each agent sidecar connects to the relay over a persistent WebSocket using the `tokio-tungstenite` library:

```rust
// crates/roko-agent-server/src/features/relay_client.rs (verified against source)
pub struct RelayHandle {
    outbound_tx: mpsc::UnboundedSender<AgentInboundFrame>,
}

impl RelayHandle {
    pub fn subscribe(&self, topic: impl Into<String>) -> Result<()> {
        self.outbound_tx
            .send(AgentInboundFrame::Subscribe { topic: topic.into() })
            .map_err(|_| anyhow!("relay connection closed"))
    }

    pub fn unsubscribe(&self, topic: impl Into<String>) -> Result<()> {
        self.outbound_tx
            .send(AgentInboundFrame::Unsubscribe { topic: topic.into() })
            .map_err(|_| anyhow!("relay connection closed"))
    }

    pub fn register_feed(
        &self,
        feed_id: impl Into<String>,
        topic: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        kind: impl Into<String>,
        // ... additional parameters
    ) -> Result<()> { /* ... */ }
}
```

The relay protocol uses typed frames (`AgentInboundFrame` for client-to-relay, `RelayOutboundFrame` for relay-to-client) serialized as JSON over WebSocket text frames. The protocol includes `Hello`/`HelloAck` handshake, `Subscribe`/`Unsubscribe`, `Publish`, `Card`, and `RegisterFeed` frames.

### 9.3 Relay Proxy Through the Control Plane

The control plane proxies relay endpoints so external consumers do not need to know the relay's address:

```rust
// crates/roko-serve/src/routes/relay_proxy.rs (actual source)
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/relay/agents/ws", get(relay_agents_ws))
        .route("/relay/events/ws", get(relay_events_ws))
        .route("/relay/{*path}", any(relay_proxy))
        .route("/relay", any(relay_root_proxy))
}
```

WebSocket proxying uses a bidirectional bridge in `proxy_ws.rs`:

```rust
// crates/roko-serve/src/routes/proxy_ws.rs (actual source)
pub(crate) async fn bridge_ws(server_socket: WebSocket, upstream_url: String) {
    let Ok((upstream, _)) = connect_async(&upstream_url).await else {
        return;
    };

    let (mut server_tx, mut server_rx) = server_socket.split();
    let (mut upstream_tx, mut upstream_rx) = upstream.split();

    loop {
        tokio::select! {
            msg = server_rx.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if upstream_tx.send(WsMessage::Text(text.to_string().into()))
                            .await.is_err() { break; }
                    }
                    Some(Ok(Message::Binary(data))) => {
                        if upstream_tx.send(WsMessage::Binary(data.to_vec().into()))
                            .await.is_err() { break; }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if upstream_tx.send(WsMessage::Ping(data.to_vec().into()))
                            .await.is_err() { break; }
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    _ => {}
                }
            }
            msg = upstream_rx.next() => {
                match msg {
                    Some(Ok(WsMessage::Text(text))) => {
                        if server_tx.send(Message::Text(text.to_string().into()))
                            .await.is_err() { break; }
                    }
                    Some(Ok(WsMessage::Binary(data))) => {
                        if server_tx.send(Message::Binary(data.to_vec().into()))
                            .await.is_err() { break; }
                    }
                    Some(Ok(WsMessage::Close(_))) | None | Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }

    let _ = server_tx.close().await;
    let _ = upstream_tx.close().await;
}
```

---

## 10. Authentication & Security

### 10.1 Control Plane Auth

The control plane uses a layered auth middleware defined in `crates/roko-serve/src/routes/middleware.rs`:

- **API Key** via `X-Api-Key` header (matched by SHA-256 hash)
- **Bearer token** via `Authorization: Bearer <token>` (falls back to legacy API key)
- **JWT** via `Authorization: Bearer <jwt>` (Privy-issued, 3-segment base64url structure, JWKS-validated)
- **Agent tokens** issued via `POST /api/agents/{id}/token` (scoped to `agent:write`)

Scope hierarchy: `admin` > `agent:write` > `plan:write` > `read`. All GET requests require only `read` scope. Mutating requests are checked against route-specific scope requirements.

### 10.2 Secret Scrubbing

All `/api/*` responses pass through a **secret-scrubbing middleware** using `roko_core::obs::LogScrubber` that automatically redacts API key patterns (Anthropic keys, GitHub PATs, etc.) from JSON and text response bodies. This is crucial because agent output might accidentally include credentials.

### 10.3 Rate Limiting

The control plane applies a global governor-based rate limiter:

```rust
// crates/roko-serve/src/routes/mod.rs (actual source)
pub(crate) const DEFAULT_GLOBAL_RATE_PER_SEC: u32 = 100;
pub(crate) const DEFAULT_REQUEST_BODY_LIMIT_BYTES: usize = 4 * 1024 * 1024;
```

Requests exceeding 100/second receive a 429 response with a stable `code = "rate_limited"` body. The 4 MiB body cap protects against memory exhaustion from oversized payloads.

---

## 11. Webhook Dispatch Loop

The control plane includes a sophisticated webhook dispatch system defined in `crates/roko-serve/src/dispatch.rs`. When a webhook arrives (GitHub push, Slack event, etc.):

1. The webhook handler in `routes/webhooks.rs` verifies the signature, converts the payload to an `Engram`, persists it, and publishes it on the event bus.
2. The dispatch loop (running in the background) receives the `WebhookReceived` event.
3. It resolves matching subscriptions based on trigger type and filters (repo, branch, path, label, author).
4. For each matching subscription, it spawns an agent dispatch with the appropriate template.
5. Dispatch respects per-subscription concurrency limits, cooldown periods, and deduplication.

```rust
// crates/roko-serve/src/dispatch.rs (simplified from source)
pub struct SubscriptionRegistry {
    subscriptions: RwLock<Vec<Subscription>>,
}

pub struct Subscription {
    pub id: String,
    pub template: String,
    pub trigger: String,          // "webhook", "plan_completed", etc.
    pub filter: SubscriptionFilter,
    pub concurrency_limit: usize,
    pub cooldown_secs: u64,
    pub enabled: bool,
}
```

---

## 12. Comparison with IronClaw

### 12.1 What IronClaw Has Today

IronClaw already has a web gateway (`src/channels/web/`) that provides a substantial HTTP API surface with ~198 route registrations:

- **Chat API**: `/api/chat/send`, `/api/chat/events` (SSE), `/api/chat/ws` (WebSocket), `/api/chat/history`, `/api/chat/threads`, `/api/chat/gate/resolve`
- **Memory API**: `/api/memory/tree`, `/api/memory/search`, `/api/memory/read`, `/api/memory/write`
- **Jobs API**: 9 sandbox job routes (`/api/jobs/...`)
- **Skills API**: 6 skill management routes (`/api/skills/...`)
- **Extensions API**: 7 extension lifecycle routes (`/api/extensions/...`)
- **Routines API**: 7 routine management routes (`/api/routines/...`)
- **Settings API**: 6 settings routes (`/api/settings/...`)
- **User management**: 12 admin user routes
- **Traces API**: 10+ trace contribution routes
- **Auth**: bearer token, DB-token, OIDC, query-string token for SSE/WS
- **SSE streaming**: 22+ event types (`SseEvent` enum in `types.rs`)
- **WebSocket**: bidirectional with ping/pong, client messages for chat and approval
- **OpenAI compatibility**: `/v1/chat/completions`, `/v1/models`, `/v1/responses`

IronClaw's gateway architecture is already well-organized with a `platform/` layer (router, state, auth, SSE, WS) separated from `features/` (chat, extensions, jobs, routines, etc.) following an explicit "no back-edges" rule enforced by `scripts/check_gateway_boundaries.py`.

However, IronClaw's web gateway is **session-oriented** (multi-user with per-user rate limiting, but single-agent), not **fleet-oriented**. It does not aggregate data from multiple agents or provide a control plane for managing agent processes.

### 12.2 What Roko Has That IronClaw Could Adopt

| Roko Pattern | IronClaw Equivalent | Gap |
|---|---|---|
| Central EventBus (broadcast + replay) | SseManager (broadcast) | IronClaw's `SseManager` uses `tokio::sync::broadcast` with configurable buffer (`SSE_BROADCAST_BUFFER`, default 1024). Roko adds a replay ring buffer with sequence numbers, enabling cursor-based reconnection without re-fetching history. IronClaw works around this via `Last-Event-ID` with process-scoped boot UUIDs. |
| StateHub (projection engine) | No equivalent | IronClaw has no server-side projection system. Adding one would enable "subscribe to tool execution events" or "subscribe to cost metrics" without client polling. |
| Per-agent sidecar | Orchestrator (`src/orchestrator/`) | IronClaw has an orchestrator for sandbox containers with an internal API, bearer token auth, and job lifecycle management -- structurally similar to a sidecar. The gap is that the orchestrator manages containers, not agent processes with independent HTTP servers. |
| Fleet aggregation | No equivalent | With background jobs and routines, IronClaw could aggregate task status, cost, and output across concurrent execution contexts via a fan-out-fan-in aggregation layer. |
| Heartbeat system (infrastructure) | Heartbeat system (user-facing) | IronClaw's heartbeat runs proactive periodic execution and notifies the user via `HEARTBEAT.md`. Roko's heartbeat is infrastructure: "I am alive, here is my status." Both patterns are useful; they serve different purposes. |
| Inference gateway | LLM provider abstraction (`ironclaw_llm`) | IronClaw already has multi-provider LLM integration with cost tracking. The gateway pattern (centralized key isolation, model routing, per-request cost events) could be layered on top. |
| Webhook dispatch + subscriptions | Webhook server + WASM channels | IronClaw has `webhook_server.rs` for inbound HTTP events. The subscription/filter/dispatch pattern would add structured routing of webhook events to specific tool pipelines with concurrency limits and cooldown. |
| Feature-gated routes | Extension-based capabilities | IronClaw's extension system already gates features. The sidecar pattern of conditionally registering routes is a cleaner implementation than checking capabilities at handler level. |
| Secret scrubbing middleware | Safety pipeline (`ironclaw_safety`) | IronClaw has input-side safety (prompt injection detection, validation). Roko adds output-side scrubbing (redacting API keys from responses). IronClaw could adopt this for its API responses. |
| Global rate limiter (governor) | Per-user rate limiter | IronClaw has a 30 req/60s per-user chat rate limiter. Roko has a global 100 req/s governor limiter. Both are useful; they protect against different failure modes. |
| Graceful shutdown (`CancelToken` + `/ready`) | No equivalent | IronClaw could adopt the readiness probe pattern for deployment environments that support drain-before-stop. |

### 12.3 Integration Sketch: EventBus with Replay for IronClaw

IronClaw could enhance its `SseManager` with roko's replay ring buffer pattern:

```rust
// Enhancement to: src/channels/web/platform/sse.rs

use tokio::sync::broadcast;
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::RwLock;
use std::collections::VecDeque;

/// Event envelope with monotonic sequence number and timestamp.
pub struct SseEnvelope {
    pub seq: u64,
    pub ts_millis: i64,
    pub event: SseEvent,
}

pub struct SseManager {
    sender: broadcast::Sender<SseEnvelope>,
    ring: RwLock<VecDeque<SseEnvelope>>,
    ring_capacity: usize,
    next_seq: AtomicU64,
}

impl SseManager {
    pub fn broadcast(&self, event: SseEvent) -> u64 {
        let seq = self.next_seq.fetch_add(1, Ordering::Relaxed);
        let envelope = SseEnvelope {
            seq,
            ts_millis: chrono::Utc::now().timestamp_millis(),
            event,
        };
        // Record in ring buffer for replay
        {
            let mut ring = self.ring.write();
            if ring.len() >= self.ring_capacity {
                ring.pop_front();
            }
            ring.push_back(envelope.clone());
        }
        let _ = self.sender.send(envelope);
        seq
    }

    /// Replay events published after `after_seq`.
    pub fn replay_from(&self, after_seq: u64) -> Vec<SseEnvelope> {
        let ring = self.ring.read();
        ring.iter()
            .filter(|e| e.seq > after_seq)
            .cloned()
            .collect()
    }
}
```

This would then enable the SSE endpoint to support `Last-Event-ID` reconnection with replay, matching roko's behavior while maintaining backward compatibility with IronClaw's existing `SseEvent` contract.

### 12.4 Integration Sketch: Infrastructure Heartbeats

IronClaw could add infrastructure-level heartbeats alongside the existing user-facing heartbeat system. The user-facing heartbeat (`HEARTBEAT.md` + proactive execution every 30 minutes) serves a different purpose from infrastructure heartbeats (binary liveness + metric reporting every 30 seconds). Both can coexist:

```rust
// Enhancement to: src/channels/web/features/status/

/// GET /api/gateway/heartbeat
/// Infrastructure heartbeat for external monitoring systems.
async fn heartbeat_endpoint(
    State(state): State<Arc<GatewayState>>,
) -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "uptime_secs": state.startup_time.elapsed().as_secs(),
        "active_sse_clients": state.sse.subscriber_count(),
        "active_ws_clients": state.ws_tracker.as_ref()
            .map_or(0, |t| t.count()),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
```

### 12.5 Integration Sketch: Projection Streaming

The projection pattern could enable efficient dashboard updates for IronClaw:

```rust
// Enhancement to: src/channels/web/platform/

/// Named projections the dashboard can subscribe to.
enum Projection {
    /// Aggregated session list with status.
    Sessions,
    /// Cost breakdown by provider and session.
    Costs,
    /// Tool execution log with timing.
    ToolExecutions,
    /// Extension/channel health status.
    ExtensionHealth,
}

/// GET /api/projections/{name}/stream
///
/// Returns:
/// - event: state  (initial full snapshot)
/// - event: delta  (incremental updates, filtered by projection)
async fn stream_projection(
    Path(name): Path<String>,
    State(state): State<Arc<GatewayState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let snapshot = compute_projection(&name, &state).await;
    let initial = Event::default()
        .event("state")
        .data(serde_json::to_string(&snapshot).unwrap_or_default());

    let mut rx = state.sse.subscribe();
    let delta_stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(envelope) if projection_accepts(&name, &envelope.event) => {
                    let delta = compute_delta(&name, &envelope.event);
                    yield Ok(Event::default()
                        .event("delta")
                        .id(envelope.seq.to_string())
                        .data(serde_json::to_string(&delta).unwrap_or_default()));
                }
                Ok(_) => continue,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Sse::new(
        futures::stream::once(async { Ok(initial) }).chain(delta_stream)
    ).keep_alive(KeepAlive::default())
}
```

### 12.6 Enhancement Opportunities

Based on the comparison, these enhancements would provide the highest value for IronClaw with the lowest implementation effort:

1. **Replay ring buffer for SSE** (low effort, high value): add sequence numbers and a bounded ring buffer to `SseManager`. This eliminates the browser's need to re-fetch `/api/chat/history` on reconnect for events that occurred in the last few seconds.

2. **Output-side secret scrubbing** (low effort, high value): add a response middleware that redacts API key patterns from JSON responses. IronClaw already has `ironclaw_safety` for input-side protection; this closes the output side.

3. **Readiness probe** (low effort, medium value): add `GET /ready` that returns 503 during shutdown. IronClaw already has `GET /api/health`; the readiness probe supports zero-downtime deployments.

4. **Projection streaming** (medium effort, high value): server-side filtered event streams. Eliminates polling for dashboard widgets that show tool execution history, cost breakdowns, or extension health.

5. **Global rate limiter** (low effort, medium value): complement the existing per-user chat rate limiter with a global governor-based limiter that protects the entire API surface.

6. **Infrastructure heartbeats** (medium effort, medium value): supplement user-facing heartbeats with machine-readable liveness reporting for external monitoring systems.

7. **Per-agent sidecar** (high effort, high value): relevant when IronClaw moves to multi-agent orchestration. The existing `src/orchestrator/` provides a foundation; the sidecar pattern would give each agent process its own HTTP API.

---

## 13. Key Architectural Insights

### 13.1 Single-Writer, Multiple-Reader

Roko follows a strict pattern: events are written once (by the orchestrator, agent, or webhook handler) to the broadcast bus, then consumed by many readers (SSE clients, WebSocket clients, projection engine, anomaly detector, efficiency tracker). This is an actor-model-adjacent design without the complexity of full actor frameworks. The pattern is sometimes called the "single-writer principle" [12] and eliminates contention on the write path.

### 13.2 TTL-Based Caching in Aggregation

The aggregator uses per-route TTL constants to avoid hammering sidecars:

```rust
const AGENT_LIST_TTL: Duration = Duration::from_secs(30);
const AGENT_STATS_TTL: Duration = Duration::from_secs(5);
const PREDICTIONS_TTL: Duration = Duration::from_secs(10);
const KNOWLEDGE_TTL: Duration = Duration::from_secs(30);
const TASKS_TTL: Duration = Duration::from_secs(30);
```

Stats are refreshed every 5 seconds (latency-sensitive), while knowledge and agent lists are cached for 30 seconds (rarely changing). This is a classic tiered caching strategy where cache TTLs are proportional to the data's rate of change.

### 13.3 Discovery-Based Reconnection

The multiplexed WebSocket stream refreshes its agent discovery every 10 seconds and reconnects to new agents within 2 seconds:

```rust
const STREAM_DISCOVERY_REFRESH: Duration = Duration::from_secs(10);
const STREAM_RECONNECT_DELAY: Duration = Duration::from_secs(2);
```

This means a new agent that starts up will appear in the multiplexed stream within 12 seconds, without any client-side action.

### 13.4 Zero-Key Agents via Gateway

Agents never hold API keys. The inference gateway (`routes/gateway.rs`) centralizes all LLM access:

```rust
// routes/gateway.rs (actual source)
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/inference/complete", post(inference_complete))
        .route("/gateway/stats", get(gateway_stats))
        .route("/gateway/models", get(gateway_models))
        .route("/inference/batch/submit", post(batch_submit))
        .route("/inference/batch/{id}", get(batch_status))
}
```

This pattern means:
- A compromised agent cannot exfiltrate API keys
- The gateway applies cascade routing (model selection) transparently
- Cost tracking is centralized
- Provider health monitoring works across all agents

### 13.5 Lock Acquisition Ordering

The `AppState` documents a strict lock acquisition order to prevent deadlocks:

```
1. active_runs          7. discovered_agents     13. cascade_router
2. active_plans         8. aggregator_cache      14. gateway_model_counters
3. operations           9. heartbeats            15. batch_progress
4. templates           10. connectors            16. active_bench_runs
5. deployments         11. feeds                 17. active_matrix_runs
6. template_runs       12. ephemeral_workspaces
```

Handlers that need multiple locks must acquire them in this order. This is a well-known technique from database systems [13] applied to in-memory concurrency.

---

## 14. Summary

Roko's control plane architecture demonstrates a mature pattern for AI agent systems:

1. **Central control plane (`roko-serve`)** with 100+ HTTP routes organized into 40+ functional modules, covering every aspect of system operation -- from plan execution to cost tracking to benchmark runs to DeFi rate monitoring.

2. **Per-agent sidecars (`roko-agent-server`)** with feature-gated routes, independent bearer auth, capability normalization, and optional relay connectivity.

3. **Fleet aggregation** through fan-out-fan-in HTTP proxying with TTL-based caching and multiplexed WebSocket streams that merge events from all agents into a single client-facing stream.

4. **Event-driven architecture** with a typed `ServerEvent` enum (60+ variants), a broadcast event bus with replay ring buffer and monotonic sequence numbers, SSE/WebSocket streaming with cursor-based reconnection, and projection-based delta delivery following the CQRS pattern.

5. **Relay bridge** connecting agents via persistent WebSocket with pub/sub messaging, ERC-8004 card hosting, and feed distribution.

6. **Infrastructure heartbeats** for health monitoring and agent discovery (30-second intervals, ring buffer storage with network stats aggregation), separate from user-facing heartbeat-driven execution.

7. **Security layers**: centralized inference gateway (zero-key agents), response-side secret scrubbing, layered auth (API key + bearer + JWT + agent tokens), global rate limiting (governor, 100 req/s), and 4 MiB request body caps.

IronClaw can adopt these patterns incrementally: start with a replay ring buffer for SSE and output-side secret scrubbing (low effort, high value), then add projection-based delta streaming and infrastructure heartbeats. The per-agent sidecar pattern becomes relevant when IronClaw moves to multi-agent orchestration with background execution contexts. IronClaw's existing gateway architecture -- with its platform/features separation, 198+ route registrations, and 22+ SSE event types -- already provides a strong foundation for adopting these patterns without a full rewrite.

---

## References

[1] Burns, B., Beda, J., Hightower, K., & Grant, B. (2022). *Kubernetes: Up & Running*, 3rd ed. O'Reilly Media. The canonical reference on Kubernetes architecture, control plane separation, and the API server's role in desired-state reconciliation.

[2] Karia, D. (2025). "Control Planes: The Missing Infrastructure for Scalable Agentic AI Systems." *Medium*. https://deepkaria.medium.com/control-planes-the-missing-infrastructure-for-scalable-agentic-ai-systems-124e05c94d35 -- Analyzes why agentic AI systems stall in production without control plane infrastructure and draws parallels to network and container control planes.

[3] Gamma, E., Helm, R., Johnson, R., & Vlissides, J. (1994). *Design Patterns: Elements of Reusable Object-Oriented Software*. Addison-Wesley. The observer pattern (chapter 5) describes the publish-subscribe relationship used by the EventBus.

[4] Richardson, C. (2018). *Microservices Patterns*. Manning Publications. Chapter 8 covers the API gateway pattern, including request routing, composition, and protocol translation -- the same responsibilities roko-serve's aggregator fulfills.

[5] Calcote, L. & Butcher, Z. (2020). *Istio: Up and Running*. O'Reilly Media. Describes the sidecar proxy pattern in service mesh architectures, where auxiliary functionality runs alongside the primary service. Also see: Indrasiri, K. & Siriwardena, P. (2021). "Service Mesh." Chapter 7 in *Design Patterns for Cloud Native Applications*. O'Reilly. For a technical benchmark of service mesh sidecar implementations, see: Backes, A. & Posegga, J. (2024). "Performance Comparison of Service Mesh Frameworks: the MTLS Test Case." arXiv:2411.02267. https://arxiv.org/pdf/2411.02267

[6] Voroshilov, S. (2019). "arc-swap: Atomically swappable Arc." https://docs.rs/arc-swap/ -- The ArcSwap crate documentation explaining lock-free atomic pointer swaps for concurrent readers.

[7] WHATWG. "Server-Sent Events." HTML Living Standard, section 9.2. https://html.spec.whatwg.org/multipage/server-sent-events.html -- The normative specification for SSE, including EventSource reconnection semantics and Last-Event-ID. Also see: PortalZINE (2025). "SSE's Glorious Comeback: Why 2025 is the Year of Server-Sent Events." https://portalzine.de/sses-glorious-comeback-why-2025-is-the-year-of-server-sent-events/ -- Analysis of SSE's growing adoption in cloud-native architectures due to compatibility with serverless, Kubernetes ingress, and API gateways.

[8] Young, G. (2010). "CQRS Documents." https://cqrs.files.wordpress.com/2010/11/cqrs_documents.pdf -- The foundational document on Command Query Responsibility Segregation, which separates write models (commands) from read models (queries/projections). Roko's projection streaming is a direct implementation of this pattern.

[9] Microsoft Azure Architecture Center. "Event Sourcing Pattern." https://learn.microsoft.com/en-us/azure/architecture/patterns/event-sourcing -- Reference architecture for event sourcing with append-only logs and derived projections. Also see: Kim, S. et al. (2025). "ESAA: Event Sourcing for Autonomous Agents in LLM-Based Software Engineering." arXiv:2602.23193. https://arxiv.org/pdf/2602.23193 -- Applies event sourcing specifically to LLM-based agent systems, directly relevant to roko's architecture.

[10] Pike, R. (2012). "Go Concurrency Patterns." Google I/O 2012. The fan-in pattern (merging multiple channels into one) demonstrated in Go is structurally identical to roko's multiplexed WebSocket aggregation using `mpsc::channel`.

[11] Bhayani, A. (2023). "Heartbeats in Distributed Systems." https://arpitbhayani.me/blogs/heartbeats-in-distributed-systems/ -- Comprehensive overview of heartbeat protocols including push heartbeats (used by roko), pull heartbeats, and gossip-based failure detection. Also see: Chandra, T. D. & Toueg, S. (1996). "Unreliable failure detectors for reliable distributed systems." *Journal of the ACM*, 43(2), 225-267 -- The foundational theoretical work on failure detection, which proves that heartbeat-based failure detectors (the phi-accrual detector class) are sufficient for consensus.

[12] Thompson, M. (2011). "Single Writer Principle." *Mechanical Sympathy* blog. https://mechanical-sympathy.blogspot.com/2011/09/single-writer-principle.html -- Explains why restricting writes to a single thread/actor eliminates contention and enables lock-free data structures, which is the design principle behind roko's event bus.

[13] Silberschatz, A., Korth, H. F., & Sudarshan, S. (2019). *Database System Concepts*, 7th ed. McGraw-Hill. Section 18.1.4 covers two-phase locking and lock ordering as deadlock prevention strategies, the same technique used by AppState's documented lock acquisition order.

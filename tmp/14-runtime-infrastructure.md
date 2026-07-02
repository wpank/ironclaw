# Runtime Infrastructure

**Source crate**: `roko-runtime` (`crates/roko-runtime/src/`)
**Priority**: MEDIUM -- system resilience, process management, observability
**Roko docs**: `docs/v1/00-architecture/07b-bus-transport-fabric.md`, `docs/v2-depth/07-agent-runtime/26-agent-lifecycle-type-state.md`, `docs/v1/07-conductor/13-process-supervision-wiring.md`, `docs/v1/12-interfaces/22-statehub-projection-layer.md`

---

## Why Runtime Infrastructure Matters for AI Agent Systems

Traditional software has a well-understood lifecycle: start, run, stop. AI agent systems break this model fundamentally. An agent loop makes LLM calls that may hang for 30+ seconds, spawns tool-execution subprocesses that may orphan grandchild processes, runs verification gates that may infinite-loop, and operates on multi-minute workflows where a crash mid-pipeline means restarting expensive computation from scratch. Without structured runtime infrastructure, you get:

1. **Silent resource leaks.** An LLM call times out, but the HTTP connection stays alive. A `cargo check` subprocess is killed, but its `rustc` grandchild keeps running. Over hours of operation, these orphans consume CPU and memory until the system starves.

2. **Unrecoverable crashes.** A workflow completes 80% of its pipeline (strategy, implementation, gate verification) and then the process crashes. Without checkpointing and event replay, the entire pipeline restarts from zero -- burning tokens, time, and money.

3. **Opaque failures.** An agent is "running" but producing no output. Is it waiting on an LLM? Did a tool call hang? Is it stuck in a retry loop? Without structured events, health probes, and lifecycle state machines, the operator has no way to distinguish a healthy slow agent from a stuck broken one.

4. **Cascading cancellation failures.** A user cancels a session, but only the top-level task stops. Background heartbeat tasks, in-flight tool calls, and spawned subprocesses continue running, consuming resources and potentially producing stale side effects.

The `roko-runtime` crate addresses all four concerns with a cohesive set of primitives: an EventBus for structured event flow, hierarchical CancellationTokens for cascading shutdown, a FIPA-informed lifecycle state machine for agent provisioning and health, a pure state machine + effect driver separation for testable workflow logic, process supervision for OS-level process management, and a StateHub for unified dashboard projections.

### Academic Foundations

These patterns draw from well-established computer science research:

- **Event sourcing and append-only logs.** Fowler's 2005 Event Sourcing pattern [1] and Kleppmann's formalization in *Designing Data-Intensive Applications* (2017) [2] established that storing state as an ordered sequence of immutable events enables replay, audit, and crash recovery. Helland's "Immutability Changes Everything" (CIDR, 2015) [3] extended this principle to distributed systems.

- **Structured concurrency and hierarchical cancellation.** The Go `context` package (2014) [4] introduced tree-structured cancellation where cancelling a parent context cancels all children. Elizarov's structured concurrency in Kotlin coroutines (2018) [5] and Smith's nursery pattern in Python's Trio (2017) [6] independently formalized the same parent-child cancellation invariant: no child task outlives its parent scope.

- **FIPA agent lifecycle.** The Foundation for Intelligent Physical Agents (FIPA) Agent Management Specification (FIPA00023, 2002) [7] defined the normative framework for agent creation, registration, operation, migration, and retirement. The IEEE Computer Society absorbed FIPA's standards work in 2005. Roko's lifecycle state machine adapts the FIPA model with cloud-native extensions (health probes, degradation stages, GitOps).

- **Process supervision.** Armstrong's 2003 PhD thesis "Making reliable distributed systems in the presence of software errors" [8] formalized the Erlang/OTP supervision tree: a hierarchical arrangement of processes where supervisors monitor children and restart them on failure. The "let it crash" philosophy -- distinguished from "let it fail silently" -- ensures crashes are always observed and handled by a supervisor.

- **Type-state provisioning.** The type-state pattern encodes runtime state in compile-time types, making invalid state transitions impossible. Roko uses Rust's `PhantomData` to enforce a provisioning pipeline order at compile time, following the approach described by Cliffle (2019) [9] and rooted in session type theory (Honda, 1993; Kiselyov & Imai, 2020) [10].

- **Pure state machines and effect separation.** Separating a pure state machine from its side-effectful driver is a well-known functional programming technique. The Elm Architecture (Czaplicki, 2012) [11] and algebraic effect systems (Bauer & Pretnar, 2013) [12] both formalize the pattern of expressing computation as a pure function from state and input to state and effects, with a separate runtime that interprets the effects.

---

## Crate Overview

The `roko-runtime` crate is the shared async runtime substrate for all Roko applications. Its `lib.rs` module doc states the design intent clearly:

> This crate extracts the foundational runtime concerns that Mori (and other Roko applications) depend on [...] No domain types. This crate knows nothing about agents, plans, gates, or TUI. It provides generic infrastructure that higher layers parameterise. [...] Tokio-native. All primitives are `Send + Sync + 'static` and designed for multi-task Tokio runtimes. [...] Zero unsafe. All concurrency goes through `tokio::sync` or `std::sync::atomic`.

**Source**: `crates/roko-runtime/src/lib.rs`, lines 1-23

The crate exports 23 public modules:

| Module | Purpose |
|--------|---------|
| `event_bus` | Typed broadcast channel with bounded replay ring |
| `pulse_bus` | Topic-filtered Pulse transport (Bus trait implementation) |
| `cancel` | Hierarchical cooperative cancellation tokens |
| `lifecycle` | FIPA-informed agent lifecycle state machine |
| `process` | OS-level process supervision (spawn, track, kill, reap) |
| `pipeline_state` | Pure state machine for config-driven workflows |
| `effect_driver` | Side-effect executor bridging state machine to I/O |
| `workflow_engine` | High-level workflow orchestration (combines state machine + driver) |
| `state_hub` | Unified dashboard projection hub |
| `state_snapshot` | Checksummed atomic state snapshots |
| `projection` | JSONL event log reconstruction into per-run summaries |
| `run_ledger` | Typed workflow run record (replaces event bus replay) |
| `heartbeat` | Cognitive heartbeat policy (gamma/theta/delta ticks) |
| `heartbeat_probes` | Health probe evaluation logic |
| `heartbeat_attention` | Attention auction and tick gating |
| `metrics` | Append-only JSONL metric recording |
| `jsonl_logger` | RuntimeEvent persistence to JSONL files |
| `http_event_sink` | HTTP-based event forwarding |
| `resource` | Token, cost, and time budget accounting |
| `energy` | Cognitive energy model for metabolic cost tracking |
| `task_scheduler` | Schedulable task management |
| `delta_consumer` | Delta tick event consumer |
| `theta_consumer` | Theta tick event consumer |
| `demurrage_consumer` | Demurrage decay consumer |

---

## 1. EventBus with Bounded Replay Ring

### What It Is

The EventBus is a typed, bounded broadcast channel with a monotonically sequenced replay ring. It generalizes the ad-hoc `mpsc::UnboundedSender<AgentEvent>` channels that were scattered through earlier agent code into a single, generic primitive.

**Source**: `crates/roko-runtime/src/event_bus.rs`

### Architecture

```
  Producer --emit()--> EventBus --broadcast--> Subscriber1
                          |                    Subscriber2
                          |                    Subscriber3
                          v
                     ReplayRing (bounded VecDeque)
```

The bus combines three mechanisms:
- A `tokio::sync::broadcast` channel for live fan-out to all subscribers
- A bounded `VecDeque` ring for durable replay (new subscribers can catch up on recent history)
- An `AtomicU64` counter for monotonic sequence numbering, enabling gap detection and ordered replay

### Why This Pattern Matters for AI Agents

In an AI agent system, multiple subsystems need to observe the same events: the TUI needs tool-call progress, the cost guard needs token usage, the heartbeat needs lifecycle transitions, the SSE gateway needs everything. Without a broadcast bus, each consumer requires its own channel, creating an N-to-M wiring problem. The EventBus solves this with fan-out: emit once, every subscriber receives. The replay ring adds temporal decoupling -- a subscriber that starts late (e.g., a web dashboard reconnecting) can catch up on recent history without replaying the full event log from disk.

### Actual Implementation

The core data structure stores shared state behind an `Arc`:

```rust
// crates/roko-runtime/src/event_bus.rs, lines 199-204

struct Shared<E> {
    tx: broadcast::Sender<Envelope<E>>,
    ring: Mutex<VecDeque<Envelope<E>>>,
    seq: AtomicU64,
    capacity: usize,
}
```

Every event is wrapped in a timestamped envelope before emission:

```rust
// crates/roko-runtime/src/event_bus.rs, lines 62-71

/// A sequenced, timestamped envelope wrapping a user event.
#[derive(Debug, Clone)]
pub struct Envelope<E> {
    /// Monotonically increasing sequence number (bus-scoped).
    pub seq: u64,
    /// Unix timestamp in milliseconds when the event was emitted.
    pub ts_millis: u64,
    /// The wrapped event payload.
    pub payload: E,
}
```

The `emit_inner` function is the critical path. It atomically increments the sequence counter, appends to the replay ring (evicting the oldest entry if at capacity), and broadcasts to live subscribers:

```rust
// crates/roko-runtime/src/event_bus.rs, lines 206-229

impl<E: Clone + Send + Sync + 'static> Shared<E> {
    fn emit_inner(&self, event: E) -> u64 {
        let envelope = Envelope {
            seq: self.seq.fetch_add(1, Ordering::Relaxed),
            ts_millis: current_ts_millis(),
            payload: event,
        };

        let seq = envelope.seq;

        // Append to replay ring (short lock).
        {
            let mut ring = self.ring.lock();
            if ring.len() >= self.capacity {
                ring.pop_front();
            }
            ring.push_back(envelope.clone());
        }

        trace!(seq = seq, "event emitted");
        let _ = self.tx.send(envelope);
        seq
    }
}
```

Key design decisions:

- **The bus never blocks producers.** If a subscriber falls behind the broadcast channel capacity, it will miss events on the live channel but can always catch up via `replay_from()`. This is documented explicitly: "if a subscriber falls behind, it will miss events on the live broadcast channel (but can always catch up via `replay_from`)" (line 234).
- **The ring uses `parking_lot::Mutex`, not `tokio::sync::Mutex`.** The lock is held only for a `VecDeque::push_back` and optional `pop_front` -- microsecond-scale operations that do not warrant the overhead of an async mutex.
- **Sequence numbers use `Ordering::Relaxed`.** Monotonicity is guaranteed because `fetch_add` is atomic; strict ordering between different CPUs is not needed because the ring itself is mutex-protected.

### The Public API

```rust
// crates/roko-runtime/src/event_bus.rs, lines 248-314

impl<E: Clone + Send + Sync + 'static> EventBus<E> {
    /// Creates a new event bus with the given capacity.
    pub fn new(capacity: usize) -> Self;

    /// Emits an event to all live subscribers and appends to the replay ring.
    /// Never blocks. Returns the assigned sequence number.
    pub fn emit(&self, event: E) -> u64;

    /// Subscribes to live events. Returns a broadcast receiver.
    /// Events emitted before this call are NOT received -- use replay_from().
    pub fn subscribe(&self) -> broadcast::Receiver<Envelope<E>>;

    /// Returns events in the replay ring with seq >= after_seq.
    pub fn replay_from(&self, after_seq: u64) -> Vec<Envelope<E>>;

    /// Total events ever emitted (including evicted ones).
    pub fn total_emitted(&self) -> u64;

    /// Current replay ring occupancy.
    pub fn ring_len(&self) -> usize;

    /// Returns a send-only handle for subsystems that only produce events.
    pub fn sender(&self) -> BusSender<E>;
}
```

The `BusSender` is a key ergonomic feature: it is a `Clone + Send` handle that can only emit, not subscribe or replay. This lets you pass event production capability to subsystems without giving them the full bus API.

### Ring Buffer Mechanics

The replay ring is a bounded `VecDeque` with FIFO eviction. When the ring reaches capacity, the oldest event is evicted. This means:

- Replay only covers what remains in the ring
- Subscribers that fall behind lose history unless the event is also persisted elsewhere
- The ring size is a tuning parameter: too small and late joiners miss events, too large and memory grows

The default capacities in the codebase are:
- **1024** for the global `RokoEvent` bus (line 340: `ROKO_EVENT_BUS.get_or_init(|| EventBus::new(1024))`)
- **2048** for the per-type `RuntimeEvent` buses (line 368: `EventBus::new(2048)`)

### Global Event Buses

The crate provides two global bus singletons:

```rust
// crates/roko-runtime/src/event_bus.rs, lines 334-381

/// Process-local shared runtime event bus for RokoEvent.
pub fn global_event_bus() -> &'static EventBus<RokoEvent>;

/// Global event bus for workflow runtime events (type-parameterized).
pub fn runtime_event_bus<RuntimeEvent>() -> &'static EventBus<RuntimeEvent>
where
    RuntimeEvent: Clone + Send + Sync + 'static;

/// Convenience: emit a RuntimeEvent to the global bus.
pub fn emit_runtime_event<RuntimeEvent>(event: RuntimeEvent) -> u64;
```

The `runtime_event_bus` function uses a `TypeId`-keyed `HashMap` behind a `OnceLock` to support multiple event types without compile-time coupling. Each type gets its own `Box::leak`-allocated bus. This avoids crate-cycle issues: `roko-runtime` cannot import `roko_core::RuntimeEvent` directly because `roko-core` depends on `roko-runtime`.

### Concrete Event Types

The `RokoEvent` enum carries the runtime's domain events:

```rust
// crates/roko-runtime/src/event_bus.rs, lines 116-196

pub enum RokoEvent {
    /// Plan revision triggered by repeated gate failures.
    PlanRevision { request_id, plan_id, task_id, reason, ... },
    /// PRD promoted to published state.
    PrdPublished { slug, path, published_at, origin },
    /// Heartbeat tick from cognitive loop.
    HeartbeatTick(HeartbeatTick),
    /// Urgent wakeup bypassing normal heartbeat cadence.
    HeartbeatWakeup { condition, issued_at },
    /// Cognitive control signal from heartbeat governance.
    CognitiveSignal { signal, issued_at },
    /// Agent lifecycle state change.
    AgentLifecycleTransition(LifecycleTransition),
    /// Tick broadcast for downstream consumers.
    TickBroadcast { tick_id, agent_id, tier, passed, cost_usd, broadcast_at },
    /// React-step decision from the policy.
    ReactDecision { tick_id, decision, signals, decided_at },
}
```

### How EventBus Prevents Common Runtime Failures

**Failure: Observer misses critical events during reconnection.**
Without replay, a web dashboard that reconnects after a network blip loses all events emitted during disconnection. With the replay ring, the dashboard sends its last-seen sequence number and receives all events from that point forward:

```rust
// Reconnection handler
let recent = event_bus().replay_from(last_seen_seq);
for envelope in recent {
    send_to_client(envelope.payload);
}
// Then switch to live subscription
let mut rx = event_bus().subscribe();
```

**Failure: Event emitter blocks on slow subscriber.**
A slow logging subscriber should never block the agent's LLM call path. The bus uses `tokio::sync::broadcast` which drops events for lagged receivers rather than blocking the sender. The subscriber detects the lag and can catch up from the replay ring.

### PulseBus: Topic-Filtered Transport

The `PulseBus` wraps `EventBus<Pulse>` and implements the `Bus` trait from `roko-core`, adding topic-based filtering on subscription:

```rust
// crates/roko-runtime/src/pulse_bus.rs, lines 35-37, 68-81

pub struct PulseBus {
    inner: Arc<EventBus<Pulse>>,
}

impl Bus for PulseBus {
    type Receiver = PulseBusReceiver;

    fn publish(&self, pulse: Pulse) -> Result<u64>;
    fn subscribe(&self, filter: TopicFilter) -> Result<PulseBusReceiver>;
}
```

Subscribers only see pulses matching their `TopicFilter`. Non-matching pulses are silently skipped. The `PulseBusReceiver` handles lagged receivers by logging a warning and continuing:

```rust
// crates/roko-runtime/src/pulse_bus.rs, lines 97-115

pub async fn recv(&mut self) -> Option<Pulse> {
    loop {
        match self.rx.recv().await {
            Ok(envelope) => {
                if self.filter.matches(&envelope.payload.topic) {
                    return Some(envelope.payload);
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!(skipped = n, "PulseBusReceiver lagged, skipped pulses");
            }
            Err(broadcast::error::RecvError::Closed) => return None,
        }
    }
}
```

The roko docs describe the target architecture for the Bus as a kernel-level transport primitive in `docs/v1/00-architecture/07b-bus-transport-fabric.md`:

> Bus is the kernel's ephemeral transport fabric at L0. It exists for communication, not durable storage. Where Substrate preserves Engrams, Bus delivers Pulses to subscribers that care about a topic family. [...] The design separates transport from persistence so that: high-frequency Pulses can move without forcing storage writes, late subscribers can catch up from the bounded replay ring, cross-layer couplings can be expressed as topics instead of direct crate dependencies.

---

## 2. Hierarchical CancellationTokens

### What It Is

A cooperative cancellation system where tokens form a tree: cancelling a parent automatically cancels all its descendants, but cancelling a child does not affect its parent. This enables scoped resource cleanup -- cancel a session and all its jobs, tools, and background tasks stop; cancel just one tool call and the session keeps running.

**Source**: `crates/roko-runtime/src/cancel.rs`

### Why Not `tokio_util::sync::CancellationToken`?

The crate's doc comment explains: this provides "a lightweight alternative to `tokio_util::sync::CancellationToken` that integrates with the event bus and supports hierarchical cancellation (parent cancels all children)" (lines 3-4). The roko implementation is simpler (~165 lines including tests vs. tokio-util's more complex implementation) and designed specifically for the parent-child hierarchy that agent systems need.

### Relationship to Structured Concurrency

The hierarchical cancellation token implements the core invariant of structured concurrency [5][6]: no child task outlives its parent scope. Go's `context.WithCancel` (2014) [4] established the tree-structured pattern -- a parent context creates child contexts, and cancelling the parent closes the `Done()` channels of all descendants. Roko's `CancelToken` provides the same guarantee in Rust's async ecosystem: `cancel()` on a parent token walks all descendant `Notify` handles and wakes them, ensuring async tasks watching any token in the chain can observe the cancellation.

The critical safety property is **one-way propagation**: cancellation flows strictly from parent to children, never from children to parent. This prevents a failing tool call from bringing down an entire session.

### Actual Implementation

```rust
// crates/roko-runtime/src/cancel.rs, lines 37-47

#[derive(Clone)]
pub struct CancelToken {
    inner: Arc<CancelInner>,
}

struct CancelInner {
    cancelled: AtomicBool,
    notify: Notify,
    /// Parent tokens. Cancellation propagates downward: if any ancestor
    /// is cancelled, this token is considered cancelled.
    parent: Option<CancelToken>,
}
```

Key design: `CancelToken` is `Clone` (via `Arc`), and the parent is stored as an `Option<CancelToken>` -- the parent is itself a cloned token, so the entire ancestry chain is kept alive as long as any descendant exists.

**Creating the hierarchy:**

```rust
// crates/roko-runtime/src/cancel.rs, lines 49-73

impl CancelToken {
    /// Create a new root cancellation token.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CancelInner {
                cancelled: AtomicBool::new(false),
                notify: Notify::new(),
                parent: None,
            }),
        }
    }

    /// Create a child token. The child is cancelled when:
    /// - The parent is cancelled, OR
    /// - The child itself is cancelled directly.
    pub fn child(&self) -> Self {
        Self {
            inner: Arc::new(CancelInner {
                cancelled: AtomicBool::new(false),
                notify: Notify::new(),
                parent: Some(self.clone()),
            }),
        }
    }
}
```

**Cancellation is one-way and permanent:**

```rust
// crates/roko-runtime/src/cancel.rs, lines 76-95

/// Cancel this token. All tasks awaiting cancelled() will be woken.
pub fn cancel(&self) {
    self.inner.cancelled.store(true, Ordering::Release);
    self.inner.notify.notify_waiters();
}

/// Returns true if this token (or any ancestor) has been cancelled.
pub fn is_cancelled(&self) -> bool {
    if self.inner.cancelled.load(Ordering::Acquire) {
        return true;
    }
    // Walk the parent chain (iterative, not recursive).
    let mut current = self.inner.parent.as_ref();
    while let Some(parent) = current {
        if parent.inner.cancelled.load(Ordering::Acquire) {
            return true;
        }
        current = parent.inner.parent.as_ref();
    }
    false
}
```

The `is_cancelled()` method walks the entire parent chain iteratively. This is O(depth) but the hierarchy is typically 3-4 levels deep, so this is trivial.

**Async waiting:**

The `cancelled()` method is an async function that returns when any token in the ancestor chain is cancelled:

```rust
// crates/roko-runtime/src/cancel.rs, lines 106-148

pub async fn cancelled(&self) {
    if self.is_cancelled() {
        return;
    }

    let mut notifies = vec![&self.inner.notify];
    let mut current = self.inner.parent.as_ref();
    while let Some(parent) = current {
        notifies.push(&parent.inner.notify);
        current = parent.inner.parent.as_ref();
    }

    loop {
        if self.is_cancelled() {
            return;
        }
        match notifies.len() {
            1 => { notifies[0].notified().await; }
            2 => {
                tokio::select! {
                    () = notifies[0].notified() => {}
                    () = notifies[1].notified() => {}
                }
            }
            _ => {
                let self_notify = notifies[0].notified();
                let root_notify = notifies.last().expect("non-empty").notified();
                tokio::select! {
                    () = self_notify => {}
                    () = root_notify => {}
                }
            }
        }
    }
}
```

This collects `Notify` handles from every token in the ancestor chain and uses `tokio::select!` to wake on whichever fires first. For deep chains (3+ levels), it optimizes by only watching self and root, since cancellation always propagates through `notify_waiters()`.

### Cancellation Hierarchy for AI Agents

The intended hierarchy maps naturally to agent system scopes:

```
Session Token (root)
 +-- Job Token
 |    +-- Tool Call Token
 |    +-- LLM Call Token
 +-- Background Task Token
      +-- Heartbeat Token
      +-- Consolidation Token
```

Cancelling the session token cascades to every job, tool call, LLM call, heartbeat, and consolidation task. Cancelling just a tool call token leaves the parent job and sibling LLM call unaffected.

### How Cancellation Prevents Common Runtime Failures

**Failure: Orphaned background tasks after session disconnect.**
Without hierarchical cancellation, a user disconnects and the session's LLM call is cancelled, but the heartbeat task, the consolidation task, and an in-flight tool call continue running indefinitely. With a cancellation tree, the session token's `cancel()` wakes all descendant `Notify` handles, and each task's `cancelled().await` resolves, allowing cleanup.

**Failure: Resource leak from partial cancellation.**
An LLM call times out and is cancelled, but the tool call running alongside it continues. The tool call eventually completes and writes results to the database for a session that no longer exists. With cancellation tokens, the tool call's token is a sibling child of the same job token -- if the job is cancelled (e.g., due to the LLM timeout), both the LLM call and tool call observe cancellation.

The tests in `cancel.rs` verify these guarantees explicitly:

- `child_inherits_cancel`: grandchild reports cancelled when root is cancelled
- `child_independent_cancel`: child cancel does NOT propagate upward
- `child_cancelled_when_parent_cancelled`: async `cancelled()` future resolves when parent is cancelled

---

## 3. FIPA Lifecycle State Machine

### What It Is

A structured model for agent process lifecycle based on FIPA (Foundation for Intelligent Physical Agents) standard FIPA00023 [7], extended with cloud-native concepts like health probes, degradation stages, and GitOps configuration management.

**Source**: `crates/roko-runtime/src/lifecycle.rs`
**Docs**: `docs/v2-depth/07-agent-runtime/26-agent-lifecycle-type-state.md`

### FIPA Background

The FIPA Agent Management Specification (FIPA00023, 2002) [7] defines the normative framework within which FIPA agents exist and operate. It establishes a reference model for agent creation, registration, location, communication, migration, and retirement. Two directory services -- the Agent Management System (AMS) for lifecycle tracking and the Directory Facilitator (DF) for capability discovery -- form the backbone of FIPA-compliant agent platforms. The IEEE Computer Society absorbed FIPA's standards work in 2005, and FIPA specifications are now maintained under the IEEE umbrella.

Roko adapts FIPA's lifecycle model for cloud-native AI agents. The original FIPA states (Initiated, Active, Suspended, Waiting, Transit) map to Roko's extended set, with additions for budget-constrained operation (`Degraded`), capability evolution (`Metamorphosing`), cold storage (`Hibernated`), and explicit compute provisioning (`MachineLifecycleState`).

### Agent Lifecycle States

The `AgentLifecycleState` enum defines all possible states an agent can be in:

```rust
// crates/roko-runtime/src/lifecycle.rs, lines 15-37

pub enum AgentLifecycleState {
    /// Manifest accepted, but the process is not yet running.
    Initiated,
    /// Infrastructure and runtime dependencies are being allocated.
    Provisioning,
    /// Agent registered, cognitive loop running, and accepting work.
    Active,
    /// Operator-initiated pause with state retained.
    Suspended,
    /// Agent is self-blocked on an external event.
    Waiting,
    /// Logical state preserved to cold storage while the process is stopped.
    Hibernated,
    /// Role, capability, or tool transition is in progress.
    Metamorphosing,
    /// Budget-constrained operation at reduced capability.
    Degraded { stage: DegradationStage },
    /// Process terminated and runtime resources released.
    Deleted,
}
```

### State Transition Graph

```
Initiated --ManifestValidated--> Provisioning --RuntimeReady--> Active
                                                                  |
                                                   +--------------+--------------+
                                                   |              |              |
                                            OperatorPause   ExternalWait   BudgetConstrained
                                                   |              |              |
                                                   v              v              v
                                              Suspended       Waiting     Degraded(stage)
                                                   |              |              |
                                            OperatorResume  ExternalReady  BudgetRestored
                                                   |              |              |
                                                   +--------------+--------------+
                                                                  |
                                                                  v
                                                               Active
                                                                  |
                                                          MetamorphosisStarted
                                                                  |
                                                                  v
                                                          Metamorphosing
                                                                  |
                                                          MetamorphosisFinished
                                                                  |
                                                                  v
                                                               Active
                                                                  |
                                                           OperatorDelete
                                                                  |
                                                                  v
                                                              Deleted
```

### Transition Records

Every lifecycle transition is captured as a serializable record for event logs and replay:

```rust
// crates/roko-runtime/src/lifecycle.rs, lines 108-120

pub struct LifecycleTransition {
    /// Stable agent identifier.
    pub agent_id: String,
    /// Previous lifecycle state.
    pub from: AgentLifecycleState,
    /// New lifecycle state.
    pub to: AgentLifecycleState,
    /// Why the transition occurred.
    pub reason: LifecycleTransitionReason,
    /// UTC timestamp for the transition.
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}
```

These transitions are emitted on the EventBus as `RokoEvent::AgentLifecycleTransition(LifecycleTransition)` events, making them available to all subscribers for dashboards, audit logs, and reactive policies.

### Transition Reasons

```rust
// crates/roko-runtime/src/lifecycle.rs, lines 74-105

pub enum LifecycleTransitionReason {
    OperatorCreate,
    ManifestValidated,
    RuntimeReady,
    OperatorPause,
    OperatorResume,
    ExternalWait,
    ExternalReady,
    OperatorDelete,
    BudgetConstrained,
    BudgetRestored,
    MetamorphosisStarted,
    MetamorphosisFinished,
    CleanupComplete,
    Custom(String),
}
```

### Budget Degradation Stages

When budget constraints reduce agent capability, the agent enters a progressive degradation path:

```rust
// crates/roko-runtime/src/lifecycle.rs, lines 40-53

pub enum DegradationStage {
    /// Cheaper models are preferred.
    ModelDowngrade,
    /// Zero-LLM probes are emphasized.
    T0Emphasis,
    /// Runtime tick frequency is reduced.
    ReducedFrequency,
    /// Agent observes and reports but avoids taking actions.
    MonitoringOnly,
    /// Cognitive loop is paused until the budget window resets.
    BudgetPaused,
}
```

Each stage represents a progressively more restrictive operating mode. The docs explain the intent in `docs/v2-depth/07-agent-runtime/26-agent-lifecycle-type-state.md`:

> Graceful degradation: Budget pressure dims the Agent progressively through well-defined stages, never kills it. [...] The degradation-stage -> reduced cost -> budget recovery -> degradation lifted loop means budget pressure self-corrects via reduced spending.

### Machine Lifecycle State (Compute Provisioning)

Separate from the agent's logical lifecycle, there is a machine-level lifecycle for compute provisioning:

```rust
// crates/roko-runtime/src/lifecycle.rs, lines 56-71

pub enum MachineLifecycleState {
    /// Manifest validated and resources requested.
    Provisioning,
    /// VM or local process has been spawned and is booting.
    Booting,
    /// Health checks pass and the agent can accept work.
    Ready,
    /// Deletion requested; work is draining before shutdown.
    Draining,
    /// Resources have been released.
    Destroyed,
    /// Supervisor restart budget was exceeded.
    Crashed,
}
```

### Health Probes (Kubernetes-style)

Three probe types, modeled after Kubernetes probe specifications:

```rust
// crates/roko-runtime/src/lifecycle.rs, lines 181-262

pub struct HealthProbeConfig {
    /// Liveness: checks that the process is responsive.
    pub liveness: ProbeSpec,
    /// Readiness: checks whether the agent can accept new work.
    pub readiness: ProbeSpec,
    /// Startup: gates liveness and readiness during initial boot.
    pub startup: ProbeSpec,
}

pub struct ProbeSpec {
    pub handler: ProbeHandler,
    pub initial_delay_secs: u64,
    pub period_secs: u64,
    pub timeout_secs: u64,
    pub success_threshold: u32,
    pub failure_threshold: u32,
}

pub enum ProbeHandler {
    Internal,                    // Runtime health check
    Http { path, port },         // HTTP GET
    Tcp { port },                // TCP connection
    Exec { command: Vec<String> }, // Custom command (exit 0 = healthy)
}
```

Default probe configuration:

| Probe | Initial Delay | Period | Timeout | Success Threshold | Failure Threshold |
|-------|---------------|--------|---------|-------------------|-------------------|
| Liveness | 15s | 20s | 1s | 1 | 3 |
| Readiness | 5s | 10s | 1s | 1 | 3 |
| Startup | 0s | 10s | 1s | 1 | 30 |

The startup probe has a high failure threshold (30) because bootstrapping can take significant time, while liveness and readiness probes have a low threshold (3) because a running agent should respond quickly.

### Type-State Provisioning Pipeline

The lifecycle module enforces correct provisioning order at compile time using Rust's type-state pattern [9]. Each provisioning stage is a distinct type marker:

```rust
// crates/roko-runtime/src/lifecycle.rs, lines 397-592

pub struct Unvalidated;
pub struct Validated;
pub struct ResourcesAllocated;
pub struct NeuroInitialized;
pub struct RoutingConfigured;
pub struct ToolsLoaded;
pub struct MeshRegistered;
pub struct Ready;

pub struct Agent<S> {
    manifest_id: String,
    state: AgentState,
    stage: PhantomData<S>,
}

// Each transition consumes self and produces the next stage.
// Illegal transitions are compile-time errors.

impl Agent<Unvalidated> {
    pub fn new(manifest_id: impl Into<String>) -> Self;
    pub fn validate(self) -> Agent<Validated>;
}

impl Agent<Validated> {
    pub fn allocate_resources(self, resource: impl Into<String>) -> Agent<ResourcesAllocated>;
}
```

The type parameter `S` is a phantom type -- it carries no runtime data (using `PhantomData<S>`) but constrains which methods are callable at compile time. Calling `.ready()` on an `Agent<Validated>` is a type error. The compiler enforces the pipeline order: `Unvalidated -> Validated -> ResourcesAllocated -> NeuroInitialized -> RoutingConfigured -> ToolsLoaded -> MeshRegistered -> Ready`.

Usage is a linear pipeline that accumulates state:

```rust
let agent = Agent::new("manifest-1")
    .validate()
    .allocate_resources("small")
    .init_neuro()
    .configure_routing()
    .load_tools("standard")
    .register_mesh(true)
    .ready();

// agent.state() now contains the full accumulated provisioning state.
// Trying to call .ready() on an Agent<Validated> is a compile error.
```

### How Lifecycle Prevents Common Runtime Failures

**Failure: Agent accepts work before initialization is complete.**
Without the type-state pipeline, an agent could accept messages before its tools are loaded or its routing is configured, producing confusing errors. With the type-state pattern, only `Agent<Ready>` has the method to enter the cognitive loop -- an incompletely initialized agent cannot.

**Failure: Inconsistent state reporting after crash.**
Without lifecycle transitions, a crashed agent shows as "Active" in the dashboard. With lifecycle transitions emitted to the EventBus, every state change is recorded. The dashboard subscribes to `AgentLifecycleTransition` events and always reflects the true state, including `MachineLifecycleState::Crashed`.

### OCI-Inspired Lifecycle Hooks

The module provides lifecycle hooks for operator customization at five points:

```rust
// crates/roko-runtime/src/lifecycle.rs, lines 141-178

pub struct LifecycleHooks {
    pub before_provision: Vec<HookSpec>,
    pub before_start: Vec<HookSpec>,
    pub after_start: Vec<HookSpec>,
    pub before_stop: Vec<HookSpec>,
    pub after_stop: Vec<HookSpec>,
}

pub struct HookSpec {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub timeout_secs: u64,  // default: 30
}
```

### Restart Backoff

Failed agent processes use exponential backoff before restart:

```rust
// crates/roko-runtime/src/lifecycle.rs, lines 265-306

pub struct RestartBackoff {
    pub failure_count: u32,
    pub base_delay_ms: u64,    // default: 100
    pub max_delay_ms: u64,     // default: 300,000 (5 minutes)
    pub reset_after_ms: u64,   // default: 300,000 (5 minutes)
}

impl RestartBackoff {
    pub fn next_delay(&self) -> Duration {
        let multiplier = 10_u128.saturating_pow(self.failure_count);
        let delay = u128::from(self.base_delay_ms).saturating_mul(multiplier);
        let capped = delay.min(u128::from(self.max_delay_ms));
        Duration::from_millis(u64::try_from(capped).unwrap_or(u64::MAX))
    }
}
```

Note: the backoff uses base-10 exponential growth (`10^n`), not the more common base-2 (`2^n`). With a 100ms base delay: failure 1 = 1s, failure 2 = 10s, failure 3 = 100s, capped at 300s (5 minutes). The `reset_after_ms` field tracks how long the agent must run successfully before the failure count resets to zero.

---

## 4. Pure State Machine + Effect Driver Separation

### Why This Pattern Matters

The most architecturally significant pattern in `roko-runtime` is the separation between a **pure state machine** (no I/O, no side effects, fully deterministic) and an **effect driver** (executes I/O, talks to external services). This separation follows the same principle as the Elm Architecture [11] and algebraic effect systems [12]: express the "what" as a pure function, delegate the "how" to a separate runtime.

1. **Testability.** The state machine is a pure function: given a state and an input, it produces a deterministic output. You can test every state transition exhaustively without mocking LLM providers, git commands, or network calls.

2. **Replayability.** Feed a recorded sequence of `PipelineInput` events through the state machine and you reproduce the exact state transitions that occurred in production. This makes debugging workflow failures trivial.

3. **Serializability.** The state machine's state can be serialized to JSON at any point, written to disk, and later restored to resume the workflow from exactly where it left off. This enables crash recovery without re-executing expensive LLM calls.

4. **Composability.** Different effect drivers can be plugged in (test mocks, production services, replay drivers) without changing the state machine logic.

### The Pure State Machine: PipelineStateV2

**Source**: `crates/roko-runtime/src/pipeline_state.rs`

The module doc states the design rule explicitly:

> This is a PURE state machine with no side effects. It takes events and returns actions. The effect driver executes the actions.

The state machine represents a config-driven workflow pipeline with these phases:

```rust
// crates/roko-runtime/src/pipeline_state.rs, lines 426-450

pub enum Phase {
    Pending,
    Strategizing,
    Implementing,
    Gating,
    AutoFixing,
    Reviewing,
    Committing,
    Complete,
    Halted { reason: String },
    Cancelled,
}
```

Three workflow configurations determine which phases are active:

```
Express:  Implement -> Gate -> Commit
Standard: Implement -> Gate -> Review -> Commit
Full:     Strategy -> Implement -> Gate -> Review -> Commit
```

Inputs to the state machine (things that happened in the outside world):

```rust
// crates/roko-runtime/src/pipeline_state.rs, lines 477-550

pub enum PipelineInput {
    Start,
    StrategyComplete { brief: String },
    StrategySkipped,
    AgentCompleted { output: String, files_changed: u32 },
    AgentFailed { error: String },
    GatesPassed,
    GateFailed { gate: String, output: String },
    ReviewApproved { summary: String },
    ReviewRejected { reason: String },
    ReviewUnclear { summary: String },
    ReviewRevise { findings: Vec<String> },
    CommitFinished { outcome: CommitOutcome },
    UserCancel,
    ResourceExhausted { reason: String },
}
```

Outputs from the state machine (actions the effect driver should execute):

```rust
// crates/roko-runtime/src/pipeline_state.rs, lines 554-591

pub enum PipelineOutput {
    SpawnStrategist { prompt: String },
    SpawnImplementer { prompt: String, context: Option<String> },
    SpawnAutoFixer { error_output: String },
    RunGates,
    SpawnReviewer { diff_context: Option<String> },
    Commit,
    Done { outcome: WorkflowOutcome },
    Halt { reason: String },
}
```

The `step()` function is the core: a pure function from `(Phase, PipelineInput) -> PipelineOutput`:

```rust
// crates/roko-runtime/src/pipeline_state.rs, lines 663-688

/// Feed an event into the state machine, get an action back.
/// This is the ONLY way to drive the state machine. No side effects here.
pub fn step(&mut self, input: PipelineInput) -> PipelineOutput {
    match (&self.phase, input) {
        (Phase::Pending, PipelineInput::Start) => {
            if self.config.has_strategy {
                self.phase = Phase::Strategizing;
                PipelineOutput::SpawnStrategist { prompt: self.original_prompt.clone() }
            } else {
                self.phase = Phase::Implementing;
                self.iteration = 1;
                PipelineOutput::SpawnImplementer {
                    prompt: self.original_prompt.clone(),
                    context: None,
                }
            }
        }
        // ... more transitions ...
    }
}
```

Checkpoint and restore are built into the state machine:

```rust
// crates/roko-runtime/src/pipeline_state.rs, lines 643-658

/// Serialize the current pipeline state to a JSON string.
pub fn checkpoint(&self) -> Result<String, ...> {
    Ok(serde_json::to_string(self)?)
}

/// Restore from a JSON checkpoint.
pub fn from_checkpoint(json: &str) -> Result<Self, ...> {
    Ok(serde_json::from_str(json)?)
}
```

### The Effect Driver

**Source**: `crates/roko-runtime/src/effect_driver.rs`

The effect driver translates `PipelineOutput` actions into real-world side effects and returns `PipelineInput` results that feed back into the state machine.

```rust
// crates/roko-runtime/src/effect_driver.rs, lines 40-74

pub struct EffectServices {
    pub default_model: String,
    pub model_caller: Arc<dyn ModelCaller>,
    pub prompt_assembler: Arc<dyn PromptAssembler>,
    pub feedback_sink: Arc<dyn FeedbackSink>,
    pub gate_runner: Arc<dyn GateRunner>,
    pub affect_policy: Option<Arc<tokio::sync::Mutex<dyn AffectPolicy>>>,
}

pub struct EffectDriver {
    services: EffectServices,
    run_id: String,
    workdir: PathBuf,
    feedback_totals: tokio::sync::Mutex<WorkflowFeedbackTotals>,
}
```

The driver's methods map 1:1 to effect types:

```rust
impl EffectDriver {
    /// PipelineOutput::SpawnImplementer -> PipelineInput::AgentCompleted/AgentFailed
    pub async fn spawn_agent(&self, role: &str, prompt: &str, context: Option<&str>)
        -> PipelineInput;

    /// PipelineOutput::RunGates -> PipelineInput::GatesPassed/GateFailed
    pub async fn run_gates(&self, gates: &[String], shell_gates: &[ShellGateCommand])
        -> PipelineInput;

    /// PipelineOutput::Commit -> PipelineInput::CommitFinished
    pub async fn commit(&self, message: &str) -> PipelineInput;

    /// Checkpoint the state machine to disk.
    pub async fn save_checkpoint(&self, state: &PipelineStateV2, path: &Path) -> Result<()>;
}
```

The driver also integrates affect policy modulation. When an `AffectPolicy` is configured, the driver modulates model call parameters (temperature, token budget, cache policy) based on the agent's behavioral state:

```rust
// crates/roko-runtime/src/effect_driver.rs, lines 104-109 (within spawn_agent)

let mut modulation = DispatchModulation::default();
if let Some(ref affect) = self.services.affect_policy {
    let policy = affect.lock().await;
    let _ctx = policy.pre_dispatch(&agent_id, role);
    policy.modulate_dispatch(role, &mut modulation);
}
```

The modulation adjusts three parameters: `tier_bias` (model selection), `turn_limit_factor` (token budget scaling from 0.25x to 2.0x), and `exploration_rate` (temperature adjustment and cache bypass).

### The Complete Event Flow

```
User Prompt
     |
     v
PipelineStateV2::new(config, prompt)
     |
     v
step(Start) --> PipelineOutput::SpawnImplementer { prompt }
     |
     v
EffectDriver::spawn_agent("implementer", prompt, None)
     |  (LLM call, tool loop, file changes)
     v
PipelineInput::AgentCompleted { output, files_changed }
     |
     v
step(AgentCompleted) --> PipelineOutput::RunGates
     |
     v
EffectDriver::run_gates(enabled_gates, shell_gates)
     |  (compile, clippy, test, fmt, diff checks)
     v
PipelineInput::GatesPassed
     |
     v
step(GatesPassed) --> PipelineOutput::Commit  (or SpawnReviewer if review enabled)
     |
     v
EffectDriver::commit("implement: ...")
     |
     v
PipelineInput::CommitFinished { outcome: Created { hash } }
     |
     v
step(CommitFinished) --> PipelineOutput::Done { outcome: Success { commit_hash } }
```

At any point in this flow, the state machine can be checkpointed to JSON and restored later. The state machine never makes any I/O call -- it only decides what should happen next.

### How State Machine Separation Prevents Common Runtime Failures

**Failure: Untestable agent loop.**
Without separation, testing the agent loop requires mocking LLM providers, tool executors, git, and the filesystem. With separation, the state machine is a pure function -- test every transition with simple `assert_eq!(state.step(input), expected_output)` calls, no mocks needed.

**Failure: Lost progress after crash.**
A workflow completes strategy, implementation, and three of five gate checks, then crashes. Without checkpointing, all work is lost. With the state machine's `checkpoint()` method, the serialized state captures the current phase, iteration count, and accumulated context. On restart, `from_checkpoint()` restores the state and the workflow continues from the gate-checking phase.

---

## 5. Process Supervision

### What It Is

OS-level process lifecycle management: spawn, track, monitor, timeout, kill, and reap child processes and their entire process trees. This is the structural solution to three production failures documented in `docs/v1/07-conductor/13-process-supervision-wiring.md`: spawn races, orphaned cargo processes, and cold start overhead.

**Source**: `crates/roko-runtime/src/process.rs`

### Relationship to Erlang/OTP Supervision

The `ProcessSupervisor` is the OS-level counterpart to Erlang/OTP's supervision trees [8]. Armstrong's key insight was that in large systems, crashes are inevitable -- the question is not how to prevent all crashes but how to contain and recover from them. The supervisor pattern provides three guarantees: (1) every process has a supervisor, (2) supervisors restart failed processes according to a policy, (3) cascading failures are bounded by the supervision tree structure.

Roko applies these principles to OS processes rather than lightweight Erlang processes. A `ProcessSupervisor` manages a pool of `ProcessHandle` instances, each wrapping a `tokio::process::Child` with unique identity, cooperative shutdown, and exit tracking.

### ProcessSupervisor Architecture

```
ProcessSupervisor
  +-- ProcessHandle (PID 4201, Agent 1)
  |    +-- child: cargo (PID 4202)
  |    +-- grandchild: rustc (PID 4203)
  +-- ProcessHandle (PID 4205, Agent 2)
  +-- ProcessHandle (PID 4209, Agent 3)
```

Key types from `process.rs`:

```rust
// crates/roko-runtime/src/process.rs, lines 59, 555-567, 839-844

/// Monotonically increasing process identifier, unique within a single runtime.
pub struct ProcessId(pub u64);

/// Wraps a tokio::process::Child with:
/// - Unique ProcessId
/// - Cooperative shutdown with configurable grace period
/// - CancelToken integration for hierarchical cancellation
/// - Exit status tracking
pub struct ProcessHandle {
    pub id: ProcessId,
    pub label: String,
    child: Child,
    os_pid: Option<u32>,
    grace_period: Duration,
    cancel: CancelToken,
    spawn_config: SpawnConfig,
    session: Option<ProcessSessionConfig>,
    started_at: Instant,
}

/// Manages a pool of ProcessHandles with bulk operations.
pub struct ProcessSupervisor {
    handles: Arc<Mutex<HashMap<ProcessId, ProcessHandle>>>,
    restart_history: Mutex<HashMap<String, Vec<Instant>>>,
    cancel: CancelToken,
    strategy: SupervisionStrategy,
}
```

### Five Guarantees

The docs in `docs/v1/07-conductor/13-process-supervision-wiring.md` describe five guarantees:

1. **PID tracking.** Every spawned process is registered with its PID, parent PID, attempt ID, and plan association.

2. **Descendant discovery.** For any registered PID, the supervisor enumerates the full process tree (children, grandchildren) via platform-specific mechanisms (cgroups on Linux, `pgrep -P` on macOS).

3. **Lifecycle management.** Spawn, monitor, timeout, and terminate are atomic operations on the process tree, not individual processes.

4. **Orphan prevention.** On parent exit, all registered descendants are terminated. No process outlives its supervisor.

5. **Attempt isolation.** Each spawn attempt gets a monotonically increasing attempt ID. Exit events carry the attempt ID, preventing confusion between retries.

### SIGTERM -> SIGKILL Escalation

Process termination follows a two-phase protocol, implemented in `ProcessHandle::shutdown()` (line 778):

```
Phase 1: SIGTERM (graceful, configurable grace period)
    Process can: write checkpoint, flush buffers, close connections, exit cleanly
Phase 2: SIGKILL (forced, after grace period expires)
    Cannot be caught or ignored. Process terminated immediately.
```

The default grace period is 5 seconds (`DEFAULT_GRACE_PERIOD` at line 44). The `shutdown()` method first cancels the `CancelToken`, drops stdin, and waits for the process to exit within the grace period. If the process does not exit, it is killed forcefully.

### CancelToken Integration

The `ProcessHandle` integrates with the hierarchical `CancelToken` from the `cancel` module. Each process handle stores a `CancelToken` (line 563). Cancelling the token triggers graceful process termination via `shutdown()`. This means process lifecycle is automatically tied to the cancellation hierarchy -- cancelling a session token cascades through job tokens to process handles, terminating all spawned processes.

### Supervisor Operations

```rust
// Key supervisor operations (process.rs, lines 845-951)

impl ProcessSupervisor {
    /// Create a new supervisor with a root cancellation token.
    pub fn new(cancel: CancelToken) -> Self;

    /// Shut down a single managed process by ID.
    pub async fn shutdown(&self, id: ProcessId) -> Option<ProcessOutcome>;

    /// Shut down all managed processes, returning their outcomes.
    pub async fn shutdown_all(&self) -> Vec<ProcessOutcome>;
}
```

The `shutdown_all` method first cancels the supervisor's root `CancelToken`, then drains all handles from the map and shuts them down. This ensures that even if individual shutdown calls hang, the cancellation signal has been propagated.

### How Supervision Prevents Common Runtime Failures

**Failure: Orphaned cargo/rustc processes.**
An agent spawns `cargo check`, which spawns `rustc`. The agent is cancelled, but only `cargo` is killed -- `rustc` continues consuming CPU. With `ProcessSupervisor`, the handle's `shutdown()` terminates the entire process tree.

**Failure: Stale processes from previous runs.**
The supervisor's process session ledger tracks all spawned processes persistently. On restart, the supervisor can check for and clean up processes from a previous session that may still be running.

---

## 6. StateHub (Dashboard Projections)

### What It Is

The StateHub is the single source of truth for all dashboard consumers. It bridges the event bus to a materialized `DashboardSnapshot` via a `tokio::sync::watch` channel, serving three consumer interfaces with different performance characteristics.

**Source**: `crates/roko-runtime/src/state_hub.rs`
**Docs**: `docs/v1/12-interfaces/22-statehub-projection-layer.md`

### Architecture

```
Orchestrator
    | publish(DashboardEvent)
    v
StateHub
    |-- watch<DashboardSnapshot>  <- TUI reads (60fps, zero-copy borrow)
    |-- broadcast<DashboardEvent> <- WebSocket/SSE clients subscribe
    +-- ring buffer (1024)        <- replay for late joiners
```

### Implementation

```rust
// crates/roko-runtime/src/state_hub.rs, lines 80-86

pub struct StateHub {
    snapshot_tx: watch::Sender<DashboardSnapshot>,
    snapshot_rx: watch::Receiver<DashboardSnapshot>,
    event_bus: EventBus<DashboardEvent>,
    event_log: Option<SharedEventLog>,
}
```

The `publish` method is the critical path. Each call does four things atomically:

```rust
// crates/roko-runtime/src/state_hub.rs, lines 144-153

pub fn publish(&self, event: DashboardEvent) -> u64 {
    let seq = self.event_bus.emit(event.clone());
    self.snapshot_tx.send_modify(|snap| snap.apply(&event));
    if let Some(log) = &self.event_log {
        if let Ok(mut writer) = log.lock() {
            writer.append(&event);
        }
    }
    seq
}
```

1. Broadcasts the event to live subscribers (WebSocket, SSE)
2. Records the event in the replay ring for late joiners
3. Applies the event to the materialized snapshot so the TUI can borrow it
4. Optionally appends to the on-disk event log (`.roko/events.jsonl`)

### Three Consumer Interfaces

| Consumer | Method | Performance |
|----------|--------|-------------|
| **TUI** | `snapshot()` returns `watch::Receiver` | 60fps, zero-copy borrow via `borrow_and_update()` |
| **WebSocket/SSE** | `subscribe_events()` returns `broadcast::Receiver` | Live event stream |
| **REST API** | `current_snapshot()` returns `DashboardSnapshot` | Clone on demand |

### StateHubSender

Like `BusSender` for the EventBus, the `StateHubSender` is a clone-safe, send-safe handle for publishing events without full hub access:

```rust
// crates/roko-runtime/src/state_hub.rs, lines 287-306

#[derive(Clone)]
pub struct StateHubSender {
    snapshot_tx: watch::Sender<DashboardSnapshot>,
    bus_sender: event_bus::BusSender<DashboardEvent>,
    event_log: Option<SharedEventLog>,
}

impl StateHubSender {
    pub fn publish(&self, event: DashboardEvent) -> u64 {
        let seq = self.bus_sender.emit(event.clone());
        self.snapshot_tx.send_modify(|snap| snap.apply(&event));
        if let Some(log) = &self.event_log {
            if let Ok(mut writer) = log.lock() {
                writer.append(&event);
            }
        }
        seq
    }
}
```

### Event Log Persistence and Replay

The StateHub can persist events to a JSONL file and replay them to reconstruct state:

```rust
// crates/roko-runtime/src/state_hub.rs, lines 201-205

/// Replay events from the on-disk event log into the snapshot.
pub fn replay_from_log(log_path: &Path) -> (Self, usize) {
    let mut hub = Self::default_capacity();
    let count = hub.ingest_log(log_path);
    (hub, count)
}
```

This enables crash recovery: on restart, the hub replays the event log to reconstruct the dashboard state without re-executing any workflows.

### SharedStateHub

For concurrent access across async tasks, the hub is wrapped in an `Arc`:

```rust
// crates/roko-runtime/src/state_hub.rs, lines 310-327

#[derive(Clone)]
pub struct SharedStateHub(Arc<StateHub>);

impl SharedStateHub {
    pub fn new_in_process() -> Self;
    pub fn bootstrap_from_workdir(&self, workdir: &Path) -> Result<(), io::Error>;
}
```

The `bootstrap_from_workdir` method seeds the materialized snapshot from on-disk state files (executor state, plan states, gate results) when starting a standalone dashboard.

### Target Architecture: Named Projections

The docs in `docs/v1/12-interfaces/22-statehub-projection-layer.md` describe the target evolution toward named projections with a formal `Projection` trait:

> Every projection is defined once and shared by every consumer. The projection name is stable, the state shape is typed, and deltas are folded in one place.

The target trait surface:

```rust
pub trait Projection: Send + Sync + 'static {
    const NAME: &'static str;
    type State: Serialize + DeserializeOwned + Clone + Send + 'static;
    type Delta: Serialize + DeserializeOwned + Clone + Send + 'static;

    fn apply(state: &mut Self::State, delta: Self::Delta);
    fn topics() -> &'static [&'static str];
    async fn hydrate(ctx: &ProjectionContext) -> Result<Self::State>;
    fn reduce(pulse: &Pulse) -> Option<Self::Delta>;
}
```

---

## 7. State Snapshots and Event Persistence

### Checksummed State Snapshots

**Source**: `crates/roko-runtime/src/state_snapshot.rs`

All mutable runtime state is bundled into a single atomic snapshot with SHA-256 integrity verification:

```rust
// crates/roko-runtime/src/state_snapshot.rs, lines 17-34

pub struct StateSnapshot {
    pub version: u32,                // Schema version for compatibility
    pub timestamp_ms: u64,           // Wall-clock time
    pub executor_json: String,       // Executor state (opaque)
    pub orchestrator_json: String,   // Orchestrator state (merge queue)
    pub run_state_json: String,      // Run counters
    pub gate_thresholds_json: String, // Gate threshold EMA state
    pub checksum: String,            // SHA-256 of concatenated payloads
}
```

The checksum is computed over all four JSON payloads and verified on load:

```rust
// crates/roko-runtime/src/state_snapshot.rs, lines 86-98

fn compute_checksum(executor: &str, orchestrator: &str, run_state: &str, gate_thresholds: &str)
    -> String
{
    let mut hasher = Sha256::new();
    hasher.update(executor.as_bytes());
    hasher.update(orchestrator.as_bytes());
    hasher.update(run_state.as_bytes());
    hasher.update(gate_thresholds.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

The `verify()` method checks both version compatibility and checksum integrity. This prevents two classes of bugs: using a snapshot from an incompatible code version, and loading a snapshot that was partially written during a crash.

### JSONL Event Logger

**Source**: `crates/roko-runtime/src/jsonl_logger.rs`

Runtime events are persisted to JSONL for replay and state reconstruction:

```rust
// crates/roko-runtime/src/jsonl_logger.rs

pub struct JsonlLogger {
    path: PathBuf,
    seq: AtomicU64,
    writer: Mutex<Option<BufWriter<File>>>,
}

impl EventConsumer for JsonlLogger {
    fn consume(&self, event: &RuntimeEvent) {
        let _ = self.write_event(event);
    }
}
```

Each event is wrapped in a `RuntimeEventEnvelope` with run_id, sequence number, source label, and the event payload, then serialized as a single JSON line and flushed immediately. A contract guard test (`jsonl_logger_does_not_serialize_events_as_debug_strings`) enforces that events are serialized with `serde_json`, not Rust `Debug` formatting -- ensuring forward-compatible deserialization.

### RuntimeProjection: Reconstructing State from Logs

**Source**: `crates/roko-runtime/src/projection.rs`

The `RuntimeProjection` reads JSONL event logs and builds per-run summaries:

```rust
// crates/roko-runtime/src/projection.rs

pub struct RunSummary {
    pub run_id: String,
    pub template: Option<String>,
    pub prompt: Option<String>,
    pub current_phase: Option<String>,
    pub phases_visited: Vec<String>,
    pub gates_passed: Vec<String>,
    pub gates_failed: Vec<String>,
    pub agents_spawned: u32,
    pub agents_completed: u32,
    pub agents_failed: u32,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub is_complete: bool,
    pub outcome: Option<String>,
    pub last_checkpoint: Option<String>,
    pub agent_errors: Vec<String>,
}
```

This enables the "resume" workflow: read the event log, reconstruct where the workflow was, and continue from the last checkpoint.

---

## 8. Resource Accounting

**Source**: `crates/roko-runtime/src/resource.rs`

Per-plan and per-task resource consumption is tracked against budgets:

```rust
// crates/roko-runtime/src/resource.rs, lines 11-25

pub struct ResourceAccount {
    pub tokens: BudgetEntry<u64>,
    pub cost: BudgetEntry<f64>,
    pub time_limit: Duration,
    pub started_at: Option<Instant>,
    pub label: String,
}
```

Pre-defined budget tiers:

| Tier | Token Limit | Cost Limit | Time Limit |
|------|-------------|------------|------------|
| Trivial | 50,000 | $0.50 | 5 minutes |
| Simple | 200,000 | $2.00 | 15 minutes |
| Standard | 500,000 | $5.00 | 30 minutes |
| Complex | 2,000,000 | $20.00 | 60 minutes |

The `any_exceeded()` method checks all three dimensions. When a budget is exceeded, the system can throttle (reduce model tier), warn (emit degradation event), or halt (terminate the workflow). The account also provides utilisation fractions (`token_utilisation()`, `cost_utilisation()`, `time_utilisation()`) for dashboard rendering and degradation policy decisions.

---

## 9. Run Ledger

**Source**: `crates/roko-runtime/src/run_ledger.rs`

The `RunLedger` provides a typed source of truth for a single workflow run, replacing the need to replay the entire event bus to reconstruct a report:

```rust
pub struct RunLedger {
    pub run_id: String,
    pub prompt: String,
    pub workflow: WorkflowConfig,
    pub started_at_ms: u64,
    pub phase_history: Vec<PhaseTransitionRecord>,
    pub agent_outcomes: Vec<AgentOutcome>,
    pub gate_runs: Vec<GateRunOutcome>,
    pub artifacts: Vec<ArtifactOutcome>,
    pub commit: Option<CommitOutcome>,
    pub cancellation: Option<CancellationOutcome>,
    pub event_persistence: EventPersistenceHealth,
    pub checkpoint_path: Option<PathBuf>,
}
```

Agent outcomes are typed enums that capture the full context of each effect:

```rust
pub enum AgentOutcome {
    Completed {
        role: String,
        output: String,
        files_changed: u32,
        requested_model: String,
        routed_model: Option<String>,
        final_model: String,
        provider_id: String,
        usage: TokenUsage,
        request_id: Option<String>,
    },
    Failed {
        role: String,
        kind: EffectErrorKind,
        message: String,
    },
}
```

The ledger provides a `to_report_compat()` method that bridges the typed ledger fields to the legacy `WorkflowRunReport` shape without replaying the event bus.

---

## IronClaw Integration Plan

IronClaw's current runtime architecture in `src/agent/` uses a different set of patterns than roko-runtime, but the core problems are the same. This section maps each roko pattern to IronClaw's existing architecture and identifies concrete integration points.

### Current IronClaw Architecture (src/agent/)

IronClaw's agent module (`src/agent/CLAUDE.md`) defines the following execution model:

- **Session/Thread/Turn model**: Sessions contain threads, threads contain turns. Each turn pairs a user input with a response, tool calls, and a state (`Pending | Running | Complete | Failed`).
- **Shared agentic loop** (`agentic_loop.rs`): Three delegates (`ChatDelegate`, `JobDelegate`, `ContainerDelegate`) drive a common loop: check signals, pre-LLM hook, LLM call, handle response, execute tools, post-iteration hook.
- **Scheduler** (`scheduler.rs`): Manages concurrent jobs under `Arc<RwLock<HashMap>>` with cleanup polling.
- **Cost guard** (`cost_guard.rs`): Daily budget (cents) and hourly call rate enforcement.
- **Heartbeat** (`heartbeat.rs`): Proactive periodic execution reading `HEARTBEAT.md`.
- **Self-repair** (`self_repair.rs`): Stuck job detection and recovery.
- **Context monitor** (`context_monitor.rs`): Memory pressure detection and compaction triggers.

### A. Hierarchical Cancellation

**Where**: `src/agent/agent_loop.rs`, `src/agent/agentic_loop.rs`, `src/agent/scheduler.rs`
**Current state**: IronClaw uses `tokio::select!` with channel-based shutdown signals and `check_signals()` in the agentic loop delegate. Each subsystem (heartbeat, scheduler, session manager) manages its own shutdown independently.

**Integration plan:**

```rust
// In session creation (session_manager.rs):
let session_token = CancelToken::new();

// In job dispatch (scheduler.rs):
let job_token = session_token.child();

// In tool execution (agentic_loop.rs execute_tool_calls):
let tool_token = job_token.child();

// In heartbeat (heartbeat.rs):
let heartbeat_token = session_token.child();
```

The `check_signals()` method in `LoopDelegate` currently checks for stop/cancel via channel reads. With cancellation tokens, this becomes:

```rust
// In agentic_loop.rs, before each LLM call:
tokio::select! {
    result = delegate.call_llm(messages) => handle_result(result),
    () = cancel_token.cancelled() => return LoopOutcome::Cancelled,
}
```

Benefits over current approach:
- A new background task (routine, consolidation) automatically inherits cancellation from its parent scope without manual channel wiring
- Tool call timeouts and user cancellation flow through the same token hierarchy
- `SessionManager` pruning no longer needs separate shutdown channels

**Complexity**: ~200-300 lines to replace existing cancellation wiring. The `CancelToken` is already `Clone + Send + Sync` and requires no wrapper.

### B. EventBus for Observability

**Where**: `src/observability/`, `src/channels/web/`
**Current state**: IronClaw has `noop` and `log` observer backends in `src/observability/`. The web gateway uses SSE for real-time updates but constructs events ad-hoc.

**Integration plan:**

```rust
use roko_runtime::event_bus::EventBus;

#[derive(Debug, Clone)]
enum IronClawEvent {
    SessionStarted { session_id: String, user_id: String },
    ToolDispatched { tool: String, session_id: String },
    ToolCompleted { tool: String, duration_ms: u64, success: bool },
    LlmCallStarted { provider: String, model: String },
    LlmCallCompleted { tokens: u64, cost_usd: f64, latency_ms: u64 },
    HeartbeatFired { findings: Vec<String> },
    JobStateChanged { job_id: String, from: JobState, to: JobState },
    CostGuardTriggered { budget_remaining_cents: i64 },
}

// Global bus with 2048 replay capacity
static EVENT_BUS: OnceLock<EventBus<IronClawEvent>> = OnceLock::new();
```

Integration points:
- **Web SSE gateway** (`src/channels/web/`): Replace ad-hoc event construction with `event_bus().subscribe()`. Late-joining clients get recent history via `replay_from()`.
- **Observability backends** (`src/observability/`): Add an `EventBusObserver` that subscribes and logs/records structured events.
- **Cost guard** (`src/agent/cost_guard.rs`): Emit `CostGuardTriggered` events when budgets are approached or exceeded.
- **Self-repair** (`src/agent/self_repair.rs`): Subscribe to `JobStateChanged` events to detect stuck jobs reactively instead of polling.

**Complexity**: ~400-500 lines for event type definitions, bus initialization, and SSE/WebSocket adapters.
**Risk**: Low. The EventBus is purely additive and does not replace existing logging.

### C. Agent Lifecycle Management

**Where**: `src/agent/agent_loop.rs`, `src/agent/session.rs`
**Current state**: Agent state is tracked through the job state machine (`Pending -> InProgress -> Completed -> Submitted -> Accepted`) and `ThreadState` (`Idle, Processing, AwaitingApproval, Completed, Interrupted`).

**Integration plan:**

Map IronClaw's existing states to FIPA lifecycle states:

| IronClaw State | FIPA Lifecycle State | Transition Reason |
|---------------|---------------------|-------------------|
| Session created | `Initiated` | `OperatorCreate` |
| Agent deps loaded | `Provisioning` | `ManifestValidated` |
| First message processed | `Active` | `RuntimeReady` |
| Thread idle | `Waiting` | `ExternalWait` |
| User disconnected | `Suspended` | `OperatorPause` |
| Budget exceeded | `Degraded(ModelDowngrade)` | `BudgetConstrained` |
| Session destroyed | `Deleted` | `OperatorDelete` |

```rust
// In agent_loop.rs, emit transitions on state changes:
fn emit_transition(&self, from: AgentLifecycleState, to: AgentLifecycleState,
                   reason: LifecycleTransitionReason) {
    event_bus().emit(IronClawEvent::LifecycleTransition(
        LifecycleTransition::new(&self.session_id, from, to, reason)
    ));
}

// Liveness check (usable by heartbeat or health endpoints):
fn liveness_check(&self) -> bool {
    self.last_activity.elapsed() < Duration::from_secs(300)
}

// Readiness check (can accept new work):
fn readiness_check(&self) -> bool {
    matches!(self.thread_state, ThreadState::Idle)
        && !self.cost_guard.budget_exceeded()
}
```

**Complexity**: ~300 lines to add lifecycle state tracking and probe checks.
**Risk**: Low. This is metadata and health reporting, not control flow changes.

### D. State Machine Extraction (Long-term)

**Where**: `src/agent/agentic_loop.rs`, `src/agent/dispatcher.rs`
**Current state**: The shared agentic loop in `agentic_loop.rs` mixes state transitions (check signals, pre-LLM hook, handle response, execute tools) with I/O operations (LLM calls, tool dispatch, database writes). The `LoopDelegate` trait partially separates concerns but delegates still perform I/O directly.

The roko `PipelineStateV2 + EffectDriver` pattern demonstrates how to fully extract the state machine:

1. Define an `AgentPhase` enum: `Idle`, `Processing`, `WaitingForLlm`, `ExecutingTools`, `AwaitingApproval`, `Reflecting`, `Compacting`
2. Define `AgentInput` (UserMessage, LlmResponse, ToolResult, ApprovalDecision, Timeout, Cancel, CompactionNeeded) and `AgentOutput` (CallLlm, DispatchTool, SendResponse, WriteMemory, RequestApproval, RunCompaction)
3. Implement a pure `step(input) -> output` function
4. Build an `AgentEffectDriver` that executes the outputs and feeds results back as inputs

This maps directly to the existing `LoopDelegate` methods:

| LoopDelegate method | AgentInput | AgentOutput |
|-------------------|------------|-------------|
| `check_signals()` | `Cancel`, `Timeout` | `Done(Cancelled)` |
| `before_llm_call()` | -- | `CallLlm(messages)` |
| `call_llm()` | `LlmResponse(text|tools)` | -- |
| `handle_text_response()` | `LlmResponse::Text` | `SendResponse`, `Done` |
| `execute_tool_calls()` | `ToolResult` | `DispatchTool` |
| `after_iteration()` | -- | `Continue`, `Done` |

Benefits:
- The agent loop becomes fully testable without mocking any external service
- Replay: feed recorded inputs through the state machine to reproduce any bug
- Checkpoint: serialize the state machine to JSON and restore after a crash

**Complexity**: Large refactor, 1000+ lines changed across `src/agent/`.
**Risk**: High. This touches the core agent loop and requires careful migration. Recommended as a phased refactor after cancellation and EventBus are stable.

### Recommended Adoption Order

```
Phase 1 (Low risk, high value):
  1. CancellationTokens (~200-300 lines)
     - Replace channel-based shutdown in agent_loop, scheduler, heartbeat
     - Immediate safety win: no more orphaned background tasks

Phase 2 (Low risk, additive):
  2. EventBus (~400-500 lines)
     - Add structured event emission to tool dispatch, LLM calls, cost guard
     - Wire SSE gateway to EventBus subscribers
     - Enables real-time dashboard without ad-hoc event construction

Phase 3 (Low risk, metadata only):
  3. Lifecycle State Machine (~300 lines)
     - Add FIPA-informed lifecycle tracking to sessions
     - Health probe endpoints for monitoring
     - Budget degradation stages integrated with cost guard

Phase 4 (High risk, highest value):
  4. Pure State Machine Extraction (~1000+ lines)
     - Refactor agentic loop into pure step() + effect driver
     - Enables checkpointing, replay, exhaustive testing
```

---

## Complexity Assessment

| Component | Lines | Risk | Dependencies |
|-----------|-------|------|-------------|
| Hierarchical cancellation | ~200-300 | Low | `tokio` (already in use) |
| EventBus with replay ring | ~400-500 | Low | `tokio`, `parking_lot` |
| Lifecycle state machine | ~300 | Low | `chrono`, `serde` |
| Health probes | ~200 | Low | `tokio` |
| StateHub projections | ~500 | Medium | EventBus, `serde_json` |
| Pure state machine extraction | ~1000+ | High | Core agent loop refactor |
| Process supervision | ~800 | Medium | Platform-specific (`nix` for Unix signals) |
| State snapshots + JSONL persistence | ~300 | Low | `serde_json`, `sha2` |
| Resource accounting | ~200 | Low | `serde` |
| Run ledger | ~300 | Low | `serde`, pipeline state types |

---

## References

[1] M. Fowler, "Event Sourcing," martinfowler.com, December 2005. Available: https://martinfowler.com/eaaDev/EventSourcing.html

[2] M. Kleppmann, *Designing Data-Intensive Applications*, O'Reilly Media, 2017.

[3] P. Helland, "Immutability Changes Everything," in *Proc. 7th Biennial Conference on Innovative Data Systems Research (CIDR)*, January 2015.

[4] S. Ajmani, "Go Concurrency Patterns: Context," The Go Blog, July 2014. Available: https://pkg.go.dev/context

[5] R. Elizarov, "Structured Concurrency," Medium, October 2018. Available: https://elizarov.medium.com/structured-concurrency-722d765aa952

[6] N. J. Smith, "Notes on Structured Concurrency, or: Go Statement Considered Harmful," April 2018.

[7] Foundation for Intelligent Physical Agents, "FIPA Agent Management Specification," FIPA00023, 2002. Available: http://www.fipa.org/specs/fipa00023/XC00023H.html. Now maintained under IEEE Computer Society.

[8] J. Armstrong, "Making Reliable Distributed Systems in the Presence of Software Errors," PhD thesis, Royal Institute of Technology (KTH), Stockholm, 2003.

[9] Cliffle, "The Typestate Pattern in Rust," 2019. Available: https://cliffle.com/blog/rust-typestate/

[10] O. Kiselyov and K. Imai, "Session Types Without Sophistry: System Description," in *Proc. FLOPS*, 2020.

[11] E. Czaplicki, "Elm: Concurrent FRP for Functional GUIs," Senior thesis, Harvard University, 2012.

[12] A. Bauer and M. Pretnar, "An Effect System for Algebraic Effects and Handlers," in *Logical Methods in Computer Science*, vol. 10, no. 4, 2014. Available: https://arxiv.org/abs/1306.6316

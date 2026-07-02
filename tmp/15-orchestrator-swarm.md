# Orchestrator & Swarm Coordination

**Source crates**: `roko-orchestrator`, `roko-runtime`
**Priority**: MEDIUM -- multi-job orchestration, event sourcing
**Roko code path**: `crates/roko-orchestrator/src/`

---

## Table of Contents

1. [Why Orchestration Matters for AI Systems](#1-why-orchestration-matters-for-ai-systems)
2. [Architectural Overview](#2-architectural-overview)
3. [The Pure State Machine](#3-the-pure-state-machine)
4. [Event Sourcing: Journal, Replay, and Recovery](#4-event-sourcing-journal-replay-and-recovery)
5. [Unified Cross-Plan Task DAG](#5-unified-cross-plan-task-dag)
6. [Wave Scheduling Algorithm](#6-wave-scheduling-algorithm)
7. [File-Conflict Inference](#7-file-conflict-inference)
8. [Three-Level Recovery System](#8-three-level-recovery-system)
9. [BLAKE3 Hash-Linked Audit Chain](#9-blake3-hash-linked-audit-chain)
10. [Live DAG Mutation](#10-live-dag-mutation)
11. [Pheromone-Based Swarm Coordination](#11-pheromone-based-swarm-coordination)
12. [Worked Examples: Multi-Agent Execution with Recovery](#12-worked-examples-multi-agent-execution-with-recovery)
13. [IronClaw Integration](#13-ironclaw-integration)
14. [References](#14-references)
15. [Complexity Assessment](#15-complexity-assessment)

---

## 1. Why Orchestration Matters for AI Systems

When an AI system handles a single request with a single agent, orchestration is trivial -- it is just a request-response loop. But when you need to decompose a large task into subtasks, assign those subtasks to multiple agents running in parallel, handle failures gracefully, merge results safely, and survive crashes without losing progress, you need a proper orchestration layer.

The naive approach -- spawning agents ad hoc, polling for completion, retrying on failure -- quickly becomes untenable. Race conditions corrupt shared state. Crash recovery requires complex ad hoc logic. Debugging production failures means sifting through unstructured logs with no causal ordering. Auditing becomes impossible because there is no authoritative record of what decisions were made and why.

Roko's orchestrator solves this with a pattern borrowed from distributed systems: **a pure state machine driven by an event-sourced journal**. This is the same pattern used by systems like Apache Flink (stream processing checkpoints), Temporal.io (workflow replay), and event-sourced CQRS architectures [Fowler2005, Temporal2024]. The insight is that if you separate *computation of the next action* (pure function) from *performing the action* (I/O), you get deterministic replay, trivial crash recovery, and a tamper-evident audit trail for free.

### The Core Insight

The orchestrator never performs I/O. It is a function:

```
(current_state, event) -> (new_state, actions_to_perform)
```

The runtime harness performs the I/O (spawning agents, running tests, merging branches), then feeds the results back as new events. This separation has three consequences:

1. **Deterministic replay**: Given the same event sequence, the state machine always reaches the same state. Useful for debugging, testing, and crash recovery. This is the same guarantee that Temporal.io's durable execution engine provides: "workflows must be deterministic -- given the same inputs, they produce the same sequence of steps" [Temporal2024].
2. **Crash safety**: Write the event to the journal *before* performing the I/O. If you crash mid-action, replay the journal on restart to reconstruct state, then re-dispatch the pending action.
3. **Auditability**: The event journal is a complete, ordered record of every decision. Hash-chain it and you get tamper evidence.

---

## 2. Architectural Overview

The orchestrator crate (`roko-orchestrator`) is organized into several modules, each handling a distinct concern. Understanding the module layout is essential before diving into any single subsystem.

```
crates/roko-orchestrator/src/
  lib.rs              -- Crate root: ParallelExecutor, ExecutorConfig, ResourceBudget
  dag.rs              -- UnifiedTaskDag: cross-plan DAG, wave scheduling, file-overlap
  event_log.rs        -- EventLog: append-only BLAKE3 hash-chained event journal
  coordination.rs     -- Pheromone types, subnets, collectives, swarm coordination
  mesh_relay.rs       -- WebSocket relay for pheromone synchronization across peers
  merge_queue.rs      -- Ordered merge queue with dependency-aware sequencing
  plan_discovery.rs   -- Filesystem plan discovery with YAML frontmatter parsing
  post_merge.rs       -- Post-merge verification and cleanup
  progress.rs         -- Progress tracking and reporting
  repair.rs           -- Plan repair and healing
  replan.rs           -- Re-planning strategies (retry, escalate, decompose, skip, regenerate)
  runtime_snapshot.rs -- Runtime snapshot support
  service_factory.rs  -- Service construction helpers
  worktree.rs         -- Git worktree isolation manager

  executor/
    mod.rs            -- ParallelExecutor: main orchestration engine
    action.rs         -- ExecutorAction enum: the vocabulary of side-effects
    plan_state.rs     -- PlanState: per-plan mutable state
    state_machine.rs  -- PlanStateMachine: phase transition logic (pure function)
    snapshot.rs       -- ExecutorSnapshot, DeltaSnapshot, SnapshotVerifier (BLAKE3 envelope)
    recovery.rs       -- RecoveryEngine: crash recovery from snapshots + event replay
    reorder.rs        -- Queue reordering strategies
    priority_ceiling.rs -- Priority inheritance for resource contention
    resource_budget.rs  -- Token/cost/rate-limit budgets

  safety/
    audit_chain.rs    -- AuditChain: BLAKE3 hash-linked append-only audit log
    capability_tokens.rs -- Capability-based agent permissions
    loop_guard.rs     -- Infinite loop detection
    permit.rs         -- Permit issuance for privileged operations
    sandboxing.rs     -- Agent sandboxing controls
    taint_propagation.rs -- Taint tracking through agent outputs
```

*Source: directory listing of `crates/roko-orchestrator/src/`*

### Design Principles

The orchestrator doc (`docs/v2/27-ORCHESTRATOR.md`) states seven design principles:

| Principle | Summary |
|-----------|---------|
| **P1: Single-threaded event loop, async I/O** | All mutations happen in one `tokio::select!` loop. No race conditions on state. |
| **P2: Channels as event bus** | Four independent channels (agent events, executor actions, gate results, TUI input) decouple subsystems. |
| **P3: Stream, don't batch** | Parse agent output line-by-line as it arrives. Real-time TUI updates. |
| **P4: Flush after every task** | Write snapshot + episode + efficiency event after each task. Crash loses at most one task. |
| **P5: The plan dir is the plan** | `tasks.toml` is the source of truth. No scanning, no overwriting. |
| **P6: Executor is pure, runner does I/O** | `tick()` returns actions. Runner dispatches. Results fed back as events. |
| **P7: Align with unified spec** | Use naming conventions from the unified spec. Structure as a precursor to the Engine. |

*Source: `docs/v2/27-ORCHESTRATOR.md`, section 2*

---

## 3. The Pure State Machine

### Plan Phase Lifecycle

Every plan flows through a defined set of phases. The `PlanStateMachine` is a zero-sized struct that contains no mutable state -- it is a pure function from `(PlanState, ExecutorEvent) -> PlanPhase`. All mutable state lives in the `PlanState` struct, owned by the `ParallelExecutor`.

The complete phase lifecycle, with every legal transition:

```
                                  +---------+
                                  | Queued  |
                                  +----+----+
                                       |
                                  Start|
                                       v
                                  +---------+
                                  |Enriching|
                                  +----+----+
                                       |
                            EnrichmentDone|
                                       v
                              +------------+
                         +--->|Implementing|<---+
                         |    +------+-----+    |
                         |           |          |
               ReviewRejected  ImplDone    |
                         |           |          |
                         |           v          |
                         |    +--------+        |
                    +----+    | Gating |        |
                    |         +---+--+-+        |
                    |             |  |           |
                    |     GatePassed GateFailed  |
                    |             |  |           |
                    |             |  v           |
                    |             |  +---------+ |
                    |             |  |AutoFixing| |
                    |             |  +----+----+ |
                    |             |       |      |
                    |             | AutoFixDone  |
                    |             |       |      |
                    |             |       +------+
                    |             |         (back to Gating)
                    |             v
                    |       +----------+
                    |       | Verifying|
                    |       +---+----+-+
                    |           |    |
                    |   VerifyPassed  VerifyFailed
                    |           |    |
                    |           |    v
                    |           | +------------------+
                    |           | |RegeneratingVerify|
                    |           | +--------+---------+
                    |           |          |
                    |           |  VerifyRegenDone
                    |           |          |
                    |           |          +--->(back to Verifying)
                    |           v
                    |    +-----------+
                    +----| Reviewing |
                         +-----+----+
                               |
                       ReviewApproved
                               |
                               v
                        +-----------+
                        |DocRevision|
                        +-----+-----+
                              |
                       DocRevisionDone
                              |
                              v
                        +---------+
                        | Merging |
                        +----+----+
                             |
                  MergeSucceeded / MergeFailed
                        |           |
                        v           v
                   +----------+ +--------+
                   | Complete | | Failed |
                   +----------+ +--------+

    (Any non-terminal phase) --Skip--> [Skipped]
    (Any non-terminal phase) --Fatal(reason)--> [Failed]
    [Done] --OperatorMerge--> [Merging]
```

### The ExecutorEvent Enum

Every transition is triggered by an `ExecutorEvent`. Here is the complete enum from the source:

```rust
// Source: crates/roko-orchestrator/src/executor/state_machine.rs, lines 52-87

pub enum ExecutorEvent {
    /// Plan has been dispatched -- start enrichment.
    Start,
    /// Enrichment completed successfully.
    EnrichmentDone,
    /// Implementation completed (all tasks done).
    ImplementationDone,
    /// A gate passed.
    GatePassed,
    /// A gate failed.
    GateFailed,
    /// Auto-fix completed -- retry gating.
    AutoFixDone,
    /// Verification (verify-chain) passed.
    VerifyPassed,
    /// Verification (verify-chain) failed -- needs regeneration.
    VerifyFailed,
    /// Verify regeneration completed -- retry verification.
    VerifyRegenDone,
    /// Review approved -- proceed to doc revision.
    ReviewApproved,
    /// Review requested rework -- back to implementing.
    ReviewRejected,
    /// Doc revision completed.
    DocRevisionDone,
    /// Merge succeeded.
    MergeSucceeded,
    /// Merge failed.
    MergeFailed,
    /// Done phase: operator triggers merge.
    OperatorMerge,
    /// Operator requested skip.
    Skip,
    /// Unrecoverable failure with reason.
    Fatal(String),
}
```

*Source: `crates/roko-orchestrator/src/executor/state_machine.rs`, lines 52-87*

### The Transition Function

The transition function is the heart of the state machine. It takes the current `PlanState` plus an event, and returns either a new `PlanPhase` or a `TransitionError`. Before returning, it validates the proposed transition against a canonical transition table (`valid_transitions`) defined in `roko-core`.

Key implementation detail: the Gating -> AutoFixing transition has a **bounded retry loop**. The constant `MAX_AUTO_FIX_ITERATIONS` (sourced from `roko_core::defaults::DEFAULT_MAX_AUTO_FIX_ITERATIONS`) caps how many times a plan can cycle through AutoFixing -> Gating before it is declared terminally failed with `FailureKind::AutoFixExhausted`. Similarly, merge failures are capped at `MAX_MERGE_ATTEMPTS = 3` before declaring `FailureKind::Deadlock`.

```rust
// Source: crates/roko-orchestrator/src/executor/state_machine.rs, lines 110-224

pub fn transition(
    plan_state: &PlanState,
    event: &ExecutorEvent,
) -> Result<PlanPhase, TransitionError> {
    let current = &plan_state.current_phase;
    let current_kind = current.kind();

    let next = match (current_kind, event) {
        // -- Queued --
        (PhaseKind::Queued, ExecutorEvent::Start) => PlanPhase::Enriching,
        (PhaseKind::Queued, ExecutorEvent::Skip) => PlanPhase::Skipped,

        // -- Enriching --
        (PhaseKind::Enriching, ExecutorEvent::EnrichmentDone) => PlanPhase::Implementing,
        (PhaseKind::Enriching, ExecutorEvent::Skip) => PlanPhase::Skipped,

        // -- Implementing --
        (PhaseKind::Implementing, ExecutorEvent::ImplementationDone) => PlanPhase::Gating,
        (PhaseKind::Implementing, ExecutorEvent::Skip) => PlanPhase::Skipped,

        // -- Gating (with bounded auto-fix loop) --
        (PhaseKind::Gating, ExecutorEvent::GatePassed) => PlanPhase::Verifying,
        (PhaseKind::Gating, ExecutorEvent::GateFailed) => {
            if plan_state.iteration >= MAX_AUTO_FIX_ITERATIONS {
                PlanPhase::Failed {
                    reason: FailureKind::AutoFixExhausted,
                }
            } else {
                PlanPhase::AutoFixing
            }
        }

        // -- Merging (with bounded retry) --
        (PhaseKind::Merging, ExecutorEvent::MergeSucceeded) => PlanPhase::Complete,
        (PhaseKind::Merging, ExecutorEvent::MergeFailed) => {
            if plan_state.merge_attempts >= MAX_MERGE_ATTEMPTS {
                PlanPhase::Failed {
                    reason: FailureKind::Deadlock,
                }
            } else {
                PlanPhase::Failed {
                    reason: FailureKind::Other("merge conflict -- retry".into()),
                }
            }
        }

        // -- Fatal from any non-terminal phase --
        (kind, ExecutorEvent::Fatal(reason)) => {
            let target = PhaseKind::Failed;
            if valid_transitions(kind).contains(&target) {
                PlanPhase::Failed {
                    reason: FailureKind::Other(reason.clone()),
                }
            } else {
                return Err(TransitionError {
                    from: kind,
                    to: target,
                    reason: format!("cannot fail from {kind:?}"),
                });
            }
        }
        // ... remaining transitions (Skip from every non-terminal phase) ...
    };

    // Validate against the canonical transition table.
    let next_kind = next.kind();
    if !valid_transitions(current_kind).contains(&next_kind) {
        return Err(TransitionError {
            from: current_kind,
            to: next_kind,
            reason: format!(
                "transition {current_kind:?} -> {next_kind:?} not in valid_transitions table",
            ),
        });
    }
    Ok(next)
}
```

*Source: `crates/roko-orchestrator/src/executor/state_machine.rs`, lines 110-224*

### The Action Vocabulary

After each transition, the state machine suggests the next action via `PlanStateMachine::next_action()`. Actions are the vocabulary of side-effects the executor can request -- the executor itself never performs I/O.

```rust
// Source: crates/roko-orchestrator/src/executor/action.rs, lines 16-117

pub enum ExecutorAction {
    /// Begin executing a plan that was queued.
    DispatchPlan { plan_id: String },

    /// Spawn an agent process for a specific task within a plan.
    SpawnAgent { plan_id: String, role: AgentRole, task: String },

    /// Run a verification gate (compile, test, clippy, etc.) at a given rung.
    RunGate { plan_id: String, rung: u32 },

    /// Run task-level verification commands declared in tasks.toml.
    RunVerify { plan_id: String },

    /// Apply a DAG mutation between execution boundaries.
    ApplyDagMutation { mutation: DagMutation },

    /// Launch a backup execution for a slow task.
    StartSpeculativeExecution {
        plan_id: String, task: String, backup_role: AgentRole,
        expected_minutes: u32, elapsed_minutes: u32,
    },

    /// Cancel the losing branch of a speculative execution.
    CancelSpeculativeExecution { plan_id: String, task: String },

    /// Merge a plan's worktree branch into the batch branch.
    MergeBranch { plan_id: String },

    /// Mark a plan as terminally failed.
    FailPlan { plan_id: String, reason: String },

    /// Mark a plan as successfully completed.
    CompletePlan { plan_id: String },

    /// Move a plan to a different position in the execution queue.
    Reorder { plan_id: String, new_position: usize },

    /// Pause a running plan (e.g. due to resource contention).
    PausePlan { plan_id: String },

    /// Resume a previously paused plan.
    ResumePlan { plan_id: String },
}
```

*Source: `crates/roko-orchestrator/src/executor/action.rs`, lines 16-117*

Each phase maps to a specific agent role. The `next_action()` function encodes this mapping:

| Phase | Action | Agent Role |
|-------|--------|------------|
| Queued | `DispatchPlan` | -- |
| Enriching | `SpawnAgent` | Strategist |
| Implementing | `SpawnAgent` | Implementer |
| Gating | `RunGate` | -- (gate pipeline) |
| AutoFixing | `SpawnAgent` | AutoFixer |
| Verifying | `RunVerify` | -- (verify pipeline) |
| RegeneratingVerify | `SpawnAgent` | AutoFixer |
| Reviewing | `SpawnAgent` | Auditor |
| DocRevision | `SpawnAgent` | Scribe |
| Merging | `MergeBranch` | -- (git) |

*Source: `crates/roko-orchestrator/src/executor/state_machine.rs`, lines 231-283*

### PlanState: Per-Plan Mutable State

Every plan in the executor gets a `PlanState` struct that tracks everything the state machine needs to make scheduling decisions:

```rust
// Source: crates/roko-orchestrator/src/executor/plan_state.rs, lines 19-43

pub struct PlanState {
    pub plan_id: String,
    pub current_phase: PlanPhase,
    pub assigned_agents: Vec<String>,
    pub gate_results: Vec<GateResult>,
    pub iteration: u32,           // starts at 1, bumps on retry
    pub started_at_ms: u64,
    pub files_changed: Vec<String>, // for conflict detection
    pub merge_attempts: u32,
    pub last_error: Option<String>,
    pub paused: bool,
    pub priority: u32,            // higher runs first
}
```

*Source: `crates/roko-orchestrator/src/executor/plan_state.rs`, lines 19-43*

The `files_changed` field is critical for the file-conflict inference algorithm (section 7). As agents modify files, they report which files they touched. The orchestrator uses this to detect when two plans would conflict during merge.

### The ParallelExecutor: Putting It All Together

The `ParallelExecutor` is the main orchestration engine. It holds a queue of plans keyed by `plan_id`, tracks cross-plan dependencies, manages speculative executions, and optionally attaches an audit chain.

```rust
// Source: crates/roko-orchestrator/src/executor/mod.rs, lines 241-254

pub struct ParallelExecutor {
    config: ExecutorConfig,
    plans: HashMap<String, PlanState>,
    queue: Vec<String>,                              // plan_ids in priority order
    plan_deps: HashMap<String, Vec<String>>,         // cross-plan dependency graph
    speculative_executions: HashMap<String, SpeculativeExecution>,
    audit_chain: Option<AuditChain>,
}
```

*Source: `crates/roko-orchestrator/src/executor/mod.rs`, lines 241-254*

The `tick()` method is the core loop iteration. On each tick, it walks the queue in priority order, skips terminal/paused/dependency-blocked plans, respects the concurrency limit (`max_concurrent_plans`), and asks the state machine what action each active plan needs:

```rust
// Source: crates/roko-orchestrator/src/executor/mod.rs, lines 464-501

pub fn tick(&self) -> Vec<ExecutorAction> {
    let mut actions = Vec::new();
    let mut active_count = 0;

    for plan_id in &self.queue {
        let Some(state) = self.plans.get(plan_id) else { continue };
        if state.is_terminal() { continue }
        if state.paused { continue }
        if !self.deps_satisfied(plan_id) { continue }

        active_count += 1;
        if active_count > self.config.max_concurrent_plans { break }

        if let Some(action) = PlanStateMachine::next_action(state) {
            actions.push(action);
        }
    }
    actions
}
```

*Source: `crates/roko-orchestrator/src/executor/mod.rs`, lines 464-501*

The `apply_event()` method is the event ingestion path. It validates the transition, updates plan state, records failures, and appends to the audit chain if attached:

```rust
// Source: crates/roko-orchestrator/src/executor/mod.rs, lines 509-542

pub fn apply_event(
    &mut self,
    plan_id: &str,
    event: &ExecutorEvent,
) -> Result<PlanPhase, TransitionError> {
    let state = self.plans.get(plan_id).ok_or_else(|| TransitionError {
        from: PhaseKind::Queued,
        to: PhaseKind::Failed,
        reason: format!("plan '{plan_id}' not found"),
    })?;
    let from_kind = state.current_phase.kind();
    let new_phase = PlanStateMachine::transition(state, event)?;
    let to_kind = new_phase.kind();

    if let Some(state) = self.plans.get_mut(plan_id) {
        state.current_phase = new_phase.clone();
        if let PlanPhase::Failed { reason } = &new_phase {
            state.last_error = Some(reason.to_string());
        }
    }

    // Append to audit chain if attached.
    if let Some(chain) = &self.audit_chain {
        let kind = format!("phase.{from_kind:?}->{to_kind:?}");
        let entry = AuditEntry::new([0u8; 32], kind, "executor", plan_id.to_string());
        let _ = chain.append(entry);
    }

    Ok(new_phase)
}
```

*Source: `crates/roko-orchestrator/src/executor/mod.rs`, lines 509-542*

### Configuration

The `ExecutorConfig` controls all tuning knobs:

```rust
// Source: crates/roko-orchestrator/src/executor/mod.rs, lines 149-180

pub struct ExecutorConfig {
    pub max_concurrent_plans: usize,           // default: 4
    pub max_concurrent_tasks: usize,           // default: 8
    pub max_auto_fix_iterations: u32,          // default: 5
    pub max_merge_attempts: u32,               // default: 3
    pub task_timeout_secs: u64,                // default: 600
    pub budget_usd: Option<f64>,               // deprecated, use resource_budget
    pub resource_budget: ResourceBudget,       // composite budget (ORCH-08)
    pub speculative_threshold_multiplier: f64, // default: 2.0
    pub auto_replan: bool,                     // default: false
    pub use_worktrees: bool,                   // default: false
}
```

*Source: `crates/roko-orchestrator/src/executor/mod.rs`, lines 149-180*

### Speculative Execution

When a task takes longer than `expected_minutes * speculative_threshold_multiplier`, the executor can register a speculative execution -- a backup agent working on the same task in parallel. Whichever finishes first wins; the other is cancelled.

```rust
// Source: crates/roko-orchestrator/src/executor/mod.rs, lines 64-80

pub struct SpeculativeExecution {
    pub plan_id: String,
    pub task: String,
    pub expected_minutes: u32,
    pub elapsed_minutes: u32,
    pub backup_role: AgentRole,
    pub projected_cost_usd: f64,
    pub started_at_ms: u64,
}
```

The executor checks budget constraints before starting speculation (`projected_cost_usd` must fit within `budget_usd`), and deduplicates by `plan_id:task` key to prevent multiple speculative branches for the same task.

*Source: `crates/roko-orchestrator/src/executor/mod.rs`, lines 357-423*

---

## 4. Event Sourcing: Journal, Replay, and Recovery

### What Is Event Sourcing?

Event sourcing is a pattern where the state of an application is determined by a sequence of events rather than by storing the current state directly [Fowler2005]. Instead of mutating a record in place, every change is captured as an immutable event and appended to a log. The current state is derived by replaying the full event sequence from the beginning.

This pattern was first described by Martin Fowler in 2005 and later elaborated as part of the Command Query Responsibility Segregation (CQRS) architecture [Young2010]. The core insight is that an append-only event log gives you three things for free: (1) a complete audit trail, (2) the ability to reconstruct state at any point in time, and (3) crash recovery by replaying events up to the failure point. Temporal.io's durable execution engine [Temporal2024] uses the same principle: "every step of a workflow is persisted as an event in an Event History, and if a worker process crashes halfway through, Temporal replays the event history on a new worker and resumes from exactly where it left off."

### The Event Log

The `EventLog` is an append-only, BLAKE3 hash-chained journal of every significant orchestration event. It serves three purposes:

1. **State reconstruction**: Replay the full event sequence to rebuild orchestrator state from scratch.
2. **Crash recovery**: After a crash, replay events since the last snapshot to advance the snapshot to the crash point.
3. **Tamper detection**: Each entry's hash includes the previous entry's hash. Any mutation, deletion, insertion, or reordering of historical entries is detectable by walking the chain and recomputing hashes.

```rust
// Source: crates/roko-orchestrator/src/event_log.rs, lines 28-53

pub enum EventKind {
    PlanStarted,
    TaskAssigned,
    AgentSpawned,
    CognitiveWorkspaceRecorded,
    GateResult,
    MergeAttempted,
    PlanCompleted,
    PlanFailed,
    ErrorOccurred,
    InterventionFired,
    PhaseTransition,
    EnrichmentValidated,
}
```

*Source: `crates/roko-orchestrator/src/event_log.rs`, lines 28-53*

### Event Entry Structure

Each event entry carries:

```rust
// Source: crates/roko-orchestrator/src/event_log.rs, lines 79-90

pub struct EventEntry {
    /// Monotonically increasing sequence number (0-based).
    pub sequence_number: u64,
    /// Unix millisecond timestamp of when the event was recorded.
    pub timestamp_ms: i64,
    /// The kind of event.
    pub event_kind: EventKind,
    /// Structured payload (event-specific data).
    pub payload: serde_json::Value,
    /// BLAKE3 content hash of this entry (includes the previous hash).
    pub content_hash: [u8; 32],
}
```

*Source: `crates/roko-orchestrator/src/event_log.rs`, lines 79-90*

### Hash Chain Construction

The hash for each entry is computed over a deterministic canonical encoding that includes the previous entry's hash. This makes the chain tamper-evident -- changing any historical entry invalidates every subsequent hash.

The construction follows the same principle as Merkle hash trees [Merkle1988] and certificate transparency logs [Laurie2013], adapted for a linear chain rather than a tree. Each entry's hash is:

```
H(entry_n) = BLAKE3("eventv1|" || seq_be || ts_be || H(entry_{n-1}) || LP(kind) || LP(payload))
```

where `LP(data)` means a 4-byte big-endian length prefix followed by the data, and `||` denotes concatenation.

```rust
// Source: crates/roko-orchestrator/src/event_log.rs, lines 95-118

fn compute_hash(
    seq: u64,
    ts_ms: i64,
    kind: &EventKind,
    payload: &serde_json::Value,
    prev_hash: &[u8; 32],
) -> [u8; 32] {
    let kind_str = kind.to_string();
    let payload_bytes = serde_json::to_vec(payload).unwrap_or_default();

    let mut buf: Vec<u8> = Vec::with_capacity(...);
    buf.extend_from_slice(b"eventv1|");        // version tag
    buf.extend_from_slice(&seq.to_be_bytes()); // sequence number
    buf.extend_from_slice(&ts_ms.to_be_bytes()); // timestamp
    buf.extend_from_slice(prev_hash);          // link to previous entry
    push_lp(&mut buf, kind_str.as_bytes());    // length-prefixed kind
    push_lp(&mut buf, &payload_bytes);         // length-prefixed payload
    ContentHash::of(&buf).0                    // BLAKE3 hash
}
```

*Source: `crates/roko-orchestrator/src/event_log.rs`, lines 95-118*

The genesis entry (sequence 0) uses `[0u8; 32]` as its `prev_hash`.

### Integrity Verification

The `verify_integrity()` method walks the chain from genesis and recomputes every hash. If any entry's stored hash does not match the recomputation, or if the stored tip hash does not match the final entry, it returns an `IntegrityError` identifying the broken link:

```rust
// Source: crates/roko-orchestrator/src/event_log.rs, lines 284-319

pub fn verify_integrity(&self) -> Result<(), IntegrityError> {
    let guard = self.inner.lock();
    verify_entry_sequences(&guard.entries)?;  // check monotonic seq numbers
    let mut prev_hash = ZERO_HASH;

    for entry in &guard.entries {
        let expected = EventEntry::compute_hash(
            entry.sequence_number, entry.timestamp_ms,
            &entry.event_kind, &entry.payload, &prev_hash,
        );
        if entry.content_hash != expected {
            return Err(IntegrityError {
                at_sequence: entry.sequence_number,
                reason: format!("hash mismatch: expected {}, got {}", ...),
            });
        }
        prev_hash = entry.content_hash;
    }

    if guard.tip != prev_hash {
        return Err(IntegrityError {
            at_sequence: guard.entries.len() as u64,
            reason: "tip hash does not match final entry".into(),
        });
    }
    Ok(())
}
```

*Source: `crates/roko-orchestrator/src/event_log.rs`, lines 284-319*

Thread safety is handled via `Arc<Mutex<LogInner>>` (using `parking_lot::Mutex`), and concurrent appends from multiple threads preserve integrity (tested with 4 threads x 25 events each).

*Source: `crates/roko-orchestrator/src/event_log.rs`, lines 617-635 (test: `concurrent_appends_preserve_integrity`)*

### Snapshot and Restore

The event log supports snapshot/restore for crash recovery:

```rust
// Source: crates/roko-orchestrator/src/event_log.rs, lines 322-350

pub fn snapshot(&self) -> EventLogSnapshot {
    let guard = self.inner.lock();
    EventLogSnapshot {
        entries: guard.entries.clone(),
        tip: guard.tip,
    }
}

pub fn restore_verified(snapshot: EventLogSnapshot) -> Result<Self, IntegrityError> {
    let log = Self::restore(snapshot);
    log.verify_integrity()?;
    Ok(log)
}
```

*Source: `crates/roko-orchestrator/src/event_log.rs`, lines 322-350*

---

## 5. Unified Cross-Plan Task DAG

### What It Is

The `UnifiedTaskDag` merges tasks from multiple plans into a single directed acyclic graph. This is crucial for multi-plan orchestration because it enables cross-plan dependency tracking, global scheduling optimization, and file-conflict inference across plan boundaries.

DAG-based task scheduling is a well-studied problem in parallel computing [Kwok1999]. The critical path method (CPM) was originally developed for project management by Kelley and Walker in 1959 and has since been adapted for parallel processor scheduling, where the critical path length determines the minimum wall-clock time regardless of available parallelism.

### DAG Construction

The `build()` method takes three inputs:

1. `plan_tasks: &BTreeMap<String, Vec<Task>>` -- tasks grouped by plan ID
2. `plan_deps: &HashMap<String, HashSet<String>>` -- plan-level dependencies
3. `config: DagConfig` -- configuration (file overlap inference, max wave width)

It constructs edges from four sources:

```
1. Intra-plan depends_on:      t1 -> t2 inside one plan
2. Cross-plan depends_on:      "09-foo:t3" inside plan "10-bar"
3. Plan-level depends_on:      Plan B depends on Plan A
                                => every task in A -> every task in B
4. File-overlap inference:      t1 touches src/lib.rs, t2 touches src/lib.rs
                                => t1 -> t2 (lexicographic GlobalTaskId order)
```

*Source: `crates/roko-orchestrator/src/dag.rs`, lines 1-20 (module doc)*

### GlobalTaskId

Tasks are identified globally by a `GlobalTaskId`, which is a `(plan, task)` pair. This allows unambiguous cross-plan references:

```
plan: "09-chain-layer", task: "T1" => GlobalTaskId("09-chain-layer", "T1")
```

Cross-plan references use the qualified format `"09-chain-layer:T1"` in `depends_on` fields.

### Cycle Detection

Before returning, `build()` runs topological sort to eagerly reject cyclic dependencies. The `detect_cycle_nodes()` function uses a DFS-based algorithm with 3-state coloring (unvisited/in-progress/complete) that identifies every node participating in a cycle, returning them sorted for deterministic diagnostics:

```rust
// Source: crates/roko-orchestrator/src/dag.rs, lines 33-92

pub fn detect_cycle_nodes<N>(deps: &BTreeMap<N, BTreeSet<N>>) -> Vec<N>
where
    N: Clone + Ord,
{
    // DFS with 3-state coloring:
    //   0 = unvisited, 1 = in progress, 2 = complete
    // When we encounter a node still in state 1, all nodes
    // on the stack from that point forward form a cycle.
    // ...
}
```

*Source: `crates/roko-orchestrator/src/dag.rs`, lines 33-92*

### Critical Path Method (CPM) Analysis

The DAG supports CPM analysis for scheduling optimization. Each task has an `estimated_minutes` field. The DAG computes:

- **Earliest start time**: The longest path from any root to this task.
- **Latest start time**: The latest this task can start without extending the overall plan.
- **Critical path**: The longest path through the DAG (the minimum possible wall-clock time).

```rust
// Source: crates/roko-orchestrator/src/dag.rs

pub fn earliest_start(&self, task: &GlobalTaskId) -> Duration { ... }
pub fn latest_start(&self, task: &GlobalTaskId) -> Duration { ... }
pub fn stats(&self) -> DagStats {
    DagStats {
        nodes: self.nodes.len(),
        edges: edge_count,
        waves: wave_count,
        critical_path_minutes: critical, // longest path by estimated_minutes
    }
}
```

*Source: `crates/roko-orchestrator/src/dag.rs`, lines 444-636*

---

## 6. Wave Scheduling Algorithm

### Overview

Wave scheduling layers the DAG into groups of tasks that can execute in parallel. This is a variant of the classical list scheduling approach for DAG-based parallel task allocation [Kwok1999]. Wave 0 contains tasks with no dependencies. Wave 1 contains tasks whose dependencies are all in wave 0. Wave N contains tasks whose dependencies are all in waves 0 through N-1.

### Algorithm Step by Step

1. **Topological sort**: Confirm the DAG is acyclic. Obtain a topological ordering.

2. **Compute depths**: Walk nodes in topological order. Each node's depth is `max(depth_of_deps) + 1`. Roots get depth 0.

3. **Bucket by depth**: Group tasks into buckets by their computed depth.

4. **Apply wave width limit**: If `max_wave_width > 0`, split any bucket that exceeds the limit. Overflow tasks spill into the next wave.

5. **Sort within wave**: Tasks within each wave are sorted by `GlobalTaskId` (lexicographic on `(plan, task)`) for deterministic output.

6. **Estimate wave duration**: Each wave's `estimated_minutes` is the max across all tasks in the wave (wall-clock estimate assumes parallel execution).

```rust
// Source: crates/roko-orchestrator/src/dag.rs, lines 546-620

pub fn waves(&self) -> Result<Vec<ExecutionWave>, DagError> {
    let topo = self.topological_sort()?;

    // Step 2: Compute depths
    let mut depth: HashMap<GlobalTaskId, usize> = HashMap::with_capacity(self.nodes.len());
    for node in &topo {
        let d = self.edges.get(node)
            .into_iter().flatten()
            .map(|dep| depth.get(dep).copied().unwrap_or(0) + 1)
            .max()
            .unwrap_or(0);
        depth.insert(node.clone(), d);
    }

    // Step 3: Bucket by depth
    let mut by_depth: BTreeMap<usize, BTreeSet<GlobalTaskId>> = BTreeMap::new();
    for (id, d) in &depth {
        by_depth.entry(*d).or_default().insert(id.clone());
    }

    // Steps 4-6: Apply width limit, sort, estimate
    let max_width = self.config.max_wave_width;
    let mut waves: Vec<ExecutionWave> = Vec::new();
    let mut overflow: Vec<GlobalTaskId> = Vec::new();

    for (_, bucket) in by_depth {
        let mut combined: Vec<GlobalTaskId> = std::mem::take(&mut overflow);
        combined.extend(bucket);
        combined.sort_by(|a, b|
            (a.plan.as_str(), a.task.as_str()).cmp(&(b.plan.as_str(), b.task.as_str()))
        );

        while !combined.is_empty() {
            let take = if max_width == 0 { combined.len() }
                       else { combined.len().min(max_width) };
            let batch: Vec<GlobalTaskId> = combined.drain(..take).collect();
            let est = batch.iter()
                .map(|id| self.estimates.get(id).copied().unwrap_or(0))
                .max().unwrap_or(0);
            waves.push(ExecutionWave { index: waves.len(), tasks: batch, estimated_minutes: est });

            if max_width > 0 && !combined.is_empty() {
                overflow.append(&mut combined);
                break;
            }
        }
    }
    // Drain final overflow...
    Ok(waves)
}
```

*Source: `crates/roko-orchestrator/src/dag.rs`, lines 546-620*

### Concrete Example

Given tasks with dependencies:

```
t1 (no deps)           t2 (no deps)        t3 (no deps)
     |                      |
     v                      v
t4 (depends on t1)     t5 (depends on t2)
     |                      |
     +----------+-----------+
                |
                v
          t6 (depends on t4, t5)
```

Wave scheduling produces:

```
Wave 0: [t1, t2, t3]   -- all independent, run in parallel
Wave 1: [t4, t5]        -- depend only on wave 0
Wave 2: [t6]            -- depends on wave 1
```

If `max_wave_width = 2`, wave 0 splits:

```
Wave 0: [t1, t2]        -- width-limited to 2
Wave 1: [t3, t4]        -- t3 overflowed from wave 0, t4's dep (t1) is in wave 0
Wave 2: [t5]            -- t5's dep (t2) is in wave 0
Wave 3: [t6]            -- deps (t4, t5) are in waves 1 and 2
```

### Execution Status per Task

Each task in the DAG tracks a fine-grained execution status:

```rust
// Source: crates/roko-orchestrator/src/dag.rs, lines 142-191

pub enum DagTaskExecutionStatus {
    Pending,       // waiting for dependencies
    Ready,         // all deps passed, can be dispatched
    Running,       // agent currently working
    Gating,        // gate/verify checks running
    Passed,        // completed successfully
    Retrying {     // backoff before next attempt
        attempt: u32,
        backoff_until_ms: u64,
    },
    Exhausted {    // retry budget exceeded
        attempts: u32,
        last_error: String,
    },
    Skipped,       // intentionally skipped
}
```

*Source: `crates/roko-orchestrator/src/dag.rs`, lines 142-191*

---

## 7. File-Conflict Inference

### The Problem

When two tasks run in parallel and both modify the same file, the merge step will produce conflicts. Rather than discovering this at merge time (expensive -- the agent has already done all its work), the orchestrator infers likely file conflicts *before* scheduling and serializes conflicting tasks.

### The Algorithm

File-conflict inference is opt-in via `DagConfig::infer_file_overlap` (default: `true`). During DAG construction (in `rebuild_indexes()`), the algorithm:

1. **Build a file-to-tasks index**: For each file mentioned in any task's `files` field, collect the set of tasks that touch it.

2. **Add serialization edges**: For each file touched by more than one task, create edges between all pairs. The task with the lexicographically earlier `GlobalTaskId` becomes the dependency (runs first).

```rust
// Source: crates/roko-orchestrator/src/dag.rs, lines 1113-1133

if self.config.infer_file_overlap {
    let mut by_file: HashMap<String, BTreeSet<GlobalTaskId>> = HashMap::new();
    for (id, task) in &self.tasks {
        for file in &task.files {
            by_file.entry(file.clone()).or_default().insert(id.clone());
        }
    }
    for tasks in by_file.into_values() {
        let ordered: Vec<_> = tasks.into_iter().collect();
        for i in 0..ordered.len() {
            for j in 0..i {
                if ordered[i] != ordered[j] {
                    self.edges
                        .entry(ordered[i].clone())
                        .or_default()
                        .insert(ordered[j].clone());
                }
            }
        }
    }
}
```

*Source: `crates/roko-orchestrator/src/dag.rs`, lines 1113-1133*

### Concrete Example

```
task_a: files = ["src/auth.rs", "src/config.rs"]
task_b: files = ["src/api.rs", "src/config.rs"]
task_c: files = ["src/api.rs", "src/routes.rs"]
```

File-to-tasks index:
```
src/auth.rs   -> {task_a}
src/config.rs -> {task_a, task_b}     <- conflict!
src/api.rs    -> {task_b, task_c}     <- conflict!
src/routes.rs -> {task_c}
```

Serialization edges added:
```
task_b -> task_a  (src/config.rs conflict, task_a < task_b lexicographically)
task_c -> task_b  (src/api.rs conflict, task_b < task_c lexicographically)
```

Result: task_a runs first, then task_b, then task_c (fully serialized due to transitive dependency). Without file-conflict inference, all three would run in wave 0 and produce merge conflicts.

### Disabling Inference

When inference is disabled (`infer_file_overlap: false`), tasks are scheduled purely by explicit `depends_on` declarations. This is tested:

```rust
// Source: crates/roko-orchestrator/src/dag.rs (test)

fn file_overlap_can_be_disabled() {
    let cfg = DagConfig { infer_file_overlap: false, max_wave_width: 0 };
    let dag = UnifiedTaskDag::build(&plans, &HashMap::new(), cfg).unwrap();
    let waves = dag.waves().unwrap();
    assert_eq!(waves.len(), 1, "with inference off they run in parallel");
    assert_eq!(waves[0].tasks.len(), 2);
}
```

*Source: `crates/roko-orchestrator/src/dag.rs`, test section*

---

## 8. Three-Level Recovery System

### Overview

The orchestrator has three distinct levels of failure recovery, each progressively more aggressive. These are implemented across the `recovery.rs`, `replan.rs`, and `plan_state.rs` modules.

### Level 1: Task Retry (Within Iteration)

When a single task fails (gate failure, agent error), the simplest response is to retry it. The `PlanState` tracks the current `iteration` count, starting at 1. On each retry, `reset_for_retry()` increments the iteration and clears gate results:

```rust
// Source: crates/roko-orchestrator/src/executor/plan_state.rs, lines 246-250

pub fn reset_for_retry(&mut self) {
    self.gate_results.clear();
    self.iteration += 1;
    self.last_error = None;
}
```

*Source: `crates/roko-orchestrator/src/executor/plan_state.rs`, lines 246-250*

The state machine enforces a ceiling: when `iteration >= MAX_AUTO_FIX_ITERATIONS`, a gate failure transitions to `PlanPhase::Failed { reason: FailureKind::AutoFixExhausted }` instead of `PlanPhase::AutoFixing`.

**Example**: A plan is Implementing. The agent finishes. The plan enters Gating. The compile gate fails. The plan enters AutoFixing. An AutoFixer agent runs and makes corrections. The plan re-enters Gating. This loop repeats up to `MAX_AUTO_FIX_ITERATIONS` times.

The gate results also support "mostly passing" detection. If more than 90% of tests pass (with at least 20 total and at least 1 failure), the system classifies this as a targeted test failure rather than a broad problem, enabling more focused retry strategies:

```rust
// Source: crates/roko-orchestrator/src/executor/plan_state.rs, lines 144-168

pub fn is_mostly_passing(results: &[Self]) -> bool {
    // ... counts passed/failed/ignored across all gate results ...
    saw_failed_test_gate
        && total > 20
        && failed > 0
        && f64::from(passed) / f64::from(total) > 0.9
}
```

*Source: `crates/roko-orchestrator/src/executor/plan_state.rs`, lines 144-168*

### Level 2: Re-Planning (Structural Repair)

When retries are exhausted or the failure pattern indicates a structural problem (wrong task decomposition, missing dependency), the orchestrator escalates to re-planning. The `replan.rs` module defines five strategies:

```rust
// Source: crates/roko-orchestrator/src/replan.rs, lines 183-196

pub enum ReplanStrategy {
    /// Retry the same task with the current model and context.
    RetrySame,
    /// Retry the same task after upgrading to a stronger model.
    RetryWithEscalation,
    /// Split the failed task into smaller subtasks before retrying.
    Decompose,
    /// Mark the task skipped and continue with the rest of the plan.
    Skip,
    /// Rebuild the plan from scratch and restart execution.
    RegeneratePlan,
}
```

*Source: `crates/roko-orchestrator/src/replan.rs`, lines 183-196*

The `FailureDisposition` enum classifies failures into four categories that drive strategy selection:

```rust
// Source: crates/roko-orchestrator/src/replan.rs, lines 7-17

pub enum FailureDisposition {
    Retry,          // retry or deterministic remediation first
    NeedsReplan,    // current plan is the wrong shape
    Blocked,        // external condition blocks progress
    NeedsHuman,     // human input required
}
```

*Source: `crates/roko-orchestrator/src/replan.rs`, lines 7-17*

### Structured Re-Plan Evidence

When escalating to a re-plan, the system constructs a `PlanRevisionRequest` with structured evidence so the planner agent has context:

```rust
// Source: crates/roko-orchestrator/src/replan.rs, lines 101-124

pub struct PlanRevisionRequest {
    pub request_id: String,           // "replan-{hash}" for deduplication
    pub plan_id: String,
    pub task_id: String,
    pub disposition: FailureDisposition,
    pub reason: String,               // e.g. "gate_failure_limit"
    pub attempts: u32,
    pub evidence: Vec<PlanRevisionEvidence>,
    pub failure_pattern_ids: Vec<String>,   // e.g. ["E0425::src/lib.rs"]
    pub blocking_findings: Vec<String>,
    pub resume_token: String,         // BLAKE3 hash for dedupe
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

*Source: `crates/roko-orchestrator/src/replan.rs`, lines 101-124*

The `resume_token` is a BLAKE3 hash of the plan ID, task ID, reason, attempt count, and evidence -- ensuring that identical failure scenarios produce the same token for deduplication.

The `ReplanResult` enum captures structured outcomes that the executor can act on:

```rust
// Source: crates/roko-orchestrator/src/replan.rs, lines 219-263

pub enum ReplanResult {
    RetrySame { plan_id, task_id },
    RetryWithEscalation { plan_id, task_id, escalated_model },
    Decompose { plan_id, task_id, new_task_ids },
    RegeneratePlan { plan_id, task_id, new_task_ids },
    Skip { plan_id, task_id },
}
```

*Source: `crates/roko-orchestrator/src/replan.rs`, lines 219-263*

Only `Decompose` and `RegeneratePlan` require a plan restart (re-queuing from `Queued`). The others can be handled within the current iteration.

### Level 3: Full Crash Recovery (Snapshot + Event Replay)

When the orchestrator process crashes entirely, the `RecoveryEngine` reconstructs state from two complementary sources:

1. **Executor Snapshot** (`executor.json`) -- a point-in-time capture of every plan's mutable state plus the execution queue order.
2. **Event Log** -- the append-only hash-chained event journal.

```rust
// Source: crates/roko-orchestrator/src/executor/recovery.rs, lines 1-12

// After a crash, the orchestrator can recover its state from two sources:
//
// 1. Executor snapshot (executor.json) -- a point-in-time capture of
//    every plan's mutable state plus the execution queue order.
// 2. Event log -- an append-only, hash-chained sequence of orchestration
//    events that can be replayed to reconstruct state from scratch.
//
// The RecoveryEngine orchestrates the full recovery pipeline:
// deserialize the snapshot, replay the event log, merge both (event log
// wins on conflict), and validate the result for inconsistencies.
```

*Source: `crates/roko-orchestrator/src/executor/recovery.rs`, lines 1-12*

#### Recovery Pipeline

The `RecoveryEngine` follows a three-step pipeline:

**Step 1: Recover from snapshot** -- Deserialize `executor.json` into a `RecoveredState`.

**Step 2: Recover from event log** -- Replay events to reconstruct state. Each event is mapped to a plan state update, building the plan phase and queue order from scratch.

**Step 3: Merge** -- Event log state wins on conflict:

```rust
// Source: crates/roko-orchestrator/src/executor/recovery.rs, lines 384-407

pub fn merge_recovery(
    snapshot: Option<RecoveredState>,
    event_log: Option<RecoveredState>,
) -> RecoveredState {
    match (snapshot, event_log) {
        (None, None) => RecoveredState::empty(current_timestamp_ms()),
        (Some(s), None) => s,
        (None, Some(e)) => e,
        (Some(snap), Some(log)) => {
            let mut merged_plans = snap.plan_states;
            // Event-log state overwrites snapshot state for any plan
            // present in both.
            for (id, log_info) in log.plan_states {
                merged_plans.insert(id, log_info);
            }
            // Queue order: prefer event log if non-empty, otherwise snapshot.
            let queue_order = if log.queue_order.is_empty() {
                snap.queue_order
            } else {
                log.queue_order
            };
            // ...
        }
    }
}
```

*Source: `crates/roko-orchestrator/src/executor/recovery.rs`, lines 384-407*

#### Resume Directives

After recovery, each plan is classified into a resume directive:

```rust
// Source: crates/roko-orchestrator/src/executor/plan_state.rs, lines 64-85

pub enum PlanResumeDirective {
    ContinueActive,           // non-terminal, keep going
    TerminalComplete,          // completed successfully, do not requeue
    TerminalSkipped,           // skipped by policy, do not requeue
    RetryTerminalFailure {     // failed but can auto-retry
        failure: FailureKind,
        cooldown_secs: u64,
    },
    AwaitManualRepair {        // failed, needs human intervention
        failure: FailureKind,
    },
}
```

*Source: `crates/roko-orchestrator/src/executor/plan_state.rs`, lines 64-85*

The `RecoveryResumePlan` groups recovered plans into five buckets: `active`, `retryable_terminal`, `manual_repair`, `completed`, and `skipped`, plus any non-fatal `warnings` discovered during validation.

*Source: `crates/roko-orchestrator/src/executor/recovery.rs`, lines 127-142*

#### Recovery Warnings

The recovery engine validates recovered state for inconsistencies and surfaces non-fatal warnings with severity levels:

```rust
// Source: crates/roko-orchestrator/src/executor/recovery.rs, lines 51-70

pub enum WarningSeverity {
    Info,      // informational, recovery can proceed
    Warning,   // state may be slightly stale
    Critical,  // recovered state is likely incorrect, manual inspection needed
}

pub struct RecoveryWarning {
    pub plan_id: String,
    pub message: String,
    pub severity: WarningSeverity,
}
```

*Source: `crates/roko-orchestrator/src/executor/recovery.rs`, lines 51-70*

---

## 9. BLAKE3 Hash-Linked Audit Chain

### Purpose

The audit chain is a separate data structure from the event log. While the event log records orchestration events (plan started, task assigned, gate result), the audit chain records **privileged operations**: capability consumption, permit issuance, sandbox boundary crossings, loop-guard trips, and phase transitions. The two serve different purposes:

- **Event log**: State reconstruction and crash recovery.
- **Audit chain**: Security auditing and tamper detection for privileged operations.

BLAKE3 [OConnor2020] is a cryptographic hash function that is significantly faster than SHA-256 while maintaining equivalent security properties. Its tree-based internal structure allows for parallelized hashing, making it particularly suitable for high-throughput event logging where hash computation must not become a bottleneck.

### AuditEntry Structure

```rust
// Source: crates/roko-orchestrator/src/safety/audit_chain.rs, lines 37-54

pub struct AuditEntry {
    /// Hash of the preceding entry. Zeroed for the genesis entry.
    pub prev_hash: [u8; 32],
    /// Operation kind, e.g. "capability.issued" or "sandbox.violation".
    pub kind: String,
    /// Actor that triggered the operation (agent id, role, service).
    pub actor: String,
    /// Resource the operation targeted (worktree path, permit id, capability id).
    pub resource: String,
    /// Unix millisecond timestamp of when the entry was recorded.
    pub ts_ms: i64,
    /// Optional detached signature over the entry body.
    pub signature: Option<String>,
}
```

*Source: `crates/roko-orchestrator/src/safety/audit_chain.rs`, lines 37-54*

### Hash Computation

The entry hash is computed over a deterministic hand-rolled canonical encoding (not serde_json, to ensure stability across serde versions):

```rust
// Source: crates/roko-orchestrator/src/safety/audit_chain.rs, lines 91-114

pub fn content_hash(&self) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::with_capacity(...);
    buf.extend_from_slice(b"auditv1|");        // version tag
    buf.extend_from_slice(&self.prev_hash);    // chain link
    push_field(&mut buf, b"kind", self.kind.as_bytes());
    push_field(&mut buf, b"actor", self.actor.as_bytes());
    push_field(&mut buf, b"resource", self.resource.as_bytes());
    push_field(&mut buf, b"ts_ms", &self.ts_ms.to_be_bytes());
    match &self.signature {
        Some(sig) => push_field(&mut buf, b"sig+", sig.as_bytes()),
        None => push_field(&mut buf, b"sig-", b""),
    }
    ContentHash::of(&buf).0  // BLAKE3
}
```

*Source: `crates/roko-orchestrator/src/safety/audit_chain.rs`, lines 91-114*

The `push_field` helper uses 4-byte big-endian length prefixes to prevent field-body collisions:

```rust
fn push_field(buf: &mut Vec<u8>, tag: &[u8], body: &[u8]) {
    buf.push(b'|');
    buf.extend_from_slice(tag);
    buf.push(b'=');
    let len = u32::try_from(body.len()).unwrap_or(u32::MAX);
    buf.extend_from_slice(&len.to_be_bytes());
    buf.extend_from_slice(body);
}
```

*Source: `crates/roko-orchestrator/src/safety/audit_chain.rs`, lines 117-125*

### AuditChain: Append-Only Container

The `AuditChain` is thread-safe (`Arc<Mutex<ChainInner>>`) and provides three key operations:

1. **`append(entry)`**: Automatically wires the `prev_hash` from the current tip, computes the content hash, and stores the entry.
2. **`verify()`**: Walks the chain from genesis, recomputing hashes. Returns `false` if any entry's `prev_hash` does not match the recomputed hash of its predecessor.
3. **`tip()`**: Returns the hash of the most recent entry.

```rust
// Source: crates/roko-orchestrator/src/safety/audit_chain.rs, lines 136-144

pub struct AuditChain {
    inner: Arc<Mutex<ChainInner>>,
}

struct ChainInner {
    entries: Vec<AuditEntry>,
    tip: [u8; 32],
}
```

*Source: `crates/roko-orchestrator/src/safety/audit_chain.rs`, lines 136-144*

### Snapshot Verification Envelope

For snapshot files, the `SnapshotVerifier` wraps serialized JSON in a binary envelope with BLAKE3 integrity:

```
+------+--------+---------+--------+---------+
| ROKO | Length | Payload | BLAKE3 |  END!   |
| 4B   |  8B    | N bytes | 32B    |   4B    |
+------+--------+---------+--------+---------+
```

```rust
// Source: crates/roko-orchestrator/src/executor/snapshot.rs, lines 472-660

const MAGIC: &[u8; 4] = b"ROKO";
const TRAILER: &[u8; 4] = b"END!";

pub struct SnapshotVerifier;

impl SnapshotVerifier {
    pub fn compute_hash(data: &[u8]) -> [u8; 32] {
        *blake3::hash(data).as_bytes()
    }

    pub fn save_verified(snapshot: &ExecutorSnapshot) -> Result<Vec<u8>, ...> {
        let payload = serde_json::to_vec(snapshot)?;
        let hash = Self::compute_hash(&payload);
        let len = payload.len() as u64;
        let mut buf = Vec::with_capacity(4 + 8 + payload.len() + 32 + 4);
        buf.extend_from_slice(MAGIC);          // "ROKO"
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(&payload);
        buf.extend_from_slice(&hash);
        buf.extend_from_slice(TRAILER);        // "END!"
        Ok(buf)
    }

    pub fn load_verified(data: &[u8]) -> Result<ExecutorSnapshot, SnapshotIntegrityError> {
        // Check magic, length, BLAKE3 hash, and trailer before deserializing.
        if &data[..4] != MAGIC { return Err(BadMagic) }
        if &data[data.len() - 4..] != TRAILER { return Err(BadTrailer) }
        // ... length check, hash verification, deserialize ...
    }
}
```

*Source: `crates/roko-orchestrator/src/executor/snapshot.rs`, lines 472-660*

### Delta Snapshots

To reduce write amplification, the system supports delta snapshots (ORCH-03). A `DeltaSnapshot` records only which plan IDs were added, removed, or changed between two full snapshots:

```rust
// Source: crates/roko-orchestrator/src/executor/snapshot.rs, lines 430-446

pub struct DeltaSnapshot {
    pub base_hash: [u8; 32],        // BLAKE3 hash of the base snapshot
    pub expected_hash: [u8; 32],    // BLAKE3 hash of the expected result
    pub changed: serde_json::Value, // only changed top-level fields
    pub removed_plan_ids: Vec<String>,
    pub added_plan_ids: Vec<String>,
    pub sequence: u64,              // position in delta chain
}
```

*Source: `crates/roko-orchestrator/src/executor/snapshot.rs`, lines 430-446*

---

## 10. Live DAG Mutation

### The Problem

Plans are not static. During execution, an agent might discover that a task needs to be split into subtasks, or that a new dependency exists that was not visible during planning, or that a task has become unnecessary. The DAG must support mutations while execution is in progress.

### DagMutation Enum

Five mutation operations are supported:

```rust
// Source: crates/roko-orchestrator/src/dag.rs, lines 296-336

pub enum DagMutation {
    /// Add a new task to the DAG.
    AddTask {
        task_id: GlobalTaskId,
        task: Task,
        depends_on: Vec<GlobalTaskId>,
    },

    /// Remove a task from the DAG and reconnect its dependents to its deps.
    RemoveTask {
        task_id: GlobalTaskId,
    },

    /// Replace one task with a serial chain of subtasks.
    SplitTask {
        task_id: GlobalTaskId,
        into: Vec<Task>,  // replacement tasks in execution order
    },

    /// Add an additional dependency edge from -> to.
    AddDependency {
        from: GlobalTaskId,  // the task that should wait
        to: GlobalTaskId,    // the task that must complete first
    },

    /// Replace an existing task spec wholesale.
    UpdateTaskMetadata {
        task_id: GlobalTaskId,
        task: Task,
    },
}
```

*Source: `crates/roko-orchestrator/src/dag.rs`, lines 296-336*

### Safety Checks

Mutations are validated before application:

- `UnknownTask`: The target task does not exist.
- `CompletedTask`: The task already completed and must not be mutated.
- `InvalidMutation`: The mutation payload is structurally invalid.
- `Cycle`: The mutation introduced a cycle.

```rust
// Source: crates/roko-orchestrator/src/dag.rs, lines 275-291

pub enum DagMutationError {
    #[error("unknown task: {0}")]
    UnknownTask(GlobalTaskId),
    #[error("completed task cannot be mutated: {0}")]
    CompletedTask(GlobalTaskId),
    #[error("invalid DAG mutation: {0}")]
    InvalidMutation(String),
    #[error("mutation introduced a cycle involving: {0:?}")]
    Cycle(Vec<GlobalTaskId>),
}
```

*Source: `crates/roko-orchestrator/src/dag.rs`, lines 275-291*

### Integration with Executor

DAG mutations flow through the executor as `ExecutorAction::ApplyDagMutation`. The runtime harness applies the mutation between execution boundaries (never mid-task) and feeds the result back. This ensures that the DAG is always in a consistent state when tasks are dispatched.

### DAG Culling

The DAG also supports `cull()` -- removing tasks not required to produce a set of target task IDs. This uses backward BFS from targets to collect all transitive dependencies, then removes everything else:

```rust
// Source: crates/roko-orchestrator/src/dag.rs, lines 690-767

pub fn cull(&mut self, targets: &[String]) -> usize {
    // Backward BFS from targets collects all transitive dependencies.
    // Every node NOT in that set is removed from the DAG.
    // Returns the number of culled tasks.
}
```

*Source: `crates/roko-orchestrator/src/dag.rs`, lines 690-767*

### Linear Chain Fusion

The DAG supports `fuse_linear_chains()` -- collapsing eligible linear chains (A -> B -> C where each has exactly one dependent and one dependency) into single compound tasks. This reduces scheduling overhead for trivially sequential work.

*Source: `crates/roko-orchestrator/src/dag.rs`, lines 775 onwards*

---

## 11. Pheromone-Based Swarm Coordination

### Overview

When multiple agents work on related tasks, they need to coordinate without centralized scheduling. Roko uses a **pheromone-based coordination model** inspired by stigmergic systems (like ant colonies), where agents communicate indirectly through environmental signals rather than direct messaging [Dorigo1992, Grasse1959].

Stigmergy -- a term coined by Pierre-Paul Grasse in 1959 to describe termite nest construction behavior -- is indirect coordination through environment modification. Ants deposit pheromones on paths they traverse; subsequent ants preferentially follow paths with stronger pheromone concentrations, creating a positive feedback loop that converges on optimal paths. Ant Colony Optimization (ACO) [Dorigo1996] formalized this biological insight into a family of optimization algorithms.

Recent research has established theoretical connections between pheromone-mediated stigmergy and reinforcement learning [SwarmSys2025], motivating the use of digital signals as coordination primitives in multi-agent AI systems.

### Pheromone Kinds

Seven built-in pheromone kinds provide the coordination vocabulary:

```rust
// Source: crates/roko-orchestrator/src/coordination.rs, lines 189-207

pub enum PheromoneKind {
    Threat,       // something dangerous or harmful detected
    Opportunity,  // favorable condition detected
    Wisdom,       // validated knowledge or insight
    Alpha,        // first-mover advantage or ephemeral edge
    Pattern,      // recurring structure or regularity detected
    Anomaly,      // deviation from expected behavior
    Consensus,    // agreement among multiple agents
    Custom(String), // user-defined kinds with validation
}
```

*Source: `crates/roko-orchestrator/src/coordination.rs`, lines 189-207*

Custom kinds must pass validation: ASCII alphanumeric + underscore, 1-64 characters, no underscore prefix (reserved), no collision with built-in names.

*Source: `crates/roko-orchestrator/src/coordination.rs`, lines 92-115*

### Usage Patterns

- **Agent finds useful API pattern** -> deposits `Opportunity` pheromone. Other agents working on related code can discover and reuse the pattern.
- **Agent encounters compile error** -> deposits `Threat` pheromone with error details. Nearby agents can proactively adjust their code.
- **Agent claims a task** -> deposits `Alpha` pheromone. Other agents know not to duplicate the work.
- **Agent discovers recurring test failure pattern** -> deposits `Pattern` pheromone. The orchestrator can use this for targeted retry strategies.
- **Agent completes a validated solution** -> deposits `Wisdom` pheromone. Future agents can reference it as established knowledge.

### Subnets and Collectives

Pheromones are scoped to **subnets** within **collectives**. This prevents pheromone pollution in large multi-project environments:

```rust
// Source: crates/roko-orchestrator/src/coordination.rs, lines 148-163

pub struct SubnetId {
    pub collective: CollectiveId,
    pub name: String,
}
```

*Source: `crates/roko-orchestrator/src/coordination.rs`, lines 148-163*

### MeshRelay: Peer-to-Peer Pheromone Synchronization

The `MeshRelay` handles pheromone distribution across agents with three key mechanisms:

1. **Version-vector deduplication**: Each agent tracks the highest sequence number seen from each origin. Duplicate pheromones are silently dropped.

2. **Subscription filtering**: Agents subscribe to specific `PheromoneKind`s. The relay only delivers pheromones matching the agent's subscriptions.

3. **Store-and-forward for offline agents**: When a target agent is disconnected, pheromones are queued and delivered when the agent reconnects.

```rust
// Source: crates/roko-orchestrator/src/mesh_relay.rs, lines 57-60

pub struct MeshRelay {
    inner: Arc<Mutex<Inner>>,
    local_seq: Arc<AtomicU64>,  // lock-free sequence counter
}

struct Inner {
    peers: HashMap<AgentId, PeerState>,
    version_vectors: HashMap<AgentId, SeqNo>,
    store_forward: HashMap<AgentId, Vec<SequencedPheromone>>,
}
```

*Source: `crates/roko-orchestrator/src/mesh_relay.rs`, lines 42-60*

The publish path:

```rust
// Source: crates/roko-orchestrator/src/mesh_relay.rs, lines 87-100

pub fn publish(&self, origin: &AgentId, pheromone: Pheromone) -> SeqNo {
    let seq = self.local_seq.fetch_add(1, Ordering::Relaxed);
    let mut inner = self.inner.lock();

    // Version-vector dedup: skip if we already saw a higher seq from this origin.
    let seen = inner.version_vectors.entry(origin.clone()).or_insert(0);
    if seq <= *seen { return 0 }
    *seen = seq;

    // Deliver to connected subscribers, store-and-forward for offline peers.
    // ...
}
```

*Source: `crates/roko-orchestrator/src/mesh_relay.rs`, lines 87-100*

---

## 12. Worked Examples: Multi-Agent Execution with Recovery

### Example 1: Happy Path -- Three-Plan Parallel Execution

Consider three plans with one cross-plan dependency:

```
Plan A: "add-auth"    (tasks: T1-auth-model, T2-auth-handler)
Plan B: "add-api"     (tasks: T1-api-routes, T2-api-tests; depends_on: Plan A)
Plan C: "update-docs" (tasks: T1-readme; no dependencies)
```

**Tick 1**: Executor sees all three plans queued. Plan B is blocked (depends on A). Plans A and C can proceed.

```
Actions returned: [
    DispatchPlan { plan_id: "add-auth" },
    DispatchPlan { plan_id: "update-docs" },
]
```

**Tick 2**: After Start events are fed back, both plans enter Enriching. Strategist agents are spawned.

**Ticks 3-8**: Plans A and C progress through Enriching -> Implementing -> Gating -> Verifying -> Reviewing -> DocRevision -> Merging -> Complete independently and in parallel.

**Tick 9**: Plan A reaches Complete. Now `deps_satisfied("add-api")` returns true. Plan B is dispatched.

**Ticks 10-16**: Plan B proceeds through the full lifecycle.

Total wall-clock time: the critical path is max(Plan A, Plan C) + Plan B. If Plan C finishes before Plan A, its compute is fully overlapped with Plan A.

### Example 2: Gate Failure with Auto-Fix Recovery

A plan reaches Gating. The compile gate fails (missing import).

```
State: plan="add-auth", phase=Gating, iteration=1

Event:  GateFailed
Result: iteration(1) < MAX_AUTO_FIX_ITERATIONS(5), so -> AutoFixing

Action: SpawnAgent { role: AutoFixer, task: "fix" }
```

The AutoFixer agent adds the missing import.

```
Event:  AutoFixDone
Result: -> Gating (iteration still 1, gate_results cleared by reset_for_retry)

Action: RunGate { rung: 0 }
```

The compile gate passes. Test gate runs.

```
Event:  GatePassed
Result: -> Verifying
```

If the gate had failed again, the loop would repeat. After 5 failures:

```
State: iteration=5 (= MAX_AUTO_FIX_ITERATIONS)
Event: GateFailed
Result: -> Failed { reason: AutoFixExhausted }
```

The plan is now terminally failed. Recovery Level 2 (re-planning) would evaluate the failure evidence and potentially decompose the task.

### Example 3: Crash Recovery

The orchestrator is running 3 plans. It crashes after writing the following events to the log but before the snapshot is flushed:

```
Event log (persisted):
  seq=0  PlanStarted   { plan: "A" }
  seq=1  PlanStarted   { plan: "B" }
  seq=2  PhaseTransition { plan: "A", to: "Implementing" }
  seq=3  PhaseTransition { plan: "B", to: "Enriching" }

Snapshot (persisted, but stale):
  Plan A: phase=Enriching, iteration=1
  Plan B: phase=Queued
  Queue: ["A", "B"]
```

On restart, the `RecoveryEngine` runs:

1. **Recover from snapshot**: A=Enriching, B=Queued.
2. **Recover from event log**: A=Implementing (seq=2), B=Enriching (seq=3).
3. **Merge**: Event log wins on conflict. Final state: A=Implementing, B=Enriching.
4. **Validate**: No warnings. Both plans are non-terminal -> `ContinueActive`.

The executor resumes from A=Implementing, B=Enriching with no lost progress.

### Example 4: Speculative Execution

Plan "big-refactor" has task T3 with `expected_minutes=10`. After 25 minutes (exceeds `10 * 2.0 = 20` threshold):

```rust
executor.register_speculative_execution(
    "big-refactor", "T3", 10, 25,
    AgentRole::Implementer, 3.50  // projected cost
);
// Returns: Some(StartSpeculativeExecution { ... })
```

The runtime spawns a backup Implementer agent. If the original finishes first:

```rust
executor.resolve_speculative_execution("big-refactor", "T3");
// Returns: Some(CancelSpeculativeExecution { ... })
// Runtime kills the backup agent.
```

---

## 13. IronClaw Integration

### Current State: IronClaw's Existing Orchestrator

IronClaw already has an orchestrator at `src/orchestrator/` (4 modules, ~700 lines) that manages sandboxed worker containers. It provides:

- An internal HTTP API (`api.rs`) at port 50051 for worker-to-host communication (LLM proxy, credential injection, status updates)
- Per-job bearer token authentication (`auth.rs`)
- Container lifecycle management (`job_manager.rs`) with three modes: Worker, ClaudeCode, and ACP
- Stale container reaping (`reaper.rs`)

This orchestrator is focused on **single-job container management** -- it does not handle multi-job coordination, event sourcing, task DAGs, or crash recovery. The roko orchestrator patterns complement it rather than replacing it.

### A. Job Pipeline Enhancement

**Where**: `src/agent/` -- job management

**Current flow**:
```
User message -> Single agent loop -> Response
```

**Enhanced with orchestrator patterns**:
```
User message -> Plan decomposition -> Task DAG -> Wave scheduling
    -> Parallel agent execution -> Gate verification -> Merge -> Response
```

**Code sketch** (how the `ParallelExecutor` pattern would integrate with IronClaw's job state machine):

IronClaw's existing job states (`Pending -> InProgress -> Completed -> Submitted -> Accepted | Failed | Stuck`) map naturally to a subset of the plan phases:

| IronClaw Job State | Roko Plan Phase Equivalent |
|-------------------|---------------------------|
| Pending | Queued |
| InProgress | Implementing / Gating / AutoFixing |
| Completed | Verifying / Reviewing |
| Submitted | Merging |
| Accepted | Complete |
| Failed | Failed |
| Stuck | AutoFixing (with bounded retry) |

```rust
// In src/agent/orchestrator.rs (new module)

pub struct OrchestratedJobRunner {
    executor: ParallelExecutor,
    event_log: EventLog,
    dag: Option<UnifiedTaskDag>,
}

impl OrchestratedJobRunner {
    pub fn new() -> Self {
        let config = ExecutorConfig {
            max_concurrent_plans: 4,
            max_concurrent_tasks: 8,
            ..Default::default()
        };
        Self {
            executor: ParallelExecutor::new(config),
            event_log: EventLog::new(),
            dag: None,
        }
    }

    /// Main orchestration loop.
    pub async fn run(&mut self) {
        loop {
            // 1. Tick the executor to get pending actions.
            let actions = self.executor.tick();
            if actions.is_empty() && self.all_plans_terminal() {
                break;
            }

            // 2. Dispatch each action to the appropriate subsystem.
            for action in actions {
                match action {
                    ExecutorAction::SpawnAgent { plan_id, role, task } => {
                        // Use IronClaw's existing ContainerJobManager to
                        // spawn agents in Docker containers.
                        self.spawn_agent(&plan_id, role, &task).await;
                    }
                    ExecutorAction::RunGate { plan_id, rung } => {
                        // Run gate pipeline (compile, test, clippy)
                        // using IronClaw's sandbox infrastructure.
                        self.run_gate(&plan_id, rung).await;
                    }
                    ExecutorAction::MergeBranch { plan_id } => {
                        // Merge the plan's worktree into main.
                        self.merge_branch(&plan_id).await;
                    }
                    // ... other actions ...
                }
            }

            // 3. Feed results back as events.
            // (results arrive via channels, mapped to ExecutorEvent)
        }
    }
}
```

### B. Event-Sourced Job History

**Where**: `src/db/`, `src/history/`

**How**: Record all job state transitions as events using the `EventLog`. The event log is persisted to IronClaw's dual-backend database (PostgreSQL + libSQL) and enables:

- **Full job replay**: Reconstruct any point in a job's history by replaying events up to that timestamp. This aligns with IronClaw's "LLM data is never deleted" principle.
- **Analytics**: Query event patterns across jobs (e.g., "which tasks fail most often?", "what is the average gate pass rate?").
- **Crash recovery**: On restart, replay the event log to reconstruct the executor state, then resume from where the crash occurred.

### C. Multi-Job Coordination

**Where**: `src/agent/`

**How**: When multiple jobs are pending, build a `UnifiedTaskDag` across all jobs. This integrates with IronClaw's existing `ToolDispatcher::dispatch()` pipeline (all actions go through tools):

- **Cross-job dependency tracking**: Job B's task depends on Job A's output.
- **Automatic conflict prevention**: File-overlap inference serializes jobs that would conflict.
- **Global wave scheduling**: Independent tasks from different jobs execute in the same wave.

### D. Audit Chain for ActionRecords

**Where**: `src/observability/`, `src/tools/dispatch.rs`

**How**: Add BLAKE3 hash linking to IronClaw's existing `ActionRecord` system for a tamper-evident audit trail. IronClaw already records every tool call through `ToolDispatcher::dispatch()` -- augmenting this with hash chaining provides regulatory-grade auditability (relevant for EU AI Act compliance).

```rust
// Code sketch for audit chain integration

pub struct AuditableToolDispatcher {
    inner: ToolDispatcher,
    audit_chain: AuditChain,
}

impl AuditableToolDispatcher {
    pub async fn dispatch(&self, tool: &str, params: Value) -> Result<ToolOutput> {
        let result = self.inner.dispatch(tool, params).await?;

        // Record the tool call on the audit chain.
        let entry = AuditEntry::new(
            [0u8; 32],  // prev_hash wired by AuditChain::append
            format!("tool.{}", tool),
            "agent",    // actor
            tool,       // resource
        );
        self.audit_chain.append(entry);

        Ok(result)
    }

    pub fn verify_integrity(&self) -> bool {
        self.audit_chain.verify()
    }
}
```

### E. Worktree Isolation for Parallel Execution

**Where**: `src/sandbox/`

**How**: The orchestrator's `WorktreeManager` gives each plan its own isolated git worktree, preventing file conflicts during parallel execution. This complements IronClaw's existing Docker sandbox infrastructure (`src/sandbox/`) by adding git-level isolation on top of container-level isolation.

```rust
// Source: crates/roko-orchestrator/src/worktree.rs, lines 38-55

pub struct WorktreeConfig {
    pub repo_root: PathBuf,        // main repository checkout
    pub base_branch: String,       // branch to fork from
    pub worktrees_root: PathBuf,   // directory for worktree checkouts
    pub max_live: Option<usize>,   // maximum concurrent worktrees
    pub idle_ttl: Duration,        // reclaim after this idle duration
}
```

*Source: `crates/roko-orchestrator/src/worktree.rs`, lines 38-55*

Features implemented:
- Create/remove worktrees with ephemeral branch naming
- Health checks (missing directories, stale locks, detached HEAD)
- Budget enforcement + idle reclamation
- Stale lock detection (locks older than 60 seconds)
- Force-remove and prune stale git metadata

*Source: `crates/roko-orchestrator/src/worktree.rs`, lines 10-19*

### F. Integration with Engine v2 Per-Project Sandbox

IronClaw's engine v2 already supports per-project Docker containers (see `CLAUDE.md`, "Engine v2 Per-Project Sandbox"). The orchestrator's `WorktreeManager` would sit at the git layer:

```
User Request
  -> Plan Decomposition (new)
    -> UnifiedTaskDag (new)
      -> Wave Scheduling (new)
        -> Per-plan git worktree (new, from roko-orchestrator)
          -> Per-plan Docker container (existing, from src/sandbox/)
            -> Agent execution with tool dispatch (existing)
```

This layered isolation gives each parallel agent its own filesystem view (worktree) running inside its own sandboxed container.

---

## 14. References

### Event Sourcing and CQRS

- **[Fowler2005]** Fowler, Martin. "Event Sourcing." martinfowler.com, 2005. https://martinfowler.com/eaaDev/EventSourcing.html -- Original description of the event sourcing pattern, establishing the foundational vocabulary.

- **[Young2010]** Young, Greg. "CQRS Documents." 2010. -- Introduced Command Query Responsibility Segregation as a companion pattern to event sourcing, separating read and write models.

- **[Temporal2024]** Temporal Technologies. "Durable Execution and Event History." Temporal Platform Documentation, 2024. https://docs.temporal.io/workflow-execution/event -- Demonstrates how deterministic replay from event history enables crash recovery: "every step of a workflow is persisted as an event in an Event History."

### Hash Chains and Tamper-Evident Logs

- **[Merkle1988]** Merkle, Ralph C. "A Digital Signature Based on a Conventional Encryption Function." Advances in Cryptology -- CRYPTO '87, Springer, 1988. -- Foundational work on Merkle hash trees, which generalize hash chains to tree structures.

- **[Laurie2013]** Laurie, Ben, Adam Langley, and Emilia Kasper. "Certificate Transparency." RFC 6962, 2013. https://www.rfc-editor.org/rfc/rfc6962 -- Defines tamper-evident append-only logs using Merkle trees for public audit of TLS certificates.

- **[OConnor2020]** O'Connor, Jack, et al. "BLAKE3 -- One Function, Fast Everywhere." 2020. https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf -- Specification of the BLAKE3 hash function: significantly faster than SHA-256 with equivalent 256-bit security, tree-structured for parallelism.

### DAG Scheduling and Critical Path

- **[Kwok1999]** Kwok, Yu-Kwong, and Ishfaq Ahmad. "Static Scheduling Algorithms for Allocating Directed Task Graphs to Multiprocessors." ACM Computing Surveys 31.4 (1999): 406-471. -- Comprehensive survey of DAG-based task scheduling algorithms for parallel systems, covering critical path methods and list scheduling.

- **[Kelley1959]** Kelley, James E., and Morgan R. Walker. "Critical-Path Planning and Scheduling." Proceedings of the Eastern Joint Computer Conference, 1959. -- Original description of the Critical Path Method (CPM), showing that the longest path through a task dependency graph determines the minimum project duration.

### Swarm Intelligence and Stigmergy

- **[Grasse1959]** Grasse, Pierre-Paul. "La reconstruction du nid et les coordinations interindividuelles chez Bellicositermes natalensis et Cubitermes." Insectes Sociaux 6 (1959): 41-80. -- Coined the term "stigmergy" to describe indirect coordination through environment modification in termite nest construction.

- **[Dorigo1992]** Dorigo, Marco. "Optimization, Learning and Natural Algorithms." PhD thesis, Politecnico di Milano, 1992. -- Introduced Ant Colony Optimization (ACO), formalizing biological pheromone-based coordination into a family of optimization algorithms.

- **[Dorigo1996]** Dorigo, Marco, Vittorio Maniezzo, and Alberto Colorni. "Ant System: Optimization by a Colony of Cooperating Agents." IEEE Transactions on Systems, Man, and Cybernetics 26.1 (1996): 29-41. -- Formalization of the Ant System algorithm with experimental results.

- **[SwarmSys2025]** "SwarmSys: Decentralized Swarm-Inspired Agents for Scalable and Adaptive Reasoning." arXiv:2510.10047, 2025. https://arxiv.org/html/2510.10047v1 -- Recent work establishing theoretical connections between pheromone-mediated stigmergy and reinforcement learning for LLM-based multi-agent coordination.

### Crash Recovery Patterns

- **[Bernstein1987]** Bernstein, Philip A., Vassos Hadzilacos, and Nathan Goodman. "Concurrency Control and Recovery in Database Systems." Addison-Wesley, 1987. -- Foundational textbook on write-ahead logging (WAL), checkpointing, and crash recovery protocols that underpin the snapshot + event replay pattern.

---

## 15. Complexity Assessment

### Lines of Code (Roko Reference Implementation)

| Module | Lines | Purpose |
|--------|------:|---------|
| `executor/mod.rs` | 1,222 | ParallelExecutor, config, speculative execution |
| `executor/state_machine.rs` | 636 | Phase transition logic + tests |
| `executor/plan_state.rs` | 456 | Per-plan mutable state + tests |
| `executor/action.rs` | 261 | ExecutorAction enum + tests |
| `executor/snapshot.rs` | 1,086 | Snapshot, delta, BLAKE3 verifier + tests |
| `executor/recovery.rs` | 1,279 | Crash recovery engine + tests |
| `executor/reorder.rs` | 163 | Queue reordering |
| `executor/priority_ceiling.rs` | 439 | Priority inheritance |
| `executor/resource_budget.rs` | 578 | Token/cost budgets |
| `dag.rs` | 2,559 | Unified DAG, waves, file overlap, CPM, mutations, culling, fusion |
| `event_log.rs` | 647 | Hash-chained event journal + tests |
| `coordination.rs` | 1,991 | Pheromones, subnets, collectives |
| `mesh_relay.rs` | 308 | Pheromone relay |
| `replan.rs` | 446 | Re-planning strategies |
| `safety/audit_chain.rs` | 565 | BLAKE3 audit chain |
| `worktree.rs` | 1,205 | Git worktree management |
| Other modules | ~4,936 | merge_queue, plan_discovery, post_merge, progress, repair, safety/* |
| **Total** | **~18,777** | Full orchestrator crate (code + tests) |

*Line counts verified from source on 2026-07-02.*

### IronClaw Integration Estimates

| Component | Estimated Lines | Risk |
|-----------|---------------:|------|
| Orchestrated job runner | 800-1,000 | Medium -- core integration, needs thorough testing |
| Event-sourced job history | 400-500 | Low -- builds on existing DB infrastructure |
| Multi-job DAG construction | 300-400 | Medium -- cross-job dependency resolution |
| Audit chain for ActionRecords | 200-300 | Low -- straightforward BLAKE3 append |
| Worktree isolation adapter | 200-300 | Low -- wraps git operations |
| **Total** | **~2,000-2,500** | |

### Dependencies

- `blake3` -- BLAKE3 hashing for audit chain and snapshot verification
- `chrono` -- timestamps for event entries
- `parking_lot` -- efficient mutexes for thread-safe audit chain and event log
- `serde`/`serde_json` -- serialization for snapshots, events, and state
- `thiserror` -- error type derivation
- `roko-core` -- shared types (`PlanPhase`, `PhaseKind`, `GlobalTaskId`, `Task`, `AgentRole`, `ContentHash`, `FailureKind`, `Verdict`, `TestCount`)

### Key Risks

1. **Orchestration complexity**: The state machine has 12+ phases and ~30 legal transitions. Exhaustive testing is essential -- the roko reference has extensive test coverage (every transition, every error path, full lifecycle roundtrips with 636 lines of tests in `state_machine.rs` alone).

2. **Crash recovery correctness**: The snapshot + event log merge has subtle semantics (event log wins on conflict, queue order preference). The recovery engine's validation and warning system catches inconsistencies, but edge cases around concurrent snapshots and event appends need careful testing.

3. **File-conflict inference false positives**: The current algorithm is conservative -- any shared file creates a serialization edge. This can over-serialize in practice (e.g., two tasks that both touch `Cargo.toml` but for unrelated reasons). Future work could use finer-grained conflict analysis (e.g., line-range overlap, semantic diff).

4. **Pheromone coordination overhead**: The swarm coordination system (1,991 lines in `coordination.rs`, 308 in `mesh_relay.rs`) adds substantial complexity. The MeshRelay with store-and-forward queues, version vectors, and subscription filtering is non-trivial to operate and debug. Start with the pure state machine orchestrator and add pheromone coordination as a second phase.

5. **Integration with IronClaw's existing "everything goes through tools" principle**: The orchestrator must dispatch actions through `ToolDispatcher::dispatch()` to maintain the audit trail, safety pipeline, and channel-agnostic surface. This means wrapping roko's `ExecutorAction` variants as tool invocations rather than calling subsystems directly.

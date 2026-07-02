# DAG Execution Engine

**Source crate**: `roko-graph` (`crates/roko-graph/`)
**Priority**: HIGH -- replaces ad-hoc job chaining with declarative workflows
**Roko doc references**: `docs/v2/03-GRAPH.md`, `docs/v2/04-EXECUTION.md`, `docs/v1/01-orchestration/02-unified-task-dag.md`, `docs/v2-depth/05-execution-engine/cognitive-loop-as-graph.md`

---

## Table of Contents

1. [Background: What Is a DAG Execution Engine?](#1-background-what-is-a-dag-execution-engine)
2. [Why DAGs Over Linear Pipelines](#2-why-dags-over-linear-pipelines)
3. [Academic Context and Prior Art](#3-academic-context-and-prior-art)
4. [The roko-graph Crate: Full Architecture](#4-the-roko-graph-crate-full-architecture)
5. [The Cell Trait: Universal Computation Unit](#5-the-cell-trait-universal-computation-unit)
6. [CellRegistry: The Factory Pattern](#6-cellregistry-the-factory-pattern)
7. [Graph Types and Data Model](#7-graph-types-and-data-model)
8. [Edge Conditions: Conditional Routing](#8-edge-conditions-conditional-routing)
9. [Condition System: Deep-Path Field Evaluation](#9-condition-system-deep-path-field-evaluation)
10. [TOML Loader: Declarative Graph Definitions](#10-toml-loader-declarative-graph-definitions)
11. [Topological Sort and Dependency Resolution](#11-topological-sort-and-dependency-resolution)
12. [The GraphEngine: Sequential Execution](#12-the-graphengine-sequential-execution)
13. [Budget Tracking System](#13-budget-tracking-system)
14. [Hot Graphs: Tick-Driven Resident Execution](#14-hot-graphs-tick-driven-resident-execution)
15. [Plan-to-Graph Conversion Pipeline](#15-plan-to-graph-conversion-pipeline)
16. [Built-in Cell Implementations](#16-built-in-cell-implementations)
17. [Design Concepts from Roko v2 Docs](#17-design-concepts-from-roko-v2-docs)
18. [Practical Workflow Examples](#18-practical-workflow-examples)
19. [IronClaw Integration Architecture](#19-ironclaw-integration-architecture)
20. [Implementation Plan for IronClaw](#20-implementation-plan-for-ironclaw)
21. [Complexity Assessment](#21-complexity-assessment)
22. [References](#22-references)

---

## 1. Background: What Is a DAG Execution Engine?

A **Directed Acyclic Graph (DAG) execution engine** is a runtime system that models workflows as graphs where **nodes represent units of computation** (tasks, commands, LLM calls) and **directed edges represent data dependencies and control flow** between them. "Acyclic" means there are no circular dependencies -- task A cannot depend on task B if task B already depends on task A.

The key insight is that a DAG captures the *actual dependency structure* of work. In contrast to a linear pipeline (step 1, then step 2, then step 3), a DAG allows expressing:

- **Parallelism**: If tasks B and C both depend only on task A, they can execute simultaneously once A completes.
- **Fan-out/fan-in**: One task can trigger multiple downstream tasks (fan-out), and multiple tasks can converge into a single downstream task (fan-in).
- **Conditional routing**: Edges can carry conditions -- an "on failure" edge only fires if the upstream task failed, enabling structured error-handling branches.
- **Partial results**: If one branch of the graph fails, unrelated branches that do not depend on it can still complete.

DAG execution engines are foundational across computing:

- **Data engineering**: Apache Airflow [1], Dagster, Prefect [3], Dask [5] all model ETL pipelines as task DAGs.
- **CI/CD**: GitHub Actions, GitLab CI, and Jenkins Pipeline define build steps as DAG nodes with dependency edges.
- **Build tools**: Bazel, Ninja, and Make use DAGs to express compilation dependencies and enable incremental rebuilds.
- **Durable workflow engines**: Temporal.io [2] and Cadence use DAG-like workflow definitions with deterministic replay for fault tolerance.
- **AI agent orchestration**: LangGraph [4], CrewAI, and DSPy are increasingly adopting DAG-based orchestration for multi-step agent workflows.

The roko-graph crate is purpose-built for AI agent workflows, with first-class support for LLM calls, budget tracking, conditional routing, and hot (resident) execution loops.

### Comparison: Linear Pipeline vs. DAG

```
Linear Pipeline:
  A --> B --> C --> D --> E
  (Total time: sum of all steps. No parallelism.)

DAG Pipeline:
  A --> B --> D --> E
  A --> C ---^
  (B and C run in parallel. Total time: A + max(B,C) + D + E.)
```

For a concrete example: "Build and deploy my project" as a linear pipeline forces sequential execution of every step. As a DAG:

```
Parse Requirements --> [Generate Code, Write Tests] --> Run Tests --> Build --> Deploy
                                                           |
                                                    (on failure) --> Fix Code --> Run Tests
```

Generate Code and Write Tests run in parallel. If tests fail, a conditional edge routes to a Fix Code step that loops back. This is impossible to express in a sequential pipeline without manual control flow.

### Why This Matters for AI Agents

AI agent workflows have properties that make DAG execution particularly valuable:

1. **Non-deterministic execution**: LLM calls can fail, produce unexpected output, or vary in quality. DAGs allow structured fallback and retry branches.
2. **Cost sensitivity**: LLM API calls have per-token costs. DAG engines can track budget across an entire workflow and halt gracefully when limits are reached.
3. **Latency sensitivity**: Many agent tasks (web search, code compilation, file I/O) are I/O-bound and can run in parallel. Sequential execution wastes wall-clock time.
4. **Composability**: Complex agent behaviors (code review, research synthesis, deployment) are naturally composed from smaller steps, which map to DAG nodes.

---

## 2. Why DAGs Over Linear Pipelines

IronClaw currently executes jobs as sequential agent turns -- one step after another. This has several limitations that a DAG engine solves:

### Problem 1: Wasted Time from Artificial Sequencing
If a job involves "check email" and "check GitHub notifications" followed by "summarize," the checks have no dependency on each other but execute sequentially. A DAG allows them to run in parallel, cutting wall-clock time.

### Problem 2: No Conditional Branching
When a tool call fails, IronClaw's current job system has no structured way to branch to error-handling logic vs. success-path logic. DAG edges with conditions (`OnSuccess`, `OnFailure`, `When`) make this declarative.

### Problem 3: No Budget Enforcement Across Multi-Step Workflows
IronClaw tracks costs per-tool-call but has no mechanism to enforce a budget across an entire workflow. A DAG engine checks the budget before each node execution and terminates gracefully when limits are hit, returning partial results.

### Problem 4: No Reusable Workflow Definitions
Complex sequences of tool calls are currently defined imperatively in Rust code or agent prompts. TOML-defined DAGs make workflows declarative, shareable, and composable -- a "code review" workflow can be defined once and reused across projects.

### Problem 5: No Resident/Periodic Execution Model
IronClaw's heartbeat system runs simple periodic checks. Hot Graphs generalize this to any periodic workflow with full DAG structure, state persistence between ticks, and configurable tick intervals.

---

## 3. Academic Context and Prior Art

The roko-graph crate draws on a rich body of prior work in workflow orchestration, task scheduling, and incremental computation. Understanding this context clarifies why specific design decisions were made and where the system sits in the broader landscape.

### 3.1 Classical DAG Scheduling Theory

The theoretical foundation for DAG-based task scheduling dates to the study of parallel processor scheduling in the 1960s-1970s. Coffman and Graham's algorithm (1972) established that optimal scheduling of unit-time tasks on two processors can be solved in polynomial time via topological labeling [6]. For the general case (variable-time tasks on heterogeneous processors), the problem is NP-hard, motivating a rich literature of heuristic schedulers.

**Kahn's algorithm** (1962) [7] provides the canonical approach for topological sorting. The algorithm maintains a set of nodes with zero in-degree, repeatedly removing one and decrementing the in-degrees of its successors. If all nodes are removed, the graph is acyclic; otherwise a cycle exists. It runs in O(V + E) time. The roko-graph crate delegates to petgraph's `toposort()` function [8], which implements a DFS-based variant with the same time complexity.

### 3.2 Workflow Orchestration Systems

**Apache Airflow** [1] pioneered DAG-as-code for data pipeline orchestration. Workflows are defined as Python code that constructs operator DAGs, which the scheduler executes based on time triggers and dependency resolution. Airflow 3.0 (2025) introduced event-driven scheduling and DAG versioning, moving beyond pure time-based triggers.

**Prefect** [3] introduced a "negative engineering" philosophy, focusing on what goes wrong in production workflows. Its task DAG model adds first-class support for retries, caching, parameter injection, and state handlers -- concerns that roko-graph addresses through `EdgeCondition`, `BudgetTracker`, and the `CellContext` injection pattern.

**Temporal.io** [2] takes a fundamentally different approach: **durable execution through deterministic replay**. Workflows are written as normal code, but activities (non-deterministic operations like API calls) are recorded in an event history. On failure, the workflow function is re-executed from scratch, replaying the recorded history to reconstruct state without re-executing activities. This workflow/activity split -- deterministic orchestration vs. non-deterministic execution -- directly inspired roko's v2 architecture (see Section 17.2).

### 3.3 Heterogeneous Task Scheduling

The **HEFT (Heterogeneous Earliest Finish Time)** algorithm, introduced by Topcuoglu, Hariri, and Wu (2002) [9], addresses DAG scheduling on heterogeneous processors. HEFT prioritizes tasks by their upward rank (longest path to exit node, weighted by average computation cost), then assigns each task to the processor that minimizes its earliest finish time. HEFT runs in O(V^2 * P) time for V tasks and P processors. The roko v1 documentation references HEFT-like heuristics for multi-agent task dispatch across agents with different model capabilities.

### 3.4 Graph Optimization Passes

**Dask** [5] implements several optimization passes on task graphs before execution:

- **Culling** (`cull()`): Removes tasks not required to produce the requested outputs, analogous to dead-code elimination in compilers.
- **Fusion** (`fuse()`): Merges linear chains of single-dependency tasks into compound tasks, reducing inter-task communication overhead.
- **Inlining** (`inline()`, `inline_functions()`): Replaces cheap tasks with their inlined computation to reduce scheduling overhead.

These ideas appear in roko's v1 documentation as "DAG Optimization Passes" for task fusion, DAG culling, and speculative execution.

### 3.5 Incremental Computation

**Adapton** [10] (Hammer et al., PLDI 2014) introduced the *demanded computation graph* (DCG) and a demand-driven change propagation algorithm for incremental computation. When inputs change, only the affected portion of the computation graph is re-evaluated. **Salsa** [11], inspired by Adapton, provides the same capability for the Rust compiler's query system. Roko's v1 documentation references "Adapton/Salsa-inspired dirty/clean propagation" for partial DAG re-execution, where only nodes whose inputs have changed are re-computed.

### 3.6 DAG Execution for AI Agents

Recent academic work has formalized the connection between AI agent execution and DAG scheduling theory. Notably, "From Agent Loops to Structured Graphs: A Scheduler-Theoretic Framework for LLM Agent Execution" (arXiv:2604.11378) [12] characterizes the standard agent loop as a "single ready unit scheduler" and places agent loops and graph-based execution engines on a single semantic continuum. The paper identifies three structural weaknesses of agent loops -- implicit dependencies, unbounded recovery loops, and mutable execution history -- that graph-based execution directly addresses.

**LangGraph** [4] implements graph-based agent orchestration in Python, modeling agent steps as nodes with conditional edges for branching. **GraphFlow** (arXiv:2605.22566) [13] proposes a graph-based workflow management system specifically for efficient LLM-agent serving. **GRADE** (arXiv:2606.22741) [14] introduces formal graph representations for LLM agent dependency and execution tracking.

---

## 4. The roko-graph Crate: Full Architecture

The `roko-graph` crate is the implementation of the DAG execution engine. It lives at `crates/roko-graph/` in the roko repository and provides the following modules:

```
crates/roko-graph/
├── Cargo.toml              # Dependencies: roko-core, petgraph, toml, serde, tokio, async-trait
└── src/
    ├── lib.rs              # Crate root, re-exports primary types
    ├── cell.rs             # Cell trait definition, CellContext, CellVersion
    ├── registry.rs         # CellRegistry: factory pattern for Cell instantiation
    ├── types.rs            # Graph, Node, Edge, EdgeCondition, NodeOutput, GraphConfig, GraphError
    ├── loader.rs           # TOML parser: load_from_str(), load_from_file()
    ├── topo.rs             # Topological sort, cycle detection, root/leaf node discovery
    ├── engine.rs           # GraphEngine: sequential execution, validation, default_registry()
    ├── budget.rs           # BudgetTracker: token, cost, and deadline enforcement
    ├── condition.rs        # Condition enum, CompareOp, field resolution, evaluate()
    ├── hot.rs              # Hot Graphs: tick-driven resident execution, HotGraphHandle
    ├── convert.rs          # Plan-to-Graph conversion: plan_to_graph(), PlanTaskInfo
    ├── error.rs            # GraphError re-export, Result alias
    └── cells/              # Built-in Cell implementations
        ├── mod.rs           # Re-exports: AgentCell, ComposeCell, GraduationCell, etc.
        ├── agent.rs         # AgentCell: LLM dispatch wrapper
        ├── compose.rs       # ComposeCell: template variable substitution
        ├── graduation.rs    # GraduationCell: promotes Bus Pulses to durable Signals
        ├── task_executor.rs # TaskExecutorCell: stub for plan-converted tasks
        └── stubs.rs         # PassthroughCell + cognitive loop stub names
```

### Dependencies (from `Cargo.toml`)
_Source: `crates/roko-graph/Cargo.toml`_

```toml
[dependencies]
roko-core = { path = "../roko-core" }     # Core types: Engram, Kind, Body, error types
petgraph = { workspace = true }            # Directed graph data structure + algorithms
toml = { workspace = true }                # TOML parsing for graph definitions
serde = { workspace = true }               # Serialization/deserialization
serde_json = { workspace = true }          # JSON for node output data
indexmap = { workspace = true }            # Insertion-ordered map for node ordering
parking_lot = { workspace = true }         # Fast mutex for budget breakdown
async-trait = "0.1"                        # Async trait support for Cell
tokio-util = { workspace = true }          # CancellationToken for Hot Graph shutdown
tracing = { workspace = true }             # Structured logging
thiserror = { workspace = true }           # Error type derivation

[dependencies.tokio]
workspace = true
features = ["process", "time", "rt", "macros"]
```

### Crate-level re-exports (from `lib.rs`)
_Source: `crates/roko-graph/src/lib.rs`_

```rust
// Primary types
pub use cell::{Cell, CellContext, CellVersion};
pub use engine::{GraphEngine, GraphOutput, NodeResult, NodeStatus, default_registry};
pub use registry::{CellFactory, CellRegistry};
pub use types::{
    Edge, EdgeCondition, Graph, GraphConfig, GraphError, GraphMetadata, GraphNodeIdx,
    Node, NodeId, NodeOutput, NodeOutputStatus,
};

// Budget tracking
pub use budget::{BudgetLimits, BudgetTracker, NodeCost};

// Condition evaluation
pub use condition::{CompareOp, Condition, evaluate};

// Plan conversion
pub use convert::{PlanTaskInfo, plan_to_graph, plan_to_graph_with_endpoints};

// Result alias
pub use error::Result as GraphResult;

// Hot Graph execution
pub use hot::{HotGraphHandle, HotPolicy, start_hot};
```

---

## 5. The Cell Trait: Universal Computation Unit

The `Cell` trait is the fundamental abstraction in roko-graph. **Every node in a graph is backed by a Cell implementation.** Cells are the units of work -- they receive input, perform computation, and produce output. The trait is defined in `crates/roko-graph/src/cell.rs`.

### Full Cell Trait Definition
_Source: `crates/roko-graph/src/cell.rs`_

```rust
/// Semantic version tuple for Cell implementations.
pub type CellVersion = (u32, u32, u32);

/// Universal computation unit. Every graph node is backed by a Cell implementation.
///
/// The Cell trait provides identity, cost estimation, and an async execute method.
/// Implementations include gates (compile, test, clippy), agent dispatch, compose
/// steps, and user-defined cells registered via `CellRegistry`.
#[async_trait]
pub trait Cell: Send + Sync + 'static {
    /// Unique identifier for this cell instance.
    fn cell_id(&self) -> &str;

    /// Human-readable name for display and logging.
    fn cell_name(&self) -> &str;

    /// Semantic version of this cell's implementation.
    fn cell_version(&self) -> CellVersion {
        (0, 1, 0)
    }

    /// Protocol names this cell conforms to (e.g. `["Gate", "Scorer"]`).
    fn protocols(&self) -> &[&str] {
        &[]
    }

    /// Estimated USD cost per invocation, when known.
    fn estimated_cost(&self) -> Option<f64> {
        None
    }

    /// Estimated wall-clock duration per invocation, when known.
    fn estimated_duration(&self) -> Option<Duration> {
        None
    }

    /// Execute this cell with the given input engrams, producing output engrams.
    ///
    /// The graph engine calls this in topological order, feeding outputs from
    /// upstream cells as inputs to downstream cells.
    async fn execute(&self, input: Vec<Engram>, ctx: &CellContext) -> Result<Vec<Engram>>;
}
```

### Key Design Decisions in the Cell Trait

**1. Input and output are `Vec<Engram>`**: Engrams are roko-core's universal data type -- typed, tagged, serializable values with provenance metadata. This means all inter-cell communication uses a single data type, avoiding schema mismatch issues.

**2. Cells are `Send + Sync + 'static`**: This allows cells to be used across async tasks and stored in the registry without lifetime constraints. Cells must be thread-safe.

**3. Cost and duration estimation are optional**: Not all cells can estimate their resource consumption in advance. LLM calls depend on prompt length; shell commands depend on the project. But when estimates are available, the budget tracker uses them for pre-execution checks.

**4. Protocol declarations**: Cells can declare which protocols they conform to (e.g., `["Gate"]`, `["React"]`, `["TaskExecution"]`). This is used for runtime validation and documentation, not for dispatch.

### CellContext: Runtime Context
_Source: `crates/roko-graph/src/cell.rs`_

```rust
/// Runtime context passed to `Cell::execute()`.
///
/// Provides the cell with access to shared infrastructure (cancel tokens,
/// budgets, trace context) without cells needing to manage their own handles.
#[derive(Debug, Clone)]
pub struct CellContext {
    /// Trace context for observability.
    pub trace_id: Option<String>,
    /// Run identifier (if executing within a Graph/Flow).
    pub run_id: Option<String>,
    /// Remaining budget for this execution (USD).
    pub budget_remaining: Option<f64>,
}
```

The CellContext is constructed by the engine and passed to each cell at execution time. It provides:
- **Observability**: `trace_id` for distributed tracing correlation
- **Identity**: `run_id` for associating outputs with a specific graph execution
- **Budget awareness**: `budget_remaining` so cells can adjust their behavior (e.g., use a cheaper LLM model when budget is low)

The context uses a builder pattern for construction:

```rust
let ctx = CellContext::new()
    .with_trace_id("trace-abc123".into())
    .with_run_id("run-001".into())
    .with_budget(0.45);
```

---

## 6. CellRegistry: The Factory Pattern

The `CellRegistry` maps cell type name strings to factory functions. When the graph engine encounters a node with `cell_type = "gate.compile"`, it looks up `"gate.compile"` in the registry to obtain a factory, calls it with the node's TOML config, and gets back a `Box<dyn Cell>` ready for execution.

### Full CellRegistry Implementation
_Source: `crates/roko-graph/src/registry.rs`_

```rust
/// A factory function that takes a TOML config and produces a boxed Cell.
pub type CellFactory = Box<dyn Fn(toml::Value) -> Box<dyn Cell> + Send + Sync>;

/// Registry that maps cell type name strings to factory functions.
///
/// When the graph engine encounters a node with `cell_type = "gate.compile"`,
/// it looks up "gate.compile" in this registry to obtain a factory, then calls
/// it with the node's config to instantiate the cell.
pub struct CellRegistry {
    factories: HashMap<String, CellFactory>,
}

impl CellRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self { /* ... */ }

    /// Register a factory function for a cell type name.
    /// If a factory was already registered for this name, it is replaced.
    pub fn register<F>(&mut self, cell_type: &str, factory: F)
    where
        F: Fn(toml::Value) -> Box<dyn Cell> + Send + Sync + 'static,
    { /* ... */ }

    /// Look up a factory by cell type name and instantiate a Cell with the given config.
    /// Returns `GraphError::UnknownCellType` if no factory is registered.
    pub fn create(&self, cell_type: &str, config: toml::Value)
        -> Result<Box<dyn Cell>, GraphError> { /* ... */ }

    /// Check if a cell type is registered.
    pub fn contains(&self, cell_type: &str) -> bool { /* ... */ }

    /// Return the number of registered cell types.
    pub fn len(&self) -> usize { /* ... */ }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool { /* ... */ }

    /// Return an iterator over registered cell type names.
    pub fn cell_types(&self) -> impl Iterator<Item = &str> { /* ... */ }
}
```

### Why Factory Functions, Not Direct Instantiation

Cell factories take a `toml::Value` (the `[nodes.config]` section from the TOML definition) and return a `Box<dyn Cell>`. This design means:

1. **Cells are configured at graph load time**, not hard-coded. The same cell type can be instantiated with different configs in different graph nodes.
2. **Cell implementations can be swapped** by re-registering the factory. Test registries can replace real cells with mocks.
3. **The graph definition is decoupled from Rust types**. TOML graph files reference cell types by string name, which the registry resolves at runtime.

### The Default Registry
_Source: `crates/roko-graph/src/engine.rs` (function `default_registry`)_

The default registry ships with the following cell types pre-registered:

| Cell Type | Implementation | Purpose |
|-----------|---------------|---------|
| `gate.compile` | `ShellCell("cargo", ["check", "--workspace"])` | Rust compilation gate |
| `gate.test` | `ShellCell("cargo", ["test", "--workspace"])` | Test suite gate |
| `gate.clippy` | `ShellCell("cargo", ["clippy", "--workspace", "--no-deps", "--", "-D", "warnings"])` | Lint gate |
| `noop` | `NoopCell` | Pass-through (testing/placeholder) |
| `score` | `NoopCell` (named "ScoreCell") | Placeholder for scoring |
| `compose` | `NoopCell` (named "ComposeCell") | Placeholder for prompt assembly |
| `act` | `NoopCell` (named "ActCell") | Placeholder for action execution |
| `task-executor` | `TaskExecutorCell` | Plan-to-graph task execution stub |
| `signal-reader` | `PassthroughCell` | Cognitive loop stub |
| `relevance-scorer` | `PassthroughCell` | Cognitive loop stub |
| `system-prompt-builder` | `PassthroughCell` | Cognitive loop stub |
| `claude-agent` | `PassthroughCell` | Cognitive loop stub |
| `gate-pipeline` | `PassthroughCell` | Cognitive loop stub |
| `store-writer` | `PassthroughCell` | Cognitive loop stub |
| `event-publisher` | `PassthroughCell` | Cognitive loop stub |

The cognitive loop stubs are registered so that TOML graph definitions referencing cognitive-loop cell types can load and validate without error. Real implementations will replace these stubs as they are built. The stub names are defined in `crates/roko-graph/src/cells/stubs.rs`:

```rust
pub const COGNITIVE_LOOP_STUBS: &[&str] = &[
    "signal-reader",
    "relevance-scorer",
    "system-prompt-builder",
    "claude-agent",
    "gate-pipeline",
    "store-writer",
    "event-publisher",
];
```

---

## 7. Graph Types and Data Model

The graph data model is defined in `crates/roko-graph/src/types.rs`. A `Graph` is a petgraph-backed directed graph with string-keyed nodes, typed edges, and metadata.

### Core Types

```rust
/// Unique identifier for a node within a graph.
pub type NodeId = String;

/// Index type used by petgraph for node indices.
pub type GraphNodeIdx = petgraph::graph::NodeIndex;

/// A directed acyclic graph of execution nodes, backed by petgraph.
#[derive(Debug, Clone)]
pub struct Graph {
    /// Graph metadata (name, description, labels).
    pub metadata: GraphMetadata,
    /// The underlying petgraph directed graph.
    pub inner: DiGraph<Node, Edge>,
    /// Mapping from `NodeId` (string) to petgraph node index.
    pub node_map: IndexMap<NodeId, GraphNodeIdx>,
}
```

The `Graph` uses `IndexMap` for the node map to preserve insertion order -- this matters for deterministic iteration when nodes have no dependency ordering between them.

### Node Definition

```rust
/// A node in the execution graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    /// Unique identifier within this graph.
    pub id: NodeId,
    /// Cell type name used to look up the factory in `CellRegistry`.
    pub cell_type: String,
    /// Configuration passed to the cell factory function.
    #[serde(default = "default_config")]
    pub config: toml::Value,
    /// Named inputs this node consumes (from upstream edges).
    #[serde(default)]
    pub inputs: Vec<String>,
    /// Named outputs this node produces (for downstream edges).
    #[serde(default)]
    pub outputs: Vec<String>,
}
```

The `cell_type` field is the key that links a node to its Cell implementation via the registry. The `config` field is arbitrary TOML passed to the factory -- this is how node-specific configuration (model name, timeout, command, template) is provided. The `default_config` function returns an empty TOML table.

### Edge Definition

```rust
/// An edge connecting two nodes in the execution graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    /// Source node ID.
    pub from: NodeId,
    /// Target node ID.
    pub to: NodeId,
    /// Optional condition for this edge to fire.
    #[serde(default)]
    pub condition: Option<EdgeCondition>,
}
```

### EdgeCondition (Type-Level Conditions)

These are the simple, type-level conditions defined on the `Edge` type itself:

```rust
/// Condition that gates execution along an edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum EdgeCondition {
    /// Edge fires only if the source node succeeded.
    Success,
    /// Edge fires only if the source node failed.
    Failure,
    /// Edge fires only if the named output equals the given value.
    OutputEquals {
        key: String,
        value: String,
    },
    /// Edge always fires (unconditional dependency).
    Always,
}
```

### GraphMetadata

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Human-readable name of the graph.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional version string.
    pub version: Option<String>,
    /// Arbitrary key-value annotations.
    pub labels: HashMap<String, String>,
}
```

Labels are used for metadata that does not affect execution -- team ownership, priority classification, source attribution (e.g., `"source": "plan-converter"` for graphs generated by the plan-to-graph converter).

### NodeOutput (Inter-Node Communication)

```rust
/// Output produced by a single node execution, used for condition evaluation
/// and inter-cell communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOutput {
    /// The node that produced this output.
    pub node_id: NodeId,
    /// Execution status.
    pub status: NodeOutputStatus,
    /// Structured output data (JSON).
    pub data: serde_json::Value,
    /// Error message if status is `Failed` or `Skipped`.
    pub error: Option<String>,
    /// Tokens consumed during execution.
    pub tokens_used: u64,
    /// Estimated cost in USD.
    pub cost_usd: f64,
    /// Wall-clock duration of execution.
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeOutputStatus {
    Success,
    Failed,
    Skipped,
}
```

NodeOutput carries both the structured data result (`serde_json::Value`) and resource consumption metadata (tokens, cost, duration). This allows downstream nodes and the budget tracker to make informed decisions. The type provides convenience constructors:

```rust
NodeOutput::success(node_id, data)   // Success status, data payload
NodeOutput::failed(node_id, error)   // Failed status, error string
NodeOutput::skipped(node_id, reason) // Skipped status, reason string
```

### GraphError

```rust
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GraphError {
    #[error("duplicate node: `{0}`")]
    DuplicateNode(NodeId),
    #[error("node not found: `{0}`")]
    NodeNotFound(NodeId),
    #[error("cycle detected in graph")]
    CycleDetected,
    #[error("loader error: {0}")]
    LoaderError(String),
    #[error("unknown cell type: `{0}`")]
    UnknownCellType(String),
    #[error("node '{node_id}' failed: {reason}")]
    NodeFailed { node_id: String, reason: String },
    #[error("edge references unknown node '{node_id}'")]
    InvalidEdge { node_id: String },
    #[error("budget exceeded: {reason}")]
    BudgetExceeded { reason: String },
    #[error("condition evaluation failed for edge {from} -> {to}: {reason}")]
    ConditionError { from: String, to: String, reason: String },
    #[error("invalid graph definition: {reason}")]
    InvalidGraph { reason: String },
}
```

---

## 8. Edge Conditions: Conditional Routing

Edges in a roko-graph can carry conditions that control whether the downstream node executes. This is how branching, error handling, and conditional workflows are expressed.

### EdgeCondition (on the Edge type)

The four `EdgeCondition` variants defined in `types.rs`:

| Variant | Fires When | Use Case |
|---------|-----------|----------|
| `Always` | Always, regardless of upstream outcome | Unconditional dependency |
| `Success` | Upstream node succeeded | Happy-path continuation |
| `Failure` | Upstream node failed | Error-handling branch |
| `OutputEquals { key, value }` | Upstream output field equals a value | Content-based routing |

### TOML Syntax for Edge Conditions

```toml
# Unconditional (default when condition is omitted)
[[edges]]
from = "analyze"
to = "report"

# Success-only
[[edges]]
from = "compile"
to = "test"
[edges.condition]
type = "success"

# Failure-only
[[edges]]
from = "compile"
to = "fix_errors"
[edges.condition]
type = "failure"

# Output-based routing
[[edges]]
from = "classify"
to = "handle_urgent"
[edges.condition]
type = "output_equals"
key = "priority"
value = "urgent"
```

The TOML loader uses serde's `#[serde(tag = "type")]` attribute on a `RawEdgeCondition` enum with lowercase aliases, so `type = "success"` maps to `EdgeCondition::Success`. The raw conditions are converted to `EdgeCondition` via a `From` impl.

---

## 9. Condition System: Deep-Path Field Evaluation

Beyond the simple `EdgeCondition` on edges, roko-graph has a richer `Condition` system in `crates/roko-graph/src/condition.rs` that supports field path resolution, comparison operators, and cross-type value comparison (JSON output vs. TOML expected values).

### Condition Enum
_Source: `crates/roko-graph/src/condition.rs`_

```rust
/// Comparison operators for `When` conditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareOp {
    Eq,        // Field equals value
    Ne,        // Field does not equal value
    Gt,        // Greater than (numeric)
    Gte,       // Greater than or equal
    Lt,        // Less than (numeric)
    Lte,       // Less than or equal
    Contains,  // String substring or array element
}

/// Condition that must be satisfied for an edge to be traversed.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    /// Always traverse this edge (default).
    #[default]
    Always,
    /// Only traverse if the source node succeeded.
    OnSuccess,
    /// Only traverse if the source node failed.
    OnFailure,
    /// Traverse if a field in the node's output data matches a condition.
    When {
        /// JSON pointer path into the node output data (e.g. "score" or "result.status").
        field: String,
        /// Comparison operator.
        op: CompareOp,
        /// Value to compare against.
        value: toml::Value,
    },
}
```

Note: The `Condition` enum is separate from `EdgeCondition`. `EdgeCondition` is used in the `Edge` type for TOML graph definitions. `Condition` is the richer evaluation system with `When` support and `CompareOp`. The `Condition` enum uses `#[serde(tag = "type", rename_all = "snake_case")]` for serialization, and `Always` is the `#[default]` variant.

### Field Path Resolution

The `When` condition supports dot-separated paths into nested JSON objects and arrays:

```rust
/// Resolve a dot-separated field path into a JSON value.
fn resolve_field<'a>(data: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    let mut current = data;
    for segment in path.split('.') {
        match current {
            serde_json::Value::Object(map) => {
                current = map.get(segment)?;
            }
            serde_json::Value::Array(arr) => {
                let idx: usize = segment.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(current)
}
```

This means conditions can target deeply nested output data:

```toml
# Route based on a nested field
[edges.condition]
type = "when"
field = "result.analysis.confidence"
op = "Gte"
value = 0.8

# Route based on an array element
[edges.condition]
type = "when"
field = "tags.0"
op = "Eq"
value = "critical"
```

### Cross-Type Comparison

The `compare_values` function handles the type mismatch between JSON output values (from `serde_json::Value`) and TOML expected values (from `toml::Value`) transparently:

- **String equality**: JSON string vs. TOML string -- direct comparison
- **Numeric comparison**: JSON number vs. TOML integer or float. Both are coerced to `f64` via `as_f64()` / `as_integer()` / `as_float()`. Float equality uses `f64::EPSILON` tolerance. Ordering uses `partial_cmp`.
- **Boolean equality**: JSON bool vs. TOML bool -- direct comparison
- **Contains**: JSON string substring check (`haystack.contains(needle)`) or JSON array element membership (`arr.iter().any(|item| values_equal(item, expected))`)
- **Missing fields**: If `resolve_field` returns `None`, the condition evaluates to `false`.

### Evaluation Logic

```rust
/// Evaluate a condition against a node's output.
/// Returns `true` if the edge should be traversed, `false` otherwise.
pub fn evaluate(condition: &Condition, node_output: &NodeOutput) -> bool {
    match condition {
        Condition::Always => true,
        Condition::OnSuccess => node_output.status.is_success(),
        Condition::OnFailure => node_output.status.is_failed(),
        Condition::When { field, op, value } => evaluate_when(field, op, value, node_output),
    }
}
```

A key detail: **Skipped nodes satisfy neither OnSuccess nor OnFailure**. From the test suite:

```rust
#[test]
fn skipped_node_is_neither_success_nor_failure() {
    let output = NodeOutput::skipped("n1", "budget exceeded");
    assert!(!evaluate(&Condition::OnSuccess, &output));
    assert!(!evaluate(&Condition::OnFailure, &output));
}
```

This means edges from a skipped node will only fire if the condition is `Always`, which prevents cascading execution through error-handling paths when the real problem is budget exhaustion.

---

## 10. TOML Loader: Declarative Graph Definitions

Graphs are defined in TOML files and loaded at runtime. The loader in `crates/roko-graph/src/loader.rs` parses the TOML into raw intermediate types (`RawGraphFile`, `RawNode`, `RawEdge`, `RawEdgeCondition`), then constructs the validated `Graph` struct.

### TOML Schema

A graph TOML file has three sections:

```toml
# Required: graph metadata
[graph]
name = "ci-pipeline"              # Required: human-readable name
description = "Compile then test" # Optional: description
version = "1.0.0"                 # Optional: semver string
[graph.labels]                    # Optional: arbitrary key-value annotations
team = "platform"
priority = "high"

# Required: at least one node
[[nodes]]
id = "compile"                    # Required: unique string ID
cell_type = "gate.compile"        # Required: registry lookup key
inputs = []                       # Optional: named input ports
outputs = ["artifact"]            # Optional: named output ports
[nodes.config]                    # Optional: TOML table passed to CellFactory
workspace = "."
timeout_secs = 300

[[nodes]]
id = "test"
cell_type = "gate.test"
inputs = ["compile.artifact"]
outputs = ["report"]

# Optional: edges define dependencies
[[edges]]
from = "compile"                  # Required: source node ID (must exist)
to = "test"                       # Required: target node ID (must exist)
[edges.condition]                 # Optional: condition for edge activation
type = "success"
```

### Loader Functions

```rust
/// Load a graph from a TOML string.
pub fn load_from_str(toml_str: &str) -> Result<Graph, GraphError>;

/// Load a graph from a TOML file on disk.
pub fn load_from_file(path: &Path) -> Result<Graph, GraphError>;
```

### Internal Deserialization Types

The loader uses private serde-derived types for parsing:

```rust
struct RawGraphFile {
    graph: RawGraphMeta,
    nodes: Vec<RawNode>,   // defaults to empty vec
    edges: Vec<RawEdge>,   // defaults to empty vec
}

#[serde(tag = "type")]
enum RawEdgeCondition {
    #[serde(alias = "success")] Success,
    #[serde(alias = "failure")] Failure,
    #[serde(alias = "always")]  Always,
    #[serde(alias = "output_equals")] OutputEquals { key: String, value: String },
}
```

The `alias` attributes allow both capitalized and lowercase `type` values in TOML (e.g., `type = "success"` or `type = "Success"`).

### Validation During Loading

The loader performs structural validation as it builds the graph:

1. **Duplicate node detection**: `Graph::add_node()` returns `GraphError::DuplicateNode` if a node ID already exists.
2. **Dangling edge detection**: `Graph::add_edge()` returns `GraphError::NodeNotFound` if either the `from` or `to` node ID does not exist in the graph.
3. **TOML parse errors**: Invalid TOML syntax produces `GraphError::LoaderError`.

Cycle detection is NOT performed during loading -- that is the responsibility of `topo::topological_order()`, which is called by the engine before execution.

---

## 11. Topological Sort and Dependency Resolution

The `topo` module in `crates/roko-graph/src/topo.rs` provides topological sorting, cycle detection, and structural analysis of graphs.

### Topological Sort
_Source: `crates/roko-graph/src/topo.rs`_

```rust
/// Perform a topological sort on the graph, returning node IDs in execution order.
/// Nodes with no dependencies come first; nodes that depend on others come later.
///
/// Returns `GraphError::CycleDetected` if the graph contains a cycle.
pub fn topological_order(graph: &Graph) -> Result<Vec<NodeId>, GraphError> {
    let sorted = toposort(&graph.inner, None)
        .map_err(|_| GraphError::CycleDetected)?;
    Ok(sorted.into_iter().map(|idx| graph.inner[idx].id.clone()).collect())
}
```

The implementation delegates to petgraph's `toposort` function [8], which uses a DFS-based algorithm (not Kahn's BFS-based algorithm, despite the document originally stating so -- petgraph implements `Topo` via a depth-first post-order traversal, reversing the result). Both approaches run in O(V + E) time. If the sort cannot visit all nodes, the graph contains a cycle and the function returns `Err(GraphError::CycleDetected)`.

### Dependency Queries

```rust
/// Return the immediate dependencies (predecessors) of a node.
pub fn dependencies(graph: &Graph, node_id: &str) -> Vec<NodeId>;

/// Return the immediate dependents (successors) of a node.
pub fn dependents(graph: &Graph, node_id: &str) -> Vec<NodeId>;
```

Both functions use `graph.inner.neighbors_directed(idx, Direction::Incoming)` and `Direction::Outgoing` respectively. If the node ID does not exist in the graph, they return an empty vector.

### Structural Analysis

```rust
/// Check if the graph is a valid DAG (no cycles).
pub fn is_dag(graph: &Graph) -> bool;

/// Return nodes that have no incoming edges (root/entry nodes).
pub fn root_nodes(graph: &Graph) -> Vec<NodeId>;

/// Return nodes that have no outgoing edges (leaf/terminal nodes).
pub fn leaf_nodes(graph: &Graph) -> Vec<NodeId>;
```

`root_nodes` and `leaf_nodes` iterate over the `node_map` and filter by checking whether `neighbors_directed(..., Direction::Incoming).next().is_none()` or `neighbors_directed(..., Direction::Outgoing).next().is_none()`, respectively.

These functions are used by:
- The engine for validation (`is_dag`)
- The plan converter for identifying entry and exit points (`root_nodes`, `leaf_nodes`)
- The hot graph system for determining which nodes receive initial input

### Graph Shape Examples from Tests

**Linear chain** (A -> B -> C):
```
Topological order: [A, B, C]
Root nodes: [A]
Leaf nodes: [C]
```

**Diamond** (A -> B, A -> C, B -> D, C -> D):
```
Topological order: [A, B, C, D] (B and C may be in either order)
Root nodes: [A]
Leaf nodes: [D]
Constraint: A before B, A before C, B before D, C before D
```

**Cycle** (A -> B, B -> A):
```
Result: Err(GraphError::CycleDetected)
```

---

## 12. The GraphEngine: Sequential Execution

The `GraphEngine` in `crates/roko-graph/src/engine.rs` is the runtime that takes a `Graph` and a `CellRegistry`, topologically sorts the nodes, and executes each cell sequentially.

### Engine Structure

```rust
/// The graph execution engine. Holds a graph and registry, executes nodes
/// sequentially in topological order.
pub struct GraphEngine {
    graph: Graph,
    registry: CellRegistry,
}

impl GraphEngine {
    pub const fn new(graph: Graph, registry: CellRegistry) -> Self;
    pub async fn execute(&self, ctx: &CellContext) -> Result<GraphOutput, GraphError>;
    pub fn validate(&self) -> Vec<String>;
}
```

### Execution Algorithm

The `execute()` method implements the core execution loop:

```
1. Topological sort the graph -> ordered list of node IDs
2. Initialize:
   - outputs: HashMap<NodeId, Vec<Engram>> (stores each node's output engrams)
   - failed_nodes: HashSet<NodeId> (tracks failed and skipped nodes)
   - results: Vec<NodeResult> (per-node execution records)
3. For each node in topological order:
   a. Check if any upstream dependency failed -> skip this node (add to failed_nodes)
   b. Instantiate the Cell from the registry using the node's cell_type and config
   c. Gather input engrams from all upstream nodes' outputs
   d. Execute the cell with the gathered inputs and context
   e. On success: store output engrams, record NodeResult(Complete)
   f. On failure: add to failed_nodes, record NodeResult(Failed) with error message
4. Compute overall success: all nodes must have status Complete
5. Return GraphOutput with per-node results and total duration
```

### Key Implementation Details

**Failed ancestor propagation** -- When a node fails, all its transitive dependents are marked as `Skipped`, not `Failed`. The engine checks direct predecessors (incoming neighbors) against the `failed_nodes` set. Because nodes are processed in topological order, if a predecessor was skipped due to its own predecessor failing, it is already in the `failed_nodes` set, so the skip propagates transitively:

```rust
fn has_failed_ancestor(&self, node_id: &str, failed: &HashSet<NodeId>) -> bool {
    use petgraph::Direction;
    let Some(&idx) = self.graph.node_map.get(node_id) else {
        return false;
    };
    for pred_idx in self.graph.inner
        .neighbors_directed(idx, Direction::Incoming)
    {
        let pred_id = &self.graph.inner[pred_idx].id;
        if failed.contains(pred_id) {
            return true;
        }
    }
    false
}
```

**Input gathering** -- A node receives the concatenation of all output engrams from all its upstream nodes:

```rust
fn gather_inputs(&self, node_id: &str, outputs: &HashMap<NodeId, Vec<Engram>>)
    -> Vec<Engram>
{
    use petgraph::Direction;
    let Some(&idx) = self.graph.node_map.get(node_id) else {
        return vec![];
    };
    let mut input = Vec::new();
    for pred_idx in self.graph.inner
        .neighbors_directed(idx, Direction::Incoming)
    {
        let pred_id = &self.graph.inner[pred_idx].id;
        if let Some(engrams) = outputs.get(pred_id) {
            input.extend(engrams.iter().cloned());
        }
    }
    input
}
```

### GraphOutput: Execution Results

```rust
pub struct GraphOutput {
    /// Name of the graph that was executed.
    pub graph_name: String,
    /// Whether the entire graph completed successfully (all nodes Complete).
    pub success: bool,
    /// Per-node execution results in topological order.
    pub node_results: Vec<NodeResult>,
    /// Total wall-clock duration for the full graph execution.
    pub total_duration: Duration,
}

pub struct NodeResult {
    pub node_id: NodeId,
    pub cell_type: String,
    pub status: NodeStatus,     // Pending | Running | Complete | Failed | Skipped
    pub duration: Duration,
    pub error: Option<String>,
    pub output_count: usize,
}
```

GraphOutput provides a `summary()` method for human-readable display:

```
Graph: ci-pipeline
Status: SUCCESS
Duration: 4.2s
Nodes: 4

  [complete] compile (gate.compile) (1.2s)
  [complete] lint (gate.clippy) (0.8s)
  [complete] test (gate.test) (2.1s)
  [complete] report (llm_call) (0.1s)
```

### Validation

The `validate()` method checks the graph without executing it:

```rust
pub fn validate(&self) -> Vec<String> {
    let mut issues = Vec::new();

    // Check for cycles
    if topological_order(&self.graph).is_err() {
        issues.push("graph contains a cycle".to_string());
    }

    // Check all node cell types are registered
    for (node_id, idx) in &self.graph.node_map {
        let node = &self.graph.inner[*idx];
        if !self.registry.contains(&node.cell_type) {
            issues.push(format!(
                "node '{}' references unknown cell type '{}'",
                node_id, node.cell_type
            ));
        }
    }

    issues
}
```

---

## 13. Budget Tracking System

The `BudgetTracker` in `crates/roko-graph/src/budget.rs` enforces resource limits during graph execution across three dimensions: **tokens**, **cost**, and **wall-clock time**.

### BudgetTracker Architecture
_Source: `crates/roko-graph/src/budget.rs`_

```rust
/// Tracks resource consumption during graph execution and enforces limits.
#[derive(Debug)]
pub struct BudgetTracker {
    /// Total tokens consumed so far.
    tokens_used: AtomicU64,
    /// Total cost in USD (stored as microdollars for atomics).
    cost_microdollars: AtomicU64,
    /// When execution started.
    start_time: Instant,
    /// Configured limits.
    limits: BudgetLimits,
    /// Detailed per-node cost breakdown (for reporting).
    breakdown: Mutex<Vec<NodeCost>>,
}

/// Configured budget limits extracted from GraphConfig.
#[derive(Debug, Clone)]
pub struct BudgetLimits {
    pub max_tokens: Option<u64>,
    pub max_cost_usd: Option<f64>,
    pub deadline: Option<Duration>,
}

/// Per-node cost record.
#[derive(Debug, Clone)]
pub struct NodeCost {
    pub node_id: String,
    pub tokens: u64,
    pub cost_usd: f64,
    pub duration: Duration,
}
```

### Key Design Decision: Microdollar Storage

Cost is stored as **microdollars** (1 USD = 1,000,000 microdollars) using an `AtomicU64`. This avoids floating-point atomics (which do not exist in the standard library) while maintaining sub-cent precision:

```rust
pub fn record(&self, node_id: &str, tokens: u64, cost_usd: f64, duration: Duration) {
    self.tokens_used.fetch_add(tokens, Ordering::Relaxed);
    let microdollars = (cost_usd * 1_000_000.0) as u64;
    self.cost_microdollars.fetch_add(microdollars, Ordering::Relaxed);
    // ... breakdown recording via parking_lot::Mutex
}

pub fn cost_usd(&self) -> f64 {
    let microdollars = self.cost_microdollars.load(Ordering::Relaxed);
    microdollars as f64 / 1_000_000.0
}
```

**Precision analysis**: At 1 microdollar resolution, the smallest representable cost is $0.000001. For LLM API pricing (typically $0.001-$0.015 per 1K tokens), this provides 3 significant digits below the typical unit price. The `AtomicU64` maximum of 2^64 - 1 microdollars corresponds to approximately $18.4 trillion, which is sufficient for any practical budget.

**Memory ordering**: All atomic operations use `Ordering::Relaxed`. This is correct because the budget tracker is only read for advisory purposes (pre-execution budget checks) and exact ordering between concurrent reads and writes is not required for correctness. The breakdown vector uses a `parking_lot::Mutex` for thread-safe append.

### Budget Check Protocol

Before each node execution, the engine calls `tracker.check()`:

```rust
pub fn check(&self) -> Result<()> {
    // Check token limit
    if let Some(max) = self.limits.max_tokens {
        let used = self.tokens_used.load(Ordering::Relaxed);
        if used >= max {
            return Err(GraphError::BudgetExceeded {
                reason: format!("token limit reached: {used}/{max}"),
            });
        }
    }

    // Check cost limit
    if let Some(max) = self.limits.max_cost_usd {
        let used_usd = self.cost_usd();
        if used_usd >= max {
            return Err(GraphError::BudgetExceeded {
                reason: format!("cost limit reached: ${used_usd:.4}/{max:.4}"),
            });
        }
    }

    // Check deadline
    if let Some(deadline) = self.limits.deadline {
        let elapsed = self.elapsed();
        if elapsed >= deadline {
            return Err(GraphError::BudgetExceeded {
                reason: format!("deadline exceeded: {:.1}s/{:.1}s",
                    elapsed.as_secs_f64(), deadline.as_secs_f64()),
            });
        }
    }

    Ok(())
}
```

### Budget Reporting

The tracker provides introspection methods:

```rust
pub fn tokens_used(&self) -> u64;                  // Total tokens consumed
pub fn cost_usd(&self) -> f64;                     // Total cost in USD
pub fn elapsed(&self) -> Duration;                 // Wall-clock time since start
pub fn remaining_cost_usd(&self) -> Option<f64>;   // Budget remaining (if configured)
pub fn remaining_time(&self) -> Option<Duration>;  // Time remaining (if configured)
pub fn breakdown(&self) -> Vec<NodeCost>;           // Per-node cost breakdown
```

### Graph-Level Configuration

Budget limits are configured via `BudgetLimits`, which can be constructed from a `GraphConfig`:

```rust
pub struct GraphConfig {
    pub max_tokens: Option<u64>,
    pub max_cost_usd: Option<f64>,
    pub deadline: Option<Duration>,
}
```

The tracker is created via `BudgetTracker::from_config(&config)` or `BudgetTracker::with_limits(limits)` for testing.

---

## 14. Hot Graphs: Tick-Driven Resident Execution

A **Hot Graph** is a graph that stays resident in memory and re-executes on a periodic tick, persisting state between ticks. This is the mechanism for monitoring workflows, periodic checks, and the cognitive loop.

### Design Reference

From `docs/v2/03-GRAPH.md`:

> **Hot Graphs** stay resident and re-fire per tick. The Workflow/Activity split separates deterministic orchestration from non-deterministic execution for replay.

From `docs/v2-depth/05-execution-engine/cognitive-loop-as-graph.md`:

> The 7-step cognitive loop (SENSE, ASSESS, COMPOSE, ACT, VERIFY, PERSIST/BROADCAST, REACT) is a Hot Graph -- a resident Graph that re-fires on each tick of the Agent's adaptive clock, with state retained between ticks. This is not a metaphor or a "conceptual mapping." The claim is concrete.

### HotPolicy Configuration
_Source: `crates/roko-graph/src/hot.rs`_

```rust
/// Policy controlling Hot Graph tick behavior.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HotPolicy {
    /// How long to wait between ticks (ms). 0 = run as fast as possible.
    pub tick_interval_ms: u64,
    /// Stop after this many ticks. None = run until cancelled.
    pub max_ticks: Option<u64>,
    /// If true, persist cell output state between ticks so cells can
    /// resume from their previous output.
    pub persist_tick_state: bool,
}
```

The default policy uses `tick_interval_ms = 1000`, `max_ticks = None`, and `persist_tick_state = false`.

### HotGraphHandle: Control Interface

```rust
/// A running Hot Graph instance.
pub struct HotGraphHandle {
    /// Cancellation token -- call `.cancel()` to stop the tick loop.
    cancel: CancellationToken,
    /// Monotonic tick counter (incremented after each completed tick).
    tick: Arc<AtomicU64>,
    /// Most recent graph output (from the last completed tick).
    last_output: Arc<parking_lot::Mutex<Option<GraphOutput>>>,
    /// Background task handle (taken by `wait`).
    join_handle: parking_lot::Mutex<Option<JoinHandle<()>>>,
}

impl HotGraphHandle {
    pub fn cancel(&self);                        // Request cancellation
    pub fn tick_count(&self) -> u64;             // Completed tick count
    pub fn last_output(&self) -> Option<GraphOutput>; // Latest output
    pub async fn wait(&self);                    // Wait for completion
    pub fn is_running(&self) -> bool;            // Check if still running
}
```

### Tick Loop Implementation

The `start_hot()` function spawns a tokio task that runs the graph repeatedly:

```rust
pub fn start_hot(
    graph: Graph,
    registry: CellRegistry,
    policy: HotPolicy,
    parent_cancel: Option<CancellationToken>,
) -> HotGraphHandle
```

The tick loop:

```
1. Create GraphEngine from graph + registry
2. Loop:
   a. Check cancellation token -> break if triggered
   b. Check max_ticks limit -> break if reached
   c. Execute full graph via engine.execute()
   d. On success: store output in shared Mutex, increment tick counter
   e. On error: log error and break (FailFast policy for hot graphs)
   f. If tick_interval_ms > 0: tokio::select! between sleep and cancellation
   g. If tick_interval_ms == 0: yield_now() then check cancellation
3. Log total ticks and stop
```

### Cancellation Protocol

Hot graphs support clean shutdown via `CancellationToken`. Parent cancellation tokens can be passed in so that shutting down the engine cancels all hot graphs:

```rust
let handle = start_hot(graph, registry, policy, Some(engine_cancel.clone()));
```

Between ticks, the loop checks for cancellation during sleep using `tokio::select!`:

```rust
if policy.tick_interval_ms > 0 {
    tokio::select! {
        () = tokio::time::sleep(sleep_dur) => {}
        () = cancel_clone.cancelled() => {
            info!("hot graph cancelled during sleep");
            break;
        }
    }
} else {
    tokio::task::yield_now().await;
    if cancel_clone.is_cancelled() {
        break;
    }
}
```

### Use Cases for Hot Graphs

| Use Case | tick_interval_ms | max_ticks | persist_tick_state |
|----------|-----------------|-----------|-------------------|
| Monitoring dashboard | 30000 (30s) | None | true |
| Periodic health check | 60000 (1min) | None | false |
| Cognitive loop | 0 (as fast as possible) | None | true |
| One-shot test | 0 | Some(1) | false |
| Batch processing (10 iterations) | 0 | Some(10) | true |

---

## 15. Plan-to-Graph Conversion Pipeline

The `convert` module in `crates/roko-graph/src/convert.rs` transforms roko's task plan format (`tasks.toml`) into a `Graph` that can be executed by the engine. This is the bridge between the plan-driven orchestrator and the graph execution engine.

### Conversion Functions

```rust
/// Convert a loaded plan into a Graph ready for Engine execution.
pub fn plan_to_graph(
    plan_id: &str,
    plan_dir: &str,
    tasks: &[(String, PlanTaskInfo)],
    max_parallel: u32,
) -> Result<Graph, GraphError>;

/// Convenience: convert and return entry/exit node IDs alongside the graph.
pub fn plan_to_graph_with_endpoints(
    plan_id: &str,
    plan_dir: &str,
    tasks: &[(String, PlanTaskInfo)],
    max_parallel: u32,
) -> Result<(Graph, Vec<String>, Vec<String>), GraphError>;
```

### PlanTaskInfo: Boundary Type

```rust
/// Minimal task information needed by the converter.
/// Crate-boundary-safe struct that does not depend on roko-cli types.
pub struct PlanTaskInfo {
    pub title: String,
    pub description: Option<String>,
    pub role: Option<String>,           // "implementer", "researcher", etc.
    pub tier: String,                   // "mechanical", "focused", "architectural"
    pub model_hint: Option<String>,     // e.g., "claude-sonnet-4-20250514"
    pub files: Vec<String>,             // Files this task modifies
    pub depends_on: Vec<String>,        // Task IDs in the same plan
    pub depends_on_plan: Vec<String>,   // Cross-plan dependencies (skipped)
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub domain: Option<String>,         // "coding", "research", etc.
    pub sequence: usize,                // Definition order index
    pub full_config_json: serde_json::Value, // Full serialized task config
}
```

### Conversion Algorithm

```
Phase 1 -- Build known ID set:
  Collect all task IDs into a HashSet for dependency validation.

Phase 2 -- Add Nodes:
  For each task in the plan:
    Build a TOML config table from the PlanTaskInfo fields
    (plan_id, plan_dir, title, role, tier, model_hint, domain, timeout_secs,
     max_retries, sequence, task_def_json, files).
    Create a Node with:
      - id = task ID
      - cell_type = "task-executor"
      - config = the built TOML table

Phase 3 -- Add Edges:
  For each task's depends_on list:
    If the dependency exists in the known ID set:
      Add edge from dependency -> task (no condition)
    If the dependency is missing:
      Return Err(GraphError::InvalidGraph) with descriptive message

  For each task's depends_on_plan list:
    Log a warning (cross-plan deps are out of scope for single-graph conversion)

Phase 4 -- Validate:
  Run topological_order() to detect cycles
  If cycle detected: return Err(GraphError::CycleDetected)
```

### Metadata Propagation

The converter stores plan metadata in graph labels for traceability:

```rust
labels.insert("source".to_string(), "plan-converter".to_string());
labels.insert("plan_id".to_string(), plan_id.to_string());
labels.insert("plan_dir".to_string(), plan_dir.to_string());
labels.insert("max_parallel".to_string(), max_parallel.to_string());
```

### Example: Diamond Dependency Conversion

Given a plan with tasks:
```
T1 (no deps) -> T2 (depends on T1) -> T4 (depends on T2, T3)
              -> T3 (depends on T1) ---^
```

The converter produces:
```
Graph: 4 nodes, 4 edges
Entry nodes: [T1]
Exit nodes: [T4]
```

This is validated in the test suite:

```rust
#[test]
fn convert_diamond_dependencies() {
    let tasks = vec![
        make_task("T1", &[]),
        make_task("T2", &["T1"]),
        make_task("T3", &["T1"]),
        make_task("T4", &["T2", "T3"]),
    ];
    let (graph, entries, exits) =
        plan_to_graph_with_endpoints("diamond", "/tmp", &tasks, 2).unwrap();
    assert_eq!(graph.node_count(), 4);
    assert_eq!(graph.edge_count(), 4);
    assert_eq!(entries, vec!["T1"]);
    assert_eq!(exits, vec!["T4"]);
}
```

---

## 16. Built-in Cell Implementations

The `cells/` subdirectory contains concrete Cell implementations that ship with the crate.

### AgentCell: LLM Dispatch
_Source: `crates/roko-graph/src/cells/agent.rs`_

The AgentCell wraps LLM dispatch for use in graph execution. It sends a prompt to an LLM backend and returns the response as a node output.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCellConfig {
    pub model: String,               // e.g., "claude-sonnet-4-20250514" (default)
    pub provider: String,            // e.g., "anthropic" (default)
    pub system_prompt: String,       // default: empty
    pub tools: Vec<String>,          // default: empty
    pub max_response_tokens: u32,    // default: 4096
    pub temperature: f32,            // default: 0.7
}

pub struct AgentCell {
    config: AgentCellConfig,
    dispatcher: Box<dyn AgentDispatcher>,
}

/// Trait for LLM dispatch -- abstracts the actual API call.
#[async_trait]
pub trait AgentDispatcher: Send + Sync {
    async fn dispatch(
        &self, model: &str, provider: &str, system_prompt: &str,
        user_message: &str, tools: &[String], max_tokens: u32, temperature: f32,
    ) -> Result<AgentResponse, String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub text: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: f64,
}
```

**Note on the Cell trait**: `AgentCell` implements the `Cell` trait but also provides its own `execute` method that takes `node_id: &str` and `inputs: &[NodeOutput]` instead of `Vec<Engram>`. The `Cell::execute` implementation passes through input engrams. The more detailed `execute` method is used when the cell is invoked directly with `NodeOutput` context.

**Input construction**: The `build_user_message` method constructs the user message from upstream node outputs. If an upstream output has a `"text"` field in its data, that text is used directly. Otherwise, the entire data JSON is serialized with `serde_json::to_string_pretty`. Multiple upstream outputs are joined with `"\n\n---\n\n"` separators. Only successful upstream outputs are included.

**Testing support**: `MockAgentDispatcher` (always succeeds with configurable text and token counts) and `FailingAgentDispatcher` (always fails with a configurable error message) are provided for testing without real API calls.

### ComposeCell: Template Substitution
_Source: `crates/roko-graph/src/cells/compose.rs`_

The ComposeCell assembles prompts from templates with `{{variable}}` placeholders.

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComposeCellConfig {
    pub template: String,                       // e.g., "Hello, {{name}}!"
    pub variables: HashMap<String, String>,      // Static substitutions
}
```

Variable resolution order:
1. Static variables from config
2. Upstream node outputs (node ID becomes the variable name, with text extracted from the `"text"` field or serialized JSON)
3. Special `{{inputs}}` variable: all successful upstream outputs joined with `"\n\n"`
4. Unresolved placeholders are left as-is (e.g., `{{unknown}}` remains in the output)

Only successful upstream outputs contribute to variables -- failed and skipped outputs are excluded.

```toml
[[nodes]]
id = "compose"
cell_type = "compose"
[nodes.config]
template = "Context: {{research}}\n\nPlease implement based on the above."
[nodes.config.variables]
project = "IronClaw"
```

### GraduationCell: Pulse-to-Signal Promotion
_Source: `crates/roko-graph/src/cells/graduation.rs`_

The GraduationCell is a domain-specific cell that evaluates Pulses (ephemeral events on the roko Bus) against graduation policies and promotes qualifying ones to durable Signals (Engrams). It implements both the `Cell` trait and the `React` trait from roko-core.

Key behavior:
- Evaluates each Pulse against configured `GraduationPolicy` entries via `GraduationConfig::should_graduate()`
- `never` policy overrides `always` policy for the same topic
- Sampling uses `pulse.seq` (not a random counter) for deterministic behavior across restarts
- Graduated Signals carry audit tags: `pulse_topic` and `pulse_seq`
- The `Cell::execute` implementation passes through input engrams unchanged; the real graduation work happens via `decide_with_pulses()` when the Bus delivers pulses
- A telemetry counter (`AtomicU64`) tracks total pulses processed

### TaskExecutorCell: Plan Task Stub
_Source: `crates/roko-graph/src/cells/task_executor.rs`_

A placeholder cell for plan-to-graph converted tasks. Currently operates in dry-run mode, returning synthetic output engrams without real LLM dispatch. The live mode (delegating to the Runner v2 agent dispatch path) is planned for when the Engine replaces Runner v2.

```rust
pub struct TaskExecutorCell {
    pub dry_run: bool,  // default: true
}
```

In dry-run mode, it extracts a task label from the first input engram's body text (truncated to 60 chars) and returns a synthetic `Kind::AgentOutput` engram with body `"task-output:dry-run:{label}"`. Live mode currently falls back to the same behavior with a warning log.

### ShellCell: Command Execution (Engine-internal)

Defined directly in `engine.rs` (not in the `cells/` module), the ShellCell runs a shell command via `tokio::process::Command` and succeeds if exit code is 0:

```rust
struct ShellCell {
    id: &'static str,
    name: &'static str,
    program: &'static str,
    args: &'static [&'static str],
}
```

On failure, the ShellCell uses stderr if non-empty, otherwise stdout, and truncates the error detail to 2000 characters to prevent massive error messages from consuming memory. Errors use `roko_core::error::RokoError::Verify` with the cell name as the gate identifier.

### PassthroughCell: Stubs
_Source: `crates/roko-graph/src/cells/stubs.rs`_

Passes input engrams through unchanged, logging an `info!` trace message with the cell name and input count. Used as a placeholder for unimplemented cognitive loop cells. Each instance carries a `name: String` field so logs indicate which stub was invoked.

---

## 17. Design Concepts from Roko v2 Docs

The roko documentation describes several advanced concepts that go beyond the current implementation and inform the future direction of the graph engine.

### 17.1 Graph as Cell (Fractal Composition)
_Reference: `docs/v2/03-GRAPH.md`, section 1_

> **Design invariant**: A Graph IS a Cell (fractal composition). Any Graph can be embedded as a SubGraph node inside another Graph. The Engine does not distinguish between "top-level" and "nested" Graphs.

This means a complex workflow can be composed from simpler workflows. A "CI pipeline" graph could contain a "build" sub-graph, a "test" sub-graph, and a "deploy" sub-graph, each independently defined and testable. This is analogous to function composition in programming languages -- graphs compose like functions.

### 17.2 Workflow/Activity Split
_Reference: `docs/v2/04-EXECUTION.md`, section 3_

Nodes are classified as either **Workflow** (deterministic -- produces the same output given the same input) or **Activity** (non-deterministic -- LLM calls, HTTP requests, shell commands). During replay/resume, Workflow nodes re-execute; Activity nodes return their recorded output without re-execution. This is directly inspired by Temporal.io's workflow engine [2], which uses the same split to enable durable execution through deterministic replay.

### 17.3 The Unified Task DAG (Cross-Plan Scheduling)
_Reference: `docs/v1/01-orchestration/02-unified-task-dag.md`_

The `UnifiedTaskDag` in the roko orchestrator handles cross-plan scheduling with:
- **GlobalTaskId**: `"plan_id:task_id"` composite keys for uniqueness across plans
- **File-conflict inference**: If two tasks from different plans modify the same files, a dependency edge is added to prevent concurrent execution
- **Wave computation**: Groups of tasks that can run in parallel, respecting `max_wave_width`
- **Critical path estimation**: DP on topological order to find the minimum sequential path length
- **HEFT-like scheduling**: Heterogeneous Earliest Finish Time heuristic [9] for multi-agent dispatch, assigning tasks to agents with different model capabilities

### 17.4 Advanced DAG Optimizations
_Reference: `docs/v1/01-orchestration/02-unified-task-dag.md`, "DAG Optimization Passes"_

The documentation describes several optimization passes inspired by the prior art covered in Section 3:

1. **Task Fusion**: Merge linear chains of single-dependency tasks into compound tasks (inspired by Dask's `fuse()` pass [5])
2. **Speculative Execution**: Schedule backup tasks for stragglers on the critical path (inspired by Spark's speculative execution)
3. **DAG Culling**: Remove tasks not required to produce target outputs (inspired by Dask's `cull()` [5])
4. **Graph Partitioning**: METIS-inspired balanced partitioning for large DAGs (100+ tasks)
5. **Incremental Recomputation**: Adapton/Salsa-inspired dirty/clean propagation [10][11] for partial DAG re-execution

### 17.5 The Cognitive Loop as a Hot Graph
_Reference: `docs/v2-depth/05-execution-engine/cognitive-loop-as-graph.md`_

The 7-step cognitive loop (SENSE, ASSESS, COMPOSE, ACT, VERIFY, PERSIST/BROADCAST, REACT) is implemented as a Hot Graph. Each step is a Cell:

| Step | Cell | Input | Output | Execution Class |
|------|------|-------|--------|-----------------|
| SENSE | SenseCell | CorticalSnapshot | SensedMaterial | Activity |
| ASSESS | AssessCell | SensedMaterial | Assessment | Workflow |
| COMPOSE | ComposeCell | Assessment + Snapshot | ComposedPrompt | Workflow |
| ACT | ActCell | ComposedPrompt | ActionResult | Activity |
| VERIFY | VerifyCell | ActionResult | Verdict | Workflow |
| PERSIST | PersistCell | Verdict | PersistResult | Activity |
| REACT | ReactCell | All above | ReactOutput | Workflow |

The T0 short-circuit handles ~80% of ticks: if ASSESS determines that no action is needed (low relevance/urgency), the loop short-circuits before COMPOSE, avoiding the LLM call entirely. This is zero-cost for idle ticks.

---

## 18. Practical Workflow Examples

This section demonstrates real-world workflows that the DAG engine enables, with complete TOML definitions.

### 18.1 Multi-Step Research Pipeline

A workflow that researches a topic using multiple sources in parallel, then synthesizes the findings:

```toml
[graph]
name = "research-pipeline"
description = "Multi-source research with synthesis"
version = "1.0.0"

[graph.labels]
category = "research"

# Step 1: Parse the research query
[[nodes]]
id = "parse_query"
cell_type = "llm_call"
[nodes.config]
prompt = "Extract key search terms and subtopics from this research request."

# Step 2a: Search documentation (parallel with 2b, 2c)
[[nodes]]
id = "search_docs"
cell_type = "tool_call"
[nodes.config]
tool = "web_fetch"
params = { query_source = "parse_query" }

# Step 2b: Search code repositories
[[nodes]]
id = "search_code"
cell_type = "tool_call"
[nodes.config]
tool = "web_fetch"
params = { query_source = "parse_query" }

# Step 2c: Search academic papers
[[nodes]]
id = "search_papers"
cell_type = "tool_call"
[nodes.config]
tool = "web_fetch"
params = { query_source = "parse_query" }

# Step 3: Synthesize all findings
[[nodes]]
id = "synthesize"
cell_type = "llm_call"
[nodes.config]
prompt = "Synthesize these research findings into a comprehensive summary."

# parse_query fans out to all three searches
[[edges]]
from = "parse_query"
to = "search_docs"

[[edges]]
from = "parse_query"
to = "search_code"

[[edges]]
from = "parse_query"
to = "search_papers"

# All searches fan in to synthesis (only on success)
[[edges]]
from = "search_docs"
to = "synthesize"
[edges.condition]
type = "success"

[[edges]]
from = "search_code"
to = "synthesize"
[edges.condition]
type = "success"

[[edges]]
from = "search_papers"
to = "synthesize"
[edges.condition]
type = "success"
```

This graph executes `parse_query` first, then runs all three search nodes in parallel, then waits for all three to complete before synthesizing. Total time is `parse_query + max(search_docs, search_code, search_papers) + synthesize`, versus the sum of all steps in a linear pipeline.

### 18.2 Code Generation Pipeline with Error Recovery

A workflow for generating code with compilation verification and automatic error correction:

```toml
[graph]
name = "code-gen-pipeline"
description = "Generate code, compile, fix errors if needed"
version = "1.0.0"

[[nodes]]
id = "analyze"
cell_type = "llm_call"
[nodes.config]
prompt = "Analyze the requirements and produce a detailed implementation plan."

[[nodes]]
id = "generate"
cell_type = "llm_call"
[nodes.config]
prompt = "Generate Rust code based on the implementation plan."

[[nodes]]
id = "write_file"
cell_type = "tool_call"
[nodes.config]
tool = "file_write"

[[nodes]]
id = "compile"
cell_type = "gate.compile"

[[nodes]]
id = "fix_errors"
cell_type = "llm_call"
[nodes.config]
prompt = "Fix the compilation errors shown below."

[[nodes]]
id = "test"
cell_type = "gate.test"

[[nodes]]
id = "report"
cell_type = "llm_call"
[nodes.config]
prompt = "Summarize what was built and the test results."

# Happy path: analyze -> generate -> write -> compile -> test -> report
[[edges]]
from = "analyze"
to = "generate"

[[edges]]
from = "generate"
to = "write_file"

[[edges]]
from = "write_file"
to = "compile"

# On compile success -> run tests
[[edges]]
from = "compile"
to = "test"
[edges.condition]
type = "success"

# On compile failure -> fix errors -> rewrite -> recompile
[[edges]]
from = "compile"
to = "fix_errors"
[edges.condition]
type = "failure"

# Test results -> report
[[edges]]
from = "test"
to = "report"
[edges.condition]
type = "success"
```

### 18.3 Morning Standup Automation

A periodic workflow (Hot Graph) for daily standup preparation:

```toml
[graph]
name = "morning-standup"
description = "Daily morning standup automation"

[[nodes]]
id = "check_email"
cell_type = "tool_call"
[nodes.config]
tool = "gmail_check"

[[nodes]]
id = "check_github"
cell_type = "tool_call"
[nodes.config]
tool = "github_notifications"

[[nodes]]
id = "check_calendar"
cell_type = "tool_call"
[nodes.config]
tool = "calendar_today"

[[nodes]]
id = "summarize"
cell_type = "llm_call"
[nodes.config]
prompt = "Summarize these updates into a brief standup report with: done yesterday, doing today, blockers."

# All three checks run in parallel, then summarize waits for all
[[edges]]
from = "check_email"
to = "summarize"

[[edges]]
from = "check_github"
to = "summarize"

[[edges]]
from = "check_calendar"
to = "summarize"
```

---

## 19. IronClaw Integration Architecture

### A. Key Design Decision: Tools vs. Cells

In roko, Cells are the unit of graph execution. In IronClaw, **tools are the universal dispatch mechanism** ("everything goes through tools"). The bridge between these concepts is a `ToolCell` -- a Cell implementation that dispatches through IronClaw's `ToolDispatcher`:

```rust
/// A Cell that dispatches through IronClaw's ToolDispatcher.
/// This preserves IronClaw's audit trail, safety pipeline, and
/// channel-agnostic dispatch while gaining DAG execution.
pub struct ToolCell {
    tool_name: String,
    params_template: serde_json::Value,
}

#[async_trait]
impl Cell for ToolCell {
    fn cell_id(&self) -> &str { &self.tool_name }
    fn cell_name(&self) -> &str { &self.tool_name }
    fn protocols(&self) -> &[&str] { &["ToolDispatch"] }

    async fn execute(
        &self,
        input: Vec<Engram>,
        ctx: &CellContext,
    ) -> Result<Vec<Engram>> {
        // Merge the input data into the params template
        let params = merge_template(&self.params_template, &input);

        // Dispatch through IronClaw's ToolDispatcher
        // This ensures audit trail (ActionRecord), safety pipeline
        // (param validation, sensitive-param redaction, output sanitization),
        // and channel-agnostic execution.
        let result = ctx.dispatcher()
            .dispatch(&self.tool_name, params)
            .await?;

        // Convert ToolOutput to Engram
        Ok(vec![result.into_engram()])
    }
}
```

This approach means:
- Every graph node execution goes through `ToolDispatcher::dispatch()`, satisfying IronClaw's "everything goes through tools" invariant
- The safety pipeline (param validation, redaction, output sanitization) applies to every cell execution
- ActionRecords are created for every node, providing audit trail
- Rate limiting and budget tracking apply uniformly

### B. LLM Cell for IronClaw

```rust
/// A Cell that makes an LLM call through IronClaw's LLM provider system.
pub struct LlmCell {
    model: String,
    provider: String,
    system_prompt: String,
    max_tokens: u32,
    temperature: f32,
}

#[async_trait]
impl Cell for LlmCell {
    fn cell_id(&self) -> &str { "llm_call" }
    fn cell_name(&self) -> &str { "LLM Call" }
    fn protocols(&self) -> &[&str] { &["AgentDispatch"] }

    fn estimated_cost(&self) -> Option<f64> {
        // Rough estimate based on model pricing
        Some(0.01)
    }

    async fn execute(
        &self,
        input: Vec<Engram>,
        ctx: &CellContext,
    ) -> Result<Vec<Engram>> {
        let user_message = build_message_from_engrams(&input);

        // Use IronClaw's LlmProvider for the actual call
        let response = ctx.llm_provider()
            .complete(&self.model, &self.system_prompt, &user_message)
            .await?;

        // Convert to Engram-compatible output
        let output = serde_json::json!({
            "text": response.text,
            "tokens_used": response.input_tokens + response.output_tokens,
            "cost_usd": response.cost_usd,
        });

        Ok(vec![/* engram from output */])
    }
}
```

### C. Integration Points

| IronClaw Module | Integration | Description |
|----------------|-------------|-------------|
| `src/tools/dispatch.rs` | ToolCell | Every cell execution routes through ToolDispatcher |
| `crates/ironclaw_llm/` | LlmCell | LLM calls use the existing multi-provider system |
| `src/agent/` | Job DAGs | Complex jobs expressed as graph workflows |
| `src/workspace/` | Workflow storage | Graph definitions stored in workspace memory |
| `src/channels/` | NotifyCell | Notifications sent through channel system |
| `src/sandbox/` | ShellCell | Shell commands sandboxed per policy |
| `src/estimation/` | BudgetTracker | Budget connects to IronClaw's cost estimation |
| `src/hooks/` | Lifecycle hooks | BeforeToolCall/BeforeOutbound hooks on cell execution |

### D. Job Orchestration
**Where**: `src/agent/` -- replace linear job execution with DAG-based workflows
**How**: Complex jobs that currently run as sequential agent turns could be expressed as DAGs.

Example: "Build and deploy my project"
```
Parse Requirements --> [Generate Code, Write Tests] --> Run Tests --> Build --> Deploy
```
With conditional routing: if tests fail, go back to code generation with error context.

### E. TOML-Defined Workflows
**Where**: `~/.ironclaw/workflows/` or workspace `workflows/` directory
**How**: Users define custom workflows in TOML files. The agent can invoke them by name.

### F. Heartbeat Enhancement via Hot Graphs

IronClaw's current heartbeat system runs simple periodic checks by reading `HEARTBEAT.md`. Hot Graphs generalize this:

```toml
# ~/.ironclaw/workflows/heartbeat.toml
[graph]
name = "enhanced-heartbeat"

[[nodes]]
id = "check_workspace"
cell_type = "tool_call"
[nodes.config]
tool = "memory_search"
params = { query = "recent changes" }

[[nodes]]
id = "check_notifications"
cell_type = "tool_call"
[nodes.config]
tool = "notification_check"

[[nodes]]
id = "evaluate"
cell_type = "llm_call"
[nodes.config]
prompt = "Based on the workspace and notification checks, are there any items requiring attention?"

[[nodes]]
id = "notify"
cell_type = "tool_call"
[nodes.config]
tool = "message"
params = { channel = "primary" }

[[edges]]
from = "check_workspace"
to = "evaluate"

[[edges]]
from = "check_notifications"
to = "evaluate"

# Only notify if the evaluation found something worth reporting
[[edges]]
from = "evaluate"
to = "notify"
[edges.condition]
type = "output_equals"
key = "action_needed"
value = "true"
```

---

## 20. Implementation Plan for IronClaw

### Proposed Crate Structure

```
crates/ironclaw_graph/
├── src/
│   ├── lib.rs         # Public API: Cell trait, graph types, engine, registry
│   ├── cell.rs        # IronClaw Cell trait (adapted from roko-graph, uses serde_json::Value
│   │                  # instead of Engram for interop with IronClaw's tool system)
│   ├── cells/
│   │   ├── tool_cell.rs    # ToolCell: dispatches through ToolDispatcher
│   │   ├── llm_cell.rs     # LlmCell: LLM calls through IronClaw's LlmProvider
│   │   ├── shell_cell.rs   # ShellCell: sandboxed shell commands
│   │   ├── condition_cell.rs # Evaluate a predicate and route
│   │   ├── aggregate_cell.rs # Collect outputs from multiple parents
│   │   └── notify_cell.rs  # Send notification to a channel
│   ├── types.rs       # Graph, Node, Edge, EdgeCondition, NodeOutput, GraphError
│   ├── registry.rs    # IronClaw default registry (registers all IronClaw cell types)
│   ├── engine.rs      # DAG executor (topological execution with budget tracking)
│   ├── budget.rs      # Budget integration (connects to IronClaw's cost estimation)
│   ├── condition.rs   # Condition evaluation (CompareOp, field path resolution)
│   ├── topo.rs        # Topological sort (wraps petgraph)
│   ├── loader.rs      # TOML loader for workflow definitions
│   └── hot.rs         # Hot graph integration with IronClaw's heartbeat system
```

### Implementation Phases

**Phase 1: Core Engine (weeks 1-2)**
- Port `types.rs`, `topo.rs`, `loader.rs`, `condition.rs` from roko-graph with minimal changes
- Replace `Engram` with `serde_json::Value` as the inter-cell data type (IronClaw does not have roko-core's Engram type)
- Implement `GraphEngine` with sequential topological execution
- Add `BudgetTracker` with integration to IronClaw's existing cost estimation

**Phase 2: Cell Implementations (weeks 3-4)**
- Implement `ToolCell` that dispatches through `ToolDispatcher`
- Implement `LlmCell` using `ironclaw_llm::LlmProvider`
- Implement `ShellCell` with sandbox integration (`SandboxPolicy`)
- Implement `NotifyCell` that sends through the channel system
- Register all cells in the default registry

**Phase 3: Workflow Loading and Agent Integration (weeks 5-6)**
- Implement workflow directory scanning (`~/.ironclaw/workflows/` and workspace `workflows/`)
- Add `workflow_run` and `workflow_list` tools to the tool registry
- Connect DAG execution to the agent's job system in `src/agent/`

**Phase 4: Hot Graph Integration (week 7)**
- Implement Hot Graph support for periodic workflows
- Migrate heartbeat to use Hot Graph infrastructure
- Add cancellation integration with IronClaw's shutdown protocol

### Data Type Mapping: roko-graph to IronClaw

| roko-graph Type | IronClaw Equivalent | Notes |
|-----------------|-------------------|-------|
| `Engram` | `serde_json::Value` | IronClaw uses JSON for tool I/O |
| `CellContext` | `WorkflowContext` | Extended with `ToolDispatcher`, `LlmProvider`, `SandboxManager` |
| `roko_core::error::Result` | `crate::error::IronClawError` | Map via `From` impl |
| `Kind`, `Body` | N/A | Not needed; IronClaw tools return structured JSON |
| `Provenance`, `Score` | N/A | GraduationCell is roko-specific |

---

## 21. Complexity Assessment

### Core Engine (adapted from roko-graph)
- **Graph types + loader**: ~500 lines (adapt from roko-graph, replace Engram with JSON)
- **Cell trait + registry**: ~300 lines (simplify from roko-graph)
- **Topological sort**: ~100 lines (direct port, uses petgraph)
- **Execution engine**: ~400 lines (adapt from roko-graph)
- **Budget tracker**: ~200 lines (adapt, integrate with IronClaw's cost estimation)
- **Condition system**: ~200 lines (direct port)
- **Hot graphs**: ~250 lines (adapt, integrate with heartbeat)

### IronClaw-Specific Code
- **ToolCell + LlmCell**: ~400 lines (new, bridges to ToolDispatcher and LlmProvider)
- **ShellCell + NotifyCell**: ~300 lines (new, sandbox + channel integration)
- **IronClaw registry**: ~200 lines (new, registers all IronClaw cell types)
- **Workflow loader**: ~300 lines (new, scans ~/.ironclaw/workflows/)
- **Agent integration**: ~500 lines (moderate, connects to src/agent/ for DAG-based jobs)
- **Heartbeat migration**: ~200 lines (rewrite heartbeat as Hot Graph)

### Total Estimated Effort
- **Adapted roko-graph code**: ~1,950 lines (core engine, mostly reusable with type substitutions)
- **New IronClaw-specific code**: ~1,900 lines
- **Risk**: Medium -- DAG execution introduces new failure modes (cycles, deadlocks, budget exhaustion mid-graph), but the roko-graph crate already handles these with cycle detection, failed-ancestor propagation, and graceful budget termination.
- **Dependencies**: `petgraph` (graph data structure), `toml` (graph definition parsing), `tokio` (async execution), `parking_lot` (fast mutexes for budget tracking)

---

## 22. References

[1] Apache Software Foundation. "Apache Airflow Documentation." https://airflow.apache.org/docs/

[2] Temporal Technologies. "Temporal Workflow Execution Overview." https://docs.temporal.io/workflow-execution

[3] Prefect Technologies. "Prefect Documentation." https://docs.prefect.io/

[4] LangChain. "LangGraph: Multi-Agent Orchestration Framework." https://www.langchain.com/langgraph

[5] Dask Development Team. "Dask: Optimization." https://docs.dask.org/en/stable/optimize.html

[6] E. G. Coffman Jr. and R. L. Graham. "Optimal scheduling for two-processor systems." *Acta Informatica*, 1(3):200-213, 1972.

[7] A. B. Kahn. "Topological sorting of large networks." *Communications of the ACM*, 5(11):558-562, 1962.

[8] petgraph contributors. "petgraph::algo::toposort." https://docs.rs/petgraph/latest/petgraph/algo/fn.toposort.html

[9] H. Topcuoglu, S. Hariri, and M.-Y. Wu. "Performance-effective and low-complexity task scheduling for heterogeneous computing." *IEEE Transactions on Parallel and Distributed Systems*, 13(3):260-274, March 2002.

[10] M. A. Hammer, K. Y. Phang, M. Hicks, and J. S. Foster. "Adapton: Composable, demand-driven incremental computation." *Proceedings of the 35th ACM SIGPLAN Conference on Programming Language Design and Implementation (PLDI)*, 2014.

[11] N. Matsakis. "Salsa: A generic framework for on-demand, incrementalized computation." https://github.com/salsa-rs/salsa

[12] "From Agent Loops to Structured Graphs: A Scheduler-Theoretic Framework for LLM Agent Execution." arXiv:2604.11378, April 2026. https://arxiv.org/abs/2604.11378

[13] "GraphFlow: A Graph-Based Workflow Management for Efficient LLM-Agent Serving." arXiv:2605.22566, 2026. https://arxiv.org/abs/2605.22566

[14] "GRADE: Graph Representation of LLM Agent Dependency and Execution." arXiv:2606.22741, 2026. https://arxiv.org/abs/2606.22741

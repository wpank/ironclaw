# Roko Architecture Overview

**A comprehensive reference for someone seeing this system for the first time.**

> Roko is a Rust toolkit for building agents that build themselves. 30 crates
> plus 3 application binaries, ~727K lines of Rust, ~8,300 tests. It reads
> requirements, generates implementation plans, dispatches LLM agents, verifies
> output with compilation and test gates, persists results as content-addressed
> engrams, and learns from outcomes to improve over time.

---

## Table of Contents

1. [What Is Roko?](#1-what-is-roko)
2. [Vision, Goals, and Design Philosophy](#2-vision-goals-and-design-philosophy)
3. [The Core Mental Model: One Noun, Nine Verbs](#3-the-core-mental-model-one-noun-nine-verbs)
4. [Five-Layer Model (L0-L4)](#4-five-layer-model-l0-l4)
5. [Crate Dependency Graph](#5-crate-dependency-graph)
6. [Crate-by-Crate Reference](#6-crate-by-crate-reference)
7. [Data Flow Through the System](#7-data-flow-through-the-system)
8. [The Universal Cognitive Loop](#8-the-universal-cognitive-loop)
9. [Key Data Types](#9-key-data-types)
10. [Agent Lifecycle and Coordination](#10-agent-lifecycle-and-coordination)
11. [Gate Pipeline and Verification](#11-gate-pipeline-and-verification)
12. [Learning and Self-Improvement](#12-learning-and-self-improvement)
13. [Knowledge and Memory (Neuro)](#13-knowledge-and-memory-neuro)
14. [Affect Engine (Daimon)](#14-affect-engine-daimon)
15. [Offline Consolidation (Dreams)](#15-offline-consolidation-dreams)
16. [Conductor: Reactive Intelligence](#16-conductor-reactive-intelligence)
17. [Code Intelligence (Index)](#17-code-intelligence-index)
18. [Graph Execution Engine](#18-graph-execution-engine)
19. [Key Architectural Decisions](#19-key-architectural-decisions)
20. [Security Model](#20-security-model)
21. [Deployment Model](#21-deployment-model)
22. [Comparison: Roko vs. IronClaw](#22-comparison-roko-vs-ironclaw)
23. [File Path Reference](#23-file-path-reference)

---

## 1. What Is Roko?

Roko is a Rust platform for building autonomous software agents. Its defining
characteristic is **self-hosting**: Roko develops itself. It reads PRDs
(product requirement documents), generates implementation plans as TOML task
DAGs, dispatches LLM-powered agents (Claude, Codex, Gemini, Ollama, and
others) to execute tasks, validates every output through a multi-rung
compilation and testing pipeline, persists verified results as
content-addressed engrams, and feeds outcomes back into learning systems that
improve model selection, prompt assembly, and gate thresholds for the next
run.

The system runs as a single binary (`roko`) with subcommands that cover the
full lifecycle:

```
roko prd idea "Add retry logic to HTTP client"
roko prd draft new "http-retry"
roko prd plan http-retry
roko plan run plans/
roko dashboard
```

At its core, Roko is not a chat wrapper or thin LLM client. It is a complete
orchestration runtime: a typed event bus, a multi-stage gate pipeline, a
self-improving model router, a durable knowledge store, an affect engine, a
code intelligence indexer, and a DAG-based graph execution engine -- all
unified by a single data type and a small set of composable trait contracts.

### Scale

| Metric | Value |
|--------|-------|
| Workspace crates | 30 |
| Application binaries | 3 (mirage-rs, agent-relay, roko-chain-watcher) |
| Total Rust LOC | ~727K |
| Test functions | ~8,300 |
| CLI subcommands | 60+ |
| HTTP control plane routes | ~85 |
| Rust edition | 2024 (minimum rustc 1.85) |

---

## 2. Vision, Goals, and Design Philosophy

### Vision

A system sophisticated enough to improve its own codebase: read a
requirement, plan the implementation, execute it through LLM agents, verify
the result mechanically, persist the outcome, and learn from failures to
do better next time.

### Design Principles

1. **One Noun, Nine Verbs.** Every capability -- agent dispatch, gate
   verification, prompt assembly, model routing, memory retrieval, chain
   participation -- is expressed as one of nine trait operations on the
   universal `Engram` type. No special-case APIs.

2. **Content-Addressed Immutability.** Every engram is identified by a BLAKE3
   hash of its content (kind + body + author + taint + lineage + tags).
   Score, decay, timestamps, and attestations are excluded from the hash --
   they are metadata that can evolve without changing the engram's identity.
   This gives the system a Git-like audit trail.

3. **Wire, Don't Build.** The codebase's historical pattern was
   "built but never connected." The current development discipline is to wire
   existing code into the runtime via CLI subcommands before building
   anything new.

4. **Mechanical Verification Over Human Review.** Every agent output passes
   through a multi-rung gate pipeline (compile, test, clippy, diff, LLM
   judge, property tests, benchmarks) before it is accepted. The pipeline is
   adaptive: threshold learning adjusts pass/fail boundaries based on
   accumulated evidence.

5. **Learning is Structural, Not Fine-Tuning.** Roko does not fine-tune
   LLMs. Instead, it learns by adjusting routing weights (CascadeRouter),
   gate thresholds (EMA adaptation), prompt sections (A/B experiments),
   context pack selection (bandit algorithms), and playbook rules (pattern
   discovery from episodes).

6. **Decay as a First-Class Concept.** Engrams have half-lives. A compiler
   warning decays in hours; a design insight decays over weeks. The substrate
   prunes below-threshold engrams, and the cold store archives them. This
   prevents unbounded knowledge accumulation.

7. **Layered Architecture with Enforced Boundaries.** Every crate declares
   its layer (L0--L4) via `[package.metadata.roko] layer = N`. A CI script
   (`scripts/layer_check.rs`) verifies that no crate depends on a crate at
   a higher layer. This prevents accidental coupling.

---

## 3. The Core Mental Model: One Noun, Nine Verbs

The entire system is built from one universal data type and nine trait
operations:

```
Engram              -- the universal datum: addressable, decaying, scored, traced

  Core Six (defined in roko-core/src/traits.rs):
    Store           -- persist and retrieve engrams
    Score           -- rate along multi-dimensional axes
    Verify          -- check against ground truth
    Route           -- select one candidate from many
    Compose         -- combine under a budget into a new engram
    React           -- watch engram streams and emit interventions

  Extended Three:
    Bus             -- publish/subscribe transport for ephemeral Pulses
    ColdStore       -- archival store for aged-out engrams
    Observe/Connect/Trigger -- peripheral Cell-based protocols
```

### The Six Core Traits

| Trait | Purpose | Example Implementations |
|-------|---------|------------------------|
| `Store` | Persist and query engrams | `MemorySubstrate`, `FileSubstrate`, `HdcSubstrate`, `ChainSubstrate` |
| `Score` | Rate engrams on multi-dimensional axes | `RelevanceScorer`, `RecencyScorer`, `ReputationScorer`, `CatalyticScorer` |
| `Verify` | Verify engrams against ground truth | `CompileGate`, `TestGate`, `ClippyGate`, `DiffGate`, `LlmJudgeGate` |
| `Route` | Select one engram from many candidates | `StaticRouter`, `CascadeRouter`, `LinUCBRouter`, `WeightedRouter` |
| `Compose` | Combine engrams into a new engram under a budget | `PromptComposer`, `ContextAssembler`, `TaskBriefComposer` |
| `React` | Watch engram streams and emit new engrams | Conductor watchers, circuit breaker, episode logging |

### Engram vs. Pulse (Duality)

The system has two complementary data modes:

- **Engram**: durable, content-addressed, stored in a `Store`. Think of it
  as a committed Git blob with metadata.
- **Pulse**: ephemeral, flowing through the `Bus` for real-time reactions.
  Think of it as a message on a pub/sub channel.

Pulses that prove valuable get promoted to Engrams and stored. This two-tier
model separates hot-path real-time data from durable persistence.

---

## 4. Five-Layer Model (L0-L4)

Roko enforces a strict layered architecture. Each crate declares its layer
in `Cargo.toml` via `[package.metadata.roko] layer = N`. A CI-enforced
check (`scripts/layer_check.rs`) ensures no crate depends on a higher
layer.

```
L4  Applications        roko-cli, roko-serve, roko-agent-server, roko-acp
     |
L3  Orchestration       roko-orchestrator, roko-gate, roko-conductor
     |
L2  Capabilities        roko-agent, roko-compose, roko-learn, roko-neuro,
                         roko-daimon, roko-dreams, roko-fs, roko-std,
                         roko-chain, roko-plugin, roko-graph, roko-index,
                         roko-lang-*, roko-mcp-*, roko-demo
     |
L1  Kernel + Runtime    roko-core, roko-runtime
     |
L0  Primitives          roko-primitives
```

### Layer Rules

| Layer | Can Depend On | Purpose |
|-------|---------------|---------|
| L0 | External crates only | Zero-knowledge compute primitives (HDC vectors, tier routing, manifold geometry, TDA) |
| L1 | L0 | The Engram type, six core traits, typed event bus, process supervision |
| L2 | L0, L1 | Concrete implementations: agents, gates, composers, scorers, persistence, knowledge, learning |
| L3 | L0, L1, L2 | Multi-crate orchestration: DAG execution, plan discovery, gate pipeline composition |
| L4 | All layers | User-facing surfaces: CLI binary, HTTP server, per-agent sidecar, editor integration |

---

## 5. Crate Dependency Graph

This graph is derived from actual `Cargo.toml` files as of July 2026.
Arrows point from dependent to dependency.

### ASCII Dependency Map

```
                              L4 — Applications
  ┌─────────────────────────────────────────────────────────────┐
  │                                                             │
  │   roko-cli ──────────┬──────────── roko-serve               │
  │   (main binary)      │            (HTTP ~85 routes)         │
  │       │              │                │                     │
  │       ├── roko-acp   │    roko-agent-server                 │
  │       │   (editor)   │    (per-agent sidecar)               │
  │       │              │                                      │
  └───────┴──────────────┴──────────────────────────────────────┘
               │                    │
               ▼                    ▼
                              L3 — Orchestration
  ┌─────────────────────────────────────────────────────────────┐
  │                                                             │
  │   roko-orchestrator ◄─── roko-gate ◄─── roko-conductor     │
  │   (DAG executor)         (verify)       (watchers)          │
  │       │                     │               │               │
  └───────┴─────────────────────┴───────────────┘               │
               │                                                │
               ▼                                                │
                              L2 — Capabilities                 │
  ┌─────────────────────────────────────────────────────────────┐
  │                                                             │
  │  roko-agent     roko-compose    roko-learn     roko-neuro   │
  │  (LLM backends) (prompts)      (episodes,     (knowledge)  │
  │       │              │          playbooks)          │       │
  │  roko-fs        roko-std       roko-daimon     roko-dreams  │
  │  (JSONL)        (defaults)     (affect)        (offline)    │
  │       │              │              │               │       │
  │  roko-chain     roko-plugin    roko-graph      roko-index   │
  │  (on-chain)     (event src)    (DAG engine)    (code intel) │
  │       │              │              │               │       │
  │  roko-demo      roko-mcp-*    roko-lang-*                   │
  │  (scenario)     (MCP servers)  (lang support)               │
  │                                                             │
  └─────────────────────────────────────────────────────────────┘
               │
               ▼
                         L1 — Kernel + Runtime
  ┌─────────────────────────────────────────────────────────────┐
  │                                                             │
  │   roko-core                    roko-runtime                 │
  │   (Engram + 9 traits           (event bus,                  │
  │    + 30K LOC kernel)            process supervision)        │
  │                                                             │
  └─────────────────────────────────────────────────────────────┘
               │
               ▼
                            L0 — Primitives
  ┌─────────────────────────────────────────────────────────────┐
  │                                                             │
  │   roko-primitives                                           │
  │   (HDC vectors, tier routing, manifold, TDA, sheaf, codebook)│
  │                                                             │
  └─────────────────────────────────────────────────────────────┘
```

### Mermaid Dependency Graph (Detailed)

```mermaid
graph TD
    subgraph L0["L0 — Primitives"]
        primitives["roko-primitives"]
    end

    subgraph L1["L1 — Kernel"]
        core["roko-core"]
        runtime["roko-runtime"]
    end

    subgraph L2["L2 — Capabilities"]
        agent["roko-agent"]
        compose["roko-compose"]
        learn["roko-learn"]
        neuro["roko-neuro"]
        daimon["roko-daimon"]
        dreams["roko-dreams"]
        fs["roko-fs"]
        std["roko-std"]
        chain["roko-chain"]
        plugin["roko-plugin"]
        graph["roko-graph"]
        index["roko-index"]
        lang_rust["roko-lang-rust"]
        lang_ts["roko-lang-typescript"]
        lang_go["roko-lang-go"]
        mcp_stdio["roko-mcp-stdio"]
        mcp_github["roko-mcp-github"]
        mcp_slack["roko-mcp-slack"]
        mcp_scripts["roko-mcp-scripts"]
        mcp_code["roko-mcp-code"]
        demo["roko-demo"]
    end

    subgraph L3["L3 — Orchestration"]
        orchestrator["roko-orchestrator"]
        gate["roko-gate"]
        conductor["roko-conductor"]
    end

    subgraph L4["L4 — Applications"]
        cli["roko-cli"]
        serve["roko-serve"]
        agent_server["roko-agent-server"]
        acp["roko-acp"]
    end

    %% L0 → L1
    core --> primitives
    runtime --> primitives
    runtime --> core
    runtime --> gate

    %% L1 → L2
    agent --> core
    agent --> fs
    agent --> std
    compose --> core
    compose --> agent
    compose --> learn
    compose --> neuro
    learn --> core
    learn --> agent
    learn --> daimon
    learn --> fs
    learn --> primitives
    neuro --> core
    neuro --> fs
    neuro --> agent
    neuro --> learn
    daimon --> core
    dreams --> core
    dreams --> neuro
    dreams --> learn
    dreams --> agent
    dreams --> primitives
    fs --> core
    std --> core
    std --> chain
    chain --> core
    plugin --> core
    graph --> core
    index --> core
    index --> lang_rust
    index --> lang_ts
    index --> lang_go
    lang_rust --> core
    lang_ts --> core
    lang_go --> core
    demo --> chain
    mcp_github --> mcp_stdio
    mcp_slack --> mcp_stdio
    mcp_scripts --> mcp_stdio
    mcp_code --> core
    mcp_code --> index
    mcp_code --> mcp_stdio

    %% L2 → L3
    gate --> core
    gate --> agent
    conductor --> core
    conductor --> learn
    orchestrator --> core
    orchestrator --> agent
    orchestrator --> compose
    orchestrator --> conductor
    orchestrator --> daimon
    orchestrator --> gate
    orchestrator --> learn
    orchestrator --> neuro
    orchestrator --> runtime

    %% L3 → L4
    cli --> core
    cli --> std
    cli --> fs
    cli --> learn
    cli --> compose
    cli --> agent
    cli --> agent_server
    cli --> gate
    cli --> orchestrator
    cli --> dreams
    cli --> daimon
    cli --> neuro
    cli --> conductor
    cli --> plugin
    cli --> runtime
    cli --> serve
    cli --> index
    cli --> graph
    cli --> chain
    cli --> acp

    serve --> core
    serve --> agent
    serve --> agent_server
    serve --> chain
    serve --> learn
    serve --> neuro
    serve --> dreams
    serve --> gate
    serve --> fs
    serve --> compose
    serve --> std
    serve --> orchestrator
    serve --> conductor
    serve --> plugin
    serve --> daimon
    serve --> runtime

    agent_server --> agent
    agent_server --> chain
    agent_server --> core
    agent_server --> learn
    agent_server --> neuro

    acp --> core
    acp --> runtime
    acp --> agent
    acp --> gate
    acp --> compose
    acp --> orchestrator
    acp --> learn
    acp --> dreams
    acp --> neuro
    acp --> std
```

---

## 6. Crate-by-Crate Reference

### L0 -- Primitives

#### `roko-primitives` (L0)
**Purpose:** Zero-dependency compute primitives shared across the workspace.

**Key modules and types:**
- `hdc` -- `HdcVector` (10,240-bit hyperdimensional computing vector), `BundleAccumulator`, `DecayingBundleAccumulator`, `ItemMemory`
- `codebook` -- `Codebook`, `PatternStore`, `CodingCodebook`, `role_bind`, `detect_cross_domain_resonance`
- `tier` -- `InferenceTier` (T0/T1/T2 three-tier model routing), `TierRouter`
- `manifold` -- Riemannian metric tensors, Christoffel symbols, geodesics, Ricci curvature, Frechet means for execution cost manifolds
- `tda` -- Topological Data Analysis: persistence diagrams, Takens embedding, persistence landscapes
- `sheaf` -- Cellular sheaves for oracle consistency: coboundary operators, sheaf Laplacian, inconsistency scoring
- `tropical` -- `TropicalF64`, tropical polynomials, tropical attention, adversarial distance
- `robust_stats` -- Trimmed mean, MAD, Hodges-Lehmann estimator
- `pad` -- `PadVector` (Pleasure-Arousal-Dominance affective state)

**Dependencies:** serde, serde_json, uuid. No internal workspace dependencies.

---

### L1 -- Kernel + Runtime

#### `roko-core` (L1)
**Purpose:** The kernel. Defines the `Engram` type, all nine protocol traits,
and ~100 supporting types that the rest of the system builds on. At ~30K LOC,
this is the largest single crate.

**Key modules and types:**
- `engram` -- `Engram`, `EngramBuilder`, `HdcFingerprint`
- `traits` -- `Store`, `Score`, `Verify`, `Route`, `Compose`, `React`, `Bus`, `ColdStore`, `Observe`, `Connect`, `Trigger`, `Substrate`
- `loop_tick` -- `loop_tick()`, `TickConfig`, `TickOutcome`
- `agent` -- `AgentBackend`, `AgentRole`, `ModelSpec`, `ModelTier`, `ProviderKind`, `TurnBudget`
- `cell` -- `Cell` supertrait for all protocol implementations
- `tool` -- `ToolDef`, `ToolHandler`, `ToolRegistry`, `ToolContext`, `ToolCall`, `ToolResult`, `EpsilonGreedyBandit`
- `foundation` -- `ModelCaller`, `GateRunner`, `PromptAssembler`, `EffectExecutor`, `FeedbackSink`
- `verdict` -- `Verdict`, `Outcome`, `Selection`, `TestCount`
- `kind` -- `Kind` enum (Task, GateVerdict, Episode, Prompt, Insight, Warning, and more)
- `body` -- `Body` (Text, Json, Binary, Empty)
- `score` -- `Score` struct (confidence, novelty, utility, reputation)
- `decay` -- `Decay` (None, HalfLife, Exponential)
- `provenance` -- `Provenance`, `Taint`, `TaintInfo`
- `policy_manifest` -- `RolePolicyManifest`, `RoleProfile`, `PromptPolicy`, `ToolCapabilityPolicy`
- `config` -- `GraduationConfig`, `GraduationPolicy`
- `cognitive_workspace` -- `CognitiveWorkspace`, `TaskInvocationContract`, `InvocationGateOutcome`
- `connector` -- `ConnectorRegistry`, `ConnectorKind` (MCP, API, Database, Blockchain, Feed, Custom)
- `prediction` -- `Oracle`, `PredictionStore`, `PredictiveScorer`, `CalibrationTracker`
- `dashboard_snapshot` -- `DashboardSnapshot`, `AgentTopology`, `DashboardEvent`
- `immune` -- `QuarantineVault`, `AnomalyScore`, `ImmuneResponse`
- `forensic` -- `ForensicReplay`, `GateVerdictRecord`, `PolicyDecisionRecord`
- `job` -- `MarketplaceJob`, `JobSubmission`, `FileJobStore`
- `namespace` -- `CognitiveNamespace`, `Channel`, `NamespaceRegistry`
- `operating_frequency` -- `OperatingFrequencyScheduler`
- `attestation` -- `Attestation`, `Ed25519Signature`, `ChainAttestation`
- `obs` -- `MetricRegistry`, `ProbeRegistry`, `HealthStatus`, `Histogram`, `LogScrubber`

**Dependencies:** roko-primitives, serde, tokio, blake3, ed25519-dalek, chrono, thiserror, parking_lot, notify, regex, toml, arc-swap

#### `roko-runtime` (L1)
**Purpose:** Shared async runtime primitives: typed event bus, process
supervision, pipeline state, and cancellation.

**Key modules and types:**
- `event_bus` -- Typed pub/sub event bus
- `pulse_bus` -- `PulseBus` for ephemeral `Pulse` transport
- `process` -- `ProcessSupervisor` for agent lifecycle management
- `workflow_engine` -- `WorkflowEngine` and `PipelineStateV2` for DAG execution
- `effect_driver` -- `EffectDriver` pattern for side-effect management
- `cancel` -- Cooperative cancellation tokens
- `heartbeat` -- Heartbeat protocol for health monitoring
- `lifecycle` -- Agent lifecycle state machine
- `task_scheduler` -- Task scheduling primitives
- `state_hub` -- Centralized state management
- `run_ledger` -- Execution cost accounting

**Dependencies:** roko-primitives, roko-core, roko-gate, tokio, reqwest, nix (Unix)

---

### L2 -- Capabilities

#### `roko-agent` (L2)
**Purpose:** Agent trait and LLM backend implementations. Dispatches work to
LLMs and manages the tool-use loop.

**Key modules and types:**
- `claude_agent` / `claude_cli_agent` -- Claude API and CLI agent backends
- `codex_agent` -- OpenAI Codex backend
- `openai_agent` / `openai_compat_backend` -- OpenAI and OpenAI-compatible backends
- `cursor_agent` / `cursor_cli_agent` -- Cursor IDE agent
- `ollama/` -- Local Ollama model backend
- `gemini/` -- Google Gemini backend
- `perplexity/` -- Perplexity research backend
- `dispatcher/` -- `ToolDispatcher` with parallel execution, timeout, dedup cache, hook chains, validation, truncation, metric emission, cancellation
- `safety/` -- Pre/post execution safety checks
- `tool_loop/` -- Agent tool-use conversation loop
- `mcp/` -- MCP client integration
- `pool` / `multi_pool` -- Agent connection pooling
- `rate_limit` -- Token-bucket rate limiting via `governor`
- `harness/` -- Test harness for agent backends
- `hermes/` -- Agent-to-agent messaging
- `metamorphosis` -- Agent capability evolution
- `introspection` -- Agent self-inspection

**Dependencies:** roko-core, roko-fs, roko-std, reqwest, tokio, governor, futures

#### `roko-compose` (L2)
**Purpose:** Prompt assembly and context composition. Implements the
`Compose` trait with budget-aware token counting.

**Key modules and types:**
- `system_prompt_builder` -- 9-layer `SystemPromptBuilder` for structured prompt assembly
- `context_assembler` -- `ContextAssembler` for composing context from multiple sources
- `attention` -- `AttentionBidder` variants (Neuro, Task, Research)
- `auction` -- VCG (Vickrey-Clarke-Groves) auction for context slot allocation
- `token_counter` -- Token counting via tiktoken-rs and HuggingFace tokenizers
- `enrichment/` -- Context enrichment pipeline
- `templates/` -- 9 role prompt templates
- `scorer` -- Section scoring for relevance-based selection
- `budget` / `budget_predictor` -- Token budget management and prediction
- `foraging` -- Information foraging for context discovery
- `symbol_resolver` -- Code symbol resolution for context
- `cognitive_workspace` -- CognitiveWorkspace prompt assembly
- `prompt_assembly_service` -- Full prompt assembly pipeline

**Dependencies:** roko-core, roko-agent, roko-learn, roko-neuro, tiktoken-rs, tokenizers

#### `roko-learn` (L2)
**Purpose:** Learning subsystems -- all the mechanisms by which Roko improves
over time.

**Key modules and types:**
- `episode_logger` -- Records agent turns and gate results to `.roko/episodes.jsonl`
- `cascade_router` / `cascade/` -- `CascadeRouter`: 3-stage model selection (confidence threshold, UCB exploration, contextual bandit)
- `playbook` / `playbook_rules` -- Pattern extraction and reuse from successful episodes
- `skill_library` -- Learned skill patterns
- `bandits` -- Epsilon-greedy and contextual bandit algorithms
- `prompt_experiment` / `model_experiment` -- A/B testing for prompts and models
- `efficiency` -- Per-turn efficiency tracking
- `hdc_fingerprint` / `hdc_clustering` -- Episode fingerprinting and clustering via HDC
- `pattern_discovery` -- Automated pattern extraction from episode history
- `active_inference` -- Active inference for exploration-exploitation
- `calibration_policy` -- Prediction calibration tracking
- `context_pack_cache` -- Caching assembled context packs
- `cost_table` / `costs_db` / `costs_log` -- Cost tracking and analysis
- `error_pattern_store` / `error_enrichment` -- Error pattern learning
- `provider_health` -- LLM provider health tracking
- `wal` -- Write-ahead log for learning state durability

**Dependencies:** roko-core, roko-agent, roko-daimon, roko-fs, roko-primitives

#### `roko-neuro` (L2)
**Purpose:** Knowledge and memory subsystems -- durable knowledge store with
distillation and tier progression.

**Key modules and types:**
- `knowledge_store` -- Durable knowledge persistence
- `distiller` -- Knowledge distillation (compress episodic memory into durable insights)
- `tier_progression` -- Knowledge tier promotion (ephemeral -> durable -> core)
- `admission` -- Knowledge admission control
- `context` -- Knowledge-enriched context provider
- `episode_completion` -- Episode completion analysis
- `lifecycle` -- Knowledge lifecycle management
- `temporal` -- Temporal knowledge indexing
- `hdc` -- HDC-based semantic knowledge search (optional, behind `hdc` feature)

**Dependencies:** roko-core, roko-fs, roko-agent, roko-learn

#### `roko-daimon` (L2)
**Purpose:** Affect and motivation modeling. The "somatic marker" system that
modulates agent behavior based on emotional state.

**Key modules and types:**
- `policy` -- `DaimonPolicy`: affect-modulated decision making
- `somatic_ta` -- Somatic marker hypothesis implementation
- `goals` -- Goal tracking and priority management
- `mortality` -- Temporal urgency modeling
- `life_review` -- Retrospective performance analysis
- Uses `kiddo` for k-d tree nearest-neighbor lookup in affect space

**Dependencies:** roko-core, kiddo

#### `roko-dreams` (L2)
**Purpose:** Offline consolidation -- background processing that runs during
idle periods to consolidate knowledge and improve capabilities.

**Key modules and types:**
- `cycle` -- Dream cycle orchestration
- `hypnagogia` -- Hypnagogic state for creative recombination
- `imagination` -- Counterfactual scenario generation
- `rehearsal` -- Mental rehearsal for upcoming tasks
- `replay` -- Experience replay for learning reinforcement
- `staging` -- Dream staging pipeline
- `routing_advice` -- Post-consolidation routing recommendations
- `threat` -- Threat scenario simulation
- `runner` -- Dream cycle runner
- `phase2/` -- Phase 2 dream subsystem stubs

**Dependencies:** roko-core, roko-neuro, roko-learn, roko-agent, roko-primitives, cron

#### `roko-fs` (L2)
**Purpose:** Filesystem-backed `Store` implementation. JSONL append-only
persistence with in-memory indexing.

**Key type:** `FileSubstrate` -- stores engrams to `.roko/signals.jsonl`
with concurrent read/write access. Optional `hdc` feature for HDC-indexed
similarity queries. Also provides `ArchiveColdSubstrate` for cold storage.

**Dependencies:** roko-core, tokio, parking_lot

#### `roko-std` (L2)
**Purpose:** Standard trait implementations and defaults. The "batteries
included" crate.

**Key modules and types:**
- `memory` -- `MemorySubstrate` (in-memory `Store` for testing)
- `noop` -- No-op defaults for all traits
- `router` -- Standard router implementations
- `scorer` -- Standard scorer implementations
- `roles` -- Standard role definitions
- `tool/builtin/` -- 19 built-in tools: `apply_patch`, `bash`, `edit_file`, `exit_plan_mode`, `glob`, `grep`, `isfr`, `ls`, `multi_edit`, `notebook_edit`, `read_file`, `run_tests`, `sandbox`, `task_agent`, `todo_write`, `web_fetch`, `web_search`, `write_file`
- `tool/mock_dispatcher` -- Mock tool dispatcher for testing
- `greeting` -- Greeting generation
- `math` -- Mathematical utilities

**Dependencies:** roko-core, roko-chain, reqwest, tokio

#### `roko-chain` (L2)
**Purpose:** On-chain client abstractions. Trait contracts for blockchain
reads and signed writes.

**Key types:** `ChainClient`, `ChainWallet` traits, mock implementations.
Optional `alloy-backend` feature for real JSON-RPC endpoints (mirage-rs,
Anvil, live testnets).

**Dependencies:** roko-core, alloy (optional), tokio

#### `roko-plugin` (L2)
**Purpose:** Plugin SDK for external event sources and feedback collectors.

**Key types:** `EventSource`, `FeedbackCollector`. Supports cron-based
scheduling, filesystem watching (via `notify`), and glob-based event
filtering.

**Dependencies:** roko-core, cron, notify, globset, tokio

#### `roko-graph` (L2)
**Purpose:** DAG-based graph execution engine with fan-out/fan-in support.

**Key modules and types:**
- `engine` -- Graph execution engine with topological ordering
- `cell` / `cells/` -- `Cell` implementations for graph nodes
- `loader` -- TOML-based graph definition loader
- `registry` -- `CellRegistry` for cell type discovery
- `topo` -- Topological sort with cycle detection (via petgraph)
- `budget` -- Execution budget tracking
- `condition` -- Conditional edge evaluation
- `hot` -- Hot-reloading support

**Dependencies:** roko-core, petgraph, tokio, toml

#### `roko-index` (L2)
**Purpose:** Code intelligence -- source parsing, symbol graphs, PageRank
scoring, and HDC fingerprinting.

**Key modules and types:**
- `parser` -- Multi-language source parser
- `symbol` -- Symbol extraction (functions, types, imports)
- `graph` -- Symbol dependency graph with PageRank scoring
- `hdc` -- HDC fingerprints for code similarity
- `sqlite` -- SQLite-backed persistent index (optional `sqlite` feature)
- `workspace` -- Workspace-level index management

**Dependencies:** roko-core, roko-lang-rust, roko-lang-typescript, roko-lang-go, regex, rusqlite (optional)

#### Language Providers: `roko-lang-rust`, `roko-lang-typescript`, `roko-lang-go` (L2)
**Purpose:** Language-specific implementations of the `LanguageProvider`
trait from roko-core. Each provides symbol extraction, build system
detection, and compilation support for its language.

**Dependencies:** roko-core only. `roko-lang-rust` optionally uses tree-sitter for parsing.

#### MCP Servers: `roko-mcp-stdio`, `roko-mcp-github`, `roko-mcp-slack`, `roko-mcp-scripts`, `roko-mcp-code` (L2)
**Purpose:** Standalone Model Context Protocol (MCP) servers that agents
invoke via `--mcp-config`. Each exposes domain-specific tools over stdio
JSON-RPC.

| Crate | Tool Surface |
|-------|-------------|
| `roko-mcp-stdio` | Shared stdio JSON-RPC transport (dependency for other MCP servers) |
| `roko-mcp-github` | GitHub API: PR management, file reading, issue tracking |
| `roko-mcp-slack` | Slack Web API: channel messaging, search |
| `roko-mcp-scripts` | Wraps arbitrary scripts as MCP tools |
| `roko-mcp-code` | Code intelligence tools backed by roko-index |

#### `roko-demo` (L2)
**Purpose:** Demo environment orchestrator. Deploys contracts, seeds
fixtures, and spawns agent clades per declarative scenario manifests.

**Dependencies:** roko-chain (with alloy-backend), alloy, tokio, ratatui

---

### L3 -- Orchestration

#### `roko-gate` (L3)
**Purpose:** Concrete `Verify` trait implementations and gate pipeline
composition.

**Key modules and types (42 source files):**
- `gate_pipeline` -- Multi-rung gate pipeline composition
- `rung_dispatch` / `rung_selector` -- Rung selection and dispatch
- `adaptive_threshold` -- EMA-based threshold adaptation
- `compile` / `compile_errors` -- Compilation gate
- `test_gate` / `generated_test_gate` / `property_test_gate` / `integration_gate` -- Testing gates
- `clippy_gate` -- Clippy lint gate
- `diff_gate` -- Diff analysis gate
- `llm_judge_gate` -- LLM-as-judge gate
- `benchmark_gate` -- Performance benchmark gate
- `security_scan_gate` -- Security scanning
- `format_check_gate` -- Code formatting check
- `fact_check` -- Factual accuracy verification
- `shell` -- Shell command execution gate
- `symbol_gate` -- Symbol resolution verification
- `verify_chain_gate` -- On-chain verification
- `artifact_store` -- Gate artifact persistence
- `process_reward` -- Process reward model
- `ratchet` -- Ratchet-based quality progression
- `spc` -- Statistical process control
- `pelt` -- PELT changepoint detection
- `hotelling` -- Hotelling T-squared test

**Dependencies:** roko-core, roko-agent, tokio, toml

#### `roko-orchestrator` (L3)
**Purpose:** Plan discovery, task DAG management, worktree isolation, and
parallel execution.

**Key modules and types:**
- `plan_discovery` -- Scan filesystem for plan files
- `dag` -- Task DAG construction and traversal
- `executor/` -- Parallel task executor with concurrency control
- `worktree` -- Git worktree manager for task isolation
- `merge_queue` -- Merge queue for completed tasks
- `post_merge` -- Post-merge validation
- `coordination` -- Multi-agent coordination
- `replan` -- Replanning on gate failure
- `repair` -- Execution repair strategies
- `safety/` -- Safety checks for orchestrator operations
- `mesh_relay` -- Mesh relay for distributed execution
- `runtime_snapshot` -- Execution state snapshots for `--resume`
- `event_log` -- Event logging
- `service_factory` -- Service construction

**Dependencies:** roko-core, roko-agent, roko-compose, roko-conductor, roko-daimon, roko-gate, roko-learn, roko-neuro, roko-runtime

#### `roko-conductor` (L3)
**Purpose:** Reactive intelligence layer -- watchers that monitor execution
and trigger interventions.

**Key modules and types:**
- `conductor` -- Main conductor orchestration
- `circuit_breaker` -- Circuit breaker for cascading failure prevention
- `watchers/` -- 10 specialized watchers:
  - `compile_fail_repeat` -- Repeated compilation failure detection
  - `context_window_pressure` -- Context window utilization monitoring
  - `cost_overrun` -- Budget overrun detection
  - `ghost_turn` -- Unproductive turn detection
  - `iteration_loop` -- Infinite loop detection
  - `review_loop` -- Review cycle detection
  - `spec_drift` -- Specification drift monitoring
  - `stuck_pattern` -- Stuck agent pattern detection
  - `test_failure_budget` -- Test failure budget tracking
  - `time_overrun` -- Time budget overrun detection
- `diagnosis` -- Problem diagnosis and root cause analysis
- `interventions` -- Intervention action execution
- `pattern_detector` -- Cross-watcher pattern correlation
- `self_healing` -- Self-healing recovery strategies
- `stuck_detection` -- Agent stuck state detection
- `threshold_learner` -- Adaptive threshold learning for watcher triggers
- `state_machine` -- Conductor state machine
- `federation` -- Federated conductor coordination
- `health` -- Conductor health monitoring
- `yerkes_dodson` -- Yerkes-Dodson optimal arousal modeling

**Dependencies:** roko-core, roko-learn, dashmap, parking_lot

---

### L4 -- Applications

#### `roko-cli` (L4)
**Purpose:** The main `roko` binary. 60+ CLI subcommands covering the full
agent lifecycle: plan management, PRD workflow, research, knowledge, learning
inspection, agent management, deployment, TUI dashboard, and more.

**Key modules:**
- `main.rs` -- Entry point and CLI argument parsing (clap)
- `orchestrate.rs` -- The plan execution engine that wires all subsystems
- `tui/` -- Interactive ratatui terminal UI (F1-F7 tabs)
- `chat.rs` / `chat_session.rs` -- Interactive chat REPL
- `prd/` / `prd.rs` -- PRD lifecycle management
- `plan.rs` / `plan_generate.rs` / `plan_validate.rs` -- Plan management
- `research.rs` -- Research agent integration
- `runner/` -- Plan runner and dispatch
- `dispatch/` / `dispatch_v2.rs` -- Agent dispatch coordination
- `daemon/` -- Background daemon management
- `vision_loop/` -- Visual processing loop
- `worker/` -- Deployed worker mode
- `inline/` -- Inline execution mode

**Dependencies:** All L0-L3 crates, plus ratatui, crossterm, clap, dotenvy, indicatif, pulldown-cmark, webbrowser

#### `roko-serve` (L4)
**Purpose:** HTTP control plane server with ~85 REST routes, SSE streaming,
and WebSocket support. Runs on port 6677 via `roko serve`.

**Key modules:**
- `routes/` -- All HTTP route handlers
- `config_watcher` -- Hot-reload for configuration changes
- `event_bus` / `events` -- Server-side event distribution
- `runtime` -- Server runtime lifecycle
- `deploy/` -- Deployment integration
- `feed_agents/` -- Feed agent management
- `job_runner` -- Job execution management
- `relay` -- Request relay
- `scheduler` -- Task scheduling
- `terminal` -- Terminal PTY support (via portable-pty)
- `jwks` -- JWT/JWKS authentication

**Dependencies:** Most L0-L3 crates, plus axum, tower, tower-http, jsonwebtoken, utoipa (OpenAPI), rust-embed, portable-pty

#### `roko-agent-server` (L4)
**Purpose:** Per-agent HTTP sidecar server. Provides REST endpoints for
direct agent interaction: `/message` (real LLM dispatch), `/stream`
(WebSocket), `/predictions`, `/research`, `/tasks`. Also handles ERC-8004
agent card registration.

**Dependencies:** roko-core, roko-agent, roko-chain, roko-learn, roko-neuro, agent-relay, axum, tower

#### `roko-acp` (L4)
**Purpose:** Agent Client Protocol (ACP) server surface for editor
integrations (VS Code, JetBrains, etc.). Bridges editor events to the Roko
pipeline.

**Key modules:**
- `pipeline` -- ACP pipeline orchestration
- `session` -- Editor session management
- `transport` -- ACP transport layer
- `config` / `config_watch` -- ACP configuration with hot-reload
- `workflow` -- Workflow management
- `knowledge` -- Knowledge integration
- `builtin_tools` -- ACP-specific tool implementations
- `handler` -- Request handler
- `bridge_events` / `event_forward` -- Event bridging

**Dependencies:** roko-core, roko-runtime, roko-agent, roko-gate, roko-compose, roko-orchestrator, roko-learn, roko-dreams, roko-neuro, roko-std

---

### Applications (apps/)

#### `mirage-rs`
**Purpose:** In-process Ethereum fork simulator. Provides lazy upstream
reads, copy-on-write scenario branching, and a JSON-RPC server. Optional
`chain` feature adds HDC-indexed knowledge, stigmergy pheromone subsystems.
Optional `roko` feature bridges to roko-core traits (`Gate`, `Substrate`).

**Dependencies:** revm, alloy-primitives, jsonrpsee, roko-runtime, roko-primitives (optional), roko-core (optional)

#### `agent-relay`
**Purpose:** Narrow in-memory relay for agent WebSocket presence, agent
cards, and message forwarding between agents.

**Dependencies:** axum, tokio, serde, uuid (no roko-* dependencies)

#### `roko-chain-watcher`
**Purpose:** Long-running agent that subscribes to a mirage chain via
JSON-RPC and posts insights back via HTTP.

**Dependencies:** roko-core, roko-chain, reqwest, tokio

---

## 7. Data Flow Through the System

### End-to-End: PRD to Merged Code

```
User                                                         System
  │                                                            │
  │  roko prd idea "Add retry logic"                           │
  │ ──────────────────────────────────────────────────────────► │
  │                                                            │
  │  roko prd draft new "http-retry"                           │
  │ ──────────────────────────────────────────────────────────► │
  │                     ┌──────────────────────────────────┐   │
  │                     │ LLM generates PRD from idea      │   │
  │                     │ PRD saved to .roko/prd/           │   │
  │                     └──────────────────────────────────┘   │
  │                                                            │
  │  roko prd plan http-retry                                  │
  │ ──────────────────────────────────────────────────────────► │
  │                     ┌──────────────────────────────────┐   │
  │                     │ LLM reads PRD + codebase context │   │
  │                     │ Generates tasks.toml (DAG)       │   │
  │                     └──────────────────────────────────┘   │
  │                                                            │
  │  roko plan run plans/                                      │
  │ ──────────────────────────────────────────────────────────► │
  │                     ┌──────────────────────────────────┐   │
  │                     │ Orchestrator reads tasks.toml    │   │
  │                     │ Topological sort → execution DAG │   │
  │                     │                                  │   │
  │                     │ For each task:                   │   │
  │                     │   1. Compose prompt (9 layers)   │   │
  │                     │   2. Route to model (Cascade)    │   │
  │                     │   3. Dispatch agent + tool loop  │   │
  │                     │   4. Gate pipeline verification  │   │
  │                     │   5. Persist result as Engram    │   │
  │                     │   6. Episode logging             │   │
  │                     │   7. Learning feedback           │   │
  │                     │                                  │   │
  │                     │ On gate failure:                 │   │
  │                     │   → Replan + retry (configurable)│   │
  │                     │                                  │   │
  │                     │ State snapshots for --resume     │   │
  │                     └──────────────────────────────────┘   │
  │                                                            │
  │  roko dashboard                                            │
  │ ──────────────────────────────────────────────────────────► │
  │                     ┌──────────────────────────────────┐   │
  │                     │ TUI: F1-F7 tabs                  │   │
  │                     │ Real-time agent status           │   │
  │                     │ Gate pass/fail metrics           │   │
  │                     │ Cost tracking                    │   │
  │                     └──────────────────────────────────┘   │
```

### Internal Data Flow: Single Task Execution

```
                  ┌─────────────┐
                  │  tasks.toml │
                  │  (DAG)      │
                  └──────┬──────┘
                         │
                         ▼
            ┌────────────────────────┐
            │    Plan Discovery      │  roko-orchestrator
            │    + DAG Resolution    │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │   SystemPromptBuilder  │  roko-compose
            │   (9-layer assembly)   │
            │                        │
            │   L1: Mission/identity │
            │   L2: Role policy      │
            │   L3: Domain context   │
            │   L4: Task brief       │
            │   L5: Playbook rules   │
            │   L6: Research context │
            │   L7: Recent episodes  │
            │   L8: Knowledge store  │
            │   L9: Affect modulation│
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    CascadeRouter       │  roko-learn
            │    (model selection)   │
            │                        │
            │    Stage 1: Confidence │
            │    Stage 2: UCB        │
            │    Stage 3: Bandit     │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    Agent Dispatch      │  roko-agent
            │    + Tool Loop         │
            │                        │
            │    Claude / Codex /    │
            │    Gemini / Ollama /   │
            │    OpenAI / Perplexity │
            └───────────┬────────────┘
                        │
                        ▼
            ┌────────────────────────┐
            │    Gate Pipeline       │  roko-gate
            │    (multi-rung)        │
            │                        │
            │    R1: Compile         │
            │    R2: Test            │
            │    R3: Clippy          │
            │    R4: Diff analysis   │
            │    R5: LLM judge       │
            │    R6: Property tests  │
            │    R7: Benchmarks      │
            └───────────┬────────────┘
                        │
                   ┌────┴────┐
                   │         │
                Pass       Fail
                   │         │
                   ▼         ▼
        ┌──────────────┐  ┌──────────────┐
        │ Store engram  │  │ Replan       │
        │ + Episode log │  │ + Retry      │
        │ + Learning    │  │              │
        │   feedback    │  │ (configurable│
        │ + Conductor   │  │  via learning│
        │   notification│  │  config)     │
        └──────────────┘  └──────────────┘
```

---

## 8. The Universal Cognitive Loop

Every operation in Roko reduces to calling `loop_tick()` with different trait
implementations. The function is defined in `roko-core/src/loop_tick.rs`:

```
candidates = store.query(q, ctx)
      |
selection  = router.select(candidates, ctx)
      |
composed   = composer.compose([selection], budget, scorer, ctx)
      |
verdict    = gate.verify(composed, ctx)
      |
if passed: store.put(composed) + policy.decide(stream, ctx)
```

The signature:

```rust
pub async fn loop_tick(
    store:    &dyn Store,
    scorer:   &dyn Score,
    gate:     &dyn Verify,
    router:   &dyn Route,
    composer: &dyn Compose,
    policy:   &dyn React,
    query:    &Query,
    budget:   &Budget,
    ctx:      &Context,
) -> Result<TickOutcome>
```

This is the core abstraction: by plugging in different implementations of the
six traits, the same loop serves as:

- **Task executor**: store=FileSubstrate, gate=CompileGate+TestGate, router=CascadeRouter
- **Model selector**: store=MemorySubstrate(model candidates), router=LinUCBRouter
- **Context assembler**: composer=PromptComposer, scorer=RelevanceScorer
- **Knowledge retriever**: store=HdcSubstrate, scorer=RecencyScorer

### TickOutcome

```rust
pub struct TickOutcome {
    pub candidates_examined: usize,
    pub composed: Option<Engram>,
    pub verdict: Option<Verdict>,
    pub policy_emissions: Vec<Engram>,
}
```

---

## 9. Key Data Types

### Engram (the universal datum)

```rust
pub struct Engram {
    pub id: ContentHash,                      // BLAKE3 content hash
    pub fingerprint: Option<HdcFingerprint>,  // 10,240-bit HDC vector
    pub kind: Kind,                           // Task, GateVerdict, Episode, ...
    pub body: Body,                           // Text, Json, Binary, Empty
    pub created_at_ms: i64,                   // Unix milliseconds
    pub decay: Decay,                         // None, HalfLife, Exponential
    pub provenance: Provenance,               // Author, taint chain
    pub score: Score,                         // confidence, novelty, utility, reputation
    pub lineage: Vec<ContentHash>,            // Parent engrams (DAG)
    pub tags: BTreeMap<String, String>,        // Arbitrary metadata
    pub attestation: Option<Attestation>,      // Cryptographic proof
    pub emotional_tag: Option<EmotionalTag>,   // PAD affect state
}
```

**Content hash covers:** kind + body + author + taint + lineage + tags.

**Excluded from hash:** score, decay, timestamp, attestation, emotional tag
(these can change without changing identity).

**Effective weight at time t:** `score.effective() * decay.apply(t - created_at_ms)`

### Verdict (gate output)

```rust
pub struct Verdict {
    pub passed: bool,
    pub gate_name: String,
    pub details: String,
    pub outcome: Outcome,
    pub test_count: Option<TestCount>,
}
```

### Score (multi-dimensional quality)

```rust
pub struct Score {
    pub confidence: f32,  // 0.0..=1.0
    pub novelty: f32,     // 0.0..=1.0
    pub utility: f32,     // 0.0..=1.0
    pub reputation: f32,  // 0.0..=1.0
}
```

### Kind (engram classification)

Non-exhaustive enum with variants including: `Task`, `GateVerdict`, `Episode`,
`Prompt`, `Insight`, `Warning`, `Plan`, `PlanRevision`, `Research`, `Knowledge`,
`Message`, `Observation`, `Intervention`, `Heartbeat`, and others.

### Pulse (ephemeral real-time data)

The transient counterpart to `Engram`. Flows through the `Bus` for immediate
reactions; only promoted to Engram when worth persisting.

### HdcVector (semantic fingerprint)

10,240-bit hyperdimensional computing vector. Operations: XOR bind, majority
bundle, Hamming similarity. Used for semantic search, episode clustering, and
cross-domain resonance detection.

---

## 10. Agent Lifecycle and Coordination

### Agent Backends

Roko supports multiple LLM providers through the `AgentBackend` abstraction:

| Backend | Module | Transport |
|---------|--------|-----------|
| Claude API | `claude_agent.rs` | HTTP (Anthropic API) |
| Claude CLI | `claude_cli_agent.rs` | Process spawn |
| OpenAI | `openai_agent.rs` | HTTP (OpenAI API) |
| OpenAI-compatible | `openai_compat_backend.rs` | HTTP (any OpenAI-compatible endpoint) |
| Codex | `codex_agent.rs` | HTTP |
| Cursor | `cursor_agent.rs` / `cursor_cli_agent.rs` | HTTP / Process spawn |
| Ollama | `ollama/` | HTTP (local) |
| Gemini | `gemini/` | HTTP (Google API) |
| Perplexity | `perplexity/` | HTTP (Perplexity API) |
| Mock | `mock.rs` | In-process (testing) |

### Tool Dispatcher

The `ToolDispatcher` (in `roko-agent/src/dispatcher/`) is the central
execution engine for agent tool calls. It provides:

- **Parallel execution** -- concurrent tool calls within a single turn
- **Validation** -- parameter validation before execution
- **Timeout** -- per-call and total timeout enforcement
- **Dedup cache** -- deduplicate identical tool calls
- **Hook chains** -- pre/post execution hooks
- **Truncation** -- output truncation for context window management
- **Metric emission** -- tool call metrics
- **Cancellation** -- cooperative cancellation support
- **Result cache** -- cache tool results for re-use
- **Tool selector** -- relevance-based tool selection

### Agent Coordination

- `ProcessSupervisor` (roko-runtime) -- manages agent process lifecycles
- `hermes/` (roko-agent) -- agent-to-agent messaging
- `pool` / `multi_pool` (roko-agent) -- connection pooling
- `agent-relay` (apps/) -- WebSocket presence and message relay

---

## 11. Gate Pipeline and Verification

The gate pipeline is a multi-rung verification system. Each rung runs a
specific type of check, and adaptive thresholds (learned via EMA) determine
pass/fail boundaries.

### Gate Types

| Gate | What It Checks |
|------|---------------|
| `CompileGate` | Code compiles without errors |
| `TestGate` | Unit tests pass |
| `GeneratedTestGate` | LLM-generated tests pass |
| `PropertyTestGate` | Property-based tests (proptest) pass |
| `IntegrationGate` | Integration tests pass |
| `ClippyGate` | Clippy lints pass cleanly |
| `DiffGate` | Diff analysis (no regressions, reasonable scope) |
| `LlmJudgeGate` | LLM evaluates output quality |
| `BenchmarkGate` | Performance benchmarks within thresholds |
| `SecurityScanGate` | Security vulnerability scan |
| `FormatCheckGate` | Code formatting compliance |
| `FactCheckGate` | Factual accuracy verification |
| `SymbolGate` | Symbol resolution verification |
| `VerifyChainGate` | On-chain state verification |
| `ShellGate` | Arbitrary shell command gate |

### Adaptive Thresholds

Gate thresholds are not fixed. The system uses exponential moving averages
(EMA) to adapt thresholds based on historical pass rates. This is persisted
in `.roko/learn/gate-thresholds.json`.

### Statistical Quality Control

- **SPC** (Statistical Process Control) -- control charts for quality metrics
- **PELT** -- changepoint detection for identifying quality regime shifts
- **Hotelling T-squared** -- multivariate statistical testing
- **Ratchet** -- quality can only improve, never regress

---

## 12. Learning and Self-Improvement

Roko's learning is structural, not parameter-based. It does not fine-tune
LLMs. Instead, it improves through five mechanisms:

### 1. CascadeRouter (Model Selection)

Three-stage model selection:
1. **Confidence stage** -- route based on task complexity and confidence thresholds
2. **UCB stage** -- upper confidence bound exploration for new model/task pairs
3. **Contextual bandit stage** -- contextual bandit (LinUCB) for personalized routing

Persisted to `.roko/learn/cascade-router.json`.

### 2. Episode Logger

Every agent turn (prompt, model response, tool calls, gate result) is logged
as a structured episode to `.roko/episodes.jsonl`. Episodes include:
- Agent identity and model used
- Full prompt and response
- Tool calls and results
- Gate verdicts
- Cost and latency metrics
- HDC fingerprint for clustering

### 3. Playbook System

Successful episodes are analyzed for reusable patterns. When a pattern is
detected (e.g., "for Rust HTTP retry logic, use the backoff crate with
exponential delay"), it is extracted as a playbook rule and injected into
future prompts for similar tasks.

### 4. Prompt Experiments (A/B Testing)

The `ExperimentStore` runs A/B tests on prompt sections. Different prompt
variants are tracked with Thompson sampling for statistical significance.
Results are persisted to `.roko/learn/experiments.json`.

### 5. Efficiency Tracking

Per-turn efficiency events track cost, latency, token usage, and outcome
quality. Trends are analyzed for budget optimization. Persisted to
`.roko/learn/efficiency.jsonl`.

### Additional Learning Subsystems

- **Active inference** -- exploration-exploitation balancing
- **Error pattern store** -- learn from recurring error types
- **Provider health** -- track LLM provider reliability
- **Context pack cache** -- reuse assembled context packs
- **HDC clustering** -- cluster episodes by semantic similarity
- **Pattern discovery** -- automated pattern extraction

---

## 13. Knowledge and Memory (Neuro)

The Neuro subsystem provides durable knowledge with a three-tier progression:

```
Ephemeral (Pulse) → Durable (Engram in Store) → Core (distilled insight)
```

### Knowledge Store

The knowledge store (`roko-neuro/src/knowledge_store.rs`) provides:
- Admission control -- not all information is worth persisting
- Temporal indexing -- knowledge is time-aware
- Distillation -- compress episodic memory into durable insights
- Tier progression -- promote high-value knowledge from ephemeral to core

### Knowledge Lifecycle

1. **Admission** -- new information evaluated for novelty and utility
2. **Storage** -- persisted as engrams with decay curves
3. **Distillation** -- LLM-driven compression of related episodes into insights
4. **Promotion** -- high-value knowledge promoted to core tier (slower decay)
5. **Archival** -- aged-out knowledge moved to cold store
6. **Retrieval** -- query by text, HDC similarity, or metadata

---

## 14. Affect Engine (Daimon)

The Daimon subsystem models agent affect (emotional state) to modulate
decision-making. It implements the somatic marker hypothesis: past
experiences create emotional associations that guide future decisions.

### PAD Model

Affect state is represented as a three-dimensional vector:
- **Pleasure** -- positive/negative valence
- **Arousal** -- activation level (calm to excited)
- **Dominance** -- sense of control

### Affect Modulation

The `DaimonPolicy` adjusts agent behavior based on affect state:
- High arousal + low dominance = more conservative (stuck/frustrated state)
- High pleasure + high dominance = more exploratory (confident state)
- Somatic markers from past failures bias away from repeating mistakes

### Components

- `somatic_ta.rs` -- Somatic marker computation
- `goals.rs` -- Goal tracking and priority
- `mortality.rs` -- Temporal urgency (deadline pressure)
- `life_review.rs` -- Retrospective analysis
- `policy.rs` -- Affect-modulated policy decisions

---

## 15. Offline Consolidation (Dreams)

The Dreams subsystem runs during idle periods to consolidate knowledge and
improve capabilities. Inspired by the neuroscience of memory consolidation
during sleep.

### Dream Cycle Phases

1. **Hypnagogia** -- creative recombination of recent experiences (looser
   associative connections, novel pattern discovery)
2. **Imagination** -- counterfactual scenario generation ("what if we had
   used a different approach?")
3. **Rehearsal** -- mental rehearsal for upcoming tasks
4. **Replay** -- experience replay for learning reinforcement (re-evaluate
   past decisions with current knowledge)
5. **Staging** -- stage consolidated insights for integration

### Scheduling

Dreams use the `cron` crate for scheduling. The cycle is not triggered at
runtime by default; it requires explicit invocation via
`roko knowledge dream run` or integration into a daemon schedule.

---

## 16. Conductor: Reactive Intelligence

The Conductor is the reactive intelligence layer. It runs 10 specialized
watchers that monitor execution in real-time and trigger interventions.

### Watchers

| Watcher | Detects |
|---------|---------|
| `compile_fail_repeat` | Same compilation error recurring across turns |
| `context_window_pressure` | Context window approaching capacity |
| `cost_overrun` | Budget exceeding thresholds |
| `ghost_turn` | Agent turns that produce no useful output |
| `iteration_loop` | Agent stuck in infinite retry loops |
| `review_loop` | Excessive review-revise cycles |
| `spec_drift` | Output drifting from specification |
| `stuck_pattern` | General agent stuck patterns |
| `test_failure_budget` | Test failures exceeding budget |
| `time_overrun` | Task exceeding time estimates |

### Circuit Breaker

When watchers detect critical patterns, the circuit breaker can:
- Pause the current task
- Escalate to a different model tier
- Request human intervention
- Abort the task with a diagnostic

### Yerkes-Dodson Modeling

The conductor uses the Yerkes-Dodson inverted-U curve to model optimal
arousal: moderate pressure improves performance, but extreme pressure (too
many failures, too many interventions) degrades it. This feeds into the
Daimon affect system.

---

## 17. Code Intelligence (Index)

The `roko-index` crate provides code intelligence capabilities:

- **Multi-language parsing** via language providers (Rust, TypeScript, Go)
- **Symbol extraction** -- functions, types, imports, exports
- **Dependency graph** -- symbol-level dependency tracking
- **PageRank scoring** -- rank symbols by importance in the dependency graph
- **HDC fingerprinting** -- semantic fingerprints for code similarity search
- **SQLite persistence** -- optional persistent index (feature-gated)

This powers the `roko-mcp-code` MCP server, which agents use for
codebase navigation during task execution.

---

## 18. Graph Execution Engine

The `roko-graph` crate provides a DAG-based execution engine that is the
newer alternative to the legacy plan runner:

- **Cell abstraction** -- every computation unit is a `Cell` with typed
  inputs and outputs
- **TOML loader** -- graph definitions loaded from TOML files
- **Topological sort** -- execution order determined by dependency analysis
  (via petgraph)
- **Fan-out/fan-in** -- parallel execution with synchronization barriers
- **Budget tracking** -- per-cell and total execution budgets
- **Conditional edges** -- edges that activate based on runtime conditions
- **Hot reloading** -- reload graph definitions without restart
- **CellRegistry** -- extensible cell type registration

---

## 19. Key Architectural Decisions

### Decision 1: One Noun, Nine Verbs

**Choice:** Every piece of data in the system is an Engram. Every operation
is one of nine trait implementations.

**Rationale:** This uniformity means the universal loop (`loop_tick`) works
for any capability. Adding a new capability means implementing a trait, not
changing the core. The system's expressiveness grows without architectural
changes.

**Trade-off:** Engram is a large type (~12 fields). Not all fields are
relevant for all use cases. The system uses `Option` and `Kind` to handle
this.

### Decision 2: Content-Addressed Immutability

**Choice:** Engrams are identified by BLAKE3 content hashes. Score, decay,
and timestamps are excluded from the hash.

**Rationale:** This gives a Git-like audit trail. Any engram can be verified
by recomputing its hash. Lineage forms a DAG that can be replayed for
forensic analysis.

**Trade-off:** Updates require creating new engrams with different content,
creating a new hash. The lineage chain tracks this evolution.

### Decision 3: Layered Architecture with CI Enforcement

**Choice:** Five strict layers (L0-L4) with CI-enforced dependency checks.

**Rationale:** Prevents accidental coupling between infrastructure and
application code. A change to roko-primitives can never accidentally depend
on CLI-specific logic.

**Trade-off:** Some natural dependency edges are awkward (e.g., roko-runtime
at L1 depends on roko-gate at L3 -- this is a known architectural tension
that exists because the runtime needs to verify gate outcomes).

### Decision 4: Structural Learning Over Fine-Tuning

**Choice:** Learn by adjusting routing weights, gate thresholds, prompt
sections, and playbook rules rather than by fine-tuning LLMs.

**Rationale:** Fine-tuning requires training infrastructure, is expensive,
and creates model drift risks. Structural learning is observable, debuggable,
and reversible. The CascadeRouter's state is a JSON file you can inspect and
edit.

**Trade-off:** The system cannot learn truly novel capabilities that require
model-level knowledge. It can only improve within the space of existing model
capabilities.

### Decision 5: Mechanical Verification Over Human Review

**Choice:** Multi-rung gate pipeline with adaptive thresholds, not human
review.

**Rationale:** Human review does not scale for self-hosting. The system must
verify its own output mechanically. Adaptive thresholds prevent both over-
and under-sensitivity.

**Trade-off:** Mechanical verification cannot catch all quality issues.
LLM-judge gates partially address this but add cost and latency.

### Decision 6: Decay as a First-Class Concept

**Choice:** Every engram has a decay function. Knowledge has a half-life.

**Rationale:** Without decay, the knowledge store grows unboundedly and
older information drowns out newer information. Decay ensures the system
naturally forgets stale knowledge while the cold store preserves it for
audit purposes.

**Trade-off:** Choosing the right decay parameters is difficult. Too fast
and valuable knowledge is lost; too slow and noise accumulates.

### Decision 7: Affect Modeling (Daimon)

**Choice:** Model agent emotional state using PAD vectors and somatic
markers.

**Rationale:** Agents that repeatedly fail on similar tasks should become
more cautious. Agents that succeed should explore more. Affect provides a
compact state that modulates behavior without explicit rules for every
scenario.

**Trade-off:** The affect model adds complexity and can produce non-obvious
behavior. It requires careful calibration.

---

## 20. Security Model

### Code Execution Safety

- **Sandbox tools** -- built-in tools include sandboxed execution
- **Safety layer** -- pre/post execution checks in `roko-agent/src/safety/`
- **Tool validation** -- parameter validation before execution in the ToolDispatcher
- **Hook chains** -- pre/post execution hooks for audit and control

### Cryptographic Primitives

- **BLAKE3** -- content hashing for engrams
- **Ed25519** -- cryptographic attestations for engram provenance
- **HMAC-SHA256** -- webhook signature verification
- **JWT/JWKS** -- HTTP API authentication (roko-serve)
- **K256 (secp256k1)** -- Ethereum-compatible signing (mirage-rs, chain integration)

### Policy Framework

- **RolePolicyManifest** -- per-role security policies (tool access, prompt constraints)
- **AgentContract** -- safety contracts for agent execution
- **QuarantineVault** -- quarantine suspicious engrams for manual review
- **ImmuneSystem** -- anomaly detection and incident linking
- **ForensicReplay** -- causal decision reconstruction for post-incident analysis
- **LogScrubber** -- redact sensitive data from logs

### Supply Chain

- `unsafe_code = "deny"` workspace-wide
- `clippy::unwrap_used = "deny"` workspace-wide
- `cargo-deny` for dependency auditing (`deny.toml`)

---

## 21. Deployment Model

### Single Binary

The primary deployment is the `roko` CLI binary with subcommands:

```bash
roko run "<prompt>"      # Single prompt execution
roko plan run plans/     # Execute a plan
roko serve               # Start HTTP control plane on :6677
roko dashboard           # Interactive TUI
roko daemon install      # Install as OS service
roko worker              # Run as deployed worker
```

### Deployment Targets

Prebuilt binaries via cargo-dist for:
- macOS ARM (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Linux glibc (`x86_64-unknown-linux-gnu`)
- Linux musl (`x86_64-unknown-linux-musl`)

### Cloud Deployment

Built-in deployment support for:
- **Railway** -- `railway.json` / `railway.toml` configuration
- **Fly.io** -- `fly.toml` configuration
- **Docker** -- `Dockerfile` + `docker/` configuration directory

### Daemon Mode

`roko daemon install` installs a system service (launchd on macOS, systemd
on Linux) that runs the HTTP control plane and background tasks (heartbeat,
dream cycles, event sources).

### Release Profile

```toml
[profile.release]
lto = "thin"
codegen-units = 1
strip = true
panic = "abort"
```

---

## 22. Comparison: Roko vs. IronClaw

Both systems are Rust-based AI agent platforms, but they serve different
purposes and make different architectural choices.

### Purpose

| Aspect | Roko | IronClaw |
|--------|------|----------|
| **Primary goal** | Self-developing autonomous agent system | Secure personal AI assistant |
| **User model** | Developer-operator running agent swarms | Individual user with multi-channel access |
| **Core metaphor** | "Agents that build themselves" | "User-first security with proactive execution" |

### Data Model

| Aspect | Roko | IronClaw |
|--------|------|----------|
| **Universal type** | `Engram` (content-addressed, decaying, scored) | No single universal type; uses conventional DB rows |
| **Persistence** | JSONL append-only files (`.roko/signals.jsonl`) | Dual-backend: PostgreSQL + libSQL/Turso |
| **Content addressing** | BLAKE3 hashes, engram DAGs | Not used |
| **Decay** | First-class (every engram has a half-life) | Not applicable |
| **Memory** | Neuro knowledge store with HDC similarity search | Workspace memory with hybrid FTS + vector RRF search |

### Architecture

| Aspect | Roko | IronClaw |
|--------|------|----------|
| **Core abstraction** | 1 noun + 9 verbs (universal loop) | Traits: Channel, Tool, LlmProvider, Database, etc. |
| **Layering** | Five strict layers (L0-L4) with CI enforcement | Module-based with module specs |
| **Crate count** | 30 crates + 3 apps | ~6 crates (ironclaw_safety, ironclaw_llm, ironclaw_skills, ironclaw_engine, ironclaw_gateway, ironclaw_embeddings) |
| **LOC** | ~727K Rust | Large (not measured here) |
| **LLM integration** | Multiple backends in roko-agent | Multiple backends in ironclaw_llm (rig-core based) |
| **MCP** | Standalone MCP servers (stdio) + client | MCP client (HTTP/stdio/Unix) in src/tools/mcp/ |

### Verification and Safety

| Aspect | Roko | IronClaw |
|--------|------|----------|
| **Verification** | Multi-rung gate pipeline (compile, test, clippy, LLM judge, etc.) | No equivalent gate pipeline |
| **Safety** | Agent contracts, safety layer, quarantine vault, immune system | ironclaw_safety crate (prompt injection, validation, leak detection, policy) |
| **Sandboxing** | Tool-level sandboxing | Docker-based per-project sandbox with proxy network |
| **Secrets** | Not a primary concern | AES-256-GCM encryption, OS keychain integration |

### Learning

| Aspect | Roko | IronClaw |
|--------|------|----------|
| **Model routing** | CascadeRouter (3-stage, learns from outcomes) | Static configuration |
| **Prompt optimization** | A/B experiments with Thompson sampling | Skills system (SKILL.md files, no A/B) |
| **Episode logging** | Structured JSONL with HDC fingerprints | Action records via ToolDispatcher |
| **Playbooks** | Automated pattern extraction from episodes | Manual skills (trusted vs installed) |

### User Interaction

| Aspect | Roko | IronClaw |
|--------|------|----------|
| **Primary interface** | CLI (60+ subcommands) + TUI (ratatui) | Multi-channel: CLI TUI, Web, HTTP webhooks, WASM channels |
| **HTTP surface** | ~85 routes on :6677 | Web gateway with SSE/WebSocket |
| **Editor integration** | ACP server (roko-acp) | No dedicated editor integration |
| **Background execution** | Daemon mode, dream cycles | Heartbeat system, routines |

### Chain/Blockchain

| Aspect | Roko | IronClaw |
|--------|------|----------|
| **Chain integration** | roko-chain (ChainClient/ChainWallet traits), mirage-rs (EVM simulator), chain attestations | Not applicable |
| **On-chain verification** | VerifyChainGate | Not applicable |

### Opportunities for Cross-Pollination

1. **Roko's gate pipeline for IronClaw.** IronClaw could benefit from
   mechanical verification of agent outputs, particularly for code-related
   tasks. A simplified gate pipeline (compile + test) would catch obvious
   errors before presenting results to users.

2. **IronClaw's dual-backend DB for Roko.** Roko's JSONL persistence is
   simple but does not scale well. IronClaw's PostgreSQL + libSQL dual-backend
   pattern with proper migrations would give Roko better query performance
   and operational reliability.

3. **IronClaw's security model for Roko.** Roko's security model is focused
   on agent-to-agent trust. IronClaw's user-facing security (AES-256-GCM
   secrets, OS keychain, prompt injection detection, credential injection)
   would strengthen Roko's deployment story.

4. **Roko's learning systems for IronClaw.** IronClaw uses static model
   configuration. Roko's CascadeRouter (adaptive model selection) and
   playbook system (learning from past interactions) could improve IronClaw's
   response quality over time.

5. **Roko's affect engine for IronClaw.** IronClaw's psychographic profile
   (9-dimension analysis) is a user model. Roko's Daimon affect engine is
   an agent behavior model. Combining both could create an assistant that
   adapts its behavior to user state while also managing its own internal
   state.

6. **IronClaw's multi-channel architecture for Roko.** Roko's primary
   interface is CLI-centric. IronClaw's Channel trait with web, HTTP webhook,
   and WASM channel support would give Roko broader accessibility.

7. **Shared MCP infrastructure.** Both systems implement MCP clients. A
   shared MCP library could reduce duplication. Roko's standalone MCP servers
   (roko-mcp-github, roko-mcp-code) could be directly consumed by IronClaw
   agents.

8. **Roko's code intelligence for IronClaw.** The roko-index crate
   (multi-language parsing, symbol graphs, PageRank) could enhance IronClaw's
   tool system for code-aware tasks.

---

## 23. File Path Reference

### Workspace

| What | Path |
|------|------|
| Workspace root | `/Users/will/dev/nunchi/roko/roko/` |
| All crates | `/Users/will/dev/nunchi/roko/roko/crates/` |
| All apps | `/Users/will/dev/nunchi/roko/roko/apps/` |
| Workspace Cargo.toml | `/Users/will/dev/nunchi/roko/roko/Cargo.toml` |
| CLAUDE.md | `/Users/will/dev/nunchi/roko/roko/CLAUDE.md` |

### Key Source Files

| What | Path |
|------|------|
| Engram type | `crates/roko-core/src/engram.rs` |
| Nine core traits | `crates/roko-core/src/traits.rs` |
| Universal loop | `crates/roko-core/src/loop_tick.rs` |
| Cell supertrait | `crates/roko-core/src/cell.rs` |
| Agent dispatcher | `crates/roko-agent/src/dispatcher/mod.rs` |
| Tool loop | `crates/roko-agent/src/tool_loop/` |
| System prompt builder | `crates/roko-compose/src/system_prompt_builder.rs` |
| CascadeRouter | `crates/roko-learn/src/cascade_router.rs` |
| Episode logger | `crates/roko-learn/src/episode_logger.rs` |
| Gate pipeline | `crates/roko-gate/src/gate_pipeline.rs` |
| Plan executor (legacy) | `crates/roko-cli/src/orchestrate.rs` |
| TUI | `crates/roko-cli/src/tui/` |
| HTTP routes | `crates/roko-serve/src/routes/` |
| Conductor watchers | `crates/roko-conductor/src/watchers/` |
| Knowledge store | `crates/roko-neuro/src/knowledge_store.rs` |
| Dream cycle | `crates/roko-dreams/src/cycle.rs` |
| Daimon policy | `crates/roko-daimon/src/policy.rs` |
| HDC vectors | `crates/roko-primitives/src/hdc.rs` |
| Graph engine | `crates/roko-graph/src/engine.rs` |
| Code index | `crates/roko-index/src/graph.rs` |
| ACP pipeline | `crates/roko-acp/src/pipeline.rs` |
| Workflow engine | `crates/roko-runtime/src/workflow_engine.rs` |

### Documentation

| What | Path |
|------|------|
| Architecture guide | `docs/v2/ARCHITECTURE-GUIDE.md` |
| API reference | `docs/v2/API-REFERENCE.md` |
| CLI reference | `docs/v2/CLI-REFERENCE.md` |
| Design docs (v2) | `docs/v2/` (28 topic files) |
| Deep dives (v2-depth) | `docs/v2-depth/` (22 topic directories) |

### Persistence (Runtime)

| What | Path |
|------|------|
| Roko data directory | `.roko/` |
| Signal log | `.roko/signals.jsonl` |
| Episode log | `.roko/episodes.jsonl` |
| Executor snapshots | `.roko/state/` |
| PRD storage | `.roko/prd/` |
| Research artifacts | `.roko/research/` |
| Learning state | `.roko/learn/` |
| CascadeRouter state | `.roko/learn/cascade-router.json` |
| Gate thresholds | `.roko/learn/gate-thresholds.json` |
| Experiments | `.roko/learn/experiments.json` |
| Efficiency log | `.roko/learn/efficiency.jsonl` |
| Gaps tracker | `.roko/GAPS.md` |

---

*Document generated from roko source code at `/Users/will/dev/nunchi/roko/roko/`
as of July 2, 2026. Verified against actual Cargo.toml files, source modules,
and CLAUDE.md project instructions.*

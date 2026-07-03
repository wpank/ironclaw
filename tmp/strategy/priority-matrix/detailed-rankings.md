# Detailed Rankings: All 25 Items with Full Justifications

> **Navigation**:
> [README](README.md) |
> [Detailed Rankings (you are here)](detailed-rankings.md) |
> [Implementation Sketches](implementation-sketches.md) |
> [Quick Wins](quick-wins.md) |
> [Synergy Analysis](synergy-analysis.md) |
> [Benchmarking Plans](benchmarking-plans.md) |
> [References](references.md)

---

## Stars Tier (Ranks 1–6): Build in Week 1–2

All Stars can be completed in 1–5 days each. They are low-effort, high-safety, and fully
independent. Every one is additive — none change existing code paths in a breaking way.

---

### Rank 1: Metacognitive Monitor (Composite: 4.55)

**Scores:** User=5, System=4, Ease=4, Safety=5, Independence=5
**Source**: [../../agent-intelligence/agent-patterns.md](../../agent-intelligence/agent-patterns.md)
**Implementation**: See [implementation-sketches.md](implementation-sketches.md#rank-1-metacognitive-monitor)

**What it is:** A self-monitoring layer inside the agent loop that detects three pathologies:
stuck loops (agent repeating the same actions), cost runaways (spend trending toward budget
exhaustion), and self-contradiction (agent asserting X then not-X). On detection it injects a
corrective action.

**Why User Impact = 5:** Stuck loops are the most frequent failure mode in agentic systems.
When the agent gets stuck, it burns the full turn budget ($0.04–$0.20 per occurrence) without
making progress. Users notice immediately and lose trust.

**Why System Impact = 4:** Extends the existing `src/agent/self_repair.rs` (job-level
detection) with turn-level EWMA tracking. Hooks cleanly into the existing `CostGuard`
infrastructure.

**Why Ease = 4:** New file `src/agent/metacognitive.rs` (~300 lines of logic + tests). The
integration in `agentic_loop.rs` is 10–15 lines. No migrations, no new dependencies.

**Why Safety = 5:** Feature-flagged via `METACOGNITIVE_MONITOR_ENABLED`. When disabled, zero
code paths change. When enabled, adds one `if let Some(pathology)` branch per turn.

**Why Independence = 5:** Ships entirely alone. Uses existing `CostGuard` data but does not
require it — if CostGuard is absent, cost tracking simply reads from the turn's LLM response.

**Location:** `src/agent/metacognitive.rs` (new file)
**Estimated LOC:** 400–500 | **Estimated time:** 2–3 days

---

### Rank 2: Ebbinghaus Decay (Composite: 4.50)

**Scores:** User=4, System=4, Ease=5, Safety=5, Independence=5
**Source**: [../../core-concepts/universal-engram.md](../../core-concepts/universal-engram.md)
**Implementation**: See [implementation-sketches.md](implementation-sketches.md#rank-2-ebbinghaus-decay)

**What it is:** Applies the Ebbinghaus forgetting curve to workspace memory entries. Each
entry has a `stability_secs` value. Retrieval strength decays as `e^(-elapsed/stability)`.
Each access doubles stability, so frequently-accessed memories become effectively permanent
while one-time observations fade below the retrieval threshold.

**Why User Impact = 4:** After months of use, IronClaw's memory fills with stale observations
that pollute search results. Decay makes the memory self-curating without any user action.

**Why System Impact = 4:** The `DecayVariant` enum supports three strategies (None, HalfLife,
Ttl, Ebbinghaus) for future use. Integrates cleanly with the existing RRF search in
`src/workspace/search.rs`.

**Why Ease = 5:** New file `src/workspace/decay.rs` (~100 lines), a migration with 3 columns,
and a 5-line change to the search query to multiply scores by decay strength. Done in 1–2 days.

**Why Safety = 5:** Existing entries default to `DecayVariant::None` (always strength=1.0),
preserving all existing behavior. New entries get Ebbinghaus decay. Fully backward-compatible.

**Why Independence = 5:** Ships alone. Synergizes with BLAKE3 Dedup (#3) but does not require it.

**Location:** `src/workspace/decay.rs` (new); integrate into `src/workspace/repository.rs`
and `src/tools/builtin/memory.rs`. DB migration required (both PostgreSQL and libSQL).
**Estimated LOC:** 150 + migration | **Estimated time:** 1–2 days

---

### Rank 3: BLAKE3 Content Dedup (Composite: 4.30)

**Scores:** User=4, System=3, Ease=5, Safety=5, Independence=5
**Source**: [../../core-concepts/universal-engram.md](../../core-concepts/universal-engram.md)
**Implementation**: See [implementation-sketches.md](implementation-sketches.md#rank-3-blake3-content-dedup)

**What it is:** Before writing a memory entry via `memory_write`, compute its BLAKE3 content
hash. If an entry with the same hash already exists, merge metadata (bump access count, merge
tags) rather than inserting a duplicate. BLAKE3 is already a direct dependency (`blake3 = "1"`
in `Cargo.toml`).

**Why User Impact = 4:** In long-running use, the same fact is written to memory repeatedly
across sessions (e.g., "User prefers async Rust patterns"). Without dedup, search results
return 10 identical entries, wasting context window tokens and degrading answer quality.

**Why System Impact = 3:** Additive improvement to the write path only. Does not change
search or read paths for existing entries.

**Why Ease = 5:** New file `src/workspace/dedup.rs` (~80 lines). The integration is a
5-line check in the memory write handler. BLAKE3 is already in `Cargo.toml`. Migration
adds one nullable BYTEA column with a unique index.

**Why Safety = 5:** The dedup check is a purely additive gate before INSERT. If dedup fails
(bug in hash comparison), the fallback is a regular INSERT — no data is lost.

**Why Independence = 5:** Ships alone. Synergizes with Ebbinghaus Decay (#2) via the
access-bump mechanism.

**Location:** `src/workspace/dedup.rs` (new); modify `src/tools/builtin/memory.rs` and
`src/workspace/repository.rs`. DB migration required (both backends).
**Estimated LOC:** 100 + migration | **Estimated time:** 1 day

---

### Rank 4: Robust Statistics (Composite: 4.20)

**Scores:** User=3, System=4, Ease=5, Safety=5, Independence=5
**Source**: [../../core-concepts/mathematical-primitives.md](../../core-concepts/mathematical-primitives.md)
**Implementation**: See [implementation-sketches.md](implementation-sketches.md#rank-4-robust-statistics)

**What it is:** Pure functions replacing naive mean/standard-deviation computations in
`src/estimation/` and `src/evaluation/` with outlier-resistant alternatives. Zero new
dependencies. Functions: `median()`, `trimmed_mean()`, `mad()`, `hodges_lehmann()`,
`ewma_update()`, `robust_z_score()`.

**Why User Impact = 3:** Users notice estimation accuracy indirectly — the dashboard shows
"~29 seconds" instead of "~74 seconds" after an LLM timeout spike.

**Why System Impact = 4:** These functions become the canonical statistics primitives for
the entire codebase. Any future subsystem needing outlier-robust estimation uses them.
Eliminates a class of subtle estimation bugs permanently.

**Why Ease = 5:** Append to `src/util.rs`. No new modules, no migrations, no dependencies.
Done in 1–2 hours. Has the highest ROI-per-hour of any item in the matrix.

**Why Safety = 5:** Pure functions with no side effects. Call sites are opt-in replacements
for existing `mean()` calls. Zero risk to existing behavior.

**Why Independence = 5:** Fully standalone. Other features use them, but none require them.

**Location:** Append to `src/util.rs` (no new file needed).
**Estimated LOC:** 100–200 | **Estimated time:** 1 day

---

### Rank 5: Composable Scorers (Composite: 3.90)

**Scores:** User=2, System=4, Ease=5, Safety=5, Independence=5
**Source**: [../../agent-intelligence/agent-patterns.md](../../agent-intelligence/agent-patterns.md)
**Implementation**: See [implementation-sketches.md](implementation-sketches.md#rank-5-composable-scorers)

**What it is:** A `Scorer` trait with `WeightedScorer`, `ThresholdScorer`, and `ChainScorer`
combinators. Replaces ad-hoc scoring logic in `src/evaluation/` and `src/skills/` with a
reusable, testable, composable framework. Concrete scorers: `RecencyScorer`, `UtilityScorer`,
`PopularityScorer`, `TagScorer`.

**Why User Impact = 2:** Users do not interact with scorers directly. Value is in quality
improvements over time as scoring is unified.

**Why System Impact = 4:** Eliminates 5–8 scattered scoring implementations with one
canonical framework. New features (Gate Pipeline rungs, Cascade Router reward signals) can
reuse these primitives.

**Why Ease = 5:** New file `src/evaluation/scorer.rs` (~200 lines). Zero risk to existing
code until a call site is refactored. The first PR can ship with zero call-site changes.

**Why Safety = 5:** Purely additive. Existing ad-hoc scorers continue to work until they
are migrated (optional, incremental).

**Why Independence = 5:** Ships entirely alone.

**Location:** `src/evaluation/scorer.rs` (new file).
**Estimated LOC:** 200 | **Estimated time:** 1–2 days

---

### Rank 6: Hierarchical Cancellation (Composite: 3.50)

**Scores:** User=2, System=4, Ease=4, Safety=4, Independence=5
**Source**: [../../execution-verification/runtime-infrastructure.md](../../execution-verification/runtime-infrastructure.md)
**Implementation**: See [implementation-sketches.md](implementation-sketches.md#rank-6-hierarchical-cancellation)

**What it is:** Replace IronClaw's ad-hoc cancellation channels with structured
`tokio_util::sync::CancellationToken` trees. Session token is root, job tokens are children,
tool/LLM call tokens are grandchildren. Cancelling any ancestor cascades to descendants.

**Why User Impact = 2:** Users rarely notice cancellation correctness directly. The improvement
is in edge-case reliability: cancelled sessions that clean up properly instead of leaking tasks.

**Why System Impact = 4:** Structured cancellation eliminates the entire class of "orphaned
async task" bugs. Tokio task metrics become clean. Future features (parallel tool execution,
streaming responses) can use child tokens safely.

**Why Ease = 4:** New file `src/agent/cancel.rs` (~150 lines of wrappers). Integration
requires touching the session lifecycle and job spawning — not trivial, but well-understood.

**Why Safety = 4:** `tokio_util` is already a transitive dependency. The API is additive.
Risk: if cancellation tokens are attached to futures that don't poll them, they won't
cancel — requires careful integration testing.

**Why Independence = 5:** Ships alone; uses only `tokio_util::sync::CancellationToken`.

**Location:** `src/agent/cancel.rs` (new file); integrate into `src/agent/` and `src/context/`.
**Estimated LOC:** 200–300 | **Estimated time:** 3–5 days

---

## Big Bets Tier (Ranks 7–10): Build in Weeks 3–8

High value features requiring 2–4 weeks each. Worth planning carefully.

---

### Rank 7: Cascade Router (Composite: 4.15)

**Scores:** User=5, System=4, Ease=3, Safety=4, Independence=5
**Source**: [../../agent-intelligence/online-learning.md](../../agent-intelligence/online-learning.md)
**Implementation**: See [implementation-sketches.md](implementation-sketches.md#rank-7-cascade-router)

**What it is:** A 3-stage routing system: (1) static rules for known patterns, (2) rule
confidence gate, (3) LinUCB contextual bandit that learns which LLM model works best for
which request context. Extends `crates/ironclaw_llm/src/smart_routing.rs`.

**Why User Impact = 5:** 30–50% LLM cost reduction. For a user spending $100/month on LLM,
this saves $30–50/month — the largest potential saving of any single feature.

**Why Ease = 3:** ~1,500 lines across 4 files. LinUCB requires matrix operations (use `ndarray`).
Reward signal design requires careful thought. The cold-start problem (50+ rounds of warmup)
means the bandit falls back to static rules initially.

**Why Safety = 4:** The cascade design ensures the bandit is only used after warmup. Static
rules handle the first 50 requests. If the bandit makes a bad decision, the reward signal
corrects it in the next update.

**Location:** `crates/ironclaw_llm/src/router/` (new module with 4 files).
**Estimated LOC:** 1,500 | **Estimated time:** 2–3 weeks

---

### Rank 8: Cognitive Speed Labels (Composite: 3.50)

**Scores:** User=3, System=3, Ease=4, Safety=4, Independence=4
**Source**: [../../core-concepts/cognitive-architecture.md](../../core-concepts/cognitive-architecture.md)
**Implementation**: See [implementation-sketches.md](implementation-sketches.md#rank-8-cognitive-speed-labels)

**What it is:** Classify agent processing into three speed tiers — Gamma (reactive, ≤15s,
cheapest model), Theta (reflective, ~75s, standard model), Delta (consolidation, hours, best
model). Drives model selection in the Cascade Router.

**Why System Impact = 3:** Classification becomes a feature in the LinUCB bandit context
vector, helping convergence. Also useful as a logging dimension for debugging slow requests.

**Why Independence = 4:** Soft dependency on Cascade Router (#7) — the labels are most
valuable when driving routing decisions, but can ship alone as metadata.

**Location:** `src/agent/cognitive_speed.rs` (new file).
**Estimated LOC:** 200–300 | **Estimated time:** 3–5 days

---

### Rank 9: HDC Similarity Engine (Composite: 3.85)

**Scores:** User=4, System=4, Ease=3, Safety=4, Independence=5
**Source**: [../../core-concepts/hyperdimensional-computing/README.md](../../core-concepts/hyperdimensional-computing/README.md)
**Implementation**: See [implementation-sketches.md](implementation-sketches.md#rank-9-hdc-similarity-engine)

**What it is:** Encode semantic information into 10,240-bit binary hypervectors. Similarity
via Hamming distance — pure bitwise operations, no GPU, no model inference. Scanning 100K
HDC vectors takes under 1ms. Adds a third search signal to the existing FTS + embeddings RRF.

**Why User Impact = 4:** Compositional queries ("memories about Rust AND deployment") work
naturally in HDC via bind+search. Current FTS + embedding search handles them poorly.

**Why Ease = 3:** New crate `crates/ironclaw_hdc/` (~800–1,200 lines). The integration into
`src/workspace/search.rs` (adding HDC as a third RRF signal) is straightforward. The hard part
is the encoding pipeline — mapping text tokens to atomic hypervectors.

**Location:** New `crates/ironclaw_hdc/`.
**Estimated LOC:** 800–1,200 | **Estimated time:** 2–3 weeks

---

### Rank 10: Gate Verification Pipeline (Composite: 3.85)

**Scores:** User=4, System=4, Ease=3, Safety=4, Independence=5
**Source**: [../../execution-verification/gate-verification.md](../../execution-verification/gate-verification.md)
**Implementation**: See [implementation-sketches.md](implementation-sketches.md#rank-10-gate-verification-pipeline)

**What it is:** A four-rung verification pipeline for agent-generated code: (1) Compile,
(2) Lint, (3) Test, (4) Symbol resolution. Each rung shells out to the appropriate
language tool (initially Rust-only) and captures pass/fail with structured output.

**Why User Impact = 4:** Syntax errors and lint violations in generated code never reach
the user — the gate catches them first and the agent self-corrects.

**Why Ease = 3:** ~1,000 lines extending `src/tools/builder/validation.rs`. Async subprocess
management with timeouts. Structured output capture and storage.

**Location:** `src/tools/builder/gate.rs` (extends existing builder subsystem).
**Estimated LOC:** 1,000 | **Estimated time:** 2–4 weeks

---

## Nice to Have Tier (Ranks 11–17)

Build as convenient, between major projects. Most have soft dependencies on earlier features.

---

### Rank 11: Enhanced Heartbeat / Dream Consolidation Lite (Composite: 3.30)

**Source**: [../../agent-intelligence/dream-consolidation.md](../../agent-intelligence/dream-consolidation.md)

Extends `src/agent/heartbeat.rs` with NREM Replay (review recent interactions, strengthen
validated memories) and Threat Rehearsal (analyze recent failures, generate defensive strategies).
This is a lightweight version — it defers the expensive REM Imagination and Hypnagogic Creativity
phases until Enhanced Heartbeat proves its value in production.

**Why Ease = 3:** Background LLM calls need careful resource management (budget caps, timeout
handling) and all actions must go through `ToolDispatcher` per IronClaw's "everything through
tools" principle. The heartbeat scheduler is already in place; adding NREM/Rehearsal extends
it rather than replacing it.

**Why Safety = 3:** Background LLM calls introduce a new cost vector. Hard cap consolidation
budget at 5% of the user's monthly budget to mitigate.

**Hard dependency:** Requires Ebbinghaus Decay (#2) — replay strengthens memories by bumping
access counts, which only makes sense if decay exists.

**Location:** Extend `src/agent/heartbeat.rs`, new `src/consolidation/`.
**Estimated LOC:** 800 | **Estimated time:** 2–3 weeks

---

### Rank 12: DAG Execution Engine (Composite: 3.35)

**Source**: [../../execution-verification/dag-execution.md](../../execution-verification/dag-execution.md)

A directed acyclic graph executor where nodes are "Cells" (LLM calls, tool invocations, shell
commands) and edges define data flow with conditional routing. Defined in TOML, executed with
budget tracking. Supports parallel execution of independent cells and hot graphs (resident,
tick-driven workflows).

**Why User Impact = 4:** Multi-step tasks currently execute as sequential agent turns. A DAG
engine allows parallel execution of independent steps, reducing wall-clock time significantly.
Users can define custom workflows in TOML without writing Rust.

**Why Ease = 2 (Hard):** ~2,500 lines. All cells must dispatch through `ToolDispatcher`. Requires
topological sort implementation and careful error propagation across concurrent cell executions.

**Location:** New `crates/ironclaw_graph/`.
**Estimated LOC:** 2,500 | **Estimated time:** 4–8 weeks

---

### Rank 13: EventBus with Replay Ring (Composite: 3.25)

**Source**: [../../execution-verification/runtime-infrastructure.md](../../execution-verification/runtime-infrastructure.md)

A publish-subscribe event system with a bounded circular buffer. Events get monotonically
increasing sequence numbers. New subscribers replay from a specific sequence number to catch up.
Enables real-time web UI updates via SSE, background task coordination, and structured observability.

**Why User Impact = 2:** Users do not interact with the event bus directly. Value is in enabling
other features cleanly.

**Why System Impact = 4:** Replaces ad-hoc channels for inter-component communication with a
single, consistent, observable mechanism.

**Location:** New `src/events/` or `crates/ironclaw_events/`.
**Estimated LOC:** 400–500 | **Estimated time:** 2–3 weeks

---

### Rank 14: Conductor (Composite: 3.25)

**Source**: [../../execution-verification/conductor-anomaly.md](../../execution-verification/conductor-anomaly.md)

Ensemble of 10 health watchers (latency, error rate, token usage, cost, quality, throughput,
saturation, availability, drift, coherence) with predictive circuit breaking via Holt double
exponential smoothing. IronClaw already has a reactive `CircuitBreakerProvider`; the Conductor
replaces it with predictive breaking.

**Why Ease = 2 (Hard):** ~2,000+ lines of stateful monitoring code running continuously alongside
the agent. Thompson Sampling for threshold learning adds statistical complexity.

**Soft dependency:** Benefits from Cascade Router (#7) — without a router, the Conductor
can trip circuit breakers but cannot gracefully route to alternatives.

**Location:** New `crates/ironclaw_conductor/`.
**Estimated LOC:** 2,000+ | **Estimated time:** 4–6 weeks

---

### Rank 15: Resumable Checkpoints (Composite: 3.20)

**Source**: [../../agent-intelligence/agent-patterns.md](../../agent-intelligence/agent-patterns.md)

Serialize agent state at key points so that if the agent crashes mid-turn, it can resume from
the last checkpoint. Practical approach: checkpoint only conversation history and pending tool
calls (not in-memory caches or pending futures).

**Why User Impact = 3:** Crashes during long-running jobs lose all progress. However, crashes
are relatively rare in a stable Rust system.

**Why Ease = 3:** The challenge is serialization — agent state includes references to external
resources. A conservative first version checkpoints only conversation history.

**Location:** `src/agent/checkpoint.rs`, `src/db/` (migration).
**Estimated LOC:** 300–400 | **Estimated time:** 2–3 weeks

---

### Rank 16: Declarative TOML Tools (Composite: 3.15)

**Source**: [../../ecosystem/plugin-extension.md](../../ecosystem/plugin-extension.md)

Define simple tools in TOML without Rust or WASM. A TOML file specifies tool name, description,
parameter schema, and execution method (HTTP call, shell command, or chained tool invocation).
Lowers the barrier for custom tools significantly.

**Why User Impact = 3:** Currently, custom tools require WASM or MCP. TOML tools make simple
integrations (webhook calls, shell wrappers) accessible without compilation.

**Why System Impact = 2:** Additive feature; does not improve existing infrastructure.

**Location:** `src/tools/declarative/` (new module).
**Estimated LOC:** 400–500 | **Estimated time:** 2–3 weeks

---

### Rank 17: User Engagement PAD Tracker (Composite: 3.00)

**Source**: [../../agent-intelligence/affect-engine.md](../../agent-intelligence/affect-engine.md)

A lightweight Pleasure-Arousal-Dominance tracker estimating the user's engagement state from
interaction patterns. Modulates response style: frustrated users get step-by-step responses;
confident power users get concise answers. Uses three running metrics (response delay, message
length trend, question rate) to infer PAD dimensions.

**Why Safety = 3:** Misclassification (detecting frustration when the user is fine) could make
responses worse. Mitigation: use PAD as a soft bias on response style, not a hard switch.

**Location:** Extend `src/profile.rs` (existing psychographic profile), `crates/ironclaw_engine/`.
**Estimated LOC:** 300–400 | **Estimated time:** 2–3 weeks

---

## Long-Term Tier (Ranks 18–25): Research Phase

Prerequisites missing or value uncertain. Build only after earlier tiers are validated.

| Rank | Concept | Composite | Why It Ranks Lower |
|------|---------|-----------|-------------------|
| 18 | Full Dream Consolidation | 2.80 | Depends on Enhanced Heartbeat (#11) + HDC (#9); signal-to-noise for LLM "insights" is uncertain; high ongoing LLM cost |
| 19 | Budget Composition (VCG) | 2.75 | High integration complexity with prompt composition in `crates/ironclaw_engine/`; progressive tool disclosure already partially implemented |
| 20 | Code Intelligence | 2.65 | Requires HDC (#9) + tree-sitter; 2,500+ lines; valuable only for users working on larger codebases |
| 21 | NEAR On-Chain Reputation | 2.30 | Smart contract development + auditing is expensive; depends on NEAR ecosystem adoption; start with off-chain reputation first |
| 22 | Pheromone System | 2.25 | Requires multi-agent support not yet in IronClaw; in single-agent context, equivalent to tagged memories with decay |
| 23 | Pure SM Extraction | 2.20 | Major refactor of `src/agent/agentic_loop.rs` (most complex code in IronClaw); holy grail of testability but extremely risky; incremental approach required |
| 24 | Full Affect Engine | 2.10 | k-d tree somatic markers, 3-layer temporal tracking, Nietzsche vitality phases — diminishing returns over lightweight PAD tracker (#17) |
| 25 | TDA / Sheaves | 1.55 | Research-grade mathematics (persistent homology, cellular sheaf cohomology) with negligible practical benefit for a personal AI assistant |

---

## Moonshots

High-risk, high-reward items that would be transformative if they work.
These are research projects, not engineering tasks.

### Moonshot 1: Full Dream Consolidation (Rank 18)

**Source**: [../../agent-intelligence/dream-consolidation.md](../../agent-intelligence/dream-consolidation.md)

**Potential reward:** An agent that proactively generates insights by connecting knowledge across
domains. Example: "I noticed your deployments consistently fail on Thursdays between 3–5 PM UTC.
This correlates with the EU traffic spike you mentioned in session 47 last month."

**Why risky:** Signal-to-noise ratio for LLM-generated "insights" is likely very low (5–20%
genuine insight rate). High ongoing LLM cost for background calls. No ground truth for "is
this insight useful?" Requires Ebbinghaus Decay + Enhanced Heartbeat as prerequisites.

**Gate strategy:** Build Enhanced Heartbeat first and run it for 4 weeks. If NREM Replay and
Threat Rehearsal produce measurable improvements in future task performance, extend to REM
Imagination. Each step is a go/no-go checkpoint with a 4-week validation window.

**Prerequisite trail:** Ebbinghaus Decay (#2) → Enhanced Heartbeat (#11) → HDC Similarity (#9)
→ Full Dream Consolidation (#18).

---

### Moonshot 2: Pure State Machine Extraction (Rank 23)

**Source**: [../../execution-verification/runtime-infrastructure.md](../../execution-verification/runtime-infrastructure.md)

**Potential reward:** A fully testable, replayable agent loop. Any bug is reproducible by
feeding recorded events through the pure state machine. Eliminates the hardest class of
agent loop bugs: those that depend on timing, interleaving, or external state.

**Why risky:** `src/agent/agentic_loop.rs` is the most complex code in IronClaw. Refactoring
it while maintaining backward compatibility and full test coverage requires enormous discipline.
The estimated 3,000+ LOC is conservative — the refactoring may require touching every subsystem
that the agentic loop touches.

**Gate strategy:** Extract one effect at a time. Start with `Effect::Emit` (the simplest).
Validate that the pure state machine produces the same effects as the original for every
existing test case. Do not proceed to the next effect until the previous one is fully validated.
Estimated total timeline: 6–12 months at an incremental pace.

---

### Moonshot 3: NEAR On-Chain Reputation (Rank 21)

**Source**: [../../ecosystem/chain-reputation/README.md](../../ecosystem/chain-reputation/README.md)

**Potential reward:** Verifiable, portable agent identity and reputation. Agents prove their
track record when interacting with other agents or marketplace participants. A "trust score"
that follows the agent across deployments and counterparties.

**Why risky:** Smart contract development requires specialized expertise. Auditing costs
$20K–$100K per contract for production deployments. Token economics add legal and design
complexity. Requires NEAR ecosystem adoption to have value.

**Gate strategy:** Start with off-chain reputation tracking. Build the 7-domain EMA scoring
engine in `crates/ironclaw_reputation/` using local storage. Once validated and stable with
real user data (6–12 months), deploy a minimal smart contract for reputation anchoring.

---

## Implementation Location Reference

| Rank | Concept | IronClaw Location | New Crate? | DB Migration? | Est. LOC |
|------|---------|------------------|------------|---------------|----------|
| 1 | Metacognitive Monitor | `src/agent/metacognitive.rs` | No | No | 400–500 |
| 2 | Ebbinghaus Decay | `src/workspace/decay.rs` | No | Yes (both) | 150 + migration |
| 3 | BLAKE3 Dedup | `src/workspace/dedup.rs` + `src/tools/builtin/memory.rs` | No | Yes (both) | 100 + migration |
| 4 | Robust Statistics | `src/util.rs` | No | No | 100–200 |
| 5 | Composable Scorers | `src/evaluation/scorer.rs` | No | No | 200 |
| 6 | Hier. Cancellation | `src/agent/cancel.rs` | No | No | 200–300 |
| 7 | Cascade Router | `crates/ironclaw_llm/src/router/` | Optional | Yes (episodes) | 1,500 |
| 8 | Cognitive Speed Labels | `src/agent/cognitive_speed.rs` | No | No | 200–300 |
| 9 | HDC Similarity | `crates/ironclaw_hdc/` | Yes | Yes (fingerprints) | 800–1,200 |
| 10 | Gate Pipeline | `src/tools/builder/gate.rs` | No (initial) | No | 1,000 |
| 11 | Enhanced Heartbeat | `src/agent/heartbeat.rs` ext. + `src/consolidation/` | No | Optional | 800 |
| 12 | DAG Execution | `crates/ironclaw_graph/` | Yes | No | 2,500 |
| 13 | EventBus | `src/events/` | Optional | No | 400–500 |
| 14 | Conductor | `crates/ironclaw_conductor/` | Yes | No | 2,000+ |
| 15 | Resumable Checkpoints | `src/agent/checkpoint.rs` | No | Yes | 300–400 |
| 16 | Decl. TOML Tools | `src/tools/declarative/` | No | No | 400–500 |
| 17 | User Engagement PAD | `src/profile.rs` ext. | No | No | 300–400 |
| 18 | Full Dream Consol. | `crates/ironclaw_dreams/` | Yes | Yes | 1,500 |
| 19 | Budget Composition | `crates/ironclaw_engine/` ext. | No | No | 1,500 |
| 20 | Code Intelligence | `crates/ironclaw_index/` | Yes | Yes | 2,500+ |
| 21 | NEAR Reputation | `crates/ironclaw_reputation/` | Yes | Yes | 1,100–1,500 |
| 22 | Pheromone System | `src/workspace/pheromone.rs` | No | Optional | 400–500 |
| 23 | Pure SM Extraction | `src/agent/` refactor | No | No | 3,000+ |
| 24 | Full Affect Engine | `src/affect/` | No | No | 1,000–1,500 |
| 25 | TDA / Sheaves | `crates/ironclaw_math/` | Yes | No | 2,000+ |

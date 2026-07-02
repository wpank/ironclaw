# Integration Roadmap: Roko Concepts into IronClaw

## Executive Summary

### What This Document Is

This is a phased engineering plan for adopting novel concepts from the [roko](https://github.com/nunchi/roko) codebase into IronClaw, a secure personal AI assistant built in Rust. Each concept has been analyzed in its own companion document (`01-*.md` through `17-*.md` in this directory), with supplementary analysis in documents `19-*.md` through `28-*.md`. This roadmap synthesizes those analyses into a concrete, sequenced implementation plan with specific files, trait signatures, test strategies, rollback procedures, and measurable success criteria for each phase.

### What IronClaw Is (For First-Time Readers)

IronClaw is a multi-channel AI assistant platform with these core properties:

- **Multi-provider LLM integration** (`crates/ironclaw_llm/`): Supports NEAR AI, OpenAI, Anthropic, Ollama, AWS Bedrock, GitHub Copilot, Tinfoil, and OpenAI-compatible endpoints. Provider selection, retry, circuit breaking, failover, and response caching are all handled by a decorator chain. A 13-dimension `SmartRoutingProvider` already classifies request complexity into four tiers (Flash/Standard/Pro/Frontier) and routes cheap vs primary models.
- **Multi-channel input** (`src/channels/`): CLI/TUI, web gateway (browser UI), HTTP webhooks, WASM channels. Channels are interchangeable; all dispatch through the same tool pipeline.
- **Tool system** (`src/tools/`): Built-in Rust tools, WASM-sandboxed tools (wasmtime), and MCP (Model Context Protocol) servers. Every action goes through `ToolDispatcher::dispatch()` for audit, safety, and policy enforcement.
- **Workspace memory** (`src/workspace/`): Path-based persistent memory with hybrid search (FTS5 + vector embeddings via RRF merge). Four tools: `memory_search`, `memory_write`, `memory_read`, `memory_tree`. Identity files (AGENTS.md, SOUL.md, USER.md, IDENTITY.md) are injected into the system prompt. Documents are stored as `MemoryDocument` structs with `path`, `content`, `metadata` (JSON), and timestamps.
- **Agent loop** (`src/agent/`): Session/Thread/Turn model. Three execution delegates (ChatDelegate, JobDelegate, ContainerDelegate) share `run_agentic_loop()`. Self-repair detects stuck jobs. Cost guard enforces daily budget and hourly rate limits. A `DuplicateToolCallTracker` already detects consecutive identical failing tool call batches.
- **Estimation and evaluation** (`src/estimation/`, `src/evaluation/`): EMA-based learning for cost/time estimation. Rule-based and LLM-based success evaluation.
- **Heartbeat system** (`src/agent/heartbeat.rs`): Proactive periodic execution (default 30 min). Reads HEARTBEAT.md, runs an agent turn, notifies user only if action is needed.
- **Dual-backend persistence** (`src/db/`): PostgreSQL + libSQL/Turso. All new features must support both.
- **Security-first** (`crates/ironclaw_safety/`): Prompt injection detection, validation, leak detection, policy enforcement. Tool outputs pass through a safety pipeline before returning to the LLM.

The crate workspace currently includes 60+ extracted crates under `crates/` covering everything from the agent loop to webhooks to the TUI.

### What Roko Is

Roko is a research-grade AI agent framework with 18+ crates and 155+ depth research documents. It explores biologically-inspired and mathematically-grounded approaches to agent intelligence: hyperdimensional computing for fast similarity, dream consolidation for offline learning, contextual bandits for model routing, DAG-based workflow execution, gate verification pipelines, and more. Many of these concepts are mature enough to adopt into a production system. See companion document `26-roko-architecture-overview.md` for the full system walkthrough, and `27-v2-depth-research.md` for the 155 depth documents.

### Why Adopt Roko Concepts

IronClaw has solid production infrastructure but several gaps that roko's research directly addresses:

1. **Cost optimization**: IronClaw's `SmartRoutingProvider` uses a static 13-dimension scorer. Roko's LinUCB contextual bandit _learns_ which model works for which task, delivering 30-50% cost reduction. See `07-online-learning.md`.
2. **Agent reliability**: Stuck loops are a common failure mode. IronClaw's `DuplicateToolCallTracker` (in `agentic_loop.rs`) detects consecutive identical _failing_ tool call batches, but does not detect broader repetitive patterns (e.g., alternating between two failing strategies) or cost runaway within a turn. Roko's metacognitive monitor covers these additional cases. See `17-agent-patterns.md`.
3. **Memory quality**: IronClaw memories never decay or deduplicate. Over time, the workspace accumulates redundant entries that dilute search quality. See `10-universal-engram.md`.
4. **Background intelligence**: IronClaw's heartbeat reads a checklist. Roko's dream consolidation actively replays experiences, strengthens patterns, and generates insights. See `02-dream-consolidation.md`.
5. **Verification**: IronClaw's tool builder has basic validation. Roko's gate pipeline provides 7 rungs of progressive verification with adaptive thresholds. See `05-gate-verification.md`.
6. **Observability**: No predictive health monitoring. Roko's conductor ensemble provides 10 watchers with Holt forecasting and predictive circuit breaking. See `06-conductor-anomaly.md`.

### Phase Overview at a Glance

| Phase | Theme | Items | Timeline | Total Effort |
|-------|-------|-------|----------|-------------|
| **1** | Quick Wins | 4 independent items | 2-3 weeks | 6-10 dev-days |
| **2** | Core Enhancements | 4 items, some Phase 1 deps | 6-9 weeks | 23-36 dev-days |
| **3** | Architecture Evolution | 4 items, Phase 2 deps | 9-14 weeks | 35-51 dev-days |
| **4** | Advanced Features | 5 items, external deps | Ongoing | 52-87 dev-days |

### Expected Outcome

At full adoption of Phases 1-3 (approximately 6-9 months of engineering):

- **30-50% LLM cost reduction** through bandit-based model routing
- **Near-zero stuck agent loops** through enhanced metacognitive monitoring
- **Higher memory precision** through deduplication and decay
- **Proactive insight generation** through dream consolidation
- **Structured code verification** for generated code
- **Predictive health monitoring** for LLM providers
- **Declarative workflow execution** replacing ad-hoc job chaining

---

## Background: The Roko Concept Catalog

Before diving into phasing, here is a summary of every concept analyzed. Each references its companion document for full detail.

| # | Concept | Companion Doc | One-Line Summary |
|---|---------|---------------|------------------|
| 01 | Hyperdimensional Computing | `01-hyperdimensional-computing.md` | 10,240-bit binary vectors for O(nanosecond) semantic similarity via XOR/popcount |
| 02 | Dream Consolidation | `02-dream-consolidation.md` | Five-subsystem offline learning: NREM replay, REM imagination, hypnagogic creativity, threat rehearsal, staging buffer |
| 03 | Affect Engine | `03-affect-engine.md` | PAD (Pleasure-Arousal-Dominance) emotional vectors modulating agent behavior across three temporal layers |
| 04 | DAG Execution | `04-dag-execution.md` | TOML-defined workflow DAGs with Cell registry, conditional edges, budget tracking, hot graphs |
| 05 | Gate Verification | `05-gate-verification.md` | 7-rung progressive verification (Compile through Integration) with adaptive thresholds (CUSUM/EWMA/BOCPD) |
| 06 | Conductor Anomaly Detection | `06-conductor-anomaly.md` | 10-watcher ensemble with Holt exponential smoothing, compound pattern detection, Thompson Sampling thresholds |
| 07 | Online Learning / Cascade Router | `07-online-learning.md` | LinUCB contextual bandit with 18D feature vector, 3-stage cascade, Pareto frontier optimization |
| 08 | On-Chain Reputation | `08-chain-reputation.md` | Soulbound passports, 7-domain EMA reputation, bounty marketplace, TraceRank collusion detection |
| 09 | Budget-Constrained Composition | `09-budget-composition.md` | VCG auction for prompt assembly, 9-layer cache-aware builder, Thompson Sampling bidders |
| 10 | Universal Engram | `10-universal-engram.md` | Content-addressed data type with 7-axis scoring (Confidence/Novelty/Utility/Reputation/Precision/Salience/Coherence), 4 decay variants, taint propagation |
| 11 | Mathematical Primitives | `11-mathematical-primitives.md` | TDA, cellular sheaves, Riemannian geometry, tropical algebra, robust statistics (trimmed mean, MAD) |
| 12 | Code Intelligence | `12-code-intelligence.md` | Four-mode indexing (Symbol, Graph/PageRank, HDC fingerprint, FTS5) with RRF merge and privacy overlays |
| 13 | Cognitive Architecture | `13-cognitive-architecture.md` | Three cognitive speeds (Gamma/Theta/Delta), five-layer architecture (L0-L4), stigmergic pheromone coordination, C-factor |
| 14 | Runtime Infrastructure | `14-runtime-infrastructure.md` | EventBus with replay ring, hierarchical cancellation tokens, FIPA lifecycle, pure state machine + effect driver |
| 15 | Orchestrator & Swarm | `15-orchestrator-swarm.md` | Event-sourced state machine, cross-plan task DAG, wave scheduling, 3-level recovery, BLAKE3 audit chain |
| 16 | Plugin & Extension | `16-plugin-extension.md` | EventSource/FeedbackCollector traits, 4-tier extensibility, TOML declarative tools, hot-reload |
| 17 | Agent Patterns | `17-agent-patterns.md` | Translator pattern, stream reassembly, resumable checkpoints, metacognitive monitor, harness adapter, composable scorers |

### Supplementary Documents

| # | Title | Summary |
|---|-------|---------|
| 19 | Priority Matrix | Impact vs. effort quadrant ranking of all 25 concepts. ROI formula and scoring rubric. |
| 20 | Persistence & Storage | Roko's `roko-fs` append-only JSONL persistence layer. |
| 21 | MCP & Editor Integration | ACP protocol and 5 MCP server crates. |
| 22 | Language Support | Multi-language code analysis (`roko-lang-rust`, `-typescript`, `-go`). |
| 23 | Control Plane | `roko-serve` and `roko-agent-server` hub-and-spoke architecture. |
| 24 | Smart Contracts | 13 Solidity contracts for on-chain AI agent infrastructure. |
| 25 | Research Citations | Comprehensive bibliography of all academic references in roko. |
| 26 | Architecture Overview | Full roko system architecture walkthrough. |
| 27 | v2-Depth Research | Catalog of 155 depth documents across 23 thematic sections. |
| 28 | Plans Catalog | Every implementation plan in roko's `plans/` directory. |

---

## Phase 1: Quick Wins (1-2 Weeks Each, Independent)

These four items can be implemented in any order. Each is self-contained, touches a narrow area of the codebase, and delivers immediate value with minimal risk.

**Phasing rationale**: These are all low-effort (100-500 lines), high-confidence changes that improve existing behavior without introducing new subsystems, new crates, or new database schemas (except one small migration). They serve as warm-up work that familiarizes the team with the roko concepts before tackling the larger Phase 2 items.

---

### Phase 1.1: Robust Statistics

**What we are building**: Replacing naive mean and standard deviation calculations with outlier-resistant alternatives (trimmed mean and Median Absolute Deviation) in the estimation and evaluation subsystems.

**Why it matters**: LLM response times and costs are heavy-tailed distributions. A single extremely slow response or an anomalous cost spike can distort the mean by 50%+. The current `EstimationLearner` in `src/estimation/learner.rs` uses a plain EMA with a fixed `alpha = 0.1` that converges slowly in the presence of outliers. Trimmed mean removes the top and bottom k% of observations before averaging. MAD (Median Absolute Deviation) is a robust spread measure: `MAD = median(|x_i - median(x)|)`.

**Source concept**: Doc 11 (Mathematical Primitives), "Robust Statistics" section.

**IronClaw files affected**:

| File | Change |
|------|--------|
| `src/estimation/learner.rs` | Replace raw ratio computation with trimmed ratios; add MAD-based outlier detection before EMA update |
| `src/estimation/cost.rs` | Use trimmed mean for tool cost aggregation |
| `src/estimation/time.rs` | Use trimmed mean for tool time aggregation |
| `src/evaluation/metrics.rs` | Use MAD for quality score spread measurement |
| `src/util.rs` | Add `trimmed_mean()`, `mad()`, `median()` utility functions (private to crate, not re-exported) |

**Step-by-step implementation**:

1. **Write tests first** (per IronClaw testing discipline). Add functions to `src/util.rs` with inline test module:
   - `fn trimmed_mean(values: &[f64], trim_fraction: f64) -> f64` -- verify that `trimmed_mean(&[1,2,3,100], 0.25) == 2.5` (discards 100)
   - `fn mad(values: &[f64]) -> f64` -- verify against known values
   - `fn median(values: &[f64]) -> f64` -- basic median for odd/even length
   - Edge cases: empty slice (return 0.0), single element, all identical values

2. **Implement the functions** in `src/util.rs`. Approximately 80-120 lines:
   ```rust
   /// Trimmed mean: sort values, discard the top and bottom `trim_fraction`
   /// of observations, and average the remainder.
   pub(crate) fn trimmed_mean(values: &[f64], trim_fraction: f64) -> f64 {
       if values.is_empty() { return 0.0; }
       let n = values.len();
       let trim = (n as f64 * trim_fraction.clamp(0.0, 0.49)) as usize;
       let mut sorted = values.to_vec();
       sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
       let trimmed = &sorted[trim..n.saturating_sub(trim)];
       if trimmed.is_empty() { return 0.0; }
       trimmed.iter().sum::<f64>() / trimmed.len() as f64
   }

   /// Median Absolute Deviation: a robust spread estimator.
   /// MAD = median(|x_i - median(x)|)
   pub(crate) fn mad(values: &[f64]) -> f64 {
       if values.is_empty() { return 0.0; }
       let med = median(values);
       let deviations: Vec<f64> = values.iter().map(|v| (v - med).abs()).collect();
       median(&deviations)
   }
   ```

   Note: These are `pub(crate)` (not `pub`), following the "No `pub use` re-exports unless exposing to downstream consumers" rule. No `mod stats;` or re-export needed.

3. **Wire into `EstimationLearner::record()`** in `src/estimation/learner.rs`:
   - The `LearningModel` currently stores `cost_factor` and `time_factor` as simple EMA-updated multipliers. Before computing cost_ratio and time_ratio, check if the ratio is an outlier (> 3 MAD from the running median). If so, dampen the EMA alpha for this update (e.g., use `alpha * 0.1` instead of `alpha`).
   - This requires tracking a small window of recent ratios (e.g., `VecDeque<f64>` with capacity 50) per `LearningModel`.
   - This prevents a single $50 anomalous LLM call from distorting the cost_factor for 20+ subsequent estimates.

4. **Wire into `src/evaluation/metrics.rs`**: Replace any `f64` standard deviation calculations with MAD for quality score spread measurement.

**Testing strategy**:
- Unit tests on `util.rs` functions: 8-10 tests covering normal, edge, and adversarial inputs
- Extend the existing estimation tests in `learner.rs` with an outlier scenario: inject one `actual_cost = 10000x estimated` and verify the cost_factor does not spike
- No integration test needed -- this is pure computation

**Rollback plan**: Revert the commit. No database changes, no config changes, no external dependencies. The `LearningModel` struct gains one field (`recent_ratios: VecDeque<f64>`); removing it is a single-field deletion.

**Success criteria**:
- [ ] `cargo test` passes with all new and existing estimation tests
- [ ] `cargo clippy --all --benches --tests --examples --all-features` has zero warnings
- [ ] Outlier scenario test: injecting 10x cost spike dampens EMA update to < 10% of undampened value
- [ ] Manual verification: run the agent, check that estimation outputs are reasonable after injecting a few extreme values

**Risks and mitigations**:

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Trimmed mean discards valid data points | Low | Low | Use conservative trim fraction (10-15%), not 25% |
| Performance regression from sorting | Very Low | None | Sorting 10-100 values is nanoseconds; these arrays are tiny |

**Estimated effort**: 1-2 developer-days.

---

### Phase 1.2: Metacognitive Monitor (Enhancement)

**What we are building**: Enhancing the existing `DuplicateToolCallTracker` in `src/agent/agentic_loop.rs` with two capabilities it currently lacks: (a) broader repetition detection beyond consecutive identical batches (e.g., alternating between two failing strategies), and (b) cost runaway detection that projects whether the current turn will exhaust its budget before completion.

**What already exists**: IronClaw already has `DuplicateToolCallTracker` in `src/agent/agentic_loop.rs`. It tracks consecutive identical failing tool call batches using fingerprint hashing (tool name + canonicalized args). When 3+ consecutive identical failing batches are detected, it injects a warning message asking the LLM to change strategy. This is effective for the simplest stuck pattern (exact repetition) but misses:
- **Alternating patterns**: Agent tries strategy A, fails, tries strategy B, fails, goes back to A -- neither A nor B repeats consecutively, but the agent is stuck.
- **Semantic repetition**: Agent calls `shell` with slightly different arguments each time but is fundamentally doing the same thing (e.g., adding different import lines to fix the same missing symbol).
- **Cost runaway**: Agent is making progress but spending tokens at a rate that will exhaust the budget before task completion.

The existing `CostGuard` in `src/agent/cost_guard.rs` enforces a hard daily budget and hourly call rate but does not project whether the current turn's spend trajectory will exhaust the per-turn allocation.

**Source concept**: Doc 17 (Agent Patterns), "Metacognitive Monitor" section.

**IronClaw files affected**:

| File | Change |
|------|--------|
| `src/agent/agentic_loop.rs` | Extend `DuplicateToolCallTracker` into `MetacognitiveMonitor`; add window-based diversity tracking and cost projection |
| `src/agent/cost_guard.rs` | Expose remaining budget for projection (`remaining_daily_budget_cents()`) |

**Step-by-step implementation**:

1. **Write tests first**. Extend the existing tests in `src/agent/agentic_loop.rs`:

   ```rust
   #[test]
   fn test_diversity_detection_alternating_pattern() {
       let mut monitor = MetacognitiveMonitor::new(10);
       let fp_a = 111;
       let fp_b = 222;
       // Alternate between two patterns -- not caught by consecutive tracker
       for _ in 0..5 {
           monitor.record_fingerprint(fp_a, true);
           monitor.record_fingerprint(fp_b, true);
       }
       // 10 iterations, only 2 unique fingerprints = 20% diversity
       assert!(monitor.is_low_diversity()); // < 33% unique
   }

   #[test]
   fn test_cost_runaway_projection() {
       let mut monitor = MetacognitiveMonitor::new(10);
       monitor.set_budget_cents(100); // $1 budget
       for _ in 0..5 {
           monitor.record_spend_cents(15); // 15 cents each
       }
       // 5 iterations spent 75 cents; projecting 10 total = 150 cents > 100 budget
       assert!(monitor.is_cost_runaway());
   }
   ```

2. **Evolve `DuplicateToolCallTracker` into `MetacognitiveMonitor`** (~150 lines added):

   ```rust
   struct MetacognitiveMonitor {
       // Existing consecutive duplicate detection (unchanged)
       last_fingerprint: Option<u64>,
       consecutive_dup_count: u32,
       // New: sliding window of recent fingerprints for diversity analysis
       recent_fingerprints: VecDeque<u64>,
       window_size: usize,
       // New: cost projection
       total_spend_cents: u32,
       iteration_count: u32,
       budget_cents: Option<u32>,
   }
   ```

   - `is_low_diversity()`: returns true when unique fingerprints < 33% of window size
   - `is_cost_runaway()`: returns true when projected total (`spend / iterations * max_iterations`) > 90% of budget
   - The existing `record_with_fingerprint()` method continues to work unchanged for the consecutive duplicate path

3. **Wire into `run_agentic_loop()`**: After tool execution, in addition to the existing duplicate detection, check:
   ```rust
   if monitor.is_low_diversity() {
       // Inject a stronger intervention message than the duplicate warning
       reason_ctx.messages.push(ChatMessage::user(
           "I notice I've been cycling between the same few approaches. \
            Let me step back and consider a fundamentally different strategy."
       ));
   }
   if monitor.is_cost_runaway() {
       tracing::debug!(
           spent = monitor.total_spend_cents,
           projected = monitor.projected_total_cents(),
           "metacognitive: cost runaway detected"
       );
       // Optionally: signal the delegate to downgrade model tier
   }
   ```

4. **Do NOT abort the loop on detection** -- instead, inject a reflection prompt. This gives the LLM a chance to self-correct before we force-terminate. The existing consecutive duplicate path already uses this pattern.

**Testing strategy**:
- Extend the existing tests in `agentic_loop.rs` (consolidate, don't proliferate)
- Add the alternating-pattern and cost-runaway unit tests above
- The existing `test_duplicate_failing_tool_calls_inject_warning` integration test validates the consecutive path; add one more that validates the diversity path with `MockDelegate`

**Rollback plan**: Revert to the original `DuplicateToolCallTracker`. The monitor is a strict superset -- removing the new fields returns to baseline behavior.

**Success criteria**:
- [ ] `cargo test` passes with all existing and new agentic loop tests
- [ ] Zero clippy warnings
- [ ] Alternating stuck pattern (A-B-A-B-A-B...) detected within 10 iterations
- [ ] Cost runaway detected when projected spend exceeds 90% of budget
- [ ] Existing consecutive duplicate detection still works (no regression)

**Risks and mitigations**:

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| False positive: legitimate repetition (e.g., retrying with different params) flagged as low diversity | Medium | Low | The diversity threshold is < 33% unique in a window of 10. Legitimate retries typically vary parameters enough to produce different fingerprints (the canonicalization is on full tool name + args). |
| Reflection injection disrupts the agent's reasoning | Low | Medium | The reflection message is phrased as a gentle suggestion, not a hard override. The LLM can ignore it if the repetition is intentional. This matches the existing pattern for consecutive duplicate warnings. |
| CostRunaway projection is too simple (linear) | Low | Low | Linear projection is a floor; we can upgrade to EMA-based projection in Phase 2 if needed. Roko uses a similar linear projection as the baseline. |

**Estimated effort**: 1-2 developer-days.

---

### Phase 1.3: Ebbinghaus Decay for Memory

**What we are building**: A forgetting curve for workspace memory entries. Memories that are accessed frequently become stronger (longer retention). Memories that are never accessed gradually lose strength and eventually fall below a retrieval threshold. This implements the Ebbinghaus spacing effect: each access doubles the memory's stability period.

**Why it matters**: IronClaw's workspace memories are permanent by default. Over months of use, a user's workspace accumulates hundreds or thousands of memory entries, many of which are stale, irrelevant, or superseded. When `memory_search` returns results, stale entries compete with relevant ones, diluting search quality. Decay ensures the workspace stays "sharp" -- frequently-used knowledge stays strong, while rarely-used knowledge fades naturally.

Critically, this does NOT violate IronClaw's "LLM data is never deleted" invariant. Decayed memories are _archived_ (marked with an `archived` status in their metadata and excluded from default search results), not deleted. They remain in the database and can be recovered.

**Source concept**: Doc 10 (Universal Engram), "Ebbinghaus Forgetting Curve" section. See also Doc 25 (Research Citations) for the original Ebbinghaus reference (1885/1913).

**IronClaw files affected**:

| File | Change |
|------|--------|
| `src/workspace/document.rs` | Add `DecayVariant` enum and helper methods |
| `src/workspace/mod.rs` | Update `write()` to set default decay metadata; add `archive_decayed()` method |
| `src/workspace/repository.rs` | Update read queries to bump `last_accessed` metadata on read; add archival scan query |
| `src/workspace/search.rs` | Factor decay strength into search ranking; filter archived entries from default results |
| `src/tools/builtin/memory.rs` | Set default decay metadata on `memory_write`; pass-through for `memory_read` access bump |
| `src/db/migrations/` | New migration adding metadata columns for both PostgreSQL and libSQL (if workspace entries are stored as rows; otherwise, metadata is part of the JSON metadata field on `MemoryDocument`) |

**Key architectural note**: IronClaw's workspace uses `MemoryDocument` structs with a `metadata: serde_json::Value` field. Decay state can be stored in this metadata JSON without a database schema migration:
```json
{
  "decay": {
    "variant": "ebbinghaus",
    "stability_seconds": 3600.0,
    "last_accessed": "2026-07-01T12:00:00Z",
    "access_count": 0
  }
}
```

This avoids the dual-backend migration complexity entirely. The `metadata` field already exists and is persisted.

**Step-by-step implementation**:

1. **Define the `DecayVariant` enum** in `src/workspace/document.rs`:
   ```rust
   /// Decay variant for workspace memory entries.
   ///
   /// Determines how a memory's retrieval strength changes over time.
   /// All variants compute `current_strength()` in [0.0, 1.0].
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(tag = "variant", rename_all = "snake_case")]
   pub enum DecayVariant {
       /// Never decays. For core identity files, critical facts.
       None,
       /// Exponential decay with configurable half-life.
       HalfLife { half_life_seconds: i64 },
       /// Hard expiration. For temporary context.
       Ttl { expires_at_epoch: i64 },
       /// Ebbinghaus forgetting curve. Stability doubles on each access.
       Ebbinghaus { stability_seconds: f64 },
   }

   impl DecayVariant {
       /// Compute current retrieval strength in [0.0, 1.0].
       pub fn current_strength(
           &self,
           created_epoch: i64,
           last_accessed_epoch: i64,
           now_epoch: i64,
       ) -> f64 {
           match self {
               DecayVariant::None => 1.0,
               DecayVariant::HalfLife { half_life_seconds } => {
                   let elapsed = (now_epoch - created_epoch) as f64;
                   0.5_f64.powf(elapsed / *half_life_seconds as f64)
               }
               DecayVariant::Ttl { expires_at_epoch } => {
                   if now_epoch < *expires_at_epoch { 1.0 } else { 0.0 }
               }
               DecayVariant::Ebbinghaus { stability_seconds } => {
                   let elapsed = (now_epoch - last_accessed_epoch) as f64;
                   (-elapsed / stability_seconds).exp()
               }
           }
       }
   }
   ```

2. **Store decay state in metadata**: When `memory_write` is called, set default decay metadata in the `MemoryDocument::metadata` JSON:
   ```rust
   // In src/tools/builtin/memory.rs, before workspace.write():
   let decay_meta = serde_json::json!({
       "decay": {
           "variant": "ebbinghaus",
           "stability_seconds": 3600.0,
           "last_accessed": Utc::now().to_rfc3339(),
           "access_count": 0
       }
   });
   ```
   Identity files (MEMORY.md, SOUL.md, USER.md, etc.) use `DecayVariant::None` -- they never decay.

3. **Update `memory_read`** to bump decay metadata on each access:
   ```rust
   // In workspace.read() or the memory_read tool handler:
   if let Some(decay) = metadata.get_mut("decay") {
       let count = decay.get("access_count").and_then(|v| v.as_u64()).unwrap_or(0);
       decay["access_count"] = serde_json::json!(count + 1);
       decay["last_accessed"] = serde_json::json!(Utc::now().to_rfc3339());
       // Ebbinghaus strengthening: double stability on each access
       if let Some(stability) = decay.get("stability_seconds").and_then(|v| v.as_f64()) {
           decay["stability_seconds"] = serde_json::json!(stability * 2.0);
       }
   }
   ```

4. **Update `memory_search`** to factor strength into ranking:
   - Compute `current_strength()` for each result
   - Filter out results with strength < 0.05 (effectively archived)
   - Multiply search relevance score by strength so decayed memories rank lower

5. **Add `archive_decayed()`** method to Workspace, called periodically (e.g., daily, or as part of heartbeat):
   - Scan all entries where `current_strength() < 0.01`
   - Set `metadata.status = "archived"` on those entries
   - Do NOT delete the database rows

6. **Write tests**:
   - Unit test for `DecayVariant::current_strength()` with known timestamps
   - Unit test for Ebbinghaus strengthening: create entry, simulate 5 accesses, verify stability has doubled 5 times (3600 -> 115200 seconds)
   - Integration test: write a memory with short stability, advance time (use mockable clock), verify it no longer appears in search results

**Testing strategy**:
- Unit tests for decay math: add to the test module in `document.rs`
- Integration test (`cargo test --features integration`) that writes memories with short stability, advances time, and verifies archival behavior
- Existing workspace tests must continue to pass (default decay is Ebbinghaus, which should not affect short-lived tests since stability starts at 1 hour)

**Rollback plan**: Remove the decay metadata handling. Since decay state is stored in the `metadata` JSON field (not in new database columns), rollback requires no migration reversal. Existing entries with decay metadata are simply ignored if the code is reverted -- the metadata field is opaque JSON.

**Success criteria**:
- [ ] All existing workspace tests pass
- [ ] New entries get Ebbinghaus decay metadata by default
- [ ] Entries accessed 5 times have stability > 32 hours (doubled 5 times from 1 hour)
- [ ] Entries never accessed have strength < 0.05 after approximately 3 hours
- [ ] Identity files (SOUL.md, USER.md, etc.) use `DecayVariant::None` and never decay

**Risks and mitigations**:

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Aggressive decay hides useful memories | Medium | Medium | Start with a conservative initial stability (1 hour). Can be tuned up. Archived entries can be recovered via explicit search with a flag. |
| Performance impact from strength computation on every search | Low | Low | `current_strength()` is a single floating-point exponentiation. Negligible cost even for 10,000 entries. |
| Metadata JSON bloat | Very Low | Low | Decay metadata is ~150 bytes per entry. Negligible. |

**Estimated effort**: 2-3 developer-days.

---

### Phase 1.4: Content-Addressed Deduplication

**What we are building**: BLAKE3 hashing of memory content before writing, with automatic merge when a duplicate is detected. Instead of creating a second entry with the same content, we update the existing entry's metadata (bump access count, merge tags if present, update timestamp).

**Why it matters**: LLM agents frequently "re-learn" the same fact. The agent might write "User prefers dark mode" three times across different sessions. Each duplicate dilutes search results and wastes the context window when memories are injected into prompts.

**Source concept**: Doc 10 (Universal Engram), "Content-Addressed Identity" section.

**IronClaw files affected**:

| File | Change |
|------|--------|
| `Cargo.toml` (workspace root) | Add `blake3` dependency (if not already present) |
| `src/workspace/mod.rs` | Add content-hash check before insert; implement merge-on-duplicate |
| `src/workspace/document.rs` | Add `content_hash` field to metadata convention |
| `src/tools/builtin/memory.rs` | Hash content before calling `workspace.write()` |

**Step-by-step implementation**:

1. **Add `blake3` to workspace `Cargo.toml`** (blake3 is a pure-Rust hash function, no C dependencies, ~2x faster than SHA-256):
   ```toml
   blake3 = "1"
   ```

2. **Compute hash before write**:
   ```rust
   let content_hash = blake3::hash(content.as_bytes());
   let hash_hex = content_hash.to_hex().to_string();
   ```

3. **Check for existing entry with same hash**: Before writing, search for an existing document with `metadata.content_hash == hash_hex` in the same user's workspace:
   ```rust
   // In workspace.write(), before inserting:
   if let Some(existing) = self.find_by_content_hash(user_id, &hash_hex).await? {
       // Merge metadata: bump access count, update timestamp
       // Return the existing entry with a note: "Merged with existing memory"
       return Ok(existing);
   }
   ```

4. **Store the hash in metadata** on new writes:
   ```json
   {
     "content_hash": "a3f8...b4c2",
     "decay": { ... }
   }
   ```

5. **If duplicate found, merge instead of insert**:
   - Bump `metadata.decay.access_count` (treats dedup as an implicit "access")
   - Update `metadata.decay.last_accessed`
   - Return the existing entry's path to the caller with a note: "Merged with existing memory: {path}"

6. **Write tests** (extend existing workspace tests):
   - Write same content twice; verify only one entry exists
   - Write slightly different content (one character change); verify two separate entries exist
   - Verify BLAKE3 hash is deterministic for identical inputs

**Testing strategy**:
- Unit tests on dedup logic (extend existing workspace write tests)
- Integration test: two `memory_write` calls with identical content, verify `memory_search` returns one result (not two)
- Verify existing tests still pass (no behavioral change for unique content)

**Rollback plan**: Remove the content-hash check from `workspace.write()`. The `content_hash` metadata remains on existing entries harmlessly. No database migration to reverse.

**Success criteria**:
- [ ] Duplicate content is detected and merged
- [ ] Access count on the existing entry increases when a duplicate is detected
- [ ] The agent sees a "merged with existing" message instead of silent duplication
- [ ] BLAKE3 hash computation adds < 1ms overhead per write

**Risks and mitigations**:

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Hash collision (two different contents with same hash) | Astronomically low | High | BLAKE3 256-bit has collision resistance of 2^128. Not a practical concern. |
| Overly aggressive dedup merges entries that should be separate | Low | Medium | Dedup is scoped to same `user_id` in the same workspace. Content must be _exactly_ identical, not merely similar. |

**Estimated effort**: 1-2 developer-days.

---

## Phase 2: Core Enhancements (2-4 Weeks Each)

These items introduce new subsystems with more moving parts. Each delivers significant value but requires careful design, more extensive testing, and potentially new crates.

**Phasing rationale**: Phase 2 items either build on Phase 1 foundations (2.4 requires 1.3's decay) or are independent but larger in scope. They introduce the first new crates and the first cross-module integrations. Each can still be developed independently of the others.

---

### Phase 2.1: Online Learning / Cascade Router

**What we are building**: A contextual bandit (LinUCB) that learns which LLM model to use for each type of request, wrapped in a 3-stage cascade: (1) static rules for obvious patterns, (2) confidence check, (3) bandit selection for everything else. This replaces the static scoring in `SmartRoutingProvider` with a system that improves over time.

**Why it matters**: IronClaw already has `SmartRoutingProvider` in `crates/ironclaw_llm/src/smart_routing.rs` with a 13-dimension complexity scorer that maps to four tiers (Flash/Standard/Pro/Frontier). But this scoring is _static_ -- the thresholds are hand-tuned and never change. If a new model is added, or if a model's capabilities change after an update, the static scorer does not adapt. The LinUCB bandit observes the actual reward (task completion, latency, cost) and learns which model works best for which context. Based on roko's experience, this delivers 30-50% cost reduction: 40-60% of requests can be handled by cheaper models without quality loss.

**Source concept**: Doc 07 (Online Learning). See also `19-priority-matrix.md` for ROI ranking (highest-ROI medium-effort feature).

**Architecture decision**: Implement as new modules within `crates/ironclaw_llm/` rather than a separate crate, because it needs tight integration with the provider chain and `SmartRoutingProvider`'s existing `Tier` enum and scoring.

**IronClaw files affected**:

| File | Change |
|------|--------|
| `crates/ironclaw_llm/src/bandit.rs` (NEW) | LinUCB implementation: `LinUCBArm`, `LinUCBBandit`, matrix operations |
| `crates/ironclaw_llm/src/cascade_router.rs` (NEW) | `CascadeRouter`: static rules + confidence check + bandit selection |
| `crates/ironclaw_llm/src/routing_features.rs` (NEW) | Feature vector encoding from `CompletionRequest` context |
| `crates/ironclaw_llm/src/routing_episode.rs` (NEW) | `RoutingEpisode` struct, reward computation |
| `crates/ironclaw_llm/src/smart_routing.rs` | Refactor to optionally delegate to `CascadeRouter` when enabled (feature-gated) |
| `crates/ironclaw_llm/src/lib.rs` | Add modules, wire into provider chain construction |
| `crates/ironclaw_llm/Cargo.toml` | Add feature flag `cascade-router`, optional `nalgebra` dependency for matrix operations |
| `src/config/llm.rs` | Add `LLM_CASCADE_ROUTER_ENABLED`, `LLM_CASCADE_ALPHA` env vars |
| Database migration | `model_routing_episodes` table for episode logging (both backends) |

**Step-by-step implementation**:

1. **Implement LinUCB** (`bandit.rs`, ~300-400 lines):
   - Each "arm" is an available model (e.g., claude-haiku, claude-sonnet, gpt-4o-mini)
   - Each arm maintains matrices `A` (d x d) and `b` (d x 1) where `d` is the feature dimension
   - Selection: for each arm, compute `p = theta^T x + alpha * sqrt(x^T A^{-1} x)` and pick the arm with highest `p`
   - Update: on observing reward `r`, update `A += x x^T` and `b += r x`
   - Warm-start: initialize with prior from `SmartRoutingProvider`'s static tier mapping
   - Start with 8 feature dimensions (not 18); add features incrementally as validated

   ```rust
   pub struct LinUCBArm {
       /// d x d matrix (regularized identity + outer products)
       a: nalgebra::DMatrix<f64>,
       /// d x 1 vector (reward-weighted context sums)
       b: nalgebra::DVector<f64>,
       /// Model name this arm represents
       pub model: String,
   }

   pub struct LinUCBBandit {
       arms: Vec<LinUCBArm>,
       alpha: f64,  // Exploration parameter
       dim: usize,  // Feature dimension
   }

   impl LinUCBBandit {
       pub fn select(&self, context: &[f64]) -> usize { /* ... */ }
       pub fn update(&mut self, arm_idx: usize, context: &[f64], reward: f64) { /* ... */ }
   }
   ```

2. **Implement feature encoding** (`routing_features.rs`, ~200 lines):
   - Start with 8 features from `CompletionRequest` + context:
     - `estimated_complexity` (reuse the 13-dimension scorer from `SmartRoutingProvider`, normalized to [0,1])
     - `message_count` (conversation length, log-scaled)
     - `tool_count` (number of tools in the request, normalized)
     - `has_images` (binary: whether attachments are present)
     - `avg_message_length` (mean token estimate across messages)
     - `time_of_day_sin`, `time_of_day_cos` (cyclical encoding)
     - `recent_error_rate` (from circuit breaker state, if available)
   - Normalize all features to [0, 1] range

3. **Implement cascade** (`cascade_router.rs`, ~400-500 lines):
   - Stage 1: Static rules (regex patterns). Example: greeting detected -> Flash tier. Security audit keywords -> Frontier tier. These match `SmartRoutingProvider`'s existing pattern overrides.
   - Stage 2: Check static rule confidence. If > 0.95, use directly.
   - Stage 3: LinUCB selection from available (non-circuit-broken) models.
   - The `CascadeRouter` implements `LlmProvider` as a decorator, wrapping the selected model's provider.

4. **Implement reward computation** (`routing_episode.rs`, ~200 lines):
   - After each LLM call, compute reward:
     ```
     reward = w1 * task_success + w2 * (1 - normalized_cost) + w3 * (1 - normalized_latency)
     ```
   - `task_success`: did the agent produce a tool call? Did the tool succeed? (Binary, from delegate feedback)
   - Log the episode to database for offline analysis

5. **Wire into the provider chain**: Insert `CascadeRouter` as an alternative to `SmartRoutingProvider` in the provider chain construction, gated by `#[cfg(feature = "cascade-router")]` and `LLM_CASCADE_ROUTER_ENABLED=true`.

6. **Add episode logging**: Create a `model_routing_episodes` table (PostgreSQL and libSQL) with: `id, timestamp, context_vector (JSON), selected_model, reward, latency_ms, token_count, cost_cents`.

**Testing strategy**:
- Unit tests on `LinUCBBandit`: after 100 training examples where arm A always gives reward 1.0 and arm B gives 0.5, verify arm A is selected with high probability
- Unit tests on feature encoding: known inputs produce expected feature vectors
- Integration test: wire `CascadeRouter` over `StubLlm` (from `crates/ironclaw_llm/src/testing/`), simulate 50 requests, verify that the bandit converges toward the higher-reward model
- Compare-mode test: run same workload through `SmartRoutingProvider` and `CascadeRouter`, log both routing decisions, verify CascadeRouter matches or improves

**Rollback plan**: Disable via feature flag `cascade-router` (compile-time) or `LLM_CASCADE_ROUTER_ENABLED=false` (runtime). The `SmartRoutingProvider` remains as the fallback. No change to existing behavior when disabled. The episode logging table remains but is inert.

**Success criteria**:
- [ ] After 200+ requests, the bandit selects cheaper models for simple tasks with >80% accuracy
- [ ] No degradation in quality for complex tasks (Frontier tier)
- [ ] Cost per request decreases by at least 20% on a representative workload
- [ ] Episode logging captures all routing decisions for offline analysis
- [ ] `SmartRoutingProvider` continues to work unchanged when cascade-router is disabled

**Risks and mitigations**:

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Bandit selects wrong model for critical task | Medium (early) | High | During cold-start, alpha is set high (exploration mode). Static rules override the bandit for safety-critical patterns. Frontier tier is never bypassed for security-related prompts. |
| Matrix inversion instability (LinUCB) | Low | Medium | Use regularized inverse: `A = I + x x^T` (identity initialization). Use `nalgebra`'s numerically stable inversion. |
| Reward signal is noisy/delayed | Medium | Medium | Use composite reward with multiple signals. Apply exponential discounting for delayed rewards. |
| Feature encoding is fragile | Medium | Low | Start with 8 dimensions instead of 18; add features incrementally as we validate each one adds signal. |

**Estimated effort**: 8-12 developer-days.

---

### Phase 2.2: Hyperdimensional Computing (HDC)

**What we are building**: A standalone crate (`crates/ironclaw_hdc/`) implementing 10,240-bit binary vectors with bind (XOR), bundle (majority vote), permute (rotation), and Hamming distance operations. These vectors provide an extremely fast (nanosecond-scale) similarity measure that can augment or replace embedding-based similarity for certain use cases.

**Why it matters**: IronClaw's `memory_search` currently uses two search modes: FTS5 (keyword match) and vector embeddings (semantic similarity), merged via RRF. Adding HDC fingerprints as a third signal improves recall for compositional queries (e.g., "find memories about both Rust AND async") because HDC vectors are _compositional_ -- you can bind two concept vectors together and search for the combination. HDC is also ~1000x faster than embedding similarity (~13ns per comparison vs ~2.5us for cosine distance on dense vectors), making it feasible to scan 100K entries in under 1ms.

**Source concept**: Doc 01 (Hyperdimensional Computing). See also Doc 25 (Research Citations) for the Kanerva (2009) reference.

**IronClaw files affected**:

| File | Change |
|------|--------|
| `crates/ironclaw_hdc/` (NEW crate) | `HdcVector`, `Codebook`, `HdcEncoder`, `HdcIndex` |
| `crates/ironclaw_hdc/src/lib.rs` | Public API: `HdcVector`, `Codebook`, `encode_text()`, `similarity()` |
| `crates/ironclaw_hdc/src/vector.rs` | `HdcVector` struct (`[u64; 160]`), bind/bundle/permute/hamming operations |
| `crates/ironclaw_hdc/src/codebook.rs` | `Codebook` mapping tokens to random base vectors |
| `crates/ironclaw_hdc/src/encoder.rs` | Text-to-HDC encoding: tokenize, look up in codebook, bind with positional permutation, bundle |
| `crates/ironclaw_hdc/src/index.rs` | Brute-force scan with SIMD-accelerated Hamming distance |
| `src/workspace/search.rs` | Add HDC as third signal in RRF merge |
| `src/workspace/mod.rs` | Compute and store HDC fingerprint on write |

**Step-by-step implementation**:

1. **Create the crate** with `cargo new --lib crates/ironclaw_hdc`.

2. **Implement `HdcVector`** (`vector.rs`, ~200 lines):
   ```rust
   #[derive(Clone, PartialEq, Eq)]
   pub struct HdcVector {
       bits: [u64; 160], // 160 * 64 = 10,240 bits
   }

   impl HdcVector {
       pub fn random(rng: &mut impl Rng) -> Self { /* fill with random bits */ }
       pub fn bind(&self, other: &Self) -> Self { /* XOR each u64 */ }
       pub fn permute(&self, n: usize) -> Self { /* rotate bits by n positions */ }
       pub fn hamming_distance(&self, other: &Self) -> u32 {
           self.bits.iter().zip(other.bits.iter())
               .map(|(a, b)| (a ^ b).count_ones())
               .sum()
       }
       pub fn similarity(&self, other: &Self) -> f32 {
           1.0 - (self.hamming_distance(other) as f32 / 10_240.0)
       }
       pub fn to_bytes(&self) -> Vec<u8> { /* serialize for storage */ }
       pub fn from_bytes(bytes: &[u8]) -> Option<Self> { /* deserialize */ }
   }
   ```
   Use `u64::count_ones()` which compiles to the `POPCNT` SIMD instruction on x86.

3. **Implement `bundle()`** (majority vote per bit across N vectors, ~50 lines). For each bit position, count how many input vectors have a 1; if majority, output 1.

4. **Implement `Codebook`** (`codebook.rs`, ~100 lines): A `HashMap<String, HdcVector>` that lazily generates random vectors for unseen tokens. Deterministic seeding via `blake3::hash(token)` so the same token always maps to the same vector.

5. **Implement `HdcEncoder`** (`encoder.rs`, ~150 lines):
   - Tokenize input text (split on whitespace + basic normalization)
   - For each token at position `i`, compute `permute(codebook[token], i)`
   - Bundle all permuted vectors into a single fingerprint

6. **Integrate into workspace search** (`src/workspace/search.rs`):
   - On `memory_write`: compute HDC fingerprint, store in metadata as base64-encoded bytes
   - On `memory_search`: compute HDC fingerprint of query, scan all entries, add as third RRF signal
   - RRF merge formula: `score(d) = sum(1/(k + rank_fts(d)) + 1/(k + rank_vec(d)) + 1/(k + rank_hdc(d)))`

**Testing strategy**:
- Unit tests in `crates/ironclaw_hdc/`: vector operations, codebook determinism, encoder output stability
- Benchmark test: scan 100,000 vectors in < 5ms
- Integration test: write 10 memories, search with HDC enabled, verify results include HDC signal in RRF merge
- A/B comparison: run the same search queries with and without HDC; verify recall improves for compositional queries

**Rollback plan**: Feature-gate with `#[cfg(feature = "hdc")]`. Disable the feature to fall back to FTS + vector only. The HDC fingerprints in metadata are inert when the feature is off.

**Success criteria**:
- [ ] `similarity("rust async tokio", "tokio async runtime")` > 0.7
- [ ] `similarity("rust async tokio", "cooking recipe pasta")` < 0.3
- [ ] 100K vector scan in < 5ms on Apple Silicon
- [ ] Memory search recall improves for compositional queries (measured by manual evaluation on 20 test queries)

**Risks and mitigations**:

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| HDC fingerprints do not add signal beyond existing FTS + vector | Medium | Low | HDC is a third signal; even if it does not help, it does not hurt (RRF handles gracefully). Can be disabled via feature flag. |
| Storage overhead (1280 bytes per entry as base64 in metadata) | Low | Low | For 100K entries, that's ~170MB. Acceptable for a personal assistant. |
| Codebook grows unbounded | Low | Low | Use deterministic seeding from token hash; no persistent codebook needed. |

**Estimated effort**: 5-8 developer-days.

---

### Phase 2.3: Gate Verification Pipeline (Rungs 1-4)

**What we are building**: A progressive verification pipeline for code that the agent generates or modifies. In Phase 2, we implement the first four rungs: (1) Compile check, (2) Lint, (3) Run existing tests, (4) Symbol resolution. Rungs 5-7 (LLM-generated tests, property tests, full integration) are deferred to Phase 3/4 as they require LLM calls and are significantly more complex.

**Why it matters**: When IronClaw generates code (via the tool builder, via shell commands, or via code generation in conversations), there is no structured verification beyond "did the shell command succeed." The gate pipeline adds structured, progressive verification that catches errors early and provides actionable diagnostic information when something fails.

**Source concept**: Doc 05 (Gate Verification Pipeline).

**Architecture decision**: New crate `crates/ironclaw_gate/` because it is a self-contained subsystem with no dependency on IronClaw internals (it shells out to compilers and linters).

**IronClaw files affected**:

| File | Change |
|------|--------|
| `crates/ironclaw_gate/` (NEW crate) | Full gate pipeline |
| `crates/ironclaw_gate/src/lib.rs` | Public API: `GatePipeline`, `Rung`, `RungResult`, `GateContext` |
| `crates/ironclaw_gate/src/rung.rs` | `Rung` trait, `RungResult`, `Diagnostic` |
| `crates/ironclaw_gate/src/pipeline.rs` | `GatePipeline`: run rungs in sequence, abort on failure |
| `crates/ironclaw_gate/src/complexity.rs` | `ComplexityAssessor`: determine which rungs to run |
| `crates/ironclaw_gate/src/rungs/compile.rs` | Rung 1: invoke `cargo check`, `tsc --noEmit`, `python -m py_compile` |
| `crates/ironclaw_gate/src/rungs/lint.rs` | Rung 2: invoke `cargo clippy`, `eslint`, `ruff` |
| `crates/ironclaw_gate/src/rungs/test.rs` | Rung 3: invoke `cargo test`, `npm test`, `pytest` |
| `crates/ironclaw_gate/src/rungs/symbol.rs` | Rung 4: parse compiler output for unresolved symbols |
| `src/tools/builder/validation.rs` | Call `GatePipeline::run()` after WASM tool build |

**Step-by-step implementation**:

1. **Define the `Rung` trait**:
   ```rust
   #[async_trait]
   pub trait Rung: Send + Sync {
       /// Human-readable rung name (e.g., "Compile", "Lint").
       fn name(&self) -> &str;
       /// Estimated cost category.
       fn cost_estimate(&self) -> RungCost;
       /// Run the verification and return the result.
       async fn run(&self, context: &GateContext) -> RungResult;
   }

   pub enum RungCost { Cheap, Medium, Expensive }

   pub struct RungResult {
       pub passed: bool,
       pub diagnostics: Vec<Diagnostic>,
       pub duration: Duration,
   }

   pub struct Diagnostic {
       pub file: Option<PathBuf>,
       pub line: Option<u32>,
       pub column: Option<u32>,
       pub severity: Severity,
       pub message: String,
       pub code: Option<String>,
   }

   pub enum Severity { Error, Warning, Info }
   ```

2. **Implement `ComplexityAssessor`** (~150 lines):
   - Input: list of changed files, lines added/removed
   - Output: `Complexity` enum (Trivial, Simple, Standard, Complex)
   - Trivial (rungs 1-2): < 5 lines changed, no public API changes
   - Simple (rungs 1-4): < 50 lines, single file
   - Standard (rungs 1-4+): 50+ lines or multiple files
   - Complex: public API changes, security-sensitive paths

3. **Implement Rungs 1-4** (each ~80-120 lines):
   - Detect language from file extension (Rust, TypeScript, Python)
   - Run the appropriate tool via `tokio::process::Command` (never through a shell -- use `Command::new()` with args)
   - Parse stdout/stderr for errors, collect into `Diagnostic` structs

4. **Implement `GatePipeline`** (~200 lines):
   - Takes a list of rungs and a `GateContext` (working directory, changed files, language)
   - Runs rungs sequentially until one fails or all pass
   - Returns `GateResult` with per-rung outcomes

5. **Wire into tool builder**: After `SoftwareBuilder` builds a WASM tool, run the gate pipeline before declaring success.

**Testing strategy**:
- Unit tests for `ComplexityAssessor` with known inputs
- Integration tests: create a temp directory with a Rust project containing a compile error, run the gate pipeline, verify Rung 1 catches it
- Integration tests: create a project with a failing test, verify Rung 3 catches it

**Rollback plan**: Feature-gate with `#[cfg(feature = "gate")]`. The tool builder falls back to its current basic validation. The crate exists but is inert.

**Success criteria**:
- [ ] Gate pipeline correctly identifies compile errors, lint warnings, test failures
- [ ] Pipeline runs rungs 1-4 in < 30 seconds for a typical Rust project
- [ ] The agent receives actionable diagnostics (file, line, error message) when a rung fails
- [ ] Complexity assessor correctly categorizes trivial, simple, standard, and complex changes

**Risks and mitigations**:

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Language detection is wrong | Medium | Low | Default to "skip rung" for unrecognized languages. Only Rust is fully supported in Phase 2. |
| Pipeline is too slow for interactive use | Medium | Medium | Rungs 1-2 are fast (< 5s). Rung 3 can be slow for large test suites -- add a timeout (default 60s). |
| Shell command injection in file paths | Low | High | All shell commands use `Command::new()` with args (not string interpolation). Never pass file paths through a shell. |

**Estimated effort**: 6-10 developer-days.

---

### Phase 2.4: Enhanced Heartbeat (Dream Consolidation Lite)

**What we are building**: Extending IronClaw's existing heartbeat system with two dream consolidation subsystems: (a) NREM replay -- reviewing and strengthening recent memories, and (b) threat rehearsal -- analyzing recent failures and preparing defenses. This is the "lite" version; full dream consolidation (REM imagination, hypnagogic creativity) is deferred to Phase 3.4.

**Why it matters**: IronClaw's heartbeat currently reads a static `HEARTBEAT.md` checklist and runs an agent turn. That is useful but passive. Enhanced heartbeat actively reviews recent activity, strengthens important memories, identifies patterns, and learns from failures -- all without user interaction.

**Prerequisite**: Phase 1.3 (Ebbinghaus Decay) -- NREM replay needs the ability to strengthen memories by updating their stability metadata.

**Source concept**: Doc 02 (Dream Consolidation), "NREM Replay" and "Threat Rehearsal" sections.

**IronClaw files affected**:

| File | Change |
|------|--------|
| `src/agent/consolidation.rs` (NEW) | `ConsolidationEngine` with NREM replay and threat rehearsal |
| `src/agent/consolidation_replay.rs` (NEW) | NREM replay: query recent memories, compute Mattar-Daw utility, strengthen high-utility entries |
| `src/agent/consolidation_rehearsal.rs` (NEW) | Threat rehearsal: query recent failed jobs, generate defensive strategies via LLM |
| `src/agent/heartbeat.rs` | Add consolidation cycle after HEARTBEAT.md processing |
| `src/agent/mod.rs` | Add modules |
| `src/config/mod.rs` | Add `CONSOLIDATION_ENABLED`, `CONSOLIDATION_INTERVAL_HOURS` env vars |

**Step-by-step implementation**:

1. **Implement NREM replay** (`consolidation_replay.rs`, ~300 lines):
   - Query workspace for memories updated in the last 24 hours
   - For each memory, compute Mattar-Daw utility: `utility = need * gain * probability`
     - `need`: how recently was this accessed? (inverse of time since last access)
     - `gain`: how much would strengthening help? (inverse of current strength)
     - `probability`: how frequently has the user asked about related topics? (access_count proxy)
   - Sort by utility, process top N (budget-constrained, default: 20 memories per cycle)
   - For each processed memory: double `stability_seconds` in the metadata (Ebbinghaus strengthening without a user access)
   - Detect patterns across replayed memories: if multiple memories share path prefixes or keywords, consider writing a "pattern" memory that synthesizes them

2. **Implement threat rehearsal** (`consolidation_rehearsal.rs`, ~300 lines):
   - Query database for failed jobs (via `ContextManager`) in the last 48 hours
   - For each failure:
     - Extract the error message and the tool call that caused it
     - Use the LLM (via standard tool dispatch) to generate a brief analysis: "This failed because X. Next time, try Y."
     - Write the analysis as a workspace memory at `daily/consolidation/YYYY-MM-DD.md`
   - Budget: max 3 LLM calls per consolidation cycle (to control cost)

3. **Wire into heartbeat** (`heartbeat.rs`):
   - After the existing HEARTBEAT.md processing, check if consolidation is enabled and enough time has passed since the last cycle
   - Run NREM replay (cheap -- no LLM calls, just workspace reads and writes)
   - Run threat rehearsal (uses LLM -- 3 calls max)
   - Default schedule: every 2 hours (configurable via `CONSOLIDATION_INTERVAL_HOURS`)

4. **Add `ConsolidationEngine`** (~100 lines):
   ```rust
   pub struct ConsolidationEngine {
       replay_enabled: bool,
       rehearsal_enabled: bool,
       last_run: Option<DateTime<Utc>>,
       interval: Duration,
       max_replay_entries: usize,
       max_rehearsal_calls: usize,
   }
   ```

**Testing strategy**:
- Unit tests: NREM replay utility scoring with known inputs
- Integration test: write 10 memories with short stability, run replay, verify stability increased for high-utility entries
- Integration test: create a failed job record, run threat rehearsal with `StubLlm`, verify a defensive memory was written to the workspace
- Verify heartbeat timing: consolidation only runs after the configured interval

**Rollback plan**: Disable via `CONSOLIDATION_ENABLED=false`. The heartbeat falls back to HEARTBEAT.md-only mode. Remove `mod consolidation*;` from `src/agent/mod.rs` for a clean revert.

**Success criteria**:
- [ ] After a consolidation cycle, high-utility memories have increased stability
- [ ] After a failure, threat rehearsal produces an actionable defensive memory
- [ ] Consolidation runs within budget (< $0.05 per cycle with cheap model)
- [ ] No impact on interactive agent responsiveness (consolidation runs in heartbeat context)

**Risks and mitigations**:

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Consolidation uses too many tokens | Medium | Medium | Hard budget cap: max 3 LLM calls per cycle, each with max_tokens=200. NREM replay uses zero LLM calls. |
| Threat rehearsal generates bad advice | Low | Medium | Defensive memories are written to `daily/consolidation/` path so the agent knows they are machine-generated and treats them with appropriate skepticism. |
| Consolidation runs during active conversation | Low | Low | Consolidation runs in the heartbeat context (background). It uses the standard LLM dispatch, not interfering with active sessions. |

**Estimated effort**: 4-6 developer-days.

---

## Phase 3: Architecture Evolution (4-8 Weeks Each)

These are larger initiatives that introduce significant new capabilities. They build on Phase 1-2 foundations and require more architectural consideration.

**Phasing rationale**: Phase 3 items depend on Phase 2 outputs: the conductor needs the cascade router's model selection data (3.2 depends on 2.1), cognitive speeds need the cascade router's model tier mapping (3.3 depends on 2.1), full dreams need the heartbeat enhancements and HDC for cross-domain resonance (3.4 depends on 2.4 and 2.2), and the DAG engine benefits from having gate pipeline cells available (3.1 benefits from 2.3).

---

### Phase 3.1: DAG Execution Engine

**What we are building**: A declarative workflow engine where workflows are defined in TOML as directed acyclic graphs. Nodes ("Cells") represent units of work (tool calls, LLM calls, shell commands), and edges represent data flow with conditional routing. The engine executes the DAG with budget tracking, parallel execution of independent nodes, and graceful handling of partial failures.

**Why it matters**: IronClaw currently handles multi-step tasks through sequential agent turns -- the agent does step 1, then step 2, then step 3. This is linear and wasteful: independent steps could run in parallel, and the ordering is implicit in the LLM's reasoning rather than declared. DAG execution makes workflows explicit, parallelizable, and repeatable. Users can define custom workflows (e.g., "morning standup" that checks email + GitHub + calendar in parallel, then summarizes) in TOML files.

**Prerequisite**: None hard, but benefits from 2.3 (gate pipeline cells for verification steps in workflows).

**Source concept**: Doc 04 (DAG Execution Engine).

**Architecture decision**: New crate `crates/ironclaw_graph/` because it is a self-contained execution engine. The key design constraint is that Cells dispatch through IronClaw's `ToolDispatcher` (preserving audit trail and safety), not bypassing it.

**IronClaw files affected**:

| File | Change |
|------|--------|
| `crates/ironclaw_graph/` (NEW crate) | Full DAG execution engine |
| `crates/ironclaw_graph/src/cell.rs` | `Cell` trait, `CellInput`, `CellOutput`, `CellContext` |
| `crates/ironclaw_graph/src/registry.rs` | `CellRegistry` with factory pattern |
| `crates/ironclaw_graph/src/graph.rs` | `Graph` definition, `Edge`, `Condition` (Always/OnSuccess/OnFailure/When) |
| `crates/ironclaw_graph/src/executor.rs` | DAG executor: topological sort, parallel execution via `tokio::JoinSet` |
| `crates/ironclaw_graph/src/budget.rs` | `GraphBudget`: token, cost, and time tracking |
| `crates/ironclaw_graph/src/loader.rs` | TOML graph loading and validation |
| `src/tools/builtin/` | New `workflow` tool that executes a named DAG |
| `src/agent/routine_engine.rs` | Optionally allow routines to reference DAG workflows |

**Step-by-step implementation**:

1. **Define the `Cell` trait** and built-in cells:
   ```rust
   #[async_trait]
   pub trait Cell: Send + Sync {
       fn name(&self) -> &str;
       async fn execute(&self, input: CellInput, ctx: &CellContext) -> Result<CellOutput, CellError>;
   }
   ```
   - `ToolCell`: dispatches through `ToolDispatcher`
   - `LlmCell`: makes an LLM call with a prompt template
   - `ShellCell`: executes a shell command (sandboxed)
   - `ConditionCell`: evaluates a predicate and routes
   - `AggregateCell`: collects outputs from multiple parents

2. **Implement the TOML loader**: parse a TOML workflow definition into a `Graph` struct. Validate: no cycles, all cell references resolve, budget is specified.

3. **Implement the executor**: topological sort to determine execution order. Group nodes with no unmet dependencies into "waves" and execute each wave in parallel. Track budget consumption per cell.

4. **Create the `workflow` tool**: a built-in tool that takes a workflow name, loads the TOML, and executes the DAG. Results are returned as structured JSON.

5. **Add user-facing workflow directory**: `~/.ironclaw/workflows/` for user-defined TOML workflows.

**Testing strategy**:
- Unit tests: TOML parsing, topological sort, cycle detection
- Unit tests: budget enforcement (verify pipeline aborts when budget exhausted)
- Integration test: define a simple workflow (two parallel tasks -> one aggregation), execute it, verify results
- Integration test: define a workflow with a failing node and OnFailure edge, verify the fallback path is taken

**Rollback plan**: Remove the `workflow` tool registration. The crate exists but is inert. No database changes.

**Success criteria**:
- [ ] A 5-node workflow with 2 parallel steps executes correctly
- [ ] Budget exhaustion mid-workflow returns partial results gracefully
- [ ] Conditional edges route correctly based on cell outcomes
- [ ] User can define and execute a custom workflow in TOML

**Risks and mitigations**:

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Cycle in user-defined graph | Medium | Low | Validate at load time with cycle detection (topological sort fails on cycles) |
| Deadlock in parallel execution | Low | High | Use `tokio::JoinSet` with timeouts per cell. No cross-cell synchronization beyond data dependencies. |
| Security: arbitrary shell execution via workflow TOML | Medium | High | ShellCell must go through the sandbox. User-defined workflows run with the user's permission level, not elevated. |

**Estimated effort**: 12-18 developer-days.

---

### Phase 3.2: Conductor Anomaly Detection

**What we are building**: An ensemble of health watchers for LLM providers that monitor latency, error rate, cost, and quality in real-time. When anomalies are detected, the conductor can predict failures (using Holt exponential smoothing) and pre-emptively route away from degrading providers before they fail. This augments the existing `CircuitBreakerProvider` (which reacts after failures) with prediction (which acts before failures).

**Why it matters**: IronClaw's existing circuit breaker (`crates/ironclaw_llm/src/circuit_breaker.rs`) is reactive: it trips after `failure_threshold` consecutive failures (default 5). By that point, 5 requests have already failed. The conductor uses Holt's double exponential smoothing to forecast the next observation and trips pre-emptively when the forecast breaches a threshold. It also detects compound patterns (latency spike + error rate increase = likely provider degradation) that single-signal circuit breakers miss.

**Prerequisite**: Soft dependency on 2.1 (Cascade Router). The conductor's health data informs routing decisions. Without the cascade router, the conductor still provides health monitoring and alerting, but routing action requires manual intervention.

**Source concept**: Doc 06 (Conductor Anomaly Detection).

**IronClaw files affected**:

| File | Change |
|------|--------|
| `crates/ironclaw_llm/src/conductor.rs` (NEW) | `Conductor` struct, watcher ensemble, compound pattern detection |
| `crates/ironclaw_llm/src/holt.rs` (NEW) | Holt double exponential smoothing implementation |
| `crates/ironclaw_llm/src/watcher.rs` (NEW) | `Watcher` trait, `LatencyWatcher`, `ErrorRateWatcher`, `CostWatcher`, `QualityWatcher` |
| `crates/ironclaw_llm/src/lib.rs` | Wire conductor into provider chain |
| `crates/ironclaw_llm/src/circuit_breaker.rs` | Extend with `Predicted` state (pre-emptive open based on forecast) |

**Step-by-step implementation**:

1. **Implement Holt smoothing** (`holt.rs`, ~100 lines):
   ```rust
   pub struct HoltForecast {
       level: f64,
       trend: f64,
       alpha: f64, // level smoothing (default 0.3)
       beta: f64,  // trend smoothing (default 0.1)
   }
   impl HoltForecast {
       pub fn observe(&mut self, value: f64) { /* update level and trend */ }
       pub fn forecast(&self, steps_ahead: u32) -> f64 {
           self.level + self.trend * steps_ahead as f64
       }
   }
   ```

2. **Implement watchers** (start with 4 of the 10):
   - `LatencyWatcher`: tracks P50/P95/P99 latency, uses Holt to forecast
   - `ErrorRateWatcher`: sliding window error rate, threshold detection
   - `CostWatcher`: per-request cost tracking, budget projection
   - `QualityWatcher`: output quality scores (from evaluation module), drift detection

3. **Implement compound pattern detection** (~200 lines):
   - Define patterns: `latency_spike AND error_increase -> provider_degradation`
   - Pattern matching: check all active watchers for correlated anomalies within a time window

4. **Wire into provider chain**: The conductor wraps the provider chain and observes every call. When it forecasts a threshold breach, it adds a `Predicted` state to the circuit breaker.

**Testing strategy**:
- Unit tests: Holt smoothing convergence on known time series
- Unit tests: compound pattern detection with simulated watcher data
- Integration test: simulate a gradually degrading provider (increasing latency) and verify the conductor predicts the breach before it happens

**Rollback plan**: Feature-gate with `#[cfg(feature = "conductor")]`. Falls back to existing circuit breaker behavior.

**Success criteria**:
- [ ] Conductor detects a steadily increasing latency trend and raises an alert before the actual threshold is breached
- [ ] Compound pattern detection correctly identifies correlated latency + error rate anomalies
- [ ] No false positives on stable providers over a 24-hour monitoring period

**Estimated effort**: 10-14 developer-days.

---

### Phase 3.3: Cognitive Speed Classification

**What we are building**: Formalizing IronClaw's processing into three explicit cognitive speeds -- Gamma (reactive, fast, cheap model), Theta (reflective, planning, standard model), and Delta (consolidation, background, best model) -- with automatic classification of incoming requests into the appropriate speed.

**Why it matters**: Currently, model selection is based on static complexity scoring (SmartRoutingProvider) or the cascade router (Phase 2.1). But the model choice should also depend on the _type_ of processing needed, not just the _complexity_ of the request. A simple question that requires accessing memory (Gamma) should use a different model than a request that requires multi-step planning (Theta), even if both score similarly on complexity. Cognitive speed classification formalizes this distinction and maps it to model tiers.

**Prerequisite**: 2.1 (Cascade Router) -- cognitive speeds map to model tiers through the router.

**Source concept**: Doc 13 (Cognitive Architecture), "Three Cognitive Speeds" section.

**IronClaw files affected**:

| File | Change |
|------|--------|
| `src/agent/cognitive_speed.rs` (NEW) | `CognitiveSpeed` enum, `SpeedClassifier` |
| `src/agent/mod.rs` | Add module |
| `src/agent/dispatcher.rs` | Classify speed before LLM call; pass to routing context |
| `crates/ironclaw_llm/src/routing_features.rs` | Add `cognitive_speed` as a feature dimension |
| `src/agent/heartbeat.rs` | Explicitly set Delta speed for consolidation tasks |

**Step-by-step implementation**:

1. **Define the enum and classifier** (~200-300 lines):
   ```rust
   /// Cognitive processing speed, mapped from neuroscience terminology.
   /// See 13-cognitive-architecture.md for theoretical foundations.
   pub enum CognitiveSpeed {
       /// Reactive: < 15s target, cheapest model. Direct responses, tool calls, simple queries.
       Gamma,
       /// Reflective: < 75s target, standard model. Planning, multi-step reasoning, error recovery.
       Theta,
       /// Consolidation: no time pressure, best model. Background tasks, heartbeat, dream consolidation.
       Delta,
   }

   pub struct SpeedClassifier;
   impl SpeedClassifier {
       pub fn classify(context: &TurnContext) -> CognitiveSpeed {
           // Delta: background tasks, heartbeat, consolidation
           if context.is_background() { return CognitiveSpeed::Delta; }
           // Theta: planning, multi-step, or recovery from prior failure
           if context.requires_planning() || context.has_prior_failure() {
               return CognitiveSpeed::Theta;
           }
           // Gamma: everything else
           CognitiveSpeed::Gamma
       }
   }
   ```

2. **Wire into dispatcher**: Before the LLM call, classify the cognitive speed and include it in the routing context for the cascade router.

3. **Map speeds to model tiers**: Gamma -> Flash/Standard tier, Theta -> Pro tier, Delta -> Frontier tier.

**Testing strategy**:
- Unit tests: known inputs map to expected speeds
- Integration test: verify that a heartbeat turn uses Delta speed, a simple chat uses Gamma, and a multi-step task uses Theta

**Rollback plan**: Remove the classifier; the cascade router continues to select models based on complexity alone.

**Success criteria**:
- [ ] Background tasks always use Delta (best model for quality)
- [ ] Simple queries always use Gamma (cheapest model for speed)
- [ ] Planning tasks always use Theta (balanced model)

**Estimated effort**: 3-5 developer-days.

---

### Phase 3.4: Full Dream Consolidation

**What we are building**: Completing the dream consolidation system by adding the two remaining subsystems deferred from Phase 2.4: (a) REM Imagination -- generating counterfactual scenarios for recent experiences ("what would have happened if..."), and (b) Hypnagogic Creativity -- finding unexpected connections between knowledge from different domains.

**Why it matters**: NREM replay (Phase 2.4) strengthens existing memories. REM imagination _extends_ the agent's understanding by exploring alternative outcomes. Hypnagogic creativity _discovers_ connections the agent would not find during normal operation. Together, they make the agent genuinely learn from experience, not just memorize.

**Prerequisite**: 2.4 (Enhanced Heartbeat) for the consolidation framework and scheduling, and 2.2 (HDC) for cross-domain resonance detection in the hypnagogic pipeline.

**Source concept**: Doc 02 (Dream Consolidation), "REM Imagination" and "Hypnagogic Creativity" sections.

**Architecture decision**: New crate `crates/ironclaw_dreams/` because this is a substantial subsystem with its own scheduling, budget management, and LLM integration.

**IronClaw files affected**:

| File | Change |
|------|--------|
| `crates/ironclaw_dreams/` (NEW crate) | Full dream consolidation engine |
| `crates/ironclaw_dreams/src/imagination.rs` | REM imagination: counterfactual generation via LLM |
| `crates/ironclaw_dreams/src/creativity.rs` | Hypnagogic pipeline: find cross-domain connections via HDC similarity + LLM evaluation |
| `crates/ironclaw_dreams/src/staging.rs` | Confidence staging buffer (Raw -> Replayed -> Validated -> Promoted) |
| `crates/ironclaw_dreams/prompts/` | Prompt templates for counterfactual generation and insight evaluation (loaded via `include_str!()`) |
| `src/agent/consolidation.rs` | Wire the new subsystems into the consolidation engine |
| `src/agent/heartbeat.rs` | Schedule REM imagination (every 6 hours) and hypnagogic creativity (every 24 hours) |

**Step-by-step implementation**:

1. **Implement REM Imagination** (~400 lines):
   - Select recent experiences (last 48 hours) that had negative outcomes
   - For each, generate a counterfactual prompt (loaded from `prompts/counterfactual.md`): "Given this scenario [X], what would have happened if [Y]?"
   - Use LLM (via tool dispatch) to generate 2-3 alternative outcomes
   - Store the alternatives as workspace memories at `daily/consolidation/counterfactuals/YYYY-MM-DD.md`
   - Budget: max 5 LLM calls per cycle

2. **Implement Hypnagogic Creativity** (~500 lines):
   - Use HDC fingerprints (Phase 2.2) to find pairs of memories from different paths/domains with surprising similarity (HDC similarity > 0.3 despite different path prefixes)
   - For each pair, generate an "insight candidate" via LLM: "I notice [memory A] from domain X shares patterns with [memory B] from domain Y. The connection might be..."
   - Apply the "homuncular observer": LLM evaluates whether the insight is genuinely useful or just noise
   - Store validated insights as workspace memories at `daily/consolidation/insights/YYYY-MM-DD.md`
   - Budget: max 3 LLM calls per cycle

3. **Implement confidence staging** (~200 lines):
   - Confidence level stored in memory metadata: `metadata.confidence_stage`
   - New memories start as `"raw"`
   - After NREM replay: promoted to `"replayed"`
   - After cross-reference check (no contradictions with other memories): promoted to `"validated"`
   - After repeated access (access_count > 5): promoted to `"promoted"`

4. **Update heartbeat scheduling**: REM imagination runs every 6 hours, hypnagogic creativity every 24 hours.

**Testing strategy**:
- Unit tests: confidence staging transitions
- Integration test with `StubLlm`: verify REM imagination produces counterfactual memories
- Integration test: verify hypnagogic creativity finds cross-domain connections between intentionally related but differently-pathed memories
- Budget enforcement tests: verify the system never exceeds its LLM call budget

**Rollback plan**: Disable via `CONSOLIDATION_REM_ENABLED=false` and `CONSOLIDATION_CREATIVITY_ENABLED=false`. The enhanced heartbeat (Phase 2.4) continues with NREM replay and threat rehearsal only.

**Success criteria**:
- [ ] REM imagination produces at least one actionable counterfactual per cycle
- [ ] Hypnagogic creativity finds at least one genuine cross-domain connection per week
- [ ] Total consolidation cost < $1/day with default settings
- [ ] Prompt templates live in `prompts/*.md` files (not inline Rust strings)

**Estimated effort**: 10-14 developer-days.

---

## Phase 4: Advanced Features (Ongoing / Research)

These are long-term projects that build on the Phase 1-3 foundation. They are listed here for completeness and planning purposes, but each requires its own detailed design document before implementation begins.

### Phase 4.1: NEAR On-Chain Identity & Reputation
- **Prerequisite**: NEAR SDK integration, operational deployment
- **What**: Soulbound agent passports (non-transferable NFTs), 7-domain EMA reputation tracking (Code Quality, Reliability, Accuracy, Creativity, Collaboration, Security, Efficiency), bounty marketplace with escrow
- **Start with**: Off-chain reputation tracking for installed extensions/tools (no blockchain needed). Extension reputation based on success rate, latency, safety violations. Track in `src/registry/` metadata.
- **Source concept**: Doc 08 (On-Chain Reputation). See also Doc 24 (Smart Contracts) for the 13-contract Solidity suite.
- **Estimated effort**: 15-25 developer-days for off-chain reputation; additional 20-30 for on-chain contracts

### Phase 4.2: Pheromone System (Multi-Agent Coordination)
- **Prerequisite**: Multi-agent support, workspace sharing
- **What**: Seven pheromone types (Threat, Opportunity, Wisdom, Alpha, Pattern, Anomaly, Consensus) implemented as tagged workspace memories with configurable half-life decay. SINR interference model prevents pheromone flooding.
- **Start with**: Simple implementation using existing `memory_write` with pheromone-prefixed paths and HalfLife decay (from Phase 1.3). One agent deposits pheromones; the next agent's session reads them.
- **Source concept**: Doc 13 (Cognitive Architecture), "Stigmergic Coordination" section
- **Estimated effort**: 8-12 developer-days for basic implementation

### Phase 4.3: Code Intelligence (Multi-Modal Index)
- **Prerequisite**: tree-sitter integration, HDC (Phase 2.2)
- **What**: Four-mode code indexing (Symbol, Graph/PageRank, HDC fingerprint, FTS5) with RRF merge. Enables "what functions call X?", "find code similar to this pattern", and impact analysis.
- **Start with**: Symbol index for Rust only (use `syn` crate for parsing). Graph index with basic call-graph extraction. HDC fingerprints per function.
- **Source concept**: Doc 12 (Code Intelligence). See also Doc 22 (Language Support) for the multi-language provider architecture.
- **Estimated effort**: 15-20 developer-days for Rust-only; 5-10 per additional language

### Phase 4.4: Full Affect Engine
- **Prerequisite**: User engagement data collection, profile enhancement
- **What**: PAD (Pleasure-Arousal-Dominance) vectors tracking user and agent affective state across three temporal layers (Emotion/Mood/Temperament). Somatic markers for tool selection. Behavioral states (Engaged/Struggling/Coasting/Exploring/Focused/Resting) modulating response style.
- **Start with**: User engagement tracker in `src/profile.rs` -- derive PAD from interaction patterns (message frequency, feedback signals, task completion rate). Use to adjust response verbosity and proactivity.
- **Source concept**: Doc 03 (Affect Engine). See also Doc 25 (Research Citations) for the Mehrabian & Russell (1974) and Gebhard (2005) references.
- **Estimated effort**: 6-10 developer-days for engagement tracker; 15-20 for full engine

### Phase 4.5: Budget-Constrained Composition (VCG Auction)
- **Prerequisite**: Progressive tool disclosure (already partially implemented in IronClaw, flag-gated in `crates/ironclaw_engine/`)
- **What**: VCG auction-based prompt assembly where subsystems (skills, memory, tools, history) bid for context window space. 9-layer cache-aware prompt builder with strategic placement (U-shaped attention). Thompson Sampling bidders that learn which content is most valuable.
- **Start with**: Cache-aware prompt ordering -- place static content (identity, safety rules) first with cache breakpoints, dynamic content (tools, skills, memories) second. Anthropic's API already supports cache control headers.
- **Source concept**: Doc 09 (Budget-Constrained Composition)
- **Estimated effort**: 8-12 developer-days for cache-aware ordering; 15-25 for full VCG auction

---

## Dependency Graph

This graph shows why items are ordered the way they are. Solid arrows (`-->`) indicate hard dependencies (must complete before starting). Dashed arrows (`..>`) indicate soft dependencies (benefits from but does not require).

```
Phase 1 (Independent, parallel)
  1.1 Robust Stats ──────────────────────────────────────────────────────┐
  1.2 Metacognitive Monitor ─────────────────────────────────────────────┤
  1.3 Ebbinghaus Decay ──────────────────────────────────────────────────┤
  1.4 BLAKE3 Dedup ──────────────────────────────────────────────────────┤
                                                                         │
Phase 2 (Independent, some build on Phase 1)                             │
  2.1 Cascade Router (no deps) ──────────────────────────────────────────┤
  2.2 HDC (no deps) ─────────────────────────────────────────────────────┤
  2.3 Gate Pipeline (no deps) ───────────────────────────────────────────┤
  1.3 ──> 2.4 Enhanced Heartbeat (needs Ebbinghaus for strengthening) ──┤
                                                                         │
Phase 3 (Build on Phase 2)                                               │
  2.3 ..> 3.1 DAG Engine (gate cells are useful but not required) ──────┤
  2.1 --> 3.2 Conductor (needs router for routing action) ──────────────┤
  2.1 --> 3.3 Cognitive Speeds (needs router for model tier mapping) ────┤
  2.4 + 2.2 --> 3.4 Full Dreams (needs heartbeat + HDC for creativity) ─┤
                                                                         │
Phase 4 (Long-term, independent)                                         │
  NEAR SDK ───> 4.1 On-Chain Reputation ────────────────────────────────┤
  Multi-agent ──> 4.2 Pheromone System ─────────────────────────────────┤
  2.2 + tree-sitter ──> 4.3 Code Intelligence ──────────────────────────┤
  Profile data ──> 4.4 Full Affect Engine ──────────────────────────────┤
  Prog. tool disc. ──> 4.5 Budget Composition ──────────────────────────┤
                                                                         │
                                                                         v
                                                              IronClaw Enhanced
```

**Why this ordering**:

- **Phase 1 first** because every item is independent, low-risk, and immediately useful. They also establish patterns (decay, hashing, monitoring) that Phase 2 builds on.
- **Phase 2 items are independent of each other** but some depend on Phase 1. The cascade router (2.1) is the most impactful Phase 2 item because Phase 3.2 and 3.3 both depend on it.
- **Phase 3 items are interdependent** through Phase 2. The DAG engine (3.1) is the most independent Phase 3 item; it can start as soon as Phase 2.3 is complete (or even earlier, without gate cells). Full dreams (3.4) is the most dependent, requiring both 2.4 and 2.2.
- **Phase 4 items depend on external prerequisites** (NEAR SDK, multi-agent support, tree-sitter) that are beyond the scope of this roadmap.

### Critical Path

The longest dependency chain determines the minimum calendar time:

```
1.3 Ebbinghaus Decay (2w) --> 2.4 Enhanced Heartbeat (3w) --> 3.4 Full Dreams (6w) = 11 weeks

2.1 Cascade Router (3w) --> 3.2 Conductor (4w) = 7 weeks
2.1 Cascade Router (3w) --> 3.3 Cognitive Speeds (2w) = 5 weeks
```

The critical path is through the dream consolidation pipeline (11 weeks). The cascade router path (7 weeks) can run in parallel.

---

## Validation Checkpoints

After each phase, verify that the investment delivered value before proceeding. These are concrete, measurable checkpoints.

### After Phase 1 (Week 4)
- [ ] `cargo test` passes with all new tests (expect 30+ new test functions)
- [ ] `cargo clippy --all --benches --tests --examples --all-features` has zero warnings
- [ ] Estimation accuracy improves by 15%+ on a held-out test set (robust statistics)
- [ ] Alternating stuck pattern detected within 10 iterations (metacognitive monitor)
- [ ] Workspace duplicate entries reduced by 50%+ on a test workspace with known duplicates (BLAKE3 dedup)
- [ ] Infrequently accessed memories decay below search threshold after approximately 3 hours (Ebbinghaus decay)
- [ ] All existing test suites pass with zero regressions

### After Phase 2 (Week 12)
- [ ] LLM cost per request reduced by 20%+ compared to Phase 1 baseline (cascade router)
- [ ] `memory_search` recall improves by 10%+ on compositional queries (HDC)
- [ ] Gate pipeline catches 90%+ of compile errors and lint warnings in generated code (gate verification)
- [ ] Consolidation produces at least 5 strengthened memories and 2 defensive strategies per day (enhanced heartbeat)
- [ ] All Phase 2 features can be independently disabled via feature flags or env vars without affecting other features

### After Phase 3 (Week 24)
- [ ] A 5-node TOML workflow executes correctly with parallel steps (DAG engine)
- [ ] Conductor predicts provider degradation at least 30 seconds before actual failure (anomaly detection)
- [ ] Cognitive speed classification matches human labeling in 85%+ of test cases
- [ ] Dream consolidation generates at least one actionable insight per week (full dreams)
- [ ] Total Phase 1-3 additions pass `cargo test` and `cargo clippy` cleanly

---

## Risk Matrix (Cross-Cutting)

These risks apply across all phases.

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| **Scope creep**: Each phase expands beyond its original scope | High | High | Strict phase boundaries. Each item is feature-gated. No item depends on unfinished items in the same phase. |
| **Dual-backend burden**: Every DB change must work on both PostgreSQL and libSQL | Medium | Medium | Phase 1 avoids DB schema changes by using the existing `metadata` JSON field. Phase 2+ migrations: write PostgreSQL first, then translate to libSQL. Test both with `cargo test --features integration`. |
| **"Everything through tools" constraint**: New features must dispatch through `ToolDispatcher` | Medium | Low | Design for this from the start. DAG cells dispatch through tools. Consolidation writes through `memory_write`. Annotate any exceptions with `// dispatch-exempt: <reason>`. |
| **Cargo workspace bloat**: Adding new crates increases compile times | Medium | Medium | Only Phase 2.2 (HDC), 2.3 (Gate), 3.1 (DAG), and 3.4 (Dreams) are new crates. Others are modules within existing crates. |
| **Testing discipline**: "Consolidate, don't proliferate" test rule | Medium | Low | Before adding a new test file, check if an existing test exercises most of the path. Document why a new test is needed. |
| **LLM data never deleted**: Dream consolidation must not delete memories | Low | High | Consolidation archives (marks via `metadata.status = "archived"`), never deletes. This is enforced by code review and by the lack of any `DELETE FROM workspace_entries` query in the codebase. |
| **Production logging constraint**: New subsystems must not use `info!` for internal diagnostics | Low | Medium | Use `debug!` for all consolidation, metacognition, and routing internals. Reserve `info!` for user-facing status in the REPL/TUI. |
| **nalgebra dependency weight**: LinUCB requires matrix operations | Low | Low | `nalgebra` is feature-gated behind `cascade-router`. It does not affect build times when the feature is disabled. |

---

## Estimated Effort Summary

| Phase | Items | Total Lines (Est.) | Developer-Days (Est.) | Calendar Time (Est.) |
|-------|-------|--------------------|-----------------------|---------------------|
| **Phase 1** | 4 items | 750-1,000 | 6-10 | 2-3 weeks |
| **Phase 2** | 4 items | 3,100-4,300 | 23-36 | 6-9 weeks |
| **Phase 3** | 4 items | 4,500-6,500 | 35-51 | 9-14 weeks |
| **Phase 4** | 5 items | 5,000-8,000 | 52-87 | 13-22 weeks |
| **Total (P1-P3)** | 12 items | 8,350-11,800 | 64-97 | 17-26 weeks |

These are estimates for a single developer working full-time. Parallel development across phases is possible for items without dependencies (e.g., 2.1 and 2.2 can be developed simultaneously by two developers, saving 3-5 weeks).

---

## Implementation Principles

These principles apply to every item in the roadmap:

1. **Each feature is a standalone crate or module** -- no monolithic cross-cutting changes. A feature that touches 10+ files is too big; break it down.

2. **Feature flags for everything** -- `#[cfg(feature = "hdc")]`, `#[cfg(feature = "cascade-router")]`, `#[cfg(feature = "gate")]`, `#[cfg(feature = "conductor")]`, etc. Every new capability can be disabled without code changes. Runtime env vars (e.g., `CONSOLIDATION_ENABLED=false`) for features that are always compiled but optionally active.

3. **Test-first** -- per IronClaw testing discipline. Write the test that pins the behavior, watch it fail, then implement. Every bug fix includes a regression test.

4. **Consolidate, don't proliferate** -- before adding a new test file, check if an existing test exercises most of the path. Extend it with a new case or assertion. Add a new test only for genuinely distinct scenarios.

5. **Tools pipeline** -- all new features that mutate state dispatch through `ToolDispatcher::dispatch()`. This ensures audit trail, safety pipeline, and channel-agnostic behavior. Annotate exceptions with `// dispatch-exempt: <reason>`.

6. **Both backends** -- any database change must support PostgreSQL and libSQL. Phase 1 avoids this by storing new state in the existing `metadata` JSON field. Phase 2+ migrations: write PostgreSQL first, then translate.

7. **No `info!` for internals** -- all new subsystem diagnostics use `debug!` or `trace!`. `info!` is reserved for user-facing status in the REPL/TUI.

8. **No `.unwrap()` or `.expect()` in production** -- use `?` with proper error mapping via `thiserror`. Tests are fine.

9. **Prefer `crate::` for cross-module imports** -- `super::` is fine in tests and intra-module refs only. No `pub use` re-exports unless exposing to downstream consumers.

10. **Prompt templates in files** -- multi-line prompt strings (for dream consolidation, gate verification, etc.) go in `crates/*/prompts/*.md` and are loaded via `include_str!()`. Never inline large prompt templates as Rust string constants.

---

## Appendix A: Companion Document Cross-Reference

Every claim in this roadmap traces to a companion document. This table maps each phase item to its primary source and the specific sections relevant to implementation.

| Phase Item | Primary Doc | Key Sections |
|-----------|-------------|-------------|
| 1.1 Robust Stats | `11-mathematical-primitives.md` | "Robust Statistics" (trimmed mean, MAD, Hodges-Lehmann) |
| 1.2 Metacognitive Monitor | `17-agent-patterns.md` | "Pattern 4: Metacognitive Monitor" (stuck detection, cost runaway) |
| 1.3 Ebbinghaus Decay | `10-universal-engram.md` | "Ebbinghaus Forgetting Curve", "Decay Variants" |
| 1.4 BLAKE3 Dedup | `10-universal-engram.md` | "Content-Addressed Identity", "ContentHash" |
| 2.1 Cascade Router | `07-online-learning.md` | "LinUCB Algorithm", "3-Stage Cascade", "Reward Signal" |
| 2.2 HDC | `01-hyperdimensional-computing.md` | "HdcVector", "Codebook", "Knowledge Fingerprinting" |
| 2.3 Gate Pipeline | `05-gate-verification.md` | "7-Rung Pipeline", "Complexity-Driven Selection", "Verify Trait" |
| 2.4 Enhanced Heartbeat | `02-dream-consolidation.md` | "NREM Replay", "Threat Rehearsal", "Mattar-Daw Utility" |
| 3.1 DAG Engine | `04-dag-execution.md` | "Cell Trait", "TOML Loader", "Budget Tracking", "Hot Graphs" |
| 3.2 Conductor | `06-conductor-anomaly.md` | "Holt Smoothing", "10 Watchers", "Compound Patterns" |
| 3.3 Cognitive Speeds | `13-cognitive-architecture.md` | "Three Cognitive Speeds (Gamma/Theta/Delta)" |
| 3.4 Full Dreams | `02-dream-consolidation.md` | "REM Imagination", "Hypnagogic Creativity", "Staging Buffer" |
| 4.1 Reputation | `08-chain-reputation.md`, `24-smart-contracts.md` | "Soulbound Passports", "7-Domain Reputation" |
| 4.2 Pheromones | `13-cognitive-architecture.md` | "Stigmergic Coordination", "SINR Interference" |
| 4.3 Code Intelligence | `12-code-intelligence.md`, `22-language-support.md` | "4-Mode Indexing", "LanguageProvider Trait" |
| 4.4 Affect Engine | `03-affect-engine.md` | "PAD Vectors", "ALMA Temporal Layers", "Somatic Markers" |
| 4.5 Budget Composition | `09-budget-composition.md` | "VCG Auction", "U-Shaped Attention", "Thompson Sampling" |

## Appendix B: IronClaw Architecture Reference

Key IronClaw files referenced throughout this document. Read these before implementing any phase item.

| File/Module | What It Is | Relevant Phases |
|------------|-----------|-----------------|
| `src/agent/agentic_loop.rs` | Shared agentic loop engine. Contains `DuplicateToolCallTracker`, `LoopDelegate` trait, `run_agentic_loop()`. | 1.2, 2.4, 3.3 |
| `src/agent/cost_guard.rs` | LLM spend and action-rate enforcement. | 1.2, 2.1 |
| `src/agent/heartbeat.rs` | Proactive periodic execution. Reads HEARTBEAT.md. | 2.4, 3.4 |
| `src/agent/self_repair.rs` | Detects stuck jobs and broken tools. | 1.2 |
| `src/agent/dispatcher.rs` | Chat delegate for the agentic loop. | 1.2, 3.3 |
| `crates/ironclaw_llm/src/smart_routing.rs` | 13-dimension complexity scorer. `Tier` enum. Pattern overrides. | 2.1, 3.3 |
| `crates/ironclaw_llm/src/circuit_breaker.rs` | Reactive circuit breaker (Closed/Open/HalfOpen). | 3.2 |
| `src/estimation/learner.rs` | EMA-based estimation learning. `LearningModel`, `EstimationLearner`. | 1.1 |
| `src/workspace/mod.rs` | Workspace API: `read()`, `write()`, `search()`. | 1.3, 1.4, 2.4, 3.4 |
| `src/workspace/document.rs` | `MemoryDocument` struct. `metadata: serde_json::Value`. | 1.3, 1.4 |
| `src/workspace/search.rs` | Hybrid FTS + vector search with RRF merge. | 1.3, 2.2 |
| `src/tools/builtin/memory.rs` | `memory_write`, `memory_read`, `memory_search`, `memory_tree` tool implementations. | 1.3, 1.4, 2.4 |
| `src/tools/builder/validation.rs` | Current WASM tool build validation. | 2.3 |
| `src/agent/routine_engine.rs` | Cron ticker and event matcher for routines. | 3.1 |
| `src/config/llm.rs` | LLM configuration env vars. | 2.1 |
| `CLAUDE.md` | Project-wide development guide. | All |
| `src/agent/CLAUDE.md` | Agent module spec. | 1.2, 2.4, 3.3, 3.4 |
| `crates/ironclaw_llm/CLAUDE.md` | LLM module spec. | 2.1, 3.2 |

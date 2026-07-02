# Priority Matrix: Roko Concepts for IronClaw Adoption

## Purpose and Audience

This document ranks 25 concepts extracted from the [roko](https://github.com/nunchi/roko) codebase (18+ crates, 200K+ lines, 155+ research docs) for potential adoption into IronClaw, a secure personal AI assistant built in Rust. Each concept has a dedicated analysis document in this directory (referenced by number). This matrix answers: **what should we build first, what should we defer, and why?**

If you are reading this for the first time, you do not need to have read the other documents. Each ranked item below includes enough context to understand what it is, why it matters, and what building it would entail. Cross-references point to the full analysis documents for deeper exploration.

**Companion documents:**
- [00-INDEX.md](00-INDEX.md) -- master index of all 28 analysis documents
- [18-integration-roadmap.md](18-integration-roadmap.md) -- 4-phase engineering plan with dependency graph

---

## Methodology: How Items Were Ranked

### Scoring Criteria

Every concept is evaluated on five axes, each scored 1-5. Scores are combined into a single Composite ROI number that determines rank. The five axes, their weights, and what they measure:

| Axis | Weight | Scale | What It Measures |
|------|--------|-------|-----------------|
| **User Impact** | 30% | 1-5 (higher = more impact) | Direct, observable improvement to the end user: faster responses, better answers, fewer failures, lower cost |
| **System Impact** | 20% | 1-5 (higher = more impact) | Internal improvements to reliability, observability, maintainability, or extensibility |
| **Ease of Implementation** | 25% | 1-5 (higher = easier) | Inverse of effort. 5 = trivial (under 200 lines, 1 day), 1 = very hard (3000+ lines, 8+ weeks) |
| **Safety** | 15% | 1-5 (higher = safer) | Inverse of risk. 5 = purely additive, no regressions, easy to feature-flag. 1 = research-grade, uncertain feasibility, changes core flows |
| **Independence** | 10% | 1-5 (higher = more independent) | Inverse of dependency burden. 5 = no prerequisites, ships alone. 1 = blocked by 3+ other features |

All five axes use the same direction: **higher numbers are better**. This avoids the confusion of mixing "high score = good" axes with "high score = bad" axes in a single formula.

### Scoring Rubric

**User Impact:**
- 5 (Critical): Fixes an active pain point or unlocks a capability users explicitly ask for
- 4 (High): Measurably improves a common workflow or eliminates a class of failures
- 3 (Medium): Improves quality or efficiency in ways users would notice over time
- 2 (Low): Nice polish; most users would not notice its absence
- 1 (Minimal): Primarily of academic interest; no near-term user benefit

**System Impact:**
- 5 (Critical): Eliminates a systemic weakness or enables an entire class of future features
- 4 (High): Substantially improves reliability, observability, or extensibility
- 3 (Medium): Meaningful internal improvement; reduces tech debt or enables specific features
- 2 (Low): Minor internal benefit; nice cleanup
- 1 (Minimal): No significant internal benefit

**Ease of Implementation** (inverted effort):
- 5 (Trivial): Under 200 lines, 1-2 files, no migrations, done in a day
- 4 (Low): 200-500 lines, 2-4 modules, possibly a migration, done in a week
- 3 (Medium): 500-1500 lines, new module or crate, integration across subsystems, 2-4 weeks
- 2 (Hard): 1500-3000 lines, new crate with multiple modules, significant integration, 4-8 weeks
- 1 (Very Hard): 3000+ lines or major refactor of existing systems, 8+ weeks

**Safety** (inverted risk):
- 5 (Very Safe): Purely additive, no behavior change to existing code, easy to feature-flag
- 4 (Safe): Additive with minor integration points, straightforward to test
- 3 (Medium): Touches existing behavior, requires careful testing, tuning parameters
- 2 (Risky): Changes core flows, introduces new failure modes, hard to test without production data
- 1 (Very Risky): Research-grade, uncertain feasibility, may require fundamental architecture changes

**Independence** (inverted dependency burden):
- 5 (Fully Independent): Ships alone, no prerequisites
- 4 (Mostly Independent): Minor soft dependency on one other feature
- 3 (Moderate): Benefits significantly from 1-2 other features
- 2 (Dependent): Requires 1-2 features to be built first
- 1 (Heavily Dependent): Blocked by 3+ other features or infrastructure changes

### ROI Formula

```
Composite ROI = UserImpact * 0.30
              + SystemImpact * 0.20
              + Ease * 0.25
              + Safety * 0.15
              + Independence * 0.10
```

This is a weighted average of five 1-5 scores, producing a composite between 1.0 and 5.0. Higher is better. The formula weights the value axes (User Impact + System Impact = 50%) slightly higher than the cost axes (Ease + Safety + Independence = 50%), because a high-impact feature is worth building even if moderately difficult, while a low-impact feature is not worth building even if easy.

**Verification**: For the top-ranked item (Metacognitive Monitor): UserImpact=5, SystemImpact=4, Ease=4, Safety=5, Independence=5. Composite = 5(0.30) + 4(0.20) + 4(0.25) + 5(0.15) + 5(0.10) = 1.50 + 0.80 + 1.00 + 0.75 + 0.50 = **4.55**. Any reader can verify any row by plugging scores into this formula.

### What "Impact" and "Effort" Mean for IronClaw Specifically

IronClaw is a personal AI assistant with multi-channel access (CLI, web, Telegram, etc.), background execution (heartbeat), a workspace memory system, WASM-sandboxed tool extensions, and multi-provider LLM integration. Scoring is calibrated to IronClaw's specific architecture:

- **High user impact** means: fewer stuck agent loops, lower LLM costs on the user's bill, better memory recall, faster response times, or fewer failed jobs.
- **High system impact** means: better test coverage, cleaner module boundaries, easier debugging, or enabling future features.
- **High effort** (low ease) means: new crates to maintain, database migrations across both PostgreSQL and libSQL backends, integration with the ToolDispatcher pipeline, or touching the agent loop (the most complex and sensitive code path).

### Existing IronClaw Infrastructure That Affects Scoring

Several IronClaw subsystems already exist and directly influence effort estimates. Features that extend existing code score higher on Ease than features requiring greenfield work.

| Existing System | Location | How It Affects Scoring |
|----------------|----------|----------------------|
| Self-repair / stuck job detection | `src/agent/self_repair.rs` | Metacognitive Monitor extends turn-level detection on top of existing job-level detection. Enhancement, not greenfield. |
| CostGuard | `src/agent/cost_guard.rs` | Tracks daily budget + hourly rate limits. Metacognitive Monitor adds EWMA trend forecasting. |
| SmartRoutingProvider | `crates/ironclaw_llm/src/smart_routing.rs` | Already routes cheap vs primary model via 13-dimension static scorer. Cascade Router adds **learning** (LinUCB bandit) -- the hard part. |
| Heartbeat | `src/agent/heartbeat.rs` | Runs every 30 min, reads HEARTBEAT.md. Dream consolidation extends this with replay and rehearsal. |
| BLAKE3 dependency | `Cargo.toml` (`blake3 = "1"`) | Already in dependency tree via WASM storage. Content dedup requires zero new deps. |
| RRF merge in memory search | `src/workspace/search.rs` | Memory search already fuses FTS + vector via RRF. HDC adds a third signal to existing fusion -- incremental. |
| Progressive tool disclosure | `crates/ironclaw_engine/` (flag-gated, default off) | Budget composition enhances a feature that is already partially implemented. |
| Dual-backend DB | `src/db/` (PostgreSQL + libSQL) | Any DB migration must be implemented and tested twice. Increases effort for features needing schema changes. |
| Evaluation system | `src/evaluation/` | Uses EMA-based learning. Composable scorers replace ad-hoc scoring with a reusable trait framework. |
| Circuit breaker + retry + failover | `crates/ironclaw_llm/` | LLM provider chain already has CircuitBreaker, Retry, and Failover decorators. Conductor formalizes health monitoring that feeds into these. |
| Estimation module | `src/estimation/` | Uses EMA for cost/time estimation. Currently uses arithmetic mean; robust statistics replace this. |

---

## Quadrant Diagram

Each concept is placed by its composite impact (average of User Impact and System Impact, y-axis) against its composite cost (average of Ease and Safety, inverted so higher = more effort/risk, x-axis). Items in the upper-left are the highest-priority work.

```
                              HIGH IMPACT
                                  |
     5 +...........................|...........................+
       |                          |                           |
       |    STARS                  |    BIG BETS               |
       |    (Build first --        |    (Plan carefully --     |
       |     best ROI,             |     high value but        |
       |     significant effort)   |     significant effort)   |
       |                          |                           |
     4 +  [1] Metacognitive  .....|..... [5] Cascade Router  +
       |  [2] Ebbinghaus Decay    |      [6] HDC Similarity   |
       |  [3] BLAKE3 Dedup        |      [7] Gate Pipeline    |
       |  [4] Robust Statistics   |      [8] Enhanced Hrtbeat |
       |                          |      [12] Conductor       |
       |                          |      [13] DAG Execution   |
     3 +..........................|............................+
       |                          |                           |
       |    NICE TO HAVE          |    LONG-TERM /            |
       |    (Low effort,          |    RESEARCH               |
       |     modest returns --    |    (Defer until           |
       |     fill gaps when       |     foundation is         |
       |     convenient)          |     solid)                |
       |                          |                           |
     2 +  [9]  Hier. Cancel  .....|..... [21] NEAR Rep       +
       |  [10] Composable Scorers |      [22] Pheromone Sys   |
       |  [11] Cog Speed Labels   |      [23] Full Affect     |
       |  [16] Decl. TOML Tools   |      [24] TDA / Sheaves   |
       |                          |      [25] Pure SM Extract |
       |  [14] Resum. Checkpoints |      [20] Code Intel      |
       |  [15] EventBus           |      [19] Budget Comp     |
       |  [17] User Engage PAD    |      [18] Full Dream Cons |
     1 +...........................|...........................+
       |                          |                           |
       1          2          3    |    4          5           |
                                  |                           |
              LOW EFFORT ---------|---------- HIGH EFFORT
```

Reading the quadrant:
- **Upper-left (Stars)**: Highest priority. Most value for least work. Build in the order listed.
- **Upper-right (Big Bets)**: High value but require significant planning and execution. Start after Stars are complete.
- **Lower-left (Nice to Have)**: Low effort, lower impact. Good fill-in work between major projects.
- **Lower-right (Long-Term)**: High effort, lower near-term impact. Research projects or features that become valuable only after other infrastructure exists.

---

## Detailed Scoring Table

Every score is verifiable: plug any row's five scores into `Composite = U*0.30 + S*0.20 + E*0.25 + Sa*0.15 + I*0.10` to reproduce the ROI.

| Rank | Concept | User (U) | System (S) | Ease (E) | Safety (Sa) | Indep (I) | **Composite** | Source Doc |
|------|---------|----------|------------|----------|-------------|-----------|--------------|-----------|
| 1 | Metacognitive Monitor | 5 | 4 | 4 | 5 | 5 | **4.55** | [17](17-agent-patterns.md) |
| 2 | Ebbinghaus Decay | 4 | 4 | 5 | 5 | 5 | **4.50** | [10](10-universal-engram.md) |
| 3 | BLAKE3 Content Dedup | 4 | 3 | 5 | 5 | 5 | **4.30** | [10](10-universal-engram.md) |
| 4 | Robust Statistics | 3 | 4 | 5 | 5 | 5 | **4.20** | [11](11-mathematical-primitives.md) |
| 5 | Composable Scorers | 2 | 4 | 5 | 5 | 5 | **3.90** | [17](17-agent-patterns.md) |
| 6 | Hierarchical Cancellation | 2 | 4 | 4 | 4 | 5 | **3.50** | [14](14-runtime-infrastructure.md) |
| 7 | Cascade Router | 5 | 4 | 3 | 4 | 5 | **4.15** | [07](07-online-learning.md) |
| 8 | Cognitive Speed Labels | 3 | 3 | 4 | 4 | 4 | **3.50** | [13](13-cognitive-architecture.md) |
| 9 | HDC Similarity | 4 | 4 | 3 | 4 | 5 | **3.85** | [01](01-hyperdimensional-computing.md) |
| 10 | Gate Pipeline (Rungs 1-4) | 4 | 4 | 3 | 4 | 5 | **3.85** | [05](05-gate-verification.md) |
| 11 | Enhanced Heartbeat | 4 | 3 | 3 | 3 | 3 | **3.30** | [02](02-dream-consolidation.md) |
| 12 | DAG Execution Engine | 4 | 4 | 2 | 3 | 4 | **3.35** | [04](04-dag-execution.md) |
| 13 | EventBus | 2 | 4 | 3 | 4 | 5 | **3.25** | [14](14-runtime-infrastructure.md) |
| 14 | Conductor | 4 | 4 | 2 | 3 | 3 | **3.25** | [06](06-conductor-anomaly.md) |
| 15 | Resumable Checkpoints | 3 | 3 | 3 | 3 | 5 | **3.20** | [17](17-agent-patterns.md) |
| 16 | Declarative TOML Tools | 3 | 2 | 3 | 4 | 5 | **3.15** | [16](16-plugin-extension.md) |
| 17 | User Engagement PAD | 3 | 2 | 3 | 3 | 5 | **3.00** | [03](03-affect-engine.md) |
| 18 | Full Dream Consolidation | 4 | 3 | 2 | 2 | 2 | **2.80** | [02](02-dream-consolidation.md) |
| 19 | Budget Composition (VCG) | 3 | 3 | 2 | 3 | 3 | **2.75** | [09](09-budget-composition.md) |
| 20 | Code Intelligence | 3 | 3 | 2 | 3 | 2 | **2.65** | [12](12-code-intelligence.md) |
| 21 | NEAR On-Chain Reputation | 3 | 2 | 2 | 2 | 2 | **2.30** | [08](08-chain-reputation.md) |
| 22 | Pheromone System | 2 | 3 | 2 | 3 | 1 | **2.25** | [13](13-cognitive-architecture.md) |
| 23 | Pure SM Extraction | 2 | 4 | 1 | 1 | 4 | **2.20** | [14](14-runtime-infrastructure.md) |
| 24 | Full Affect Engine | 2 | 2 | 2 | 2 | 3 | **2.10** | [03](03-affect-engine.md) |
| 25 | TDA / Sheaves | 1 | 2 | 1 | 2 | 3 | **1.55** | [11](11-mathematical-primitives.md) |

**Reading the table:** Items are grouped into priority tiers (Stars, Big Bets, Nice to Have, Long-Term) and sorted by composite ROI within each tier. The tier assignment reflects both the composite score and practical scheduling considerations (some items with similar scores differ in when they should be built due to dependencies).

### Priority Tiers

| Tier | Ranks | Composite Range | When to Build |
|------|-------|----------------|---------------|
| **Stars** | 1-6 | 3.50-4.55 | Week 1-2 (all are low-effort, high-safety) |
| **Big Bets** | 7-10 | 3.85-4.15 | Weeks 3-8 (high impact but require significant engineering) |
| **Nice to Have** | 11-17 | 3.00-3.35 | As convenient (between major projects) |
| **Long-Term** | 18-25 | 1.55-2.80 | Research phase or when prerequisites exist |

Note that the Cascade Router (Rank 7) has a composite of 4.15 -- close to Robust Statistics (Rank 4, composite 4.20) -- but is placed in Big Bets rather than Stars because its Ease score of 3 indicates 2-3 weeks of work, while all Stars can be completed in 1-5 days each. The ranking system separates "when to build" from "how valuable is it."

---

## Detailed Justifications

Each entry below explains *why* the concept received its scores, what risks could shift the assessment, and what dependencies affect when it can be built.

---

### Rank 1: Metacognitive Monitor (Composite: 4.55)

**What it is:** A self-monitoring layer inside the agent loop that detects three pathologies: stuck loops (the agent repeating the same actions), cost runaways (spending trending toward budget exhaustion), and self-contradiction (the agent asserting X then asserting not-X). When a pathology is detected, the monitor injects a corrective action -- breaking the loop, downgrading the model, or flagging the contradiction.

**Why User Impact = 5 (Critical):** Stuck agent loops are the single most common failure mode in production AI assistants. When an agent gets stuck, the user watches their LLM bill climb while the agent produces no useful work. A monitor that detects and breaks the loop within 3-4 iterations instead of letting it run for 20+ turns directly saves money and time. Cost runaway detection prevents surprise bills. Contradiction detection improves answer quality.

**Why Ease = 4 (Low effort):** IronClaw already has relevant infrastructure in two places:
- `src/agent/self_repair.rs` detects "stuck jobs" at the job level (jobs in `InProgress` state longer than a threshold). The metacognitive monitor extends this to the *turn* level, detecting repetitive actions within a single job.
- `src/agent/cost_guard.rs` tracks spending and enforces budgets (daily cents cap, hourly rate limit). The monitor adds *trend forecasting* (EWMA projection) so it can predict cost overrun before it happens.

The implementation is approximately 400-500 lines: a `MetacognitiveMonitor` struct with a sliding window of recent actions (for loop detection), a cost trend forecaster wrapping the existing `CostGuard` with EWMA projection, and a simple assertion tracker (hash recent claims, check for conflicts). Integration is a few lines in `src/agent/agentic_loop.rs` -- call `monitor.observe()` after each iteration.

**Why Safety = 5 (Very Safe):** Purely additive. The monitor observes and recommends but does not alter existing control flow unless a pathology is detected. Feature-flaggable. False positives (incorrectly detecting a loop when the agent is legitimately retrying) are mitigated by tuning the uniqueness threshold (start conservative at 25% unique actions over a window of 10, tune from there).

**Dependencies:** None. Ships independently.

**Estimated time:** 2-3 days.

**Implementation location:** `src/agent/metacognitive.rs` (new file), with integration points in `src/agent/agentic_loop.rs`.

---

### Rank 2: Ebbinghaus Decay for Memory (Composite: 4.50)

**What it is:** A memory decay model based on the Ebbinghaus forgetting curve from cognitive psychology (Hermann Ebbinghaus, 1885). Each memory entry gets a "stability" value. Memories that are never accessed decay exponentially -- their retrieval strength fades over time. Each time a memory is accessed (retrieved by `memory_search` or `memory_read`), its stability doubles, making it decay more slowly. The effect: frequently-used memories become effectively permanent, while one-time observations gradually fade below the retrieval threshold.

**Why User Impact = 4 (High):** IronClaw's workspace memory currently has no decay. Every memory entry persists at full strength forever. Over weeks and months of use, the memory system accumulates stale, irrelevant, and outdated entries that pollute search results. The user experience degrades as the agent retrieves increasingly irrelevant context. Ebbinghaus decay requires no manual curation, no "forget" commands, and no cleanup routines. Memories that stay relevant naturally survive because they keep getting accessed.

**Why Ease = 5 (Trivial):** Approximately 150 lines plus a database migration:
1. Add a `DecayVariant` enum with four variants (`None`, `HalfLife`, `Ttl`, `Ebbinghaus`) -- about 40 lines including the `current_strength()` method.
2. Add `decay_variant`, `stability`, `last_accessed`, and `access_count` columns to the memory tables in both PostgreSQL and libSQL -- two migration files.
3. In `memory_search`, multiply each result's relevance score by `decay.current_strength(now)` -- 5 lines.
4. In `memory_read`, call `entry.on_access()` which doubles stability and resets `last_accessed` -- 10 lines.
5. Default new memories to `Ebbinghaus { stability: 3600.0 }` (1-hour initial half-life) -- 2 lines.

No new crates, no new dependencies, no changes to the agent loop.

**Why Safety = 5 (Very Safe):** The only risk is choosing poor default stability values. Starting at 1-hour initial stability with doubling per access is conservative -- a memory accessed once per day reaches 30-day stability within a week. Existing memories can be migrated with `DecayVariant::None` to preserve backward compatibility.

**Dependencies:** None. Ships independently. Synergizes strongly with BLAKE3 Dedup (#3) -- when a duplicate is detected, bumping the existing entry's access count strengthens its decay stability.

**Estimated time:** 1-2 days.

**Implementation location:** `src/workspace/mod.rs` (decay types), `src/workspace/repository.rs` (queries), `src/db/` (migrations for both backends).

---

### Rank 3: BLAKE3 Content Deduplication (Composite: 4.25)

**What it is:** Before writing a new memory entry via `memory_write`, compute the BLAKE3 hash of the content. If an entry with the same hash already exists, merge metadata (update tags, bump access count, update timestamps) instead of creating a duplicate. BLAKE3 is already in IronClaw's dependency tree (`Cargo.toml`: `blake3 = "1"`) and used in WASM storage.

**Why User Impact = 4 (High):** Memory duplication is a common problem in long-running agent sessions. The agent learns "User prefers Rust" in session 1, learns it again in session 3, and again in session 7. Each duplicate wastes storage, pollutes search results with redundant entries, and consumes context window tokens when injected into prompts. Dedup eliminates this class of waste entirely.

**Why Ease = 5 (Trivial):** Approximately 100 lines of code:
1. Add a `content_hash BLOB` column (32 bytes) to memory tables -- one migration per backend.
2. In `memory_write`, compute `blake3::hash(content.as_bytes())` -- 1 line (zero new deps).
3. Before INSERT, check for existing entry with same hash -- 1 query.
4. If found, UPDATE metadata instead of INSERT -- 10 lines.

**Why Safety = 5 (Very Safe):** BLAKE3 collision probability is negligible (2^-128). The only subtlety is deciding whether whitespace normalization should be applied before hashing. The simplest approach (hash the raw content string) handles 95% of duplicates. Normalization can be added later if needed.

**Dependencies:** None. Ships independently.

**Estimated time:** 1 day.

**Implementation location:** `src/tools/builtin/memory.rs` (memory_write tool), `src/workspace/repository.rs` (queries), `src/db/` (migrations).

---

### Rank 4: Robust Statistics (Composite: 4.15)

**What it is:** Replace naive mean/standard-deviation computations in estimation and evaluation code with outlier-resistant alternatives: trimmed mean (discard top/bottom k% before averaging), MAD (Median Absolute Deviation, a robust alternative to standard deviation), and Hodges-Lehmann estimator (median of all pairwise averages). These are standard statistical techniques validated across decades of practice.

**Why User Impact = 3 (Medium):** The benefit is indirect but real. IronClaw's `src/estimation/learner.rs` uses EMA (Exponential Moving Average) for cost/time estimation. EMA is better than raw arithmetic mean but still sensitive to outliers (a single anomalous LLM response can skew the running average). Robust statistics produce stable estimates regardless of outliers, which means better predictions ("this will take about 30 seconds and cost $0.05" instead of wildly inaccurate estimates).

**Why System Impact = 4 (High):** Pure utility functions that improve accuracy across multiple subsystems (`src/estimation/`, `src/evaluation/`). Any future feature that computes statistics (Cascade Router reward signals, Conductor health metrics, dream consolidation utility scores) inherits the improvement automatically.

**Why Ease = 5 (Trivial):** Approximately 100-200 lines of pure functions:
```rust
pub fn trimmed_mean(values: &[f64], trim_fraction: f64) -> f64;
pub fn mad(values: &[f64]) -> f64;
pub fn hodges_lehmann(values: &[f64]) -> f64;
pub fn median(values: &[f64]) -> f64;
```
No dependencies, no database changes, no integration complexity.

**Dependencies:** None. Foundation for Cascade Router (#7) reward signal computation.

**Estimated time:** 1 day.

**Implementation location:** `src/util.rs` or new `crates/ironclaw_math/src/lib.rs`.

---

### Rank 5: Composable Scorers (Composite: 3.85)

**What it is:** A `Scorer` trait with combinators that allow building complex scoring functions from simple, reusable primitives. Combinators include `weighted` (weighted average of scorers), `threshold` (pass/fail gate), and `chain` (sequential pipeline). Replaces ad-hoc scoring logic scattered across `src/evaluation/` and `src/skills/`.

**Why User Impact = 2 (Low):** Users do not interact with scoring directly. The benefit is indirect: better scoring leads to better skill selection, better evaluation, and better tool recommendations.

**Why System Impact = 4 (High):** Ad-hoc scoring scattered across modules is the kind of code that accumulates bugs and resists testing. A composable scorer framework makes scoring logic testable in isolation, composable without copy-paste, and configurable without code changes. Foundation for Gate Pipeline (#10) pass/fail criteria.

**Why Ease = 5 (Trivial):** Approximately 200 lines: the `Scorer` trait, three combinator functions, and a few concrete implementations. No dependencies, no database changes.

**Dependencies:** None.

**Estimated time:** 1-2 days.

**Implementation location:** `src/evaluation/scorer.rs` (new file), with usage in `src/skills/`, `src/evaluation/`.

---

### Rank 6: Hierarchical Cancellation Tokens (Composite: 3.50)

**What it is:** Structured cancellation tokens that form a tree: a session token is the root, job tokens are children, tool call and LLM call tokens are grandchildren. Cancelling a parent cascades to all descendants. Cancelling a child leaves the parent alive. IronClaw currently uses ad-hoc tokio cancellation channels; this replaces them with a formal hierarchy.

**Why User Impact = 2 (Low):** Users rarely cancel operations, and when they do, the current ad-hoc cancellation mostly works. The improvement is in edge cases: cancelling a single tool call without killing the entire session, or cancelling a session and having all nested operations stop promptly instead of leaking.

**Why System Impact = 4 (High):** Leaked tasks (tool calls that continue running after their parent session is gone) waste resources and produce confusing logs. A formal cancellation hierarchy makes resource cleanup deterministic and debuggable.

**Why Ease = 4 (Low effort):** `tokio_util::sync::CancellationToken` already provides exactly this abstraction. Integration means replacing ad-hoc cancellation channels with token creation at session/job/tool boundaries. Approximately 200-300 lines.

**Dependencies:** None.

**Estimated time:** 3-5 days.

**Implementation location:** `src/agent/`, `src/context/`.

---

### Rank 7: Cascade Router (Composite: 4.15)

**What it is:** A 3-stage model routing system that selects which LLM model to use for each request, optimizing for cost-quality-latency tradeoffs. Stage 1 applies static rules (known patterns like "translate this" always go to the cheapest model). Stage 2 checks rule confidence. Stage 3 uses a LinUCB contextual bandit that learns from every request which model works best for which context. The context is encoded as an 18-dimensional feature vector (task type, complexity, tool count, budget remaining, etc.).

**Why User Impact = 5 (Critical):** This is the single highest-impact feature for cost savings. IronClaw already has a `SmartRoutingProvider` in `crates/ironclaw_llm/src/smart_routing.rs` that routes cheap vs primary model using a static 13-dimension scorer. But the scorer does not learn -- it uses fixed thresholds. The Cascade Router replaces static thresholds with a LinUCB contextual bandit that learns user-specific patterns. Based on roko's analysis (doc 07), 40-60% of LLM requests can be handled by cheaper models without quality loss, projecting 30-50% cost reduction. For a user spending $100/month on LLM calls, that is $30-50/month in savings.

**Why Ease = 3 (Medium):** Approximately 1,500 lines across several components. The LinUCB bandit itself is ~300-400 lines of straightforward linear algebra. The cascade router is ~400-500 lines. Feature encoding is ~200 lines. Episode logging is ~200 lines. The hardest part is integration with the existing `SmartRoutingProvider` -- the router must intercept LLM calls, observe outcomes, and feed reward signals back. The existing provider chain architecture (decorator pattern) makes this feasible but requires touching `crates/ironclaw_llm/`.

**Why Safety = 4 (Safe):** The router can gracefully fall back to current behavior (use the configured default model) when the bandit is uncertain. The learning curve is the main risk: if the bandit receives bad reward signals, it may learn incorrectly. Mitigation: start with a simple reward signal (task completion + cost) and add user feedback later. Convergence within 200-500 requests per task type.

**Dependencies:** Benefits from Robust Statistics (#4) for reward signals and Cognitive Speed Labels (#8) for task classification, but neither is required.

**Estimated time:** 2-3 weeks.

**Implementation location:** New module in `crates/ironclaw_llm/src/router/` or new `crates/ironclaw_router/` crate.

---

### Rank 8: Cognitive Speed Labels (Composite: 3.50)

**What it is:** An explicit classification of agent processing into three speeds: Gamma (reactive, 5-15 seconds, cheapest model), Theta (reflective, ~75 seconds, standard model), and Delta (consolidation, hours, best model). This formalizes the reactive/reflective/background split that IronClaw already exhibits informally and maps each speed to a model tier.

**Why User Impact = 3 (Medium):** Combined with the Cascade Router (#7), speed labels drive model selection automatically. Simple "what time is it?" queries use the cheapest model. Multi-step planning uses a standard model. Background learning uses the best model.

**Why Ease = 4 (Low effort):** Approximately 200-300 lines: a `CognitiveSpeed` enum, a classifier function, and model tier mapping. The classifier examines request context (message length, tool call presence, conversation depth) to determine speed.

**Dependencies:** Benefits significantly from Cascade Router (#7) -- speed labels are most valuable when they drive model routing.

**Estimated time:** 3-5 days.

**Implementation location:** `src/agent/cognitive_speed.rs` (new file).

---

### Rank 9: HDC Similarity Engine (Composite: 3.85)

**What it is:** Hyperdimensional Computing encodes semantic information into 10,240-bit binary vectors. Operations on these vectors (XOR for binding, majority vote for bundling, bit rotation for sequencing) preserve semantic relationships. Similarity is computed via Hamming distance -- pure bitwise operations, no model inference, no GPU required. Scanning 100,000 HDC vectors takes under 1 millisecond on modern hardware (doc 01).

**Why User Impact = 4 (High):** IronClaw's memory search currently uses FTS + vector embeddings merged via RRF. HDC adds a third signal that captures *compositional* similarity that embeddings miss. You can query "find memories similar to the combination of concept X and concept Y" by binding their vectors -- something no embedding model can do directly. For tool and skill selection, HDC provides O(n) scans with bitwise ops that are faster than regex/keyword matching for large registries.

**Why Ease = 3 (Medium):** Approximately 800-1,200 lines for the core crate plus 300-400 lines of integration across workspace, skills, and tools subsystems.

**Dependencies:** None strictly required. Benefits from BLAKE3 Dedup (#3) -- HDC fingerprints can complement content hashing.

**Estimated time:** 2-3 weeks.

**Implementation location:** New `crates/ironclaw_hdc/` crate. Integration in `src/workspace/`, `src/skills/`, `src/tools/registry.rs`.

---

### Rank 10: Gate Verification Pipeline (Rungs 1-4) (Composite: 3.85)

**What it is:** A progressive verification pipeline with increasing rigor. Four rungs: (1) Compile -- syntax/type checking, (2) Lint -- static analysis, (3) Test -- run existing test suite, (4) Symbol -- verify all references resolve. Complexity is assessed automatically. A process reward model tracks "promise" (expected final quality) and "progress" (how far through the pipeline), enabling early termination when promise drops below threshold.

**Why User Impact = 4 (High):** When IronClaw's agent generates code (tool builder, sandbox execution), quality varies. The first four rungs catch the most common errors before the user sees them. Especially valuable for `src/tools/builder/` and `src/sandbox/`, where running unverified code wastes container resources and user time.

**Why Ease = 3 (Medium):** Approximately 1,000 lines for rungs 1-4. Each rung must handle multiple languages (at least Rust). Starting with Rust-only reduces initial effort significantly. Each rung is a shell-out to compiler/linter/test runner with output parsing.

**Dependencies:** None strictly required. Benefits from Composable Scorers (#5) for rung evaluation criteria and DAG Execution (#12) for parallel rung execution.

**Estimated time:** 2-4 weeks.

**Implementation location:** New `crates/ironclaw_gate/` crate or extension of `src/tools/builder/validation.rs`.

---

### Rank 11: Enhanced Heartbeat / Dream Consolidation Lite (Composite: 3.30)

**What it is:** An extension of IronClaw's existing heartbeat system with two dream consolidation subsystems: NREM Replay (review recent interactions, strengthen patterns, increase confidence on validated memories) and Threat Rehearsal (analyze recent failures, generate root cause hypotheses, prepare defensive strategies). This is a lightweight version of full dream consolidation (Rank 19) -- it includes the two cheapest subsystems and defers the expensive ones (REM Imagination, Hypnagogic Creativity).

**Why User Impact = 4 (High):** The current heartbeat reads `HEARTBEAT.md` and executes instructions. Enhanced heartbeat adds proactive learning: the agent reviews what went well and wrong, strengthening good patterns and preparing for failure scenarios. Over time, the agent becomes noticeably better at tasks the user does frequently.

**Why Ease = 3 (Medium):** Approximately 800 lines. Background LLM calls need careful resource management (budget caps, timeout handling, non-blocking execution) and all actions must go through `ToolDispatcher` per IronClaw's "everything through tools" principle.

**Why Safety = 3 (Medium):** Background LLM calls introduce a new cost vector. If the budget for consolidation is too generous, it burns money on low-value replay. Mitigation: hard cap consolidation budget at 5% of the user's configured monthly budget.

**Dependencies:** Depends on Ebbinghaus Decay (#2) -- replay strengthens memories by bumping access counts, which only makes sense if decay exists.

**Estimated time:** 2-3 weeks.

**Implementation location:** Extend `src/agent/heartbeat.rs`, new `src/consolidation/` module.

---

### Rank 12: DAG Execution Engine (Composite: 3.35)

**What it is:** A directed acyclic graph executor where nodes are "Cells" (units of work: LLM calls, tool invocations, shell commands) and edges define data flow with conditional routing. Workflows defined in TOML files, executed with budget tracking. Supports parallel execution of independent nodes (wave scheduling) and hot graphs (resident, tick-driven workflows).

**Why User Impact = 4 (High):** Multi-step tasks currently execute as sequential agent turns. A DAG engine allows parallel execution of independent steps, reducing wall-clock time significantly. Users can also define custom workflows in TOML.

**Why Ease = 2 (Hard):** Approximately 2,500 lines. All cells must dispatch through `ToolDispatcher` per IronClaw's "everything through tools" principle. Dependencies on `toml` (already in deps) and `petgraph` (or custom topological sort).

**Dependencies:** None strictly required, but benefits from Gate Pipeline (#10) -- gate rungs are a natural DAG.

**Estimated time:** 4-8 weeks.

**Implementation location:** New `crates/ironclaw_graph/` crate.

---

### Rank 13: EventBus with Replay Ring (Composite: 3.25)

**What it is:** A publish-subscribe event system with a bounded circular buffer. Every event gets a monotonically increasing sequence number. New subscribers can catch up on recent history by replaying from a specific sequence number.

**Why User Impact = 2 (Low):** Users do not interact with the event bus directly. The benefit is in enabling other features: real-time web UI updates via SSE, background task coordination, structured observability.

**Why System Impact = 4 (High):** Replaces ad-hoc channels for inter-component communication with a single, consistent mechanism.

**Dependencies:** None.

**Estimated time:** 2-3 weeks.

**Implementation location:** New `src/events/` module or `crates/ironclaw_events/`.

---

### Rank 14: Conductor (Anomaly Detection & Circuit Breaking) (Composite: 3.25)

**What it is:** An ensemble of 10 "watchers" monitoring system health in real time: latency, error rate, token usage, cost, quality, throughput, saturation, availability, drift, and coherence. Predictive circuit breaking via Holt double exponential smoothing. Thompson Sampling learns optimal thresholds. IronClaw already has `CircuitBreakerProvider` in `crates/ironclaw_llm/src/circuit_breaker.rs` (reactive, 5-failure threshold). The Conductor replaces this with *predictive* circuit breaking.

**Why Ease = 2 (Hard):** Approximately 2,000+ lines. Stateful, runs continuously alongside the agent.

**Dependencies:** Benefits strongly from Cascade Router (#7) -- without a router, the Conductor can only trip circuit breakers, not gracefully route to alternatives.

**Estimated time:** 4-6 weeks.

**Implementation location:** New `crates/ironclaw_conductor/` crate or module in `crates/ironclaw_llm/`.

---

### Rank 15: Resumable Checkpoints (Composite: 3.20)

**What it is:** Serialize agent state at key points so that if the agent crashes mid-turn, it can resume from the last checkpoint rather than replaying the entire conversation.

**Why User Impact = 3 (Medium):** Crashes during long-running jobs are frustrating because they lose all progress. However, crashes are relatively rare in practice.

**Why Ease = 3 (Medium):** The challenge is in serialization: agent state includes in-memory caches, pending futures, and references to external resources. A practical approach checkpoints only conversation history and pending tool calls.

**Dependencies:** None.

**Estimated time:** 2-3 weeks.

**Implementation location:** `src/agent/checkpoint.rs` (new file), `src/db/` (migration).

---

### Rank 16: Declarative TOML Tools (Composite: 3.15)

**What it is:** Define simple tools in TOML without writing Rust or WASM code. A TOML file specifies tool name, description, parameter schema, and execution method (HTTP call, shell command, or chained tool invocation).

**Why User Impact = 3 (Medium):** Lowers the barrier for custom tools. Currently requires WASM or MCP.

**Why Ease = 3 (Medium):** Approximately 400-500 lines.

**Dependencies:** None.

**Estimated time:** 2-3 weeks.

**Implementation location:** `src/tools/declarative/` (new module).

---

### Rank 17: User Engagement PAD Tracker (Composite: 3.00)

**What it is:** A lightweight Pleasure-Arousal-Dominance tracker that estimates the user's engagement state based on interaction patterns. Modulates response style: frustrated users get more careful, step-by-step responses; confident power users get concise, direct answers.

**Why User Impact = 3 (Medium):** Adaptive response style is nice but not critical. The benefit emerges only with extended use as the tracker accumulates enough data to classify engagement states reliably.

**Why Safety = 3 (Medium):** Misclassification (detecting frustration when the user is fine) could make responses worse. Mitigation: use PAD as a soft bias, not a hard switch.

**Dependencies:** None.

**Estimated time:** 2-3 weeks.

**Implementation location:** `src/profile.rs` (extend existing psychographic profile), `crates/ironclaw_engine/` (behavioral hints in prompts).

---

### Ranks 18-25: Long-Term Items

Brief justifications for the bottom third:

| Rank | Concept | Composite | Why It Ranks Lower |
|------|---------|-----------|-------------------|
| 18 | Full Dream Consolidation | 2.80 | Depends on Enhanced Heartbeat (#11) + HDC (#9); quality of creative insights from LLM is uncertain; high ongoing cost |
| 19 | Budget Composition (VCG) | 2.75 | High integration complexity with existing prompt composition in `crates/ironclaw_engine/`; progressive tool disclosure already partially implemented |
| 20 | Code Intelligence | 2.65 | Requires HDC (#9) + tree-sitter; 2,500+ lines; valuable only for users working on larger codebases |
| 21 | NEAR On-Chain Reputation | 2.30 | Smart contract development + auditing is expensive; depends on NEAR ecosystem adoption; start with off-chain reputation first |
| 22 | Pheromone System | 2.25 | Requires multi-agent support that does not yet exist in IronClaw; in single-agent context, just tagged memories with decay |
| 23 | Pure SM Extraction | 2.20 | Major refactor of the most complex code in IronClaw (`src/agent/agentic_loop.rs`). Holy grail of testability, but extremely risky and long. Incremental approach required. |
| 24 | Full Affect Engine | 2.10 | k-d tree somatic markers, 3-layer temporal tracking, Nietzsche vitality phases -- diminishing returns over the lightweight PAD tracker (#17); risk of anthropomorphization |
| 25 | TDA / Sheaves | 1.55 | Research-grade mathematics (persistent homology, cellular sheaf cohomology) with negligible practical benefit for a personal assistant |

---

## Quick Wins: What You Can Build This Week

These items can each be implemented in 1-2 days. They require no new crates, no complex integration, and deliver immediate value. A single developer could ship all of them in a week.

### Quick Win 1: Robust Statistics (Rank 4, ~1 day)

**What to do:**
1. Create four functions in `src/util.rs`: `median()`, `trimmed_mean()`, `mad()`, `hodges_lehmann()`.
2. Write unit tests with known-good values.
3. Find call sites using manual averaging in `src/estimation/` and `src/evaluation/`.
4. Replace with `trimmed_mean(values, 0.1)`.
5. Total: ~150 lines including tests.

**Why this is a quick win:** Pure functions. Zero integration risk. Immediate accuracy improvement in cost/time estimation.

### Quick Win 2: BLAKE3 Content Dedup (Rank 3, ~1 day)

**What to do:**
1. Add `content_hash BLOB` column to memory tables (one PostgreSQL + one libSQL migration).
2. In `memory_write` tool handler, compute `blake3::hash(content.as_bytes())`.
3. Before INSERT, query for existing entry with same hash.
4. If found: UPDATE tags, bump access_count, update timestamp. Return "merged with existing entry."
5. Total: ~100 lines including tests.

**Why this is a quick win:** BLAKE3 is already a dependency. The change is localized to one tool handler and one repository method.

### Quick Win 3: Ebbinghaus Decay (Rank 2, ~1-2 days)

**What to do:**
1. Add `decay_variant TEXT DEFAULT 'ebbinghaus'`, `stability REAL DEFAULT 3600.0`, `last_accessed TIMESTAMPTZ`, `access_count INTEGER DEFAULT 0` columns to memory tables.
2. Define `DecayVariant` enum with `current_strength(&self, now) -> f64`.
3. In `memory_search`, multiply each result's score by `decay.current_strength(now)`.
4. In `memory_read`, UPDATE stability and last_accessed.
5. Total: ~150 lines including migration and tests.

**Why this is a quick win:** Simple math (exponential decay formula), localized changes, backward-compatible (existing memories get `DecayVariant::None`).

### Quick Win 4: Composable Scorers (Rank 5, ~1-2 days)

**What to do:**
1. Define `trait Scorer: Send + Sync { fn score(&self, item: &dyn Scoreable) -> f64; }`.
2. Implement `WeightedScorer`, `ThresholdScorer`, `ChainScorer` combinators.
3. Implement concrete scorers: `RecencyScorer`, `RelevanceScorer`, `UtilityScorer`.
4. Write tests showing composition: `weighted(vec![(0.7, recency), (0.3, relevance)])`.
5. Total: ~200 lines including tests.

**Why this is a quick win:** Pure trait system. No integration required initially -- refactor existing ad-hoc scoring to use it incrementally.

### Quick Win 5: Hierarchical Cancellation (Rank 6, ~3-5 days)

**What to do:**
1. Add `tokio_util` dependency (likely already transitive).
2. Create `CancellationToken` trees at session/job/tool boundaries.
3. Replace ad-hoc cancellation channels with token checks.
4. Total: ~200-300 lines.

**Why this is a quick win:** Uses an existing, well-tested crate (`tokio_util::sync::CancellationToken`). Purely structural improvement with no behavior change when cancellation is not triggered.

---

## Synergy Analysis: Items That Multiply Each Other's Value

Some features are worth more in combination than the sum of their individual values. This section identifies the four strongest synergy clusters, their combined value proposition, and the optimal build order within each.

### Cluster A: The Self-Managing Memory Stack

```
BLAKE3 Dedup (#3)  +  Ebbinghaus Decay (#2)  +  HDC Similarity (#9)
       |                      |                        |
       v                      v                        v
   No duplicate          Unused memories          Third signal
   entries               fade naturally           in search ranking
       |                      |                        |
       +----------+-----------+------------------------+
                  |
                  v
        Memory system that self-curates:
        deduplicates on write, fades on non-use,
        searches with compositional similarity
```

**Why the combination is worth more than the sum:** These three features address three different failure modes (duplication, staleness, relevance) that compound. A system with duplicates AND stale entries AND poor search is much worse than one with only one problem because: (a) duplicates inflate the stale entry count, (b) stale entries make search results noisy, and (c) noisy search leads to poor context injection which causes the agent to write more redundant memories. The three fixes create a virtuous cycle instead of the current vicious one.

**Key synergy:** When BLAKE3 Dedup detects a duplicate, it bumps the existing entry's access count. If Ebbinghaus Decay is active, this access count bump strengthens the entry's stability -- meaning genuinely repeated knowledge becomes long-lived automatically. Without both features, you cannot distinguish "the user keeps mentioning this" (should strengthen) from "the agent keeps re-learning this" (should deduplicate).

**Build order:** BLAKE3 Dedup first (simplest, 1 day), then Ebbinghaus Decay (requires migration, 1-2 days), then HDC Similarity (most complex, 2-3 weeks). The first two should be built together so the dedup-on-access-bump synergy is immediately available.

### Cluster B: The Cost Optimization Stack

```
Robust Stats (#4)  +  Cascade Router (#7)  +  Cog Speed (#8)  +  Conductor (#14)
      |                     |                      |                   |
      v                     v                      v                   v
   Stable reward        Learn which           Classify task       Monitor provider
   signals for          model fits            complexity          health
   the bandit           each request          automatically       in real-time
      |                     |                      |                   |
      +----------+----------+---------------------+-------------------+
                 |
                 v
        Intelligent model routing:
        stable metrics (robust stats),
        learned patterns (bandit),
        automatic classification (speeds),
        failure avoidance (conductor)
```

**Why the combination is worth more than the sum:** The Cascade Router's learning speed depends on the quality of its reward signals. Robust Statistics prevents outlier LLM responses (30-second timeouts, malformed outputs) from poisoning the bandit's model. Without robust stats, the router may take 2-3x longer to converge because it is learning from noisy signals.

Cognitive Speed Labels give the router better features -- classifying "this is a Gamma/reactive request" lets the bandit specialize its model per speed tier instead of learning a single global policy. With speed labels, the bandit effectively has 3 separate learning problems (one per speed) instead of one noisy global one, leading to faster convergence and better routing decisions.

The Conductor provides runtime health information that the router can use to avoid degraded providers. Without it, the router may learn that Provider A is best for Gamma requests, but if Provider A is degraded *right now*, the router will still route to it and get poor results. The Conductor's real-time health signal lets the router make context-aware decisions.

**Projected combined savings:** The Cascade Router alone projects 30-50% cost reduction. Adding Cognitive Speed Labels projects an additional 5-10% improvement in routing accuracy. Adding the Conductor prevents the 5-15% waste that occurs when the router sends requests to degraded providers. Combined: 35-60% cost reduction, roughly $35-60/month savings for a user spending $100/month on LLM calls.

**Build order:** Robust Statistics first (1 day, foundation for reward signals), then Cascade Router (2-3 weeks, core routing), then Cognitive Speed Labels (3-5 days, classification for the router), then Conductor (4-6 weeks, health monitoring that feeds into routing decisions).

### Cluster C: The Verification Stack

```
Composable Scorers (#5)  +  Gate Pipeline (#10)  +  DAG Engine (#12)
        |                         |                        |
        v                         v                        v
   Configurable            Progressive              Parallel rung
   pass/fail               verification             execution
   criteria                for code
        |                         |                        |
        +----------+--------------+------------------------+
                   |
                   v
         Code quality pipeline where:
         verification rungs run as DAG nodes,
         pass/fail is determined by composable scorers,
         and independent rungs execute in parallel
```

**Why the combination is worth more than the sum:** Gate Pipeline rungs are a natural DAG (lint and test are independent of each other; both depend on compile succeeding). Running them sequentially wastes time -- lint + test could run in parallel while compile runs first. Expressing rungs as DAG cells enables this parallelism, reducing wall-clock verification time by 30-50%.

Composable Scorers let users configure what "passing" means without code changes. A project might require "compile + lint must pass; tests optional but scored." Another might require "all four rungs pass." Without composable scorers, this configurability requires custom code per project.

**Build order:** Composable Scorers first (1-2 days, simplest foundation), then Gate Pipeline (2-4 weeks, uses scorers for rung evaluation), then DAG Execution (4-8 weeks, enables parallel rung execution). The first two can be built without the DAG engine -- the gate pipeline just runs rungs sequentially. The DAG engine is an optimization applied later.

### Cluster D: The Proactive Learning Stack

```
Ebbinghaus Decay (#2)  +  Enhanced Heartbeat (#11)  +  Full Dreams (#18)
         |                          |                         |
         v                          v                         v
   Memories strengthen       Background review        Creative insight
   through replay            and rehearsal             generation
         |                          |                         |
         +-------------+-----------+-------------------------+
                       |
                       v
         Agent that learns from experience:
         strengthens patterns (decay),
         reviews recent work (heartbeat),
         generates novel connections (dreams)
```

**Why the combination is worth more than the sum:** Without Ebbinghaus Decay, the Enhanced Heartbeat's NREM Replay cannot strengthen memories -- it bumps access counts that have no effect on retrieval ranking. Without the heartbeat, there is no scheduling infrastructure for consolidation. Without dream consolidation, the agent can only strengthen existing knowledge, not generate new connections.

The learning stack has a gate structure: each layer only delivers value if the previous one works. Ebbinghaus Decay is the foundation (does it make memory self-curating?). If yes, Enhanced Heartbeat adds value (does replay actually improve future performance?). If yes, Full Dream Consolidation adds the creative layer (do generated insights have genuine value?). This gate structure means you should build incrementally and validate each layer before proceeding.

**Build order:** Ebbinghaus Decay first (1-2 days), then Enhanced Heartbeat (2-3 weeks, validate for 2-4 weeks of real use), then Full Dream Consolidation (4-6 weeks, only if Enhanced Heartbeat proves its value). Each step is a go/no-go checkpoint.

---

## Implementation Sketches: Top 5 Priorities

Concrete Rust code sketches for the five highest-priority items. These are meant to show architectural direction, not production-ready code.

### Sketch 1: Metacognitive Monitor

```rust
// src/agent/metacognitive.rs

use std::collections::{HashSet, VecDeque};
use std::time::Instant;

/// Detects agent pathologies: stuck loops, cost runaways, contradictions.
pub struct MetacognitiveMonitor {
    /// Sliding window of recent action fingerprints for loop detection.
    action_window: VecDeque<u64>,
    /// Maximum window size before oldest actions are evicted.
    window_size: usize,
    /// Threshold: if fewer than this fraction of actions are unique,
    /// the agent is likely stuck. Default: 0.25 (25% unique).
    uniqueness_threshold: f64,
    /// EWMA of cost-per-turn for trend detection.
    cost_ewma: f64,
    /// Smoothing factor for EWMA. Default: 0.3.
    cost_alpha: f64,
    /// Recent assertions for contradiction detection.
    recent_claims: VecDeque<(u64, bool)>, // (claim_hash, polarity)
    /// Whether monitoring is enabled (feature flag).
    enabled: bool,
}

/// Pathology detected by the monitor.
#[derive(Debug, Clone)]
pub enum Pathology {
    /// Agent is repeating the same actions.
    StuckLoop {
        unique_ratio: f64,
        window_size: usize,
    },
    /// Cost per turn is trending toward budget exhaustion.
    CostRunaway {
        projected_total: f64,
        budget_remaining: f64,
    },
    /// Agent asserted X and then not-X within recent turns.
    Contradiction {
        claim_summary: String,
    },
}

/// Recommended corrective action.
#[derive(Debug, Clone)]
pub enum Correction {
    /// Break the loop: inject a "you are repeating yourself" prompt.
    BreakLoop,
    /// Downgrade to a cheaper model to conserve budget.
    DowngradeModel,
    /// Flag the contradiction to the user.
    FlagContradiction(String),
    /// No action needed.
    None,
}

impl MetacognitiveMonitor {
    pub fn new(window_size: usize, uniqueness_threshold: f64) -> Self {
        Self {
            action_window: VecDeque::with_capacity(window_size),
            window_size,
            uniqueness_threshold,
            cost_ewma: 0.0,
            cost_alpha: 0.3,
            recent_claims: VecDeque::with_capacity(50),
            enabled: true,
        }
    }

    /// Observe an action (tool call name + key params hashed) and its cost.
    pub fn observe(&mut self, action_fingerprint: u64, cost_cents: f64) {
        if !self.enabled {
            return;
        }
        // Update action window
        if self.action_window.len() >= self.window_size {
            self.action_window.pop_front();
        }
        self.action_window.push_back(action_fingerprint);

        // Update cost EWMA
        self.cost_ewma =
            self.cost_alpha * cost_cents + (1.0 - self.cost_alpha) * self.cost_ewma;
    }

    /// Check for pathologies. Returns the most severe one, if any.
    pub fn diagnose(&self, budget_remaining: f64, turns_remaining: u32)
        -> Option<Pathology>
    {
        if !self.enabled || self.action_window.len() < 4 {
            return None;
        }

        // Loop detection: count unique actions in the window
        let unique: HashSet<&u64> = self.action_window.iter().collect();
        let unique_ratio = unique.len() as f64 / self.action_window.len() as f64;
        if unique_ratio < self.uniqueness_threshold {
            return Some(Pathology::StuckLoop {
                unique_ratio,
                window_size: self.action_window.len(),
            });
        }

        // Cost runaway: project remaining spend
        if turns_remaining > 0 {
            let projected = self.cost_ewma * turns_remaining as f64;
            if projected > budget_remaining {
                return Some(Pathology::CostRunaway {
                    projected_total: projected,
                    budget_remaining,
                });
            }
        }

        None
    }

    /// Recommend a corrective action for a detected pathology.
    pub fn recommend(&self, pathology: &Pathology) -> Correction {
        match pathology {
            Pathology::StuckLoop { .. } => Correction::BreakLoop,
            Pathology::CostRunaway { .. } => Correction::DowngradeModel,
            Pathology::Contradiction { ref claim_summary } => {
                Correction::FlagContradiction(claim_summary.clone())
            }
        }
    }
}
```

**Integration point** in `src/agent/agentic_loop.rs`:
```rust
// After each iteration in run_agentic_loop():
let fingerprint = hash_action(&tool_calls);
monitor.observe(fingerprint, turn_cost_cents);
if let Some(pathology) = monitor.diagnose(budget_remaining, max_turns - turn) {
    let correction = monitor.recommend(&pathology);
    match correction {
        Correction::BreakLoop => {
            // Inject a system message: "You appear to be repeating actions.
            // Try a different approach or ask the user for clarification."
        }
        Correction::DowngradeModel => {
            // Switch to cheap_llm for remaining turns
        }
        _ => {}
    }
}
```

### Sketch 2: Ebbinghaus Decay

```rust
// In src/workspace/mod.rs or src/workspace/decay.rs

use chrono::{DateTime, Utc};

/// Decay model for memory entries.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DecayVariant {
    /// No decay. Entry persists at full strength forever.
    None,
    /// Simple half-life decay. Strength halves every `half_life_secs`.
    HalfLife { half_life_secs: f64 },
    /// Time-to-live. Entry is dead after `ttl_secs`.
    Ttl { ttl_secs: f64 },
    /// Ebbinghaus forgetting curve with spaced-repetition strengthening.
    /// Each access doubles `stability`, making the memory decay more slowly.
    Ebbinghaus { stability_secs: f64 },
}

impl DecayVariant {
    /// Compute current retrieval strength in [0.0, 1.0].
    ///
    /// For Ebbinghaus: strength = e^(-elapsed / stability)
    /// Each access doubles stability, so frequently-accessed memories
    /// become effectively permanent.
    pub fn current_strength(
        &self,
        last_accessed: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> f64 {
        let elapsed = (now - last_accessed).num_seconds().max(0) as f64;
        match self {
            DecayVariant::None => 1.0,
            DecayVariant::HalfLife { half_life_secs } => {
                0.5_f64.powf(elapsed / half_life_secs)
            }
            DecayVariant::Ttl { ttl_secs } => {
                if elapsed >= *ttl_secs { 0.0 } else { 1.0 }
            }
            DecayVariant::Ebbinghaus { stability_secs } => {
                (-elapsed / stability_secs).exp()
            }
        }
    }

    /// Default decay for new memory entries.
    pub fn default_ebbinghaus() -> Self {
        // 1-hour initial stability. A memory accessed once per day
        // reaches ~30-day stability within a week (2^7 * 3600 = ~128 hours).
        DecayVariant::Ebbinghaus {
            stability_secs: 3600.0,
        }
    }
}

/// Call this when a memory is accessed (read or returned in search results).
/// Doubles stability, resets last_accessed, increments access_count.
pub fn on_memory_access(
    stability: f64,
    _access_count: u32,
) -> (f64, DateTime<Utc>) {
    (stability * 2.0, Utc::now())
}
```

**Integration in `memory_search`:**
```rust
// In the search result ranking phase:
let now = Utc::now();
for result in &mut search_results {
    let decay = result.decay_variant();
    let strength = decay.current_strength(result.last_accessed, now);
    result.score *= strength; // Decay-weighted relevance
}
// Re-sort by adjusted score
search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
```

### Sketch 3: BLAKE3 Content Deduplication

```rust
// In src/tools/builtin/memory.rs, within the memory_write handler:

use blake3;

/// Check for content duplication before writing a memory entry.
/// Returns Some(existing_id) if a duplicate was found and merged,
/// None if the content is new and should be inserted.
async fn dedup_or_insert(
    workspace: &Workspace,
    path: &str,
    content: &str,
    tags: &[String],
) -> Result<DedupResult, ToolError> {
    let content_hash = blake3::hash(content.as_bytes());
    let hash_bytes = content_hash.as_bytes();

    // Check for existing entry with same content hash
    if let Some(existing) = workspace.find_by_content_hash(hash_bytes).await? {
        // Merge: update tags, bump access count, update timestamp
        workspace.merge_memory_entry(
            &existing.id,
            tags,          // merge new tags with existing
            Utc::now(),    // update last_modified
        ).await?;

        Ok(DedupResult::Merged {
            existing_id: existing.id,
            existing_path: existing.path,
        })
    } else {
        // New content: insert with the computed hash
        let id = workspace.write_with_hash(path, content, tags, hash_bytes).await?;
        Ok(DedupResult::Inserted { id })
    }
}

enum DedupResult {
    Merged { existing_id: String, existing_path: String },
    Inserted { id: String },
}
```

### Sketch 4: Robust Statistics

```rust
// In src/util.rs (append to existing file):

/// Median of a slice. Returns NaN for empty input.
pub fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

/// Trimmed mean: discard the top and bottom `trim_fraction` of values
/// before computing the mean. `trim_fraction` should be in [0.0, 0.5).
/// A trim_fraction of 0.1 discards the top and bottom 10%.
pub fn trimmed_mean(values: &[f64], trim_fraction: f64) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let trim_count = (sorted.len() as f64 * trim_fraction).floor() as usize;
    let trimmed = &sorted[trim_count..sorted.len() - trim_count];
    if trimmed.is_empty() {
        return median(values); // Fallback if over-trimmed
    }
    trimmed.iter().sum::<f64>() / trimmed.len() as f64
}

/// Median Absolute Deviation -- a robust measure of spread.
/// MAD = median(|x_i - median(x)|).
pub fn mad(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    let med = median(values);
    let deviations: Vec<f64> = values.iter().map(|x| (x - med).abs()).collect();
    median(&deviations)
}

/// Hodges-Lehmann estimator: median of all pairwise averages.
/// More efficient than trimmed mean for symmetric distributions.
/// O(n^2) -- suitable for small to medium datasets (<10,000 values).
pub fn hodges_lehmann(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    let n = values.len();
    let mut pairwise_avgs = Vec::with_capacity(n * (n + 1) / 2);
    for i in 0..n {
        for j in i..n {
            pairwise_avgs.push((values[i] + values[j]) / 2.0);
        }
    }
    median(&pairwise_avgs)
}
```

### Sketch 5: Composable Scorers

```rust
// src/evaluation/scorer.rs

use std::sync::Arc;

/// A scoreable item -- anything that can be assigned a relevance score.
pub trait Scoreable: Send + Sync {
    /// Timestamp of the item (for recency scoring).
    fn timestamp(&self) -> Option<chrono::DateTime<chrono::Utc>> { None }
    /// Text content (for relevance scoring).
    fn content(&self) -> Option<&str> { None }
    /// Numeric utility value (for utility scoring).
    fn utility(&self) -> Option<f64> { None }
    /// Access count (for popularity scoring).
    fn access_count(&self) -> Option<u32> { None }
}

/// A scorer produces a value in [0.0, 1.0] for any scoreable item.
pub trait Scorer: Send + Sync {
    fn score(&self, item: &dyn Scoreable) -> f64;
    fn name(&self) -> &str;
}

/// Weighted combination of multiple scorers.
pub struct WeightedScorer {
    scorers: Vec<(f64, Arc<dyn Scorer>)>,
}

impl WeightedScorer {
    pub fn new(scorers: Vec<(f64, Arc<dyn Scorer>)>) -> Self {
        Self { scorers }
    }
}

impl Scorer for WeightedScorer {
    fn score(&self, item: &dyn Scoreable) -> f64 {
        let total_weight: f64 = self.scorers.iter().map(|(w, _)| w).sum();
        if total_weight == 0.0 {
            return 0.0;
        }
        self.scorers
            .iter()
            .map(|(weight, scorer)| weight * scorer.score(item))
            .sum::<f64>()
            / total_weight
    }

    fn name(&self) -> &str {
        "weighted"
    }
}

/// Pass/fail gate: returns 1.0 if the inner scorer exceeds the threshold,
/// 0.0 otherwise.
pub struct ThresholdScorer {
    inner: Arc<dyn Scorer>,
    threshold: f64,
}

impl ThresholdScorer {
    pub fn new(inner: Arc<dyn Scorer>, threshold: f64) -> Self {
        Self { inner, threshold }
    }
}

impl Scorer for ThresholdScorer {
    fn score(&self, item: &dyn Scoreable) -> f64 {
        if self.inner.score(item) >= self.threshold {
            1.0
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "threshold"
    }
}

/// Recency scorer: items closer to `now` score higher.
/// Uses exponential decay: score = e^(-elapsed_hours / half_life_hours).
pub struct RecencyScorer {
    half_life_hours: f64,
}

impl RecencyScorer {
    pub fn new(half_life_hours: f64) -> Self {
        Self { half_life_hours }
    }
}

impl Scorer for RecencyScorer {
    fn score(&self, item: &dyn Scoreable) -> f64 {
        match item.timestamp() {
            Some(ts) => {
                let elapsed_hours =
                    (chrono::Utc::now() - ts).num_minutes() as f64 / 60.0;
                (-elapsed_hours / self.half_life_hours).exp()
            }
            None => 0.5, // Neutral score for items without timestamps
        }
    }

    fn name(&self) -> &str {
        "recency"
    }
}
```

---

## Expected Metrics: Before vs. After

These tables estimate the measurable impact of key features. "Before" reflects current IronClaw behavior. "After" reflects expected behavior. Estimates draw from roko's documented experience and general AI engineering benchmarks.

### Agent Reliability (Metacognitive Monitor, #1)

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Stuck loop rate (per 100 sessions) | 5-10 | 1-2 | 70-80% reduction |
| Avg turns wasted in stuck loops | 15-20 | 3-4 (detected and broken early) | 75-80% reduction |
| Cost wasted on stuck loops ($/month) | $5-15 | $1-3 | 70-80% reduction |
| Contradiction rate | Not tracked | Tracked and flagged | Visibility gain |

### LLM Cost (Cascade Router, #7)

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Avg cost per request | $0.02-0.05 | $0.01-0.03 | 30-50% reduction |
| % requests on expensive models | ~60-80% (SmartRouting is static) | 40-60% (bandit learns) | Significant |
| Monthly LLM spend ($100 baseline) | $100 | $50-70 | $30-50/month savings |
| Learning convergence | N/A | 200-500 requests per task type | Days of use |

### Memory Quality (Memory Stack, #2 + #3 + #9)

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Duplicate memory entries | Unbounded growth | Near-zero (BLAKE3 dedup) | Eliminated |
| Stale entries in search results | Increasing over time | Naturally decay (Ebbinghaus) | Self-managing |
| Search relevance (P@5) | Moderate (FTS + embeddings) | Improved (+ HDC third signal) | 10-20% est. |
| Memory storage growth rate | Linear (never cleaned) | Sub-linear (dedup + decay) | Reduced |

### Code Quality (Gate Pipeline, #10)

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Syntax errors in generated code | Caught by user | Caught by rung 1 (compile) | Pre-delivery |
| Lint violations | Caught by user | Caught by rung 2 (lint) | Pre-delivery |
| Test failures from generated code | Discovered after deploy | Caught by rung 3 (test) | Pre-delivery |

### Provider Reliability (Conductor, #15)

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Time to detect provider degradation | User reports slowness | Predicted via Holt smoothing | Proactive |
| Provider failover time | CircuitBreaker: 5 failures then 30s cooldown | Predictive: switch before failures accumulate | Faster |
| False alarm rate | N/A | Learned via Thompson Sampling | Adaptive |

---

## Decision Framework: Choosing What to Build Next

Use this flowchart when deciding which feature to work on next, given your current constraint.

```
START: What is your primary constraint?

  Cost pressure         -->  Robust Stats (#4) first,
  (need to reduce            then Cascade Router (#7),
   LLM spend)                then Cog Speed Labels (#8),
                              then Conductor (#14)

  Reliability issues    -->  Metacognitive Monitor (#1),
  (stuck loops,              then Hierarchical
   crashes, errors)          Cancellation (#6)

  Memory quality        -->  BLAKE3 Dedup (#3) + Ebbinghaus Decay (#2)
  (stale, duplicates,        together (synergy), then
   poor search)              HDC Similarity (#9)

  Code generation       -->  Composable Scorers (#5) first,
  quality                    then Gate Pipeline (#10)
  (bad code output)          (rungs 1-2, then 3-4)

  Workflow automation   -->  DAG Execution (#12)
  (multi-step tasks)

  Engineering quality   -->  Composable Scorers (#5) +
  (better tests,             Robust Statistics (#4) as
   cleaner code)             foundations

  None / general        -->  Follow the ranked list:
  improvement                #1 through #5 are all
                             1-2 day implementations
```

### Time Horizon Selection

| If you have... | Build these | Expected outcome |
|---------------|-------------|-----------------|
| 1 day | Robust Statistics (#4) or BLAKE3 Dedup (#3) | Stable metrics or no more duplicate memories |
| 2-3 days | + Ebbinghaus Decay (#2) + Composable Scorers (#5) | Self-managing memory, reusable scoring |
| 1 week | + Metacognitive Monitor (#1) + Hierarchical Cancellation (#6) | Reliable agent loop, clean shutdown |
| 2-4 weeks | + Cascade Router (#7) + Cognitive Speed Labels (#8) | 30-50% cost savings |
| 1-2 months | + HDC Similarity (#9) + Gate Pipeline (#10) | Better search + verified code gen |
| 2-3 months | + Enhanced Heartbeat (#11) + Conductor (#14) | Learning agent + resilient providers |
| 3+ months | + DAG Execution (#12) + Full Dream Consolidation (#18) | Workflow automation + proactive insights |

---

## Moonshots: High-Risk, High-Reward Items

These items have uncertain feasibility or value but would be transformative if they work. They are research projects, not engineering tasks.

### Moonshot 1: Full Dream Consolidation (Rank 18)

**Potential reward:** An agent that proactively generates insights by connecting knowledge across domains. "I noticed a pattern in your deployments -- the failures cluster around 3-5 PM UTC, which correlates with the European traffic peak you mentioned last month."

**Why risky:** Signal-to-noise ratio for LLM-generated "insights" is likely very low. High ongoing cost for background LLM calls. No ground truth for "is this insight useful?"

**Prerequisite trail:** Ebbinghaus Decay (#2) -> Enhanced Heartbeat (#11) -> HDC Similarity (#9) -> Full Dream Consolidation (#18).

**Gate strategy:** Build Enhanced Heartbeat first and run it for 2-4 weeks. If NREM Replay and Threat Rehearsal produce useful results, extend to REM Imagination. Each step is a go/no-go checkpoint.

### Moonshot 2: Pure State Machine Extraction (Rank 25)

**Potential reward:** A fully testable, replayable agent loop. Any bug can be reproduced by feeding recorded events through the state machine.

**Why risky:** Refactoring `src/agent/agentic_loop.rs` (the most complex code in IronClaw) while maintaining backward compatibility is extremely risky. Must be done incrementally.

**Gate strategy:** Extract one effect at a time, starting with the simplest (e.g., `Effect::Emit`). Validate that the pure state machine produces the same effects as the original for every test case. Only proceed when fully validated.

### Moonshot 3: NEAR On-Chain Reputation (Rank 21)

**Potential reward:** Verifiable, portable agent identity and reputation. Agents can prove their track record when interacting with other agents or marketplace participants.

**Why risky:** Smart contract development and auditing is expensive. Token economics add complexity. Requires ecosystem adoption.

**Gate strategy:** Start with off-chain reputation tracking (local only). Build the 7-domain EMA scoring engine. Once validated and stable, deploy to NEAR.

---

## Dependency Graph

Solid arrows (-->) are hard dependencies (must be built first). Dashed arrows (- ->) are soft dependencies (the later feature benefits from but does not require the earlier one).

```
                    PHASE 1 (Quick Wins, Week 1)
                    ~~~~~~~~~~~~~~~~~~~~~~~~~~~~
  Robust Stats (#4)    BLAKE3 Dedup (#3)    Composable Scorers (#5)
       :                     |                       :
       : (reward signals)   | (content hashing)     : (rung evaluation)
       :                     |                       :
       v                     v                       v
  Cascade Router (#7)  Ebbinghaus Decay (#2)   Gate Pipeline (#10)
       :                     |                       :
       : (routing)          | (memory decay)        : (parallel rungs)
       :                     |                       :
       +------+------+      +------+                 v
       :      :      :      |      :           DAG Execution (#12)
       v      :      v      v      :
  Conductor   :  Cog Speed  Enhanced   :
  (#15)       :  Labels(#8) Heartbeat  :
       :      :      :      (#11)      :
       :      :      :         |       :
       :      +------+        |       :
       :             :         v       :
       :             :    Full Dream   :
       :             :    Consol(#19)  :
       :             :         ^       :
       :             :         |       :
       :             :         +-------+
       :             :
       :             v
       +-------> Full Cost-Optimized IronClaw <----- HDC Similarity (#9)
                                                           :
                                                           v
                                                     Code Intelligence (#20)

  Legend:  | = hard dependency     : = soft dependency (benefits from)
```

---

## Implementation Location Reference

| Rank | Concept | IronClaw Location | New Crate? | DB Migration? | Est. LOC |
|------|---------|------------------|------------|---------------|----------|
| 1 | Metacognitive Monitor | `src/agent/metacognitive.rs` | No | No | 400-500 |
| 2 | Ebbinghaus Decay | `src/workspace/` | No | Yes (both) | 150 |
| 3 | BLAKE3 Dedup | `src/tools/builtin/memory.rs` | No | Yes (both) | 100 |
| 4 | Robust Statistics | `src/util.rs` | No | No | 100-200 |
| 5 | Composable Scorers | `src/evaluation/scorer.rs` | No | No | 200 |
| 6 | Hier. Cancellation | `src/agent/`, `src/context/` | No | No | 200-300 |
| 7 | Cascade Router | `crates/ironclaw_llm/src/router/` | Optional | Yes (episodes) | 1,500 |
| 8 | Cognitive Speed Labels | `src/agent/cognitive_speed.rs` | No | No | 200-300 |
| 9 | HDC Similarity | `crates/ironclaw_hdc/` | Yes | Yes (fingerprints) | 800-1,200 |
| 10 | Gate Pipeline | `crates/ironclaw_gate/` | Yes | No | 1,000 |
| 11 | Enhanced Heartbeat | `src/agent/heartbeat.rs` ext. | No | Optional | 800 |
| 12 | DAG Execution | `crates/ironclaw_graph/` | Yes | No | 2,500 |
| 13 | EventBus | `src/events/` | Optional | No | 400-500 |
| 14 | Conductor | `crates/ironclaw_conductor/` | Yes | No | 2,000+ |
| 15 | Resumable Checkpoints | `src/agent/checkpoint.rs` | No | Yes | 300-400 |
| 16 | Decl. TOML Tools | `src/tools/declarative/` | No | No | 400-500 |
| 17 | User Engagement PAD | `src/profile.rs` | No | No | 300-400 |
| 18 | Full Dream Consol. | `crates/ironclaw_dreams/` | Yes | Yes | 1,500 |
| 19 | Budget Composition | `crates/ironclaw_engine/` ext. | No | No | 1,500 |
| 20 | Code Intelligence | `crates/ironclaw_index/` | Yes | Yes | 2,500+ |
| 21 | NEAR Reputation | `crates/ironclaw_reputation/` | Yes | Yes | 1,100-1,500 |
| 22 | Pheromone System | `src/workspace/pheromone.rs` | No | Optional | 400-500 |
| 23 | Pure SM Extraction | `src/agent/` refactor | No | No | 3,000+ |
| 24 | Full Affect Engine | `src/affect/` | No | No | 1,000-1,500 |
| 25 | TDA / Sheaves | `crates/ironclaw_math/` | Yes | No | 2,000+ |

---

## Estimated Total LOC by Tier

### Stars (Ranks 1-6) -- Build in Week 1-2

| # | Feature | Est. Lines | Time |
|---|---------|-----------|------|
| 1 | Metacognitive Monitor | 400-500 | 2-3 days |
| 2 | Ebbinghaus Decay | 150 + migration | 1-2 days |
| 3 | BLAKE3 Dedup | 100 + migration | 1 day |
| 4 | Robust Statistics | 100-200 | 1 day |
| 5 | Composable Scorers | 200 | 1-2 days |
| 6 | Hierarchical Cancellation | 200-300 | 3-5 days |
| | **Total** | **~1,150-1,450** | **~9-14 days** |

### Big Bets (Ranks 7-10) -- Build in Weeks 3-8

| # | Feature | Est. Lines | Time |
|---|---------|-----------|------|
| 7 | Cascade Router | 1,500 | 2-3 weeks |
| 8 | Cognitive Speed Labels | 200-300 | 3-5 days |
| 9 | HDC Similarity | 800-1,200 | 2-3 weeks |
| 10 | Gate Pipeline (1-4) | 1,000 | 2-4 weeks |
| | **Total** | **~3,500-4,000** | **~7-11 weeks** |

### Nice to Have (Ranks 11-17) -- As appropriate

| # | Feature | Est. Lines | Time |
|---|---------|-----------|------|
| 11-17 | (seven features) | ~7,200-8,200 | ~18-28 weeks |

### Long-Term (Ranks 18-25) -- Research phase

| # | Feature | Est. Lines | Time |
|---|---------|-----------|------|
| 18-25 | (eight features) | ~12,300-14,400 | ~35-55 weeks |

### Grand Total

| Tier | Lines | Time |
|------|-------|------|
| Stars | ~1,300 | 2 weeks |
| Big Bets | ~3,750 | 7-11 weeks |
| Deferred | ~6,800 | 15-25 weeks |
| Long-Term | ~12,250 | 30-50 weeks |
| **All 25** | **~24,100** | **~54-88 weeks** |

This is a multi-year backlog if built by a single developer. The Stars tier alone delivers the highest ROI and should be the focus of the first sprint. Big Bets are the next priority for significant capability improvements. Deferred and Long-Term items are built as needs arise or as prerequisites become available.

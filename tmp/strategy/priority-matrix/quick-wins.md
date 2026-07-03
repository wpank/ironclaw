# Quick Wins: First-PR Instructions for the Stars Tier

> **Navigation**:
> [README](README.md) |
> [Detailed Rankings](detailed-rankings.md) |
> [Implementation Sketches](implementation-sketches.md) |
> [Quick Wins (you are here)](quick-wins.md) |
> [Synergy Analysis](synergy-analysis.md) |
> [Benchmarking Plans](benchmarking-plans.md) |
> [References](references.md)

The five quick-win items, each designed to be reviewable and mergeable in under an hour of
review time. Each PR is deliberately scoped to the smallest change that delivers value —
avoid expanding scope during the PR; follow-up PRs can do that.

**Recommended build order:** #4 → #3 → #2 → #5 → #1

---

## Quick Win 1: Robust Statistics (1 day, Rank 4)

**PR title:** `feat(util): add trimmed_mean, median, mad, hodges_lehmann`

**Why this first:** Robust statistics are used by Metacognitive Monitor (#1) for cost EWMA
tracking and by the Cascade Router (#7) for reward signal stabilization. Building this first
gives all subsequent features a clean foundation. It is also the fastest PR to write (~1 hour
for code + tests) and carries zero integration risk.

**What to do:**

1. Open `src/util.rs`.
2. Append the six functions from [implementation-sketches.md](implementation-sketches.md#rank-4-robust-statistics):
   - `median(values: &[f64]) -> f64`
   - `trimmed_mean(values: &[f64], trim_fraction: f64) -> f64`
   - `mad(values: &[f64]) -> f64`
   - `hodges_lehmann(values: &[f64]) -> f64`
   - `ewma_update(current_ewma: f64, new_value: f64, alpha: f64) -> f64`
   - `robust_z_score(value: f64, values: &[f64]) -> f64`
3. Add 8 unit tests (already written in the implementation sketch, copy from `#[cfg(test)]` block).
4. Find one call site in `src/estimation/learner.rs` that computes a mean and replace with
   `trimmed_mean(values, 0.1)`.
5. Run `cargo test` — all tests should pass.
6. PR is ~160 lines (functions + tests) touching 1–2 files.

**Review checklist:**
- [ ] All 8 unit tests pass: `cargo test util::`
- [ ] `trimmed_mean` panics on `trim_fraction >= 0.5` (correct: assert in function)
- [ ] `median` and `mad` return `f64::NAN` for empty input (not panic)
- [ ] `hodges_lehmann` matches `trimmed_mean` for symmetric data
- [ ] No new dependencies added to `Cargo.toml`

**Why this PR over a broader refactor:** Zero integration risk, no new dependencies, no
migrations, pure functions. Any reviewer can verify correctness by computing the expected
values by hand.

---

## Quick Win 2: BLAKE3 Content Dedup (1 day, Rank 3)

**PR title:** `feat(workspace): content deduplication via BLAKE3 hash on memory_write`

**Why this second:** Ships in one day, eliminates a compounding problem (duplicate entries
pollute every search forever), and sets up the synergy with Ebbinghaus Decay (#2). Build
these two back-to-back on the same day when possible.

**What to do:**

1. Create `src/workspace/dedup.rs` with the `content_hash()`, `normalize_content()`, and
   `DedupOutcome` code from [implementation-sketches.md](implementation-sketches.md#rank-3-blake3-content-dedup).
2. Add PostgreSQL migration: `ALTER TABLE workspace_entries ADD COLUMN content_hash BYTEA` +
   unique index. File: `migrations/20260101000002_memory_content_hash.sql`.
3. Add libSQL migration: same with `BLOB` type.
   File: `migrations/libsql/20260101000002_memory_content_hash.sql`.
4. In `src/workspace/repository.rs`, add two methods:
   - `find_entry_by_content_hash(hash: &[u8; 32]) -> Result<Option<WorkspaceEntry>, DbError>`
   - `merge_memory_entry(id: &str, tags: &[String], now: DateTime<Utc>) -> Result<(), DbError>`
5. In `src/tools/builtin/memory.rs`, modify `handle_memory_write` to:
   - Compute `let hash = content_hash(content);`
   - Call `workspace.find_entry_by_content_hash(&hash).await?`
   - On duplicate: call `merge_memory_entry` and return early with "merged" message
   - On new: call existing insert path with the hash stored
6. Add 5 unit tests: identical content, whitespace-normalized, different content, merge path, insert path.

**Review checklist:**
- [ ] Both migration files present (PostgreSQL and libSQL)
- [ ] `content_hash` is deterministic: same input always produces same output
- [ ] Whitespace normalization: `"hello  world"` and `"hello world"` produce the same hash
- [ ] Merge path bumps `access_count` and updates `last_modified`
- [ ] On a bug in hash comparison, the fallback is a regular INSERT (no data loss)
- [ ] No changes to read paths (`memory_search`, `memory_read`)
- [ ] `blake3` version in `Cargo.toml` is already present — no new deps added

**Why BLAKE3 over SHA-256:** BLAKE3 is already in `Cargo.toml`. It is also 10× faster than
SHA-256 for short inputs (memory content is typically 50–500 bytes). No additional dependency
or performance cost.

---

## Quick Win 3: Ebbinghaus Decay (1–2 days, Rank 2)

**PR title:** `feat(workspace): Ebbinghaus forgetting curve for memory entries`

**Why third:** Pairs naturally with BLAKE3 Dedup (both touch `workspace_entries`). When dedup
bumps `access_count`, Ebbinghaus Decay uses that count to strengthen stability — the two features
are more valuable together than apart. Ship them on consecutive days with a shared migration review.

**What to do:**

1. Create `src/workspace/decay.rs` with the full `DecayVariant` enum and `apply_decay_weights()`
   function from [implementation-sketches.md](implementation-sketches.md#rank-2-ebbinghaus-decay).
2. Add both database migrations (both columns have defaults for backward compatibility):
   - `decay_variant TEXT/JSONB NOT NULL DEFAULT '{"type":"none"}'`
   - `last_accessed TIMESTAMPTZ/TEXT NOT NULL DEFAULT NOW()`
   - `access_count INTEGER NOT NULL DEFAULT 0`
   - Index on `last_accessed`
3. In `src/workspace/repository.rs`:
   - Modify the search query to read `decay_variant`, `last_accessed` columns
   - Multiply each result's FTS/embedding score by `DecayVariant::current_strength()`
   - Filter entries where `current_strength() < 0.05`
4. In `src/tools/builtin/memory.rs`:
   - On memory read/access: call `decay.on_access(Utc::now())` and UPDATE the row's
     `decay_variant`, `last_accessed`, and `access_count`
5. Add 6 unit tests (already written in the implementation sketch):
   - `none_variant_is_always_full_strength`
   - `ebbinghaus_decays_to_threshold_at_stability`
   - `on_access_doubles_stability`
   - `on_access_caps_at_two_years`
   - `ttl_variant_expires_at_boundary`
   - `apply_decay_weights_filters_faded`

**Review checklist:**
- [ ] Existing entries use `DecayVariant::None` (always strength=1.0) — fully backward-compatible
- [ ] New entries get `DecayVariant::Ebbinghaus { stability_secs: 3600.0 }` (1-hour initial half-life)
- [ ] `on_access` caps stability at 63,072,000s (~2 years) to prevent overflow
- [ ] Entries with `current_strength() < 0.05` are filtered from search results (not deleted)
- [ ] Both migrations present (PostgreSQL and libSQL)
- [ ] No changes to `memory_write` — only the read path and the access-bump update

**Note on "filtered, not deleted":** The IronClaw principle is that LLM data is never deleted.
Faded entries remain in the database but score below the threshold. They can be recovered by
calling `memory_read` with the explicit path.

---

## Quick Win 4: Composable Scorers (1–2 days, Rank 5)

**PR title:** `feat(evaluation): composable scorer framework`

**Why fourth:** Provides the `Scorer` trait that the Gate Pipeline (#10) will use for
per-rung pass/fail criteria, and that the Cascade Router (#7) can use for reward signal
composition. Shipping it now means those Big Bet PRs can build on it directly.

**What to do:**

1. Create `src/evaluation/scorer.rs` with the full implementation from
   [implementation-sketches.md](implementation-sketches.md#rank-5-composable-scorers):
   - `Scoreable` trait
   - `Scorer` trait
   - `WeightedScorer`, `ThresholdScorer`, `ChainScorer`
   - `RecencyScorer`, `UtilityScorer`, `PopularityScorer`, `TagScorer`
2. Add `pub mod scorer;` to `src/evaluation/mod.rs`.
3. **Optional (can be a follow-up PR):** Find the keyword scorer in `src/skills/` and
   implement `Scorer` for it. This demonstrates value but is not required for the PR to merge.
4. Add 6 unit tests (already written in the implementation sketch):
   - `weighted_scorer_normalizes_weights`
   - `threshold_scorer_gates_at_threshold`
   - `chain_scorer_short_circuits_on_zero`
   - `recency_scorer_fresh_item_scores_high`
   - Plus 2 more for `PopularityScorer` and `TagScorer`

**Review checklist:**
- [ ] `Scorer` is object-safe (`&dyn Scorer` works)
- [ ] `WeightedScorer` normalizes weights (doesn't require them to sum to 1.0)
- [ ] `ChainScorer` short-circuits on the first stage that returns 0.0
- [ ] `RecencyScorer` returns 0.5 for items without timestamps (neutral, not zero)
- [ ] No changes to existing code paths — new file only
- [ ] `pub mod scorer;` added to `src/evaluation/mod.rs`

**The refactoring step is optional for the first PR.** The framework is valuable standalone
as infrastructure for future features. Refactoring existing scorers is a lower-priority cleanup
that can follow separately.

---

## Quick Win 5: Metacognitive Monitor (2–3 days, Rank 1)

**PR title:** `feat(agent): metacognitive monitor for stuck loop and cost runaway detection`

**Why last among quick wins:** This is the highest-value feature but requires touching the
agentic loop (`src/agent/agentic_loop.rs`) — the most sensitive file in the codebase. Building
it last in Week 1 means the team has warmed up with three simpler PRs. The monitor itself is
feature-flagged, so it can be merged into main and validated gradually.

**What to do:**

1. Create `src/agent/metacognitive.rs` with the full implementation from
   [implementation-sketches.md](implementation-sketches.md#rank-1-metacognitive-monitor).
2. Add `pub mod metacognitive;` to `src/agent/mod.rs`.
3. In `src/config/agent.rs`, add:
   ```rust
   pub metacognitive_monitor_enabled: bool, // default: true
   ```
   and read from env: `METACOGNITIVE_MONITOR_ENABLED=true`.
4. In `src/agent/agentic_loop.rs`, add the integration from the implementation sketch:
   - Instantiate `MetacognitiveMonitor` before the turn loop
   - Call `observe()` after each turn's tool calls
   - Call `diagnose()` after `observe()`
   - On pathology: inject correction (break loop, downgrade model, flag contradiction)
5. Add the 5 unit tests (already written in the implementation sketch).

**Review checklist:**
- [ ] Feature flag: when `METACOGNITIVE_MONITOR_ENABLED=false` (or unset default), zero code paths change
- [ ] Warm-up: no pathology detection before 4 turns (prevents false positives on session start)
- [ ] Stuck loop: triggers when `unique_ratio < 0.25` across the last 10 actions
- [ ] Cost runaway: triggers when `ewma_cost * turns_remaining > budget_remaining`
- [ ] Contradiction: triggers only when the same claim hash appears with opposite polarities
- [ ] Corrections use `debug!()` not `info!()` — no output to REPL/TUI
- [ ] `fingerprint_action` is deterministic for the same tool name + args
- [ ] No async code in the monitor itself (it is synchronous state updates only)

**Before/After validation (manual test):**
```
# With monitor enabled, trigger a stuck loop intentionally:
# Ask IronClaw to "ssh to a server that doesn't exist"
# Observe: after 4 turns of the same failing action, the monitor injects a correction
# Observe: the agent changes its approach
# Compare cost: <6 turns vs 18+ turns without monitor
```

---

## Week 1 Schedule

| Day | PR | Estimated Hours |
|-----|-----|----------------|
| Monday | Quick Win 1: Robust Statistics | 3–4 hours |
| Tuesday AM | Quick Win 2: BLAKE3 Dedup | 4–5 hours |
| Tuesday PM | Quick Win 3: Ebbinghaus Decay (start) | 2 hours |
| Wednesday | Quick Win 3: Ebbinghaus Decay (finish + review) | 3 hours |
| Thursday AM | Quick Win 4: Composable Scorers | 3 hours |
| Thursday PM | Quick Win 5: Metacognitive Monitor (start) | 3 hours |
| Friday | Quick Win 5: Metacognitive Monitor (finish, agentic loop integration, review) | 4 hours |

**Total: ~22–24 engineering hours for 5 high-value features.**

After Week 1, all Stars are complete. Begin Phase 2 (Big Bets) starting with Cascade Router (#7),
which depends on Robust Statistics (#4) and benefits from Cognitive Speed Labels (#8).

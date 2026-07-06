# Quick Wins

Use this file when choosing first PRs. "Quick" means narrow, reversible, and testable; it does not mean skipping subsystem docs or caller-level tests.

## Recommended First Sequence

| Step | PR | Why first | Rollback |
|------|----|-----------|----------|
| 1 | Robust statistics | Improves measurement and estimator stability with little surface area. | Revert helper use sites; helpers can remain unused. |
| 2 | Exact memory dedup | Reduces duplicate memory noise without fuzzy matching. | Disable merge path; stored hashes are inert metadata. |
| 3 | Memory decay | Builds on metadata and dedup; improves retrieval hygiene. | Stop applying decay; archived entries remain recoverable. |
| 4 | Metacognitive monitor | Reduces repeated failed turns after low-level metrics exist. | Disable intervention path or return to existing duplicate tracking. |
| 5 | Composable scorer interface | Prepares gate/evaluation work without changing behavior broadly. | Remove unused abstraction if no caller lands. |
| 6 | Cancellation propagation | Fixes resource leaks, but needs integration testing across process/tool boundaries. | Revert dispatcher token propagation. |

## PR 1: Robust Statistics

Scope:

- Add `median`, `trimmed_mean`, and `mad` helpers near existing estimation utilities.
- Use the helpers in one estimator path with a recent-sample window.
- Keep raw observations and add comments only where the outlier policy is non-obvious.

Tests:

- Empty, one-item, even-count, and odd-count helper cases.
- Estimator behavior with one extreme spike.
- Estimator behavior when the new baseline persists.

Do not:

- Replace unrelated averages.
- Add dependencies.

## PR 2: Exact Memory Dedup

Scope:

- Hash normalized memory content with existing BLAKE3 dependency.
- Store the hash in metadata unless current persistence requires a dedicated field.
- Merge exact duplicates in the memory write path.

Tests:

- Duplicate writes through the memory tool produce one visible entry.
- Merge response is user-visible.
- Different content remains separate.

Do not:

- Add fuzzy matching.
- Dedup across unrelated scopes unless the current memory model already permits that.

## PR 3: Memory Decay

Scope:

- Add decay metadata and strength calculation.
- Apply decay during ranking/filtering, not by deleting rows or files.
- Add an archival job only in an existing background-safe context.

Tests:

- Ordinary memory loses rank over test-controlled time.
- Access strengthens an entry.
- Identity/system documents never decay.
- Archived entries can be restored by metadata change.

Do not:

- Run archival during an active turn.
- Hide archived entries behind metadata that operators cannot reverse.

## PR 4: Metacognitive Monitor

Scope:

- Extend existing agent-loop stuck detection with recent signature diversity and projected spend.
- Start with observe-only metrics if thresholds are uncertain.
- Keep intervention decisions structured and easy to disable.

Tests:

- Alternating failing calls trigger detection.
- Diverse successful calls do not trigger detection.
- Spend projection trips only when budget exhaustion is plausible.
- Existing duplicate-call behavior still works.

Do not:

- Add a separate agent loop.
- Persist internal monitor traces into normal chat history by default.

## PR 5: Composable Scorers

Scope:

- Add the smallest trait and result type needed by one real caller.
- Put thresholds in the caller or configuration, not inside generic helpers.
- Include confidence and diagnostics only if the caller uses them.

Tests:

- Caller consumes a scorer result and changes behavior as expected.
- Weighted composition handles zero weight and missing confidence.

Do not:

- Build a broad evaluation framework without a consuming feature.

## PR 6: Hierarchical Cancellation

Scope:

- Thread cancellation through dispatcher and at least one subprocess-based tool.
- Make cancellation terminate child processes deliberately.
- Keep timeout and cleanup behavior observable in tests.

Tests:

- Start a long-running shell command through the real tool path.
- Cancel the parent session or turn.
- Assert the tool returns a cancellation result and the process is gone.

Do not:

- Stop at unit tests for token trees.
- Leave process cleanup to drop semantics.

## First-Pass Shape

For one developer, a realistic first pass is PR 1, PR 2, and the metadata-only
part of PR 3. PR 4 should wait until there is enough trace data or a
representative harness. PR 6 is a quick win only if the test harness for
subprocess cancellation already exists; otherwise treat it as a reliability
mini-project.

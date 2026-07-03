# Benchmarking Plans

Measure before enabling behavior changes. For low-risk pure logic, unit and
fixture tests may be enough. For routing, retrieval, gates, or autonomous
background work, use shadow mode or canaries before active rollout.

## Shared Measurement Rules

- Record a baseline from current IronClaw behavior before comparing a candidate.
- Prefer fixed fixtures for regression tests and production-like traces for rollout decisions.
- Use `debug!` or structured metrics for internal diagnostics; avoid noisy `info!` output in user-facing channels.
- Track both benefit and harm: quality, latency, cost, false positives, false negatives, and rollback events.
- Use caller-level tests when a helper gates a side effect.
- Treat any target below as a gate for rollout, not a promised result.

## Metrics By Candidate

| Candidate | Baseline | Primary metric | Guardrail | Minimum evidence before active/default use |
|-----------|----------|----------------|-----------|--------------------------------------------|
| Robust statistics | Current estimator error on clean and spiky traces | Lower error on spiky traces | No worse than current on clean traces | Deterministic fixture plus estimator caller test. |
| Exact memory dedup | Duplicate rate in fixture workspace | Duplicate writes merged | No false merge of different content | Memory tool test plus sampled real workspace dry run. |
| Memory decay | Current memory relevant@10 and stale-hit rate | Lower stale-hit rate | No identity/system archival | Search fixture with controlled clock plus restore test. |
| Metacognitive monitor | Existing duplicate-call detection | Earlier detection of repeated failed loops | False intervention rate below agreed threshold | Agent-loop harness over synthetic stuck and non-stuck traces. |
| Composable scorers | Current caller-specific scoring | Same or clearer caller decision | No behavior change without a consuming caller | Caller test showing the scorer result changes the intended side effect. |
| Cancellation propagation | Current cancel-to-stop time | Tool/process stops within bounded time | No orphan subprocesses | Integration test through real tool path. |
| Cascade router | Current model choice, cost, latency, quality | Lower cost/request at equal reviewed quality | Zero safety override violations | Shadow-mode episodes, then small canary. |
| Gate expansion | Current generated-code defect catch rate | More defects caught before completion | Low false-fail rate and no secret leakage | Hermetic projects with known failures, then canary on code-edit turns. |
| HDC signal | Current FTS/vector RRF relevant@10 | Relevant@10 lift on compositional queries | No regression on ordinary queries | Offline corpus benchmark before any active ranking. |
| Cognitive speed labels | Current unlabeled routing/context behavior | Improved caller decision where label is consumed | Classification does not force lower-capability model on risky tasks | Classifier fixture plus caller test. |

## Candidate Details

### Robust Statistics

Fixture:

- 50 normal observations around a stable mean.
- 1 to 3 extreme outliers.
- 30 observations after a genuine baseline shift.

Pass condition:

- Outliers have limited short-term impact.
- Persistent shift is eventually reflected.
- Clean-series error does not regress materially.

### Memory Dedup

Fixture:

- Exact duplicate content in the same scope.
- Same content in a different scope, if scopes are isolated.
- One-character content difference.

Pass condition:

- Exact duplicates merge only where scope rules allow.
- Different content remains separate.
- Tool output makes the merge auditable.

### Memory Decay

Fixture:

- Ordinary memories with different ages and access counts.
- Identity/system memories.
- Archived entries.

Pass condition:

- Strength calculation is deterministic under a test clock.
- Search ranking changes only where expected.
- Archived entries are recoverable.

### Metacognitive Monitor

Fixture:

- Consecutive duplicate failures.
- Alternating A/B failures.
- Diverse successful tool calls.
- A long but legitimate task.
- A projected spend runaway.

Pass condition:

- Stuck patterns are detected.
- Legitimate diversity is not interrupted.
- Interventions are explainable and disableable.

### Cascade Router

Baseline fields:

- Request class, selected model, latency, cost, retry/failure status, outcome label, and whether the request touched sensitive paths.

Shadow-mode gate:

- Candidate decisions are logged without affecting routing.
- Safety overrides are evaluated before learner output.
- No candidate route violates configured safety tier.

Canary gate:

- Quality pass rate is within the agreed tolerance of baseline.
- Cost or latency improves enough to justify complexity.
- Kill switch returns immediately to baseline routing.

### Gate Expansion

Fixture:

- Small valid Rust project.
- Rust project with compile error.
- Rust project with failing test.
- Project containing a fake secret in tool output.

Pass condition:

- Correct rung fails.
- Diagnostics are bounded and redacted.
- Commands use explicit args, not shell-interpolated strings.

### HDC Signal

Fixture:

- Memory corpus with ordinary keyword queries.
- Vector-semantic queries.
- Compositional queries such as "Rust async cancellation" where terms appear separately across documents.

Pass condition:

- HDC shows enough compositional relevant@10 lift to justify a third rank source.
- Ordinary query quality does not regress.
- Latency and memory overhead remain acceptable for local use.

## Rollout Pattern

| Risk level | Examples | Rollout |
|------------|----------|---------|
| Pure helper | Robust statistics helpers | Unit tests, caller test, direct merge. |
| Metadata-only | Dedup hash, decay metadata | Feature flag or inert metadata, fixture test, small real-data dry run. |
| Runtime behavior | Monitor, cancellation, gates | Observe-only if possible, integration tests, canary. |
| Model selection | Cascade router, cognitive speed labels | Shadow mode, safety audit, canary, automatic rollback. |
| Retrieval ranking | HDC, decay-in-search | Offline benchmark, disabled-by-default flag, canary on search. |

## Rollback Triggers

- Safety override violation: disable immediately.
- Secret appears in diagnostics: disable and treat as a security incident.
- Memory loss report: check archived metadata first, restore if needed, then disable decay archival.
- Quality drop beyond agreed tolerance: roll back active routing or ranking changes.
- Cancellation leaves child processes running: disable affected integration and fix before retry.

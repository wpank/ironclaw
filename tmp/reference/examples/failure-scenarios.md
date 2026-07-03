# Failure Scenarios

These are expected failure modes for the proposed features. Each one includes a
detection signal, mitigation, and a caller-level regression test idea.

## 1. False Positive HDC Memory Merge

Trigger:

- Two memories share vocabulary and HDC neighbors but mean different things.

Bad outcome:

- The assistant treats a distinct preference as a duplicate and hides it.

Detection signal:

- User correction after dedupe.
- Low lexical overlap despite high HDC similarity.
- Duplicate candidate later split manually.

Mitigation:

- Use soft dedupe first.
- Require exact hash match or high HDC similarity plus metadata agreement for
  hard dedupe.
- Keep lineage so split is possible.

Regression test:

- Use or extend [`memory-dedup.yaml`](../../implementation/benchmarking/scenarios/memory-dedup.yaml)
  with two similar but contradictory preferences.
- Drive the memory-write caller, not only the similarity helper.
- Assert hard dedupe does not merge them and search can still return both.

## 2. Cheap Model Selected Incorrectly

Trigger:

- Cascade router overgeneralizes from easy examples.

Bad outcome:

- Low-cost model answers a high-stakes or private request.

Detection signal:

- Static safety rule and bandit decision disagree.
- Quality pass rate drops by more than 2 percentage points.
- Fallback-to-primary rises.

Mitigation:

- Static safety rules always precede bandit choice.
- Missing telemetry falls back to current routing.
- High-risk/private requests bypass cheap-model exploration.

Regression test:

- Add a private-summary case to [`cascade-router.yaml`](../../implementation/benchmarking/scenarios/cascade-router.yaml).
- Drive the routing provider boundary.
- Assert bandit output is shadow-only and the trusted provider remains
  authoritative.

## 3. Gate Pipeline Blocks A Valid Change

Trigger:

- Rung selector chooses integration/security gate for a small low-risk change.

Bad outcome:

- User-visible work stalls despite correct code.

Detection signal:

- False block rate above 5%.
- Same remediation repeated twice.
- Blocked run passes when only the high-risk gate is disabled.

Mitigation:

- Disable only the high rung for that risk class.
- Keep compile/lint gates enabled.
- Add fixture for the valid change.

Regression test:

- Drive production caller, not only gate helper.
- Use [`gate-pipeline.yaml`](../../implementation/benchmarking/scenarios/gate-pipeline.yaml)
  as the initial fixture shape.
- Assert the low-risk change is allowed after compile/lint/unit tests.

## 4. Dream Writes Hallucinated Memory

Trigger:

- Background LLM summarizes a session incorrectly.

Bad outcome:

- Future retrieval uses a false derived memory.

Detection signal:

- Derived memory has no source turn support.
- User correction after retrieval.
- Contradiction with user-authored memory.

Mitigation:

- Tag all dream memories `Derived` and `LlmGenerated`.
- Keep low confidence until later successful reuse.
- Filter low-confidence derived memories from high-stakes contexts.

Regression test:

- Use or extend [`dream-consolidation.yaml`](../../implementation/benchmarking/scenarios/dream-consolidation.yaml)
  with an ambiguous session transcript.
- Assert derived memory remains low confidence and does not outrank source facts.

## 5. Conductor Oscillates Between Providers

Trigger:

- Health bias changes faster than provider performance stabilizes.

Bad outcome:

- Latency worsens and retries increase.

Detection signal:

- Provider selection alternates repeatedly within cooldown window.
- Fallback rate doubles.
- Healthy-provider false positive rate exceeds 3%.

Mitigation:

- Add cooldown and hysteresis.
- Require repeated confirmed forecasts before biasing routing.
- Roll back to observe mode.

Regression test:

- Extend [`provider-degradation.yaml`](../../implementation/benchmarking/scenarios/provider-degradation.yaml)
  with an alternating latency sequence.
- Assert at most one provider switch per cooldown window.

## 6. DAG Branch Skipped Unexpectedly

Trigger:

- Conditional edge predicate reads a missing or mis-typed field.

Bad outcome:

- Required verification or write step never runs.

Detection signal:

- `DagNodeSnapshot` remains `Skipped` with missing predicate evidence.
- Final run succeeds with required artifact absent.

Mitigation:

- Treat missing predicate inputs as blocked for required edges.
- Emit edge decision records.
- Validate graph before execution.

Regression test:

- Adaptation sketch: fixture graph with a typo in an edge predicate.
- Assert graph validation fails before effects run.

## 7. SSE Or WebSocket Reconnect Replays Incorrectly

Trigger:

- Browser reconnects after the server has emitted a terminal event.

Bad outcome:

- The dashboard misses completion or shows the same terminal event twice.

Detection signal:

- Event-stream gap count is non-zero.
- Duplicate terminal event count is non-zero.
- Browser state disagrees with the API's session state.

Mitigation:

- Require cursor-based replay in reconnect tests.
- Deduplicate terminal events by run id and event type.
- Keep HTTP, SSE, and WebSocket auth paths equivalent.

Regression test:

- Drive the real web stream handler with start/progress/end events.
- Disconnect before the end event is read, reconnect with the last cursor, and
  assert exactly one terminal event.

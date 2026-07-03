# Failure Scenarios

These are expected failure modes for the proposed features. Each one includes a
detection signal, mitigation, and regression test idea.

## 1. False Positive HDC Memory Merge

Trigger:

- Two memories share vocabulary and HDC neighbors but mean different things.

Bad outcome:

- The assistant treats a distinct preference as a duplicate and hides it.

Detection signal:

- user correction after dedupe.
- low lexical overlap despite high HDC similarity.
- duplicate candidate later split manually.

Mitigation:

- Use soft dedupe first.
- Require exact hash match or high HDC plus metadata agreement for hard dedupe.
- Keep lineage so split is possible.

Regression test:

- Fixture with two similar but contradictory preferences.
- Assert hard dedupe does not merge them.

## 2. Cheap Model Selected Incorrectly

Trigger:

- Cascade router overgeneralizes from easy examples.

Bad outcome:

- Low-cost model answers a high-stakes or private request.

Detection signal:

- static safety rule and bandit decision disagree.
- quality pass rate drops by more than 2 percentage points.
- fallback-to-primary rises.

Mitigation:

- Static safety rules always precede bandit choice.
- Missing telemetry falls back to current routing.
- High-risk/private requests bypass cheap-model exploration.

Regression test:

- Fixture with private summary request.
- Assert bandit is shadow-only and trusted provider remains authoritative.

## 3. Gate Pipeline Blocks A Valid Change

Trigger:

- Rung selector chooses integration/security gate for a small low-risk change.

Bad outcome:

- User-visible work stalls despite correct code.

Detection signal:

- false block rate above 5%.
- same remediation repeated twice.
- blocked run passes when gate is disabled.

Mitigation:

- Disable only the high rung for that risk class.
- Keep compile/lint gates enabled.
- Add fixture for the valid change.

Regression test:

- Drive production caller, not only gate helper.
- Assert the low-risk change is allowed after compile/lint/unit tests.

## 4. Dream Writes Hallucinated Memory

Trigger:

- Background LLM summarizes a session incorrectly.

Bad outcome:

- Future retrieval uses a false derived memory.

Detection signal:

- derived memory has no source turn support.
- user correction after retrieval.
- contradiction with user-authored memory.

Mitigation:

- Tag all dream memories `Derived` and `LlmGenerated`.
- Keep low confidence until later successful reuse.
- Filter low-confidence derived memories from high-stakes contexts.

Regression test:

- Fixture with ambiguous session transcript.
- Assert derived memory remains low confidence and does not outrank source facts.

## 5. Conductor Oscillates Between Providers

Trigger:

- Health bias changes faster than provider performance stabilizes.

Bad outcome:

- Latency worsens and retries increase.

Detection signal:

- provider selection alternates repeatedly within cooldown window.
- fallback rate doubles.
- healthy-provider false positive rate exceeds 3%.

Mitigation:

- Add cooldown and hysteresis.
- Require repeated confirmed forecasts before biasing routing.
- Roll back to observe mode.

Regression test:

- Alternating latency fixture.
- Assert at most one provider switch per cooldown window.

## 6. DAG Branch Skipped Unexpectedly

Trigger:

- Conditional edge predicate reads a missing or mis-typed field.

Bad outcome:

- Required verification or write step never runs.

Detection signal:

- `DagNodeSnapshot` remains `Skipped` with missing predicate evidence.
- final run succeeds with required artifact absent.

Mitigation:

- Treat missing predicate inputs as blocked for required edges.
- Emit edge decision records.
- Validate graph before execution.

Regression test:

- Fixture graph with typo in edge predicate.
- Assert graph validation fails before effects run.


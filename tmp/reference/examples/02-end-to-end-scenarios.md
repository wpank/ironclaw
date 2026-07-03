# End-To-End Scenarios

These scenarios show realistic IronClaw task paths. They avoid assuming any
private source repository exists: every path is either a local IronClaw module,
an existing fixture, or an explicit adaptation sketch.

## 1. Cheap Routine Question With Safe Model Routing

User asks: "Summarize my last three calendar reminders."

Relevant IronClaw touchpoints:

- `crates/ironclaw_llm` routing and provider selection.
- Channel normalization into an agent turn.
- Cost and quality metric events.

Path:

1. Channel normalizes input into an agent turn.
2. Static routing marks the request low risk but private.
3. Cascade router shadow-scores cheap models but the safety rule keeps the
   current trusted provider because private data is involved.
4. Metric event records the bypass reason.

Adaptation sketch: metric payload shape, not a runnable command or stable API.

```json
{
  "feature": "cascade_router",
  "variant": "linucb_shadow",
  "scenario": "private_summary",
  "fallback_used": true,
  "policy_violation": false
}
```

Benchmark fixture: [`tmp/implementation/benchmarking/scenarios/cascade-router.yaml`](../../implementation/benchmarking/scenarios/cascade-router.yaml)
covers the non-private cheap-routing case. Add a private-summary row before
using it as the guardrail test for this scenario.

Success criteria:

- Private requests never enter cheap-model canary routing.
- Shadow scoring records the bypass reason.
- Cost reduction is measured only on non-private requests.

## 2. Generated Code Change With Progressive Gates

User asks: "Add a webhook route that validates a signature header."

Relevant IronClaw touchpoints:

- `src/channels/web/` route and auth code.
- `src/tools/` or caller-owned gate execution.
- Caller-level remediation returned to the agent loop.

Path:

1. Agent edits route and auth modules.
2. Gate selector sees auth/webhook files and chooses compile, lint, unit tests,
   and security review.
3. Compile passes, lint passes, unit tests pass, security review fails because
   body limit coverage is missing.
4. Caller receives a remediation message and asks the agent to add the missing
   test before submitting.

Success criteria:

- The failing gate blocks through the real caller.
- Remediation names the missing body-limit test.
- Artifact output is redacted and bounded.

Benchmark fixture: [`tmp/implementation/benchmarking/scenarios/gate-pipeline.yaml`](../../implementation/benchmarking/scenarios/gate-pipeline.yaml).

## 3. Memory Dedup And Signal Lineage

User tells IronClaw the same preference twice with slightly different wording.

Relevant IronClaw touchpoints:

- `src/workspace/` memory write and search.
- Shared DB trait/backends for persisted memory metadata.
- Signal lineage and confidence metadata.

Path:

1. `memory_write` computes BLAKE3 content hash and HDC fingerprint.
2. Exact hash does not match, but HDC similarity is high.
3. In observe mode, IronClaw writes the new memory and records a duplicate
   candidate link.
4. After enough confidence, soft dedupe links the records under one Signal.

Adaptation sketch: metric names, not a runnable command.

```text
duplicate_candidate_rate
false_duplicate_rate
memory_search_top5_relevance
write_latency_distribution
```

Benchmark fixture: [`tmp/implementation/benchmarking/scenarios/memory-dedup.yaml`](../../implementation/benchmarking/scenarios/memory-dedup.yaml).

Success criteria:

- Search returns one coherent preference.
- Provenance still shows both source user statements.
- False duplicates stay below the fixture guardrail.

## 4. Provider Degradation During A Long Task

User asks for a large refactor plan.

Relevant IronClaw touchpoints:

- `crates/ironclaw_llm` provider wrapper and health events.
- Existing circuit breaker behavior.
- Router health bias in observe or canary mode.

Path:

1. Conductor observes rising latency and retry count for the selected model.
2. Holt forecast crosses warning threshold before the reactive breaker trips.
3. Router receives a health bias and chooses another eligible provider.
4. Metric events show avoided degraded-provider spend.

Adaptation sketch: rollback setting name; confirm the final config key before
implementation.

```text
experimental.provider_conductor.mode = "observe"
```

Benchmark fixture: [`tmp/implementation/benchmarking/scenarios/provider-degradation.yaml`](../../implementation/benchmarking/scenarios/provider-degradation.yaml).

Rollback trigger: healthy-provider false positives exceed the fixture guardrail
or the router oscillates more than once in the cooldown window.

## 5. Dream Consolidation After A Productive Session

User finishes a multi-turn debugging session where a rare dependency issue was
solved.

Relevant IronClaw touchpoints:

- `src/agent/` heartbeat/background runtime.
- `src/workspace/` memory writes for derived lessons.
- Cost guard and budget ledger.

Path:

1. Heartbeat selects the session because utility and surprise are high.
2. Dream job creates a derived summary memory with source turn links.
3. The memory is tainted `Derived` and `LlmGenerated`.
4. Later, a similar issue appears; retrieval uses the memory successfully.
5. Confidence is promoted after the successful reuse.

Benchmark fixture: [`tmp/implementation/benchmarking/scenarios/dream-consolidation.yaml`](../../implementation/benchmarking/scenarios/dream-consolidation.yaml).

Success criteria:

- Later retrieval hit rate improves against the baseline.
- Derived memories stay low confidence until reused.
- Background spend stays within the fixture cap.

## 6. Control Plane Event Projection

Operator opens the web gateway during a long-running task.

Relevant IronClaw touchpoints:

- `src/channels/web/` SSE/WebSocket routes.
- Event bus replay cursor.
- Projection state used by the browser UI.

Path:

1. Agent emits turn, tool, gate, cost, and provider-health events.
2. Control-plane projection builds a session view from the event bus.
3. Browser reconnects with a cursor after network interruption.
4. SSE resumes without duplicate terminal events.

Adaptation sketch: target metrics for integration tests; thresholds belong in
the fixture or rollout config.

```text
event_stream_gap_count = 0
duplicate_terminal_event_count = 0
projection_latency_guardrail = "fixture-defined"
auth_fail_open_count = 0
```

Regression test: open a stream, emit start/progress/end events, disconnect
before the end event is read, reconnect with the last cursor, and assert exactly
one terminal event.

## 7. Local Reputation For Tool Selection

Two equivalent tools can perform a task. One has a recent failure streak.

Relevant IronClaw touchpoints:

- `src/tools/` dispatcher and tool outcome records.
- Extension lifecycle and sandbox policy.
- Local trust/reputation scoring.

Path:

1. Tool outcomes emit local reputation events.
2. Reputation scorer applies half-life decay.
3. Tool selection prefers the reliable tool but keeps exploration budget for
   recovery.
4. Collusion checks are irrelevant locally, but signed evidence shape is kept so
   future external trust integrations can consume the same events.

Benchmark fixture: no fixture exists yet. Add one before enabling reputation to
affect production selection.

Success criteria:

- Failed-tool retry rate decreases.
- Tools can recover after the half-life decay period.
- Sandbox and approval decisions remain hard gates, independent of reputation.

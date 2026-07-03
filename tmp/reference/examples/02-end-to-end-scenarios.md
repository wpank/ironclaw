# End-To-End Scenarios

These scenarios show how the supplemental implementation, schema, benchmark, and
rollout docs fit together in realistic IronClaw workflows.

## 1. Cheap Routine Question With Safe Model Routing

User asks: "Summarize my last three calendar reminders."

Path:

1. Channel normalizes input into an agent turn.
2. Static routing marks the request low risk but private.
3. Cascade router shadow-scores cheap models but the safety rule keeps the
   current trusted provider because private data is involved.
4. Metric event records the bypass reason.

Expected metric:

```json
{
  "feature": "cascade_router",
  "variant": "linucb_shadow",
  "scenario": "private_summary",
  "fallback_used": true,
  "policy_violation": false
}
```

Success: routing learns without changing provider choice for private data.

## 2. Generated Code Change With Progressive Gates

User asks: "Add a webhook route that validates a signature header."

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

## 3. Memory Dedup And Signal Lineage

User tells IronClaw the same preference twice with slightly different wording.

Path:

1. `memory_write` computes BLAKE3 content hash and HDC fingerprint.
2. Exact hash does not match, but HDC similarity is high.
3. In observe mode, IronClaw writes the new memory and records a duplicate
   candidate link.
4. After enough confidence, soft dedupe links the records under one Signal.

Metrics:

```text
duplicate_candidate_rate
false_duplicate_rate
memory_search_top5_relevance
write_latency_p95
```

Success: search returns one coherent preference without losing provenance.

## 4. Provider Degradation During A Long Task

User asks for a large refactor plan.

Path:

1. Conductor observes rising p95 latency and retry count for the selected model.
2. Holt forecast crosses warning threshold before the reactive breaker trips.
3. Router receives a health bias and chooses another eligible provider.
4. Metric events show avoided degraded-provider spend.

Rollback trigger: if healthy providers are incorrectly penalized twice in a
canary window, set `experimental.provider_conductor.mode = "observe"`.

## 5. Dream Consolidation After A Productive Session

User finishes a multi-turn debugging session where a rare dependency issue was
solved.

Path:

1. Heartbeat selects the session because utility and surprise are high.
2. Dream job creates a derived summary memory with source turn links.
3. The memory is tainted `Derived` and `LlmGenerated`.
4. Later, a similar issue appears; retrieval uses the memory successfully.
5. Confidence is promoted after the successful reuse.

Success: later task completion improves without background spend exceeding the
daily cap.

## 6. Control Plane Event Projection

Operator opens the web gateway during a long-running task.

Path:

1. Agent emits turn, tool, gate, cost, and provider-health events.
2. Control-plane projection builds a session view from the event bus.
3. Browser reconnects with a cursor after network interruption.
4. SSE resumes without duplicate terminal events.

Metrics:

```text
event_stream_gap_count = 0
duplicate_terminal_event_count = 0
projection_latency_p95_ms < 250
auth_fail_open_count = 0
```

## 7. Local Reputation For Tool Selection

Two equivalent tools can perform a task. One has a recent failure streak.

Path:

1. Tool outcomes emit local reputation events.
2. Reputation scorer applies half-life decay.
3. Tool selection prefers the reliable tool but keeps exploration budget for
   recovery.
4. Collusion checks are irrelevant locally, but signed evidence shape is kept so
   future external trust integrations can consume the same events.

Success: failed-tool retry rate decreases without permanently blacklisting tools.


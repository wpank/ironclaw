# Operator Debugging Runbooks

Each runbook follows:

```text
symptom -> inspect -> likely cause -> recovery -> regression test
```

## 1. LLM Provider Degradation

Symptom:

- User-visible latency spikes.
- Fallback rate rises.
- Provider errors cluster within a short window.

Inspect:

- `provider_conductor` metric events.
- p95 latency by provider and request class.
- circuit breaker state.
- routing decisions from cascade router audit logs.

Likely causes:

- Provider outage or regional degradation.
- Prompt class shifted to a slower model.
- Conductor forecast threshold too sensitive.
- Circuit breaker cooldown too short, causing oscillation.

Recovery:

1. Set `experimental.provider_conductor.mode = "observe"`.
2. Keep the existing reactive circuit breaker enabled.
3. Pin affected request class to the known-good provider if needed.
4. Preserve health metrics for postmortem.

Regression test:

- Replay `benchmarking/scenarios/provider-degradation.yaml`.
- Assert routing bias changes only after forecast threshold and does not
  oscillate more than once.

## 2. Gate Failure Triage

Symptom:

- Valid-looking generated code is blocked.
- The agent loops on the same remediation.
- Gate artifacts are too large or missing.

Inspect:

- `GateVerdict` records by `run_id`.
- rung status and duration.
- remediation text.
- artifact refs and redaction status.
- caller-level event that consumed the verdict.

Likely causes:

- Rung selector chose too strict a rung for the change.
- Test fixture is stale.
- Artifact truncation removed the useful error.
- Remediation is not specific enough for the agent.

Recovery:

1. Keep compile/lint gates enabled.
2. Disable only the failing higher rung by risk tier.
3. Re-run with artifact retention in local-only mode.
4. Patch the fixture or remediation template.

Regression test:

- Drive the real code-generation caller with a known valid change.
- Assert false-block rate stays below 5%.

## 3. Memory Pollution Or Duplicate Recall

Symptom:

- Search returns several copies of the same fact.
- Assistant repeats stale preferences.
- Derived dream memories outrank user-authored facts.

Inspect:

- `SignalRecord` by `content_hash_blake3`.
- HDC near-duplicate candidates.
- taints and confidence.
- last accessed timestamps.
- memory search fused ranking.

Likely causes:

- Soft dedupe never promoted to canonical link.
- Decay weights are too weak.
- Derived memory confidence too high.
- RRF weighting overboosts HDC candidates.

Recovery:

1. Set `experimental.signal_records.dedupe_mode = "observe"`.
2. Filter low-confidence derived memories from retrieval.
3. Rebuild duplicate candidate links.
4. Review top offending memories and add regression fixtures.

Regression test:

- Replay `benchmarking/scenarios/memory-dedup.yaml`.
- Assert top-5 relevance stays above threshold with one canonical answer.

## 4. SSE/WebSocket Reconnect Debugging

Symptom:

- Dashboard misses task completion.
- Browser reconnect shows duplicate terminal events.
- Session state differs from CLI state.

Inspect:

- event cursor in client reconnect request.
- EventBus replay ring capacity.
- projection update latency.
- auth result for SSE/WS upgrade.
- terminal event idempotency key.

Likely causes:

- replay buffer too small.
- reconnect cursor not honored.
- projection consumes events out of order.
- auth path differs between HTTP and WebSocket.

Recovery:

1. Increase replay ring only if memory budget allows.
2. Require cursor-based replay in reconnect tests.
3. Deduplicate terminal events by run id and event type.
4. Re-check bearer auth and CORS/origin handling.

Regression test:

- Open stream, emit task start/progress/end, disconnect before end, reconnect
  with cursor, assert exactly one terminal event.

## 5. Background Heartbeat Spent Too Much Budget

Symptom:

- Daily cost guard triggers unexpectedly.
- Background tasks appear in usage reports.
- User did not receive useful new memories.

Inspect:

- heartbeat run ids.
- dream consolidation metric events.
- background budget ledger.
- memory writes with `Derived` and `LlmGenerated` taints.

Likely causes:

- candidate selection too broad.
- max memories per run too high.
- promotion criteria too loose.
- retry loop after provider failure.

Recovery:

1. Disable scheduled dream jobs.
2. Keep manual fixture runs available.
3. Lower daily background budget.
4. Filter low-confidence derived memories from retrieval.

Regression test:

- Replay a failing provider fixture and assert background spend stays below cap.


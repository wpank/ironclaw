# Feature Rollout Runbooks

Each runbook follows the same path:

```text
local fixture -> shadow -> canary -> limited default -> full default
```

Do not skip shadow mode for features that change model routing, memory writes,
tool execution, approvals, or persistence.

## 1. HDC Memory Search

Flag:

```toml
[experimental.hdc_memory_search]
enabled = false
mode = "off" # off | shadow | canary | default
max_candidates = 200
```

Steps:

1. Compute fingerprints on write but do not use them in ranking.
2. Record shadow candidate overlap with current FTS/vector search.
3. Enable HDC as a third RRF input for 5% of eligible searches.
4. Promote only if relevance improves and p95 search latency stays inside the
   configured budget.

Rollback:

```text
set mode = "off"
keep fingerprint column/table
ignore HDC candidates in ranking
```

Primary metric: top-5 relevance.
Guardrails: p95 latency, memory storage growth, duplicate-write false positives.

## 2. Cascade Router

Flag:

```toml
[experimental.cascade_router]
enabled = false
mode = "shadow" # off when disabled; shadow | canary | default after enablement
min_observations_per_arm = 100
quality_floor = 0.98
```

Steps:

1. Shadow-score candidate providers while current routing remains authoritative.
2. Log selected arm, static-rule reason, bandit score, cost estimate, and
   fallback reason.
3. Canary only low-risk, non-tool, non-secret-bearing requests.
4. Expand to broader traffic only after quality and latency guardrails pass.

Rollback:

```text
set enabled = false
discard in-memory bandit state
preserve audit events for postmortem
```

Primary metric: cost/request.
Guardrails: quality pass rate, p95 latency, fallback rate, safety bypass count.

## 3. Progressive Gates

Flag:

```toml
[experimental.progressive_gates]
enabled = false
max_rung_default = "unit_test"
artifact_limit_bytes = 262144
```

Steps:

1. Run gates in report-only mode for generated code changes.
2. Compare gate verdicts with existing test/CI outcomes.
3. Start blocking only on compile and lint failures.
4. Add higher rungs by risk tier after false-block rate is measured.

Rollback:

```text
set enabled = false
keep gate verdict history
do not delete artifacts until retention job expires them
```

Primary metric: escaped defect rate.
Guardrails: false block rate, artifact redaction failures, wall-clock overhead.

## 4. Dream Consolidation

Flag:

```toml
[experimental.dream_consolidation]
enabled = false
mode = "manual"
daily_budget_microusd = 25000
max_memories_written_per_run = 20
```

Steps:

1. Manual-only local runs on fixture conversations.
2. Shadow recommendations without writing memory.
3. Enable memory writes with `Derived` and `LlmGenerated` taints.
4. Promote memories only after later retrieval confirms utility.

Rollback:

```text
disable scheduled dream job
filter derived memories from retrieval if confidence < threshold
preserve source episode links
```

Primary metric: later retrieval usefulness.
Guardrails: background spend, sensitive-data leakage, low-confidence memory
pollution.

## 5. Provider Conductor

Flag:

```toml
[experimental.provider_conductor]
enabled = false
mode = "observe" # ignored while disabled; observe before active
forecast_horizon_seconds = 300
```

Steps:

1. Observe latency/error/cost only.
2. Emit health hints without affecting routing.
3. Bias routing away from degraded providers after repeated confirmed forecasts.
4. Integrate with the existing circuit breaker, not beside it.

Rollback:

```text
set mode = "observe"
ignore routing bias
preserve health metrics for analysis
```

Primary metric: degraded-provider spend avoided.
Guardrails: oscillation count, healthy-provider false positives, latency.

## 6. Signal Content Addressing

Flag:

```toml
[experimental.signal_records]
enabled = false
dedupe_mode = "observe" # observe | soft | hard
```

Steps:

1. Compute hashes and report duplicate candidates.
2. Soft dedupe by linking duplicates without blocking writes.
3. Hard dedupe only after false-positive review.

Rollback:

```text
set dedupe_mode = "observe"
keep duplicate links
allow all writes
```

Primary metric: duplicate storage reduction.
Guardrails: false dedupe, query latency, DB migration parity.

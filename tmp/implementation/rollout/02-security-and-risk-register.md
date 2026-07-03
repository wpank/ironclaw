# Security And Risk Register

Use this register for every experimental feature before canary.

## Review Matrix

| Area | Required check |
| --- | --- |
| Auth/routes | bearer auth, webhook auth, CORS/origin, body limits, rate limits unchanged |
| Secrets | no secret in logs, metrics, artifacts, memory, prompts, or events |
| Tools | approvals, sandboxing, allowlists, and denial paths unchanged |
| Persistence | PostgreSQL/libSQL parity, migration rollback, retention policy |
| Runtime | cancellation, retries, background budgets, fail-closed defaults |
| Privacy | prompts, file bodies, private paths, and raw source omitted or redacted |

## Immediate Rollback Triggers

- Policy, approval, sandbox, auth, origin, webhook, rate-limit, or body-limit
  regression.
- Any secret or private content leak in metric/event/artifact storage.
- Candidate p95 latency regression above 10% for two windows.
- Candidate quality regression below -2pp for one meaningful window.
- DB parity failure for data required by the feature.
- Feature flag off path does not return baseline behavior.

## Risk Records

```yaml
id: risk.progressive_gates.artifact_leak
feature: progressive_gates
severity: high
failure_mode: compiler or test output contains secret-like value
detection_metric: redaction_applied=false with sensitive pattern match
rollback_switch: experimental.progressive_gates.enabled=false
owner: verification/tool owner
```

Every high-risk feature needs at least one record covering its highest-impact
failure mode.

## DB Parity Checklist

- Shared DB trait updated first.
- PostgreSQL migration added.
- libSQL migration added.
- Dual-backend contract test added.
- Rollback behavior documented for persisted rows.
- Backfill excludes raw prompts, file bodies, secrets, and private paths.

## Data Minimization

Allowed by default: ids, hashes, bounded enums, timestamps, counts, redaction
status, latency, cost, and token counts.

Disallowed by default: raw prompts, full file bodies, private paths, secrets,
bearer tokens, arbitrary URLs, unredacted stack traces, and provider raw errors.

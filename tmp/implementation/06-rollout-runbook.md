# Mandatory Rollout Runbook

Use this runbook for every IronClaw implementation derived from the analysis
documents. A feature is not ready to leave local development until every field
below is explicit.

## 1. Feature Flag Inventory

```text
feature:
  key:
  owner:
  owning module:
  default: disabled
  config source: runtime config | DB-backed setting | compile feature
  scope: local | workspace | user | fleet
  dependencies:
  kill switch:
  removal criteria:
```

Rules:

- Default is disabled unless the feature is purely internal and no-op.
- Runtime flags must be readable without restart when the feature controls
  routing, background jobs, gates, or network behavior.
- Compile features are for dependency footprint only, not rollout control.

## 2. Staged Deployment

```text
stage 0 local fixture:
  required fixtures:
  pass criteria:

stage 1 shadow:
  decision authority: baseline
  candidate records metrics: yes
  minimum events:

stage 2 canary:
  cohort:
  traffic percentage:
  duration:
  stop conditions:

stage 3 limited default:
  eligibility:
  owner review cadence:

stage 4 full default:
  stable window:
  cleanup tasks:
```

Minimum stop conditions:

- policy violation count > 0
- secret leak count > 0
- quality pass rate delta < -2 percentage points
- p95 latency delta > +10%
- fallback rate delta > +5 percentage points

## 3. Rollback Procedure

```text
rollback:
  disable command/config:
  expected time to take effect:
  data left behind:
  compatibility behavior:
  rollback validation:
  postmortem artifact:
```

Rollback validation examples:

- Router: route a fixture request and verify baseline provider is selected.
- Gates: run a code-generation fixture and verify gate verdicts are report-only
  or absent.
- Dreams: trigger heartbeat and verify no background consolidation job starts.
- Signal dedupe: write duplicate fixture and verify write is allowed.

## 4. Security Review Checklist

Answer every item before canary:

- Auth changed? If yes, which route or channel?
- CORS/origin changed?
- Webhook validation changed?
- Body limits changed?
- Rate limits changed?
- Secret handling changed?
- Prompt/user content stored in telemetry?
- Tool execution path changed?
- Sandbox policy changed?
- Approval path changed?
- Network allowlist changed?

Any `yes` requires a named reviewer and a caller-level regression test.

## 5. DB Migration Parity

```text
db:
  trait methods:
  postgres migration:
  libsql migration:
  shared contract test:
  backfill:
  downgrade behavior:
  data retention:
```

Rules:

- Add DB trait method first.
- Implement PostgreSQL and libSQL together.
- Hide JSONB vs TEXT differences behind Rust structs.
- Prefer additive migrations before destructive cleanup.

## 6. Caller-Level Test Gate

```text
test:
  caller boundary:
  fixture:
  mocked external dependency:
  side effect protected:
  failure mode:
  assertion:
```

Caller boundary examples:

- `SmartRoutingProvider` for cascade routing.
- memory tool/facade for Signal and HDC behavior.
- code-generation caller for gates.
- web gateway route/SSE/WebSocket handler for control plane.
- DB trait contract for persistence.
- `ToolDispatcher` for tool execution.

Helper-only tests are insufficient when the helper gates a side effect.


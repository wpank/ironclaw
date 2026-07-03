# Security And Risk Register

Use this register for every implementation derived from the analysis set. It is
not a substitute for code review; it is a checklist for the failure modes that
these features are most likely to introduce.

## 1. Review Matrix

| Feature | Security-sensitive surface | Must not change without review |
|---|---|---|
| HDC memory search | memory retrieval, private workspace data | redaction, memory ACLs, identity prompt loading |
| Signal records | content hashes, lineage, taints | secret handling, sensitive-memory filters, DB retention |
| Cascade router | model/provider choice, cost, data residency | safety model routing, provider auth, secret-bearing prompts |
| Progressive gates | command execution, artifacts | sandboxing, approvals, artifact redaction, path allowlists |
| Dreams | background LLM calls, memory writes | budget guard, sensitive-data filters, user notification policy |
| Conductor | provider health and routing bias | circuit breaker policy, fallback safety, provider credentials |
| Control plane | HTTP/SSE/WebSocket surfaces | bearer auth, CORS/origin checks, body limits, rate limits |
| Reputation | actor scores and selection bias | signed evidence, collusion checks, appeal/override path |

## 2. Rollback Triggers

Immediate rollback:

- Any policy violation caused by an experimental feature.
- Any secret leak in a metric, artifact, memory, prompt, or event stream.
- Any auth, CORS, webhook, or bearer-token regression.
- Any database migration that fails on either PostgreSQL or libSQL.
- Any feature that blocks user-visible work above the configured false-block
  budget.

Canary rollback:

- Quality pass rate drops more than 2 percentage points.
- p95 latency increases more than 10% without an explicit exception.
- Fallback rate rises more than 5 percentage points.
- Cost increases when the feature's primary target was cost reduction.

## 3. Risk Records

Use this shape in PR descriptions or design docs:

```text
risk:
  feature:
  failure_mode:
  affected_boundary:
  detection_metric:
  rollback_switch:
  test_coverage:
  residual_risk:
```

Example:

```text
risk:
  feature: progressive_gates
  failure_mode: gate artifact includes a secret-bearing command output
  affected_boundary: code-generation verification
  detection_metric: redaction_applied=false with sensitive pattern match
  rollback_switch: experimental.progressive_gates.enabled=false
  test_coverage: fixture command output containing API key shape
  residual_risk: novel secret formats may require safety crate updates
```

## 4. DB Parity Checklist

Before enabling any persisted feature:

- The shared DB trait has one method per operation.
- PostgreSQL and libSQL implementations are both present.
- Contract tests run against both backends.
- JSON/JSONB differences are hidden behind Rust structs.
- Migrations are additive first.
- Rollback does not drop data.

## 5. Approval And Tool Execution Rule

Features may recommend actions; they must not bypass the existing dispatcher or
approval flow.

```text
correct:
  gate -> verdict -> caller -> ToolDispatcher/approval policy

incorrect:
  gate -> shell command directly
  dream job -> memory file write bypassing memory facade
  conductor -> provider credential mutation
```

## 6. Data Minimization

Telemetry should prefer identifiers and aggregates over raw content.

Allowed by default:

- feature key
- variant
- run id
- latency/cost/token counts
- gate status
- provider family label

Requires explicit review:

- full prompts
- tool arguments
- command output
- memory body
- user message body
- webhook payloads

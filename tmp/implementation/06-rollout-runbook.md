# Mandatory Rollout Runbook

Use this runbook before any experimental feature affects user-visible behavior.

## Feature Flag Entry

```yaml
feature: cascade_router
flag: experimental.cascade_router
default: off
owner: crates/ironclaw_llm
kill_switch: experimental.cascade_router.enabled=false
stage: local | shadow | canary | limited | default
```

Runtime flags must fail closed. Compile flags may control dependency footprint
only.

## Stages

| Stage | Behavior |
| --- | --- |
| `off` | baseline only, no candidate side effects |
| `local` | fixtures and hermetic tests |
| `shadow` | candidate records decisions, baseline acts |
| `canary` | candidate acts for small eligible cohort |
| `limited` | broader opted-in rollout |
| `default` | candidate becomes baseline, flag cleanup planned |

## Promotion Requirements

- Caller-level test passes through the real side-effect boundary.
- YAML scenario parses and benchmark report has a promote verdict.
- Feature exposure and metric events are emitted.
- PostgreSQL/libSQL parity is tested for new persistence.
- Security review covers auth, secrets, sandboxing, approvals, and network.
- Kill switch has been exercised.

## Rollback Procedure

1. Disable the runtime flag or emergency kill switch.
2. Verify baseline path at the caller boundary.
3. Preserve persisted rows unless retention policy says otherwise.
4. Mark derived/experimental data inert or hidden from ranking.
5. Record the rollback reason in rollout artifacts.

Rollback is mandatory on any policy violation, secret/private data leak, auth or
approval bypass, DB parity failure, or sustained p95 latency regression > 10%.

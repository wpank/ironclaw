# IronClaw Implementation Tracker

This directory is a self-contained implementation planning package for
experimental IronClaw features. It does not require external code or copied
research implementation. Each feature must land through IronClaw-owned
modules, runtime flags, caller-level tests, and bounded rollout metrics.

## Operating Rules

| Rule | Requirement |
| --- | --- |
| Ownership | Work in the module that owns the behavior |
| Flags | Experimental runtime flags default off |
| Tests | Test through the caller that triggers the side effect |
| Persistence | PostgreSQL and libSQL support ship together |
| Metrics | One target improves and all guardrails stay bounded |
| Rollback | Disabling the flag restores baseline behavior |
| Privacy | Metrics exclude raw prompts, secrets, file bodies, and private paths |

## Implementation Order

| Phase | Focus | Start criteria | Exit criteria |
| --- | --- | --- | --- |
| 1 | Low-risk local primitives | owner and flag known | unit tests plus caller smoke where side effects exist |
| 2 | Core runtime enhancements | Phase 1 support where needed | scenario fixture, caller-level test, rollback evidence |
| 3 | Runtime architecture evolution | Phase 2 baselines available | cross-layer contract tests and rollout metrics |
| 4 | Advanced/adaptive features | stable metrics and safety review | canary-ready evidence bundle |

## Priority Features

| Feature | Flag | Owner | Fixture |
| --- | --- | --- | --- |
| Signal records | `experimental.signal_records` | `src/workspace/`, `src/db/` | `benchmarking/scenarios/memory-dedup.yaml` |
| Cascade router | `experimental.cascade_router` | `crates/ironclaw_llm/` | `benchmarking/scenarios/cascade-router.yaml` |
| Progressive gates | `experimental.progressive_gates` | `src/tools/` | `benchmarking/scenarios/gate-pipeline.yaml` |
| Provider conductor | `experimental.provider_conductor` | `crates/ironclaw_llm/` | `benchmarking/scenarios/provider-degradation.yaml` |
| Dream consolidation | `experimental.dream_consolidation` | `src/agent/`, workspace | `benchmarking/scenarios/dream-consolidation.yaml` |
| Workspace code search | `experimental.workspace_code_search` | `src/workspace/` | `benchmarking/scenarios/workspace-code-search.yaml` |

## Definition Of Done

```text
feature has owner module
feature flag exists and defaults off
flag-off path proves baseline behavior
caller-level test covers the side effect
YAML scenario or benchmark captures baseline and candidate
guardrails are bounded
PostgreSQL/libSQL parity exists for new persistence
security review covers auth, secrets, sandboxing, approvals, and network
rollback steps are documented and exercised
docs/specs are updated when behavior changes
```

## File Map

| File | Use |
| --- | --- |
| `phase-1-checklist.md` | Small primitives and low-risk local changes |
| `phase-2-checklist.md` | Core enhancements with fixtures and rollback gates |
| `phase-3-4-checklist.md` | Architecture and advanced features |
| `01-rust-core-blueprints.md` | Rust primitive sketches |
| `02-runtime-workflow-blueprints.md` | DAG, gates, conductor, replay, cancellation |
| `03-ironclaw-integration-recipes.md` | Owner-bound implementation recipes |
| `04-reputation-contract-blueprints.md` | Local reputation and optional chain adapter |
| `05-per-file-action-matrix.md` | Feature-to-owner test and metric matrix |
| `06-rollout-runbook.md` | Mandatory rollout and rollback procedure |
| `07-implementation-readiness-contract.md` | Readiness table and evidence bundle |
| `08-caller-test-matrix.md` | Caller-level regression test targets |
| `09-plan-runner-readiness.md` | Automation readiness contract |
| `decision-log.md` | ADRs for flags, storage, metrics, and clean-room work |
| `schemas/` | Canonical event and persistence contracts |
| `rollout/` | Feature flags, risk register, threat models, runbooks |
| `benchmarking/` | Metric framework, runner contract, scenario fixtures |

## Review Checklist

- Run YAML parsing for `benchmarking/scenarios/*.yaml`.
- Validate scenario required fields and `flag_default: "off"`.
- Scan for stale external-source assumptions.
- Confirm benchmark metrics are bounded.
- Confirm changed behavior has matching docs, rollout, and tests.

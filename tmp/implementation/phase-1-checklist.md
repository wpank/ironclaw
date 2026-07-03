# Phase 1 Implementation Checklist

Phase 1 contains small, low-risk primitives that should be implemented behind
off-by-default flags when they can alter behavior.

## Shared Prerequisites

- Read the owning module docs before editing production code.
- Register the runtime flag in `rollout/03-feature-flag-inventory.md`.
- Add the narrowest unit tests and caller-level tests when a side effect exists.
- Keep persistence changes dual-backend.
- Record baseline behavior before changing ranking, routing, or retry policy.

## Items

| Item | Owner | Flag | First artifact | Required validation |
| --- | --- | --- | --- | --- |
| Robust statistics | estimation/telemetry utility | none unless caller behavior changes | median, p95, MAD helpers | unit tests on normal/outlier samples; aggregation p95 < 1ms |
| Metacognitive monitor | `src/agent/` | `experimental.metacognitive_monitor` | loop/retry detector | agent-turn fixture intervenes; approvals unchanged; flag off baseline |
| Memory decay | `src/workspace/` | `experimental.memory_decay` | decay metadata on memory rows | memory search/write caller test; no deletion on rollback |
| Content dedup | `src/workspace/`, `src/db/` | `experimental.signal_records` | exact hash + duplicate candidate | dual-backend write/search test; false duplicate <= 2% |
| Composable scorers | workspace/ranking utility | `experimental.composable_scorers` if behavior changes | scorer trait + weighted sum | caller ranking fixture; invalid weights rejected |
| Hierarchical cancellation | runtime/tool dispatcher | `experimental.hierarchical_cancellation` if gated | parent/child cancellation token | session -> tool cancellation stops real side effect |

## Per-Item Checklist

For each item:

```text
[ ] owner module identified
[ ] flag default off or explicitly unnecessary
[ ] schema/DB impact documented
[ ] unit tests cover local logic
[ ] caller-level test covers side effect
[ ] benchmark or before/after metric captured
[ ] rollback path verified
[ ] security/privacy impact checked
```

## Success Criteria

- No production `.unwrap()`/`.expect()` added outside justified invariants.
- Clippy remains clean for touched Rust.
- Feature flag off keeps baseline behavior.
- Metrics use bounded labels and do not store private content.

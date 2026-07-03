# Schemas And Data Models

This folder makes the transfer documents implementation-ready by defining the
shared data shapes that the proposed IronClaw features should use. The shapes
are intentionally IronClaw-native: they do not depend on any Roko package.

| File | Contents |
|---|---|
| `01-runtime-data-models.md` | Rust structs and JSON examples for metrics, gates, DAG runs, feature flags, reputation, and Signal records |
| `02-storage-and-migrations.md` | PostgreSQL/libSQL table sketches, DB trait additions, migration order, and compatibility rules |
| `03-correlation-and-ids.md` | Cross-feature ID conventions, exposure events, join paths, and cardinality limits |
| `04-canonical-event-and-persistence-contract.md` | Single canonical event, exposure, verdict, DAG, Signal, reputation, rollout, and persistence contract |

## Design Rule

Every experimental feature should emit the same durable identifiers at every
layer:

```text
run_id -> turn_id -> feature_key -> variant -> metric_event -> verdict
```

That chain is what makes benchmarking, rollout, audit, and rollback possible.

Use `04-canonical-event-and-persistence-contract.md` as the source of truth
when other documents show abbreviated examples.

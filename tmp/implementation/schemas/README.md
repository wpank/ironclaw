# Schemas And Data Models

These files define the shared data contracts for implementation plans, rollout
telemetry, benchmark fixtures, and persistence. Keep this directory authoritative
for schema shape; other docs should link here instead of restating fields.

| File | Purpose |
| --- | --- |
| `01-runtime-data-models.md` | Rust-facing structs, field rules, and JSON examples |
| `02-storage-and-migrations.md` | DB trait additions plus PostgreSQL/libSQL parity rules |
| `03-correlation-and-ids.md` | Stable ids used to join turns, features, metrics, and rollout decisions |
| `04-canonical-event-and-persistence-contract.md` | Canonical event tables, retention, and migration order |

## Contract Rules

- Every experimental feature has a stable `experimental.<feature>` flag and a
  matching `flag.<feature>` exposure id.
- Runtime behavior defaults off until a rollout document explicitly moves it.
- Metric labels stay bounded and low cardinality; raw prompts, secrets, file
  bodies, and private paths are stored only as redacted artifacts.
- PostgreSQL and libSQL changes ship together with one contract test that runs
  against both backends.
- Fixtures must parse as YAML and include the required scenario fields listed in
  `../benchmarking/scenarios/README.md`.

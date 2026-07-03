# Storage And Migration Sketches

New persistence behavior must support PostgreSQL and libSQL in the same branch.
Start with the shared DB trait, then implement both backends and run the same
contract test against each.

## DB Trait Additions

```rust
#[async_trait::async_trait]
pub trait ExperimentStore {
    async fn insert_metric_event(&self, event: MetricEvent) -> anyhow::Result<()>;
    async fn list_metric_events(
        &self,
        feature: &str,
        since_ms: i64,
        limit: u32,
    ) -> anyhow::Result<Vec<MetricEvent>>;

    async fn insert_feature_exposure(&self, event: FeatureExposureEvent) -> anyhow::Result<()>;
    async fn insert_gate_verdict(&self, verdict: GateVerdict) -> anyhow::Result<()>;
    async fn insert_signal_record(&self, record: SignalRecord) -> anyhow::Result<()>;
}
```

Only add methods that have a caller and a test. Avoid backend-specific behavior
in the trait surface.

## Canonical Tables

`metric_events` is the single stream for benchmark, rollout, and runtime
measurements. Do not create feature-specific metric tables.

Minimum columns:

| Table | Required columns |
| --- | --- |
| `metric_events` | `event_id`, `run_id`, `feature`, `variant`, `stage`, `timestamp_ms`, metric fields, `metadata_json` |
| `feature_exposure_events` | `exposure_id`, `run_id`, `feature_flag_id`, `feature_key`, `variant`, `stage`, `enabled`, `timestamp_ms` |
| `gate_verdicts` | `verdict_id`, `run_id`, `feature`, `rung`, `status`, `duration_ms`, `redaction_applied` |
| `signal_records` | `signal_id`, `content_hash`, `kind`, `confidence`, `taint_json`, `origin_ids_json`, `metadata_json` |

Use `TEXT` ids, integer timestamps, JSON/JSONB metadata, and indexes on
`feature + timestamp`, `run_id`, and any caller lookup key.

## Migration Order

1. Add shared types and DB trait methods.
2. Add PostgreSQL migration.
3. Add libSQL migration.
4. Add dual-backend contract tests.
5. Wire writes behind disabled feature flags.
6. Enable local/shadow collection only after rollback is documented.

## Contract Test Shape

```rust
#[tokio::test]
async fn metric_events_round_trip_on_all_backends() {
    for store in test_experiment_stores().await {
        let event = fixture_metric_event("contract.run.1");
        store.insert_metric_event(event.clone()).await.unwrap();
        let rows = store
            .list_metric_events(&event.feature, event.timestamp_ms - 1, 10)
            .await
            .unwrap();
        assert_eq!(rows, vec![event]);
    }
}
```

Tests may unwrap in test code. Production write paths should return errors with
context and must not panic on optional experimental telemetry.

## Backfill Rule

Do not backfill raw prompts, secrets, private paths, or file bodies. Backfills
may write ids, hashes, bounded metrics, and redacted artifact references only.

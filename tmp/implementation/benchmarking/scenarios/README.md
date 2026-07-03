# Scenario Fixtures

These manifests are examples of the concrete benchmark inputs expected by
[`../03-harness-code.md`](../03-harness-code.md). They are intentionally
implementation-neutral: an IronClaw benchmark runner can translate them into
Rust integration tests, local JSONL runs, or CI jobs.

| Fixture | Purpose |
|---|---|
| [`cascade-router.yaml`](cascade-router.yaml) | Compare static routing with LinUCB shadow/canary routing |
| [`memory-dedup.yaml`](memory-dedup.yaml) | Measure Signal/HDC duplicate-memory handling |
| [`gate-pipeline.yaml`](gate-pipeline.yaml) | Measure progressive gate defect catching and false blocks |
| [`provider-degradation.yaml`](provider-degradation.yaml) | Measure Conductor provider health routing |
| [`dream-consolidation.yaml`](dream-consolidation.yaml) | Measure background learning usefulness and budget impact |
| [`workspace-code-search.yaml`](workspace-code-search.yaml) | Measure symbol/HDC/RRF code search quality |

## Required Fields

```yaml
schema_version: 1
id: feature.scenario
feature: feature_key
feature_flag_id: flag.feature_key
owner: owning_ironclaw_module
stage: local
repetitions: 50
input: {}
mocked_dependencies: {}
baseline: {}
variant: {}
oracle: {}
expected_metrics: {}
guardrails: {}
artifacts: {}
```

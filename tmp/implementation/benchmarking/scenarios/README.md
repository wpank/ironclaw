# Scenario Fixtures

Scenario YAML files are executable benchmark manifests for `ironclaw-bench`.
They should be small, deterministic, and safe to run in local CI with mocked
services.

## Required Fields

Each fixture must include:

```yaml
schema_version: 1
id: feature_name.short_case
feature: feature_name
feature_flag_id: flag.feature_name
flag_default: "off"
owner: src/or/crate/path
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

## Field Rules

- `stage` is one of `local`, `shadow`, `canary`, `limited`, or `default`.
- `repetitions` is a positive integer with an explicit upper bound per fixture.
- `expected_metrics` contains target improvements only.
- `guardrails` contains bounded maximum regressions, rates, latency, cost, and
  privacy requirements.
- `artifacts` states what can be retained and whether raw user content is banned.
- Fixtures may reference synthetic corpora or recorded local fixtures. They
  must not require code or services outside this workspace.

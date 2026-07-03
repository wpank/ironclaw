# Caller-Level Test Matrix

Test the boundary that triggers the side effect. Helper-only tests are not
regression coverage when a wrapper computes inputs or controls behavior.

| Feature | Test target | Fixture | Mocked dependency | Protected side effect | Required assertion |
| --- | --- | --- | --- | --- | --- |
| Signal records | memory tool write/search | near-duplicate memory | DB + embedding fixture | memory persistence/ranking | flag off writes baseline only; flag on records candidate |
| Cascade router | provider factory/wrapper | simple lookup + high-risk request | fixture providers | model/provider choice | high-risk bypasses candidate; flag off uses static router |
| Progressive gates | generated-code/tool caller | webhook signature change | command runner fixture | code/tool publish | failing rung blocks publish and redacts artifact |
| Provider conductor | circuit breaker wrapper | latency ramp | fixture providers | fallback/pre-trip behavior | observe mode emits signal without changing state |
| Dream consolidation | heartbeat/routine caller | rare dependency session | deterministic LLM + memory store | derived memory write | kill switch prevents writes; taint/origin present |
| Event replay | gateway reconnect handler | reconnect cursor | auth/session fixture | stream replay | no duplicates and no cross-session events |
| Workspace code search | index/search facade | synthetic corpus | parser fixture | search results and telemetry | expected symbol in top 5; private paths not in metrics |
| Feature flags | feature evaluation call site | kill switch toggle | settings facade | all experimental behavior | disabled flag restores baseline |

## Test Artifact Template

```yaml
feature: experimental.<feature>
caller_boundary: module::handler_or_facade
fixture: path_or_name
mocked_dependencies:
  - external_service
side_effect: persisted row | provider choice | tool execution | event emission
assertions:
  - flag off baseline behavior
  - flag on candidate behavior
  - rollback switch restores baseline
```

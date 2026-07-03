# Feature Flag Inventory

Runtime flags control behavior. Compile features may control dependency
footprint only. All experimental flags default `off`.

| Feature | Flag key | Default | Config source | Reload | Kill switch owner |
| --- | --- | --- | --- | --- | --- |
| Signal records | `experimental.signal_records` | off | DB-backed setting | yes | workspace/memory owner |
| HDC memory search | `experimental.hdc_memory_search` | off | DB-backed workspace setting | yes | workspace/memory owner |
| Cascade router | `experimental.cascade_router` | off | runtime config + DB override | yes | LLM owner |
| Progressive gates | `experimental.progressive_gates` | off | runtime config | yes | verification/tool owner |
| Provider conductor | `experimental.provider_conductor` | off | runtime config | yes | LLM owner |
| Dream consolidation | `experimental.dream_consolidation` | off | DB-backed user/workspace setting | yes | agent/runtime owner |
| DAG workflow runner | `experimental.dag_workflow_runner` | off | runtime config | restart acceptable | product workflow owner |
| Event replay | `experimental.event_replay` | off | runtime config | yes | web gateway owner |
| Workspace code search | `experimental.workspace_code_search` | off | DB-backed workspace setting | yes | workspace owner |
| Extension hooks | `experimental.extension_hooks` | off | extension registry setting | yes | extension owner |
| Local reputation | `experimental.local_reputation` | off | DB-backed setting | yes | trust/reputation owner |
| Control-plane projection | `experimental.control_plane_projection` | off | runtime config | yes | web gateway owner |
| Affect engine | `experimental.affect_engine` | off | runtime config | yes | agent/runtime owner |
| Prompt composition | `experimental.prompt_composition` | off | runtime config | yes | agent/runtime owner |
| Swarm coordination | `experimental.swarm_coordination` | off | runtime config | yes | agent/runtime owner |

## Precedence

```text
hardcoded default < config file < env/bootstrap < DB setting < emergency kill switch
```

Secrets are never stored in feature flags. A flag may refer to a secret id only
when the secret itself remains in the secret store.

## Exposure Requirement

When a flag is evaluated at a caller boundary, emit `FeatureExposureEvent` with
the flag key, variant, stage, enabled state, and run/turn ids.

## Kill Switch Validation

For each flag:

```text
enable flag -> feature path is reachable
disable flag -> baseline path is authoritative
enable emergency kill switch -> baseline path is authoritative
```

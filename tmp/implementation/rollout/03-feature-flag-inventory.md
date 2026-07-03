# Feature Flag Inventory

Every experimental feature should use an explicit flag. Runtime flags control
behavior; compile flags only control dependency footprint.

| Feature | Flag key | Default | Config source | Reload | Kill switch owner |
|---|---|---|---|---|---|
| HDC memory search | `experimental.hdc_memory_search` | off | DB-backed workspace/user setting | yes | workspace/memory owner |
| Signal records | `experimental.signal_records` | off | DB-backed setting | yes | workspace/memory owner |
| Cascade router | `experimental.cascade_router` | off | runtime config + DB override | yes | LLM owner |
| Progressive gates | `experimental.progressive_gates` | off | runtime config | yes | verification/tool owner |
| Provider conductor | `experimental.provider_conductor` | off | runtime config | yes | LLM owner |
| Dream consolidation | `experimental.dream_consolidation` | off | DB-backed user/workspace setting | yes | agent/runtime owner |
| DAG workflow runner | `experimental.dag_workflow_runner` | off | runtime config | restart may be required | product workflow owner |
| Event replay | `experimental.event_replay` | off | runtime config | yes | web gateway owner |
| Code search | `experimental.workspace_code_search` | off | DB-backed workspace setting | yes | workspace owner |
| Plugin hooks | `experimental.extension_hooks` | off | extension registry setting | yes | extension owner |
| Local reputation | `experimental.local_reputation` | off | DB-backed setting | yes | trust/reputation owner |
| Control-plane projection | `experimental.control_plane_projection` | off | runtime config | yes | web gateway owner |

## Precedence

```text
hardcoded default < config file < env/bootstrap < DB setting < kill switch
```

Secrets are never stored in feature flags. A flag may refer to a secret id but
must not include secret material.

## Exposure Requirement

Every evaluation that can affect behavior must emit `FeatureExposureEvent`
before the behavior is applied.

## Kill Switch Validation

For each flag:

```text
enable feature -> verify candidate behavior
enable kill switch -> verify baseline behavior returns
verify data remains readable
```

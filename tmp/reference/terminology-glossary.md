# Terminology Glossary

This glossary resolves captured Roko naming, v2 spec naming, and IronClaw-native
implementation terms. Use captured names only when describing captured material;
use v2 or IronClaw terms when proposing IronClaw work.

| Captured source term | v2/spec term | Meaning | IronClaw adaptation |
|---|---|---|---|
| Engram | Signal | content-addressed knowledge/data record | `SignalRecord` concept layered onto workspace memory |
| Substrate | Store | persistence backend | DB facade with PostgreSQL/libSQL parity |
| Module | Cell | atomic computation unit | typed task/tool/gate node |
| Workflow | Graph | composed execution plan | DAG runner, product workflow, or task graph |
| EventSource | Trigger | event producer outside the active turn | extension/channel/routine trigger |
| Daimon | Affect engine | PAD/somatic state subsystem | optional policy metadata, not safety authority |
| Neuro | Memory/search | knowledge indexing and retrieval | workspace memory plus FTS/vector/HDC-style indexes |
| Dreams | Consolidation | offline/background learning | heartbeat-bounded derived memories |
| Gate rung | Verification step | progressive validation stage | compile/lint/test/security gate |
| Pheromone | Coordination marker | decaying shared signal | optional workspace coordination metadata |

## Usage Rule

When a document analyzes captured Roko code, it may use the captured term
(`Engram`, `Substrate`). When it proposes IronClaw implementation, prefer the
IronClaw-native or v2-compatible term (`SignalRecord`, `Store`, `Graph`) unless
the exact captured type name is being quoted.

## Count, Status, and Absence Rule

Avoid live count claims unless they were regenerated from local evidence in the
current branch. Prefer phrases such as "captured crate family,"
"captured plan set," or "no matching feature identified" when the reference
only proves what was present in the captured corpus.

# Terminology Glossary

This glossary resolves v1/v2 naming differences and prevents cross-document
confusion. The analysis keeps original captured names when discussing source
material, then maps them to IronClaw-native terms for implementation.

| v1/captured source term | v2/spec term | Meaning | IronClaw adaptation |
|---|---|---|---|
| Engram | Signal | content-addressed knowledge/data record | `SignalRecord` layered onto workspace memory |
| Substrate | Store | persistence backend | DB facade with PostgreSQL/libSQL parity |
| Module | Cell | atomic computation unit | typed task/tool/gate node |
| Workflow | Graph | composed execution plan | DAG runner or product workflow |
| EventSource | Trigger | external event producer | extension/channel/routine trigger |
| Daimon | Affect engine | PAD/somatic state subsystem | optional policy metadata, not safety authority |
| Neuro | Memory/search | knowledge indexing and retrieval | workspace memory + FTS/vector/HDC |
| Dreams | Consolidation | offline/background learning | heartbeat-bounded derived memories |
| Gate rung | Verification step | progressive validation stage | compile/lint/test/security gate |
| Pheromone | Coordination marker | decaying shared signal | optional workspace coordination metadata |

## Usage Rule

When a document analyzes captured Roko code, it may use the captured term
(`Engram`, `Substrate`). When it proposes IronClaw implementation, prefer the
IronClaw-native or v2-compatible term (`SignalRecord`, `Store`, `Graph`) unless
the exact captured type name is being quoted.

## Crate Count Rule

Use:

```text
18+ captured core crates, with a larger extended crate/tooling surface
```

Avoid implying that every language provider, MCP server, app, contract, and
tooling package is part of the same core crate count.


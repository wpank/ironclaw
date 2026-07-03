# Practical Examples Index

This folder is the operator and user-facing layer for the transfer analysis. The
numbered documents explain the mechanisms; these examples explain how the
mechanisms show up in real workflows, failures, and debugging sessions.

| File | Contents |
|---|---|
| `real-world-use-cases.md` | Practical workflows for cost, memory, generated code, background learning, and provider degradation |
| `02-end-to-end-scenarios.md` | Full workflows combining routing, gates, dreams, control-plane events, and reputation |
| `operator-debugging-runbooks.md` | Symptom -> inspect -> likely cause -> recovery guides |
| `failure-scenarios.md` | Failure modes, detection signals, mitigations, and regression tests |
| `user-stories.md` | Persona-based stories that explain why each feature matters |

## Cross-Reference Table

| Workflow | Primary concept docs | Metrics |
|---|---|---|
| Cheap routine question | [online-learning](../../agent-intelligence/online-learning.md), [conductor-anomaly](../../execution-verification/conductor-anomaly.md) | cost/request, quality pass rate, fallback rate |
| Multi-file refactor | [code-intelligence](../../context-memory/code-intelligence.md), [dag-execution](../../execution-verification/dag-execution.md), [gate-verification](../../execution-verification/gate-verification.md), [control-plane](../../ecosystem/control-plane.md) | escaped defects, p95 gate time, event loss |
| Memory dedup | [hyperdimensional-computing](../../core-concepts/hyperdimensional-computing/README.md), [universal-engram](../../core-concepts/universal-engram.md), [persistence-storage](../../context-memory/persistence-storage.md) | duplicate rate, relevance@10, write latency |
| Background learning | [dream-consolidation](../../agent-intelligence/dream-consolidation.md), [universal-engram](../../core-concepts/universal-engram.md), [runtime-infrastructure](../../execution-verification/runtime-infrastructure.md) | useful-memory hit rate, background spend |
| Safe extension install | [plugin-extension](../../ecosystem/plugin-extension.md), [chain-reputation](../../ecosystem/chain-reputation/README.md), [smart-contracts](../../ecosystem/smart-contracts/README.md) | permission denials, trust score, sandbox violations |


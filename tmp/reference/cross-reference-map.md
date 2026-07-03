# Cross-Reference Map

This map connects the first analysis batch, the later deep dives, and the
supplemental implementation layers.

| Primary doc | Must also read | Reason |
|---|---|---|
| [HDC](../core-concepts/hyperdimensional-computing/README.md) | [universal-engram](../core-concepts/universal-engram.md), [code-intelligence](../context-memory/code-intelligence.md), [benchmarking scenarios](../implementation/benchmarking/scenarios/README.md) | HDC applies to memory identity, code search, and benchmark fixtures |
| [dream-consolidation](../agent-intelligence/dream-consolidation.md) | [universal-engram](../core-concepts/universal-engram.md), [persistence-storage](../context-memory/persistence-storage.md), [rollout](../rollout/README.md) | Dream outputs need taint, persistence, budget, and rollout controls |
| [dag-execution](../execution-verification/dag-execution.md) | [orchestrator-swarm](../execution-verification/orchestrator-swarm.md), [runtime-infrastructure](../execution-verification/runtime-infrastructure.md), [schemas](../schemas/README.md) | DAG execution depends on runtime state, cancellation, and run records |
| [gate-verification](../execution-verification/gate-verification.md) | [testing-infrastructure](../implementation/08-caller-test-matrix.md), [benchmarking](../implementation/benchmarking/README.md), [schemas](../schemas/README.md) | Gates need caller-level tests, verdict schemas, and rollout thresholds |
| [online-learning](../agent-intelligence/online-learning.md) | [conductor-anomaly](../execution-verification/conductor-anomaly.md), [benchmarking/04](../implementation/benchmarking/04-rollout-metrics.md) | Routing decisions need health signals and rollout guardrails |
| [chain-reputation](../ecosystem/chain-reputation/README.md) | [smart-contracts](../ecosystem/smart-contracts/README.md), [reputation blueprints](../implementation/04-reputation-contract-blueprints.md) | Smart contracts and local ledger sketches complete the trust model |
| [budget-composition](../context-memory/budget-composition.md) | [code-intelligence](../context-memory/code-intelligence.md), [language-support](../context-memory/language-support.md) | Prompt packing depends on relevant code and symbol context |
| [universal-engram](../core-concepts/universal-engram.md) | [persistence-storage](../context-memory/persistence-storage.md), [schemas](../schemas/README.md), [terminology](terminology-glossary.md) | Signal identity requires persistence schema and terminology clarity |
| [code-intelligence](../context-memory/code-intelligence.md) | [language-support](../context-memory/language-support.md), [benchmarking scenario](../implementation/benchmarking/scenarios/workspace-code-search.yaml) | Language providers make code search concrete |
| [orchestrator-swarm](../execution-verification/orchestrator-swarm.md) | [control-plane](../ecosystem/control-plane.md) | Operators observe orchestration through projections and dashboards |
| [plugin-extension](../ecosystem/plugin-extension.md) | [rollout risk](../rollout/02-security-and-risk-register.md) | Plugins require config, permission, and sandbox discipline |
| [agent-patterns](../agent-intelligence/agent-patterns.md) | [mcp-editor-integration](../ecosystem/mcp-editor-integration.md), [caller-test-matrix](../implementation/08-caller-test-matrix.md) | Agent patterns surface in editor sessions and caller-level tests |
| [integration-roadmap](../strategy/integration-roadmap.md) | [per-file-action-matrix](../implementation/05-per-file-action-matrix.md), [rollout runbook](../implementation/06-rollout-runbook.md) | Roadmap items need measurable gates and release discipline |
| [priority-matrix](../strategy/priority-matrix/README.md) | [benchmarking/02](../implementation/benchmarking/02-feature-playbooks.md) | Scores should align with measurement status |
| [control-plane](../ecosystem/control-plane.md) | [examples/operator](examples/operator-debugging-runbooks.md) | Control plane design is validated through operator workflows |
| [plans-catalog](plans-catalog.md) | [plan-runner-readiness](../implementation/09-plan-runner-readiness.md), [rollout runbook](../implementation/06-rollout-runbook.md) | Plan tasks need execution contracts, verification, and rollout gates |
| [architecture-overview](architecture-overview.md) | [caller-test-matrix](../implementation/08-caller-test-matrix.md), [benchmarking/05](../implementation/benchmarking/05-runner-contract.md) | Test methodology becomes executable through caller coverage and runner contracts |
| [control-plane](../ecosystem/control-plane.md) | [schemas/04](../schemas/04-canonical-event-and-persistence-contract.md), [examples/operator](examples/operator-debugging-runbooks.md) | Dashboard projections need stable event contracts and operator scenarios |

## Reading Pattern

```text
concept doc -> adjacent deep dive -> schema/benchmark/runbook -> examples/failure scenario
```

This keeps theoretical concepts tied to buildable, measurable implementation
work.

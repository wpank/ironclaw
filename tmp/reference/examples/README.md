# Practical Examples Index

This folder turns the reference material into concrete IronClaw workflows,
failure modes, and operator checks. Each example is self-contained enough to
read on its own, but points to the deeper concept docs and benchmark fixtures
when a behavior should be measured.

| File | Contents |
|---|---|
| `real-world-use-cases.md` | Practical workflows for routing cost, memory quality, generated code verification, background learning, provider health, and extension trust |
| `02-end-to-end-scenarios.md` | End-to-end task paths with expected signals, guardrails, and benchmark fixtures |
| `operator-debugging-runbooks.md` | Symptom -> inspect -> likely cause -> recovery -> regression test guides |
| `failure-scenarios.md` | Failure modes, detection signals, mitigations, and caller-level regression tests |
| `user-stories.md` | Persona-based stories that explain why each feature matters |

## Cross-Reference Table

| Workflow | Primary concept docs | Benchmark fixture | Metrics |
|---|---|---|---|
| Cheap routine question | [online-learning](../../agent-intelligence/online-learning.md), [conductor-anomaly](../../execution-verification/conductor-anomaly.md) | [`cascade-router.yaml`](../../implementation/benchmarking/scenarios/cascade-router.yaml) | cost/request, quality pass rate, fallback rate |
| Multi-file refactor | [code-intelligence](../../context-memory/code-intelligence.md), [dag-execution](../../execution-verification/dag-execution.md), [gate-verification](../../execution-verification/gate-verification.md), [control-plane](../../ecosystem/control-plane.md) | [`gate-pipeline.yaml`](../../implementation/benchmarking/scenarios/gate-pipeline.yaml), [`workspace-code-search.yaml`](../../implementation/benchmarking/scenarios/workspace-code-search.yaml) | escaped defects, p95 gate time, event loss |
| Memory dedup | [hyperdimensional-computing](../../core-concepts/hyperdimensional-computing/README.md), [universal-engram](../../core-concepts/universal-engram.md), [persistence-storage](../../context-memory/persistence-storage.md) | [`memory-dedup.yaml`](../../implementation/benchmarking/scenarios/memory-dedup.yaml) | duplicate rate, relevance@10, write latency |
| Background learning | [dream-consolidation](../../agent-intelligence/dream-consolidation.md), [universal-engram](../../core-concepts/universal-engram.md), [runtime-infrastructure](../../execution-verification/runtime-infrastructure.md) | [`dream-consolidation.yaml`](../../implementation/benchmarking/scenarios/dream-consolidation.yaml) | useful-memory hit rate, background spend |
| Provider degradation | [conductor-anomaly](../../execution-verification/conductor-anomaly.md), [online-learning](../../agent-intelligence/online-learning.md) | [`provider-degradation.yaml`](../../implementation/benchmarking/scenarios/provider-degradation.yaml) | avoided degraded-provider spend, false warnings, oscillation count |
| Safe extension install | [plugin-extension](../../ecosystem/plugin-extension.md), [chain-reputation](../../ecosystem/chain-reputation/README.md), [smart-contracts](../../ecosystem/smart-contracts/README.md) | No fixture yet; add one before rollout | permission denials, trust score, sandbox violations |

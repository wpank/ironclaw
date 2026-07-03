# v2 Depth Research Guide

This is a curated guide to the captured `docs/v2-depth/*` material. The listed
paths are provenance labels, not checkout paths. Use this guide to decide which
local concept docs and implementation owners to read next.

## How To Use It

1. Pick the theme that matches the feature area.
2. Read the local concept docs linked from that row.
3. Treat captured claims about counts, savings, or guarantees as hypotheses
   until reproduced with local fixtures.
4. Translate ideas through IronClaw owners, not through direct `roko-*`
   dependencies.

## Theme Map

| Captured theme | Useful ideas | Local follow-up |
|---|---|---|
| Architecture and protocol algebra | Layering, typed cells, graph-shaped control flow, invariant tests. | [architecture-overview.md](architecture-overview.md), [../core-concepts/cognitive-architecture.md](../core-concepts/cognitive-architecture.md) |
| Signal and store | Content identity, taint, provenance, scoring, decay. | [terminology-glossary.md](terminology-glossary.md), [../core-concepts/universal-engram.md](../core-concepts/universal-engram.md), [../context-memory/persistence-storage.md](../context-memory/persistence-storage.md) |
| Compose and context | Prompt assembly, budget allocation, positional effects, active-inference-inspired scoring. | [../context-memory/budget-composition.md](../context-memory/budget-composition.md), [../context-memory/code-intelligence.md](../context-memory/code-intelligence.md) |
| Verify and gates | Structured verdicts, progressive checks, remediation feedback, calibration. | [../execution-verification/gate-verification.md](../execution-verification/gate-verification.md), [../implementation/08-caller-test-matrix.md](../implementation/08-caller-test-matrix.md) |
| Graph execution | DAG planning, waves, recovery, event logs, work isolation. | [../execution-verification/dag-execution.md](../execution-verification/dag-execution.md), [../execution-verification/runtime-infrastructure.md](../execution-verification/runtime-infrastructure.md) |
| Agent runtime | Tool loop, MCP, provider integration, lifecycle, supervision. | [../agent-intelligence/agent-patterns.md](../agent-intelligence/agent-patterns.md), [../ecosystem/mcp-editor-integration.md](../ecosystem/mcp-editor-integration.md) |
| Learning and routing | Bandit routing, provider health, feedback loops, stability. | [../agent-intelligence/online-learning.md](../agent-intelligence/online-learning.md), [../execution-verification/conductor-anomaly.md](../execution-verification/conductor-anomaly.md) |
| Memory and dreams | Hybrid memory, AntiKnowledge, consolidation, counterfactual replay. | [../agent-intelligence/dream-consolidation.md](../agent-intelligence/dream-consolidation.md), [../core-concepts/hyperdimensional-computing/README.md](../core-concepts/hyperdimensional-computing/README.md) |
| Security | Taint barriers, prompt security, defense-in-depth, artifact provenance. | [../execution-verification/gate-verification.md](../execution-verification/gate-verification.md), [../ecosystem/plugin-extension.md](../ecosystem/plugin-extension.md) |
| Code intelligence | Symbol graph, search, context assembly, MCP-accessible code tools. | [../context-memory/code-intelligence.md](../context-memory/code-intelligence.md), [../context-memory/language-support.md](../context-memory/language-support.md) |

## High-Value Reading Paths

| Goal | Captured labels to look for | Local check before implementation |
|---|---|---|
| Improve request flow | `architectural-thesis`, `cognitive-loop-as-graph`, `tool-loop-and-mcp` | Does the change go through the existing agent loop and tool dispatcher? |
| Reduce model spend | `bandit-routing-and-cascade`, `provider-health-and-pareto`, `active-inference-context-selection` | Are privacy/high-risk rules hard gates before learned routing? |
| Improve memory quality | `knowledge-as-signal`, `antiknowledge-and-immunity`, `dream-cycle-as-loop` | Are derived memories tainted, source-linked, and budgeted? |
| Verify generated work | `verify-as-universal-oracle`, `verify-cells-and-pipeline`, `gate-feedback-and-retry` | Is there caller-level coverage at the side-effect boundary? |
| Build code search | `code-intelligence-as-cell-pipeline`, `symbol-graph-and-importance`, `search-and-context-assembly` | Does the index preserve workspace semantics and privacy constraints? |
| Harden tools and plugins | `tool-architecture-as-cell-protocol`, `mcp-as-connect-protocol`, `prompt-security-and-camel` | Are approvals, sandboxing, and bearer/origin checks unchanged? |

## Terms To Normalize

- Prefer `Signal` for the captured universal record, and use `Engram` only when
  quoting captured type names.
- Prefer `Store` for persistence and keep IronClaw DB parity explicit.
- Prefer `Graph` for a plan shape, but use the Reborn runner/driver/executor
  path for subagent or workflow execution.
- Prefer `Gate` or `Verify step` for checks, and name the caller that enforces
  them.

## Related Reference

- Captured family map: [source-corpus-map.md](source-corpus-map.md)
- Implementation bridge: [v2-implementation-summary.md](v2-implementation-summary.md)
- Citations: [research-citations.md](research-citations.md)
- Plan patterns: [plans-catalog.md](plans-catalog.md)

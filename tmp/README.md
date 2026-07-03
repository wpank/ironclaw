# IronClaw Strategy Notes

This `tmp/` tree is a planning and reference corpus for evaluating agent-system
ideas against the current IronClaw codebase. It is not an implementation spec,
not a parity claim, and not evidence that a feature is ready to build.

Treat the current IronClaw code, subsystem specs, and `FEATURE_PARITY.md` as the
source of truth. Captured external source-path names in this corpus are kept
only for provenance; they are not live external dependencies.

## Current Inventory

Verified from the current tree on 2026-07-03.

| Item | Count |
|------|-------|
| Markdown files | 116 |
| YAML benchmark fixtures | 6 |
| Total files under `tmp/` | 122 |
| Directories under `tmp/`, including `tmp/` | 19 |
| Top-level folders | 8 |

## Top-Level Map

| Folder | Files | Purpose |
|--------|-------|---------|
| [core-concepts/](core-concepts/) | 11 | Foundational ideas such as HDC, engrams, robust statistics, and cognitive labels. |
| [agent-intelligence/](agent-intelligence/) | 5 | Learning, routing, monitor, affect, and consolidation concepts. |
| [execution-verification/](execution-verification/) | 6 | Runtime, DAG, gate, anomaly, and orchestration notes. |
| [context-memory/](context-memory/) | 5 | Prompt budgets, memory persistence, code search, and language support. |
| [ecosystem/](ecosystem/) | 16 | Plugins, MCP/editor integration, control plane, reputation, and contracts. |
| [strategy/](strategy/) | 10 | Priorities, roadmap, UX recommendations, benchmarking, and references. |
| [implementation/](implementation/) | 37 | IronClaw-oriented blueprints, checklists, schemas, runbooks, and benchmark fixtures. |
| [reference/](reference/) | 31 | Architecture overview, examples, glossary, citations, corpus maps, and quality notes. |

## Recommended Starting Point

Use the strategy folder first:

1. [strategy/README.md](strategy/README.md) - planning index and decision flow.
2. [strategy/priority-matrix/README.md](strategy/priority-matrix/README.md) - scoring model and build order.
3. [strategy/integration-roadmap.md](strategy/integration-roadmap.md) - phase gates, risks, and rollout rules.
4. [strategy/priority-matrix/benchmarking-plans.md](strategy/priority-matrix/benchmarking-plans.md) - baseline and rollout evidence.
5. [implementation/README.md](implementation/README.md) - implementation tracker and readiness links.

Before coding, read the relevant subsystem spec listed in `AGENTS.md`.

## Near-Term Sequence

The current strategy favors small, reversible improvements before broad runtime
changes:

1. Robust statistics for heavy-tailed cost and latency estimates.
2. Exact memory deduplication.
3. Memory decay with archival, not deletion.
4. Turn-level stuck-loop and cost-runaway monitoring.
5. A small reusable scoring interface tied to a real caller.
6. Cancellation propagation through actual tool and process boundaries.

Adaptive routing, gate expansion, HDC search, background consolidation, DAG
execution, and on-chain reputation are later-stage or conditional work. Each
needs baseline evidence, caller-level tests, and rollback before default-on
behavior.

## Role-Based Reading

| Role | Start with |
|------|------------|
| Backend/Rust | [implementation/05-per-file-action-matrix.md](implementation/05-per-file-action-matrix.md), then the owning subsystem docs. |
| Runtime/reliability | [execution-verification/README.md](execution-verification/README.md) and [strategy/integration-roadmap.md](strategy/integration-roadmap.md). |
| Memory/search | [context-memory/README.md](context-memory/README.md) and [core-concepts/universal-engram.md](core-concepts/universal-engram.md). |
| Model routing/evaluation | [agent-intelligence/online-learning.md](agent-intelligence/online-learning.md) and [strategy/priority-matrix/benchmarking-plans.md](strategy/priority-matrix/benchmarking-plans.md). |
| Product/UX | [strategy/ux-improvements.md](strategy/ux-improvements.md) and [reference/examples/README.md](reference/examples/README.md). |
| Security/release review | [implementation/rollout/02-security-and-risk-register.md](implementation/rollout/02-security-and-risk-register.md) and relevant subsystem specs. |

## Build Gates

Do not treat any item in this corpus as ready just because it is ranked.

- Check the current owner module and subsystem spec first.
- Capture a baseline before changing routing, ranking, gates, background work,
  or user-visible behavior.
- Test through the caller when a helper gates a side effect.
- Preserve PostgreSQL and libSQL parity for persistence changes.
- Review auth, secrets, sandboxing, listeners, approvals, and outbound HTTP with
  a security mindset.
- Update `FEATURE_PARITY.md`, specs, setup docs, or API docs when behavior
  status changes.
- Keep rollback practical: a flag, inert metadata, or a small revert.

## Provenance Policy

This corpus may mention captured project concepts or source-path names. Use them
as traceability hints only. Do not add live source-repo links, do not copy
external execution loops or storage models wholesale, and do not rely on external
project status when making IronClaw build decisions.

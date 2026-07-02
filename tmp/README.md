# Roko → IronClaw: Technology Transfer Analysis

A 29-document technical analysis comparing two Rust-based AI agent systems — **Roko** (~200K+ lines, 18 crates) and **[IronClaw](https://github.com/nearai/ironclaw)** — identifying novel architectural concepts in Roko that could be adopted into IronClaw.

**Date**: 2026-07-02

---

## Start Here

| Document | What it covers |
|----------|---------------|
| [00-INDEX.md](00-INDEX.md) | Master index with executive summary, reading guides, and key statistics |
| [26-roko-architecture-overview.md](26-roko-architecture-overview.md) | Comprehensive Roko architecture reference for newcomers |
| [19-priority-matrix.md](19-priority-matrix.md) | Ranked adoption priorities — what to build first and why |
| [18-integration-roadmap.md](18-integration-roadmap.md) | 4-phase engineering plan with dependency graph |
| [AUDIT-REPORT.md](AUDIT-REPORT.md) | Cross-document consistency audit |

---

## Core Concept Analyses (01–17)

Deep dives into individual Roko subsystems with code-level detail, IronClaw mapping, and integration proposals.

| # | Document | Topic |
|---|----------|-------|
| 01 | [Hyperdimensional Computing](01-hyperdimensional-computing.md) | Sub-millisecond similarity search via 10,240-bit binary vectors |
| 02 | [Dream Consolidation](02-dream-consolidation.md) | Offline memory consolidation and knowledge compression |
| 03 | [Affect Engine](03-affect-engine.md) | Emotional state modeling for agent behavior modulation |
| 04 | [DAG Execution](04-dag-execution.md) | Graph-based parallel task execution engine |
| 05 | [Gate Verification](05-gate-verification.md) | 7-step progressive verification pipeline (compile → integration tests) |
| 06 | [Conductor Anomaly](06-conductor-anomaly.md) | Reactive anomaly detection and self-repair |
| 07 | [Online Learning](07-online-learning.md) | Runtime model selection and prompt optimization |
| 08 | [Chain Reputation](08-chain-reputation.md) | Trust scoring across tool chains and agent interactions |
| 09 | [Budget Composition](09-budget-composition.md) | Token/cost budget management and allocation |
| 10 | [Universal Engram](10-universal-engram.md) | Content-addressed data objects for all agent output |
| 11 | [Mathematical Primitives](11-mathematical-primitives.md) | Core math abstractions (topology, category theory) |
| 12 | [Code Intelligence](12-code-intelligence.md) | AST-level code analysis and symbol resolution |
| 13 | [Cognitive Architecture](13-cognitive-architecture.md) | Neuroscience-inspired agent cognition framework |
| 14 | [Runtime Infrastructure](14-runtime-infrastructure.md) | Execution runtime, sandboxing, and resource management |
| 15 | [Orchestrator Swarm](15-orchestrator-swarm.md) | Multi-agent coordination and swarm patterns |
| 16 | [Plugin Extension](16-plugin-extension.md) | Plugin system architecture and extension points |
| 17 | [Agent Patterns](17-agent-patterns.md) | Reusable agent design patterns and best practices |

## Extended Analyses (20–28)

| # | Document | Topic |
|---|----------|-------|
| 20 | [Persistence & Storage](20-persistence-storage.md) | Storage layer architecture and data durability |
| 21 | [MCP & Editor Integration](21-mcp-editor-integration.md) | ACP/MCP protocols for editor connectivity (Zed, VS Code) |
| 22 | [Language Support](22-language-support.md) | Multi-language code analysis system |
| 23 | [Control Plane](23-control-plane.md) | API server and control plane architecture |
| 24 | [Smart Contracts](24-smart-contracts.md) | On-chain smart contract architecture |
| 25 | [Research Citations](25-research-citations.md) | Comprehensive bibliography of referenced research |
| 27 | [V2 Depth Research](27-v2-depth-research.md) | Catalog of Roko v2-depth research documents |
| 28 | [Plans Catalog](28-plans-catalog.md) | Machine-executable TOML implementation plans catalog |

---

## What is Roko?

Roko is a Rust toolkit for building **agents that build themselves**. Given a product requirements document, it generates an implementation plan, dispatches LLM-powered agents to execute tasks in parallel, validates output through a 7-step progressive verification pipeline, persists results as content-addressed data objects ("Signals"), and feeds outcomes back into learning subsystems.

What makes Roko unusual is the *harness* wrapping the LLM — it draws on neuroscience, economics, topology, and cybernetics:

- **Hyperdimensional computing** for sub-millisecond similarity search (no model inference — bitwise ops on binary vectors)
- **Dream consolidation** for offline memory compression
- **Affect engine** for emotional state modeling
- **DAG execution** for parallel task orchestration with dependency resolution
- **7-rung gate verification** from compilation through property-based testing

## What is IronClaw?

IronClaw is a secure personal AI assistant built in Rust — user-first security, self-expanding WASM-sandboxed tools, defense in depth, multi-channel access (CLI, web, Telegram, Slack, Discord, etc.) with proactive background execution.

---

## Scope

- **28 analysis documents** + 1 audit report
- **~43,800 lines** of technical analysis
- **18+ Roko crates** (~200K lines of Rust, 1,600+ tests) mapped against IronClaw's production codebase
- **25 concepts ranked** by composite ROI (user impact, system impact, ease, safety, independence)

# Research Citations

This directory is a local bibliography for the reference set. It preserves the
research context behind captured Roko concepts while avoiding live-source
assumptions. Captured crate names and document paths are provenance labels only.

## Topic Files

| File | Focus |
|---|---|
| [hdc-and-vsa.md](hdc-and-vsa.md) | Hyperdimensional computing, vector symbolic architectures, and similarity search. |
| [memory-and-learning.md](memory-and-learning.md) | Replay, consolidation, forgetting, online learning, and self-improvement. |
| [affect-and-cognition.md](affect-and-cognition.md) | Affect, somatic markers, cognitive architecture, active inference, and cybernetics. |
| [verification-and-safety.md](verification-and-safety.md) | Process control, gates, calibration, security, and interruptibility. |
| [agents-and-orchestration.md](agents-and-orchestration.md) | Agent harnesses, stigmergy, lifecycle, evolution, and temporal reasoning. |
| [blockchain-and-economics.md](blockchain-and-economics.md) | Mechanism-design diagnostics, demurrage, protocol standards, and reputation economics. |
| [context-and-search.md](context-and-search.md) | Context engineering, RAG, attention patterns, and information theory. |
| [math-and-statistics.md](math-and-statistics.md) | TDA, projection, changepoints, calibration, control, and ANN search. |

## Provenance Rules

- Paper summaries are research context, not implementation evidence.
- Captured source labels should be written as provenance labels, not local paths.
- Quantitative claims from papers or captured docs need verification before they
  become product requirements.
- Use local concept docs and fixtures to decide whether an idea is buildable in
  IronClaw.

## Reading Guide

| Goal | Start with |
|---|---|
| Understand memory and consolidation | [memory-and-learning.md](memory-and-learning.md), then [hdc-and-vsa.md](hdc-and-vsa.md). |
| Improve prompt/context selection | [context-and-search.md](context-and-search.md), then [blockchain-and-economics.md](blockchain-and-economics.md) for allocation analogies. |
| Harden generated-code workflows | [verification-and-safety.md](verification-and-safety.md), then [agents-and-orchestration.md](agents-and-orchestration.md). |
| Evaluate affect or supervision ideas | [affect-and-cognition.md](affect-and-cognition.md), then [math-and-statistics.md](math-and-statistics.md). |
| Explore trust and marketplaces | [blockchain-and-economics.md](blockchain-and-economics.md). |

For supplemental leads not yet folded into the topic files, see
[../additional-papers.md](../additional-papers.md).

# Agents And Orchestration Citations

Research context for software-agent harnesses, orchestration, stigmergy,
lifecycle state, and long-running coordination.

## Selected References

| Reference | Contribution | Local relevance |
|---|---|---|
| SWE-bench and SWE-agent (2024) | Agent harness and software-task evaluation. | Reminds implementation work to test real workflows, not only model output. |
| Anthropic (2024), Building Effective Agents | Practical agent patterns and workflow design. | Baseline for simple tool loops before adding complex orchestration. |
| ADAS (2025) | Automated design of agentic systems. | Research context for self-improving scaffolds; not an MVP requirement. |
| DSPy (2024) | Compiling declarative LM pipelines. | Prompt/program optimization analogue. |
| Grasse (1959), stigmergy | Coordination through environmental traces. | Captured pheromone vocabulary. |
| Dorigo and Gambardella (1997) | Ant-colony optimization. | Decaying/reinforced coordination traces. |
| FIPA (2002) | Agent lifecycle vocabulary. | Useful for naming long-running states if it maps cleanly to IronClaw state. |
| Pirolli and Card (1999) | Information foraging. | Context-search and stopping-rule analogy. |
| Turing (1952) | Morphogenesis and reaction-diffusion. | Captured specialization metaphor; treat as research context. |
| Allen (1983); Kowalski and Sergot (1986) | Temporal interval and event calculus. | Useful for reasoning about event histories and workflows. |
| Sculley et al. (2015) | Hidden technical debt in ML systems. | Warning for learned-routing and self-improvement features. |

## Use In IronClaw

- Subagent spawn must use existing Reborn runner/driver/executor paths.
- Event projections and checkpoints should explain long-running work without a
  second agent loop.
- Pheromone-style coordination should begin as local metadata, not as delegated
  authority.

Navigation: [README](README.md) | [Verification and Safety](verification-and-safety.md) |
[Context and Search](context-and-search.md)

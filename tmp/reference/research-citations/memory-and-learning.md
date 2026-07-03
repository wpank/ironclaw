# Memory And Learning Citations

Research context for consolidation, forgetting, replay, online routing, and
self-improving agent loops.

## Selected References

| Reference | Contribution | Local relevance |
|---|---|---|
| McClelland, McNaughton, and O'Reilly (1995) | Complementary learning systems: fast episodic and slow semantic memory. | Useful analogy for session logs versus durable summaries. |
| Wilson and McNaughton (1994) | Hippocampal replay during sleep. | Background for replay-based consolidation. |
| Mattar and Daw (2018) | Prioritized replay by utility. | Candidate scoring idea for choosing background episodes. |
| Schaul et al. (2016) | Prioritized experience replay. | ML baseline for replay selection. |
| Richards and Frankland (2017) | Forgetting as useful regularization. | Supports retrieval weighting rather than physical deletion. |
| Ebbinghaus (1885) | Forgetting curve and spaced repetition. | Simple decay analogy for memory ranking. |
| Park et al. (2023), Generative Agents | Memory and reflection in language-agent simulations. | Baseline for agent memory workflows. |
| Li et al. (2010), LinUCB | Contextual bandits for personalized recommendations. | Model/provider routing inspiration. |
| FrugalGPT / RouteLLM | Cost-aware model routing and cascading. | Compare against any IronClaw routing proposal. |
| Shinn et al. (2023), Reflexion | Verbal self-reflection for agents. | Cautionary reference for derived lessons and source links. |
| Zhao et al. (2024), ExpeL | Experience-derived rules for language agents. | Captured playbook/skill extraction analogue. |
| Wang et al. (2023), Voyager | Skill library and lifelong agent learning. | Useful for skill-memory separation. |

## Use In IronClaw

- Background learning should be budgeted, source-linked, and reversible.
- Derived memories should start low confidence and not outrank user-authored
  facts without later evidence.
- Learned routing must remain below static privacy and safety rules.

Navigation: [README](README.md) | [HDC and VSA](hdc-and-vsa.md) |
[Verification and Safety](verification-and-safety.md)

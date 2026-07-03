# Verification And Safety Citations

Research context for process control, generated-output verification, calibration,
capability security, information-flow control, and safe interruption.

## Selected References

| Reference | Contribution | Local relevance |
|---|---|---|
| Page (1954), CUSUM | Detect sustained process drift. | Candidate metric for provider or gate drift. |
| Roberts (1959), EWMA | Smooth recent observations for control charts. | Useful for latency/error trend monitoring. |
| Adams and MacKay (2007), BOCPD | Online changepoint detection. | Candidate for regime changes in pass rates or provider health. |
| Killick et al. (2012), PELT | Retrospective changepoint detection. | Postmortem analysis, not necessarily runtime gate. |
| Lightman et al. (2024), process supervision | Step-level verification can improve reasoning quality. | Supports structured gate feedback. |
| Huang et al. (2024), self-correction limits | LLM self-correction is unreliable without external signal. | Supports caller-owned verification. |
| Dennis and Van Horn (1966) | Capability security foundations. | Background for tool authority and sandbox design. |
| Denning (1976) | Lattice model for secure information flow. | Background for taint and declassification ideas. |
| Orseau and Armstrong (2016) | Safely interruptible agents. | Relevant to cancellation and stop paths. |
| Debenedetti et al. (2025), CaMeL | Prompt-injection defense by control/data separation. | Useful for safety architecture sketches; verify details before implementation. |
| Guo et al. (2017), calibration | Modern neural networks can be miscalibrated. | Confidence needs measurement before enforcement. |
| Farquhar et al. (2024), semantic entropy | Detect hallucination via semantic uncertainty. | Candidate high-cost verification feature. |

## Use In IronClaw

- Helper-level tests are not enough when a helper gates a side effect.
- Security claims need enforcement in code and caller-level tests.
- Learned thresholds should start in observe mode and graduate through fixtures.

Navigation: [README](README.md) | [Math and Statistics](math-and-statistics.md) |
[Agents and Orchestration](agents-and-orchestration.md)

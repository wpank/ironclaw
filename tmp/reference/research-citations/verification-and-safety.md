# Verification and Safety — Gates, Anomaly Detection, Process Control Citations

Part of the [Research Citations](README.md) collection.

---

## Statistical Process Control (SPC)

Statistical Process Control (SPC) provides methods for monitoring processes over time to detect when they depart from a target operating state. Originally developed for manufacturing quality control, SPC methods apply directly to monitoring AI system quality metrics (gate pass rates, model accuracy, latency). Three complementary detectors run per gate rung: CUSUM (accumulated sum) detects small sustained shifts, EWMA (exponentially weighted moving average) tracks drift with recency weighting, and BOCPD (Bayesian online changepoint detection) detects abrupt structural changes.

**Documents using this section**: `tmp/05-gate-verification.md`

---

**Page, E.S. (1954). Continuous Inspection Schemes. _Biometrika_, 41(1-2), 100-115.**
[DOI: 10.2307/2333009](https://doi.org/10.2307/2333009)

- **Concept**: CUSUM (Cumulative Sum) control chart. Detects small, sustained changes in the mean of a process by accumulating deviations from target. More sensitive than individual sample tests for detecting gradual drift.
- **Roko adaptation**: CUSUM detects sustained pass rate changes per gate rung. Parameters: k=0.25 (reference value), h=4.0 (decision interval).
- **Crate**: `roko-gate` (`spc.rs`)
- **Roko source**: `docs/v2/ARCHITECTURE-GUIDE.md` (lines 1380-1425), `docs/v2-depth/02-block/verdicts-as-signals.md` (line 298)
- **tmp/ cross-refs**: `05-gate-verification.md`

---

**Roberts, S.W. (1959). Control Chart Tests Based on Geometric Moving Averages. _Technometrics_, 1(3), 239-250.**
[DOI: 10.1080/00401706.1959.10489860](https://doi.org/10.1080/00401706.1959.10489860)

- **Concept**: EWMA (Exponentially Weighted Moving Average) control chart for drift detection. Weights recent observations more heavily than distant ones, making it sensitive to smooth trends.
- **Roko adaptation**: EWMA smoothing on gate pass rates. Used alongside CUSUM for complementary drift detection — CUSUM catches abrupt shifts, EWMA catches slow drifts.
- **Crate**: `roko-gate`
- **tmp/ cross-refs**: `05-gate-verification.md`

---

**Adams, R.P. & MacKay, D.J.C. (2007). Bayesian Online Changepoint Detection.**
[arXiv:0710.3742](https://arxiv.org/abs/0710.3742)

- **Concept**: BOCPD maintains a posterior distribution over run lengths (time since last change point). When P(run_length=0) spikes above threshold, a structural change has occurred. Provides probabilities rather than binary alarms.
- **Roko adaptation**: Detects fundamental behavioral shifts (model updates, major refactors). Parameters: hazard_rate = 1/200, max_run_length = 300, changepoint_threshold = 0.5.
- **Crate**: `roko-gate` (`spc.rs`)
- **Roko source**: `docs/v2-depth/02-block/ratcheting-and-adaptive-thresholds.md` (line 325)
- **tmp/ cross-refs**: `05-gate-verification.md`

---

**Killick, R., Fearnhead, P., & Eckley, I.A. (2012). Optimal Detection of Changepoints with a Linear Computational Cost. _Journal of the American Statistical Association_, 107(500), 1590-1598.**
[DOI: 10.1080/01621459.2012.737745](https://doi.org/10.1080/01621459.2012.737745)

- **Concept**: PELT (Pruned Exact Linear Time) algorithm for retrospective change point analysis with O(n) expected computational complexity.
- **Roko adaptation**: Retrospective analysis ("when did test reliability degrade?") on historical gate data.
- **tmp/ cross-refs**: `05-gate-verification.md`

---

## Process Reward Models and Verification

Outcome-only evaluation has a fundamental limitation: a correct final answer may be reached through flawed reasoning. Process Reward Models (PRMs) verify each intermediate reasoning step, providing denser feedback signals and detecting errors before they propagate.

**Documents using this section**: `tmp/05-gate-verification.md`

---

**Lightman, H., Kosaraju, V., Burda, Y., Edwards, H., Baker, B., Lee, T., Leike, J., Schulman, J., Sutskever, I., & Cobbe, K. (2024). Let's Verify Step by Step.**
[arXiv:2305.20050](https://arxiv.org/abs/2305.20050)

- **Concept**: Process reward models that verify each reasoning step outperform outcome reward models for mathematical reasoning. Per-step verification provides denser, more informative feedback.
- **Roko adaptation**: Per-gate scoring in the 7-rung verification ladder. Each gate checks a specific quality dimension.
- **tmp/ cross-refs**: `05-gate-verification.md`

---

**Song, Y., Gu, J., Wu, W., Ye, W., Zhao, H., & Hua, X. (2025). Mind the Gap: Examining the Self-Improvement Capabilities of Large Language Models. _International Conference on Learning Representations (ICLR)_.**

- **Concept**: Generation-Verification Gap: self-improvement works only when verification ability exceeds generation ability. When the verifier is no better than the generator, self-critique degrades performance.
- **Roko adaptation**: Foundational result validating the separation of agent (generator) and Gate (verifier). The Gate must use a different, higher-quality model or verifier.
- **Roko source**: `docs/v2-depth/02-block/eval-lifecycle-and-generation.md` (line 172)
- **tmp/ cross-refs**: `05-gate-verification.md`

---

**Huang, J., Chen, X., Mishra, S., Zheng, H.S., Yu, A.W., Song, X., & Zhou, D. (2024). Large Language Models Cannot Self-Correct Reasoning Yet. _International Conference on Learning Representations (ICLR)_.**
[arXiv:2310.01798](https://arxiv.org/abs/2310.01798)

- **Concept**: LLMs self-correcting without external feedback typically make answers worse. The model's self-critique draws on the same biases that produced the original error.
- **Roko adaptation**: External verification mandate. Motivates external Gates rather than self-assessment.
- **tmp/ cross-refs**: `05-gate-verification.md`

---

**Wei, J., Wang, X., Schuurmans, D., Bosma, M., Ichter, B., Xia, F., Chi, E., Le, Q., & Zhou, D. (2022). Chain-of-Thought Prompting Elicits Reasoning in Large Language Models. _Advances in Neural Information Processing Systems (NeurIPS)_, 35.**
[arXiv:2201.11903](https://arxiv.org/abs/2201.11903)

- **Concept**: Requiring models to produce intermediate reasoning steps (chain-of-thought) before answering dramatically improves performance and makes reasoning verifiable.
- **Roko adaptation**: CoT as a means to make reasoning verifiable per-step in the gate pipeline.
- **tmp/ cross-refs**: `05-gate-verification.md`

---

## Security, Safety, and Provenance

Security in AI agent systems requires principled models for controlling what information flows where and what actions can be taken with what capabilities.

**IronClaw relevance**: Direct applicability to `src/tools/wasm/` (capability sandbox), `crates/ironclaw_safety/` (safety layer), and `src/bridge/auth_manager.rs` (authentication flow).

---

**Dennis, J.B. & Van Horn, E.C. (1966). Programming Semantics for Multiprogrammed Computations. _Communications of the ACM_, 9(3), 143-155.**
[DOI: 10.1145/365230.365252](https://doi.org/10.1145/365230.365252)

- **Concept**: Capability-based security: unforgeable capability tokens control resource access. A capability is an unguessable token conferring specific rights; possession is sufficient authorization.
- **Roko adaptation**: Tool permission model uses unforgeable capability tokens.
- **IronClaw relevance**: IronClaw's WASM sandbox uses similar capability-based permission models.

---

**Denning, D.E. (1976). A Lattice Model of Secure Information Flow. _Communications of the ACM_, 19(5), 236-243.**
[DOI: 10.1145/360051.360056](https://doi.org/10.1145/360051.360056)

- **Concept**: Information flow control modeled as a lattice. Security levels form a partial order; flows are permitted only from lower to higher levels. Prevents unauthorized downward flows.
- **Roko source**: `docs/v2/01-SIGNAL.md` (line 1490)

---

**Orseau, L. & Armstrong, S. (2016). Safely Interruptible Agents. _Proceedings of the 32nd Conference on Uncertainty in Artificial Intelligence (UAI)_.**

- **Concept**: Off-policy learning for safe shutdown. RL agents can be made safely interruptible using off-policy learning, preventing agents from learning to avoid or seek interruptions.
- **Roko adaptation**: Agent lifecycle management. Users can delete agents without agent resistance.

---

**Bai, Y., Kadavath, S., Kundu, S., Askell, A., et al. (2022). Constitutional AI: Harmlessness from AI Feedback.**
[arXiv:2212.08073](https://arxiv.org/abs/2212.08073)

- **Concept**: Constitutional constraints on behavior enforced through AI self-critique and revision.
- **Roko adaptation**: Policy trait constitutional constraints in safety layer.
- **IronClaw relevance**: IronClaw's safety layer (`crates/ironclaw_safety/`) implements similar constitutional safety constraints.

---

**Debenedetti, E., Greshake Beatrix, T., Kamath, A., Panaitescu-Liess, E., Shumailov, I., & Tramer, F. (2025). Defeating Prompt Injections by Design (CaMeL).**
[arXiv:2503.18813](https://arxiv.org/abs/2503.18813)

- **Concept**: CaMeL: Capability-tagged information flow control that separates control flow from data flow. Achieves 67% of tasks with provable security on AgentDojo benchmark.
- **Roko adaptation**: CaMeL IFC applied to Extensions. Every data flow through an Extension is tagged with its capability provenance.
- **Roko source**: `docs/v2/16-SECURITY.md` (line 232)

---

## Calibration and Uncertainty

A calibrated model's confidence should match its accuracy. Modern neural networks are notoriously overconfident. Calibration is essential for reliable decision-making, particularly in routing contexts.

**Documents using this section**: `tmp/07-online-learning.md`

---

**Guo, C., Pleiss, G., Sun, Y., & Weinberger, K.Q. (2017). On Calibration of Modern Neural Networks. _Proceedings of the International Conference on Machine Learning (ICML)_, Vol. 70, pp. 1321-1330.**
[arXiv:1706.04599](https://arxiv.org/abs/1706.04599)

- **Concept**: Modern neural networks are poorly calibrated — they are overconfident. Temperature scaling (dividing logits by a learnable scalar T) is a simple and effective post-hoc calibration fix.
- **Roko adaptation**: CalibrationTracker bias correction for model routing confidence.
- **Roko source**: `docs/v2/01-SIGNAL.md` (line 1487)
- **tmp/ cross-refs**: `07-online-learning.md`

---

**Naeini, M.P., Cooper, G., & Hauskrecht, M. (2015). Obtaining Well Calibrated Probabilities Using Bayesian Binning into Quantiles. _Proceedings of the AAAI Conference on Artificial Intelligence_, 29(1).**

- **Concept**: Expected Calibration Error (ECE): binned accuracy-confidence gaps as a calibration metric.
- **Roko adaptation**: CalibrationTracker uses ECE for measuring and correcting model confidence over time.
- **Roko source**: `docs/v2/01-SIGNAL.md` (line 1488)
- **tmp/ cross-refs**: `07-online-learning.md`

---

**Vovk, V., Gammerman, A., & Shafer, G. (2005). _Algorithmic Learning in a Random World_ (Conformal Prediction). New York: Springer. ISBN: 978-0387001524.**

- **Concept**: Conformal prediction: distribution-free prediction intervals with guaranteed marginal coverage.
- **Roko adaptation**: CalibrationTracker bounds on prediction confidence.
- **tmp/ cross-refs**: `07-online-learning.md`

---

**Farquhar, S., Kossen, J., Kuhn, L., & Gal, Y. (2024). Detecting Hallucinations in Large Language Models Using Semantic Entropy. _Nature_, 630, 625-630.**
[DOI: 10.1038/s41586-024-07421-0](https://doi.org/10.1038/s41586-024-07421-0)

- **Concept**: Semantic entropy measures consistency across independently sampled completions. High semantic entropy (inconsistent answers) indicates hallucination.
- **Roko adaptation**: Informs confidence estimation in the Gate pipeline.

---

## Streaming Algorithms

Streaming algorithms process data in a single pass with bounded memory, making hard decisions that cannot be revisited.

**Documents using this section**: `tmp/07-online-learning.md`

---

**Bifet, A. & Gavalda, R. (2007). Learning from Time-Changing Data with Adaptive Windowing. _Proceedings of the 2007 SIAM International Conference on Data Mining_, pp. 443-448.**
[DOI: 10.1137/1.9781611972771.42](https://doi.org/10.1137/1.9781611972771.42)

- **Concept**: ADWIN (ADaptive WINdowing): detects distribution change and adjusts window size automatically.
- **Roko adaptation**: Detects when a model's quality has shifted, triggering re-evaluation of routing weights.
- **tmp/ cross-refs**: `07-online-learning.md`

---

**Hansen, E.A. & Zilberstein, S. (2001). Monitoring and Control of Anytime Algorithms: A Survey. _Artificial Intelligence_, 126(1-2), 43-83.**
[DOI: 10.1016/S0004-3702(00)00063-7](https://doi.org/10.1016/S0004-3702(00)00063-7)

- **Concept**: Anytime algorithms produce progressively better results with more compute and can be interrupted at any point, with quality monitored via performance profiles.
- **Roko adaptation**: Grounds the T0/T1/T2 cascade: the agent stops at the cheapest tier that meets quality thresholds.

---

*Navigation: [README](README.md) | [HDC and VSA](hdc-and-vsa.md) | [Memory and Learning](memory-and-learning.md) | [Affect and Cognition](affect-and-cognition.md) | [Agents and Orchestration](agents-and-orchestration.md) | [Blockchain and Economics](blockchain-and-economics.md) | [Context and Search](context-and-search.md) | [Math and Statistics](math-and-statistics.md)*

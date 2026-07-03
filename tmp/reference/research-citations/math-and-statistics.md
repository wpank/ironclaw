# Math and Statistics — TDA, Sheaves, Robust Stats, Tropical Algebra Citations

Part of the [Research Citations](README.md) collection.

---

## Topological Data Analysis (TDA)

Topological Data Analysis (TDA) uses methods from algebraic topology to extract shape features from data that are invariant to continuous deformation. The key tool is persistent homology: tracking when topological features (connected components, loops, voids) are born and die as a scale parameter varies. Features that persist across many scales are genuine structure; features that immediately die are noise. TDA is particularly powerful for time-series analysis because it captures global structure that local statistics miss.

**Documents using this section**: `tmp/11-mathematical-primitives.md`

---

**Takens, F. (1981). Detecting Strange Attractors in Turbulence. In D. Rand & L.S. Young (Eds.), _Dynamical Systems and Turbulence_, Lecture Notes in Mathematics, Vol. 898, pp. 366-381. Berlin: Springer.**
[DOI: 10.1007/BFb0091924](https://doi.org/10.1007/BFb0091924)

- **Concept**: Takens delay embedding theorem: a 1-D time series can be embedded into a d-dimensional phase space that preserves the topology of the underlying dynamical system, recovering its attractor structure.
- **Roko adaptation**: Converts 1-D time series (gate pass rates over time) into point clouds in d-dimensional phase space for topological analysis.
- **Crate**: `roko-primitives` (`tda.rs`)
- **Roko source**: `crates/roko-primitives/src/tda.rs` (line 15)
- **tmp/ cross-refs**: `11-mathematical-primitives.md`

---

**Carlsson, G. (2009). Topology and Data. _Bulletin of the American Mathematical Society_, 46(2), 255-308.**
[DOI: 10.1090/S0273-0979-09-01249-X](https://doi.org/10.1090/S0273-0979-09-01249-X)

- **Concept**: Persistent homology: tracks birth and death of topological features across increasing scale parameters. Features far from the diagonal in persistence diagrams are genuine structure; near-diagonal features are noise. H0 = connected components, H1 = loops, H2 = voids.
- **Crate**: `roko-primitives` (`tda.rs`)
- **Roko source**: `crates/roko-primitives/src/tda.rs` (line 16)
- **tmp/ cross-refs**: `11-mathematical-primitives.md`

---

**Bubenik, P. (2015). Statistical Topological Data Analysis Using Persistence Landscapes. _Journal of Machine Learning Research_, 16, 77-102.**
[JMLR link](https://jmlr.org/papers/v16/bubenik15a.html)

- **Concept**: Persistence landscape: vectorization of persistence diagrams into a Banach space element, enabling statistical operations (mean, variance, hypothesis testing) on topological summaries.
- **Crate**: `roko-primitives` (`tda.rs`)
- **Roko source**: `crates/roko-primitives/src/tda.rs` (line 18)
- **tmp/ cross-refs**: `11-mathematical-primitives.md`

---

**Bauer, U. (2021). Ripser: Efficient Computation of Vietoris-Rips Persistence Barcodes. _Journal of Applied and Computational Topology_, 5, 391-423.**
[DOI: 10.1007/s41468-021-00071-5](https://doi.org/10.1007/s41468-021-00071-5)

- **Concept**: Efficient O(n^3) algorithm for computing persistence barcodes via the Vietoris-Rips complex, enabling practical TDA on datasets of hundreds to thousands of points.
- **Roko source**: `docs/v1/21-references/12-signal-processing.md`
- **tmp/ cross-refs**: `11-mathematical-primitives.md`

---

**Gidea, M. & Katz, Y. (2018). Topological Data Analysis of Financial Time Series: Landscapes of Crashes. _Physica A: Statistical Mechanics and Its Applications_, 491, 820-834.**
[DOI: 10.1016/j.physa.2017.09.028](https://doi.org/10.1016/j.physa.2017.09.028)

- **Concept**: Persistent homology detects structural changes in financial time series (specifically rising H1 norms) that systematically precede crashes by several months.
- **Roko adaptation**: Applicable to anomaly detection in agent performance metrics — topological features precede behavioral crashes.
- **Roko source**: `docs/v1/21-references/12-signal-processing.md`

---

## Johnson-Lindenstrauss Lemma

**Johnson, W.B. & Lindenstrauss, J. (1984). Extensions of Lipschitz Mappings into a Hilbert Space. _Contemporary Mathematics_, 26, 189-206.**

- **Concept**: The JL lemma: N points can be embedded into O(log N / epsilon^2) dimensions while preserving pairwise distances within factor (1 +/- epsilon). For epsilon=0.1 and N=100,000, the minimum dimensionality is D >= 4,604.
- **Roko adaptation**: Roko's 10,240 bits provide generous headroom beyond the JL lower bound. Mathematical foundation for projecting 1,536-dimensional LLM embeddings to 10,240-bit binary hypervectors without catastrophic information loss.
- **Crate**: `roko-core`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`, `11-mathematical-primitives.md`

---

## Submodularity

**Nemhauser, G.L., Wolsey, L.A., & Fisher, M.L. (1978). An Analysis of Approximations for Maximizing Submodular Set Functions — I. _Mathematical Programming_, 14(1), 265-294.**
[DOI: 10.1007/BF01588971](https://doi.org/10.1007/BF01588971)

- **Concept**: Greedy algorithm provides (1-1/e) ≈ 0.632 approximation for maximizing monotone submodular functions subject to cardinality constraints.
- **Roko adaptation**: Context selection is submodular (diminishing returns from adding more of the same type). Greedy knapsack for context window allocation is near-optimal.
- **tmp/ cross-refs**: `09-budget-composition.md`

---

## Ergodicity and Probability

**Peters, O. (2019). The Ergodicity Problem in Economics. _Nature Physics_, 15, 1216-1221.**
[DOI: 10.1038/s41567-019-0732-0](https://doi.org/10.1038/s41567-019-0732-0)

- **Concept**: The ensemble average of wealth growth (average over many agents at one time) differs from the time average (one agent over time) in multiplicative processes. Log-wealth maximization is optimal for individual agents in non-ergodic settings.
- **Roko adaptation**: Kelly criterion for routing budget allocation. Agents maximize time-average (not ensemble-average) returns.
- **tmp/ cross-refs**: `08-chain-reputation.md`

---

**Kelly, J.L. Jr. (1956). A New Interpretation of Information Rate. _Bell System Technical Journal_, 35(4), 917-926.**
[DOI: 10.1002/j.1538-7305.1956.tb03809.x](https://doi.org/10.1002/j.1538-7305.1956.tb03809.x)

- **Concept**: Kelly criterion: optimal bet sizing that maximizes the logarithm of wealth. The Kelly fraction (edge/odds) maximizes the geometric growth rate of capital over time.
- **Roko adaptation**: Position sizing in Route decisions and attention budget allocation.
- **tmp/ cross-refs**: `08-chain-reputation.md`

---

## Changepoint Detection

**Adams, R.P. & MacKay, D.J.C. (2007). Bayesian Online Changepoint Detection.**
[arXiv:0710.3742](https://arxiv.org/abs/0710.3742)

- **Concept**: BOCPD maintains a posterior distribution over run lengths (time since last change point). Provides probabilities rather than binary alarms.
- **Crate**: `roko-gate` (`spc.rs`)
- **tmp/ cross-refs**: `05-gate-verification.md`

---

**Killick, R., Fearnhead, P., & Eckley, I.A. (2012). Optimal Detection of Changepoints with a Linear Computational Cost. _Journal of the American Statistical Association_, 107(500), 1590-1598.**
[DOI: 10.1080/01621459.2012.737745](https://doi.org/10.1080/01621459.2012.737745)

- **Concept**: PELT (Pruned Exact Linear Time) algorithm for retrospective change point analysis with O(n) expected computational complexity.
- **Roko adaptation**: Retrospective analysis ("when did test reliability degrade?") on historical gate data.
- **tmp/ cross-refs**: `05-gate-verification.md`

---

**Bifet, A. & Gavalda, R. (2007). Learning from Time-Changing Data with Adaptive Windowing. _Proceedings of the 2007 SIAM International Conference on Data Mining_, pp. 443-448.**
[DOI: 10.1137/1.9781611972771.42](https://doi.org/10.1137/1.9781611972771.42)

- **Concept**: ADWIN (ADaptive WINdowing): detects distribution change and adjusts window size automatically.
- **tmp/ cross-refs**: `07-online-learning.md`

---

## Control Theory

**Page, E.S. (1954). Continuous Inspection Schemes. _Biometrika_, 41(1-2), 100-115.**
[DOI: 10.2307/2333009](https://doi.org/10.2307/2333009)

- **Concept**: CUSUM (Cumulative Sum) control chart. Detects small, sustained changes in the mean of a process.
- **Crate**: `roko-gate` (`spc.rs`)
- **tmp/ cross-refs**: `05-gate-verification.md`

---

**Roberts, S.W. (1959). Control Chart Tests Based on Geometric Moving Averages. _Technometrics_, 1(3), 239-250.**
[DOI: 10.1080/00401706.1959.10489860](https://doi.org/10.1080/00401706.1959.10489860)

- **Concept**: EWMA (Exponentially Weighted Moving Average) control chart for drift detection.
- **Crate**: `roko-gate`
- **tmp/ cross-refs**: `05-gate-verification.md`

---

**Maxwell, J.C. (1868). On Governors. _Proceedings of the Royal Society of London_, 16, 270-283.**
[DOI: 10.1098/rspl.1867.0055](https://doi.org/10.1098/rspl.1867.0055)

- **Concept**: First mathematical analysis of feedback control. The centrifugal governor prevents steam engine oscillation through proportional damping.
- **Roko adaptation**: Foundational for the adaptive clock control system.

---

## Calibration Mathematics

**Guo, C., Pleiss, G., Sun, Y., & Weinberger, K.Q. (2017). On Calibration of Modern Neural Networks. _Proceedings of the International Conference on Machine Learning (ICML)_, Vol. 70, pp. 1321-1330.**
[arXiv:1706.04599](https://arxiv.org/abs/1706.04599)

- **Concept**: Temperature scaling: dividing logits by a learnable scalar T is a simple and effective post-hoc calibration fix.
- **Roko adaptation**: CalibrationTracker bias correction for model routing confidence.
- **tmp/ cross-refs**: `07-online-learning.md`

---

**Vovk, V., Gammerman, A., & Shafer, G. (2005). _Algorithmic Learning in a Random World_ (Conformal Prediction). New York: Springer. ISBN: 978-0387001524.**

- **Concept**: Conformal prediction: distribution-free prediction intervals with guaranteed marginal coverage. No distributional assumptions required.
- **Roko adaptation**: CalibrationTracker bounds on prediction confidence.
- **tmp/ cross-refs**: `07-online-learning.md`

---

## Decay and Regularization Mathematics

**Ebbinghaus, H. (1885). _Uber das Gedachtnis: Untersuchungen zur experimentellen Psychologie_. Leipzig: Duncker & Humblot. Translated by Ruger, H.A. & Bussenius, C.E. (1913). New York: Teachers College, Columbia University.**

- **Concept**: The forgetting curve follows negative exponential decay: `R = e^(-t/S)`. Retrieval strengthens the trace and slows subsequent decay.
- **Roko adaptation**: Directly implemented as `weight = exp(-age / (strength * scale_ms))`. Per-type half-lives: Episodes 48h, Insights 7d, Heuristics 14d, Warnings 30d.
- **Crate**: `roko-core` (`decay.rs`)
- **tmp/ cross-refs**: `10-universal-engram.md`, `11-mathematical-primitives.md`

---

**Richards, B.A. & Frankland, P.W. (2017). The Persistence and Transience of Memory. _Neuron_, 94(6), 1071-1084.**
[DOI: 10.1016/j.neuron.2017.04.037](https://doi.org/10.1016/j.neuron.2017.04.037)

- **Concept**: Active forgetting is mathematically equivalent to L1 regularization — it sparsifies representations and prevents overfitting to stale environmental statistics.
- **tmp/ cross-refs**: `10-universal-engram.md`

---

## Locality-Sensitive Hashing

**Charikar, M.S. (2002). Similarity Estimation from Rounding Algorithms. _Proceedings of the 34th Annual ACM Symposium on Theory of Computing (STOC)_, pp. 380-388.**
[DOI: 10.1145/509907.509965](https://doi.org/10.1145/509907.509965)

- **Concept**: SimHash — a single random projection h(x) = sign(w^T x) produces binary codes where collision probability equals 1 - theta/pi, relating Hamming distance to angular distance.
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

**Malkov, Y.A. & Yashunin, D.A. (2020). Efficient and Robust Approximate Nearest Neighbor Search Using Hierarchical Navigable Small World Graphs. _IEEE Transactions on Pattern Analysis and Machine Intelligence_, 42(4), 824-836.**
[DOI: 10.1109/TPAMI.2018.2889473](https://doi.org/10.1109/TPAMI.2018.2889473)

- **Concept**: HNSW: O(log N) approximate nearest neighbor search at 95-99% recall for billion-scale vectors.
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

*Navigation: [README](README.md) | [HDC and VSA](hdc-and-vsa.md) | [Memory and Learning](memory-and-learning.md) | [Affect and Cognition](affect-and-cognition.md) | [Verification and Safety](verification-and-safety.md) | [Agents and Orchestration](agents-and-orchestration.md) | [Blockchain and Economics](blockchain-and-economics.md) | [Context and Search](context-and-search.md)*

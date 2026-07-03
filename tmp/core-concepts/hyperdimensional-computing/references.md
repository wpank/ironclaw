[← Back to HDC Overview](./README.md)

# Academic References

All citations for the Hyperdimensional Computing documents. Organized by topic area. Performance numbers in cited papers are external baselines only; IronClaw rollout decisions require local benchmarks on target hardware and representative corpora.

---

## Foundational Theory

**[Kanerva88]** Kanerva, P. (1988). *Sparse Distributed Memory*. Cambridge, MA: MIT Press. ISBN 9780262111324.

The original work on content-addressable memory in high-dimensional binary spaces. Introduced the mathematical framework for storing and retrieving patterns in spaces of thousands of dimensions, showing that random high-dimensional addresses are almost always far apart — the foundational observation that HDC exploits. Out of print; a PDF is available via the author's Stanford page.

---

**[Kanerva96]** Kanerva, P. (1996). "Binary Spatter-Coding of Ordered K-Tuples." In C. von der Malsburg et al. (eds.), *Artificial Neural Networks — ICANN 96* (Lecture Notes in Computer Science, vol. 1112), pp. 869–873. Berlin: Springer.

Formalized Binary Spatter Codes (BSC): the specific HDC variant used by roko and proposed for IronClaw. Defined XOR as the binding operation and majority vote as the bundling operation for binary vectors. Showed that ordered sequences can be encoded using cyclic permutation combined with XOR binding.

---

**[Kanerva09]** Kanerva, P. (2009). "Hyperdimensional Computing: An Introduction to Computing in Distributed Representation with High-Dimensional Random Vectors." *Cognitive Computation*, 1(2), 139–159. DOI: [10.1007/s12559-009-9009-8](https://doi.org/10.1007/s12559-009-9009-8).

The primary reference for the 10,240-bit BSC dimensionality choice and the systematic treatment of the four core algebraic operations. Demonstrated that high-dimensional random vectors form a robust algebra suitable for general-purpose computing. **This is the most important single citation for the overall framework.**

---

**[Gayler03]** Gayler, R.W. (2003). "Vector Symbolic Architectures Answer Jackendoff's Challenges for Cognitive Neuroscience." In *Proceedings of the Joint International Conference on Cognitive Science (ICCS/ASCS'03)*, pp. 133–138. arXiv: [cs/0412059](https://arxiv.org/abs/cs/0412059).

Introduced the term "Vector Symbolic Architectures" (VSA) to unify the family of models including BSC, MAP, HRR, and FHRR. Argued that VSAs provide a computationally adequate framework for implementing symbolic cognitive architectures using distributed representations.

---

## Holographic Representations

**[Plate95]** Plate, T.A. (1995). "Holographic Reduced Representations." *IEEE Transactions on Neural Networks*, 6(3), 623–641. DOI: [10.1109/72.377968](https://doi.org/10.1109/72.377968).

Introduced Holographic Reduced Representations (HRR), using circular convolution as the binding operation on real-valued vectors. Proved capacity bounds for bundling and showed that compositional structure can be recovered via circular correlation. HRR is the theoretical ancestor of BSC; BSC can be viewed as a binarized approximation that trades representational precision for computational efficiency.

---

**[Plate03]** Plate, T.A. (2003). *Holographic Reduced Representations: Distributed Representation for Cognitive Structures*. CSLI Publications. ISBN 9781575864303.

The comprehensive monograph on HRR theory. Contains the bundle capacity proofs that the SNR analysis builds on, and the formal treatment of role-filler binding and unbinding that the `RoleFillerEncoder` implements.

---

## Surveys and Capacity Bounds

**[Kleyko22]** Kleyko, D., Rachkovskij, D.A., Osipov, E., & Rahimi, A. (2022). "A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures, Part I: Models and Data Transformations." *ACM Computing Surveys*, 55(6), Article 130, 40 pages. DOI: [10.1145/3538531](https://doi.org/10.1145/3538531). Part II: DOI: [10.1145/3558000](https://doi.org/10.1145/3558000).

The most comprehensive HDC/VSA survey to date. Validates the bundle similarity formula and capacity bounds used throughout this document. Provides a systematic comparison of all major VSA variants (BSC, MAP, HRR, FHRR) including their algebraic properties, capacity, and computational requirements.

---

**[Thomas21]** Thomas, A., Dasgupta, S., & Rosing, T. (2021). "A Theoretical Perspective on Hyperdimensional Computing." *Journal of Artificial Intelligence Research (JAIR)*, 72, 215–249. DOI: [10.1613/jair.1.12664](https://doi.org/10.1613/jair.1.12664). arXiv: [2010.07426](https://arxiv.org/abs/2010.07426).

Provides a unified theoretical treatment of HDC. Establishes capacity scaling laws showing that bundle capacity scales as K = O(D / log(D)), which is tighter than the simple SNR bound. Also provides the theoretical foundation for trigram encoding used in the code fingerprinting system.

---

## Random Projection Theory

**[JL84]** Johnson, W.B. & Lindenstrauss, J. (1984). "Extensions of Lipschitz Mappings into a Hilbert Space." *Contemporary Mathematics*, 26, 189–206.

The JL lemma: N points can be projected into O(log N / epsilon^2) dimensions while preserving pairwise distances within (1 ± epsilon). Provides theoretical justification for the sufficiency of high-dimensional random projections, though the specific capacity of BSC vectors is better characterized by the SNR model and false positive analysis.

---

## Resonator Networks

**[Frady18]** Frady, E.P., Kleyko, D., & Sommer, F.T. (2018). "A Theory of Sequence Indexing and Working Memory in Recurrent Neural Networks." *Neural Computation*, 30(6), 1449–1513. DOI: [10.1162/neco_a_01064](https://doi.org/10.1162/neco_a_01064).

Proposed sequence indexing using HDC operations within recurrent neural networks. Showed that VSA coding principles (permutation for position, XOR for binding) can be implemented in biologically plausible recurrent networks, providing the theoretical basis for permutation-based causal link encoding.

---

**[Frady20]** Frady, E.P., Kent, S.J., Olshausen, B.A., & Sommer, F.T. (2020). "Resonator Networks, 1: An Efficient Solution for Factoring High-Dimensional, Distributed Representations of Data Structures." *Neural Computation*, 32(12), 2311–2331. DOI: [10.1162/neco_a_01331](https://doi.org/10.1162/neco_a_01331). arXiv: [2005.12649](https://arxiv.org/abs/2005.12649).

Introduced resonator networks as an efficient algorithm for decomposing composite HDC vectors into their constituent factors — solving the "inverse bundling" problem that brute-force unbinding cannot efficiently address. Relevant for future IronClaw features like automatic knowledge decomposition.

---

## Locality-Sensitive Hashing

**[Charikar02]** Charikar, M.S. (2002). "Similarity Estimation Techniques from Rounding Algorithms." In *Proceedings of the 34th Annual ACM Symposium on Theory of Computing (STOC)*, pp. 380–388. DOI: [10.1145/509907.509965](https://doi.org/10.1145/509907.509965).

SimHash: showed that single random projections produce binary codes where collision probability equals cosine similarity. HDC vectors can be viewed as SimHash signatures with the additional structure provided by the four algebraic operations. The Hamming distance between HDC vectors preserves approximate cosine similarity relationships.

---

## Statistical Correction

**[Bonferroni36]** Bonferroni, C.E. (1936). "Teoria statistica delle classi e calcolo delle probabilita." *Pubblicazioni del R. Istituto Superiore di Scienze Economiche e Commerciali di Firenze*, 8, 3–62.

The multiple-comparison correction used for threshold selection: when testing N hypotheses, divide the significance level by N to control the family-wise error rate. Applied in the false positive analysis to derive the 0.526 threshold for 100K-entry vocabularies.

---

## Hardware and Systems

**[Neubert19]** Neubert, P., Schubert, S., & Protzel, P. (2019). "An Introduction to Hyperdimensional Computing for Robotics." *KI — Künstliche Intelligenz*, 33(4), 319–330. DOI: [10.1007/s13218-019-00623-z](https://doi.org/10.1007/s13218-019-00623-z).

Practical encoding schemes for robotics applications with the place cell analogy. Demonstrates that HDC provides a lightweight alternative to deep learning for robotic perception tasks where training data is scarce.

---

**[Imani19]** Imani, M., Bosch, S., Karunaratne, G., Kim, Y., Montagna, F., Abu-Ghazaleh, N., & Wehbe, L. (2019). "FloatHD: Integer-Based Training Framework for Hyperdimensional Computing." *IEEE/ACM International Conference on Computer-Aided Design (ICCAD)*. DOI: [10.1109/ICCAD45719.2019.8942120](https://doi.org/10.1109/ICCAD45719.2019.8942120).

FPGA implementations achieving ~3–5 ns per comparison at 200 MHz. Demonstrates that HDC's bitwise operations map efficiently to hardware accelerators, achieving throughputs that scale linearly with parallelism.

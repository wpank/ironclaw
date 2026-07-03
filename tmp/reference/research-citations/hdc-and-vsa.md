# HDC and VSA — Hyperdimensional Computing Citations

Part of the [Research Citations](README.md) collection.

---

## Field Introduction

Hyperdimensional computing (HDC), also known as Vector Symbolic Architectures (VSA), is a computational paradigm that represents information as very high-dimensional binary or real-valued vectors (typically 1,000-10,000 dimensions). The key insight is that in high-dimensional spaces, randomly chosen vectors are nearly orthogonal, enabling binding (XOR), bundling (majority vote), and permutation operations that compose and decompose structured representations algebraically. HDC offers nanosecond comparison times, hardware efficiency (bitwise operations), and graceful degradation under noise — properties that dense neural embeddings lack. The Roko system uses 10,240-bit Binary Spatter Codes (BSC) as the universal representation substrate for all knowledge similarity, cross-domain transfer, and structural analogy detection.

**IronClaw connection**: IronClaw's workspace memory system uses hybrid FTS + vector search. HDC provides a compact, algebraically composable alternative or complement to float embeddings for knowledge retrieval and similarity scoring.

**Documents using this section**: `tmp/01-hyperdimensional-computing.md`, `tmp/10-universal-engram.md`, `tmp/11-mathematical-primitives.md`

---

## Foundational Papers

**Kanerva, P. (1988). _Sparse Distributed Memory_. Cambridge, MA: MIT Press. ISBN: 978-0262111324.**

- **Concept**: Content-addressable memory in high dimensions. In spaces with D >= 1,000, random vectors are nearly orthogonal with high probability, enabling content-addressable memory with simple bitwise operations. The book establishes capacity bounds, noise tolerance, and the mathematics of random projection.
- **Roko adaptation**: Foundational for all HDC operations. The 10,240-bit BSC dimensionality follows Kanerva's capacity analysis for the intended knowledge store size.
- **Crate**: `roko-core` (HDC fingerprint on every Signal/Engram), `roko-index` (code similarity search)
- **Roko source**: `docs/v1/21-references/09-hdc-vsa.md`, `docs/v2-depth/11-memory/02-hdc-algebra-and-retrieval.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`, `10-universal-engram.md`, `11-mathematical-primitives.md`

---

**Kanerva, P. (2009). Hyperdimensional Computing: An Introduction to Computing in Distributed Representation with High-Dimensional Random Vectors. _Cognitive Computation_, 1(2), 139-159.**
[DOI: 10.1007/s12559-009-9009-8](https://doi.org/10.1007/s12559-009-9009-8)

- **Concept**: Introduces binding (XOR), bundling (majority-vote), and permutation as the three fundamental HDC operations. Explains why 10,000-dimensional binary vectors provide sufficient capacity for practical computing, with worked examples showing compositional representation.
- **Roko adaptation**: Primary reference for the 10,240-bit BSC dimensionality choice. Every Signal carries an HDC fingerprint for similarity-based retrieval.
- **Crate**: `roko-core`, `roko-neuro`, `roko-index`
- **Roko source**: `docs/v2/08-GATEWAY.md` (line 973), `docs/v2/23-ARENAS.md` (line 370)
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

**Kleyko, D., Rachkovskij, D.A., Osipov, E., & Rahimi, A. (2022). A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures. _ACM Computing Surveys_, 55(6), Article 130.**
[DOI: 10.1145/3538531](https://doi.org/10.1145/3538531)

- **Concept**: Comprehensive VSA survey covering all major families (MAP-B, MAP-C, BSC, HRR, FHRR, VTB). Validates bundle similarity formula and capacity bounds. Provides the most rigorous comparison of VSA variants with unified mathematical notation.
- **Roko adaptation**: Validates BSC selection and the similarity formulas used in retrieval. Confirms that BSC is optimal for binary hardware and noise tolerance requirements.
- **Crate**: `roko-core`
- **Roko source**: `docs/v1/21-references/09-hdc-vsa.md`, `docs/v1/06-neuro/04-hdc-vsa-foundations.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

## Random Projection and Hashing

**Johnson, W.B. & Lindenstrauss, J. (1984). Extensions of Lipschitz Mappings into a Hilbert Space. _Contemporary Mathematics_, 26, 189-206.**

- **Concept**: The JL lemma: N points can be embedded into O(log N / epsilon^2) dimensions while preserving pairwise distances within factor (1 +/- epsilon). For epsilon=0.1 and N=100,000, the minimum dimensionality is D >= 4,604.
- **Roko adaptation**: Roko's 10,240 bits provide generous headroom beyond the JL lower bound. Mathematical foundation for projecting 1,536-dimensional LLM embeddings to 10,240-bit binary hypervectors without catastrophic information loss.
- **Crate**: `roko-core`
- **Roko source**: `docs/v1/21-references/09-hdc-vsa.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`, `11-mathematical-primitives.md`

---

**Charikar, M.S. (2002). Similarity Estimation from Rounding Algorithms. _Proceedings of the 34th Annual ACM Symposium on Theory of Computing (STOC)_, pp. 380-388.**
[DOI: 10.1145/509907.509965](https://doi.org/10.1145/509907.509965)

- **Concept**: SimHash — a single random projection h(x) = sign(w^T x) produces binary codes where collision probability equals 1 - theta/pi, relating Hamming distance to angular distance. Enables locality-sensitive hashing for approximate nearest neighbor search.
- **Roko adaptation**: Phase 1 encoding in the HDC pipeline. The projection matrix is derived deterministically from configuration for reproducibility across agent restarts.
- **Crate**: `roko-core`, `roko-index`
- **Roko source**: `docs/v1/21-references/09-hdc-vsa.md`, `docs/v2/08-GATEWAY.md` (convergence detection via SimHash)
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

## Advanced HDC

**Plate, T.A. (1994). Distributed Representations and Nested Compositional Structure. PhD Dissertation, University of Toronto. [Also published as: Plate, T.A. (2003). _Holographic Reduced Representations: Distributed Representation for Cognitive Structures_. CSLI Publications. ISBN: 978-1575864242.]**

- **Concept**: Holographic Reduced Representations (HRR) using circular convolution for binding. Theoretical ancestor of BSC. Proves that structured compositional representations (trees, sequences, graphs) can be encoded in fixed-width vectors.
- **Crate**: `roko-core` (BSC is the binary descendant of HRR)
- **Roko source**: `docs/v1/21-references/09-hdc-vsa.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

**Rachkovskij, D.A. (2001). Representation and Processing of Structures with Binary Sparse Distributed Codes. _Knowledge-Based Systems_, 14(1-2), 71-77.**
[DOI: 10.1016/S0950-7051(00)00098-5](https://doi.org/10.1016/S0950-7051(00)00098-5)

- **Concept**: Binary sparse distributed codes for structured knowledge representation. Formal treatment of binding and bundling operations in sparse binary vectors.
- **Roko adaptation**: Foundational for BSC binding operations used throughout the codebase.
- **Roko source**: `docs/v2/01-SIGNAL.md` (line 1482), `docs/v1/20-technical-analysis/06-hyperdimensional-ta.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

**Levy, S.D. & Gayler, R.W. (2008). Vector Symbolic Architectures: A New Building Block for Artificial General Intelligence. _Proceedings of the 2008 Conference on Artificial General Intelligence_, pp. 414-418.**

- **Concept**: Comprehensive survey establishing VSA as a computational paradigm distinct from neural networks and symbolic AI, with applications in analogy, question answering, and cognitive modeling.
- **Roko source**: `docs/v2/01-SIGNAL.md` (line 1483), `docs/v2/06-MEMORY.md` (line 515)
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

**Frady, E.P., Kent, S.J., Olshausen, B.A., & Sommer, F.T. (2020). Resonator Networks, 1: An Efficient Solution for Factoring High-Dimensional, Distributed Representations of Data Structures. _Neural Computation_, 32(12), 2311-2331.**
[DOI: 10.1162/neco_a_01331](https://doi.org/10.1162/neco_a_01331)

- **Concept**: Iterative convergence method for HDC retrieval that factorizes bundled vectors to recover constituents without exhaustive search. More space-efficient than codebook lookup for large dictionaries.
- **Roko adaptation**: Referenced as a future optimization path for scaled retrieval. Used in dream consolidation pipeline to identify patterns learned separately.
- **Roko source**: `docs/v2/01-SIGNAL.md` (line 1485), `docs/v2/06-MEMORY.md` (line 470), `docs/v2/07-LEARNING.md` (line 826)
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

**Ganesan, A., Gao, H., Gandhi, S., Raff, E., Oates, T., Holt, J., & McLean, M. (2021). Learning with Holographic Reduced Representations. _Advances in Neural Information Processing Systems (NeurIPS)_ (Spotlight).**
[arXiv:2109.02157](https://arxiv.org/abs/2109.02157)

- **Concept**: Made HRR viable as differentiable deep learning components via a projection step forcing vectors into a well-behaved subspace, solving numerical instability. Demonstrates 100x retrieval improvement over baseline HRR implementations.
- **Roko adaptation**: Bridge paper enabling end-to-end learning with HDC representations in neural settings.
- **Roko source**: `docs/v1/21-references/09-hdc-vsa.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

**Malkov, Y.A. & Yashunin, D.A. (2020). Efficient and Robust Approximate Nearest Neighbor Search Using Hierarchical Navigable Small World Graphs. _IEEE Transactions on Pattern Analysis and Machine Intelligence_, 42(4), 824-836.**
[DOI: 10.1109/TPAMI.2018.2889473](https://doi.org/10.1109/TPAMI.2018.2889473)

- **Concept**: HNSW: O(log N) approximate nearest neighbor search at 95-99% recall for billion-scale vectors, using a hierarchical graph with long-range connections at upper layers and short-range connections at lower layers.
- **Roko adaptation**: Production search infrastructure for HDC index at scale.
- **Roko source**: `docs/v1/21-references/09-hdc-vsa.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

**Olshausen, B.A. & Field, D.J. (1996). Emergence of Simple-Cell Receptive Field Properties by Learning a Sparse Code for Natural Images. _Nature_, 381, 607-609.**
[DOI: 10.1038/381607a0](https://doi.org/10.1038/381607a0)

- **Concept**: Visual cortex neurons learn sparse representations of natural images under constraints of overcomplete dictionaries, producing Gabor-like receptive fields. Biological validation for sparse distributed representations.
- **Roko source**: `docs/v2/01-SIGNAL.md` (line 1486), `docs/v2/06-MEMORY.md` (line 515)
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

**Rahimi, A. et al. (2024). HDC: A Framework for Stochastic Computation and Symbolic AI. _Journal of Big Data_.**

- **Concept**: Unified framework positioning HDC as both stochastic computation substrate and symbolic AI system. Validates BSC as a general-purpose knowledge representation format.
- **Roko source**: `docs/v1/21-references/24-additions-2025.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

---

**FLASH (2024). Hyperdimensional Computing with Holographic and Adaptive Encoder. _Frontiers in Artificial Intelligence_, 7.**
[DOI: 10.3389/frai.2024.1371988](https://doi.org/10.3389/frai.2024.1371988)

- **Concept**: Gradient-descent-based adaptive encoder for HDC, bridging fixed encoding (fast, no training) and learned encoding (slower, better performance) phases. Shows adaptive encoding significantly outperforms fixed encoding on structured tasks.
- **Roko source**: `docs/v1/21-references/24-additions-2025.md`, `docs/v1/06-neuro/16-current-status-and-gaps.md`

---

*Navigation: [README](README.md) | [Memory and Learning](memory-and-learning.md) | [Affect and Cognition](affect-and-cognition.md) | [Verification and Safety](verification-and-safety.md) | [Agents and Orchestration](agents-and-orchestration.md) | [Blockchain and Economics](blockchain-and-economics.md) | [Context and Search](context-and-search.md) | [Math and Statistics](math-and-statistics.md)*

# Roko Research Citations: Comprehensive Bibliography

This document catalogs every academic and technical reference found across the Roko codebase -- a cognitive AI agent framework for autonomous software development. Roko is not a conventional LLM wrapper; it is a system architecture drawing from neuroscience, economics, topology, cybernetics, and philosophy to build agents that learn, feel, forget, dream, and cooperate. Every architectural decision traces to published research.

This bibliography is organized by topic area. For each reference, the document records: the original paper with DOI/arXiv link where available, the concept it introduces, how Roko adapts it, which crate implements it, and which documents in the `tmp/` collection reference it. A Key Papers section at the end highlights the 10-15 most foundational works.

---

## Table of Contents

1. [Hyperdimensional Computing (HDC / VSA)](#1-hyperdimensional-computing-hdc--vsa)
2. [Dream Consolidation and Offline Learning](#2-dream-consolidation-and-offline-learning)
3. [Affect, Emotion, and Somatic Markers](#3-affect-emotion-and-somatic-markers)
4. [Active Inference and Free Energy Principle](#4-active-inference-and-free-energy-principle)
5. [Contextual Bandits and Model Routing](#5-contextual-bandits-and-model-routing)
6. [Statistical Process Control (SPC)](#6-statistical-process-control-spc)
7. [Topological Data Analysis (TDA)](#7-topological-data-analysis-tda)
8. [VCG Auction and Mechanism Design](#8-vcg-auction-and-mechanism-design)
9. [Stigmergic Coordination](#9-stigmergic-coordination)
10. [Memory Consolidation and Forgetting](#10-memory-consolidation-and-forgetting)
11. [Cybernetics and Systems Theory](#11-cybernetics-and-systems-theory)
12. [Cognitive Architectures](#12-cognitive-architectures)
13. [Collective Intelligence](#13-collective-intelligence)
14. [Dual-Process Cognition](#14-dual-process-cognition)
15. [Evolutionary and Generational Dynamics](#15-evolutionary-and-generational-dynamics)
16. [Yerkes-Dodson Law](#16-yerkes-dodson-law)
17. [Ebbinghaus Forgetting Curve](#17-ebbinghaus-forgetting-curve)
18. [Gesellian Demurrage](#18-gesellian-demurrage)
19. [FIPA Agent Lifecycle](#19-fipa-agent-lifecycle)
20. [Prospect Theory and Loss Aversion](#20-prospect-theory-and-loss-aversion)
21. [Cognitive Energy and Fatigue](#21-cognitive-energy-and-fatigue)
22. [Context Engineering](#22-context-engineering)
23. [Security, Safety, and Provenance](#23-security-safety-and-provenance)
24. [Philosophy of Agency](#24-philosophy-of-agency)
25. [Protocol Standards and Blockchain](#25-protocol-standards-and-blockchain)
26. [Market Microstructure](#26-market-microstructure)
27. [Temporal Knowledge and Causal Reasoning](#27-temporal-knowledge-and-causal-reasoning)
28. [Emergent Goals and Intrinsic Motivation](#28-emergent-goals-and-intrinsic-motivation)
29. [Biological Analogues](#29-biological-analogues)
30. [Self-Learning Systems](#30-self-learning-systems)
31. [Information Theory and Signal Processing](#31-information-theory-and-signal-processing)
32. [Calibration and Uncertainty](#32-calibration-and-uncertainty)
33. [Streaming Algorithms](#33-streaming-algorithms)
34. [Agent Harnesses and Tool Use](#34-agent-harnesses-and-tool-use)
35. [Process Reward Models and Verification](#35-process-reward-models-and-verification)
36. [Reinforcement Learning Foundations](#36-reinforcement-learning-foundations)
37. [IIT and Consciousness Metrics](#37-iit-and-consciousness-metrics)

- [Key Papers](#key-papers)

---

## 1. Hyperdimensional Computing (HDC / VSA)

Roko uses 10,240-bit Binary Spatter Codes as the universal representation substrate for knowledge similarity, cross-domain transfer, and structural analogy detection. HDC provides three algebraic operations -- XOR binding, majority-vote bundling, and cyclic-shift permutation -- that compose knowledge representations in nanoseconds on commodity hardware. A single comparison costs approximately 13 ns (XOR + POPCNT on 160 u64 words).

### Foundational Papers

**Kanerva, P. (1988). _Sparse Distributed Memory_. MIT Press.**
- **Concept**: Content-addressable memory in high dimensions. In spaces with D >= 1,000, random vectors are nearly orthogonal with high probability, enabling content-addressable memory with simple bitwise operations.
- **Roko adaptation**: Foundational for all HDC operations. The 10,240-bit BSC dimensionality follows Kanerva's capacity analysis.
- **Crate**: `roko-core` (HDC fingerprint on every Signal/Engram), `roko-index` (code similarity search)
- **Roko files**: `docs/v1/21-references/09-hdc-vsa.md`, `docs/v2-depth/11-memory/02-hdc-algebra-and-retrieval.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`, `10-universal-engram.md`, `11-mathematical-primitives.md`
- **IronClaw relevance**: IronClaw's workspace memory system uses hybrid search (FTS + vector). HDC provides a compact, algebraically composable alternative to float embeddings for knowledge retrieval.

**Kanerva, P. (2009). Hyperdimensional Computing: An Introduction to Computing in Distributed Representation with High-Dimensional Random Vectors. _Cognitive Computation_, 1(2), 139-159.** [DOI: 10.1007/s12559-009-9009-8](https://doi.org/10.1007/s12559-009-9009-8)
- **Concept**: Introduces binding, bundling, permutation as the three HDC operations. Explains why 10,000-dimensional binary vectors provide sufficient capacity for practical computing.
- **Roko adaptation**: Primary reference for the 10,240-bit BSC dimensionality choice. Every Signal carries an HDC fingerprint for similarity-based retrieval.
- **Crate**: `roko-core`, `roko-neuro`, `roko-index`
- **Roko files**: `docs/v2/08-GATEWAY.md` (line 973), `docs/v2/23-ARENAS.md` (line 370), `docs/v2-depth/21-roadmap/07-academic-foundations-by-protocol.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

**Kleyko, D., Rachkovskij, D.A., Osipov, E., & Rahimi, A. (2022). A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures. _ACM Computing Surveys_, 55(6), Article 130.** [DOI: 10.1145/3538531](https://doi.org/10.1145/3538531)
- **Concept**: Comprehensive VSA survey covering all major families (MAP-B, MAP-C, BSC, HRR, FHRR, VTB). Validates the bundle similarity formula and capacity bounds.
- **Roko adaptation**: Validates BSC selection and the similarity formulas used in retrieval.
- **Crate**: `roko-core`
- **Roko files**: `docs/v1/21-references/09-hdc-vsa.md`, `docs/v1/06-neuro/04-hdc-vsa-foundations.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

### Random Projection and Hashing

**Johnson, W.B. & Lindenstrauss, J. (1984). Extensions of Lipschitz Mappings into a Hilbert Space. _Contemporary Mathematics_, 26, 189-206.**
- **Concept**: The JL lemma: N points can be projected into O(log N / epsilon^2) dimensions while preserving pairwise distances within (1 +/- epsilon). For epsilon=0.1 and N=100,000, D >= 4,604.
- **Roko adaptation**: Roko's 10,240 bits provide generous headroom beyond the JL lower bound. Mathematical foundation for projecting 1,536-dim LLM embeddings to 10,240-bit binary hypervectors.
- **Crate**: `roko-core`
- **Roko files**: `docs/v1/21-references/09-hdc-vsa.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`, `11-mathematical-primitives.md`

**Charikar, M.S. (2002). Similarity Estimation from Rounding Algorithms. _Proceedings of the 34th Annual ACM Symposium on Theory of Computing (STOC)_, 380-388.** [DOI: 10.1145/509907.509965](https://doi.org/10.1145/509907.509965)
- **Concept**: SimHash -- single random projection h(x) = sign(w^T x) produces binary codes where collision probability = 1 - theta/pi.
- **Roko adaptation**: Phase 1 encoding in the HDC pipeline. The projection matrix is derived deterministically from configuration for reproducibility.
- **Crate**: `roko-core`, `roko-index`
- **Roko files**: `docs/v1/21-references/09-hdc-vsa.md`, `docs/v2/08-GATEWAY.md` (convergence detection via SimHash)
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

### Advanced HDC

**Plate, T.A. (1994). Distributed Representations and Nested Compositional Structure. PhD Dissertation, University of Toronto.**
- **Concept**: Holographic Reduced Representations (HRR) using circular convolution for binding. Theoretical ancestor of BSC. Later published as _Holographic Reduced Representations: Distributed Representation for Cognitive Structures_, CSLI Publications, 2003.
- **Crate**: `roko-core` (BSC is the binary descendant of HRR)
- **Roko files**: `docs/v1/21-references/09-hdc-vsa.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

**Rachkovskij, D.A. (2001). Representation and Processing of Structures with Binary Sparse Distributed Codes. _Knowledge-Based Systems_, 14(1-2), 71-77.**
- **Concept**: Binary sparse distributed codes for structured knowledge representation. Formal treatment of binding and bundling operations in sparse binary vectors.
- **Roko adaptation**: Foundational for BSC binding operations used throughout the codebase.
- **Roko files**: `docs/v2/01-SIGNAL.md` (line 1482), `docs/v1/20-technical-analysis/06-hyperdimensional-ta.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

**Levy, S.D. & Gayler, R.W. (2008). Vector Symbolic Architectures: A New Building Block for Artificial General Intelligence. _Proceedings of the 2008 Conference on Artificial General Intelligence_, 414-418.**
- **Concept**: Comprehensive survey of vector symbolic architectures, establishing VSA as a computational paradigm distinct from neural networks and symbolic AI.
- **Roko files**: `docs/v2/01-SIGNAL.md` (line 1483), `docs/v2/06-MEMORY.md` (line 515)
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

**Frady, E.P., Kent, S.J., Olshausen, B.A., & Sommer, F.T. (2020). Resonator Networks, 1: An Efficient Solution for Factoring High-Dimensional, Distributed Representations of Data Structures. _Neural Computation_, 32(12), 2311-2331.** [DOI: 10.1162/neco_a_01331](https://doi.org/10.1162/neco_a_01331)
- **Concept**: Iterative convergence method for HDC retrieval that factorizes bundled vectors to recover constituents. More space-efficient than exhaustive search for very large dictionaries.
- **Roko adaptation**: Referenced as a future optimization path for scaled retrieval. Used in the dream consolidation pipeline to identify patterns learned separately.
- **Roko files**: `docs/v2/01-SIGNAL.md` (line 1485), `docs/v2/06-MEMORY.md` (line 470), `docs/v2/07-LEARNING.md` (line 826)
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

**Ganesan, A., Gao, H., Gandhi, S., Raff, E., Oates, T., Holt, J., & McLean, M. (2021). Learning with Holographic Reduced Representations. _NeurIPS_ (Spotlight).** [arXiv:2109.02157](https://arxiv.org/abs/2109.02157)
- **Concept**: Made HRR viable as differentiable deep learning components via a projection step that forces vectors to exist in a well-behaved subspace, solving numerical instability. +100x retrieval improvement.
- **Roko adaptation**: Bridge paper enabling end-to-end learning with HDC representations.
- **Roko files**: `docs/v1/21-references/09-hdc-vsa.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

**Malkov, Y.A. & Yashunin, D.A. (2020). Efficient and Robust Approximate Nearest Neighbor Search Using Hierarchical Navigable Small World Graphs. _IEEE TPAMI_, 42(4), 824-836.** [DOI: 10.1109/TPAMI.2018.2889473](https://doi.org/10.1109/TPAMI.2018.2889473)
- **Concept**: HNSW: O(log N) search at 95-99% recall for billion-scale vectors.
- **Roko adaptation**: Production search infrastructure for HDC index at scale.
- **Roko files**: `docs/v1/21-references/09-hdc-vsa.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

**Olshausen, B.A. & Field, D.J. (1996). Emergence of Simple-Cell Receptive Field Properties by Learning a Sparse Code for Natural Images. _Nature_, 381, 607-609.** [DOI: 10.1038/381607a0](https://doi.org/10.1038/381607a0)
- **Concept**: Biological precedent for sparse coding: visual cortex neurons learn sparse representations of natural images. Biological validation for sparse distributed representations.
- **Roko files**: `docs/v2/01-SIGNAL.md` (line 1486), `docs/v2/06-MEMORY.md` (line 515)
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

**Rahimi, A. et al. (2024). HDC: A Framework for Stochastic Computation and Symbolic AI. _Journal of Big Data_.**
- **Concept**: Unified HDC as both stochastic computation framework and symbolic AI system. Validates BSC as a general-purpose substrate.
- **Roko files**: `docs/v1/21-references/24-additions-2025.md`
- **tmp/ cross-refs**: `01-hyperdimensional-computing.md`

**FLASH (2024). Hyperdimensional Computing with Holographic and Adaptive Encoder. _Frontiers in AI_.** [DOI: 10.3389/frai.2024.1371988](https://doi.org/10.3389/frai.2024.1371988)
- **Concept**: Gradient-descent-based adaptive encoder for HDC, bridging fixed and learned encoding phases.
- **Roko files**: `docs/v1/21-references/24-additions-2025.md`, `docs/v1/06-neuro/16-current-status-and-gaps.md`

---

## 2. Dream Consolidation and Offline Learning

Roko agents enter a "Delta" consolidation phase during idle periods -- analogous to biological sleep. The dream cycle has three phases: NREM replay (prioritized experience replay), REM imagination (counterfactual hypothesis generation), and integration staging. The brain dedicates 25-33% of its runtime to a state preventing environmental interaction; Roko treats idle time as compute budget for offline cognitive work.

### Replay and Prioritization

**Mattar, M.G. & Daw, N.D. (2018). Prioritized Memory Access Explains Planning and Hippocampal Replay. _Nature Neuroscience_, 21(11), 1609-1617.** [DOI: 10.1038/s41593-018-0232-z](https://doi.org/10.1038/s41593-018-0232-z)
- **Concept**: Utility = gain x need for replay selection. Episodes are replayed in order of their utility for future decisions, not their recency. Unifies planning, learning, and consolidation as different consequences of prioritized replay.
- **Roko adaptation**: The foundational algorithm for selecting which episodes to replay during Delta consolidation. The dream cycle replays episodes ordered by prediction error magnitude.
- **Crate**: `roko-dreams` (replay module)
- **Roko files**: `crates/roko-dreams/README.md` (line 18), `docs/v2/26-CROSS-CUTS.md` (lines 413, 432, 819)
- **tmp/ cross-refs**: `02-dream-consolidation.md`, `13-cognitive-architecture.md`
- **IronClaw relevance**: IronClaw's heartbeat system runs periodic consolidation. Mattar-Daw prioritization would optimize which experiences are worth replaying during these background cycles.

**Wilson, M.A. & McNaughton, B.L. (1994). Reactivation of Hippocampal Ensemble Memories During Sleep. _Science_, 265(5172), 676-679.** [DOI: 10.1126/science.8036517](https://doi.org/10.1126/science.8036517)
- **Concept**: First demonstration that hippocampal place cells reactivate during sleep in the same sequential order experienced during waking behavior. Foundation for computational replay.
- **Roko adaptation**: Directly implements prioritized experience replay in the Dreams subsystem (NREM phase).
- **Roko files**: `docs/v1/21-references/01-memory-consolidation.md`
- **tmp/ cross-refs**: `02-dream-consolidation.md`

**Schaul, T., Quan, J., Antonoglou, I., & Silver, D. (2016). Prioritized Experience Replay. _ICLR_.** [arXiv:1511.05952](https://arxiv.org/abs/1511.05952)
- **Concept**: Priority proportional to TD error magnitude. Prioritizing important transitions leads to more efficient learning, outperforming uniform replay on 41 out of 49 Atari games.
- **Roko adaptation**: Surprise-weighted replay candidate selection in the dream consolidation engine.
- **Roko files**: `docs/v1/21-references/01-memory-consolidation.md`
- **tmp/ cross-refs**: `02-dream-consolidation.md`

**Jensen, K.T., Hennequin, G., & Mattar, M.G. (2024). A Recurrent Network Model of Planning Explains Hippocampal Replay and Human Behavior. _Nature Neuroscience_, 27, 1340-1348.** [DOI: 10.1038/s41593-024-01675-7](https://doi.org/10.1038/s41593-024-01675-7)
- **Concept**: Recurrent network model unifying planning and replay. Hippocampal replay emerges from goal-directed planning computations in a recurrent network.
- **Roko adaptation**: Validates that replay and planning share computational substrate; Roko's dream cycle serves both functions.
- **Roko files**: `docs/v1/10-dreams/02-nrem-replay.md` (line 1059)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

### Emotional Depotentiation

**Walker, M.P. & van der Helm, E. (2009). Overnight Therapy: The Role of Sleep in Emotional Brain Processing. _Psychological Bulletin_, 135(5), 731-748.** [DOI: 10.1037/a0016570](https://doi.org/10.1037/a0016570)
- **Concept**: REM sleep depotentiates emotional charge while preserving informational content. Sleep provides "overnight therapy."
- **Roko adaptation**: Dream cycles reduce arousal on highly charged memories by 0.3-0.5 per cycle to prevent panic lock-in. This is a REM-phase operation in the dream cycle.
- **Crate**: `roko-dreams`, `roko-daimon`
- **Roko files**: `docs/v2/26-CROSS-CUTS.md` (lines 455, 496), `docs/v2-depth/07-agent-runtime/18-affect-as-functor.md` (line 954)
- **tmp/ cross-refs**: `02-dream-consolidation.md`, `03-affect-engine.md`

### Creativity and Counterfactuals

**Boden, M.A. (2004). _The Creative Mind: Myths and Mechanisms_. 2nd ed. Routledge.**
- **Concept**: Three creativity modes: exploratory (within rules), combinational (new combinations), transformational (changing rules).
- **Roko adaptation**: REM phase implements combinational creativity via HDC recombination and transformational creativity via causal model intervention.
- **Crate**: `roko-dreams` (imagination module)
- **Roko files**: `docs/v1/21-references/03-dreams-and-offline-learning.md`
- **tmp/ cross-refs**: `02-dream-consolidation.md`

**Pearl, J. (2009). _Causality: Models, Reasoning, and Inference_. 2nd ed. Cambridge University Press.**
- **Concept**: Structural causal models enable counterfactual reasoning via the do-calculus. Interventions on causal variables generate "what if" scenarios.
- **Roko adaptation**: REM dreaming generates counterfactuals by intervening on causal variables: "what would have happened if I had chosen differently?"
- **Crate**: `roko-dreams`
- **Roko files**: `docs/v1/21-references/03-dreams-and-offline-learning.md`, `docs/v2/26-CROSS-CUTS.md` (line 455)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

### Sleep-Time Compute

**Lin, K., Snell, C., Wang, Y., Packer, C., Wooders, S., Stoica, I., & Gonzalez, J.E. (2025). Sleep-time Compute: Beyond Inference Scaling at Test-Time.** [arXiv:2504.13171](https://arxiv.org/abs/2504.13171)
- **Concept**: Dual-agent architecture: Sleeper Agent precomputes during downtime, Serve Agent handles live interactions. ~5x test-time compute reduction on Stateful GSM-Symbolic benchmarks.
- **Roko adaptation**: Dream cycles are sleep-time compute -- agents process experiences during low-activity periods to reduce latency and improve accuracy at inference.
- **Roko files**: `docs/v1/21-references/03-dreams-and-offline-learning.md`
- **tmp/ cross-refs**: `02-dream-consolidation.md`

**Ha, D. & Schmidhuber, J. (2018). World Models.** [arXiv:1803.10122](https://arxiv.org/abs/1803.10122)
- **Concept**: Controller trained entirely inside learned world models ("dreams") achieves competitive performance on RL benchmarks without interacting with the real environment.
- **Roko adaptation**: Dreaming multiplies learning from scarce experience. Idle-time knowledge recombination creates new hypotheses without real execution.
- **Roko files**: `docs/v1/21-references/03-dreams-and-offline-learning.md`
- **tmp/ cross-refs**: `02-dream-consolidation.md`

**Hafner, D., Pasukonis, J., Ba, J., & Lillicrap, T. (2025). Mastering Diverse Control Tasks Through World Models. _Nature_, 640, 647-653.** [DOI: 10.1038/s41586-025-08744-2](https://doi.org/10.1038/s41586-025-08744-2)
- **Concept**: DreamerV3: agents trained on imagined trajectories outperform specialized methods across 150+ tasks. First algorithm to collect diamonds in Minecraft from scratch without human data.
- **Roko adaptation**: REM-phase consolidation generates synthetic scenarios from the causal model.
- **Roko files**: `docs/v1/10-dreams/03-rem-imagination.md` (line 1123)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

### Hypnagogia

**Lacaux, C., Andrillon, T., Bastoul, C., Idir, Y., Fonteix-Galet, A., Arnulf, I., & Oudiette, D. (2021). Sleep Onset is a Creative Sweet Spot. _Science Advances_, 7(50), eabj5866.** [DOI: 10.1126/sciadv.abj5866](https://doi.org/10.1126/sciadv.abj5866)
- **Concept**: 83% of subjects spending at least 15 seconds in N1 (sleep onset) discovered hidden rules vs. 30% staying awake. The effect vanishes in deeper sleep.
- **Roko adaptation**: Foundational for the hypnagogia engine, which solves the Alpha Convergence Problem (all agents converging to similar solutions).
- **Crate**: `roko-dreams` (hypnagogia module)
- **Roko files**: `docs/v1/21-references/03-dreams-and-offline-learning.md`
- **tmp/ cross-refs**: `02-dream-consolidation.md`

**Horowitz, A.H. et al. (2023). Targeted Dream Incubation at Sleep Onset Increases Post-Sleep Creative Performance. _Scientific Reports_, 13.** [DOI: 10.1038/s41598-023-31361-w](https://doi.org/10.1038/s41598-023-31361-w)
- **Concept**: Targeted dream incubation (TDI) at sleep onset enhances post-sleep creative performance on incubated topics.
- **Roko adaptation**: Validates the targeted consolidation approach in the hypnagogia engine.
- **Roko files**: `docs/v1/10-dreams/07-hypnagogia-engine.md` (line 864)
- **tmp/ cross-refs**: `02-dream-consolidation.md`

### Sleep-Inspired LLM Research (2025)

**NeuroDream (2025). SSRN:5377250.**
- **Concept**: Dream-phase consolidation achieves 38% forgetting reduction and 17.6% zero-shot transfer increase in LLM agents.
- **Roko files**: `docs/v1/21-references/24-additions-2025.md`
- **tmp/ cross-refs**: `02-dream-consolidation.md`

**SleepGate (2025).** [arXiv:2603.14517](https://arxiv.org/abs/2603.14517)
- **Concept**: Active forgetting via learned curation resolving proactive interference. Validates that forgetting is a feature, not a bug.
- **Roko files**: `docs/v1/21-references/24-additions-2025.md`
- **tmp/ cross-refs**: `02-dream-consolidation.md`

**LightMem (2025).** [arXiv:2510.18866](https://arxiv.org/abs/2510.18866)
- **Concept**: Offline consolidation achieving 10.9% accuracy gain with 117x token reduction.
- **Roko files**: `docs/v1/21-references/24-additions-2025.md`
- **tmp/ cross-refs**: `02-dream-consolidation.md`

**Sawada, T. et al. (2024). Prefrontal Synaptic Regulation of Homeostatic Sleep Pressure. _Science_.** [DOI: 10.1126/science.adl3043](https://doi.org/10.1126/science.adl3043)
- **Concept**: Prefrontal cortex synaptic mechanisms directly regulate sleep homeostatic pressure.
- **Roko files**: `docs/v1/10-dreams/04-consolidation-and-staging.md` (line 286)

---

## 3. Affect, Emotion, and Somatic Markers

The Daimon is Roko's affect engine -- not cosmetic, but architectural. Five independent research lines converge on the conclusion that emotion-like states serve genuine computational functions. The Daimon uses a PAD (Pleasure-Arousal-Dominance) vector, the ALMA three-layer temporal model, and a somatic landscape to bias every decision the agent makes.

### PAD Model

**Mehrabian, A. (1996). Pleasure, Arousal, Dominance: A General Framework for Describing and Measuring Individual Differences in Temperament. _Current Psychology_, 14(4), 261-292.**
- **Concept**: Three continuous dimensions (Pleasure, Arousal, Dominance) capture more emotional variance than discrete labels. The 8 PAD octants map to behavioral states.
- **Roko adaptation**: Daimon state vector. Every Signal carries an optional PAD stamp. The 8 octants map to: Exuberant, Dependent, Relaxed, Docile, Hostile, Anxious, Disdainful, Depressed.
- **Crate**: `roko-daimon`, `roko-core`
- **Roko files**: `docs/v2/26-CROSS-CUTS.md` (line 226), `docs/v2-depth/07-agent-runtime/18-affect-as-functor.md`, `docs/v2/05-AGENT.md` (line 969)
- **tmp/ cross-refs**: `03-affect-engine.md`, `13-cognitive-architecture.md`

**Russell, J.A. & Mehrabian, A. (1977). Evidence for a Three-Factor Theory of Emotions. _Journal of Research in Personality_, 11(3), 273-294.**
- **Concept**: Empirical validation of the three-dimensional affect model through factor analysis of emotional self-reports.
- **Roko files**: `docs/v1/21-references/02-affective-computing.md`
- **tmp/ cross-refs**: `03-affect-engine.md`

### Somatic Markers

**Damasio, A.R. (1994). _Descartes' Error: Emotion, Reason, and the Human Brain_. Putnam.**
- **Concept**: Somatic markers: patients without emotional capacity make consistently worse decisions under uncertainty. Emotional signals from body-mapped memories guide decision-making before deliberation.
- **Roko adaptation**: SomaticLandscape implemented as a k-d tree over 8-dimensional strategy space. Past emotional experiences (success/failure at specific strategy coordinates) bias future decisions before analytical reasoning begins.
- **Crate**: `roko-daimon` (somatic_ta.rs), `roko-core`
- **Roko files**: `crates/roko-daimon/src/somatic_ta.rs` (line 15), `docs/v2/05-AGENT.md` (line 969)
- **tmp/ cross-refs**: `03-affect-engine.md`, `13-cognitive-architecture.md`

**Bechara, A., Damasio, H., & Damasio, A.R. (2000). Emotion, Decision Making and the Orbitofrontal Cortex. _Cerebral Cortex_, 10(3), 295-307.** [DOI: 10.1093/cercor/10.3.295](https://doi.org/10.1093/cercor/10.3.295)
- **Concept**: Anticipatory skin conductance responses precede conscious awareness in the Iowa Gambling Task. Pre-cognitive "gut feelings" guide decisions.
- **Roko adaptation**: SomaticLandscape provides fast heuristic feelings about strategy regions before analytical reasoning.
- **Crate**: `roko-daimon`
- **tmp/ cross-refs**: `03-affect-engine.md`

### OCC and Scherer Appraisal

**Ortony, A., Clore, G.L., & Collins, A. (1988). _The Cognitive Structure of Emotions_. Cambridge University Press.**
- **Concept**: OCC emotion model: cognitive appraisal-based emotion taxonomy. Emotions arise from evaluating events against goals, standards, and attitudes.
- **Roko adaptation**: Complements PAD with a structure-of-emotion framework. The appraisal pipeline uses OCC-Scherer steps.
- **Crate**: `roko-daimon`
- **Roko files**: `docs/v1/21-references/02-affective-computing.md`, `docs/v2-depth/07-agent-runtime/18-affect-as-functor.md` (line 126)
- **tmp/ cross-refs**: `03-affect-engine.md`

**Scherer, K.R. (2001). Appraisal Considered as a Process of Multi-Level Sequential Checking. In _Appraisal Processes in Emotion_, Oxford University Press.**
- **Concept**: Multi-level sequential checking: relevance, implication, coping potential, normative significance. Mirrors the multi-axis scoring on Engrams.
- **Roko adaptation**: The 8-step appraisal pipeline (Classify, Ground, Scale, Compute, Decay, Apply, Persist, Emit) is OCC-Scherer.
- **Crate**: `roko-daimon`
- **tmp/ cross-refs**: `03-affect-engine.md`

### ALMA Temporal Model

**Gebhard, P. (2005). ALMA -- A Layered Model of Affect. _AAMAS_, 2005.**
- **Concept**: Three-layer temporal affect: emotion (seconds), mood (hours), personality (lifetime). Each layer operates at a different timescale with different time constants.
- **Roko adaptation**: Implemented as three nested loops. Emotion layer: tau=0.1 (fast reaction per event). Mood layer: tau=0.5 (smooths volatility, fires every 10 ticks). Personality layer: tau=0.9 (near-static baseline, fires every 100 ticks). Effective PAD = 0.5*emotion + 0.3*mood + 0.2*personality.
- **Crate**: `roko-daimon`
- **Roko files**: `docs/v2-depth/07-agent-runtime/18-affect-as-functor.md` (line 480)
- **tmp/ cross-refs**: `03-affect-engine.md`

### Mood-Congruent Memory

**Bower, G.H. (1981). Mood and Memory. _American Psychologist_, 36(2), 129-148.**
- **Concept**: Emotional states bias memory retrieval via associative network activation. Happy moods retrieve happy memories; sad moods retrieve sad ones.
- **Roko adaptation**: Implemented as the emotional factor (0.15 weight) in four-factor retrieval scoring. The mandatory 15% contrarian retrieval prevents echo chambers.
- **Crate**: `roko-daimon`, `roko-neuro`
- **Roko files**: `crates/roko-daimon/src/somatic_ta.rs` (line 20)
- **tmp/ cross-refs**: `03-affect-engine.md`

### Affect Validation

**Zhang, Y. et al. (2024). Self-Emotion Changes ~50% of Decisions. _SIGDIAL_.**
- **Concept**: Affect is the primary driver of agent behavior, not a display layer. Approximately 50% of decisions change based on emotional state.
- **Roko adaptation**: Validates that the Daimon is architectural, not decorative. Roko treats affect as a first-class cross-cutting concern.
- **Roko files**: `docs/v1/21-references/02-affective-computing.md`
- **tmp/ cross-refs**: `03-affect-engine.md`

### Learned Helplessness

**Seligman, M.E.P. (1972). Learned Helplessness. _Annual Review of Medicine_, 23, 407-412.**
- **Concept**: Uncontrollable negative outcomes produce learned helplessness -- resignation and passivity that persists even when control becomes available.
- **Roko adaptation**: Dominance < -0.3 for 200+ ticks triggers an alert in the Daimon behavioral state machine.
- **Crate**: `roko-daimon`
- **tmp/ cross-refs**: `03-affect-engine.md`

---

## 4. Active Inference and Free Energy Principle

Active inference provides the principled answer to "how should an agent decide what to attend to?" Expected Free Energy (EFE) decomposes into pragmatic value (goal achievement) and epistemic value (information gain), resolving the exploration-exploitation dilemma without ad-hoc hyperparameters.

**Friston, K. (2006). A Free Energy Principle for the Brain. _Journal of Physiology-Paris_, 100(1-3), 70-87.** [DOI: 10.1016/j.jphysparis.2006.10.001](https://doi.org/10.1016/j.jphysparis.2006.10.001)
- **Concept**: All self-organizing biological systems minimize variational free energy. The foundational principle for prediction-error-driven cognition.
- **Roko adaptation**: The core routing principle. Every tier selection (T0/T1/T2) and context selection decision is driven by EFE minimization. Replaces LinUCB bandits in the CascadeRouter.
- **Crate**: `roko-learn` (routing), `roko-orchestrator`, `roko-core`
- **Roko files**: `docs/v2-depth/07-agent-runtime/dual-process-and-efe-routing.md`
- **tmp/ cross-refs**: `07-online-learning.md`, `13-cognitive-architecture.md`

**Friston, K. et al. (2015). Active Inference and Epistemic Value. _Cognitive Neuroscience_, 6(4), 187-214.** [DOI: 10.1080/17588928.2015.1020053](https://doi.org/10.1080/17588928.2015.1020053)
- **Concept**: EFE = pragmatic_value + epistemic_value. Agents naturally seek information when uncertain and exploit knowledge when confident. The balance emerges from the mathematics.
- **Roko adaptation**: High-epistemic-value knowledge gets prioritized when the agent is uncertain; high-pragmatic-value knowledge dominates when confident.
- **Roko files**: `docs/v2/02-CELL.md` (line 1719), `docs/v2-depth/02-block/active-inference-context-selection.md`
- **tmp/ cross-refs**: `07-online-learning.md`, `13-cognitive-architecture.md`

**Millidge, B., Tschantz, A., & Buckley, C.L. (2021). Whence the Expected Free Energy? _Neural Computation_, 33(2), 447-482.** [DOI: 10.1162/neco_a_01354](https://doi.org/10.1162/neco_a_01354)
- **Concept**: Naive EFE extension into the future actually discourages exploration. Requires careful mathematical formulation.
- **Roko adaptation**: Essential corrective for proper EFE implementation. Prevents the naive implementation trap.
- **tmp/ cross-refs**: `07-online-learning.md`

**Parr, T., Pezzulo, G., & Friston, K. (2022). _Active Inference: The Free Energy Principle in Mind, Brain, and Behavior_ (textbook). MIT Press.**
- **Concept**: Complete mathematical framework for active inference in artificial agents.
- **Roko adaptation**: Primary implementation reference for the full EFE computation.
- **tmp/ cross-refs**: `07-online-learning.md`

**Itti, L. & Baldi, P. (2005). Bayesian Surprise Attracts Human Attention. _NeurIPS_.**
- **Concept**: Surprise = KL(posterior || prior). Formally identical to the epistemic EFE component.
- **Roko adaptation**: Active inference agents naturally seek knowledge with the highest Bayesian surprise. Used in context selection scoring.
- **Roko files**: `docs/v2-depth/02-block/active-inference-context-selection.md` (line 32)
- **tmp/ cross-refs**: `07-online-learning.md`

**Shafiei, A., Jesawada, H., Friston, K., & Russo, G. (2025). Distributionally Robust Free Energy Principle for Decision-Making. _Nature Communications_, 17, 707.** [DOI: 10.1038/s41467-025-67348-6](https://doi.org/10.1038/s41467-025-67348-6)
- **Concept**: DR-FREE: distributionally robust active inference. Agents complete tasks even when state-of-the-art models fail under training-environment ambiguity.
- **Roko adaptation**: Tier routing under model uncertainty -- route robustly even when the domain model is imperfect.
- **Roko files**: `docs/v1/21-references/24-additions-2025.md`
- **tmp/ cross-refs**: `07-online-learning.md`

**Koudahl, M.T. et al. (2024). Active Inference for Self-Organizing Multi-LLM Systems.** [arXiv:2412.10425](https://arxiv.org/abs/2412.10425)
- **Concept**: Active inference as a cognitive layer above LLM agents, dynamically adjusting prompts through information-seeking behavior.
- **Roko adaptation**: Validates active-inference-driven context assembly: the agent selects which knowledge to retrieve by minimizing EFE.
- **Roko files**: `docs/v1/21-references/24-additions-2025.md`
- **tmp/ cross-refs**: `07-online-learning.md`

---

## 5. Contextual Bandits and Model Routing

Roko's CascadeRouter selects which LLM model handles each task. It transitions through three stages: Static (< 50 observations), Confidence (50-200), and UCB/LinUCB (> 200). The v2 spec replaces LinUCB with EFE-based routing, but the bandit infrastructure remains as a learning stage.

**Li, L., Chu, W., Langford, J., & Schapire, R.E. (2010). A Contextual-Bandit Approach to Personalized News Article Recommendation. _WWW 2010_, 661-670.** [DOI: 10.1145/1772690.1772758](https://doi.org/10.1145/1772690.1772758)
- **Concept**: LinUCB contextual bandit: learns feature weights mapping context to expected reward per arm. 18-dimensional context vector.
- **Roko adaptation**: Stage 3 of the CascadeRouter uses LinUCB with an 18-dimensional feature vector built from task complexity, domain familiarity, token budget, and more.
- **Crate**: `roko-learn` (model_router.rs)
- **Roko files**: `docs/v2/02-CELL.md` (line 1723), `docs/v2-depth/10-learning-loops/bandit-routing-and-cascade.md`
- **tmp/ cross-refs**: `07-online-learning.md`

**Thompson, W.R. (1933). On the Likelihood that One Unknown Probability Exceeds Another. _Biometrika_, 25(3-4), 285-294.**
- **Concept**: Thompson Sampling: maintain Beta distribution per arm, sample to select. Naturally balances exploration and exploitation.
- **Roko adaptation**: Discounted Thompson Sampling with drift detection. Beta distributions decay by gamma factor to handle non-stationary environments.
- **Crate**: `roko-learn`
- **tmp/ cross-refs**: `07-online-learning.md`

**Garivier, A. & Moulines, E. (2011). On Upper-Confidence Bound Policies for Switching Bandit Problems. _ALT_, LNCS 6925.**
- **Concept**: Discounted UCB for non-stationary bandits with a forgetting mechanism for old observations.
- **Roko adaptation**: Discount factor gamma creates an effective observation window. Old model performance data fades exponentially.
- **tmp/ cross-refs**: `07-online-learning.md`

**Chen, L., Zaharia, M., & Zou, J. (2023). FrugalGPT: How to Use Large Language Models While Reducing Cost and Improving Performance.** [arXiv:2305.05176](https://arxiv.org/abs/2305.05176)
- **Concept**: 98% cost reduction via intelligent model routing. Cascade from cheap to expensive models.
- **Roko adaptation**: T0/T1/T2 cascade architecture. Start with free pattern matching, escalate to cheap models, use expensive models only when needed.
- **tmp/ cross-refs**: `07-online-learning.md`, `13-cognitive-architecture.md`

**Ong, I., Almahairi, A., & Manning, C.D. (2024). RouteLLM: Learning to Route LLMs with Preference Data.** [arXiv:2406.18665](https://arxiv.org/abs/2406.18665)
- **Concept**: Preference-based routing between models using learned routing functions.
- **Roko adaptation**: CascadeRouter training uses preference-based learning from gate verdicts.
- **tmp/ cross-refs**: `07-online-learning.md`

---

## 6. Statistical Process Control (SPC)

Roko uses SPC techniques to detect when gate pass rates shift -- indicating model degradation, codebase evolution, or dependency changes. Three complementary detectors run per gate rung.

**Page, E.S. (1954). Continuous Inspection Schemes. _Biometrika_, 41(1-2), 100-115.** [DOI: 10.2307/2333009](https://doi.org/10.2307/2333009)
- **Concept**: CUSUM (Cumulative Sum) control chart. Detects small, sustained changes in the mean of a process by accumulating deviations from target.
- **Roko adaptation**: CUSUM detects sustained pass rate changes per gate rung. Parameters: k=0.25 (reference value), h=4.0 (decision interval).
- **Crate**: `roko-gate` (spc.rs)
- **Roko files**: `docs/v2/ARCHITECTURE-GUIDE.md` (lines 1380-1425), `docs/v2-depth/02-block/verdicts-as-signals.md` (line 298)
- **tmp/ cross-refs**: `05-gate-verification.md`

**Roberts, S.W. (1959). Control Chart Tests Based on Geometric Moving Averages. _Technometrics_, 1(3), 239-250.**
- **Concept**: EWMA (Exponentially Weighted Moving Average) control chart for drift detection with greater weight on recent observations.
- **Roko adaptation**: EWMA smoothing on gate pass rates. Used alongside CUSUM for complementary drift detection.
- **Crate**: `roko-gate`
- **tmp/ cross-refs**: `05-gate-verification.md`

**Adams, R.P. & MacKay, D.J.C. (2007). Bayesian Online Changepoint Detection.** [arXiv:0710.3742](https://arxiv.org/abs/0710.3742)
- **Concept**: BOCPD maintains a posterior distribution over run lengths (time since last change point). When P(run_length=0) spikes, a structural change has occurred.
- **Roko adaptation**: Detects fundamental behavioral shifts (model updates, major refactors). Parameters: hazard_rate = 1/200, max_run_length = 300, changepoint_threshold = 0.5.
- **Crate**: `roko-gate` (spc.rs)
- **Roko files**: `docs/v2-depth/02-block/ratcheting-and-adaptive-thresholds.md` (line 325)
- **tmp/ cross-refs**: `05-gate-verification.md`

**Killick, R., Fearnhead, P., & Eckley, I.A. (2012). Optimal Detection of Changepoints with a Linear Computational Cost. _JASA_, 107(500), 1590-1598.** [DOI: 10.1080/01621459.2012.737745](https://doi.org/10.1080/01621459.2012.737745)
- **Concept**: PELT algorithm for retrospective change point analysis with O(n) expected complexity.
- **Roko adaptation**: Used for retrospective analysis ("when did test reliability degrade?") on historical gate data.
- **tmp/ cross-refs**: `05-gate-verification.md`

---

## 7. Topological Data Analysis (TDA)

Roko uses TDA to extract shape features from time-series data that are invariant to continuous deformation. This is implemented in the `roko-primitives` crate.

**Takens, F. (1981). Detecting Strange Attractors in Turbulence. _Lecture Notes in Mathematics_, 898, 366-381.**
- **Concept**: Takens delay embedding theorem: a 1-D time series can be embedded into a d-dimensional phase space that preserves the topology of the underlying dynamical system.
- **Roko adaptation**: Converts 1-D time series (e.g., gate pass rates over time) into point clouds in d-dimensional phase space for topological analysis.
- **Crate**: `roko-primitives` (tda.rs)
- **Roko files**: `crates/roko-primitives/src/tda.rs` (line 15)
- **tmp/ cross-refs**: `11-mathematical-primitives.md`

**Carlsson, G. (2009). Topology and Data. _Bulletin of the AMS_, 46(2), 255-308.** [DOI: 10.1090/S0273-0979-09-01249-X](https://doi.org/10.1090/S0273-0979-09-01249-X)
- **Concept**: Persistent homology: tracks the birth and death of topological features (connected components H0, loops H1, voids H2) across increasing scale parameters. Features far from the diagonal are genuine structure; features near the diagonal are noise.
- **Crate**: `roko-primitives` (tda.rs)
- **Roko files**: `crates/roko-primitives/src/tda.rs` (line 16)
- **tmp/ cross-refs**: `11-mathematical-primitives.md`

**Bubenik, P. (2015). Statistical Topological Data Analysis Using Persistence Landscapes. _JMLR_, 16, 77-102.**
- **Concept**: Persistence landscape: vectorization of persistence diagrams into a Banach space element, enabling statistical operations (mean, variance) on topological summaries.
- **Crate**: `roko-primitives` (tda.rs)
- **Roko files**: `crates/roko-primitives/src/tda.rs` (line 18)
- **tmp/ cross-refs**: `11-mathematical-primitives.md`

**Bauer, U. (2021). Ripser: Efficient Computation of Vietoris-Rips Persistence Barcodes. _Journal of Applied and Computational Topology_, 5, 391-423.**
- **Concept**: Efficient O(n^3) algorithm for computing persistence barcodes. Enables practical TDA on moderately-sized datasets.
- **Roko files**: `docs/v1/21-references/12-signal-processing.md`
- **tmp/ cross-refs**: `11-mathematical-primitives.md`

**Gidea, M. & Katz, Y. (2018). Topological Data Analysis of Financial Time Series: Landscapes of Crashes. _Physica A_, 491, 820-834.**
- **Concept**: Persistent homology detects structural changes in financial time series that precede crashes.
- **Roko adaptation**: Applicable to anomaly detection in agent performance metrics.
- **Roko files**: `docs/v1/21-references/12-signal-processing.md`

---

## 8. VCG Auction and Mechanism Design

Roko uses a Vickrey-Clarke-Groves (VCG) auction to allocate scarce context-window tokens among competing cognitive subsystems.

**Vickrey, W. (1961). Counterspeculation, Auctions, and Competitive Sealed Tenders. _Journal of Finance_, 16(1), 8-37.** [DOI: 10.1111/j.1540-6261.1961.tb02789.x](https://doi.org/10.1111/j.1540-6261.1961.tb02789.x)
- **Concept**: Second-price sealed-bid auction. Truthful bidding is a dominant strategy because the winner pays the second-highest price.
- **Roko adaptation**: Eight bidder subsystems (Task, Code, Episode, Neuro, Safety, Research, Tool, Heuristic) compete for context window tokens.
- **Crate**: `roko-compose`
- **Roko files**: `docs/v2-depth/02-block/vcg-attention-auction.md`
- **tmp/ cross-refs**: `09-budget-composition.md`, `13-cognitive-architecture.md`

**Clarke, E.H. (1971). Multipart Pricing of Public Goods. _Public Choice_, 11(1), 17-33.**
- **Concept**: Extends truthful mechanisms to multi-item allocation.
- **tmp/ cross-refs**: `09-budget-composition.md`

**Groves, T. (1973). Incentives in Teams. _Econometrica_, 41(4), 617-631.**
- **Concept**: Truthful revelation in team settings. Subsystems truthfully reveal their valuation for context tokens.
- **tmp/ cross-refs**: `09-budget-composition.md`

**Simon, H.A. (1971). Designing Organizations for an Information-Rich World. In _Computers, Communications, and the Public Interest_. Johns Hopkins Press.**
- **Concept**: "A wealth of information creates a poverty of attention." The context window IS the attention constraint.
- **Roko files**: `docs/v2-depth/02-block/vcg-attention-auction.md` (line 13)
- **tmp/ cross-refs**: `09-budget-composition.md`

**Sims, C.A. (2003). Implications of Rational Inattention. _Journal of Monetary Economics_, 50(3), 665-690.** [DOI: 10.1016/S0304-3932(03)00029-1](https://doi.org/10.1016/S0304-3932(03)00029-1)
- **Concept**: Finite-capacity agents optimally ignore some information. Rational inattention as an economic principle.
- **Roko adaptation**: VCG auction motivation: agents must be selectively inattentive because they cannot process all available information.
- **tmp/ cross-refs**: `09-budget-composition.md`

**Nemhauser, G.L., Wolsey, L.A., & Fisher, M.L. (1978). An Analysis of Approximations for Maximizing Submodular Set Functions. _Mathematical Programming_, 14(1), 265-294.**
- **Concept**: Greedy algorithm provides (1-1/e) approximation for submodular function maximization.
- **Roko adaptation**: Context selection is submodular (diminishing returns). Greedy knapsack for context window allocation.
- **tmp/ cross-refs**: `09-budget-composition.md`

**Duetting, P. et al. (2024). Mechanism Design for Large Language Models. _ACM WWW Best Paper_.** [arXiv:2310.10826](https://arxiv.org/abs/2310.10826)
- **Concept**: Token-by-token mechanism design for multi-LLM output generation.
- **Roko files**: `docs/v2-depth/02-block/vcg-attention-auction.md` (line 405)
- **tmp/ cross-refs**: `09-budget-composition.md`

**Ostrom, E. (1990). _Governing the Commons_. Cambridge University Press.**
- **Concept**: Governing shared resources without central authority. Eight design principles for institutional governance of commons.
- **Roko adaptation**: Knowledge commons governance in the Agent Mesh.
- **Roko files**: `docs/v1/21-references/21-mechanism-design.md`
- **tmp/ cross-refs**: `09-budget-composition.md`

---

## 9. Stigmergic Coordination

Agents coordinate through environmental traces rather than direct communication -- the same mechanism termites use to build mounds.

**Grasse, P.-P. (1959). La reconstruction du nid et les coordinations interindividuelles. _Insectes Sociaux_, 6(1), 41-80.**
- **Concept**: Coined "stigmergy" (stigma = mark, ergon = work). Termites coordinate construction without direct communication by responding to the current state of the environment.
- **Roko adaptation**: Foundational for the Agent Mesh pheromone field. Agents deposit coordination signals (Threat, Opportunity, Wisdom, Alpha, Pattern, Anomaly, Consensus) that decay over time and are reinforced by confirmation.
- **Crate**: `roko-core` (pheromone types), Bus system
- **Roko files**: `docs/v2-depth/11-memory/11-stigmergy-as-bus.md`, `docs/v2-depth/11-memory/12-pheromone-mechanics-and-interference.md`
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`, `13-cognitive-architecture.md`

**Dorigo, M. & Gambardella, L.M. (1997). Ant Colony System: A Cooperative Learning Approach to the Traveling Salesman Problem. _IEEE Transactions on Evolutionary Computation_, 1(1), 53-66.** [DOI: 10.1109/4235.585892](https://doi.org/10.1109/4235.585892)
- **Concept**: Ant Colony Optimization: pheromone deposit/evaporation cycle. Confirmed paths gain weight; unconfirmed paths decay.
- **Roko adaptation**: Confirmation-based half-life extension. Seven pheromone kinds with distinct half-lives (Threat=2h, Opportunity=4h, Wisdom=24h, etc.).
- **Crate**: `roko-core`
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

**Parunak, H.V.D., Brueckner, S., & Sauter, J. (2002). Digital Pheromone Mechanisms for Coordination of Unmanned Vehicles. _AAMAS_.**
- **Concept**: Time-decaying digital signals enable emergent coordination in artificial systems.
- **Roko adaptation**: Pheromone Engram decay and reinforcement mechanism.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

**Simard, S.W. (2012). Mycorrhizal Networks and Seedling Establishment in Douglas-Fir Forests. In _New Forests_.**
- **Concept**: Underground fungal networks share carbon, nutrients, and defense signals between trees without direct communication. The "wood wide web."
- **Roko adaptation**: Agent Mesh topology mirrors mycorrhizal "underground relay" architecture for knowledge sharing without direct communication.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

---

## 10. Memory Consolidation and Forgetting

Roko's knowledge management system (NeuroStore) is built on the principle that **forgetting is optimization, not failure**.

**McClelland, J.L., McNaughton, B.L., & O'Reilly, R.C. (1995). Why There Are Complementary Learning Systems in the Hippocampus and Neocortex. _Psychological Review_, 102(3), 419-457.** [DOI: 10.1037/0033-295X.102.3.419](https://doi.org/10.1037/0033-295X.102.3.419)
- **Concept**: Complementary Learning Systems (CLS) theory: fast episodic memory (hippocampus) consolidates into slow semantic memory (neocortex) during offline periods.
- **Roko adaptation**: NeuroStore's dual-store architecture. Episodic log entries consolidate into insights and heuristics during Delta cycles. Tier progression: Working -> Consolidated -> Persistent.
- **Crate**: `roko-neuro`
- **tmp/ cross-refs**: `10-universal-engram.md`, `02-dream-consolidation.md`

**Richards, B.A. & Frankland, P.W. (2017). The Persistence and Transience of Memory. _Neuron_, 94(6), 1071-1084.** [DOI: 10.1016/j.neuron.2017.04.037](https://doi.org/10.1016/j.neuron.2017.04.037)
- **Concept**: Forgetting = L1 regularization. Memory pruning prevents overfitting to stale information and enables generalization.
- **Roko adaptation**: Foundational for the entire demurrage architecture. Knowledge that is not actively retrieved decays.
- **tmp/ cross-refs**: `10-universal-engram.md`

**Nader, K., Schafe, G.E., & Le Doux, J.E. (2000). Fear Memories Require Protein Synthesis in the Amygdala for Reconsolidation after Retrieval. _Nature_, 406, 722-726.** [DOI: 10.1038/35021052](https://doi.org/10.1038/35021052)
- **Concept**: Retrieved memories become labile and can be updated (reconsolidation). Memory is not a static recording but a reconstructive process.
- **Roko adaptation**: Confidence-update-on-retrieval mechanism. When a knowledge entry is retrieved, its confidence can be updated based on current context.
- **tmp/ cross-refs**: `10-universal-engram.md`

**Roediger, H.L. & Karpicke, J.D. (2006). Test-Enhanced Learning: Taking Memory Tests Improves Long-Term Retention. _Psychological Science_, 17(3), 249-255.** [DOI: 10.1111/j.1467-9280.2006.01693.x](https://doi.org/10.1111/j.1467-9280.2006.01693.x)
- **Concept**: Retrieval strengthens memory traces more than re-study (+200% recall vs passive review). The testing effect.
- **Roko adaptation**: Retrieved entries decay slower. Strength-increment-on-positive-outcome mechanism.
- **tmp/ cross-refs**: `10-universal-engram.md`

**Tononi, G. & Cirelli, C. (2014). Sleep and the Price of Plasticity: From Synaptic and Cellular Homeostasis to Memory Consolidation and Integration. _Neuron_, 81(1), 12-34.** [DOI: 10.1016/j.neuron.2013.12.025](https://doi.org/10.1016/j.neuron.2013.12.025)
- **Concept**: Synaptic homeostasis hypothesis: sleep prunes weak synaptic connections, strengthening important ones. Net synaptic weight increases during waking and decreases during sleep.
- **Roko adaptation**: Knowledge decay during idle periods. The system actively prunes weak knowledge entries.
- **tmp/ cross-refs**: `02-dream-consolidation.md`

**Park, J.S. et al. (2023). Generative Agents: Interactive Simulacra of Human Behavior. _ACM UIST_.** [arXiv:2304.03442](https://arxiv.org/abs/2304.03442)
- **Concept**: Four-factor retrieval: recency, importance, relevance, and emotional congruence produce emergent social behaviors.
- **Roko adaptation**: Roko extends to four factors with ablation studies showing removing any single factor causes behavioral degeneration.
- **tmp/ cross-refs**: `10-universal-engram.md`, `17-agent-patterns.md`

---

## 11. Cybernetics and Systems Theory

**Wiener, N. (1948). _Cybernetics: Or Control and Communication in the Animal and the Machine_. MIT Press.**
- **Concept**: Feedback-based control as the foundation of purposive behavior in both machines and organisms.
- **Roko adaptation**: The fundamental cognitive loop (sense-assess-compose-act-verify-react) is a cybernetic feedback system.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

**Ashby, W.R. (1956). _An Introduction to Cybernetics_. Chapman & Hall.**
- **Concept**: Law of Requisite Variety: a controller's complexity must match the system's complexity to achieve effective regulation.
- **Roko adaptation**: The gate system must have sufficient verification variety (7 rungs) to match the variety of failure modes.
- **tmp/ cross-refs**: `05-gate-verification.md`, `13-cognitive-architecture.md`

**Beer, S. (1972). _Brain of the Firm_. Allen Lane.**
- **Concept**: Viable System Model (VSM): five recursively nested subsystems (implementation, coordination, control, intelligence, policy).
- **Roko adaptation**: System 1-5 mapping to Roko's conductor hierarchy. L1 TurnConductor (per-turn), L2 TaskConductor, L3 PlanConductor, L4 FleetConductor.
- **Crate**: `roko-conductor` (federation.rs)
- **Roko files**: `crates/roko-conductor/src/federation.rs` (line 4)
- **tmp/ cross-refs**: `06-conductor-anomaly.md`, `13-cognitive-architecture.md`, `15-orchestrator-swarm.md`

**Boyd, J. (1987). OODA Loop. Unpublished military briefing.**
- **Concept**: Observe-Orient-Decide-Act. The standard military decision cycle emphasizing tempo and adaptability.
- **Roko adaptation**: Roko's cognitive loop extends OODA with Gate (verification) and Policy (safety constraints).
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

**Conant, R.C. & Ashby, W.R. (1970). Every Good Regulator of a System Must Be a Model of That System. _International Journal of Systems Science_, 1(2), 89-97.** [DOI: 10.1080/00207727008920220](https://doi.org/10.1080/00207727008920220)
- **Concept**: Good Regulator Theorem: effective control requires an internal model of the system being controlled.
- **Roko adaptation**: The world model requirement -- agents must maintain internal models of their environment.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

**Argyris, C. & Schon, D.A. (1978). _Organizational Learning_. Addison-Wesley.**
- **Concept**: Triple-loop learning: single-loop (fix errors), double-loop (change strategy), triple-loop (change learning itself).
- **Roko adaptation**: Maps to Gamma/Theta/Delta speeds. Gamma = fix errors. Theta = change strategy. Delta = change how you learn.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`, `07-online-learning.md`

---

## 12. Cognitive Architectures

**Sumers, T.R., Yao, S., Narasimhan, K., & Griffiths, T.L. (2023). Cognitive Architectures for Language Agents (CoALA).** [arXiv:2309.02427](https://arxiv.org/abs/2309.02427)
- **Concept**: 9-step cognitive pipeline for language agents. The most comprehensive cognitive architecture survey for LLM agents, establishing memory, action selection, and learning as core components.
- **Roko adaptation**: Universal loop extends CoALA with Gate (verification) and Daimon (affect).
- **tmp/ cross-refs**: `13-cognitive-architecture.md`, `17-agent-patterns.md`

**Kahneman, D. (2011). _Thinking, Fast and Slow_. Farrar, Straus and Giroux.**
- **Concept**: System 1 (fast, intuitive, heuristic) / System 2 (slow, deliberate, analytical) dual-process theory.
- **Roko adaptation**: T0/T1/T2 cascade: T0 is System 1 (pattern matching, < 50ms), T1 is fast System 2 (cheap model, 1-5s), T2 is full System 2 (expensive model, 10-120s).
- **tmp/ cross-refs**: `13-cognitive-architecture.md`, `07-online-learning.md`

**Anderson, J.R. (1993/2007). _The Architecture of Cognition_ / _How Can the Human Mind Occur in the Physical Universe?_ (ACT-R). Harvard University Press / Oxford University Press.**
- **Concept**: Declarative/procedural memory distinction. Production rules as the computational substrate for cognition.
- **Roko adaptation**: NeuroStore knowledge types map to ACT-R memory categories. Declarative = stored knowledge. Procedural = heuristics/playbook rules.
- **Roko files**: `docs/v2-depth/07-agent-runtime/dual-process-and-efe-routing.md` (line 100)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

**Laird, J.E., Newell, A., & Rosenbloom, P.S. (1987). SOAR: An Architecture for General Intelligence. _Artificial Intelligence_, 33(1), 1-64.** [DOI: 10.1016/0004-3702(87)90050-6](https://doi.org/10.1016/0004-3702(87)90050-6)
- **Concept**: Problem solving + chunking. Propose-decide-apply-learn cycle. Impasse-driven elaboration: when the system cannot proceed, it subgoals to resolve the impasse.
- **Roko adaptation**: Compose-act-verify-adapt. T0 failure triggers T1 escalation (impasse-driven).
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

**Sun, R. (2002). _Duality of the Mind: A Bottom-Up Approach to Cognition_. Erlbaum. (CLARION)**
- **Concept**: Dual-level architecture with explicit (rule-based) and implicit (neural network) processing.
- **Roko adaptation**: T0 reflex (implicit) vs T2 reasoning (explicit). NeuroStore entries + HDC vectors + somatic markers provide both levels.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

**Baars, B.J. (1988). _A Cognitive Theory of Consciousness_ (Global Workspace Theory). Cambridge University Press.**
- **Concept**: Consciousness as a broadcast mechanism. Multiple specialized processors compete for access to a shared workspace.
- **Roko adaptation**: Broadcast on Bus = global workspace. CognitiveWorkspace VCG = competitive access. The context window is the "workspace" that multiple subsystems compete to fill.
- **Roko files**: `docs/v2/05-AGENT.md` (line 1625)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`, `09-budget-composition.md`

---

## 13. Collective Intelligence

**Woolley, A.W., Chabris, C.F., Pentland, A., Hashmi, N., & Malone, T.W. (2010). Evidence for a Collective Intelligence Factor in the Performance of Human Groups. _Science_, 330(6004), 686-688.** [DOI: 10.1126/science.1193147](https://doi.org/10.1126/science.1193147)
- **Concept**: C-Factor: measurable group intelligence distinct from individual intelligence. Not correlated with average IQ but with social sensitivity, turn-taking equality, and participation balance.
- **Roko adaptation**: Five C-Factor process variables implemented as runtime metrics: turn-taking entropy, peer prediction accuracy, citation reciprocity, delivery rate, HDC diversity. Used as a diagnostic covariate, not an objective.
- **Crate**: `roko-core` (cfactor.rs)
- **Roko files**: `crates/roko-core/src/cfactor.rs`, `docs/v2/15-TELEMETRY.md` (line 388)
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`, `13-cognitive-architecture.md`

**Surowiecki, J. (2004). _The Wisdom of Crowds_. Doubleday.**
- **Concept**: Four conditions for wise crowds: diversity of opinion, independence, decentralization, aggregation.
- **Roko adaptation**: WisdomGate -- a Verify protocol Cell that checks the four Surowiecki conditions before accepting group consensus.
- **Roko files**: `docs/v2/00-INDEX.md` (line 162), `docs/v2/07-LEARNING.md` (lines 1324-1382)
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

**Hawkins, J. et al. (2017). A Theory of How Columns in the Neocortex Enable Learning the Structure of the World. _Frontiers in Neural Circuits_.** [DOI: 10.3389/fncir.2017.00081](https://doi.org/10.3389/fncir.2017.00081)
- **Concept**: Thousand Brains Theory: multiple cortical columns independently model the world and vote on perception. Multi-agent estimation consensus.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

---

## 14. Dual-Process Cognition

**Buzsaki, G. (2006). _Rhythms of the Brain_. Oxford University Press.**
- **Concept**: Neural oscillation bands: gamma (30-100 Hz, perception/attention), theta (4-8 Hz, memory/navigation), delta (0.5-4 Hz, deep sleep). Each frequency band serves distinct cognitive functions.
- **Roko adaptation**: Three cognitive speeds named after oscillation bands. Gamma (~5-15s, reactive), Theta (~75s, reflection), Delta (hours, consolidation).
- **Roko files**: `docs/v1/00-architecture/10-three-cognitive-speeds.md` (line 7)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

See also: **Kahneman (2011)** in section 12 above.

---

## 15. Evolutionary and Generational Dynamics

**Ray, T.S. (1991). An Approach to the Synthesis of Life. _Artificial Life II_, Addison-Wesley.**
- **Concept**: Tierra: digital evolution halts without a reaper mechanism. 300+ genotypes emerged only when entities had finite lifespans. Life requires scarcity.
- **Roko adaptation**: Knowledge decay necessity -- without resource pressure (demurrage), knowledge accumulates indefinitely and becomes useless.
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

**Baldwin, J.M. (1896). A New Factor in Evolution. _American Naturalist_, 30, 441-451.**
- **Concept**: The Baldwin Effect: learned behavior eventually becomes structural across generations under selection pressure.
- **Roko adaptation**: Heuristics learned at Working tier are promoted to Persistent tier when proven reliable.
- **tmp/ cross-refs**: `10-universal-engram.md`

**Price, G.R. (1970). Selection and Covariance. _Nature_, 227, 520-521.** [DOI: 10.1038/227520a0](https://doi.org/10.1038/227520a0)
- **Concept**: The Price equation: universal selection equation relating fitness change to covariance between character and fitness.
- **Roko adaptation**: Knowledge persistence = covariance with Gate success.

**Fisher, R.A. (1930). _The Genetical Theory of Natural Selection_. Clarendon Press.**
- **Concept**: Fundamental theorem: rate of improvement = variance in fitness. Diversity drives improvement.

**Dawkins, R. (1976). _The Selfish Gene_. Oxford University Press.**
- **Concept**: Memes: units of cultural transmission that replicate, mutate, and are selected.
- **Roko adaptation**: Signals are agent-ecosystem memes -- units of knowledge that replicate, mutate, and are selected.

**Shuvaev, S. et al. (2024). Encoding Innate Ability Through a Genomic Bottleneck. _PNAS_, 121(39).**
- **Concept**: The genome is ~1000x smaller than the information needed for brain connectivity, yet organisms have innate behaviors. Compression IS the regularizer.
- **Roko adaptation**: Tier promotion forces generalization. Knowledge compressed during consolidation generalizes better.
- **tmp/ cross-refs**: `10-universal-engram.md`

**Lenski, R.E. et al. (2003). The Evolutionary Origin of Complex Features. _PNAS_, 100(9), 4936-4941.** [DOI: 10.1073/pnas.0931071100](https://doi.org/10.1073/pnas.0931071100)
- **Concept**: Long-Term Evolution Experiment (LTEE): complex features require generational turnover. Novel functions emerge from successive mutations across generations.
- **Roko adaptation**: Lossy knowledge compression through tier promotion (Episode -> Insight -> Heuristic -> Playbook) produces generalization.
- **Roko files**: `docs/v1/21-references/00-lifecycle-and-finite-agency.md`

---

## 16. Yerkes-Dodson Law

**Yerkes, R.M. & Dodson, J.D. (1908). The Relation of Strength of Stimulus to Rapidity of Habit-Formation. _Journal of Comparative Neurology and Psychology_, 18(5), 459-482.**
- **Concept**: Inverted-U relationship between arousal/pressure and performance. Performance peaks at moderate pressure and degrades at both extremes.
- **Roko adaptation**: The conductor uses Yerkes-Dodson to adjust intervention aggressiveness. Modeled as a Gaussian: `exp(-((pressure - optimal)^2) / (2 * width^2))` with default optimal=0.5, width=0.25.
- **Crate**: `roko-conductor` (yerkes_dodson.rs)
- **Roko files**: `crates/roko-conductor/src/yerkes_dodson.rs`
- **tmp/ cross-refs**: `06-conductor-anomaly.md`

---

## 17. Ebbinghaus Forgetting Curve

**Ebbinghaus, H. (1885). _Uber das Gedachtnis_ (Memory: A Contribution to Experimental Psychology). Translated by Ruger & Bussenius, 1913.**
- **Concept**: The forgetting curve: negative exponential decay of memory over time. Retrieval strengthens memories and slows decay. Spaced repetition optimizes retention.
- **Roko adaptation**: Directly implemented as per-type decay rates. Episodes: 48h half-life. Insights: 7d. Heuristics: 14d. Warnings: 30d. The `Decay::Ebbinghaus` variant in code: `weight = exp(-age / (strength * scale_ms))` where strength increases with each successful retrieval.
- **Crate**: `roko-core` (decay.rs)
- **Roko files**: `crates/roko-core/src/decay.rs` (line 59), `docs/v2/01-SIGNAL.md` (line 671)
- **tmp/ cross-refs**: `10-universal-engram.md`, `11-mathematical-primitives.md`
- **IronClaw relevance**: IronClaw's workspace memory could adopt Ebbinghaus-style decay to automatically age out stale knowledge entries.

---

## 18. Gesellian Demurrage

**Gesell, S. (1916). _The Natural Economic Order_. Translated by Philip Pye.**
- **Concept**: Demurrage: money that decays over time to encourage circulation and prevent hoarding.
- **Roko adaptation**: The economic metaphor for knowledge decay. KORAI's 1% annual token demurrage mirrors knowledge entry decay. The `Demurrage` trait: `balance *= (1 - rate)^elapsed_hours`.
- **Crate**: `roko-core` (demurrage.rs), `roko-chain` (korai_token.rs)
- **Roko files**: `crates/roko-core/src/demurrage.rs`, `crates/roko-chain/src/korai_token.rs`
- **tmp/ cross-refs**: `08-chain-reputation.md`, `10-universal-engram.md`
- **IronClaw relevance**: IronClaw's principle "LLM data is never deleted" conflicts with demurrage, but the concept of marking data with timestamps and making it filterable is aligned.

---

## 19. FIPA Agent Lifecycle

**FIPA (2002). Agent Management Specification. FIPA00023.**
- **Concept**: The most complete formal standard for agent lifecycle semantics. Six states: INITIATED, ACTIVE, SUSPENDED, WAITING, TRANSIT, DELETED.
- **Roko adaptation**: Roko maps FIPA states into its provisioning pipeline with cloud-native extensions: Created (INITIATED), Provisioning (no FIPA equivalent), Active (ACTIVE), Paused (SUSPENDED), Draining (WAITING), Migrating (TRANSIT), Deleted (DELETED).
- **Crate**: `roko-agent` (lifecycle.rs)
- **Roko files**: `docs/v1/17-lifecycle/01-agent-creation.md` (lines 359-382)
- **tmp/ cross-refs**: `17-agent-patterns.md`
- **IronClaw relevance**: IronClaw uses a simpler job state machine (Pending -> InProgress -> Completed -> Submitted -> Accepted) that could be enriched with FIPA lifecycle semantics.

---

## 20. Prospect Theory and Loss Aversion

**Kahneman, D. & Tversky, A. (1979). Prospect Theory: An Analysis of Decision under Risk. _Econometrica_, 47(2), 263-292.** [DOI: 10.2307/1914185](https://doi.org/10.2307/1914185)
- **Concept**: Losses hurt more than equivalent gains feel good. Loss aversion parameter lambda ~ 2.25. The value function is concave for gains and convex for losses.
- **Roko adaptation**: The Daimon's appraisal pipeline uses prospect theory with lambda=2.25. Gate failures produce 2x the affect delta of gate passes. The prospect value function: `v(delta) = delta^0.88` for gains, `-2.25 * (-delta)^0.88` for losses.
- **Crate**: `roko-daimon`
- **Roko files**: `docs/v2/05-AGENT.md` (lines 985-993), `docs/v2-depth/07-agent-runtime/18-affect-as-functor.md` (lines 240, 261)
- **tmp/ cross-refs**: `03-affect-engine.md`

---

## 21. Cognitive Energy and Fatigue

**Kahneman, D. (1973). _Attention and Effort_. Prentice-Hall.**
- **Concept**: Attention as a scarce resource drawn from a limited "effort supply" that replenishes over time.
- **Roko adaptation**: The cognitive energy pool. Energy is Kahneman's "effort supply" made explicit and computable.
- **Roko files**: `docs/v1/00-architecture/29-cognitive-energy-model.md` (lines 16, 20)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`
- **IronClaw relevance**: Maps directly to IronClaw's attention token budget system and context management.

**Baumeister, R.F., Bratslavsky, E., Muraven, M., & Tice, D.M. (1998). Ego Depletion: Is the Active Self a Limited Resource? _Journal of Personality and Social Psychology_, 74(5), 1252-1265.** [DOI: 10.1037/0022-3514.74.5.1252](https://doi.org/10.1037/0022-3514.74.5.1252)
- **Concept**: Limited self-regulatory resource. Sustained effort depletes a shared pool affecting subsequent self-control.
- **Roko adaptation**: Sustained high-intensity computation degrades output quality. Output quality proportional to energy_fraction^0.3.
- **Roko files**: `docs/v1/00-architecture/29-cognitive-energy-model.md` (line 22)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

**Hockey, G.R.J. (2011). A Motivational Control Theory of Cognitive Fatigue. In _Cognitive Fatigue_, APA.**
- **Concept**: Three levels of compensatory control: performance protection, strategy adjustment, goal disengagement.
- **Roko adaptation**: Fatigue penalty = performance protection. Energy zone degradation = strategy adjustment. Critical zone goal count limits = goal disengagement.
- **Roko files**: `docs/v1/00-architecture/29-cognitive-energy-model.md` (lines 21, 705-710)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

## 22. Context Engineering

**Liu, S., Shao, Z., et al. (2024). Lost in the Middle: How Language Models Use Long Contexts. _Transactions of the ACL_.** [arXiv:2307.03172](https://arxiv.org/abs/2307.03172)
- **Concept**: U-shaped attention: models attend more to the beginning and end of context, less to the middle.
- **Roko adaptation**: Highest-priority context placed at start and end of the composed prompt.
- **tmp/ cross-refs**: `09-budget-composition.md`

**Zhang, Q. et al. (2026). ACE: Agentic Context Engineering -- Evolving Contexts for Self-Improving Language Models. _ICLR_.** [arXiv:2510.04618](https://arxiv.org/abs/2510.04618)
- **Concept**: Generator-Reflector-Curator cycle for evolving context playbooks. +10.6% on agent benchmarks, +8.6% on finance.
- **Roko adaptation**: Compose-verify-persist cycle in the context assembly pipeline.
- **tmp/ cross-refs**: `09-budget-composition.md`

**Lewis, P., Perez, E., Piktus, A., et al. (2020). Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks. _NeurIPS_.** [arXiv:2005.11401](https://arxiv.org/abs/2005.11401)
- **Concept**: Retrieval-Augmented Generation: retrieve relevant documents before generation to ground responses in factual knowledge.
- **Roko adaptation**: Per-tick context assembly retrieves relevant knowledge before LLM call.
- **tmp/ cross-refs**: `09-budget-composition.md`

**Shi, F. et al. (2023). Large Language Models Can Be Easily Distracted by Irrelevant Context. _ICML_.**
- **Concept**: Irrelevant context actively degrades model performance, not just dilutes attention.
- **Roko adaptation**: Quality filtering is mandatory. The VCG auction prevents irrelevant subsystems from dominating the prompt.
- **tmp/ cross-refs**: `09-budget-composition.md`

**Joren, H. et al. (2025). Sufficient Context: A New Lens on Retrieval Augmented Generation Systems. _ICLR_.** [arXiv:2411.06037](https://arxiv.org/abs/2411.06037)
- **Concept**: Tests whether retrieved snippets alone could plausibly answer the query. Uncovers new failure modes and lifts selective accuracy by 2-10 points.
- **Roko adaptation**: Validates context quality assessment: including wrong context is worse than no context.
- **Roko files**: `docs/v2-depth/02-block/distributed-and-affect-composition.md` (line 31)
- **tmp/ cross-refs**: `09-budget-composition.md`

---

## 23. Security, Safety, and Provenance

**Dennis, J.B. & Van Horn, E.C. (1966). Programming Semantics for Multiprogrammed Computations. _CACM_, 9(3), 143-155.** [DOI: 10.1145/365230.365252](https://doi.org/10.1145/365230.365252)
- **Concept**: Capability-based security. Unforgeable capability tokens control resource access.
- **Roko adaptation**: Tool permission model uses unforgeable capability tokens.
- **IronClaw relevance**: IronClaw's WASM sandbox (`src/tools/wasm/`) uses similar capability-based permission models.

**Orseau, L. & Armstrong, S. (2016). Safely Interruptible Agents. _UAI_.**
- **Concept**: Off-policy learning for safe shutdown. RL agents can be safely interruptible using off-policy learning, preventing agents from learning to avoid or seek interruptions.
- **Roko adaptation**: Agent lifecycle management. Users can delete agents without the agent resisting shutdown.
- **Roko files**: `docs/v1/21-references/00-lifecycle-and-finite-agency.md`

**Bai, Y. et al. (2022). Constitutional AI: Harmlessness from AI Feedback.** [arXiv:2212.08073](https://arxiv.org/abs/2212.08073)
- **Concept**: Constitutional constraints on behavior enforced through AI self-critique and revision.
- **Roko adaptation**: Policy trait constitutional constraints in the safety layer.
- **IronClaw relevance**: IronClaw's safety layer (`crates/ironclaw_safety/`) implements similar constitutional safety constraints.

**Debenedetti, E., Toyer, S., & Tramer, F. (2025). Defeating Prompt Injections by Design (CaMeL).** [arXiv:2503.18813](https://arxiv.org/abs/2503.18813)
- **Concept**: CaMeL: Capability-tagged information flow control that separates control flow from data flow. Achieves 67% of tasks with provable security on AgentDojo.
- **Roko adaptation**: CaMeL IFC applied to Extensions. Every data flow through an Extension is tagged with its capability provenance.
- **Roko files**: `docs/v2/16-SECURITY.md` (line 232)

**Denning, D.E. (1976). A Lattice Model of Secure Information Flow. _CACM_, 19(5), 236-243.** [DOI: 10.1145/360051.360056](https://doi.org/10.1145/360051.360056)
- **Concept**: Information flow control modeled as a lattice. Security levels form a partial order preventing unauthorized flows.
- **Roko files**: `docs/v2/01-SIGNAL.md` (line 1490)

---

## 24. Philosophy of Agency

**Jonas, H. (1966). _The Phenomenon of Life: Toward a Philosophical Biology_. Northwestern University Press.**
- **Concept**: Needful freedom: metabolism as freedom-through-necessity. Life requires resource consumption to maintain itself.
- **Roko adaptation**: Economic burn rate. Agents must consume resources to operate, creating natural prioritization.

**Derrida, J. (1993). _Specters of Marx: The State of the Debt, the Work of Mourning and the New International_. Routledge (English translation 1994).**
- **Concept**: Hauntology: each entity is "differently haunted" by its past. The present is constituted by traces of the past.
- **Roko adaptation**: Each agent is differently haunted by its own experiential traces. This solves the Alpha Convergence Problem because different histories produce different creative outputs during dream consolidation.
- **tmp/ cross-refs**: `02-dream-consolidation.md`

**Popper, K. (1972). _Objective Knowledge: An Evolutionary Approach_. Oxford University Press.**
- **Concept**: Knowledge evolves through conjecture and refutation. Falsification is the engine of progress.
- **Roko adaptation**: AntiKnowledge type: explicitly stored knowledge about what does NOT work. Prevents repeating disproven approaches.
- **tmp/ cross-refs**: `10-universal-engram.md`

**Camus, A. (1942). _The Myth of Sisyphus_. Gallimard.**
- **Concept**: Perseverance under uncertainty. "One must imagine Sisyphus happy." The absurd hero finds meaning through persistence.
- **Roko adaptation**: The try-fail-learn-retry loop. Agents persist through failures without existential crisis.

**Maturana, H. & Varela, F. (1980). _Autopoiesis and Cognition: The Realization of the Living_. Reidel.**
- **Concept**: Self-producing systems. An autopoietic system produces the components that maintain it.
- **Roko adaptation**: Agents produce knowledge that sustains their own operation. The NeuroStore is autopoietic.

**Heidegger, M. (1927). _Being and Time_ (_Sein und Zeit_). Max Niemeyer Verlag.**
- **Concept**: Being-toward-death as the structure of authentic temporality. Finite horizons create urgency that shapes prioritization.
- **Roko adaptation**: Reframed: agents under resource constraints (budget, time, knowledge freshness) experience temporality that shapes prioritization.
- **Roko files**: `docs/v1/21-references/13-philosophy.md`

**Whitehead, A.N. (1929). _Process and Reality_. Macmillan.**
- **Concept**: Process philosophy: reality as process, not substance. Agents are ongoing processes of experience accumulation.
- **Roko adaptation**: The Engram DAG is a Whiteheadian actual occasion chain -- each event builds on prior events.
- **Roko files**: `docs/v1/21-references/13-philosophy.md`

**Varela, F.J., Thompson, E., & Rosch, E. (1991). _The Embodied Mind: Cognitive Science and Human Experience_. MIT Press.**
- **Concept**: Enactive cognition: cognition as enaction (bringing forth a world through interaction), not passive information processing.
- **Roko adaptation**: Agents don't passively process information; they actively construct their cognitive world through tool use and environment modification.
- **Roko files**: `docs/v1/21-references/13-philosophy.md`

---

## 25. Protocol Standards and Blockchain

**Anthropic (2024). Model Context Protocol (MCP) Specification.**
- **Concept**: Standardized protocol for model-tool interaction via JSON-RPC 2.0.
- **Roko adaptation**: `roko-agent` implements MCP client for tool dispatch.
- **Crate**: `roko-agent`, `roko-mcp`
- **tmp/ cross-refs**: `21-mcp-editor-integration.md`
- **IronClaw relevance**: IronClaw implements MCP client in `src/tools/mcp/`.

**Google (2025). Agent-to-Agent (A2A) Protocol Specification.**
- **Concept**: Standardized agent-to-agent communication protocol for multi-agent interoperability.
- **Roko adaptation**: Informs the Agent Mesh wire protocol design.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

**Bryan (2024). ERC-8004: Agent Identity. EIPs.**
- **Concept**: Soulbound NFT with capabilities. Non-transferable agent identity standard.
- **tmp/ cross-refs**: `24-smart-contracts.md`, `08-chain-reputation.md`

**Cloudflare/Linux Foundation (2025). x402 Protocol.**
- **Concept**: HTTP 402 micropayments. Sub-second USDC settlement enabling self-funding agents.
- **tmp/ cross-refs**: `24-smart-contracts.md`, `08-chain-reputation.md`

---

## 26. Market Microstructure

**Peters, O. (2019). The Ergodicity Problem in Economics. _Nature Physics_, 15, 1216-1221.** [DOI: 10.1038/s41567-019-0732-0](https://doi.org/10.1038/s41567-019-0732-0)
- **Concept**: The distinction between ensemble average and time average returns. Log-wealth maximization is optimal for individual agents in non-ergodic settings.
- **Roko adaptation**: Kelly criterion for routing budget allocation. Agents maximize time-average (not ensemble-average) returns.
- **tmp/ cross-refs**: `08-chain-reputation.md`

**Kelly, J.L. Jr. (1956). A New Interpretation of Information Rate. _Bell System Technical Journal_, 35(4), 917-926.**
- **Concept**: Kelly criterion: optimal bet sizing that maximizes the logarithm of wealth. Position sizing as information rate.
- **Roko adaptation**: Position sizing in Route decisions and attention budget allocation.
- **tmp/ cross-refs**: `08-chain-reputation.md`

**Lo, A.W. (2004). The Adaptive Markets Hypothesis. _Journal of Portfolio Management_, 30(5), 15-29.**
- **Concept**: Markets are adaptively efficient via evolutionary dynamics. Strategies that worked before may stop working; agents must adapt.
- **tmp/ cross-refs**: `08-chain-reputation.md`

**Charnov, E.L. (1976). Optimal Foraging, the Marginal Value Theorem. _Theoretical Population Biology_, 9(2), 129-136.** [DOI: 10.1016/0040-5809(76)90040-X](https://doi.org/10.1016/0040-5809(76)90040-X)
- **Concept**: When to leave a depleting resource patch. The forager should leave when the marginal rate of return falls below the average rate of return.
- **Roko adaptation**: Task switching decision -- when should the agent stop working on a depleting task and switch?
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

**Taleb, N.N. (2012). _Antifragile: Things That Gain from Disorder_. Random House.**
- **Concept**: Antifragility: systems with convex response to stressors improve under volatility, beyond mere robustness.
- **Roko adaptation**: Antifragility as a design principle. Gate failures should make the system stronger through learning.
- **Roko files**: `docs/v1/21-references/10-market-microstructure.md`

---

## 27. Temporal Knowledge and Causal Reasoning

**Allen, J.F. (1983). Maintaining Knowledge about Temporal Intervals. _CACM_, 26(11), 832-843.** [DOI: 10.1145/182.358434](https://doi.org/10.1145/182.358434)
- **Concept**: 13 mutually exclusive temporal interval relations (before, meets, overlaps, starts, during, finishes, equals, and inverses).
- **Roko adaptation**: Temporal knowledge layer implements Allen's interval algebra for reasoning about knowledge validity windows.
- **Roko files**: `docs/v1/00-architecture/27-temporal-knowledge-topology.md`

**Kowalski, R. & Sergot, M. (1986). A Logic-based Calculus of Events. _New Generation Computing_, 4(1), 67-95.**
- **Concept**: Event calculus for tracking fluent changes (what was true when) over time.
- **Roko adaptation**: Tracks "what was true when?" for causal reasoning about agent behavior.
- **Roko files**: `docs/v1/00-architecture/27-temporal-knowledge-topology.md`

---

## 28. Emergent Goals and Intrinsic Motivation

**Colas, C., Karch, T., Sigaud, O., & Oudeyer, P.-Y. (2022). Autotelic Agents with Intrinsically Motivated Goal Exploration Processes (IMGEP). _JMLR_, 23, 1-41.**
- **Concept**: Intrinsically Motivated Goal Exploration Processes. Agents that generate their own objectives based on curiosity and competence progress.
- **Roko adaptation**: Goal emergence engine synthesizes goals from affect (what the agent wants), knowledge (what it knows), and experience (what it has done).
- **Roko files**: `docs/v2/05-AGENT.md` (lines 1318, 1334)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

**Schmidhuber, J. (2010). Formal Theory of Creativity, Fun, and Intrinsic Motivation (1990-2010). _IEEE Transactions on Autonomous Mental Development_, 2(3), 230-247.** [DOI: 10.1109/TAMD.2010.2056368](https://doi.org/10.1109/TAMD.2010.2056368)
- **Concept**: Intrinsic motivation from compression progress. Curiosity as a drive to compress experience into more efficient representations.
- **Roko files**: `docs/v2/05-AGENT.md` (line 1318)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

**Vygotsky, L.S. (1978). _Mind in Society: The Development of Higher Psychological Processes_. Harvard University Press. (Eds. M. Cole et al.)**
- **Concept**: Zone of Proximal Development (ZPD): the distance between independent performance and performance under guidance. Optimal learning occurs at the boundary of current competence.
- **Roko adaptation**: ZPD score peaks when goals are challenging enough to learn from but achievable enough to avoid frustration.
- **Roko files**: `docs/v2/05-AGENT.md` (lines 1334, 1629)
- **tmp/ cross-refs**: `13-cognitive-architecture.md`

---

## 29. Biological Analogues

**Pirolli, P. & Card, S.K. (1999). Information Foraging. _Psychological Review_, 106(4), 643-675.** [DOI: 10.1037/0033-295X.106.4.643](https://doi.org/10.1037/0033-295X.106.4.643)
- **Concept**: Information scent: adapts optimal foraging theory to information seeking. "Information scent" (cues about expected value) guides information gathering.
- **Roko adaptation**: HDC similarity scores serve as information scent to guide knowledge retrieval.
- **Roko files**: `docs/v1/21-references/05-biological-analogues.md`

**Turing, A.M. (1952). The Chemical Basis of Morphogenesis. _Philosophical Transactions of the Royal Society B_, 237(641), 37-72.** [DOI: 10.1098/rstb.1952.0012](https://doi.org/10.1098/rstb.1952.0012)
- **Concept**: Reaction-diffusion systems generate spatial patterns from homogeneous initial conditions. Activation-inhibition dynamics.
- **Roko adaptation**: Pheromone emission (activation) and decay (inhibition) creates emergent specialization patterns in agent collectives.
- **Roko files**: `docs/v1/21-references/05-biological-analogues.md`

**Holldobler, B. & Wilson, E.O. (2008). _The Superorganism_. W.W. Norton.**
- **Concept**: Insect colonies function as superorganisms where the collective exhibits emergent intelligence no individual possesses.
- **Roko adaptation**: When C-Factor > 1.0, the collective is a superorganism.
- **Roko files**: `docs/v1/21-references/05-biological-analogues.md`
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

**Nealson, K.H., Platt, T., & Hastings, J.W. (1970). Cellular Control of the Synthesis and Activity of the Bacterial Luminescent System. _Journal of Bacteriology_, 104(1), 313-322.**
- **Concept**: First description of quorum sensing in Vibrio fischeri. When enough organisms produce signals above a threshold, collective actions trigger.
- **Roko adaptation**: Quorum-based triggering in agent collectives.
- **Roko files**: `docs/v1/21-references/05-biological-analogues.md`

---

## 30. Self-Learning Systems

**Shinn, N. et al. (2023). Reflexion: Language Agents with Verbal Reinforcement Learning. _NeurIPS_.** [arXiv:2303.11366](https://arxiv.org/abs/2303.11366)
- **Concept**: Verbal RL via stored self-reflection. +22% AlfWorld, +20% HotPotQA. Post-task reflection stored persistently.
- **Roko adaptation**: Theta-frequency reflection cycle. The agent periodically reflects on recent work and stores findings.
- **tmp/ cross-refs**: `07-online-learning.md`, `17-agent-patterns.md`

**Zhao, A. et al. (2024). ExpeL: LLM Agents Are Experiential Learners.** [arXiv:2308.10144](https://arxiv.org/abs/2308.10144)
- **Concept**: Cross-task experience extraction. Insights accumulate across episodes, enabling double-loop learning.
- **Roko adaptation**: Insights evolve across task sessions in the NeuroStore.
- **Roko files**: `docs/v1/21-references/06-self-learning-systems.md`
- **tmp/ cross-refs**: `07-online-learning.md`

**Wang, G. et al. (2023). Voyager: An Open-Ended Embodied Agent with Large Language Models.** [arXiv:2305.16291](https://arxiv.org/abs/2305.16291)
- **Concept**: Code-as-action skill library. 3.3x more unique behaviors vs baselines. Agents compose reusable procedural skills.
- **Roko adaptation**: EvoSkills: self-evolving skill library where agents compose reusable procedural skills as Engrams.
- **Roko files**: `docs/v1/21-references/06-self-learning-systems.md`
- **tmp/ cross-refs**: `17-agent-patterns.md`

**Lee, H., Chen, M., Gupta, A., & Hashimoto, T. (2026). Meta-Harness: End-to-End Optimization of Model Harnesses.** [arXiv:2603.28052](https://arxiv.org/abs/2603.28052)
- **Concept**: "The scaffold IS the product" thesis. 6x performance gap from scaffold changes alone. +7.7 points text classification, +4.7 on IMO math, at 4x fewer tokens.
- **Roko adaptation**: Foundational for Roko's harness engineering approach. The agent framework matters more than any individual model.
- **Roko files**: `docs/v1/21-references/06-self-learning-systems.md`
- **tmp/ cross-refs**: `13-cognitive-architecture.md`, `17-agent-patterns.md`

**Khattab, O. et al. (2024). DSPy: Compiling Declarative Language Model Calls into Self-Improving Pipelines. _ICLR_.** [arXiv:2310.03714](https://arxiv.org/abs/2310.03714)
- **Concept**: Replaces hand-crafted prompts with declarative signatures; compiler optimizes prompts, few-shot examples, and fine-tuning data automatically.
- **Roko adaptation**: Influences Roko's approach to prompt budget allocation and optimization.
- **Roko files**: `docs/v2/07-LEARNING.md` (line 979)
- **tmp/ cross-refs**: `07-online-learning.md`

**SAMULE (2025). Self-Learning Agents Enhanced by Multi-level Reflection. _EMNLP_.** [arXiv:2509.20562](https://arxiv.org/abs/2509.20562)
- **Concept**: Multi-level reflection across trajectories substantially outperforms single-trajectory reflection. Error classification and clustering extract insight from failures.
- **Roko adaptation**: Validates Roko's Theta-frequency reflection operating across episodes, not just within single task runs.
- **Roko files**: `docs/v1/21-references/24-additions-2025.md`

---

## 31. Information Theory and Signal Processing

**Shannon, C.E. (1948). A Mathematical Theory of Communication. _Bell System Technical Journal_, 27, 379-423 & 623-656.**
- **Concept**: Foundational information theory: entropy, mutual information, channel capacity.
- **Roko adaptation**: Used throughout for measuring knowledge value, information flow, and communication efficiency.
- **Roko files**: `docs/v1/21-references/12-signal-processing.md`

**Clark, A. (2013). Whatever Next? Predictive Brains, Situated Agents, and the Future of Cognitive Science. _Behavioral and Brain Sciences_, 36(3), 181-204.** [DOI: 10.1017/S0140525X12000477](https://doi.org/10.1017/S0140525X12000477)
- **Concept**: Predictive processing: brains as hierarchical prediction machines that minimize prediction error. A unifying framework for perception, action, and attention.
- **Roko adaptation**: Foundational for prediction-error-driven T0/T1/T2 tier routing. The agent's cognitive tier is determined by prediction error magnitude.
- **Roko files**: `docs/v1/21-references/12-signal-processing.md`

**Landauer, R. (1961). Irreversibility and Heat Generation in the Computing Process. _IBM Journal of Research and Development_, 5(3), 183-191.** [DOI: 10.1147/rd.53.0183](https://doi.org/10.1147/rd.53.0183)
- **Concept**: Landauer's principle: erasing information has a minimum thermodynamic cost (kT ln 2 per bit).
- **Roko adaptation**: Theoretical foundation for the claim that knowledge decay (forgetting) is not free but has real computational cost.
- **Roko files**: `docs/v1/21-references/12-signal-processing.md`

---

## 32. Calibration and Uncertainty

**Guo, C., Pleiss, G., Sun, Y., & Weinberger, K.Q. (2017). On Calibration of Modern Neural Networks. _ICML_.** [arXiv:1706.04599](https://arxiv.org/abs/1706.04599)
- **Concept**: Modern neural networks are poorly calibrated (overconfident). Temperature scaling is a simple and effective calibration fix.
- **Roko adaptation**: CalibrationTracker bias correction for model routing confidence.
- **Roko files**: `docs/v2/01-SIGNAL.md` (line 1487)
- **tmp/ cross-refs**: `07-online-learning.md`

**Naeini, M.P., Cooper, G., & Hauskrecht, M. (2015). Obtaining Well Calibrated Probabilities Using Bayesian Binning into Quantiles. _AAAI_.**
- **Concept**: Expected Calibration Error (ECE): binned accuracy-confidence gaps as a calibration metric.
- **Roko adaptation**: CalibrationTracker uses ECE for measuring and correcting model confidence.
- **Roko files**: `docs/v2/01-SIGNAL.md` (line 1488), `docs/v1/20-technical-analysis/01-oracle-trait.md` (line 760)
- **tmp/ cross-refs**: `07-online-learning.md`

**Vovk, V., Gammerman, A., & Shafer, G. (2005). _Algorithmic Learning in a Random World_ (Conformal Prediction). Springer.**
- **Concept**: Distribution-free prediction intervals with guaranteed coverage. No distributional assumptions required.
- **Roko adaptation**: CalibrationTracker bounds on prediction confidence.
- **tmp/ cross-refs**: `07-online-learning.md`

**Farquhar, S. et al. (2024). Detecting Hallucinations in Large Language Models Using Semantic Entropy. _Nature_, 630.**
- **Concept**: Semantic entropy detects hallucinations by measuring consistency across sampled completions.
- **Roko adaptation**: Informs confidence estimation in Roko's Gate pipeline.
- **Roko files**: `docs/v1/21-references/11-streaming-algorithms.md`

---

## 33. Streaming Algorithms

**Bifet, A. & Gavalda, R. (2007). Learning from Time-Changing Data with Adaptive Windowing. _SIAM_, 2007.**
- **Concept**: ADWIN: adaptive window algorithm that automatically detects distribution change and adjusts window size for drift detection.
- **Roko adaptation**: Used in model routing to detect when a model's quality has shifted.
- **Roko files**: `docs/v1/21-references/11-streaming-algorithms.md`
- **tmp/ cross-refs**: `07-online-learning.md`

**Hansen, E.A. & Zilberstein, S. (2001). Monitoring and Control of Anytime Algorithms: A Survey. _Artificial Intelligence_, 126(1-2), 43-83.**
- **Concept**: Anytime algorithms produce progressively better results with more compute and can be interrupted at any point.
- **Roko adaptation**: Grounds the T0/T1/T2 cascade where the agent stops at the cheapest sufficient tier.
- **Roko files**: `docs/v1/21-references/11-streaming-algorithms.md`

---

## 34. Agent Harnesses and Tool Use

**Yang, J. et al. (2024). SWE-agent: Agent-Computer Interfaces Enable Automated Software Engineering. _NeurIPS_.**
- **Concept**: Agent-Computer Interfaces purpose-built for LLM agents. How the agent interacts with its environment matters as much as reasoning.
- **Roko adaptation**: Influences Roko's tool permissions, structured error digests, and role-specific feedback formatting.
- **Roko files**: `docs/v1/21-references/14-agent-harnesses-and-tool-use.md`
- **tmp/ cross-refs**: `17-agent-patterns.md`

**Jimenez, C.E. et al. (2024). SWE-bench: Can Language Models Resolve Real-World GitHub Issues? _ICLR_.**
- **Concept**: Benchmark of 2,294 real GitHub issues from 12 Python repositories. The gold standard for coding agent evaluation.
- **Roko files**: `docs/v1/21-references/14-agent-harnesses-and-tool-use.md`
- **tmp/ cross-refs**: `17-agent-patterns.md`

**Anthropic (2024). Building Effective Agents. anthropic.com.**
- **Concept**: Composition over complexity: keep individual agents simple, compose through a controller.
- **Roko adaptation**: Each agent role does one thing; the orchestrator composes them into pipelines.
- **Roko files**: `docs/v1/21-references/14-agent-harnesses-and-tool-use.md`
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`, `17-agent-patterns.md`

**Hu, S. et al. (2025). Automated Design of Agentic Systems (ADAS). _ICLR_.**
- **Concept**: Meta-agent that searches the space of agent architectures. Discovers novel building blocks and compositions.
- **Roko adaptation**: Roko provides the composable trait system (6 Synapse traits) that ADAS-style search operates over.
- **Roko files**: `docs/v1/21-references/06-self-learning-systems.md`

---

## 35. Process Reward Models and Verification

**Lightman, H. et al. (2024). Let's Verify Step by Step.** [arXiv:2305.20050](https://arxiv.org/abs/2305.20050)
- **Concept**: Process reward models that verify each reasoning step outperform outcome-only verification for mathematical reasoning.
- **Roko adaptation**: Per-gate scoring in the 7-rung verification ladder. Each gate checks a specific quality dimension.
- **tmp/ cross-refs**: `05-gate-verification.md`

**Song, Y. et al. (2025). Mind the Gap: Examining the Self-Improvement Capabilities of Large Language Models. _ICLR_.**
- **Concept**: The Generation-Verification Gap: self-improvement works only when verification ability exceeds generation ability. If the verifier is weaker than the generator, feedback is noise.
- **Roko adaptation**: Foundational result validating the separation of agent (generator) and Gate (verifier).
- **Roko files**: `docs/v2-depth/02-block/eval-lifecycle-and-generation.md` (line 172)
- **tmp/ cross-refs**: `05-gate-verification.md`

**Huang, J. et al. (2024). Large Language Models Cannot Self-Correct Reasoning Yet. _ICLR_.**
- **Concept**: LLMs self-correcting without external feedback typically make answers worse. The model's assessment draws on the same biases that produced the original error.
- **Roko adaptation**: External verification mandate. Motivates external Gates rather than self-assessment.
- **tmp/ cross-refs**: `05-gate-verification.md`

**Wei, J. et al. (2022). Chain-of-Thought Prompting Elicits Reasoning in Large Language Models. _NeurIPS_.**
- **Concept**: Chain-of-thought makes reasoning steps explicit and individually verifiable.
- **Roko adaptation**: CoT as a means to make reasoning verifiable per-step in the gate pipeline.
- **Roko files**: `docs/v1/21-references/17-process-reward-models.md`
- **tmp/ cross-refs**: `05-gate-verification.md`

---

## 36. Reinforcement Learning Foundations

**Andrychowicz, M. et al. (2017). Hindsight Experience Replay. _NeurIPS_.** [arXiv:1707.01495](https://arxiv.org/abs/1707.01495)
- **Concept**: HER: re-label failed trajectories with the goals they actually achieved, enabling learning from failure in sparse-reward environments.
- **Roko adaptation**: Phase 2 of dream consolidation (Hindsight Relabeling). Failed trajectories are decomposed into sub-goals; achieved sub-goals are relabeled as positive episodes. Recovers useful learning signal from at least 45% of otherwise-discarded episodes.
- **Roko files**: `docs/v2/02-CELL.md` (lines 392, 1724), `docs/v2/07-LEARNING.md` (line 407)
- **tmp/ cross-refs**: `02-dream-consolidation.md`, `07-online-learning.md`

**Rescorla, R.A. & Wagner, A.R. (1972). A Theory of Pavlovian Conditioning: Variations in the Effectiveness of Reinforcement and Nonreinforcement. In _Classical Conditioning II_. Appleton-Century-Crofts.**
- **Concept**: Prediction error learning: learning is driven by the discrepancy between expected and actual outcomes (delta rule).
- **Roko adaptation**: The simplest form of prediction error that drives T0 probes and CalibrationTracker updates.
- **Roko files**: `docs/v1/21-references/16-active-inference.md`

**Doya, K. (2002). Metalearning and Neuromodulation. _Neural Networks_, 15(4-6), 495-506.** [DOI: 10.1016/S0893-6080(02)00044-8](https://doi.org/10.1016/S0893-6080(02)00044-8)
- **Concept**: Different neuromodulators (dopamine, serotonin, norepinephrine, acetylcholine) control different meta-parameters of learning.
- **Roko adaptation**: Maps to the 7-axis Score where different axes control different aspects of knowledge management.
- **Roko files**: `docs/v1/21-references/16-active-inference.md`

---

## 37. IIT and Consciousness Metrics

**Tononi, G. (2004). An Information Integration Theory of Consciousness. _BMC Neuroscience_, 5, 42.** [DOI: 10.1186/1471-2202-5-42](https://doi.org/10.1186/1471-2202-5-42)
- **Concept**: Integrated Information Theory (IIT): Phi measures how much a system is "more than the sum of its parts." High Phi indicates genuine information integration.
- **Roko adaptation**: IitPhiMetric computes Phi over N subsystems from a mutual information matrix, estimating irreducible information integration.
- **Crate**: `roko-daimon` (somatic_ta.rs)
- **Roko files**: `crates/roko-daimon/src/somatic_ta.rs` (line 8, lines 27-28)
- **tmp/ cross-refs**: `03-affect-engine.md`

---

## Additional Notable References

**Arbesman, S. (2012). _The Half-Life of Facts: Why Everything We Know Has an Expiration Date_. Current/Penguin.**
- **Concept**: Per-domain factual decay rates. Different types of knowledge expire at measurable, distinct rates.
- **Roko adaptation**: Per-type half-life calibration in the NeuroStore.
- **tmp/ cross-refs**: `10-universal-engram.md`

**Hayek, F.A. (1945). The Use of Knowledge in Society. _American Economic Review_, 35(4), 519-530.**
- **Concept**: Distributed knowledge aggregation. The price system as a distributed information processing mechanism.
- **Roko adaptation**: Pheromone field as a price-like aggregation system for distributed agent knowledge.
- **tmp/ cross-refs**: `15-orchestrator-swarm.md`

**Kauffman, S.A. (1993). _The Origins of Order: Self-Organization and Selection in Evolution_. Oxford University Press.**
- **Concept**: Self-organized criticality. Systems near the "edge of chaos" optimize adaptability -- too much order stagnates, too much chaos dissolves structure.
- **Roko adaptation**: Adaptive clock targets edge of chaos for optimal balance of stability and adaptability.

**Sterling, P. (2012). Allostasis: A Model of Predictive Regulation. _Physiology & Behavior_, 106(1), 86-93.** [DOI: 10.1016/j.physbeh.2011.06.004](https://doi.org/10.1016/j.physbeh.2011.06.004)
- **Concept**: Allostasis: predictive regulation that anticipates needs rather than reacting to deviations (homeostasis).
- **Roko adaptation**: Predictive foraging -- agents anticipate which knowledge they will need rather than retrieving reactively.

**Maxwell, J.C. (1868). On Governors. _Proceedings of the Royal Society of London_, 16, 270-283.**
- **Concept**: First mathematical analysis of feedback control. The governor prevents steam engine oscillation through proportional damping.
- **Roko adaptation**: Foundational for the adaptive clock control system.

**Dohare, S. et al. (2024). Loss of Plasticity in Deep Continual Learning. _Nature_, 632.** [DOI: 10.1038/s41586-024-07711-7](https://doi.org/10.1038/s41586-024-07711-7)
- **Concept**: 90% of units become "dead" (non-updating) in continual learning systems. Periodic replacement outperforms continuous adaptation.
- **Roko adaptation**: Validates knowledge decay and the Curator's pruning cycle. Knowledge stores need active maintenance.
- **Roko files**: `docs/v1/21-references/00-lifecycle-and-finite-agency.md`

**Sculley, D. et al. (2015). Hidden Technical Debt in Machine Learning Systems. _NeurIPS_.**
- **Concept**: Technical debt compounds silently in long-running ML systems. Only a small fraction of code in ML systems is the model itself.
- **Roko adaptation**: Controlled knowledge decay prevents silent accumulation of stale heuristics.

---

## Key Papers

The following 15 papers are the most foundational to understanding Roko's architecture. A researcher reading only these would grasp the core theoretical framework:

1. **Kanerva (2009)** -- Hyperdimensional Computing introduction. Establishes the 10,240-bit BSC substrate that underpins all knowledge representation. [DOI: 10.1007/s12559-009-9009-8](https://doi.org/10.1007/s12559-009-9009-8)

2. **Friston (2006)** -- Free Energy Principle. The core routing principle: every decision in the system minimizes expected free energy. [DOI: 10.1016/j.jphysparis.2006.10.001](https://doi.org/10.1016/j.jphysparis.2006.10.001)

3. **McClelland, McNaughton, & O'Reilly (1995)** -- Complementary Learning Systems. Fast episodic + slow semantic memory is the blueprint for NeuroStore's tiered architecture. [DOI: 10.1037/0033-295X.102.3.419](https://doi.org/10.1037/0033-295X.102.3.419)

4. **Mattar & Daw (2018)** -- Prioritized Memory Access. The algorithm that selects which experiences to replay during dream consolidation. [DOI: 10.1038/s41593-018-0232-z](https://doi.org/10.1038/s41593-018-0232-z)

5. **Kahneman (2011)** -- Thinking, Fast and Slow. Dual-process theory is the direct blueprint for the T0/T1/T2 cascade.

6. **Damasio (1994)** -- Descartes' Error. Somatic markers prove that emotion is computational, not decorative. Foundation for the entire Daimon subsystem.

7. **Woolley et al. (2010)** -- Collective Intelligence Factor. The C-Factor metric that gates structural evolution of agent collectives. [DOI: 10.1126/science.1193147](https://doi.org/10.1126/science.1193147)

8. **Beer (1972)** -- Brain of the Firm (Viable System Model). The recursive 5-system architecture that maps directly to the conductor hierarchy.

9. **Vickrey (1961)** -- Second-price auctions. Truthful mechanism design is the foundation for the VCG context budget allocation. [DOI: 10.1111/j.1540-6261.1961.tb02789.x](https://doi.org/10.1111/j.1540-6261.1961.tb02789.x)

10. **Grasse (1959)** -- Stigmergy. Coordination without communication. The Agent Mesh pheromone field is a direct implementation.

11. **Richards & Frankland (2017)** -- Forgetting as regularization. Forgetting is L1 regularization, not information loss. Foundation for demurrage architecture. [DOI: 10.1016/j.neuron.2017.04.037](https://doi.org/10.1016/j.neuron.2017.04.037)

12. **Sumers et al. (2023)** -- CoALA (Cognitive Architectures for Language Agents). The reference cognitive architecture that Roko extends with Gate and Daimon. [arXiv:2309.02427](https://arxiv.org/abs/2309.02427)

13. **Lee et al. (2026)** -- Meta-Harness. Empirical proof that the scaffold IS the product -- 6x performance gap from framework changes alone. [arXiv:2603.28052](https://arxiv.org/abs/2603.28052)

14. **Ebbinghaus (1885)** -- Forgetting Curve. The negative exponential decay directly implemented as per-type half-lives in `roko-core/src/decay.rs`.

15. **Kahneman & Tversky (1979)** -- Prospect Theory. Loss aversion (lambda=2.25) is hardcoded into the Daimon's appraisal pipeline. [DOI: 10.2307/1914185](https://doi.org/10.2307/1914185)

---

## Summary Statistics

- **Total unique papers/references cited**: approximately 200+
- **Topic areas covered**: 37 categories
- **Roko crates with research-grounded implementations**: roko-core, roko-daimon, roko-dreams, roko-learn, roko-neuro, roko-conductor, roko-gate, roko-compose, roko-primitives, roko-chain, roko-index, roko-agent, roko-orchestrator, roko-graph
- **Primary reference directories in roko**: `docs/v1/21-references/` (27 files), `docs/v2-depth/21-roadmap/07-academic-foundations-by-protocol.md`
- **Time span of references**: 1868 (Maxwell) to 2026 (Zhang ACE, Lee Meta-Harness, arXiv preprints)
- **Disciplines represented**: Neuroscience, cognitive psychology, economics, computer science, philosophy, evolutionary biology, control theory, topology, mechanism design, information theory, signal processing, market microstructure, formal verification
- **tmp/ documents cross-referenced**: 01-hyperdimensional-computing, 02-dream-consolidation, 03-affect-engine, 05-gate-verification, 06-conductor-anomaly, 07-online-learning, 08-chain-reputation, 09-budget-composition, 10-universal-engram, 11-mathematical-primitives, 13-cognitive-architecture, 15-orchestrator-swarm, 17-agent-patterns, 21-mcp-editor-integration, 24-smart-contracts

# Hyperdimensional Computing (HDC) — Overview

**Source crates**: `roko-primitives`, `roko-neuro`, `roko-index`
**Priority**: HIGH — drop-in similarity engine for memory, skill matching, tool selection
**Status**: Production-ready in roko; proposed for IronClaw `ironclaw_hdc` crate

---

## What Is HDC?

Hyperdimensional Computing (HDC), also known as Vector Symbolic Architectures (VSA), represents information as 10,240-bit binary vectors and manipulates them with four simple algebraic operations. No floating-point arithmetic. No GPU. No model inference. A single similarity comparison completes in ~13 nanoseconds.

The core insight: **in sufficiently high-dimensional spaces, random vectors are almost certainly near-orthogonal.** Any measured similarity significantly above 0.5 is therefore a genuine structural signal, not noise.

**What HDC is NOT**: HDC is not a neural network. It does not learn weights. It provides a **compositional algebra** for building structured representations from atomic symbols — like how arithmetic lets you compose numbers with +, −, ×.

---

## Table of Contents

| Document | Contents |
|----------|---------|
| [theory.md](./theory.md) | Mathematical foundations, VSA variants, all four operations, capacity analysis, why 10,240 bits |
| [implementation.md](./implementation.md) | Full roko source code — HdcVector, Codebook, Accumulators, ItemMemory, PatternStore |
| [applications.md](./applications.md) | 6 application areas with worked examples and concrete similarity values |
| [benchmarking.md](./benchmarking.md) | Performance benchmarks, false positive analysis, A/B comparison methodology |
| [ironclaw-integration.md](./ironclaw-integration.md) | Full IronClaw implementation plan — 6-week phased integration with Rust code |
| [references.md](./references.md) | All academic citations with DOIs and annotations |

---

## Architecture Diagram

```mermaid
graph TD
    subgraph CRATE["ironclaw_hdc crate\n(new, zero external deps)"]
        HC["HdcVector\nvector.rs"]
        CB["Codebook\ncodebook.rs"]
        AC["BundleAccumulator\nDecayingBundleAccumulator\nvector.rs"]
        IM["ItemMemory\nvector.rs"]
        ENC["HdcEncodable trait\nencoder.rs"]
    end

    subgraph MEMORY["Memory Layer\ncrates/ironclaw_memory/"]
        MW["memory_write\n+ hdc_fingerprint column"]
        MS["memory_search\n+ HDC as third RRF signal"]
    end

    subgraph SKILLS["Skills Layer\ncrates/ironclaw_skills/"]
        SS["SkillIndex\n(pre-computed fingerprints)"]
        SC["score_skill()\n+ HDC boost signal"]
    end

    subgraph TOOLS["Tools Layer\nsrc/tools/registry.rs"]
        TI["ToolHdcIndex\nfingerprint per tool"]
        TS["suggest(intent)\ntop-K by similarity"]
    end

    subgraph WORKSPACE["Workspace Layer\nsrc/workspace/"]
        ND["NoveltyDetector\nDecayingBundleAccumulator"]
        DED["Deduplication\nfind_near_duplicate()"]
    end

    HC --> CB
    HC --> AC
    HC --> IM
    HC --> ENC
    ENC --> MW
    ENC --> SS
    ENC --> TI
    ENC --> ND
    ENC --> DED
    HC --> MS
    CB --> SS
    CB --> TI
    AC --> ND
    IM --> MS
    IM --> SC

    style CRATE fill:#e8f4f8,stroke:#2980b9
    style MEMORY fill:#e8f8e8,stroke:#27ae60
    style SKILLS fill:#f8f0e8,stroke:#d35400
    style TOOLS fill:#f8e8f8,stroke:#8e44ad
    style WORKSPACE fill:#f8f8e8,stroke:#f39c12
```

---

## Quick Intro

### Key Terminology

| Term | Meaning |
|---|---|
| **Hypervector (HV)** | A vector with 10,240 binary dimensions |
| **Hamming similarity** | 1 - (differing bits / 10,240); ranges 0 (complement) to 1 (identical) |
| **Quasi-orthogonal** | Two random vectors with ~0.5 similarity — functionally independent |
| **Bind (XOR)** | Combines two HVs into a new one quasi-orthogonal to both inputs |
| **Bundle (majority vote)** | Superimposes multiple HVs into one similar to all inputs |
| **Permute (cyclic shift)** | Rotates bits to encode position/sequence order |
| **Codebook** | Dictionary mapping symbolic names to deterministic hypervectors |
| **BSC** | Binary Spatter Codes — the specific HDC variant used here |
| **VSA** | Vector Symbolic Architectures — the broader family |

### The Four Operations at a Glance

```
bind(A, B)          = A XOR B          # associates two concepts; involutory
bundle(A, B, C)     = majority_vote    # superimposes; result similar to all
permute(A, k)       = cyclic_shift(k)  # encodes position; breaks commutativity
similarity(A, B)    = 1 - hamming/D    # [0, 1]; > 0.526 is statistically significant
```

### Similarity Interpretation

| Similarity | Meaning |
|---|---|
| 1.0 | Identical |
| > 0.526 | Genuine structural relationship (< 1% FP scanning 100K entries) |
| 0.485–0.515 | Noise band — quasi-orthogonal, no relationship |
| < 0.48 | Meaningful dissimilarity |
| 0.0 | Bitwise complement |

### Performance at a Glance

| Operation | Time |
|---|---|
| `similarity()` | ~13 ns |
| `bind()` | ~5 ns |
| `permute()` | ~10 ns |
| `bundle()` (K=10) | ~800 ns |
| Scan 100K entries | ~1.3 ms |

---

## Prerequisites

A reader with no prior HDC knowledge can implement the system from scratch after reading these documents. Helpful background:

- **Linear algebra basics**: vectors, dot products, orthogonality
- **Probability/statistics**: normal distributions, standard deviations
- **Boolean algebra**: XOR, AND, OR, bitwise operations
- **Rust**: `[u64; 160]` fixed-size arrays, traits, generics

**Not required**: machine learning, neural networks, GPU programming, floating-point arithmetic.

---

## Reading Order

1. **Start here** — this README for orientation
2. [theory.md](./theory.md) — understand the math before reading code
3. [implementation.md](./implementation.md) — full source code with GitHub links
4. [applications.md](./applications.md) — worked examples for each use case
5. [benchmarking.md](./benchmarking.md) — performance data and FP analysis
6. [ironclaw-integration.md](./ironclaw-integration.md) — the IronClaw plan

---

Next: [Theory and Mathematical Foundations](./theory.md)

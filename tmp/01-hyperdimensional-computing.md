# Hyperdimensional Computing (HDC)

**Source crates**: `roko-primitives`, `roko-neuro`, `roko-index`
**Priority**: HIGH -- drop-in similarity engine for memory, skill matching, tool selection
**Status**: Production-ready in roko; proposed for IronClaw integration

---

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [What Is Hyperdimensional Computing?](#2-what-is-hyperdimensional-computing)
3. [Mathematical Foundations](#3-mathematical-foundations)
4. [Why 10,240 Bits?](#4-why-10240-bits)
5. [Core Data Structure: HdcVector](#5-core-data-structure-hdcvector)
6. [The Four Operations](#6-the-four-operations)
7. [Codebooks and Symbol Allocation](#7-codebooks-and-symbol-allocation)
8. [Accumulators: Incremental and Decaying Bundling](#8-accumulators-incremental-and-decaying-bundling)
9. [ItemMemory: Named Concept Lookup](#9-itemmemory-named-concept-lookup)
10. [Application 1: Knowledge Fingerprinting (roko-neuro)](#10-application-1-knowledge-fingerprinting-roko-neuro)
11. [Application 2: Role-Filler Structured Encoding (roko-neuro)](#11-application-2-role-filler-structured-encoding-roko-neuro)
12. [Application 3: Cross-Domain Resonance Detection (roko-neuro)](#12-application-3-cross-domain-resonance-detection-roko-neuro)
13. [Application 4: Code Fingerprinting (roko-index)](#13-application-4-code-fingerprinting-roko-index)
14. [Application 5: Admission Control and AntiKnowledge (roko-neuro)](#14-application-5-admission-control-and-antiknowledge-roko-neuro)
15. [Application 6: Context Assembly Scoring (roko-neuro)](#15-application-6-context-assembly-scoring-roko-neuro)
16. [False Positive Analysis and Threshold Selection](#16-false-positive-analysis-and-threshold-selection)
17. [Performance Characteristics](#17-performance-characteristics)
18. [IronClaw Integration Plan](#18-ironclaw-integration-plan)
19. [Complexity Assessment](#19-complexity-assessment)
20. [Academic References](#20-academic-references)

---

## 1. Prerequisites

This document is written to be self-contained, but the following background will help a reader absorb it more quickly:

**Helpful background** (not required):

- **Linear algebra basics**: familiarity with vectors, dot products, and the concept of orthogonality. HDC extends these intuitions from continuous to binary spaces.
- **Probability and statistics**: understanding of normal distributions, standard deviations, and confidence intervals. The core mathematical argument of HDC is a statistical concentration result.
- **Boolean algebra**: XOR, AND, OR, bitwise operations. All HDC operations reduce to bitwise instructions.
- **Hash functions**: conceptual understanding of how a hash maps arbitrary data to fixed-size output. HDC's `from_seed()` uses this as a building block, but the output is a structured algebraic object rather than a flat hash.
- **Rust**: the code examples are in Rust with `#[derive]` macros, traits, and generics. Familiarity with `[u64; 160]` fixed-size arrays and the `Copy` trait helps.

**What you do NOT need**:

- No machine learning or neural network background is required. HDC is not a neural network and does not learn weights.
- No GPU programming or floating-point arithmetic knowledge is needed.
- No familiarity with the roko or IronClaw codebases is assumed; all referenced types are explained inline.

**Key terminology used throughout**:

| Term | Meaning |
|---|---|
| **Hypervector (HV)** | A vector with thousands of binary dimensions (10,240 bits in this system) |
| **Hamming distance** | Number of bit positions where two binary vectors differ |
| **Hamming similarity** | 1 - (Hamming distance / D); ranges from 0 (complement) to 1 (identical) |
| **Quasi-orthogonal** | Two random hypervectors with ~0.5 similarity (chance level); functionally independent |
| **Bind (XOR)** | Combines two HVs into a new HV that is quasi-orthogonal to both inputs |
| **Bundle (majority vote)** | Superimposes multiple HVs into one that is similar to all inputs |
| **Permute (cyclic shift)** | Rotates bits to encode position/sequence order |
| **Codebook** | A dictionary mapping symbolic names to deterministic hypervectors |
| **BSC** | Binary Spatter Codes -- the specific HDC variant used here, operating on binary vectors with XOR binding |

---

## 2. What Is Hyperdimensional Computing?

Hyperdimensional Computing (HDC), also known as Vector Symbolic Architectures (VSA), is a computational framework that represents information as high-dimensional binary vectors and manipulates them with simple algebraic operations. The approach was first explored by Pentti Kanerva in 1988 in the context of Sparse Distributed Memory [Kanerva88], formalized as the Binary Spatter Codes (BSC) model [Kanerva96], and given a systematic treatment as a computing paradigm in his 2009 paper [Kanerva09]. The term "Vector Symbolic Architectures" was introduced by Ross Gayler in 2003 to unify the family of related models [Gayler03].

The core insight is deceptively simple: **in sufficiently high-dimensional spaces (thousands of bits), random vectors are almost certainly near-orthogonal**. This mathematical fact has profound computational implications:

1. **Capacity**: A space of D-bit vectors can represent an exponential number of distinct concepts without interference. Each randomly generated vector is, with overwhelming probability, distinguishable from every other.

2. **Composability**: Vectors can be combined using algebraic operations (XOR binding, majority-vote bundling, cyclic-shift permutation) that produce new vectors while preserving the ability to detect component parts. You can encode "Rust is the language" as a single vector, compose it with "async is the topic," and later recover either component.

3. **Robustness**: Similarity is distributed across many bits, so small perturbations (noise, missing data, partial information) do not destroy the overall structure. A vector with 5% of its bits flipped retains 95% similarity to the original.

4. **Efficiency**: All operations reduce to bitwise CPU instructions (XOR, popcount, shift). No floating-point arithmetic. No GPU. No model inference. A single similarity comparison completes in approximately 13 nanoseconds on modern hardware.

HDC belongs to the family of Vector Symbolic Architectures (VSA), which includes several variants:

| VSA Variant | Binding Operation | Vector Type | Key Reference |
|---|---|---|---|
| **Binary Spatter Codes (BSC)** | XOR | Binary {0,1} | Kanerva, 1996 [Kanerva96] |
| Multiply-Add-Permute (MAP) | Element-wise multiplication | Bipolar {-1,+1} | Gayler, 2003 [Gayler03] |
| Holographic Reduced Representations (HRR) | Circular convolution | Real-valued | Plate, 1995 [Plate95] |
| Fourier HRR (FHRR) | Element-wise complex multiply | Complex unit phasors | Plate, 2003 [Plate03] |

BSC was chosen for roko because it offers the best combination of simplicity (binary vectors, XOR binding), performance (pure bitwise operations on CPU), and capacity at the chosen dimensionality. The trade-off is that BSC has lower representational precision per dimension than real-valued variants like HRR, but this is compensated by using more dimensions (10,240 bits vs. typical HRR at 512-2048 real dimensions).

**What HDC is NOT**: HDC is not a neural network. It does not learn weights. It does not require training data. It does not approximate a function. Instead, it provides a **compositional algebra** for building structured representations from atomic symbols, much like how algebra lets you compose numbers with +, -, and x. The resulting vectors encode semantic structure (not just content hashes), support approximate matching (not just exact equality), and enable reasoning operations (unbinding, decomposition, analogy detection) that are impossible with traditional hashing.

> *Source: `docs/v1/06-neuro/04-hdc-vsa-foundations.md`, `docs/v1/21-references/09-hdc-vsa.md`*

---

## 3. Mathematical Foundations

### The Concentration of Measure Phenomenon

The mathematical foundation of HDC rests on the **concentration of measure** in high-dimensional spaces. For two independently generated random binary vectors of dimension D, each bit is set to 0 or 1 with equal probability. The Hamming similarity (fraction of matching bits) between any two such random vectors follows a binomial distribution that, for large D, is well approximated by a normal distribution:

```
Expected value:     mu = 0.5
Variance:           sigma^2 = 1 / (4D)
Standard deviation: sigma = 1 / (2 * sqrt(D))
```

**Derivation**: Each bit position contributes a Bernoulli random variable: if both vectors have the same bit, the similarity contribution is 1/D; if different, 0. Each position matches with probability 0.5 (since both bits are independent fair coin flips). The total similarity is the average of D independent Bernoulli(0.5) variables, so the variance of the mean is (0.5 * 0.5) / D = 1/(4D).

For D = 10,240:

```
sigma = 1 / (2 * sqrt(10240)) = 1 / (2 * 101.19) = 0.00494
```

This means any two random 10,240-bit vectors will have Hamming similarity within the extremely narrow band of 0.5 +/- 0.015 (3 sigma) with 99.7% probability. In practical terms, random vectors are **quasi-orthogonal** -- they share almost exactly 50% of their bits, and this fraction varies by less than 1.5 percentage points. Any similarity significantly above 0.5 is therefore a genuine structural signal, not noise.

**Worked example**: Generate two random 10,240-bit vectors A and B. They will share approximately 5,120 +/- 50.6 bits (the standard deviation of the count of matching bits is sqrt(D * 0.25) = 50.6). A similarity of 0.53 means 5,427 matching bits -- this is 6.1 standard deviations above the mean, which happens by chance with probability less than 1 in a billion.

### The Johnson-Lindenstrauss Connection

The Johnson-Lindenstrauss (JL) lemma [JL84] provides theoretical backing for dimensionality choices. It states that N points in high-dimensional space can be projected into D dimensions while preserving pairwise distances within a factor of (1 +/- epsilon), provided D is at least O(log(N) / epsilon^2).

The exact constant depends on the variant of the lemma. For random binary projections, a practical bound is:

```
D >= C * ln(N) / epsilon^2
```

where C is a constant that depends on the specific construction (typically between 4 and 24 in the literature; the Achlioptas 2003 variant uses C = 4 for sparse random projections).

**Important caveat**: The JL lemma applies to continuous random projections and does not directly govern the capacity of binary spatter codes. However, it provides useful intuition: 10,240 dimensions are far more than needed to faithfully embed millions of points at reasonable distortion tolerances. Even with the most conservative constant (C = 24), embedding one million points at 10% distortion requires only D >= 24 * ln(10^6) / 0.01 = 33,142 dimensions -- well within reach at D = 10,240 if distortion tolerance is relaxed to ~15%.

The practical capacity of BSC vectors is better characterized by the bundle SNR model (below) and the false positive analysis (Section 16).

### Bundle Capacity: The Signal-to-Noise Ratio

When K vectors are bundled (superimposed via majority vote), the resulting composite vector is similar to each input. The quality of this similarity degrades as K increases, governed by the signal-to-noise ratio:

```
SNR = sqrt(D / K)
```

**Derivation**: After bundling K random vectors, each input vector contributes a signal of 1/K to each bit position, while the noise from the other K-1 vectors contributes a variance of (K-1)/(4K^2) per bit. Aggregating over D bits, the SNR scales as sqrt(D/K). See [Kleyko22] Section 4.2 for a rigorous derivation.

For D = 10,240:

| K (bundled items) | SNR | Retrieval quality |
|---|---|---|
| 10 | 32.0 | Excellent -- each component reliably retrievable |
| 50 | 14.3 | Good -- components detectable above noise floor |
| 100 | 10.1 | Marginal -- useful as pre-filter, not for exact retrieval |
| 1,000 | 3.2 | Poor -- only dominant components survive |

The practical limit for reliable component retrieval from a bundle is approximately K < 100 at D = 10,240. This is consistent with the capacity analysis in [Thomas21] which establishes K = O(D / log(D)) as the theoretical maximum.

> *Source: `docs/v1/06-neuro/04-hdc-vsa-foundations.md`, `docs/v1/06-neuro/09-false-positive-math.md`*

---

## 4. Why 10,240 Bits?

The choice of 10,240 bits (160 u64 words, 1,280 bytes per vector) is deliberate and documented in the roko design:

| Dimension D | Bundle capacity (SNR > 10) | Noise band (2-sigma) | Storage per vector | Use case |
|---|---|---|---|---|
| 1,024 | ~10 items | +/-3.1% | 128 bytes | Toy / embedded |
| 4,096 | ~40 items | +/-1.6% | 512 bytes | Small vocabularies |
| **10,240** | **~100 items** | **+/-1.0%** | **1,280 bytes** | **Production knowledge systems** |
| 65,536 | ~650 items | +/-0.4% | 8,192 bytes | Very large vocabularies |

*Note: "Noise band" is 2 standard deviations (95.4% confidence interval) of the Hamming similarity between random vectors. "Bundle capacity" is the maximum K for bundle SNR > 10 (reliable individual component retrieval).*

The 10,240-bit choice balances three engineering concerns:

1. **Sufficient capacity**: A codebook of ~10,000 deterministic symbols remains cleanly separable (all pairwise similarities within the noise band around 0.5). Bundles of up to ~100 items retain reliable component retrieval. This supports workspace-scale indexing and agent knowledge bases with thousands of entries.

2. **Precision**: +/-1.0% noise band (2-sigma) means a similarity threshold of 0.526 (just 2.6% above the 0.5 baseline) is statistically meaningful at p < 10^-7 per comparison, and remains significant even after Bonferroni correction against 100K comparisons.

3. **Performance**: 160 words x 8 bytes = 1,280 bytes per fingerprint. The entire vector fits comfortably in L1 cache (typically 32-64 KB). XOR + popcount over 160 words completes in ~13 ns on modern x86 hardware with auto-vectorized SIMD.

The number 10,240 = 160 x 64 is also chosen for alignment: it maps exactly to 160 machine words with no padding or waste.

> *Source: `docs/v1/15-code-intelligence/05-hdc-fingerprints.md`, `crates/roko-primitives/src/hdc.rs:8-10`*

---

## 5. Core Data Structure: HdcVector

The foundation of the entire HDC system is the `HdcVector` type, defined in `crates/roko-primitives/src/hdc.rs`. This is a 10,240-bit binary vector stored as a fixed-size array of 160 u64 words.

### Type Definition

```rust
/// Number of bits in one HdcVector.
pub const HDC_BITS: usize = 10_240;
/// Number of serialized bytes in one HdcVector.
pub const HDC_BYTES: usize = 1_280;

/// 10,240-bit binary sparse distributed vector.
///
/// Three core operations: XOR bind, majority-vote bundle, Hamming similarity.
/// All operations are CPU-cache-friendly bit manipulation -- no floating point,
/// no matrix multiply, no GPU required.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct HdcVector {
    bits: [u64; 160],
}
```

*Source: `crates/roko-primitives/src/hdc.rs:7-32`*

Key design choices:

- **`Copy` trait**: The entire 1,280-byte vector implements `Copy`. This means it is stack-allocated and passed by value. While this is larger than typical `Copy` types, it avoids heap allocation and reference counting overhead for the most common operations (bind, similarity).

- **`[u64; 160]` not `Vec<u64>`**: Fixed-size array, not a heap-allocated vector. This enables `const` constructors, stack allocation, and compile-time size guarantees.

- **`rkyv` support**: With the `rkyv` feature flag, vectors can be zero-copy deserialized from memory-mapped files. The archived representation of `[u64; 160]` on little-endian platforms is identical to the in-memory layout.

- **Custom serde**: The `Serialize`/`Deserialize` implementations encode the vector as a 1,280-byte blob, not as 160 individual u64 values. This is critical for compact JSON and binary serialization.

### Construction Methods

**Random vector** (used for initial codebook symbols):

```rust
/// Returns a pseudo-random vector seeded from a random UUID.
pub fn random() -> Self {
    let seed = Uuid::new_v4().as_u128();
    // ... mix into splitmix64 state, fill 160 words
}
```

*Source: `crates/roko-primitives/src/hdc.rs:92-109`*

**Deterministic seeded vector** (the workhorse -- maps any byte sequence to a reproducible vector):

```rust
/// Create a deterministic vector from a byte seed.
///
/// Uses FNV-1a to hash the seed into a 64-bit state, then splitmix64 to fill bits.
/// Identical seeds always produce identical vectors.
pub fn from_seed(seed: &[u8]) -> Self {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325; // FNV-1a offset basis
    for &byte in seed {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3); // FNV prime
    }
    if hash == 0 {
        hash = 0xA5A5_A5A5_5A5A_5A5A;
    }
    let mut bits = [0u64; 160];
    for word in &mut bits {
        *word = splitmix64(&mut hash);
    }
    Self { bits }
}
```

*Source: `crates/roko-primitives/src/hdc.rs:205-220`*

The PRNG chain is `FNV-1a hash -> splitmix64 expansion`. FNV-1a is used because it is fast, has good avalanche properties for short inputs, and is deterministic. Splitmix64 then expands the 64-bit hash into 160 x 64 = 10,240 pseudorandom bits. The key property is that `from_seed(b"rust")` **always produces the same vector**, across runs, across machines, across architectures. This determinism is what makes the entire codebook and role-binding system work without any coordination or shared state.

**Zero vector** (identity for bundling):

```rust
pub const fn zeros() -> Self {
    Self { bits: [0; 160] }
}
```

*Source: `crates/roko-primitives/src/hdc.rs:86-88`*

### Serialization

```rust
/// Serialize the vector to 1280 little-endian bytes.
pub fn to_bytes(&self) -> [u8; 1280] {
    let mut out = [0u8; 1280];
    for (i, word) in self.bits.iter().enumerate() {
        out[i * 8..(i + 1) * 8].copy_from_slice(&word.to_le_bytes());
    }
    out
}

/// Deserialize a vector from 1280 little-endian bytes.
pub fn from_bytes(bytes: &[u8; 1280]) -> Self {
    let mut bits = [0u64; 160];
    for (i, word) in bits.iter_mut().enumerate() {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
        *word = u64::from_le_bytes(buf);
    }
    Self { bits }
}
```

*Source: `crates/roko-primitives/src/hdc.rs:178-198`*

### Convenience Fingerprinting Functions

Two module-level functions provide common shortcuts:

```rust
/// Compute a deterministic HDC fingerprint for any serializable value.
pub fn fingerprint(value: &impl serde::Serialize) -> HdcVector {
    let seed = serde_json::to_vec(value).unwrap_or_default();
    HdcVector::from_seed(&seed)
}

/// Compute a deterministic HDC fingerprint for raw text.
pub fn text_fingerprint(text: &str) -> HdcVector {
    HdcVector::from_seed(text.as_bytes())
}
```

*Source: `crates/roko-primitives/src/hdc.rs:509-523`*

---

## 6. The Four Operations

HDC provides four algebraic operations that, together, can encode arbitrarily complex structured data into fixed-size binary vectors. Each operation has specific algebraic properties that make it suitable for a particular role in knowledge representation.

### 6.1 Bind (XOR)

**Purpose**: Associates two concepts into a new vector that is quasi-orthogonal to both inputs. Encodes a typed relationship: "Rust in the language role."

**Mathematical definition**:
```
bind(A, B) = A XOR B    (componentwise exclusive-or)
```

**Algebraic properties**:

| Property | Formula | Significance |
|---|---|---|
| Self-inverse (involution) | bind(bind(A, B), B) = A | No separate "unbind" needed; XOR is its own inverse |
| Commutative | bind(A, B) = bind(B, A) | Role-filler binding is symmetric unless permute is used |
| Associative | bind(A, bind(B, C)) = bind(bind(A, B), C) | Multi-way binding can be done in any order |
| Distributes over bundle | bind(A, bundle(B, C)) ~ bundle(bind(A, B), bind(A, C)) | Structured queries work |
| Orthogonality-preserving | sim(bind(A, B), A) ~ 0.5 | The result is a genuinely new vector, not a blend |

The self-inverse property is the key insight that makes structured queries possible. If you encode `record = bundle(bind(role_language, hv_rust), bind(role_topic, hv_async))`, you can query "what language?" by unbinding the role: `answer = bind(record, role_language)`. The answer will be approximately `hv_rust`.

**Rust implementation**:

```rust
/// Binds two vectors using XOR. Involution: `bind(bind(a, b), b) == a`.
pub fn bind(&self, other: &Self) -> Self {
    let mut bits = [0u64; 160];
    for (slot, (left, right)) in bits.iter_mut().zip(self.bits.iter().zip(other.bits.iter())) {
        *slot = left ^ right;
    }
    Self { bits }
}
```

*Source: `crates/roko-primitives/src/hdc.rs:113-119`*

**Performance**: 160 XOR operations on u64 words. Approximately 5 ns with scalar code, approximately 2 ns with AVX-512 auto-vectorization. Each word is independent, making this embarrassingly parallel.

### 6.2 Bundle (Majority Vote)

**Purpose**: Superimposes multiple vectors into a single aggregate that is **similar to all inputs**. It is a "set union" in hypervector space.

**Mathematical definition**:
```
bundle(A, B, C)[i] = majority(A[i], B[i], C[i])
```

Where `majority(bits)` returns 1 if more than half the input bits are 1, and 0 otherwise. Ties (exactly half 1s and half 0s) break to 0 for determinism.

**Key properties**:

| Property | Details |
|---|---|
| Similarity preservation | sim(bundle(A, B), A) ~ sim(bundle(A, B), B) > 0.5 |
| Capacity | SNR = sqrt(D/K) for K bundled items; K < 100 for reliable retrieval at D=10,240 |
| NOT associative | bundle(bundle(A, B), C) != bundle(A, bundle(B, C)) |
| Requires accumulator | Incremental bundling requires integer vote counts, not binary operations |

The non-associativity is important: you cannot incrementally bundle binary vectors by XOR or any binary operation. You must maintain integer vote counts and threshold at the end. This is why the design includes `BundleAccumulator` and `DecayingBundleAccumulator` types (see section 8).

**Rust implementation**:

```rust
/// Bundles a slice of vectors using majority vote (tie -> 0).
pub fn bundle(vectors: &[&Self]) -> Self {
    if vectors.is_empty() {
        return Self::zeros();
    }
    let len = vectors.len();
    let mut bits = [0u64; 160];
    for (word_index, slot) in bits.iter_mut().enumerate() {
        let mut word = 0u64;
        for bit_index in 0..64 {
            let mut ones = 0usize;
            for vector in vectors {
                ones += ((vector.bits[word_index] >> bit_index) & 1) as usize;
            }
            if ones * 2 > len {
                word |= 1u64 << bit_index;
            }
        }
        *slot = word;
    }
    Self { bits }
}
```

*Source: `crates/roko-primitives/src/hdc.rs:129-150`*

**Performance**: O(D x K) where K is the number of vectors. For K = 10: approximately 800 ns. For K = 100: approximately 8 us. Memory-bound -- iterates through each input vector's bits.

### 6.3 Permute (Cyclic Shift)

**Purpose**: Encodes position or sequence order by rotating bits, breaking the commutativity of bind. Enables "A then B" to differ from "B then A."

**Mathematical definition**:
```
permute(A, k) = cyclic_left_shift(A, k)
```

**Properties**:

| Property | Details |
|---|---|
| Group operation | permute(permute(A, j), k) = permute(A, j+k) |
| Invertible | permute(permute(A, k), D-k) = A |
| Quasi-orthogonality | permute(A, k) is quasi-orthogonal to A for k >= 1 |
| Similarity preservation | sim(permute(A, k), permute(B, k)) = sim(A, B) |

**Rust implementation**:

```rust
/// Rotates bits left by `n` positions (cyclic permutation for sequence encoding).
pub fn permute(&self, n: usize) -> Self {
    let bits_len = self.bits.len() * 64;
    let n = n % bits_len;
    if n == 0 {
        return *self;
    }
    let word_shift = n / 64;
    let bit_shift = n % 64;
    let mut bits = [0u64; 160];
    for (index, slot) in bits.iter_mut().enumerate() {
        let src0 = (index + 160 - word_shift) % 160;
        *slot = if bit_shift == 0 {
            self.bits[src0]
        } else {
            let src1 = (src0 + 159) % 160;
            (self.bits[src0] << bit_shift) | (self.bits[src1] >> (64 - bit_shift))
        };
    }
    Self { bits }
}
```

*Source: `crates/roko-primitives/src/hdc.rs:154-176`*

**Performance**: 160 shift+OR operations: approximately 10 ns. Slightly slower than bind due to the conditional logic for cross-word bit shifting.

### 6.4 Similarity (Hamming Distance)

**Purpose**: Measures the structural relationship between two vectors on a [0, 1] scale. The fraction of matching bits.

**Mathematical definition**:
```
sim(A, B) = 1 - hamming_distance(A, B) / D
```

**Interpretation**:

| Similarity range | Meaning |
|---|---|
| 1.0 | Identical vectors |
| > 0.526 | Statistically meaningful relationship (< 1% FP rate against 100K comparisons with Bonferroni correction) |
| > 0.52 | Meaningful relationship (single-pair check, p < 3 x 10^-5) |
| 0.485 -- 0.515 | Noise band (quasi-orthogonal, no relationship, 99.7% of random pairs) |
| < 0.48 | Meaningful dissimilarity (anti-correlated) |
| 0.0 | Bitwise complement |

**Rust implementation**:

```rust
/// Returns the Hamming similarity in the range [0, 1].
pub fn similarity(&self, other: &Self) -> f32 {
    let mut differing_bits = 0u32;
    for (left, right) in self.bits.iter().zip(other.bits.iter()) {
        differing_bits += (left ^ right).count_ones();
    }
    let differing_bits = u16::try_from(differing_bits).unwrap_or(u16::MAX);
    1.0_f32 - (f32::from(differing_bits) / 10_240.0_f32)
}
```

*Source: `crates/roko-primitives/src/hdc.rs:223-230`*

**Performance**: 160 XOR + POPCNT operations: approximately 13 ns on x86 with SIMD auto-vectorization. This is the critical inner loop -- every similarity query executes this path. The `u16::try_from` conversion is a safety clamp; at D = 10,240, the maximum Hamming distance is 10,240, which fits in a u16 (max 65,535).

**Zero-copy similarity** (for memory-mapped archives):

```rust
#[cfg(feature = "rkyv")]
pub fn similarity_archived(&self, archived: &ArchivedHdcVector) -> f32 {
    let mut differing_bits = 0u32;
    for (left, right) in self.bits.iter().zip(archived.bits.iter()) {
        let right_u64: u64 = (*right).into();
        differing_bits += (left ^ right_u64).count_ones();
    }
    let differing_bits = u16::try_from(differing_bits).unwrap_or(u16::MAX);
    1.0_f32 - (f32::from(differing_bits) / 10_240.0_f32)
}
```

*Source: `crates/roko-primitives/src/hdc.rs:243-251`*

On little-endian platforms, the archived representation of `[u64; 160]` is identical to the in-memory layout, so this reads directly from the mmap'd buffer with no deserialization overhead.

> *Source: `docs/v1/06-neuro/05-hdc-operations.md`*

---

## 7. Codebooks and Symbol Allocation

A codebook maps symbolic names to deterministic HDC vectors. The `Codebook` type in `roko-primitives` provides this mapping, using domain-scoped seeded generation so that the same `(domain, name)` pair always produces the same vector across runs.

### Codebook Type

```rust
/// A codebook mapping symbolic names to deterministic HDC vectors.
///
/// Symbols are generated from a domain-specific seed, ensuring reproducibility
/// across runs.
#[derive(Debug, Clone)]
pub struct Codebook {
    /// Domain identifier used as the generation seed prefix.
    domain: String,
    /// Allocated symbols: name -> vector.
    symbols: HashMap<String, HdcVector>,
}

impl Codebook {
    /// Create a new empty codebook for the given domain.
    pub fn new(domain: impl Into<String>) -> Self { /* ... */ }

    /// Allocate a new symbol in the codebook.
    ///
    /// The vector is deterministically derived from the domain name and
    /// symbol name, so the same (domain, name) pair always produces the
    /// same vector.
    pub fn allocate(&mut self, name: impl Into<String>) -> &HdcVector {
        let name = name.into();
        self.symbols.entry(name.clone()).or_insert_with(|| {
            let seed = format!("{}:{}", self.domain, name);
            HdcVector::from_seed(seed.as_bytes())
        })
    }

    pub fn get(&self, name: &str) -> Option<&HdcVector> { /* ... */ }
    pub fn get_or_allocate(&mut self, name: &str) -> &HdcVector { /* ... */ }
}
```

*Source: `crates/roko-primitives/src/codebook.rs:28-98`*

### Role-Filler Binding Functions

The codebook module also provides standalone functions for the role-filler binding pattern:

```rust
/// Bind a role vector to a filler vector using XOR.
///
/// This produces a compound vector encoding the relationship "role = filler".
/// The binding is involutory: role_bind(role_bind(role, filler), role) == filler.
pub fn role_bind(role: &HdcVector, filler: &HdcVector) -> HdcVector {
    role.bind(filler)
}

/// Unbind a role from a compound vector, recovering the filler.
///
/// Since XOR is its own inverse, this is identical to role_bind.
pub fn unbind(compound: &HdcVector, role: &HdcVector) -> HdcVector {
    compound.bind(role)
}
```

*Source: `crates/roko-primitives/src/codebook.rs:107-117`*

### CodingCodebook: Domain-Specific Pre-Allocation

The `CodingCodebook` demonstrates how domain-specific codebooks are built:

```rust
/// Pre-allocated coding domain codebook with 15+ symbols.
pub struct CodingCodebook {
    codebook: Codebook,
}

const CODING_SYMBOLS: &[&str] = &[
    "compile_error", "test_failure", "lint_warning", "type_mismatch",
    "missing_import", "unused_variable", "borrow_check", "lifetime_error",
    "trait_bound", "refactor", "new_function", "dependency_add",
    "test_added", "performance", "security", "documentation",
];

impl CodingCodebook {
    pub fn new() -> Self {
        let mut codebook = Codebook::new("coding");
        for symbol in CODING_SYMBOLS {
            codebook.allocate(*symbol);
        }
        Self { codebook }
    }
}
```

*Source: `crates/roko-primitives/src/codebook.rs:125-183`*

Since all vectors are deterministic from the `(domain, name)` seed, two `CodingCodebook::new()` calls on different machines produce identical vectors for all 16 symbols.

### PatternStore: Similarity-Based Retrieval

The `PatternStore` provides a simple similarity-based pattern memory:

```rust
/// A stored pattern with its HDC fingerprint and metadata.
pub struct StoredPattern {
    pub label: String,
    pub fingerprint: HdcVector,
    pub observation_count: u64,
    pub source_domain: String,
}

/// Store for HDC-encoded patterns with similarity-based retrieval.
pub struct PatternStore {
    patterns: Vec<StoredPattern>,
}

impl PatternStore {
    /// Retrieve patterns similar to the probe, above the given threshold.
    pub fn query_similar(&self, probe: &HdcVector, threshold: f32) -> Vec<(&str, f32)> {
        let mut results: Vec<(&str, f32)> = self
            .patterns
            .iter()
            .map(|p| (p.label.as_str(), p.fingerprint.similarity(probe)))
            .filter(|(_, sim)| *sim >= threshold)
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    /// Find the single most similar pattern to the probe.
    pub fn nearest(&self, probe: &HdcVector) -> Option<(&str, f32)> { /* ... */ }
}
```

*Source: `crates/roko-primitives/src/codebook.rs:194-268`*

### Resonance Threshold

The codebook module defines the critical resonance threshold constant:

```rust
/// Similarity threshold for cross-domain resonance detection.
///
/// For 10,240-bit BSC vectors, random similarity is approximately 0.5.
/// A threshold of 0.526 corresponds to roughly 5.3 standard deviations above
/// random chance, giving a per-comparison false positive rate of ~7 x 10^-8
/// and <1% overall FP rate when scanning 100K entries (Bonferroni-corrected).
pub const RESONANCE_THRESHOLD: f32 = 0.526;
```

*Source: `crates/roko-primitives/src/codebook.rs:21`*

*Note: The source code comment says "roughly 3 standard deviations" but the actual Z-score at threshold 0.526 is (0.526 - 0.5) / 0.00494 = 5.26 sigma. The threshold provides much stronger statistical guarantees than 3 sigma; the comment in the source is imprecise.*

### Cross-Domain Resonance Detection

```rust
/// Detect cross-domain resonance between two pattern stores.
///
/// Returns all pairs of patterns from different domains whose similarity
/// exceeds RESONANCE_THRESHOLD (0.526), indicating genuine structural
/// similarity beyond random chance.
pub fn detect_cross_domain_resonance(
    store_a: &PatternStore,
    domain_a: &str,
    store_b: &PatternStore,
    domain_b: &str,
) -> Vec<ResonanceResult> {
    let mut results = Vec::new();
    for a in &store_a.patterns {
        for b in &store_b.patterns {
            let sim = a.fingerprint.similarity(&b.fingerprint);
            if sim >= RESONANCE_THRESHOLD {
                results.push(ResonanceResult {
                    pattern_a: a.label.clone(),
                    domain_a: domain_a.to_string(),
                    pattern_b: b.label.clone(),
                    domain_b: domain_b.to_string(),
                    similarity: sim,
                });
            }
        }
    }
    results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
    results
}
```

*Source: `crates/roko-primitives/src/codebook.rs:293-320`*

---

## 8. Accumulators: Incremental and Decaying Bundling

Because majority-vote bundling is **not associative** (you cannot incrementally bundle binary vectors with a binary operation), roko provides two accumulator types that maintain per-bit vote counts.

### BundleAccumulator

The standard accumulator tracks integer vote counts for each of the 10,240 bit positions. Each added vector contributes +1 for set bits and -1 for unset bits. The `finish()` method thresholds the vote tally at zero to produce a bundled vector.

```rust
/// Incremental majority-vote accumulator for HDC bundling.
#[derive(Debug, Clone)]
pub struct BundleAccumulator {
    votes: Vec<i32>,
    pub count: usize,
}

impl BundleAccumulator {
    pub fn new() -> Self {
        Self { votes: vec![0; HDC_BITS], count: 0 }
    }

    /// Add one vector to the running vote tally.
    pub fn add(&mut self, hv: &HdcVector) { /* +1/-1 per bit */ }

    /// Add one vector with integer weight.
    /// Negative weights subtract the vector's contribution.
    pub fn add_weighted(&mut self, hv: &HdcVector, weight: i32) { /* ... */ }

    /// Apply multiplicative decay to the vote tally.
    pub fn decay(&mut self, factor: f32) {
        assert!(factor >= 0.0, "decay factor must be non-negative");
        for vote in &mut self.votes {
            *vote = (*vote as f32 * factor) as i32;
        }
    }

    /// Collapse the vote tally into a bundled HdcVector.
    pub fn finish(&self) -> HdcVector {
        let mut bits = [0u64; 160];
        for (word_index, slot) in bits.iter_mut().enumerate() {
            let mut word = 0u64;
            for bit_index in 0..64 {
                let position = word_index * 64 + bit_index;
                if self.votes[position] > 0 {
                    word |= 1u64 << bit_index;
                }
            }
            *slot = word;
        }
        HdcVector { bits }
    }
}
```

*Source: `crates/roko-primitives/src/hdc.rs:259-327`*

**Memory**: 10,240 x 4 bytes = 40 KB per accumulator. Heap-allocated via `Vec<i32>`.

**Tie-breaking**: Ties (votes == 0) break to 0, not to random. This ensures determinism.

**Decay**: The `decay()` method enables **controlled forgetting**: multiplying all vote counts by a factor (e.g., 0.95) causes older items to lose influence while newer items retain full weight. Half-life in number of decay calls: `half_life = -ln(2) / ln(factor)`.

| Decay factor | Half-life (decay calls) |
|---|---|
| 0.90 | 6.6 |
| 0.95 | 13.5 |
| 0.99 | 69.0 |

### DecayingBundleAccumulator

A specialized accumulator where each `add()` call automatically decays prior votes before adding the new vector, biasing the finished bundle toward more recent additions.

```rust
/// Bundle accumulator with automatic temporal decay.
#[derive(Debug, Clone)]
pub struct DecayingBundleAccumulator {
    votes: Vec<f32>,
    pub count: usize,
    decay_factor: f32,
}

impl DecayingBundleAccumulator {
    /// Create a new decaying accumulator.
    ///
    /// Panics if decay_factor is outside (0.0, 1.0].
    pub fn new(decay_factor: f32) -> Self {
        assert!(decay_factor > 0.0 && decay_factor <= 1.0);
        Self {
            votes: vec![0.0; HDC_BITS],
            count: 0,
            decay_factor,
        }
    }

    /// Add one vector after decaying prior votes.
    pub fn add(&mut self, hv: &HdcVector) {
        self.count = self.count.saturating_add(1);
        for vote in &mut self.votes {
            *vote *= self.decay_factor;
        }
        update_votes_f32(&mut self.votes, hv, 1.0);
    }

    /// Effective half-life in number of additions.
    pub fn half_life(&self) -> f32 {
        -(2.0_f32.ln()) / self.decay_factor.ln()
    }

    pub fn finish(&self) -> HdcVector { /* same threshold-at-zero logic */ }
}
```

*Source: `crates/roko-primitives/src/hdc.rs:334-398`*

**Memory**: 10,240 x 4 bytes = 40 KB (f32 votes instead of i32).

**Use case**: A `DecayingBundleAccumulator` with factor 0.95 has a half-life of 13.5 additions. After 14 additions, the first vector contributes approximately half its original weight. This is used for temporal context summaries where recent events should dominate.

---

## 9. ItemMemory: Named Concept Lookup

The `ItemMemory` type is a codebook with brute-force nearest-neighbor lookup. It maps named concepts to HDC vectors and supports top-K retrieval by similarity.

```rust
/// Named HDC codebook with brute-force nearest-neighbor lookup.
#[derive(Debug, Clone, Default)]
pub struct ItemMemory {
    entries: HashMap<String, HdcVector>,
}

impl ItemMemory {
    pub fn new() -> Self { Self::default() }

    /// Insert a named concept and its vector.
    pub fn insert(&mut self, name: impl Into<String>, hv: HdcVector) -> Option<HdcVector> {
        self.entries.insert(name.into(), hv)
    }

    /// Insert a deterministic seed-based vector for a name.
    pub fn insert_seeded(&mut self, name: &str) -> Option<HdcVector> {
        self.insert(name, HdcVector::from_seed(name.as_bytes()))
    }

    /// Find the k nearest named concepts to a query.
    pub fn top_k(&self, query: &HdcVector, k: usize) -> Vec<(&str, f32)> {
        let mut scored = self.entries.iter()
            .map(|(name, hv)| (name.as_str(), query.similarity(hv)))
            .collect::<Vec<_>>();
        scored.sort_by(|left, right| right.1.partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.0.cmp(right.0)));
        scored.truncate(k);
        scored
    }

    /// Return the nearest named concept to a query.
    pub fn nearest(&self, query: &HdcVector) -> Option<(&str, f32)> {
        self.top_k(query, 1).into_iter().next()
    }
}
```

*Source: `crates/roko-primitives/src/hdc.rs:402-469`*

`ItemMemory` is the runtime equivalent of a codebook: after encoding some data into an HDC vector, you look it up in an `ItemMemory` to find the nearest named concept. This enables the decomposition workflow: encode -> bind -> bundle -> (later) unbind -> nearest-neighbor in codebook -> recover original concept name.

---

## 10. Application 1: Knowledge Fingerprinting (roko-neuro)

The `KnowledgeHdcEncoder` in `roko-neuro` encodes knowledge entries (Engrams) as HDC vectors. Every knowledge entry in the system gets a 10,240-bit fingerprint at ingestion time, enabling content-addressed similarity search without any external embedding model.

### The Encoder

```rust
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct KnowledgeHdcEncoder;

impl KnowledgeHdcEncoder {
    pub(crate) fn encode_entry(self, entry: &KnowledgeEntry) -> HdcVector {
        if entry.kind == KnowledgeKind::CausalLink {
            self.encode_causal_link(entry)
        } else {
            self.encode_generic_entry(entry)
        }
    }
}
```

*Source: `crates/roko-neuro/src/hdc.rs:8-9, 166-173`*

### Generic Entry Encoding

For non-causal entries (Insights, Heuristics, Warnings, StrategyFragments), the encoding pipeline is:

1. Hash the content into a concept vector via `text_hv()`
2. Bind the knowledge kind to a "kind" role vector
3. Bundle all tags into a tag composite vector
4. Bind the source to a "source" role vector (if present)
5. Bundle all components into the final fingerprint

```rust
fn encode_generic_entry(self, entry: &KnowledgeEntry) -> HdcVector {
    let mut vectors = vec![
        text_hv(&entry.content),
        role_hv("kind").bind(&text_hv(entry.kind.as_str())),
    ];

    if !entry.tags.is_empty() {
        let tags = entry.tags.iter()
            .map(|tag| text_hv(tag))
            .collect::<Vec<_>>();
        vectors.push(bundle(tags));
    }

    if let Some(source) = entry.source.as_deref() {
        let trimmed = source.trim();
        if !trimmed.is_empty() {
            vectors.push(role_hv("source").bind(&text_hv(trimmed)));
        }
    }

    bundle(vectors)
}
```

*Source: `crates/roko-neuro/src/hdc.rs:222-245`*

### Helper Functions

The encoder uses deterministic helper functions to convert text into HDC vectors:

```rust
fn role_hv(role: &str) -> HdcVector {
    HdcVector::from_seed(format!("role:{role}").as_bytes())
}

fn text_hv(text: &str) -> HdcVector {
    HdcVector::from_seed(normalize_text(text).as_bytes())
}

fn normalize_text(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if ch.is_alphanumeric() || ch.is_whitespace() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
```

*Source: `crates/roko-neuro/src/hdc.rs:403-429`*

Text normalization ensures that "Borrow-Checker" and "borrow checker" produce the same vector. Punctuation is replaced with spaces, consecutive whitespace is collapsed, and all characters are lowercased.

### Automatic Fingerprinting at Ingestion

In the knowledge store, every entry gets an HDC vector computed automatically if it does not already have one:

```rust
#[cfg(feature = "hdc")]
fn fingerprint_entry(entry: &KnowledgeEntry) -> HdcVector {
    if let Some(vector) = entry.hdc_vector.as_deref()
        && let Ok(bytes) = <[u8; HDC_VECTOR_BYTES]>::try_from(vector)
    {
        return HdcVector::from_bytes(&bytes);
    }
    KnowledgeHdcEncoder.encode_entry(entry)
}

#[cfg(feature = "hdc")]
fn ensure_hdc_vector(mut entry: KnowledgeEntry) -> KnowledgeEntry {
    let has_valid_vector = entry.hdc_vector.as_ref()
        .is_some_and(|vector| vector.len() == HDC_VECTOR_BYTES);
    if !has_valid_vector {
        entry.hdc_vector = Some(fingerprint_entry(&entry).to_bytes().to_vec());
    }
    entry
}
```

*Source: `crates/roko-neuro/src/knowledge_store.rs:1848-1884`*

---

## 11. Application 2: Role-Filler Structured Encoding (roko-neuro)

The `RoleFillerEncoder` provides a higher-level encoding where each attribute of a knowledge entry is explicitly bound to a named role. This enables structured queries: "find all entries where domain = coding" or "extract the domain from this entry."

### The Encoder

```rust
/// Structured role-filler HDC encoding.
///
/// Encodes structured knowledge using HDC role-filler binding:
/// `role XOR filler` creates a composite vector that preserves structure.
/// Bundling multiple role-filler pairs produces a single vector that can be
/// queried by unbinding any role to recover its filler.
pub(crate) struct RoleFillerEncoder;

impl RoleFillerEncoder {
    /// Encode a set of role-filler string pairs into a single composite HDC vector.
    pub(crate) fn encode_structured(roles_and_fillers: &[(String, String)]) -> HdcVector {
        if roles_and_fillers.is_empty() {
            return HdcVector::zeros();
        }
        let bound: Vec<HdcVector> = roles_and_fillers
            .iter()
            .map(|(role, filler)| role_hv(role).bind(&text_hv(filler)))
            .collect();
        let refs: Vec<&HdcVector> = bound.iter().collect();
        HdcVector::bundle(&refs)
    }

    /// Extract the filler for a given role by unbinding (XOR is its own inverse).
    pub(crate) fn query_role(composite: &HdcVector, role: &str) -> HdcVector {
        composite.bind(&role_hv(role))
    }
}
```

*Source: `crates/roko-neuro/src/hdc.rs:17-40`*

### Structured Knowledge Encoding

The `KnowledgeHdcEncoder` also provides a structured encoding mode where each metadata role can be individually queried via unbinding:

```rust
impl KnowledgeHdcEncoder {
    /// Encode an entry using structured role-filler bindings.
    ///
    /// Unlike encode_generic_entry (which optimizes for content-dominant
    /// similarity), this produces a composite vector where each metadata
    /// role can be individually queried via unbind_role.
    pub(crate) fn encode_structured(entry: &KnowledgeEntry) -> HdcVector {
        let mut vectors = vec![
            role_hv("content").bind(&text_hv(&entry.content)),
            role_hv("kind").bind(&text_hv(entry.kind.as_str())),
            role_hv("tier").bind(&text_hv(&format!("{:?}", entry.tier).to_ascii_lowercase())),
        ];
        let domain = extract_domain(entry);
        vectors.push(role_hv("domain").bind(&text_hv(&domain)));
        // ... source binding if present
        bundle(vectors)
    }

    /// Build a role-filler probe vector: bind(role_vector, filler_vector).
    pub(crate) fn query_by_role(role: &str, filler: &str) -> HdcVector {
        role_hv(role).bind(&text_hv(filler))
    }

    /// Extract the filler component for a given role from a composite vector.
    /// Since XOR is its own inverse, unbind(composite, role) = bind(composite, role).
    pub(crate) fn unbind_role(composite: &HdcVector, role: &str) -> HdcVector {
        composite.bind(&role_hv(role))
    }
}
```

*Source: `crates/roko-neuro/src/hdc.rs:186-220`*

### CausalLink Encoding with Directional Permutation

CausalLinks require special encoding to capture directionality. Without permutation, `bind(hv_cause, hv_effect)` would be identical to `bind(hv_effect, hv_cause)` due to XOR's commutativity. Permutation breaks this symmetry:

```rust
const CAUSE_SHIFT: usize = 1;
const EFFECT_SHIFT: usize = 2;

fn encode_causal_link(self, entry: &KnowledgeEntry) -> HdcVector {
    let Some(parts) = CausalLinkParts::from_entry(entry) else {
        return self.encode_generic_entry(entry);
    };

    let mut vectors = vec![
        text_hv(&entry.content),
        role_hv("kind").bind(&text_hv(entry.kind.as_str())),
        // Asymmetric permutation: cause at shift 1, effect at shift 2
        role_hv("cause").permute(CAUSE_SHIFT).bind(&text_hv(&parts.cause)),
        role_hv("effect").permute(EFFECT_SHIFT).bind(&text_hv(&parts.effect)),
        // Directional edge: separate permutation for cause vs effect
        role_hv("causal_edge").bind(
            &text_hv(&parts.cause).permute(CAUSE_SHIFT)
                .bind(&text_hv(&parts.effect).permute(EFFECT_SHIFT)),
        ),
        role_hv("strength").bind(&strength_hv(parts.strength)),
    ];
    // ... domain, conditions, general tags
    bundle(vectors)
}
```

*Source: `crates/roko-neuro/src/hdc.rs:247-296`*

This ensures that "high complexity -> more review" produces a **different** vector than "more review -> high complexity". The test confirms this:

```rust
#[test]
fn directional_causal_encoding_distinguishes_reversal() {
    let forward = encoder.encode_entry(&entry(
        KnowledgeKind::CausalLink, "high complexity -> more review", &["domain:coding"],
    ));
    let reverse = encoder.encode_entry(&entry(
        KnowledgeKind::CausalLink, "more review -> high complexity", &["domain:coding"],
    ));
    assert!(forward.similarity(&reverse) < 0.7);
}
```

*Source: `crates/roko-neuro/src/hdc.rs:504-518`*

### Causal Content Parsing

The encoder automatically parses causal relationships from natural language content:

```rust
fn parse_causal_content(content: &str) -> Option<(String, String)> {
    // Try arrow separators first
    for separator in ["->", "=>", "\u{2192}"] {
        if let Some((cause, effect)) = split_once_trimmed(content, separator) {
            return Some((cause, effect));
        }
    }
    // Then try natural language separators
    for separator in [
        " causes ", " caused ", " leads to ", " lead to ",
        " results in ", " result in ", " triggers ", " trigger ",
        " drives ", " drive ",
    ] {
        // ... pattern matching
    }
    None
}
```

*Source: `crates/roko-neuro/src/hdc.rs:326-356`*

---

## 12. Application 3: Cross-Domain Resonance Detection (roko-neuro)

The most novel capability of the HDC system is **cross-domain insight resonance** -- automatic detection of structural analogies across different problem domains. When a coding agent learns "complex code needs more review," the structural pattern can be detected as similar to "volatile markets need more caution" because both encode the abstract relationship `BIND(high_uncertainty, more_verification)`.

### How It Works Mathematically

Consider three knowledge entries from three domains:

- **Coding**: "High-complexity modules need more code review"
  - Encoding: `BIND(role_risk_factor, hv_high_complexity) XOR BIND(role_response, hv_more_review)`

- **Finance**: "High-volatility assets need more caution"
  - Encoding: `BIND(role_risk_factor, hv_high_volatility) XOR BIND(role_response, hv_more_caution)`

- **Research**: "Contradictory sources need more verification"
  - Encoding: `BIND(role_risk_factor, hv_contradictory_sources) XOR BIND(role_response, hv_more_verification)`

All three share the same abstract structure: `BIND(role_risk_factor, hv_high_X) XOR BIND(role_response, hv_more_Y)`. The shared role vectors (`role_risk_factor`, `role_response`) contribute similarly to all three vectors, creating measurable above-noise similarity even though the domain-specific fillers are quasi-orthogonal.

Cross-domain similarity typically falls in the range 0.53-0.58, well above the 0.526 threshold for statistical significance against 100K comparisons.

### The ResonanceDetector

```rust
/// A pair of knowledge entries from different domains whose HDC vectors
/// are highly similar, indicating a structural analogy.
pub(crate) struct ResonancePair {
    pub(crate) entry_a: String,
    pub(crate) entry_b: String,
    pub(crate) similarity: f64,
    pub(crate) domain_a: String,
    pub(crate) domain_b: String,
}

/// Detects resonant patterns across knowledge domains.
///
/// Two entries "resonate" when their HDC vectors are highly similar
/// despite coming from different source domains.
pub(crate) struct ResonanceDetector {
    min_similarity: f64,   // Default: 0.526
    max_results: usize,    // Default: 20
}

impl ResonanceDetector {
    /// Detect resonant pairs across knowledge domains.
    ///
    /// Performs pairwise comparison, skipping same-domain pairs and pruning
    /// by the similarity threshold. O(n^2) -- suitable for stores up to ~10K entries.
    pub(crate) fn detect_resonances(&self, entries: &[KnowledgeEntry]) -> Vec<ResonancePair> {
        let encoder = KnowledgeHdcEncoder;
        let encoded: Vec<(HdcVector, String)> = entries.iter()
            .map(|e| {
                let hv = encoder.encode_entry(e);
                let domain = extract_domain(e);
                (hv, domain)
            })
            .collect();

        let mut pairs = Vec::new();
        for i in 0..encoded.len() {
            for j in (i + 1)..encoded.len() {
                if encoded[i].1 == encoded[j].1 { continue; } // skip same-domain
                let sim = f64::from(encoded[i].0.similarity(&encoded[j].0));
                if sim >= self.min_similarity {
                    pairs.push(ResonancePair { /* ... */ });
                }
            }
        }
        pairs.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal));
        pairs.truncate(self.max_results);
        pairs
    }
}
```

*Source: `crates/roko-neuro/src/hdc.rs:44-138`*

The algorithm is O(n^2) pairwise comparison. At ~13 ns per comparison, scanning 10,000 entries (50 million pairs) takes approximately 650 ms. For stores up to approximately 10K entries, this is practical to run on every knowledge ingestion. For larger stores, the three-tier search strategy (Bloom filter pre-filter, reduced-precision scan, exact top-K) described in the docs would be needed.

> *Source: `docs/v1/06-neuro/08-cross-domain-hdc-transfer.md`*

---

## 13. Application 4: Code Fingerprinting (roko-index)

The `roko-index` crate provides a specialized HDC encoding for source code symbols (functions, structs, traits, enums, modules). This enables finding similar code patterns regardless of naming, using structural similarity rather than text matching.

### The HdcFingerprint Type

```rust
/// A 10,240-bit hyperdimensional computing fingerprint.
///
/// Fingerprints encode the kind, name, and contextual content of code
/// artefacts into a fixed-width binary vector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HdcFingerprint {
    bits: [u64; WORDS],  // WORDS = 160
}

impl HdcFingerprint {
    pub fn similarity(&self, other: &Self) -> f64 {
        let dist = hamming_distance(&self.bits, &other.bits);
        1.0 - (f64::from(dist) / TOTAL_BITS as f64)
    }
}
```

*Source: `crates/roko-index/src/hdc.rs:99-118`*

Note that `roko-index` uses its own `HdcFingerprint` type with `[u64; 160]` rather than importing `HdcVector` from `roko-primitives`. The two types are structurally identical but independent -- the index crate re-implements the core HDC functions (splitmix64, fnv1a, bundle, bind, hamming_distance) locally for minimal dependencies. Also note the return type is `f64` (vs `f32` in `HdcVector::similarity`).

### Symbol Fingerprinting: The Encoding Formula

Each symbol's fingerprint combines three properties:

```
fingerprint(symbol) = bind(role_vector(kind), bundle(name_vector, context_vector))
```

**Role vectors** -- one per symbol kind:

```rust
fn role_vector(kind: &SymbolKind) -> [u64; WORDS] {
    let seed: &[u8] = match kind {
        SymbolKind::Function => b"roko:role:function",
        SymbolKind::Struct   => b"roko:role:struct",
        SymbolKind::Enum     => b"roko:role:enum",
        SymbolKind::Trait    => b"roko:role:trait",
        SymbolKind::Const    => b"roko:role:const",
        SymbolKind::Type     => b"roko:role:type",
        SymbolKind::Module   => b"roko:role:module",
        SymbolKind::Impl     => b"roko:role:impl",
        _                    => b"roko:role:unknown",
    };
    vector_from_seed(seed)
}
```

*Source: `crates/roko-index/src/hdc.rs:130-143`*

**Name encoding via character trigrams** -- captures sub-word structure:

```rust
fn encode_name(name: &str) -> [u64; WORDS] {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() < 3 {
        return vector_from_seed(name.as_bytes());
    }
    let trigrams: Vec<[u64; WORDS]> = chars
        .windows(3)
        .map(|w| {
            let trigram: String = w.iter().collect();
            vector_from_seed(trigram.as_bytes())
        })
        .collect();
    bundle(&trigrams)
}
```

*Source: `crates/roko-index/src/hdc.rs:148-163`*

This means `parse_config` and `parse_input` share the trigrams "par", "ars", "rse", "se_" and will have moderate similarity. `parse_config` and `render_output` share no trigrams and will be quasi-orthogonal.

**Complete symbol fingerprint**:

```rust
pub fn fingerprint_symbol(symbol: &Symbol, context: &[u8]) -> HdcFingerprint {
    let role_vec = role_vector(&symbol.kind);
    let name_vec = encode_name(&symbol.name);
    let ctx_vec = vector_from_seed(context);
    let combined = bundle(&[name_vec, ctx_vec]);
    HdcFingerprint { bits: bind(&role_vec, &combined) }
}
```

*Source: `crates/roko-index/src/hdc.rs:173-181`*

### File Fingerprinting

Entire source files get fingerprints by bundling all their symbol fingerprints:

```rust
pub fn fingerprint_file(source: &SourceFile) -> HdcFingerprint {
    if source.symbols.is_empty() {
        return HdcFingerprint { bits: vector_from_seed(source.content.as_bytes()) };
    }
    let sym_fps: Vec<[u64; WORDS]> = source.symbols.iter()
        .map(|sym| fingerprint_symbol(sym, source.content.as_bytes()).bits)
        .collect();
    HdcFingerprint { bits: bundle(&sym_fps) }
}
```

*Source: `crates/roko-index/src/hdc.rs:187-206`*

### Integration into the Workspace Index

The workspace index (`roko-index/src/workspace.rs`) builds fingerprints for every file and symbol at index construction time, then exposes HDC similarity search:

```rust
pub struct HdcQuery {
    /// Fingerprint used as the search anchor.
    pub anchor: crate::hdc::HdcFingerprint,
    /// Minimum similarity score in [0.0, 1.0].
    pub min_similarity: f64,
    /// Maximum number of results to return.
    pub max_results: usize,
}

impl CodeIndex {
    pub fn hdc_search(&self, query: &HdcQuery) -> Vec<SearchResult> {
        // Brute-force scan all symbol fingerprints, filter by min_similarity,
        // sort descending, truncate to max_results
    }
}
```

*Source: `crates/roko-index/src/workspace.rs:137-143, 558`*

The search strategy enum supports HDC as a first-class search mode alongside keyword and structural queries, with a hybrid mode that combines all three using Reciprocal Rank Fusion (RRF):

```rust
pub enum SearchStrategy {
    Keyword(KeywordQuery),
    Structural(StructuralQuery),
    Hdc(HdcQuery),
    Hybrid {
        keyword: Option<KeywordQuery>,
        structural: Option<StructuralQuery>,
        hdc: Option<HdcQuery>,
    },
}
```

*Source: `crates/roko-index/src/workspace.rs:164-181`*

---

## 14. Application 5: Admission Control and AntiKnowledge (roko-neuro)

The knowledge store uses HDC similarity for several admission control and conflict detection functions. These are feature-gated behind `#[cfg(feature = "hdc")]`.

### HDC Similarity Thresholds for AntiKnowledge

When a new knowledge entry is ingested, it is compared against existing AntiKnowledge entries (entries that record "this is wrong" evidence). The HDC similarity score determines the action:

```rust
#[cfg(feature = "hdc")]
const HDC_SIMILARITY_BASELINE: f64 = 0.5;

/// HDC similarity threshold at which an AntiKnowledge match logs a warning.
#[cfg(feature = "hdc")]
const ANTI_KNOWLEDGE_WARN_THRESHOLD: f64 = 0.5;

/// HDC similarity threshold at which a new entry's confidence is discounted.
#[cfg(feature = "hdc")]
const ANTI_KNOWLEDGE_DISCOUNT_THRESHOLD: f64 = 0.7;

/// HDC similarity threshold at which a new entry is rejected entirely.
#[cfg(feature = "hdc")]
const ANTI_KNOWLEDGE_REJECT_THRESHOLD: f64 = 0.9;

/// Confidence multiplier applied when conflicting with AntiKnowledge.
#[cfg(feature = "hdc")]
const ANTI_KNOWLEDGE_DISCOUNT_FACTOR: f64 = 0.5;
```

*Source: `crates/roko-neuro/src/knowledge_store.rs:56-71`*

The tiered response:

| HDC Similarity to AntiKnowledge | Action |
|---|---|
| < 0.5 | No conflict detected |
| 0.5 -- 0.7 | Warning logged |
| 0.7 -- 0.9 | Confidence discounted by 50% |
| >= 0.9 | Entry rejected entirely |

### Conflict Records

When a conflict is detected, a structured record is emitted for observability:

```rust
pub struct AntiKnowledgeConflict {
    pub entry_id: String,
    pub anti_knowledge_id: String,
    pub similarity: f64,
    pub action: String,  // "warned", "discounted", or "rejected"
}
```

*Source: `crates/roko-neuro/src/knowledge_store.rs:94-104`*

---

## 15. Application 6: Context Assembly Scoring (roko-neuro)

When assembling context for an agent prompt (deciding which knowledge to inject), the knowledge store uses a weighted composite score where HDC similarity carries the highest weight:

```rust
/// Context assembly weights for scoring knowledge entries during retrieval.
///
/// Per spec:
/// - HDC similarity: 40%
/// - Pheromone/keyword weight: 30%
/// - Predictive Foraging utility: 20%
/// - Freshness/recency: 10%
pub struct ContextAssemblyWeights {
    pub hdc_similarity: f64,        // Default: 0.40
    pub keyword_relevance: f64,     // Default: 0.30
    pub pf_utility: f64,            // Default: 0.20
    pub freshness: f64,             // Default: 0.10
    pub cross_domain_bonus: f64,    // Default: 0.15
    pub tier_injection: bool,       // Default: true
}

impl ContextAssemblyWeights {
    pub fn composite(
        &self, keyword: f64, hdc: f64, recency: f64, utility: f64, is_cross_domain: bool,
    ) -> f64 {
        let base = self.hdc_similarity * hdc
            + self.keyword_relevance * keyword
            + self.pf_utility * utility
            + self.freshness * recency;
        if is_cross_domain { base * (1.0 + self.cross_domain_bonus) } else { base }
    }
}
```

*Source: `crates/roko-neuro/src/knowledge_store.rs:188-244`*

HDC similarity gets the plurality weight (40%) because it captures structural semantic similarity that keyword matching misses. Cross-domain entries get a 15% bonus to encourage diverse context assembly.

---

## 16. False Positive Analysis and Threshold Selection

The false positive analysis is critical for determining when a similarity score represents a genuine structural relationship versus random noise.

### Statistical Framework

For two independent random 10,240-bit binary vectors:

```
Expected similarity:     mu = 0.5
Standard deviation:      sigma = 0.00494
```

A similarity of `s` corresponds to a Z-score of:
```
Z = (s - 0.5) / 0.00494
```

### Threshold Table

| Threshold | Z-score | Per-Comparison FP Rate | Use Case |
|---|---|---|---|
| 0.505 | 1.01 | 15.6% | Too low -- noise |
| 0.510 | 2.02 | 2.15% | Rough screening |
| 0.515 | 3.04 | 1.2 x 10^-3 | Conservative single-pair |
| 0.520 | 4.05 | 2.6 x 10^-5 | Moderate vocabulary |
| **0.526** | **5.26** | **7.1 x 10^-8** | **100K vocabulary (Bonferroni)** |
| 0.530 | 6.07 | 6.3 x 10^-10 | 1M vocabulary |
| 0.540 | 8.10 | < 10^-15 | Extremely conservative |

### Bonferroni Correction

When scanning N entries, the probability of at least one false positive is approximately N x P(single FP). To maintain an overall false positive rate of alpha:

```
P(single FP) <= alpha / N
```

| Vocabulary Size | Target alpha | Required Z | Threshold |
|---|---|---|---|
| 100 | 1% | 3.72 | 0.518 |
| 1,000 | 1% | 4.26 | 0.521 |
| 10,000 | 1% | 4.75 | 0.523 |
| **100,000** | **1%** | **5.20** | **0.526** |
| 1,000,000 | 1% | 5.61 | 0.528 |

The recommended production threshold of **0.526** guarantees <1% overall false positive rate when scanning 100K entries, with a per-comparison false positive rate of approximately 7.1 x 10^-8.

> *Source: `docs/v1/06-neuro/09-false-positive-math.md`*

---

## 17. Performance Characteristics

### Per-Operation Timing

| Operation | Time | Notes |
|---|---|---|
| `bind()` (XOR) | ~5 ns | 160 XOR on u64 words |
| `permute()` (rotate) | ~10 ns | 160 shift+OR with cross-word logic |
| `similarity()` (Hamming) | ~13 ns | 160 XOR + POPCNT (auto-vectorized) |
| `bundle()` (K=10) | ~800 ns | O(D x K), memory-bound |
| `bundle()` (K=100) | ~8 us | Linear in K |
| `from_seed()` | ~100 ns | FNV-1a + 160 splitmix64 steps |
| `to_bytes()` / `from_bytes()` | ~50 ns | Memcpy 1,280 bytes |

### Scan Performance

| Knowledge Base Size | Brute Force Scan | Time per Query |
|---|---|---|
| 1,000 entries | 1,000 comparisons | ~13 us |
| 10,000 entries | 10,000 comparisons | ~130 us |
| 100,000 entries | 100,000 comparisons | ~1.3 ms |
| 1,000,000 entries | 1,000,000 comparisons | ~13 ms |

### Memory Usage

| Component | Size | Notes |
|---|---|---|
| One HdcVector | 1,280 bytes | Stack-allocated, `Copy` |
| BundleAccumulator | ~40 KB | 10,240 x i32 votes |
| DecayingBundleAccumulator | ~40 KB | 10,240 x f32 votes |
| 1,000 stored vectors | ~1.25 MB | Raw vectors only |
| 100,000 stored vectors | ~125 MB | Raw vectors only |
| 800,000 vectors | ~1 GB | Fits in RAM with overhead |

### Comparison with Float Embeddings

| Metric | HDC (10,240-bit) | Float Embedding (1,536-d float32) |
|---|---|---|
| Storage per vector | 1,280 bytes | 6,144 bytes (4.8x more) |
| Comparison time | ~13 ns (XOR+POPCNT) | ~500 ns (dot product) |
| Composability | Native (XOR bind, majority bundle) | None (must re-embed) |
| Model dependency | None | Requires specific embedding model |
| GPU required | No | Strongly recommended |
| Determinism | Perfect (seeded PRNG) | Depends on model version |
| Semantic fidelity | Lower (bag-of-words level) | Higher (contextual meaning) |

*Note on semantic fidelity*: HDC's `from_seed()` maps exact byte strings to vectors, so "running" and "jogging" produce quasi-orthogonal (unrelated) vectors unless explicitly linked. Neural embeddings capture contextual synonymy that HDC cannot. The two approaches are complementary: HDC excels at compositional structure and speed; neural embeddings excel at semantic similarity. Using both as separate ranking signals with RRF fusion gives the best results.

---

## 18. IronClaw Integration Plan

### Phase 1: Core Crate (ironclaw_hdc)

**Create `crates/ironclaw_hdc/`** with zero external dependencies beyond `serde` and `uuid`.

**File structure**:

```
crates/ironclaw_hdc/
  Cargo.toml
  src/
    lib.rs          # Re-exports, HDC_BITS, HDC_BYTES constants
    vector.rs       # HdcVector, splitmix64, from_seed, bind, bundle, permute, similarity
    codebook.rs     # Codebook, role_bind, unbind
    accumulator.rs  # BundleAccumulator, DecayingBundleAccumulator
    memory.rs       # ItemMemory (brute-force nearest-neighbor)
    encoder.rs      # HdcEncodable trait, text normalization, role-filler encoding
```

**Cargo.toml**:

```toml
[package]
name = "ironclaw_hdc"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }

[dev-dependencies]
# none needed -- all tests are self-contained
```

**Core trait** (in `encoder.rs`):

```rust
use crate::vector::HdcVector;
use crate::codebook::Codebook;

/// Trait for types that can be encoded as HDC vectors.
///
/// Implementors define how their data maps to a 10,240-bit fingerprint.
/// The codebook provides domain-scoped role vectors for structured encoding.
pub trait HdcEncodable {
    /// Encode this value as an HDC vector.
    fn to_hdc(&self, codebook: &Codebook) -> HdcVector;
}

/// Encode raw text with normalization (lowercase, strip punctuation, collapse whitespace).
pub fn text_hv(text: &str) -> HdcVector {
    HdcVector::from_seed(normalize_text(text).as_bytes())
}

/// Encode a role name as a deterministic role vector.
pub fn role_hv(role: &str) -> HdcVector {
    HdcVector::from_seed(format!("role:{role}").as_bytes())
}

fn normalize_text(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if ch.is_alphanumeric() || ch.is_whitespace() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
```

### Phase 2: Memory Integration

**Where**: `crates/ironclaw_memory/` and `src/workspace/search.rs`

**Step 1**: Add `hdc_fingerprint` column to the memory document schema.

In `src/workspace/schema.rs`, add to the document table migration:

```sql
ALTER TABLE documents ADD COLUMN hdc_fingerprint BLOB;
```

**Step 2**: Compute fingerprint at write time.

In the memory write path (wherever `memory_write` constructs a document), compute and store the fingerprint:

```rust
use ironclaw_hdc::{HdcVector, text_hv, role_hv};

fn compute_memory_fingerprint(content: &str, tags: &[String], path: &str) -> Vec<u8> {
    let mut components = vec![text_hv(content)];

    // Encode tags as a bundled set
    if !tags.is_empty() {
        let tag_hvs: Vec<HdcVector> = tags.iter()
            .map(|t| HdcVector::from_seed(t.as_bytes()))
            .collect();
        let refs: Vec<&HdcVector> = tag_hvs.iter().collect();
        components.push(HdcVector::bundle(&refs));
    }

    // Encode path as a role binding
    components.push(role_hv("path").bind(&text_hv(path)));

    let refs: Vec<&HdcVector> = components.iter().collect();
    let fingerprint = HdcVector::bundle(&refs);
    fingerprint.to_bytes().to_vec()
}
```

**Step 3**: Add HDC as a third signal in search fusion.

In `src/workspace/search.rs`, extend `fuse_results` to accept an optional HDC rank list:

```rust
use ironclaw_hdc::HdcVector;

/// Extended fusion that includes HDC similarity as a third ranking signal.
pub fn fuse_results_with_hdc(
    fts_results: Vec<RankedResult>,
    vector_results: Vec<RankedResult>,
    hdc_results: Vec<RankedResult>,  // NEW: HDC similarity rankings
    config: &SearchConfig,
) -> Vec<SearchResult> {
    // Three-way RRF: each result gets score = sum(1/(k + rank)) across all lists
    let k = config.rrf_k as f32;
    let mut scores: HashMap<Uuid, (f32, Option<SearchResult>)> = HashMap::new();

    for (rank, result) in fts_results.iter().enumerate() {
        let entry = scores.entry(result.chunk_id).or_insert((0.0, None));
        entry.0 += 1.0 / (k + rank as f32);
        if entry.1.is_none() { entry.1 = Some(result.into()); }
    }
    for (rank, result) in vector_results.iter().enumerate() {
        let entry = scores.entry(result.chunk_id).or_insert((0.0, None));
        entry.0 += 1.0 / (k + rank as f32);
        if entry.1.is_none() { entry.1 = Some(result.into()); }
    }
    for (rank, result) in hdc_results.iter().enumerate() {
        let entry = scores.entry(result.chunk_id).or_insert((0.0, None));
        entry.0 += 1.0 / (k + rank as f32);
        if entry.1.is_none() { entry.1 = Some(result.into()); }
    }

    // Sort by composite score, return top results
    let mut combined: Vec<_> = scores.into_values()
        .filter_map(|(score, result)| result.map(|r| (score, r)))
        .collect();
    combined.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    combined.into_iter().map(|(_, r)| r).collect()
}
```

### Phase 3: Skill Matching

**Where**: `crates/ironclaw_skills/src/selector.rs`

**Step 1**: Compute an HDC fingerprint for each loaded skill at startup.

```rust
use ironclaw_hdc::{HdcVector, text_hv, role_hv};

/// HDC profile for fast skill similarity matching.
struct SkillHdcProfile {
    fingerprint: HdcVector,
    skill_id: String,
}

fn compute_skill_fingerprint(skill: &LoadedSkill) -> HdcVector {
    let mut components = Vec::new();

    // Encode keywords
    for keyword in &skill.manifest.keywords {
        components.push(HdcVector::from_seed(keyword.as_bytes()));
    }

    // Encode tags with role binding
    let role_tag = role_hv("skill_tag");
    for tag in &skill.manifest.tags {
        components.push(role_tag.bind(&HdcVector::from_seed(tag.as_bytes())));
    }

    // Encode description
    if let Some(desc) = &skill.manifest.description {
        components.push(text_hv(desc));
    }

    if components.is_empty() {
        return HdcVector::from_seed(skill.manifest.name.as_bytes());
    }

    let refs: Vec<&HdcVector> = components.iter().collect();
    HdcVector::bundle(&refs)
}
```

**Step 2**: Use HDC similarity as a pre-filter or scoring boost in `score_skill()`.

```rust
fn score_skill_with_hdc(
    skill: &LoadedSkill,
    profile: &SkillHdcProfile,
    message: &str,
    existing_score: f32,
) -> f32 {
    let query_hv = text_hv(message);
    let hdc_sim = query_hv.similarity(&profile.fingerprint);

    // Blend: 70% existing keyword/pattern score, 30% HDC similarity
    let hdc_boost = if hdc_sim > 0.52 { (hdc_sim - 0.5) * 2.0 } else { 0.0 };
    existing_score * 0.7 + hdc_boost * 0.3
}
```

### Phase 4: Tool Selection

**Where**: `src/tools/registry.rs`

```rust
use ironclaw_hdc::{HdcVector, text_hv, role_hv};

/// Pre-computed HDC index of tool descriptions for fast similarity search.
pub struct ToolHdcIndex {
    entries: Vec<(String, HdcVector)>,  // (tool_name, fingerprint)
}

impl ToolHdcIndex {
    /// Build the index from all registered tools.
    pub fn build(tools: &[Box<dyn Tool>]) -> Self {
        let entries = tools.iter().map(|tool| {
            let mut components = vec![
                HdcVector::from_seed(tool.name().as_bytes()),
                text_hv(tool.description()),
            ];
            // Encode parameter names
            let role_param = role_hv("param");
            for param in tool.parameters() {
                components.push(role_param.bind(
                    &HdcVector::from_seed(param.name.as_bytes())
                ));
            }
            let refs: Vec<&HdcVector> = components.iter().collect();
            (tool.name().to_string(), HdcVector::bundle(&refs))
        }).collect();
        Self { entries }
    }

    /// Suggest tools that match an intent description.
    pub fn suggest(&self, intent: &str, top_k: usize) -> Vec<(&str, f32)> {
        let query = text_hv(intent);
        let mut scored: Vec<(&str, f32)> = self.entries.iter()
            .map(|(name, fp)| (name.as_str(), query.similarity(fp)))
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }
}
```

### Phase 5: Novelty Detection for Memory

**Where**: `src/workspace/` -- `memory_write` path

Before writing a new memory, compare its HDC fingerprint against existing entries. If too similar, suggest merging instead of creating a new entry.

```rust
/// Check whether a new memory is too similar to an existing one.
///
/// Returns the index and similarity of the most similar existing entry,
/// if it exceeds the deduplication threshold.
fn find_near_duplicate(
    new_content: &str,
    existing_fingerprints: &[(Uuid, Vec<u8>)],
) -> Option<(Uuid, f32)> {
    let new_hv = text_hv(new_content);
    let threshold = 0.7;  // High similarity = likely duplicate

    existing_fingerprints.iter()
        .filter_map(|(id, fp_bytes)| {
            let bytes: &[u8; 1280] = fp_bytes.as_slice().try_into().ok()?;
            let existing_hv = HdcVector::from_bytes(bytes);
            let sim = new_hv.similarity(&existing_hv);
            if sim > threshold { Some((*id, sim)) } else { None }
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
}
```

### Phase 6: Compositional Queries (Advanced)

HDC enables queries that are impossible with traditional search:

```rust
// "Find memories about Rust AND async" -- bind creates an intersection-like query
let query = HdcVector::from_seed(b"rust").bind(&HdcVector::from_seed(b"async"));

// "Find memories similar to this one BUT NOT about testing" -- subtract
let mut acc = BundleAccumulator::new();
acc.add_weighted(&source_memory_hv, 3);      // boost the source
acc.add_weighted(&HdcVector::from_seed(b"testing"), -2);  // suppress testing
let anti_test_query = acc.finish();

// "What topic connects these three memories?" -- unbind shared structure
let shared = HdcVector::bundle(&[&mem1_hv, &mem2_hv, &mem3_hv]);
let topic_signal = shared.bind(&role_hv("topic"));
// Nearest-neighbor lookup against topic codebook reveals the common topic
```

### Migration Strategy

The integration is **purely additive** and can be rolled out incrementally:

1. **Week 1**: Create `ironclaw_hdc` crate with `HdcVector`, `Codebook`, `BundleAccumulator`, `ItemMemory`. Port directly from `roko-primitives/src/hdc.rs` and `roko-primitives/src/codebook.rs`. Write unit tests.

2. **Week 2**: Add `hdc_fingerprint` column to memory schema. Compute fingerprints on `memory_write`. No search integration yet -- just persisting the data.

3. **Week 3**: Add HDC similarity as a third signal in `fuse_results` via RRF. Feature-gate behind a config flag. Run A/B comparison of search quality.

4. **Week 4**: Add skill fingerprinting and tool index. Benchmark against current keyword/regex scoring.

5. **Ongoing**: Add novelty detection, compositional queries, and cross-domain resonance as the foundation proves itself.

**Dependencies**: Zero external dependencies. Pure Rust, SIMD-optional (auto-vectorization handles the common case). The only dependencies would be `serde` (for serialization) and `uuid` (for random vector generation).

---

## 19. Complexity Assessment

| Dimension | Assessment |
|---|---|
| **Core implementation** | ~500-800 lines of Rust (HdcVector, Codebook, Accumulator, ItemMemory) |
| **Integration touchpoints** | 3-4 existing modules (workspace/search.rs, skills/selector.rs, tools/registry.rs, possibly ironclaw_embeddings) |
| **Risk** | Low -- purely additive, does not change existing behavior |
| **Dependencies** | None beyond serde and uuid (pure Rust, SIMD-optional) |
| **Memory overhead** | 1,280 bytes per fingerprint + 40 KB per active accumulator |
| **CPU overhead** | ~13 ns per comparison, ~1 ms to scan 100K entries |
| **Testing** | Well-tested in roko (100+ test cases across three crates); tests are self-contained and portable |
| **Migration** | Incremental -- add fingerprints alongside existing data, use as optional ranking signal |

---

## 20. Academic References

### Foundational Theory

- **[Kanerva88]** Kanerva, P. (1988). *Sparse Distributed Memory*. Cambridge, MA: MIT Press. ISBN 9780262111324. The original work on content-addressable memory in high-dimensional binary spaces. Introduced the mathematical framework for storing and retrieving patterns in spaces of thousands of dimensions, showing that random high-dimensional addresses are almost always far apart -- the foundational observation that HDC exploits.

- **[Kanerva96]** Kanerva, P. (1996). "Binary Spatter-Coding of Ordered K-Tuples." In C. von der Malsburg et al. (eds.), *Artificial Neural Networks -- ICANN 96* (Lecture Notes in Computer Science, vol. 1112), pp. 869-873. Berlin: Springer. Formalized Binary Spatter Codes (BSC): the specific HDC variant used by roko and proposed for IronClaw. Defined XOR as the binding operation and majority vote as the bundling operation for binary vectors. Showed that ordered sequences can be encoded using cyclic permutation combined with XOR binding.

- **[Kanerva09]** Kanerva, P. (2009). "Hyperdimensional Computing: An Introduction to Computing in Distributed Representation with High-Dimensional Random Vectors." *Cognitive Computation*, 1(2), 139-159. DOI: 10.1007/s12559-009-9009-8. The primary reference for the 10,240-bit BSC dimensionality choice and the systematic treatment of the four core algebraic operations. Demonstrated that high-dimensional random vectors form a robust algebra suitable for general-purpose computing, not just memory storage.

- **[Gayler03]** Gayler, R.W. (2003). "Vector Symbolic Architectures Answer Jackendoff's Challenges for Cognitive Neuroscience." In *Proceedings of the Joint International Conference on Cognitive Science (ICCS/ASCS'03)*, pp. 133-138. Introduced the term "Vector Symbolic Architectures" (VSA) to unify the family of models including BSC, MAP, HRR, and FHRR. Argued that VSAs provide a computationally adequate framework for implementing symbolic cognitive architectures using distributed representations.

### Holographic Representations

- **[Plate95]** Plate, T.A. (1995). "Holographic Reduced Representations." *IEEE Transactions on Neural Networks*, 6(3), 623-641. DOI: 10.1109/72.377968. Introduced Holographic Reduced Representations (HRR), using circular convolution as the binding operation on real-valued vectors. Proved capacity bounds for bundling and showed that compositional structure can be recovered via circular correlation. HRR is the theoretical ancestor of BSC; BSC can be viewed as a binarized approximation that trades representational precision for computational efficiency.

- **[Plate03]** Plate, T.A. (2003). *Holographic Reduced Representations: Distributed Representation for Cognitive Structures*. CSLI Publications. ISBN 9781575864303. The comprehensive monograph on HRR theory. Contains the bundle capacity proofs that roko's SNR analysis builds on, and the formal treatment of role-filler binding and unbinding that the `RoleFillerEncoder` implements.

### Surveys and Capacity Bounds

- **[Kleyko22]** Kleyko, D., Rachkovskij, D.A., Osipov, E., & Rahimi, A. (2022). "A Survey on Hyperdimensional Computing aka Vector Symbolic Architectures, Part I: Models and Data Transformations." *ACM Computing Surveys*, 55(6), Article 130, 40 pages. DOI: 10.1145/3538531. The most comprehensive HDC/VSA survey. Validates the bundle similarity formula and capacity bounds used throughout this document. Provides a systematic comparison of all major VSA variants (BSC, MAP, HRR, FHRR) including their algebraic properties, capacity, and computational requirements. The companion Part II (DOI: 10.1145/3558000) covers applications and cognitive models.

- **[Thomas21]** Thomas, A., Dasgupta, S., & Rosing, T. (2021). "A Theoretical Perspective on Hyperdimensional Computing." *Journal of Artificial Intelligence Research (JAIR)*, 72, 215-249. DOI: 10.1613/jair.1.12664. Provides a unified theoretical treatment of HDC. Establishes capacity scaling laws showing that bundle capacity scales as K = O(D / log(D)), which is tighter than the simple SNR bound. Also provides the theoretical foundation for trigram encoding used in `roko-index`'s `encode_name()` function.

### Random Projection Theory

- **[JL84]** Johnson, W.B. & Lindenstrauss, J. (1984). "Extensions of Lipschitz Mappings into a Hilbert Space." *Contemporary Mathematics*, 26, 189-206. The JL lemma: N points can be projected into O(log N / epsilon^2) dimensions while preserving pairwise distances within (1 +/- epsilon). Provides theoretical justification for the sufficiency of high-dimensional random projections, though the specific capacity of BSC vectors is better characterized by the SNR model and false positive analysis.

### Resonator Networks

- **[Frady18]** Frady, E.P., Kleyko, D., & Sommer, F.T. (2018). "A Theory of Sequence Indexing and Working Memory in Recurrent Neural Networks." *Neural Computation*, 30(6), 1449-1513. Proposed sequence indexing using HDC operations within recurrent neural networks. Showed that VSA coding principles (permutation for position, XOR for binding) can be implemented in biologically plausible recurrent networks, providing the theoretical basis for the permutation-based causal link encoding in `roko-neuro`.

- **[Frady20]** Frady, E.P., Kent, S.J., Olshausen, B.A., & Sommer, F.T. (2020). "Resonator Networks, 1: An Efficient Solution for Factoring High-Dimensional, Distributed Representations of Data Structures." *Neural Computation*, 32(12), 2311-2331. DOI: 10.1162/neco_a_01331. Introduced resonator networks as an efficient algorithm for decomposing composite HDC vectors into their constituent factors -- solving the "inverse bundling" problem that brute-force unbinding cannot efficiently address. Relevant for future IronClaw features like automatic knowledge decomposition.

### Locality-Sensitive Hashing

- **[Charikar02]** Charikar, M.S. (2002). "Similarity Estimation Techniques from Rounding Algorithms." In *Proceedings of the 34th Annual ACM Symposium on Theory of Computing (STOC)*, pp. 380-388. DOI: 10.1145/509907.509965. SimHash: showed that single random projections produce binary codes where collision probability equals cosine similarity. HDC vectors can be viewed as SimHash signatures with the additional structure provided by the four algebraic operations. The Hamming distance between HDC vectors preserves approximate cosine similarity relationships.

### Statistical Correction

- **[Bonferroni36]** Bonferroni, C.E. (1936). "Teoria statistica delle classi e calcolo delle probabilita." *Pubblicazioni del R. Istituto Superiore di Scienze Economiche e Commerciali di Firenze*, 8, 3-62. The multiple-comparison correction used for threshold selection: when testing N hypotheses, divide the significance level by N to control the family-wise error rate. Applied in Section 16 to derive the 0.526 threshold for 100K-entry vocabularies.

### Hardware and Systems

- **[Neubert19]** Neubert, P., Schubert, S., & Protzel, P. (2019). "An Introduction to Hyperdimensional Computing for Robotics." *KI -- Kunstliche Intelligenz*, 33(4), 319-330. DOI: 10.1007/s13218-019-00623-z. Practical encoding schemes for robotics applications with the place cell analogy. Demonstrates that HDC provides a lightweight alternative to deep learning for robotic perception tasks where training data is scarce.

- **[Imani19]** Imani, M. et al. (2019). "FloatHD: Integer-Based Training Framework for Hyperdimensional Computing." *IEEE/ACM ICCAD*. FPGA implementations achieving ~3-5 ns per comparison at 200 MHz. Demonstrates that HDC's bitwise operations map efficiently to hardware accelerators, achieving throughputs that scale linearly with parallelism.

> *Full reference list: `docs/v1/21-references/09-hdc-vsa.md`*

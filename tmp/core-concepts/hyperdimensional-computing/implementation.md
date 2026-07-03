[← Back to HDC Overview](./README.md)

# Implementation — Full Source Code

This document contains the complete roko source code for all HDC types and operations, with GitHub links to the canonical source. All code is derived from `roko-primitives` and `roko-neuro`.

---

## 1. Core Data Structure: HdcVector

The foundation of the entire HDC system is the `HdcVector` type, defined in [`crates/roko-primitives/src/hdc.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/hdc.rs). This is a 10,240-bit binary vector stored as a fixed-size array of 160 u64 words.

### 1.1 Type Definition

```rust
/// Number of bits in one HdcVector.
pub const HDC_BITS: usize = 10_240;
/// Number of serialized bytes in one HdcVector.
pub const HDC_BYTES: usize = 1_280;

/// 10,240-bit binary sparse distributed vector.
///
/// Three core operations: XOR bind, majority-vote bundle, Hamming similarity.
/// All operations are CPU-cache-friendly bit manipulation — no floating point,
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

**Key design choices**:

- **`Copy` trait**: The entire 1,280-byte vector implements `Copy`. Stack-allocated and passed by value, avoiding heap allocation and reference counting overhead.
- **`[u64; 160]` not `Vec<u64>`**: Fixed-size array enables `const` constructors, stack allocation, and compile-time size guarantees.
- **`rkyv` support**: With the `rkyv` feature flag, vectors can be zero-copy deserialized from memory-mapped files. The archived representation of `[u64; 160]` on little-endian platforms is identical to the in-memory layout.
- **Custom serde**: The `Serialize`/`Deserialize` implementations encode the vector as a 1,280-byte blob, not as 160 individual u64 values.

### 1.2 Construction Methods

**Random vector** (used for initial codebook symbols):

```rust
/// Returns a pseudo-random vector seeded from a random UUID.
pub fn random() -> Self {
    let seed = Uuid::new_v4().as_u128();
    // Mix into splitmix64 state, fill 160 words
    let mut state = seed as u64 ^ ((seed >> 64) as u64);
    let mut bits = [0u64; 160];
    for word in &mut bits {
        *word = splitmix64(&mut state);
    }
    Self { bits }
}
```

**Deterministic seeded vector** (the workhorse — maps any byte sequence to a reproducible vector):

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

/// splitmix64 PRNG: fast, high-quality, period 2^64.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}
```

*Source: [`crates/roko-primitives/src/hdc.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/hdc.rs)*

The PRNG chain is `FNV-1a hash -> splitmix64 expansion`. The key property: `from_seed(b"rust")` **always produces the same vector**, across runs, across machines, across architectures.

**Zero vector** (identity for bundling, base case):

```rust
pub const fn zeros() -> Self {
    Self { bits: [0; 160] }
}
```

### 1.3 Serialization

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

*Source: [`crates/roko-primitives/src/hdc.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/hdc.rs)*

### 1.4 Convenience Fingerprinting Functions

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

*Source: [`crates/roko-primitives/src/hdc.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/hdc.rs)*

---

## 2. The Four Operations — Rust Implementation

### 2.1 Bind (XOR)

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

*Source: [`crates/roko-primitives/src/hdc.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/hdc.rs)*

### 2.2 Bundle (Majority Vote)

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

*Source: [`crates/roko-primitives/src/hdc.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/hdc.rs)*

### 2.3 Permute (Cyclic Shift)

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

*Source: [`crates/roko-primitives/src/hdc.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/hdc.rs)*

### 2.4 Similarity (Hamming Distance)

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

*Source: [`crates/roko-primitives/src/hdc.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/hdc.rs)*

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

On little-endian platforms, the archived representation of `[u64; 160]` is identical to the in-memory layout, so this reads directly from the mmap'd buffer with no deserialization overhead.

---

## 3. Codebooks and Symbol Allocation

A codebook maps symbolic names to deterministic HDC vectors. Source: [`crates/roko-primitives/src/codebook.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/codebook.rs).

### 3.1 Codebook Type

```rust
/// A codebook mapping symbolic names to deterministic HDC vectors.
#[derive(Debug, Clone)]
pub struct Codebook {
    /// Domain identifier used as the generation seed prefix.
    domain: String,
    /// Allocated symbols: name -> vector.
    symbols: HashMap<String, HdcVector>,
}

impl Codebook {
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            symbols: HashMap::new(),
        }
    }

    /// Allocate a new symbol in the codebook.
    ///
    /// The vector is deterministically derived from the domain name and
    /// symbol name, so the same (domain, name) pair always produces the same vector.
    pub fn allocate(&mut self, name: impl Into<String>) -> &HdcVector {
        let name = name.into();
        self.symbols.entry(name.clone()).or_insert_with(|| {
            let seed = format!("{}:{}", self.domain, name);
            HdcVector::from_seed(seed.as_bytes())
        })
    }

    pub fn get(&self, name: &str) -> Option<&HdcVector> {
        self.symbols.get(name)
    }

    pub fn get_or_allocate(&mut self, name: &str) -> &HdcVector {
        self.allocate(name)
    }

    pub fn len(&self) -> usize { self.symbols.len() }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &HdcVector)> {
        self.symbols.iter().map(|(k, v)| (k.as_str(), v))
    }
}
```

### 3.2 Role-Filler Binding Functions

```rust
/// Bind a role vector to a filler vector using XOR.
pub fn role_bind(role: &HdcVector, filler: &HdcVector) -> HdcVector {
    role.bind(filler)
}

/// Unbind a role from a compound vector, recovering the filler.
/// Since XOR is its own inverse, this is identical to role_bind.
pub fn unbind(compound: &HdcVector, role: &HdcVector) -> HdcVector {
    compound.bind(role)
}
```

### 3.3 CodingCodebook: Domain-Specific Pre-Allocation

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

    pub fn get(&self, symbol: &str) -> Option<&HdcVector> {
        self.codebook.get(symbol)
    }
}
```

Two `CodingCodebook::new()` calls on different machines produce identical vectors for all 16 symbols.

### 3.4 PatternStore: Similarity-Based Retrieval

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
    pub fn new() -> Self {
        Self { patterns: Vec::new() }
    }

    pub fn add(&mut self, label: impl Into<String>, fingerprint: HdcVector, domain: impl Into<String>) {
        self.patterns.push(StoredPattern {
            label: label.into(),
            fingerprint,
            observation_count: 1,
            source_domain: domain.into(),
        });
    }

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
    pub fn nearest(&self, probe: &HdcVector) -> Option<(&str, f32)> {
        self.patterns
            .iter()
            .map(|p| (p.label.as_str(), p.fingerprint.similarity(probe)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn len(&self) -> usize {
        self.patterns.len()
    }
}
```

*Source: [`crates/roko-primitives/src/codebook.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/codebook.rs)*

### 3.5 Cross-Domain Resonance Detection

```rust
/// Similarity threshold for cross-domain resonance detection.
///
/// For 10,240-bit BSC vectors, random similarity is approximately 0.5.
/// A threshold of 0.526 corresponds to roughly 5.3 standard deviations above
/// random chance, giving a per-comparison false positive rate of ~7×10^-8
/// and <1% overall FP rate when scanning 100K entries (Bonferroni-corrected).
pub const RESONANCE_THRESHOLD: f32 = 0.526;

/// Result of a cross-domain resonance detection.
pub struct ResonanceResult {
    pub pattern_a: String,
    pub domain_a: String,
    pub pattern_b: String,
    pub domain_b: String,
    pub similarity: f32,
}

/// Detect cross-domain resonance between two pattern stores.
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

*Source: [`crates/roko-primitives/src/codebook.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/codebook.rs)*

---

## 4. Accumulators: Incremental and Decaying Bundling

Because majority-vote bundling is **not associative**, the system provides two accumulator types that maintain per-bit vote counts. Source: [`crates/roko-primitives/src/hdc.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/hdc.rs).

### 4.1 BundleAccumulator

The standard accumulator tracks integer vote counts for each of the 10,240 bit positions. Each added vector contributes +1 for set bits and -1 for unset bits.

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
    pub fn add(&mut self, hv: &HdcVector) {
        self.count = self.count.saturating_add(1);
        for (word_index, &word) in hv.bits().iter().enumerate() {
            for bit_index in 0..64 {
                let position = word_index * 64 + bit_index;
                if (word >> bit_index) & 1 == 1 {
                    self.votes[position] += 1;
                } else {
                    self.votes[position] -= 1;
                }
            }
        }
    }

    /// Add one vector with integer weight. Negative weights subtract contribution.
    pub fn add_weighted(&mut self, hv: &HdcVector, weight: i32) {
        self.count = self.count.saturating_add(1);
        for (word_index, &word) in hv.bits().iter().enumerate() {
            for bit_index in 0..64 {
                let position = word_index * 64 + bit_index;
                if (word >> bit_index) & 1 == 1 {
                    self.votes[position] += weight;
                } else {
                    self.votes[position] -= weight;
                }
            }
        }
    }

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
        HdcVector::from_bits(bits)
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}
```

**Memory**: 10,240 × 4 bytes = 40 KB per accumulator.

**Tie-breaking**: Ties (votes == 0) break to 0. This ensures determinism.

**Decay half-lives**:

| Decay factor | Half-life (decay calls) |
|---|---|
| 0.90 | 6.6 |
| 0.95 | 13.5 |
| 0.99 | 69.0 |

### 4.2 DecayingBundleAccumulator

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
        // Decay all prior votes
        for vote in &mut self.votes {
            *vote *= self.decay_factor;
        }
        // Add new vector with weight 1.0
        for (word_index, &word) in hv.bits().iter().enumerate() {
            for bit_index in 0..64 {
                let position = word_index * 64 + bit_index;
                if (word >> bit_index) & 1 == 1 {
                    self.votes[position] += 1.0;
                } else {
                    self.votes[position] -= 1.0;
                }
            }
        }
    }

    /// Effective half-life in number of additions.
    pub fn half_life(&self) -> f32 {
        -(2.0_f32.ln()) / self.decay_factor.ln()
    }

    /// Collapse the vote tally into a bundled HdcVector.
    pub fn finish(&self) -> HdcVector {
        let mut bits = [0u64; 160];
        for (word_index, slot) in bits.iter_mut().enumerate() {
            let mut word = 0u64;
            for bit_index in 0..64 {
                let position = word_index * 64 + bit_index;
                if self.votes[position] > 0.0 {
                    word |= 1u64 << bit_index;
                }
            }
            *slot = word;
        }
        HdcVector::from_bits(bits)
    }
}
```

*Source: [`crates/roko-primitives/src/hdc.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/hdc.rs)*

**Memory**: 10,240 × 4 bytes = 40 KB (f32 votes instead of i32).

**Use case**: A `DecayingBundleAccumulator` with factor 0.95 has a half-life of 13.5 additions. After 14 additions, the first vector contributes approximately half its original weight. Used for temporal context summaries where recent events should dominate.

---

## 5. ItemMemory: Named Concept Lookup

`ItemMemory` is a codebook with brute-force nearest-neighbor lookup. Source: [`crates/roko-primitives/src/hdc.rs`](https://github.com/wpank/roko/blob/main/crates/roko-primitives/src/hdc.rs).

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

    /// Return the vector for a named concept.
    pub fn get(&self, name: &str) -> Option<&HdcVector> {
        self.entries.get(name)
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

    /// Return all concepts above a similarity threshold.
    pub fn query_above(&self, query: &HdcVector, threshold: f32) -> Vec<(&str, f32)> {
        let mut results: Vec<(&str, f32)> = self.entries.iter()
            .filter_map(|(name, hv)| {
                let sim = query.similarity(hv);
                if sim >= threshold { Some((name.as_str(), sim)) } else { None }
            })
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}
```

`ItemMemory` is the runtime equivalent of a codebook: after encoding some data into an HDC vector, look it up in an `ItemMemory` to find the nearest named concept. This enables the decomposition workflow: encode → bind → bundle → (later) unbind → nearest-neighbor in codebook → recover original concept name.

---

## 6. Code Fingerprinting Types (roko-index)

The `roko-index` crate defines its own `HdcFingerprint` type — structurally identical to `HdcVector` but independent for minimal dependencies. Source: [`crates/roko-index/src/hdc.rs`](https://github.com/wpank/roko/blob/main/crates/roko-index/src/hdc.rs).

```rust
const WORDS: usize = 160;
const TOTAL_BITS: usize = WORDS * 64; // 10,240

/// A 10,240-bit hyperdimensional computing fingerprint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HdcFingerprint {
    bits: [u64; WORDS],
}

impl HdcFingerprint {
    pub fn similarity(&self, other: &Self) -> f64 {
        let dist = hamming_distance(&self.bits, &other.bits);
        1.0 - (f64::from(dist) / TOTAL_BITS as f64)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.bits.iter()
            .flat_map(|w| w.to_le_bytes())
            .collect()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != WORDS * 8 { return None; }
        let mut bits = [0u64; WORDS];
        for (i, chunk) in bytes.chunks(8).enumerate() {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(chunk);
            bits[i] = u64::from_le_bytes(buf);
        }
        Some(Self { bits })
    }
}

fn hamming_distance(a: &[u64; WORDS], b: &[u64; WORDS]) -> u32 {
    a.iter().zip(b.iter()).map(|(x, y)| (x ^ y).count_ones()).sum()
}
```

Note: the return type is `f64` (vs `f32` in `HdcVector::similarity`).

---

Next: [Applications — 6 Use Cases with Worked Examples](./applications.md)

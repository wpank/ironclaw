[← Back to HDC Overview](./README.md)

# Implementation — Captured Source And Adaptation Notes

This document captures the HDC implementation shape from the local source corpus. Treat the code blocks as implementation references and adaptation sketches unless a block is explicitly marked as IronClaw-ready. Omitted blocks intentionally avoid duplicating inaccessible source; rebuild them in the owning IronClaw modules with caller-level tests.

---

## 1. Core Data Structure: HdcVector

The foundation of the HDC system is the `HdcVector` shape: a 10,240-bit binary vector stored as a fixed-size array of 160 u64 words.

### 1.1 Type Definition

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

**Key design choices**:

- **`Copy` trait**: The entire 1,280-byte vector implements `Copy`. Stack-allocated and passed by value, avoiding heap allocation and reference counting overhead.
- **`[u64; 160]` not `Vec<u64>`**: Fixed-size array enables `const` constructors, stack allocation, and a fixed compile-time size.
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

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

*Corpus label: `crates/roko-primitives/src/hdc.rs`*

The PRNG chain is `FNV-1a hash -> splitmix64 expansion`. The key property: `from_seed(b"rust")` **always produces the same vector**, across runs, across machines, across architectures.

**Zero vector** (identity for bundling, base case):

```rust
pub const fn zeros() -> Self {
    Self { bits: [0; 160] }
}
```

### 1.3 Serialization

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

*Corpus label: `crates/roko-primitives/src/hdc.rs`*

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

*Corpus label: `crates/roko-primitives/src/hdc.rs`*

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

*Corpus label: `crates/roko-primitives/src/hdc.rs`*

### 2.2 Bundle (Majority Vote)

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

*Corpus label: `crates/roko-primitives/src/hdc.rs`*

### 2.3 Permute (Cyclic Shift)

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

*Corpus label: `crates/roko-primitives/src/hdc.rs`*

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

*Corpus label: `crates/roko-primitives/src/hdc.rs`*

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

A codebook maps symbolic names to deterministic HDC vectors.

### 3.1 Codebook Type

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

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

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

Two `CodingCodebook::new()` calls on different machines produce identical vectors for all 16 symbols.

### 3.4 PatternStore: Similarity-Based Retrieval

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

*Corpus label: `crates/roko-primitives/src/codebook.rs`*

### 3.5 Cross-Domain Resonance Detection

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

*Corpus label: `crates/roko-primitives/src/codebook.rs`*

---

## 4. Accumulators: Incremental and Decaying Bundling

Because majority-vote bundling is **not associative**, use accumulator types that maintain per-bit vote counts.

### 4.1 BundleAccumulator

The standard accumulator tracks integer vote counts for each of the 10,240 bit positions. Each added vector contributes +1 for set bits and -1 for unset bits.

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

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

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

*Corpus label: `crates/roko-primitives/src/hdc.rs`*

**Memory**: 10,240 × 4 bytes = 40 KB (f32 votes instead of i32).

**Use case**: A `DecayingBundleAccumulator` with factor 0.95 has a half-life of 13.5 additions. After 14 additions, the first vector contributes approximately half its original weight. Used for temporal context summaries where recent events should dominate.

---

## 5. ItemMemory: Named Concept Lookup

`ItemMemory` is a codebook with brute-force nearest-neighbor lookup.

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

`ItemMemory` is the runtime equivalent of a codebook: after encoding some data into an HDC vector, look it up in an `ItemMemory` to find the nearest named concept. This enables the decomposition workflow: encode → bind → bundle → (later) unbind → nearest-neighbor in codebook → recover original concept name.

---

## 6. Code Fingerprinting Types

The captured code-indexing material defines its own `HdcFingerprint` type — structurally identical to `HdcVector` but independent for minimal dependencies.

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

Note: the return type is `f64` (vs `f32` in `HdcVector::similarity`).

---

Next: [Applications — 6 Use Cases with Worked Examples](./applications.md)

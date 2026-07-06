# IronClaw HDC Crate

## Purpose

Hyperdimensional Computing primitives for workspace memory fingerprinting, deduplication, and similarity search. Provides a self-contained, deterministic binary hypervector library with no dependency on the main IronClaw crate.

## Invariants

- `HdcVector` is a fixed 10,240-bit (1,280 byte) binary vector stored as `[u64; 160]`
- All seeded operations are deterministic across platforms (SplitMix64 + FNV-1a)
- `vector.rs` is allocation-free (`Copy` type, no heap usage)
- The crate has no dependency on the main IronClaw crate (standalone)
- Golden test ensures seed stability -- any algorithm change breaks CI
- Ties in bundle majority-vote resolve to 0 (conservative)
- Little-endian byte order for serialization

## Public API Overview

### Core Types (`vector.rs`)

- `HdcVector` -- 10,240-bit binary hypervector (`[u64; 160]`, `Copy`)
  - `from_seed(domain, key)` -- deterministic construction via FNV-1a + SplitMix64
  - `bind(other)` -- XOR binding (associative, commutative, self-inverse)
  - `permute(shifts)` -- cyclic bit permutation across full width
  - `hamming_distance(other)` -- popcount of XOR
  - `similarity(other)` -- normalized Hamming similarity in `[0.0, 1.0]`
  - `to_bytes()` / `from_bytes()` -- little-endian serialization

### Bundling (`bundle.rs`)

- `BundleAccumulator` -- majority-vote bundling via per-bit i32 vote counts
  - `add(vector)` -- increment/decrement votes per bit
  - `finalize()` -- produce consensus vector (positive votes = 1, else 0)
- `DecayingBundleAccumulator` -- exponential-decay bundling for recency bias
  - `new(decay_factor)` -- decay in (0.0, 1.0) exclusive
  - `add(vector)` -- decay existing votes, then add new
  - `finalize()` -- produce recency-weighted consensus

### Symbol Mapping (`codebook.rs`)

- `Codebook` -- deterministic symbol-to-vector mapping (HashMap-backed)
  - `get_or_create(symbol)` -- lazy deterministic assignment via `from_seed`
  - `with_defaults(symbols)` -- pre-populate known symbols
- `ItemMemory` -- nearest-neighbor lookup store
  - `insert(label, vector)` -- add labeled vector
  - `lookup(vector, top_k)` -- find k most similar items

### Encoding (`encoder.rs`)

- `encode_document(input, codebook)` -- full document fingerprint
  - Formula: `weighted_bundle(content=5, tags=3, path=2)` using byte-trigram content encoding, role-bound tags, and path segments
- `encode_text(text, codebook)` -- plain text encoding (byte-trigram bundling)
- `DocumentEncodingInput` -- struct with `content`, `tags`, `path` fields

### Deduplication (`dedup.rs`)

- `DedupConfig` -- threshold configuration (`similar_threshold`, `duplicate_threshold`)
- `DedupDecision<Id>` -- enum: `Unique`, `Similar { id, path, similarity }`, `Duplicate { id, path, similarity }`
- `check_dedup(query, candidates, config)` -- compare fingerprint against candidates

### Scanning (`scan.rs`)

- `top_k_scan(query, candidates, top_k)` -- brute-force top-k by Hamming similarity
- `ScanResult<Id>` -- result with `id` and `similarity`

### Errors (`error.rs`)

- `HdcError` -- thiserror enum: `InvalidBytes`, `InvalidDecayFactor`, `InvalidDedupConfig`, `EncodingError`, `NotFound`, `EmptyInput`

## Architecture

```
vector -> bundle -> codebook -> encoder -> dedup/scan
```

- `vector.rs`: primitive operations (XOR, permute, Hamming, seeded construction)
- `bundle.rs`: aggregation (majority vote, exponential decay)
- `codebook.rs`: symbol-to-vector registry + item memory for lookup
- `encoder.rs`: high-level document/text encoding using codebook + bundle
- `dedup.rs`: threshold-based deduplication decisions
- `scan.rs`: brute-force nearest-neighbor scanning

## Performance Baselines

These are benchmark baselines to watch, not correctness guarantees. Keep regressions visible with Criterion and update the numbers when the encoder or benchmark hardware changes.

| Operation | Current expectation | Notes |
|-----------|--------|-------|
| XOR bind | sub-microsecond | Single pass over 160 words |
| Hamming distance | sub-microsecond | popcount over 160 words |
| Permute | low microseconds | Bit-level cyclic shift |
| Bundle (100 vectors) | low milliseconds | Vote accumulation + finalize |
| encode_document | low tens of milliseconds for 1KB | Byte-trigram encoding plus role/path bundling |
| top_k_scan (10k items) | low milliseconds | Brute force, no index |

## Testing

- **Unit tests**: in each module (`#[cfg(test)]` blocks)
- **Property tests**: proptest for algebraic properties (XOR self-inverse, permute cycle, bundle convergence)
- **Golden tests**: seed stability -- `from_seed` output must never change
- **Criterion benchmarks**: `benches/operations.rs` (bind, permute, hamming, bundle), `benches/scan.rs` (top-k scanning)

Run tests:
```bash
cargo test -p ironclaw_hdc
```

Run benchmarks:
```bash
cargo bench -p ironclaw_hdc
```

## How to Add New Encoders

1. Add a new public function in `encoder.rs` (or a new file if large)
2. Use `Codebook` for symbol vectors and `BundleAccumulator` for aggregation
3. Follow the pattern: tokenize input -> map tokens to vectors (via codebook) -> bind with role vectors -> bundle
4. Add position encoding via `permute()` if token order matters
5. Add unit tests and a benchmark case

## Relationship to Main Crate

- Optional dependency behind `hdc` feature flag in the workspace
- Main crate uses `encode_document` for optional workspace fingerprints and `check_dedup` for the current memory-write dedup preflight path
- No reverse dependency -- this crate is fully standalone
- Integration point: workspace write/append/patch paths can store fingerprints in shadow mode; active duplicate handling is currently gated in `MemoryWriteTool`

## Determinism Guarantees

- `from_seed(domain, key)` uses FNV-1a hash of `domain`, a NUL separator, and `key` as a SplitMix64 seed -- platform-independent, no external RNG
- `Codebook.get_or_create(symbol)` is deterministic (calls `from_seed("codebook", symbol)`)
- `encode_document` is deterministic for same input (same content + tags + path = same fingerprint)
- Domain separation in `from_seed` prevents collisions between codebook entries, role markers, and special vectors

## Conventions

- Little-endian byte order for all serialization (`to_bytes` / `from_bytes`)
- Domain separation: `from_seed(domain, key)` namespaces prevent cross-purpose collisions
- Ties in bundle majority-vote resolve to 0 (conservative, reduces false similarity)
- Similarity is normalized Hamming: `1.0 - (hamming_distance / DIMENSION_BITS)`
- Random baseline similarity is ~0.5 (uncorrelated binary vectors)
- `DedupConfig` requires `similar_threshold > 0.5` (must exceed random baseline)

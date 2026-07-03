[← Back to HDC Overview](./README.md)

# Benchmarking and Performance Analysis

This document covers per-operation timing, scan performance, memory usage, false positive analysis, the benchmark harness code, and the A/B comparison methodology for measuring HDC's search quality contribution.

---

## Per-Operation Timing

| Operation | Time | Notes |
|---|---|---|
| `bind()` (XOR) | ~5 ns | 160 XOR on u64 words |
| `permute()` (rotate) | ~10 ns | 160 shift+OR with cross-word logic |
| `similarity()` (Hamming) | ~13 ns | 160 XOR + POPCNT (auto-vectorized) |
| `bundle()` (K=10) | ~800 ns | O(D×K), memory-bound |
| `bundle()` (K=100) | ~8 µs | Linear in K |
| `from_seed()` | ~100 ns | FNV-1a + 160 splitmix64 steps |
| `to_bytes()` / `from_bytes()` | ~50 ns | Memcpy 1,280 bytes |
| `BundleAccumulator::add()` | ~300 ns | 10,240 i32 updates |
| `BundleAccumulator::finish()` | ~200 ns | 10,240 threshold comparisons |

---

## Scan Performance

| Knowledge Base Size | Brute Force Scan | Time per Query |
|---|---|---|
| 1,000 entries | 1,000 comparisons | ~13 µs |
| 10,000 entries | 10,000 comparisons | ~130 µs |
| 100,000 entries | 100,000 comparisons | ~1.3 ms |
| 1,000,000 entries | 1,000,000 comparisons | ~13 ms |

---

## Memory Usage Analysis

| Component | Size | Notes |
|---|---|---|
| One HdcVector | 1,280 bytes | Stack-allocated, `Copy` |
| BundleAccumulator | ~40 KB | 10,240 × i32 votes |
| DecayingBundleAccumulator | ~40 KB | 10,240 × f32 votes |
| 1,000 stored vectors | ~1.25 MB | Raw vectors only |
| 100,000 stored vectors | ~125 MB | Raw vectors only |
| 800,000 vectors | ~1 GB | Fits in RAM with overhead |

**Memory strategy for large corpora**: For corpora exceeding 500K entries, consider:

1. **Memory-mapped storage**: Use `rkyv` zero-copy deserialization from mmap'd files. The archived `[u64; 160]` layout is platform-identical, enabling direct similarity computation against the mmap'd buffer without copying.
2. **Tiered filtering**: Use a 1,024-bit summary vector (8× compression) as a first-pass filter. Only run full 10,240-bit comparisons on candidates passing the summary filter.
3. **HNSW index**: For million-scale corpora, maintain an HNSW (Hierarchical Navigable Small World) index over the Hamming space. This gives approximate nearest-neighbor search in O(log N) per query at ~95% recall.

---

## Comparison with Float Embedding Approaches

| Metric | HDC (10,240-bit) | Float Embedding (1,536-dim float32) |
|---|---|---|
| Storage per vector | 1,280 bytes | 6,144 bytes (4.8× more) |
| Comparison time | ~13 ns (XOR+POPCNT) | ~500 ns (dot product, scalar) |
| GPU required | No | Strongly recommended for batches |
| Model dependency | None | Requires specific embedding model |
| Determinism | Perfect (seeded PRNG) | Depends on model version |
| Composability | Native (XOR bind, majority bundle) | None (must re-embed) |
| Semantic fidelity | Lower (bag-of-words level) | Higher (contextual meaning) |
| Requires training data | No | Yes |
| Can encode structured relations | Yes (role-filler binding) | No |
| Works offline | Yes | Usually no (API call) |

**Note on semantic fidelity**: HDC's `from_seed()` maps exact byte strings to vectors, so "running" and "jogging" produce quasi-orthogonal vectors unless explicitly linked in a codebook. Neural embeddings capture contextual synonymy that HDC cannot. The two approaches are **complementary**: HDC excels at compositional structure and speed; neural embeddings excel at semantic similarity. Using both as separate ranking signals with RRF gives the best results.

---

## Benchmark Harness

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

---

## False Positive Analysis and Threshold Selection

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
| 0.505 | 1.01 | 15.6% | Too low — noise |
| 0.510 | 2.02 | 2.15% | Rough screening |
| 0.515 | 3.04 | 1.2×10^-3 | Conservative single-pair |
| 0.520 | 4.05 | 2.6×10^-5 | Moderate vocabulary |
| **0.526** | **5.26** | **7.1×10^-8** | **100K vocabulary (Bonferroni)** |
| 0.530 | 6.07 | 6.3×10^-10 | 1M vocabulary |
| 0.540 | 8.10 | <10^-15 | Extremely conservative |

### Bonferroni Correction

When scanning N entries, the probability of at least one false positive is approximately N × P(single FP). To maintain an overall false positive rate of alpha:

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

Use **0.526** as the initial threshold candidate for a 100K-entry scan. It should produce <1% random-pair false positives under the independence assumptions above, but production rollout must validate it against IronClaw's real corpus, encoders, and retrieval workload.

### False Positive Rate Measurement Code

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

**Interpreting the output**: When you run this against a real workspace corpus, you may see slightly higher FP rates than the theoretical 7×10^-8 because real text-derived vectors are not perfectly random — they share structural patterns (common words, common role prefixes). If your empirical FP rate at threshold 0.526 exceeds 0.01% across your corpus size, raise the threshold to 0.530.

---

## A/B Comparison Methodology: HDC vs. Embeddings vs. Fusion

To measure whether adding HDC as a third RRF signal improves search quality:

> Captured implementation omitted. Rebuild IronClaw code locally in owner modules with caller-level tests.

### Practical Guidance for Running the A/B Comparison

1. Collect a minimum of 200 labelled (query, relevant_ids) pairs from actual IronClaw usage — either via user feedback annotations or by logging which memory entries an agent actually referenced after a search.
2. Run System A (FTS only) first to establish the baseline. For a small corpus (<5K entries), expect MRR@10 ≈ 0.35–0.55.
3. Add System B (FTS + embeddings via `ironclaw_embeddings`). Expect MRR@10 to rise by 0.10–0.20.
4. Add System C (FTS + embeddings + HDC). Expect MRR@10 to rise by an additional 0.02–0.08, concentrated in queries with structural patterns (tag-based, path-based, role-filler queries).
5. HDC's marginal gain is **largest for structured queries** ("find all memories tagged async about error handling") and smallest for purely semantic queries ("what did I learn about managing burnout"). Weight your eval set accordingly.

---

Next: [IronClaw Integration Plan](./ironclaw-integration.md)

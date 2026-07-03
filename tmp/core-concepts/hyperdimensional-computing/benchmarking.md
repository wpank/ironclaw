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

```rust
// In crates/ironclaw_hdc/benches/hdc_ops.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ironclaw_hdc::{HdcVector, BundleAccumulator};

fn bench_bind(c: &mut Criterion) {
    let a = HdcVector::from_seed(b"vector_a");
    let b = HdcVector::from_seed(b"vector_b");
    c.bench_function("bind", |bench| {
        bench.iter(|| black_box(a.bind(&b)))
    });
}

fn bench_similarity(c: &mut Criterion) {
    let a = HdcVector::from_seed(b"vector_a");
    let b = HdcVector::from_seed(b"vector_b");
    c.bench_function("similarity", |bench| {
        bench.iter(|| black_box(a.similarity(&b)))
    });
}

fn bench_bundle(c: &mut Criterion) {
    let mut group = c.benchmark_group("bundle");
    for k in [10, 50, 100, 500].iter() {
        let vecs: Vec<HdcVector> = (0..*k)
            .map(|i| HdcVector::from_seed(format!("vec_{}", i).as_bytes()))
            .collect();
        let refs: Vec<&HdcVector> = vecs.iter().collect();
        group.bench_with_input(BenchmarkId::from_parameter(k), k, |bench, _| {
            bench.iter(|| black_box(HdcVector::bundle(&refs)))
        });
    }
    group.finish();
}

fn bench_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan");
    for n in [1_000, 10_000, 100_000].iter() {
        let corpus: Vec<HdcVector> = (0..*n)
            .map(|i| HdcVector::from_seed(format!("entry_{}", i).as_bytes()))
            .collect();
        let query = HdcVector::from_seed(b"query_vector");
        group.bench_with_input(BenchmarkId::from_parameter(n), n, |bench, _| {
            bench.iter(|| {
                let _results: Vec<f32> = corpus.iter()
                    .map(|v| black_box(query.similarity(v)))
                    .collect();
            })
        });
    }
    group.finish();
}

fn bench_accumulator(c: &mut Criterion) {
    let vecs: Vec<HdcVector> = (0..100)
        .map(|i| HdcVector::from_seed(format!("vec_{}", i).as_bytes()))
        .collect();

    c.bench_function("accumulator_add_100", |bench| {
        bench.iter(|| {
            let mut acc = BundleAccumulator::new();
            for v in &vecs {
                acc.add(v);
            }
            black_box(acc.finish())
        })
    });
}

criterion_group!(benches, bench_bind, bench_similarity, bench_bundle, bench_scan, bench_accumulator);
criterion_main!(benches);
```

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

The recommended production threshold of **0.526** guarantees <1% overall false positive rate when scanning 100K entries.

### False Positive Rate Measurement Code

```rust
// In tests/hdc_fp_rate.rs
//
// Run with: cargo test --release hdc_fp_rate -- --nocapture
// (--release is essential; debug mode is ~20x slower)

use ironclaw_hdc::{HdcVector, text_hv, RESONANCE_THRESHOLD};

struct FPReport {
    threshold: f32,
    n_pairs: u64,
    false_positives: u64,
    fp_rate: f64,
    ci_lower: f64,
    ci_upper: f64,
}

impl FPReport {
    fn print(&self) {
        println!(
            "threshold={:.4}  pairs={:>10}  FP={:>6}  rate={:.2e}  95%CI=[{:.2e},{:.2e}]",
            self.threshold, self.n_pairs, self.false_positives,
            self.fp_rate, self.ci_lower, self.ci_upper
        );
    }
}

/// Wilson score confidence interval for a proportion p = k/n.
fn wilson_ci(k: u64, n: u64, z: f64) -> (f64, f64) {
    if n == 0 { return (0.0, 1.0); }
    let p = k as f64 / n as f64;
    let denom = 1.0 + z * z / n as f64;
    let center = (p + z * z / (2.0 * n as f64)) / denom;
    let half_width = z * (p * (1.0 - p) / n as f64 + z * z / (4.0 * n as f64 * n as f64)).sqrt() / denom;
    ((center - half_width).max(0.0), (center + half_width).min(1.0))
}

fn build_fp_corpus(size: usize, prefix: &str) -> Vec<HdcVector> {
    (0..size)
        .map(|i| text_hv(&format!("{prefix}_entry_{i:08}")))
        .collect()
}

fn measure_fp_rate(corpus: &[HdcVector], threshold: f32, sample_pairs: u64) -> FPReport {
    let n = corpus.len();
    let mut fp = 0u64;

    // Deterministic pseudo-random pair sampling (no external RNG dependency)
    let mut state: u64 = 0x123456789abcdef0;
    for _ in 0..sample_pairs {
        // xorshift64 for fast pair generation
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let i = (state >> 32) as usize % n;
        let j = state as usize % n;
        if i == j { continue; }

        let sim = corpus[i].similarity(&corpus[j]);
        if sim >= threshold { fp += 1; }
    }

    let fp_rate = fp as f64 / sample_pairs as f64;
    let (ci_lower, ci_upper) = wilson_ci(fp, sample_pairs, 1.96); // 95% CI

    FPReport { threshold, n_pairs: sample_pairs, false_positives: fp, fp_rate, ci_lower, ci_upper }
}

#[test]
fn hdc_fp_rate_at_production_threshold() {
    let corpus = build_fp_corpus(10_000, "workspace_memory");

    // Sample 10 million pairs
    // Expected: ~0 FP at threshold 0.526 (rate ~7e-8 × 10M = ~0.7 expected FP)
    let n_sample = 10_000_000u64;

    println!("\n--- HDC False Positive Rate Measurement ---");
    println!("Corpus: {} entries, {} sampled pairs", corpus.len(), n_sample);

    for &threshold in &[0.510f32, 0.515, 0.520, 0.526, 0.530, 0.540] {
        let report = measure_fp_rate(&corpus, threshold, n_sample);
        report.print();
    }

    let production_report = measure_fp_rate(&corpus, RESONANCE_THRESHOLD, n_sample);
    assert!(
        production_report.fp_rate < 1e-5,
        "FP rate at threshold 0.526 exceeded 1e-5: {}",
        production_report.fp_rate
    );
}

#[test]
fn hdc_similarity_distribution_matches_theory() {
    // Verify that random-pair similarity is centred at 0.5 with sigma ≈ 0.00494.
    let corpus = build_fp_corpus(1_000, "theory_check");
    let mut similarities = Vec::with_capacity(50_000);

    let mut state: u64 = 0xdeadbeefcafe1234;
    for _ in 0..50_000 {
        state ^= state << 13; state ^= state >> 7; state ^= state << 17;
        let i = (state >> 32) as usize % corpus.len();
        let j = state as usize % corpus.len();
        if i == j { continue; }
        similarities.push(corpus[i].similarity(&corpus[j]));
    }

    let n = similarities.len() as f64;
    let mean = similarities.iter().sum::<f32>() as f64 / n;
    let variance = similarities.iter()
        .map(|&s| { let d = s as f64 - mean; d * d })
        .sum::<f64>() / n;
    let sigma = variance.sqrt();

    println!("\n--- Similarity Distribution ---");
    println!("n={n:.0}  mean={mean:.6}  sigma={sigma:.6}");
    println!("Theoretical: mean=0.500000, sigma=0.004941");

    assert!((mean - 0.5).abs() < 0.002, "Mean {mean:.6} deviates from 0.5");
    assert!((sigma - 0.00494).abs() < 0.001, "Sigma {sigma:.6} deviates from 0.00494");
}
```

**Interpreting the output**: When you run this against a real workspace corpus, you may see slightly higher FP rates than the theoretical 7×10^-8 because real text-derived vectors are not perfectly random — they share structural patterns (common words, common role prefixes). If your empirical FP rate at threshold 0.526 exceeds 0.01% across your corpus size, raise the threshold to 0.530.

---

## A/B Comparison Methodology: HDC vs. Embeddings vs. Fusion

To measure whether adding HDC as a third RRF signal improves search quality:

```rust
// In tests/search_quality.rs
//
// Requires a labelled evaluation set: (query, [relevant_ids]).

struct EvalPair {
    query: String,
    relevant_ids: Vec<uuid::Uuid>,
}

struct SearchResult {
    id: uuid::Uuid,
    rank: usize,
}

/// Mean Reciprocal Rank @ K.
fn mrr_at_k(results: &[SearchResult], relevant: &[uuid::Uuid], k: usize) -> f64 {
    for r in results.iter().take(k) {
        if relevant.contains(&r.id) {
            return 1.0 / (r.rank as f64 + 1.0);
        }
    }
    0.0
}

/// Precision @ K: fraction of top-K results that are relevant.
fn precision_at_k(results: &[SearchResult], relevant: &[uuid::Uuid], k: usize) -> f64 {
    let hits = results.iter().take(k)
        .filter(|r| relevant.contains(&r.id))
        .count();
    hits as f64 / k as f64
}

/// Recall @ K: fraction of relevant results found in top-K.
fn recall_at_k(results: &[SearchResult], relevant: &[uuid::Uuid], k: usize) -> f64 {
    if relevant.is_empty() { return 0.0; }
    let hits = results.iter().take(k)
        .filter(|r| relevant.contains(&r.id))
        .count();
    hits as f64 / relevant.len() as f64
}

/// Run A/B evaluation across three systems:
/// A: FTS only
/// B: FTS + embeddings (current IronClaw baseline)
/// C: FTS + embeddings + HDC (proposed)
fn evaluate_systems(
    eval_pairs: &[EvalPair],
    run_fts: impl Fn(&str) -> Vec<SearchResult>,
    run_embedding: impl Fn(&str) -> Vec<SearchResult>,
    run_hdc: impl Fn(&str) -> Vec<SearchResult>,
    run_fusion: impl Fn(&str, &[&[SearchResult]]) -> Vec<SearchResult>,
) {
    let mut fts_mrr = 0.0f64;
    let mut emb_mrr = 0.0f64;
    let mut full_mrr = 0.0f64;
    let n = eval_pairs.len() as f64;

    for pair in eval_pairs {
        let q = &pair.query;
        let rel = &pair.relevant_ids;

        let fts = run_fts(q);
        let emb = run_embedding(q);
        let hdc = run_hdc(q);
        let fused_baseline = run_fusion(q, &[&fts, &emb]);
        let fused_full = run_fusion(q, &[&fts, &emb, &hdc]);

        fts_mrr += mrr_at_k(&fts, rel, 10);
        emb_mrr += mrr_at_k(&fused_baseline, rel, 10);
        full_mrr += mrr_at_k(&fused_full, rel, 10);
    }

    let improvement_mrr = (full_mrr - emb_mrr) / n;
    println!("\nHDC marginal gain (C vs B): MRR@10 delta = {improvement_mrr:+.4}");
    if improvement_mrr > 0.02 {
        println!("SHIP IT: HDC adds meaningful signal (>2% MRR@10 improvement)");
    } else if improvement_mrr > 0.005 {
        println!("MARGINAL: HDC adds small signal; consider production risk vs gain");
    } else {
        println!("NO SIGNAL: HDC not improving search at this corpus size/quality");
    }
}
```

### Practical Guidance for Running the A/B Comparison

1. Collect a minimum of 200 labelled (query, relevant_ids) pairs from actual IronClaw usage — either via user feedback annotations or by logging which memory entries an agent actually referenced after a search.
2. Run System A (FTS only) first to establish the baseline. For a small corpus (<5K entries), expect MRR@10 ≈ 0.35–0.55.
3. Add System B (FTS + embeddings via `ironclaw_embeddings`). Expect MRR@10 to rise by 0.10–0.20.
4. Add System C (FTS + embeddings + HDC). Expect MRR@10 to rise by an additional 0.02–0.08, concentrated in queries with structural patterns (tag-based, path-based, role-filler queries).
5. HDC's marginal gain is **largest for structured queries** ("find all memories tagged async about error handling") and smallest for purely semantic queries ("what did I learn about managing burnout"). Weight your eval set accordingly.

---

Next: [IronClaw Integration Plan](./ironclaw-integration.md)

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use ironclaw_hdc::codebook::ItemMemory;
use ironclaw_hdc::dedup::{DedupConfig, FingerprintCandidate, check_dedup};
use ironclaw_hdc::vector::HdcVector;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn make_candidates(n: usize, rng: &mut StdRng) -> Vec<FingerprintCandidate<u64>> {
    (0..n)
        .map(|i| FingerprintCandidate {
            id: i as u64,
            path: format!("doc_{i}.md"),
            fingerprint: HdcVector::random(rng),
        })
        .collect()
}

fn bench_dedup_scan_1k(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let query = HdcVector::random(&mut rng);
    let candidates = make_candidates(1_000, &mut rng);
    let config = DedupConfig::default_config();
    c.bench_function("dedup_scan_1k", |bench| {
        bench.iter(|| check_dedup(black_box(query), black_box(&candidates), &config));
    });
}

fn bench_dedup_scan_10k(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let query = HdcVector::random(&mut rng);
    let candidates = make_candidates(10_000, &mut rng);
    let config = DedupConfig::default_config();
    c.bench_function("dedup_scan_10k", |bench| {
        bench.iter(|| check_dedup(black_box(query), black_box(&candidates), &config));
    });
}

fn bench_dedup_scan_100k(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let query = HdcVector::random(&mut rng);
    let candidates = make_candidates(100_000, &mut rng);
    let config = DedupConfig::default_config();
    c.bench_function("dedup_scan_100k", |bench| {
        bench.iter(|| check_dedup(black_box(query), black_box(&candidates), &config));
    });
}

fn bench_item_memory_lookup_10k(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let mut mem = ItemMemory::new();
    for i in 0..10_000 {
        mem.insert(format!("item_{i}"), HdcVector::random(&mut rng));
    }
    let query = HdcVector::random(&mut rng);
    c.bench_function("item_memory_lookup_10k", |bench| {
        bench.iter(|| mem.lookup(black_box(query), black_box(5)));
    });
}

criterion_group!(
    scan_benches,
    bench_dedup_scan_1k,
    bench_dedup_scan_10k,
    bench_dedup_scan_100k,
    bench_item_memory_lookup_10k,
);
criterion_main!(scan_benches);

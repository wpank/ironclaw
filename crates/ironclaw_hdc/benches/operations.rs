use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use ironclaw_hdc::bundle::{BundleAccumulator, DecayingBundleAccumulator};
use ironclaw_hdc::codebook::Codebook;
use ironclaw_hdc::encoder::{DocumentEncodingInput, encode_document, encode_text};
use ironclaw_hdc::vector::HdcVector;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn bench_bind(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let a = HdcVector::random(&mut rng);
    let b = HdcVector::random(&mut rng);
    c.bench_function("bind", |bench| {
        bench.iter(|| black_box(a).bind(black_box(b)));
    });
}

fn bench_similarity(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let a = HdcVector::random(&mut rng);
    let b = HdcVector::random(&mut rng);
    c.bench_function("similarity", |bench| {
        bench.iter(|| black_box(a).similarity(black_box(b)));
    });
}

fn bench_hamming(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let a = HdcVector::random(&mut rng);
    let b = HdcVector::random(&mut rng);
    c.bench_function("hamming_distance", |bench| {
        bench.iter(|| black_box(a).hamming_distance(black_box(b)));
    });
}

fn bench_permute_1(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let a = HdcVector::random(&mut rng);
    c.bench_function("permute_1", |bench| {
        bench.iter(|| black_box(a).permute(black_box(1)));
    });
}

fn bench_permute_5000(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let a = HdcVector::random(&mut rng);
    c.bench_function("permute_5000", |bench| {
        bench.iter(|| black_box(a).permute(black_box(5000)));
    });
}

fn bench_from_seed(c: &mut Criterion) {
    c.bench_function("from_seed", |bench| {
        bench.iter(|| HdcVector::from_seed(black_box("d"), black_box(b"test")));
    });
}

fn bench_bundle_10(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let vectors: Vec<HdcVector> = (0..10).map(|_| HdcVector::random(&mut rng)).collect();
    c.bench_function("bundle_10", |bench| {
        bench.iter(|| {
            let mut acc = BundleAccumulator::new();
            for v in &vectors {
                acc.add(black_box(v));
            }
            black_box(acc.finalize())
        });
    });
}

fn bench_bundle_100(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let vectors: Vec<HdcVector> = (0..100).map(|_| HdcVector::random(&mut rng)).collect();
    c.bench_function("bundle_100", |bench| {
        bench.iter(|| {
            let mut acc = BundleAccumulator::new();
            for v in &vectors {
                acc.add(black_box(v));
            }
            black_box(acc.finalize())
        });
    });
}

fn bench_finalize(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let mut acc = BundleAccumulator::new();
    for _ in 0..50 {
        acc.add(&HdcVector::random(&mut rng));
    }
    c.bench_function("finalize", |bench| {
        bench.iter(|| black_box(acc.finalize()));
    });
}

fn bench_decay_add(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let v = HdcVector::random(&mut rng);
    c.bench_function("decay_add", |bench| {
        bench.iter_batched(
            || DecayingBundleAccumulator::new(0.95).unwrap(),
            |mut acc| {
                acc.add(black_box(&v));
                black_box(acc)
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_encode_text_1kb(c: &mut Criterion) {
    let text: String = "the quick brown fox jumps over the lazy dog. "
        .repeat(23)
        .chars()
        .take(1024)
        .collect();
    assert!(text.len() >= 1000);
    c.bench_function("encode_text_1kb", |bench| {
        bench.iter_batched(
            Codebook::new,
            |mut cb| black_box(encode_text(black_box(&text), &mut cb)),
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_encode_document_1kb_5tags(c: &mut Criterion) {
    let content: String = "the quick brown fox jumps over the lazy dog. "
        .repeat(23)
        .chars()
        .take(1024)
        .collect();
    let tags: Vec<&str> = vec!["rust", "hdc", "memory", "dedup", "vector"];
    let path = "src/workspace/memory.rs";
    c.bench_function("encode_document_1kb_5tags", |bench| {
        bench.iter_batched(
            Codebook::new,
            |mut cb| {
                black_box(encode_document(
                    DocumentEncodingInput {
                        content: black_box(&content),
                        tags: black_box(&tags),
                        path: black_box(path),
                    },
                    &mut cb,
                ))
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_bind,
    bench_similarity,
    bench_hamming,
    bench_permute_1,
    bench_permute_5000,
    bench_from_seed,
    bench_bundle_10,
    bench_bundle_100,
    bench_finalize,
    bench_decay_add,
    bench_encode_text_1kb,
    bench_encode_document_1kb_5tags,
);
criterion_main!(benches);

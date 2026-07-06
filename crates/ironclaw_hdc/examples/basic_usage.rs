//! Basic usage of the HDC crate.
//! Run with: cargo run -p ironclaw_hdc --example basic_usage

use ironclaw_hdc::bundle::BundleAccumulator;
use ironclaw_hdc::codebook::Codebook;
use ironclaw_hdc::dedup::{DedupConfig, FingerprintCandidate, check_dedup};
use ironclaw_hdc::encoder::{DocumentEncodingInput, encode_document, encode_text};
use ironclaw_hdc::vector::HdcVector;

fn main() {
    println!("=== HDC Basic Usage ===\n");

    // 1. Creating vectors
    let v1 = HdcVector::from_seed("demo", b"hello");
    let v2 = HdcVector::from_seed("demo", b"world");
    println!("Similarity of random vectors: {:.4}", v1.similarity(v2));
    println!("Self-similarity: {:.4}", v1.similarity(v1));

    // 2. Binding (XOR)
    let bound = v1.bind(v2);
    println!("\nBinding (XOR):");
    println!("  Similarity after bind: {:.4}", v1.similarity(bound));
    let recovered = bound.bind(v2); // XOR is self-inverse
    println!("  Recovered original: {}", recovered == v1);

    // 3. Bundling (majority vote)
    let mut bundle = BundleAccumulator::new();
    bundle.add(&v1);
    bundle.add(&v1);
    bundle.add(&v1);
    bundle.add(&v2);
    let result = bundle.finalize();
    println!("\nBundling (majority vote, 3xv1 + 1xv2):");
    println!("  Similarity to v1: {:.4}", result.similarity(v1));
    println!("  Similarity to v2: {:.4}", result.similarity(v2));

    // 4. Document encoding
    let mut codebook = Codebook::new();
    let doc1 = encode_document(
        DocumentEncodingInput {
            content: "Rust async programming with tokio runtime",
            tags: &["rust", "async", "tokio"],
            path: "notes/rust-async.md",
        },
        &mut codebook,
    );
    let doc2 = encode_document(
        DocumentEncodingInput {
            content: "Rust async patterns using tokio channels",
            tags: &["rust", "async", "channels"],
            path: "notes/rust-channels.md",
        },
        &mut codebook,
    );
    let doc3 = encode_document(
        DocumentEncodingInput {
            content: "French cooking: how to make a perfect bechamel sauce",
            tags: &["cooking", "french", "sauces"],
            path: "recipes/bechamel.md",
        },
        &mut codebook,
    );

    println!("\nDocument similarities:");
    println!(
        "  doc1 vs doc2 (similar topics): {:.4}",
        doc1.similarity(doc2)
    );
    println!(
        "  doc1 vs doc3 (different topics): {:.4}",
        doc1.similarity(doc3)
    );

    // 5. Text encoding
    let mut cb = Codebook::new();
    let t1 = encode_text("the quick brown fox jumps over the lazy dog", &mut cb);
    let t2 = encode_text("the quick brown fox leaps over the lazy dog", &mut cb);
    let t3 = encode_text("a completely different sentence about mathematics", &mut cb);
    println!("\nText similarities:");
    println!("  similar sentences: {:.4}", t1.similarity(t2));
    println!("  different sentences: {:.4}", t1.similarity(t3));

    // 6. Dedup checking
    let config = DedupConfig::default_config();
    let candidates = vec![
        FingerprintCandidate {
            id: 1u32,
            path: "notes/rust-async.md".to_string(),
            fingerprint: doc1,
        },
        FingerprintCandidate {
            id: 2,
            path: "recipes/bechamel.md".to_string(),
            fingerprint: doc3,
        },
    ];

    let decision = check_dedup(doc2, &candidates, &config);
    println!("\nDedup check for doc2 against existing:");
    println!("  Decision: {:?}", decision);
}

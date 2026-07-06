//! Deduplication scenarios demonstrating exact duplicates, near-duplicates, and unique content.
//! Run with: cargo run -p ironclaw_hdc --example dedup_demo

use ironclaw_hdc::codebook::Codebook;
use ironclaw_hdc::dedup::{DedupConfig, DedupDecision, FingerprintCandidate, check_dedup};
use ironclaw_hdc::encoder::{DocumentEncodingInput, encode_document};

fn main() {
    println!("=== HDC Deduplication Demo ===\n");

    let mut codebook = Codebook::new();
    let config = DedupConfig::default_config();
    println!(
        "Config: similar_threshold={}, duplicate_threshold={}\n",
        config.similar_threshold, config.duplicate_threshold
    );

    // Build a corpus of existing documents
    let existing_docs = [
        (
            "Rust error handling with Result and Option types",
            &["rust", "errors"][..],
            "notes/rust-errors.md",
        ),
        (
            "Setting up PostgreSQL with Docker for local development",
            &["postgres", "docker", "devops"][..],
            "guides/postgres-docker.md",
        ),
        (
            "Introduction to machine learning with Python and scikit-learn",
            &["python", "ml", "scikit"][..],
            "tutorials/ml-intro.md",
        ),
    ];

    let candidates: Vec<FingerprintCandidate<usize>> = existing_docs
        .iter()
        .enumerate()
        .map(|(i, (content, tags, path))| {
            let fp = encode_document(
                DocumentEncodingInput {
                    content,
                    tags,
                    path,
                },
                &mut codebook,
            );
            FingerprintCandidate {
                id: i,
                path: path.to_string(),
                fingerprint: fp,
            }
        })
        .collect();

    // --- Scenario 1: Exact duplicate ---
    println!("--- Scenario 1: Exact Duplicate ---");
    let exact_dup = encode_document(
        DocumentEncodingInput {
            content: "Rust error handling with Result and Option types",
            tags: &["rust", "errors"],
            path: "notes/rust-errors.md",
        },
        &mut codebook,
    );
    let decision = check_dedup(exact_dup, &candidates, &config);
    print_decision("Exact same content, tags, and path", &decision);

    // --- Scenario 2: Near-duplicate (minor edit) ---
    println!("\n--- Scenario 2: Near-Duplicate (Minor Edit) ---");
    let near_dup = encode_document(
        DocumentEncodingInput {
            content: "Rust error handling with Result and Option types for robust applications",
            tags: &["rust", "errors"],
            path: "notes/rust-errors.md",
        },
        &mut codebook,
    );
    let decision = check_dedup(near_dup, &candidates, &config);
    print_decision("Same topic with a few extra words appended", &decision);

    // --- Scenario 3: Same topic, different wording ---
    println!("\n--- Scenario 3: Related Topic, Different Wording ---");
    let related = encode_document(
        DocumentEncodingInput {
            content: "How to use Result<T,E> and the ? operator in Rust programs",
            tags: &["rust", "errors", "patterns"],
            path: "notes/rust-question-mark.md",
        },
        &mut codebook,
    );
    let decision = check_dedup(related, &candidates, &config);
    print_decision("Related Rust error topic with different wording", &decision);

    // --- Scenario 4: Completely unique ---
    println!("\n--- Scenario 4: Completely Unique ---");
    let unique = encode_document(
        DocumentEncodingInput {
            content: "Baking sourdough bread: maintaining a starter and shaping techniques",
            tags: &["baking", "bread", "sourdough"],
            path: "recipes/sourdough.md",
        },
        &mut codebook,
    );
    let decision = check_dedup(unique, &candidates, &config);
    print_decision("Entirely different domain (baking)", &decision);

    // --- Scenario 5: Same content, different path ---
    println!("\n--- Scenario 5: Same Content, Different Path ---");
    let moved = encode_document(
        DocumentEncodingInput {
            content: "Rust error handling with Result and Option types",
            tags: &["rust", "errors"],
            path: "archive/2024/rust-errors-old.md",
        },
        &mut codebook,
    );
    let decision = check_dedup(moved, &candidates, &config);
    print_decision("Same content but moved to a different path", &decision);

    // --- Show similarity matrix ---
    println!("\n--- Similarity Matrix ---");
    let all_fps = [exact_dup, near_dup, related, unique, moved];
    let labels = ["exact", "near", "related", "unique", "moved"];
    print!("{:>10}", "");
    for label in &labels {
        print!("{:>10}", label);
    }
    println!();
    for (i, label) in labels.iter().enumerate() {
        print!("{:>10}", label);
        for j in 0..labels.len() {
            print!("{:>10.4}", all_fps[i].similarity(all_fps[j]));
        }
        println!();
    }
}

fn print_decision(description: &str, decision: &DedupDecision<usize>) {
    println!("  Input: {description}");
    match decision {
        DedupDecision::Unique => {
            println!("  Result: UNIQUE - no similar documents found");
        }
        DedupDecision::Similar {
            path, similarity, ..
        } => {
            println!("  Result: SIMILAR to \"{path}\" (similarity: {similarity:.4})");
            println!("  Action: warn user, allow write with note");
        }
        DedupDecision::Duplicate {
            path, similarity, ..
        } => {
            println!("  Result: DUPLICATE of \"{path}\" (similarity: {similarity:.4})");
            println!("  Action: block write, suggest updating existing");
        }
    }
}

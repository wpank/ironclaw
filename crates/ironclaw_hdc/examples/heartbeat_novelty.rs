//! Heartbeat novelty detection using DecayingBundleAccumulator.
//!
//! Demonstrates how the decaying accumulator can track a running "summary"
//! of recent observations and detect when new input is novel vs. redundant.
//!
//! Run with: cargo run -p ironclaw_hdc --example heartbeat_novelty

use ironclaw_hdc::bundle::DecayingBundleAccumulator;
use ironclaw_hdc::codebook::Codebook;
use ironclaw_hdc::encoder::{DocumentEncodingInput, encode_document, encode_text};

fn main() {
    println!("=== HDC Heartbeat Novelty Detection ===\n");

    // The decaying accumulator maintains a recency-weighted summary.
    // New observations are compared against this summary to determine novelty.
    // High similarity to the summary = redundant (seen recently)
    // Low similarity to the summary = novel (worth reporting)

    let decay_factor = 0.85;
    let novelty_threshold = 0.55; // above random (0.5) but below "same topic"

    let mut summary = DecayingBundleAccumulator::new(decay_factor).unwrap();
    let mut codebook = Codebook::new();

    println!(
        "Decay factor: {decay_factor} (half-life: {:.1} cycles)",
        summary.effective_half_life_cycles()
    );
    println!("Novelty threshold: {novelty_threshold}");
    println!("(similarity < threshold = NOVEL, >= threshold = REDUNDANT)\n");

    // Simulate heartbeat observations over time
    let observations = vec![
        // First few: establish a baseline about Rust programming
        (
            "Rust borrow checker prevents data races at compile time",
            "rust",
        ),
        (
            "Tokio async runtime for concurrent Rust applications",
            "rust",
        ),
        ("Rust ownership model eliminates null pointer bugs", "rust"),
        // Redundant: more Rust content (should be detected as not novel)
        ("Rust lifetime annotations ensure memory safety", "rust"),
        ("The Rust compiler catches borrowing errors early", "rust"),
        // Novel: completely different topic
        ("Kubernetes pod scheduling and resource allocation", "k8s"),
        ("Docker container networking with bridge mode", "docker"),
        // Back to Rust-adjacent (partially novel)
        ("WebAssembly compilation from Rust using wasm-pack", "wasm"),
        // Very novel: entirely new domain
        ("Impressionist painting techniques by Claude Monet", "art"),
        // Repeat the novel topic (should become redundant)
        ("Monet water lilies series at Musee de l'Orangerie", "art"),
        ("Impressionist use of light and color in landscapes", "art"),
        // Novel again after art saturation
        ("PostgreSQL query optimization with EXPLAIN ANALYZE", "sql"),
    ];

    println!("{:<60} {:>6} {:>10}", "Observation", "Sim", "Decision");
    println!("{}", "-".repeat(80));

    for (text, _tag) in &observations {
        let fingerprint = encode_text(text, &mut codebook);

        // Compare against current summary
        let current_summary = summary.finalize();
        let similarity = fingerprint.similarity(current_summary);

        // Determine novelty
        let is_novel = summary.count() == 0 || similarity < novelty_threshold;
        let decision = if summary.count() == 0 {
            "NOVEL (first)"
        } else if is_novel {
            "NOVEL"
        } else {
            "redundant"
        };

        // Truncate text for display
        let display_text: String = text.chars().take(55).collect();
        println!("{:<60} {:.4} {:>10}", display_text, similarity, decision);

        // Always add to summary (even redundant observations shape the running context)
        summary.add(&fingerprint);
    }

    // --- Demonstrate forgetting ---
    println!("\n\n=== Demonstrating Forgetting ===\n");
    println!("After flooding with SQL content, earlier Rust topics should be forgotten.\n");

    let mut fresh_summary = DecayingBundleAccumulator::new(0.8).unwrap();
    let mut cb2 = Codebook::new();

    // Add some Rust content
    let rust_texts = [
        "Rust ownership and borrowing rules",
        "Pattern matching with match expressions in Rust",
        "Rust trait objects and dynamic dispatch",
    ];
    for text in &rust_texts {
        fresh_summary.add(&encode_text(text, &mut cb2));
    }

    let rust_probe = encode_text("Rust memory safety guarantees", &mut cb2);
    let sql_probe = encode_text("SQL database indexing strategies", &mut cb2);

    let sim_rust_before = rust_probe.similarity(fresh_summary.finalize());
    let sim_sql_before = sql_probe.similarity(fresh_summary.finalize());
    println!("Before flooding:");
    println!("  Rust probe similarity: {:.4}", sim_rust_before);
    println!("  SQL probe similarity:  {:.4}", sim_sql_before);

    // Flood with SQL content
    let sql_texts = [
        "CREATE INDEX on large PostgreSQL tables",
        "SQL query execution plans and cost estimation",
        "Database normalization and denormalization tradeoffs",
        "PostgreSQL vacuum and autovacuum configuration",
        "Partitioning strategies for time-series data in SQL",
        "Connection pooling with PgBouncer for PostgreSQL",
        "Write-ahead logging and crash recovery in databases",
        "B-tree vs hash indexes for different query patterns",
        "SQL window functions for analytical queries",
        "Database sharding strategies for horizontal scaling",
    ];
    for text in &sql_texts {
        fresh_summary.add(&encode_text(text, &mut cb2));
    }

    let sim_rust_after = rust_probe.similarity(fresh_summary.finalize());
    let sim_sql_after = sql_probe.similarity(fresh_summary.finalize());
    println!("\nAfter flooding with 10 SQL observations:");
    println!(
        "  Rust probe similarity: {:.4} (was {:.4})",
        sim_rust_after, sim_rust_before
    );
    println!(
        "  SQL probe similarity:  {:.4} (was {:.4})",
        sim_sql_after, sim_sql_before
    );
    println!("\nThe Rust topic has been largely forgotten, SQL dominates the summary.");

    // --- Practical usage pattern ---
    println!("\n\n=== Practical Pattern: Heartbeat Report Filtering ===\n");

    let mut heartbeat_summary = DecayingBundleAccumulator::new(0.9).unwrap();
    let mut cb3 = Codebook::new();
    let report_threshold = 0.56;

    let heartbeat_findings = vec![
        ("CPU usage spike detected on prod-web-01", true),
        ("Memory usage normal across all nodes", false),
        ("CPU usage elevated on prod-web-01", false), // redundant with first
        ("New deployment detected: v2.3.1 rolled out", true),
        ("Disk usage approaching 80% on prod-db-01", true),
        ("CPU usage still high on prod-web-01", false), // redundant
        (
            "SSL certificate expiring in 7 days for api.example.com",
            true,
        ),
    ];

    println!("Simulating heartbeat findings (report_threshold={report_threshold}):");
    println!(
        "{:<55} {:>6} {:>8} {:>8}",
        "Finding", "Sim", "Novel?", "Report?"
    );
    println!("{}", "-".repeat(80));

    for (finding, expected_novel) in &heartbeat_findings {
        let fp = encode_document(
            DocumentEncodingInput {
                content: finding,
                tags: &[],
                path: "",
            },
            &mut cb3,
        );

        let sim = fp.similarity(heartbeat_summary.finalize());
        let is_novel = heartbeat_summary.count() == 0 || sim < report_threshold;

        let display: String = finding.chars().take(52).collect();
        println!(
            "{:<55} {:.4} {:>8} {:>8}",
            display,
            sim,
            if is_novel { "yes" } else { "no" },
            if is_novel { "SEND" } else { "skip" },
        );

        // Verify our expectations (roughly)
        if heartbeat_summary.count() > 0 && *expected_novel != is_novel {
            println!("    ^ (note: expectation mismatch, threshold tuning may be needed)");
        }

        heartbeat_summary.add(&fp);
    }

    println!("\nOnly novel findings are reported to the user, reducing notification fatigue.");
}

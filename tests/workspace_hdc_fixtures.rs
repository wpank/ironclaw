//! Fixture corpus harness for HDC encoder validation.
//!
//! Reads the fixture files under `tests/fixtures/hdc_memory/` and validates
//! that the HDC encoder produces similarity scores within the declared ranges
//! specified in `manifest.json`.
//!
//! Run with: cargo test --features hdc --test workspace_hdc_fixtures
//!
//! These are pure encoder tests -- no database, no async runtime needed.

#![cfg(feature = "hdc")]

use std::path::{Path, PathBuf};

use ironclaw_hdc::bundle::DecayingBundleAccumulator;
use ironclaw_hdc::codebook::Codebook;
use ironclaw_hdc::scan::top_k_scan;
use ironclaw_hdc::vector::HdcVector;
use ironclaw_hdc::{DocumentEncodingInput, encode_document};

// ─── Fixture Helpers ──────────────────────────────────────────────────────────

mod fixture_helpers {
    use std::path::{Path, PathBuf};

    use ironclaw_hdc::codebook::Codebook;
    use ironclaw_hdc::vector::HdcVector;
    use ironclaw_hdc::{DocumentEncodingInput, encode_document};

    /// A single entry parsed from a JSONL fixture file.
    #[derive(Debug)]
    pub struct FixtureEntry {
        pub id: String,
        pub content: String,
        pub tags: Vec<String>,
        pub path: String,
        /// Cycle number, present only in heartbeat fixtures.
        pub cycle: Option<u64>,
        /// Document type marker (query vs document), present in search fixtures.
        pub entry_type: Option<String>,
        /// Relevance flag, present in search fixtures.
        pub relevant: Option<bool>,
    }

    /// Returns the base directory for HDC memory fixtures.
    fn fixtures_dir() -> PathBuf {
        let manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set");
        Path::new(&manifest_dir)
            .join("tests")
            .join("fixtures")
            .join("hdc_memory")
    }

    /// Load and parse the manifest.json file.
    pub fn load_manifest() -> serde_json::Value {
        let path = fixtures_dir().join("manifest.json");
        let content = std::fs::read_to_string(&path).expect("failed to read manifest.json");
        serde_json::from_str(&content).expect("failed to parse manifest.json")
    }

    /// Load a JSONL file and parse each line into a FixtureEntry.
    pub fn load_jsonl(relative_path: &str) -> Vec<FixtureEntry> {
        let path = fixtures_dir().join(relative_path);
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read fixture file {}: {e}", path.display()));
        content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let v: serde_json::Value = serde_json::from_str(line)
                    .unwrap_or_else(|e| panic!("failed to parse JSONL line: {e}\nline: {line}"));
                let tags = v
                    .get("tags")
                    .and_then(|t| t.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|t| t.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                FixtureEntry {
                    id: v
                        .get("id")
                        .and_then(|i| i.as_str())
                        .unwrap_or("")
                        .to_string(),
                    content: v
                        .get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("")
                        .to_string(),
                    tags,
                    path: v
                        .get("path")
                        .and_then(|p| p.as_str())
                        .unwrap_or("")
                        .to_string(),
                    cycle: v.get("cycle").and_then(|c| c.as_u64()),
                    entry_type: v.get("type").and_then(|t| t.as_str()).map(String::from),
                    relevant: v.get("relevant").and_then(|r| r.as_bool()),
                }
            })
            .collect()
    }

    /// Encode a fixture entry using the full document encoder.
    pub fn encode_entry(entry: &FixtureEntry, codebook: &mut Codebook) -> HdcVector {
        let tag_refs: Vec<&str> = entry.tags.iter().map(|s| s.as_str()).collect();
        encode_document(
            DocumentEncodingInput {
                content: &entry.content,
                tags: &tag_refs,
                path: &entry.path,
            },
            codebook,
        )
    }

    /// Resolve a fixture file path relative to the fixtures directory.
    pub fn fixture_path(relative: &str) -> PathBuf {
        fixtures_dir().join(relative)
    }
}

// ─── Dedup Fixtures ───────────────────────────────────────────────────────────

mod dedup_fixtures {
    use super::fixture_helpers::{encode_entry, load_jsonl, load_manifest};
    use ironclaw_hdc::codebook::Codebook;

    /// Default dedup thresholds (must match DedupConfig::default_config()).
    const SIMILAR_THRESHOLD: f64 = 0.78;
    const DUPLICATE_THRESHOLD: f64 = 0.92;

    /// Run a single dedup fixture scenario: load the JSONL, encode both entries,
    /// compute similarity, assert it falls within the manifest's declared range,
    /// and verify the expected_decision label is consistent with default thresholds.
    fn run_dedup_scenario(scenario_key: &str) {
        let manifest = load_manifest();
        let scenario = &manifest["dedup"][scenario_key];
        let file = scenario["file"]
            .as_str()
            .unwrap_or_else(|| panic!("missing 'file' for dedup scenario '{scenario_key}'"));
        let expected_decision = scenario["expected_decision"].as_str().unwrap_or_else(|| {
            panic!("missing 'expected_decision' for dedup scenario '{scenario_key}'")
        });
        let range = &scenario["expected_similarity_range"];
        let min = range["min"]
            .as_f64()
            .unwrap_or_else(|| panic!("missing 'min' for dedup scenario '{scenario_key}'"));
        let max = range["max"]
            .as_f64()
            .unwrap_or_else(|| panic!("missing 'max' for dedup scenario '{scenario_key}'"));

        // Verify the declared range is consistent with the label and default thresholds.
        match expected_decision {
            "duplicate" => {
                assert!(
                    min >= DUPLICATE_THRESHOLD,
                    "dedup scenario '{scenario_key}': label is 'duplicate' but range min \
                     ({min}) < duplicate threshold ({DUPLICATE_THRESHOLD})"
                );
            }
            "similar" => {
                assert!(
                    min >= SIMILAR_THRESHOLD,
                    "dedup scenario '{scenario_key}': label is 'similar' but range min \
                     ({min}) < similar threshold ({SIMILAR_THRESHOLD})"
                );
                assert!(
                    max < DUPLICATE_THRESHOLD,
                    "dedup scenario '{scenario_key}': label is 'similar' but range max \
                     ({max}) >= duplicate threshold ({DUPLICATE_THRESHOLD})"
                );
            }
            "unique" => {
                assert!(
                    max <= SIMILAR_THRESHOLD,
                    "dedup scenario '{scenario_key}': label is 'unique' but range max \
                     ({max}) > similar threshold ({SIMILAR_THRESHOLD})"
                );
            }
            other => panic!("dedup scenario '{scenario_key}': unknown decision '{other}'"),
        }

        let entries = load_jsonl(file);
        assert!(
            entries.len() >= 2,
            "dedup scenario '{scenario_key}' needs at least 2 entries, found {}",
            entries.len()
        );

        let mut codebook = Codebook::new();
        let fp1 = encode_entry(&entries[0], &mut codebook);
        let fp2 = encode_entry(&entries[1], &mut codebook);
        let similarity = fp1.similarity(fp2);

        assert!(
            similarity >= min && similarity <= max,
            "dedup scenario '{scenario_key}': similarity {similarity:.6} is outside \
             expected range [{min}, {max}] (entry_a='{}', entry_b='{}')",
            entries[0].id,
            entries[1].id
        );

        // Verify the actual computed similarity matches the declared decision.
        let actual_decision = if similarity >= DUPLICATE_THRESHOLD {
            "duplicate"
        } else if similarity >= SIMILAR_THRESHOLD {
            "similar"
        } else {
            "unique"
        };
        assert_eq!(
            actual_decision, expected_decision,
            "dedup scenario '{scenario_key}': actual similarity {similarity:.6} yields \
             decision '{actual_decision}', but manifest declares '{expected_decision}'"
        );
    }

    #[test]
    fn fixture_dedup_exact_duplicate_similarity_range() {
        run_dedup_scenario("exact-duplicate-different-path");
    }

    #[test]
    fn fixture_dedup_reformatted_similarity_range() {
        run_dedup_scenario("reformatted-duplicate");
    }

    #[test]
    fn fixture_dedup_paraphrase_similarity_range() {
        run_dedup_scenario("paraphrase-duplicate");
    }

    #[test]
    fn fixture_dedup_added_paragraph_similarity_range() {
        run_dedup_scenario("added-paragraph");
    }

    #[test]
    fn fixture_dedup_same_topic_different_facts_is_unique() {
        run_dedup_scenario("same-topic-different-facts");
    }

    #[test]
    fn fixture_dedup_same_tags_different_content_is_unique() {
        run_dedup_scenario("same-tags-different-content");
    }

    #[test]
    fn fixture_dedup_same_path_different_content_is_unique() {
        run_dedup_scenario("same-path-pattern-different-content");
    }

    #[test]
    fn fixture_dedup_content_subset_is_similar() {
        run_dedup_scenario("content-subset");
    }

    #[test]
    fn fixture_dedup_translated_is_similar() {
        run_dedup_scenario("translated-content");
    }
}

// ─── Heartbeat Fixtures ───────────────────────────────────────────────────────

mod heartbeat_fixtures {
    use super::fixture_helpers::{encode_entry, load_jsonl, load_manifest};
    use ironclaw_hdc::bundle::DecayingBundleAccumulator;
    use ironclaw_hdc::codebook::Codebook;

    /// Novelty threshold: if the new observation's similarity to the
    /// accumulated bundle exceeds this, it is "repeated" (not novel).
    const NOVELTY_THRESHOLD: f64 = 0.80;

    /// Feed heartbeat observations through a DecayingBundleAccumulator and
    /// check novelty at each cycle against the manifest expectations.
    fn run_heartbeat_scenario(scenario_key: &str) {
        let manifest = load_manifest();
        let scenario = &manifest["heartbeat"][scenario_key];
        let file = scenario["file"]
            .as_str()
            .unwrap_or_else(|| panic!("missing 'file' for heartbeat scenario '{scenario_key}'"));
        let expected_cycles = scenario["expected_novelty_at_cycle"]
            .as_array()
            .unwrap_or_else(|| {
                panic!(
                    "missing 'expected_novelty_at_cycle' for heartbeat scenario '{scenario_key}'"
                )
            });

        let entries = load_jsonl(file);
        let mut codebook = Codebook::new();
        let mut accumulator =
            DecayingBundleAccumulator::new(0.9).expect("decay factor 0.9 should be valid");

        // Build a lookup from cycle number to expected novelty.
        let mut expectations: std::collections::HashMap<u64, bool> =
            std::collections::HashMap::new();
        for expectation in expected_cycles {
            let cycle = expectation["cycle"]
                .as_u64()
                .expect("cycle must be a number");
            let novel = expectation["novel"]
                .as_bool()
                .expect("novel must be a boolean");
            expectations.insert(cycle, novel);
        }

        for entry in &entries {
            let cycle = entry
                .cycle
                .unwrap_or_else(|| panic!("heartbeat entry '{}' missing cycle field", entry.id));
            let observation_hv = encode_entry(entry, &mut codebook);

            // Determine novelty: compare against the accumulated state.
            let is_novel = if accumulator.count() == 0 {
                // First observation is always novel (nothing to compare against).
                true
            } else {
                let accumulated_hv = accumulator.finalize();
                let sim = observation_hv.similarity(accumulated_hv);
                sim < NOVELTY_THRESHOLD
            };

            // Feed the observation into the accumulator regardless of novelty.
            accumulator.add(&observation_hv);

            // Check if this cycle has an expectation in the manifest.
            if let Some(&expected_novel) = expectations.get(&cycle) {
                assert_eq!(
                    is_novel,
                    expected_novel,
                    "heartbeat scenario '{scenario_key}', cycle {cycle} (entry '{}'): \
                     expected novel={expected_novel}, got novel={is_novel} \
                     (similarity to accumulated state {} threshold {NOVELTY_THRESHOLD})",
                    entry.id,
                    if is_novel { "below" } else { "at or above" }
                );
            }
        }
    }

    #[test]
    fn fixture_heartbeat_repeated_observation_converges() {
        run_heartbeat_scenario("repeated-observation");
    }

    #[test]
    fn fixture_heartbeat_recurring_after_gap() {
        run_heartbeat_scenario("recurring-after-gap");
    }

    #[test]
    fn fixture_heartbeat_similar_different_metric_is_novel() {
        run_heartbeat_scenario("similar-but-different-metric");
    }

    #[test]
    fn fixture_heartbeat_completely_novel_all_novel() {
        run_heartbeat_scenario("completely-novel");
    }
}

// ─── Search Fixtures ──────────────────────────────────────────────────────────

mod search_fixtures {
    use super::fixture_helpers::{encode_entry, load_jsonl, load_manifest};
    use ironclaw_hdc::codebook::Codebook;
    use ironclaw_hdc::scan::top_k_scan;
    use ironclaw_hdc::vector::HdcVector;
    use ironclaw_hdc::{DocumentEncodingInput, encode_document};

    /// Run a search fixture scenario: encode all documents and the query, then
    /// verify via top_k_scan that relevant documents rank above irrelevant ones.
    fn run_search_scenario(scenario_key: &str) {
        let manifest = load_manifest();
        let scenario = &manifest["search"][scenario_key];
        let file = scenario["file"]
            .as_str()
            .unwrap_or_else(|| panic!("missing 'file' for search scenario '{scenario_key}'"));
        let query_text = scenario["query"]
            .as_str()
            .unwrap_or_else(|| panic!("missing 'query' for search scenario '{scenario_key}'"));
        let relevant_ids: Vec<String> = scenario["relevant_document_ids"]
            .as_array()
            .unwrap_or_else(|| {
                panic!("missing 'relevant_document_ids' for search scenario '{scenario_key}'")
            })
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        let irrelevant_ids: Vec<String> = scenario["irrelevant_document_ids"]
            .as_array()
            .unwrap_or_else(|| {
                panic!("missing 'irrelevant_document_ids' for search scenario '{scenario_key}'")
            })
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let entries = load_jsonl(file);

        let mut codebook = Codebook::new();

        // Encode the query from the manifest text (content-only encoding).
        let query_hv = encode_document(
            DocumentEncodingInput {
                content: query_text,
                tags: &[],
                path: "",
            },
            &mut codebook,
        );

        // Encode all document entries (skip the query entry from the JSONL).
        let mut candidates: Vec<(String, HdcVector)> = Vec::new();
        for entry in &entries {
            let is_query = entry
                .entry_type
                .as_deref()
                .map(|t| t == "query")
                .unwrap_or(false);
            if is_query {
                continue;
            }
            let hv = encode_entry(entry, &mut codebook);
            candidates.push((entry.id.clone(), hv));
        }

        let total_docs = candidates.len();
        let results = top_k_scan(query_hv, &candidates, total_docs);

        // Compute the best (lowest) rank for any relevant document and the
        // worst (highest) rank for any irrelevant document.
        let mut best_relevant_rank: Option<usize> = None;
        let mut worst_relevant_rank: Option<usize> = None;
        let mut best_irrelevant_rank: Option<usize> = None;

        for (rank, result) in results.iter().enumerate() {
            if relevant_ids.contains(&result.id) {
                best_relevant_rank = Some(best_relevant_rank.map_or(rank, |r: usize| r.min(rank)));
                worst_relevant_rank =
                    Some(worst_relevant_rank.map_or(rank, |r: usize| r.max(rank)));
            }
            if irrelevant_ids.contains(&result.id) {
                best_irrelevant_rank =
                    Some(best_irrelevant_rank.map_or(rank, |r: usize| r.min(rank)));
            }
        }

        let worst_relevant = worst_relevant_rank.unwrap_or_else(|| {
            panic!("search scenario '{scenario_key}': no relevant documents found in results")
        });
        let best_irrelevant = best_irrelevant_rank.unwrap_or_else(|| {
            panic!("search scenario '{scenario_key}': no irrelevant documents found in results")
        });

        // The highest-ranked relevant document should rank above the
        // highest-ranked irrelevant document. That is, at least one relevant
        // doc must appear before all irrelevant docs, or more generally the
        // best relevant rank must be strictly better (lower index) than the
        // best irrelevant rank.
        let best_relevant = best_relevant_rank.expect("at least one relevant doc must be present");
        assert!(
            best_relevant < best_irrelevant,
            "search scenario '{scenario_key}': best relevant document (rank {best_relevant}) \
             should rank above best irrelevant document (rank {best_irrelevant}). \
             Rankings: {rankings}",
            rankings = results
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let label = if relevant_ids.contains(&r.id) {
                        "RELEVANT"
                    } else if irrelevant_ids.contains(&r.id) {
                        "IRRELEVANT"
                    } else {
                        "OTHER"
                    };
                    format!("  #{i}: {} (sim={:.4}, {label})", r.id, r.similarity)
                })
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Additionally, verify that the worst-ranked relevant doc is still
        // above the best-ranked irrelevant doc -- i.e., all relevant docs
        // rank above all irrelevant docs.
        assert!(
            worst_relevant < best_irrelevant,
            "search scenario '{scenario_key}': worst relevant document (rank {worst_relevant}) \
             should still rank above best irrelevant document (rank {best_irrelevant}). \
             Rankings: {rankings}",
            rankings = results
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    let label = if relevant_ids.contains(&r.id) {
                        "RELEVANT"
                    } else if irrelevant_ids.contains(&r.id) {
                        "IRRELEVANT"
                    } else {
                        "OTHER"
                    };
                    format!("  #{i}: {} (sim={:.4}, {label})", r.id, r.similarity)
                })
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    #[test]
    fn fixture_search_structural_query_ranks_relevant() {
        run_search_scenario("structural-query");
    }

    #[test]
    fn fixture_search_tag_heavy_query_ranks_relevant() {
        run_search_scenario("tag-heavy-query");
    }

    #[test]
    fn fixture_search_path_oriented_query_ranks_relevant() {
        run_search_scenario("path-oriented-query");
    }

    #[test]
    fn fixture_search_semantic_only_query_ranks_relevant() {
        run_search_scenario("semantic-only-query");
    }
}

// ─── Manifest Consistency ─────────────────────────────────────────────────────

mod manifest_consistency {
    use super::fixture_helpers::{fixture_path, load_jsonl, load_manifest};

    #[test]
    fn fixture_manifest_all_files_exist() {
        let manifest = load_manifest();

        // Check all dedup fixture files.
        if let Some(dedup) = manifest.get("dedup").and_then(|d| d.as_object()) {
            for (key, scenario) in dedup {
                let file = scenario["file"]
                    .as_str()
                    .unwrap_or_else(|| panic!("dedup scenario '{key}' missing 'file' field"));
                let path = fixture_path(file);
                assert!(
                    path.exists(),
                    "dedup scenario '{key}' references non-existent file: {}",
                    path.display()
                );
            }
        }

        // Check all heartbeat fixture files.
        if let Some(heartbeat) = manifest.get("heartbeat").and_then(|h| h.as_object()) {
            for (key, scenario) in heartbeat {
                let file = scenario["file"]
                    .as_str()
                    .unwrap_or_else(|| panic!("heartbeat scenario '{key}' missing 'file' field"));
                let path = fixture_path(file);
                assert!(
                    path.exists(),
                    "heartbeat scenario '{key}' references non-existent file: {}",
                    path.display()
                );
            }
        }

        // Check all search fixture files.
        if let Some(search) = manifest.get("search").and_then(|s| s.as_object()) {
            for (key, scenario) in search {
                let file = scenario["file"]
                    .as_str()
                    .unwrap_or_else(|| panic!("search scenario '{key}' missing 'file' field"));
                let path = fixture_path(file);
                assert!(
                    path.exists(),
                    "search scenario '{key}' references non-existent file: {}",
                    path.display()
                );
            }
        }
    }

    #[test]
    fn fixture_manifest_jsonl_parseable() {
        let manifest = load_manifest();

        let mut all_files: Vec<(String, String)> = Vec::new();

        if let Some(dedup) = manifest.get("dedup").and_then(|d| d.as_object()) {
            for (key, scenario) in dedup {
                if let Some(file) = scenario["file"].as_str() {
                    all_files.push((format!("dedup/{key}"), file.to_string()));
                }
            }
        }
        if let Some(heartbeat) = manifest.get("heartbeat").and_then(|h| h.as_object()) {
            for (key, scenario) in heartbeat {
                if let Some(file) = scenario["file"].as_str() {
                    all_files.push((format!("heartbeat/{key}"), file.to_string()));
                }
            }
        }
        if let Some(search) = manifest.get("search").and_then(|s| s.as_object()) {
            for (key, scenario) in search {
                if let Some(file) = scenario["file"].as_str() {
                    all_files.push((format!("search/{key}"), file.to_string()));
                }
            }
        }

        assert!(
            !all_files.is_empty(),
            "manifest contains no fixture file references"
        );

        for (scenario_label, file) in &all_files {
            // load_jsonl will panic with a clear message if parsing fails,
            // which is exactly the behavior we want for this validation test.
            let entries = load_jsonl(file);
            assert!(
                !entries.is_empty(),
                "fixture file for '{scenario_label}' ({file}) parsed to zero entries"
            );
        }
    }

    #[test]
    fn fixture_manifest_similarity_ranges_valid() {
        let manifest = load_manifest();

        // Default dedup thresholds (must match DedupConfig::default_config()).
        const SIMILAR_THRESHOLD: f64 = 0.78;
        const DUPLICATE_THRESHOLD: f64 = 0.92;

        if let Some(dedup) = manifest.get("dedup").and_then(|d| d.as_object()) {
            for (key, scenario) in dedup {
                let range = &scenario["expected_similarity_range"];
                let min = range["min"].as_f64().unwrap_or_else(|| {
                    panic!("dedup scenario '{key}': missing or non-numeric 'min'")
                });
                let max = range["max"].as_f64().unwrap_or_else(|| {
                    panic!("dedup scenario '{key}': missing or non-numeric 'max'")
                });

                assert!(
                    min < max,
                    "dedup scenario '{key}': min ({min}) must be less than max ({max})"
                );
                assert!(
                    min >= 0.0 && min <= 1.0,
                    "dedup scenario '{key}': min ({min}) must be in [0, 1]"
                );
                assert!(
                    max >= 0.0 && max <= 1.0,
                    "dedup scenario '{key}': max ({max}) must be in [0, 1]"
                );

                // Verify the declared range is fully within the band implied by the label.
                let decision = scenario["expected_decision"].as_str().unwrap_or_else(|| {
                    panic!("dedup scenario '{key}': missing 'expected_decision'")
                });
                match decision {
                    "duplicate" => {
                        assert!(
                            min >= DUPLICATE_THRESHOLD,
                            "dedup scenario '{key}': label 'duplicate' requires range min \
                             ({min}) >= {DUPLICATE_THRESHOLD}"
                        );
                    }
                    "similar" => {
                        assert!(
                            min >= SIMILAR_THRESHOLD,
                            "dedup scenario '{key}': label 'similar' requires range min \
                             ({min}) >= {SIMILAR_THRESHOLD}"
                        );
                        assert!(
                            max < DUPLICATE_THRESHOLD,
                            "dedup scenario '{key}': label 'similar' requires range max \
                             ({max}) < {DUPLICATE_THRESHOLD}"
                        );
                    }
                    "unique" => {
                        assert!(
                            max <= SIMILAR_THRESHOLD,
                            "dedup scenario '{key}': label 'unique' requires range max \
                             ({max}) <= {SIMILAR_THRESHOLD}"
                        );
                    }
                    other => panic!("dedup scenario '{key}': unknown decision '{other}'"),
                }
            }
        }
    }
}

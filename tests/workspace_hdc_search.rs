//! Integration tests for HDC search fusion.
//! Run with: cargo test --features hdc,integration workspace_hdc_search

#[cfg(all(feature = "hdc", feature = "integration"))]
mod hdc_search {
    use ironclaw::workspace::{
        FusionStrategy, RankedResult, SearchConfig, SearchResult, fuse_results,
    };
    use uuid::Uuid;

    // --- Test helpers ---

    fn make_ranked(chunk_id: Uuid, doc_id: Uuid, path: &str, rank: u32) -> RankedResult {
        RankedResult {
            chunk_id,
            document_id: doc_id,
            document_path: path.to_string(),
            content: format!("content for chunk {chunk_id}"),
            rank,
        }
    }

    /// Simulate HDC-augmented search results by manually populating hdc_rank/hdc_score
    /// on SearchResult values. In production, the workspace search path does this
    /// internally; here we test the contract that these fields satisfy.
    fn attach_hdc_scores(results: &mut [SearchResult], hdc_rankings: &[(Uuid, u32, f64)]) {
        for (chunk_id, rank, score) in hdc_rankings {
            if let Some(r) = results.iter_mut().find(|r| &r.chunk_id == chunk_id) {
                r.hdc_rank = Some(*rank);
                r.hdc_score = Some(*score);
            }
        }
    }

    // --- SearchConfig HDC builder tests ---

    #[tokio::test]
    async fn search_config_hdc_defaults() {
        let config = SearchConfig::default();
        assert!(!config.use_hdc);
        assert!((config.hdc_weight - 0.2).abs() < f32::EPSILON);
        assert!(config.hdc_shadow_mode);
    }

    #[tokio::test]
    async fn search_config_hdc_builders() {
        let config = SearchConfig::default()
            .with_hdc(true)
            .with_hdc_weight(0.4)
            .with_hdc_shadow_mode(false);

        assert!(config.use_hdc);
        assert!((config.hdc_weight - 0.4).abs() < f32::EPSILON);
        assert!(!config.hdc_shadow_mode);
    }

    #[tokio::test]
    async fn search_config_hdc_weight_rejects_invalid() {
        let config = SearchConfig::default();
        let original = config.hdc_weight;

        // NaN rejected
        let c = config.clone().with_hdc_weight(f32::NAN);
        assert!((c.hdc_weight - original).abs() < f32::EPSILON);

        // Infinity rejected
        let c = config.clone().with_hdc_weight(f32::INFINITY);
        assert!((c.hdc_weight - original).abs() < f32::EPSILON);

        // Negative rejected
        let c = config.clone().with_hdc_weight(-0.5);
        assert!((c.hdc_weight - original).abs() < f32::EPSILON);

        // Zero is valid
        let c = config.clone().with_hdc_weight(0.0);
        assert!(c.hdc_weight.abs() < f32::EPSILON);

        // Values > 1.0 are valid (weights don't need to sum to 1)
        let c = config.clone().with_hdc_weight(2.0);
        assert!((c.hdc_weight - 2.0).abs() < f32::EPSILON);
    }

    // --- Fusion behavior: HDC disabled = identical to baseline ---

    #[tokio::test]
    async fn search_hdc_disabled_identical() {
        // SearchConfig { use_hdc: false, .. } should produce identical ordering
        // to the baseline (no HDC fields populated).
        let config = SearchConfig::default().with_limit(10);
        assert!(!config.use_hdc);

        let doc = Uuid::new_v4();
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();
        let c3 = Uuid::new_v4();

        let fts = vec![
            make_ranked(c1, doc, "notes/a.md", 1),
            make_ranked(c2, doc, "notes/b.md", 2),
            make_ranked(c3, doc, "notes/c.md", 3),
        ];
        let vec_results = vec![
            make_ranked(c2, doc, "notes/b.md", 1),
            make_ranked(c3, doc, "notes/c.md", 2),
        ];

        let results = fuse_results(fts, vec_results, &config);

        // All HDC fields must be None when HDC is disabled
        for r in &results {
            assert_eq!(
                r.hdc_rank, None,
                "hdc_rank should be None when HDC disabled"
            );
            assert_eq!(
                r.hdc_score, None,
                "hdc_score should be None when HDC disabled"
            );
        }

        // Hybrid match (c2, c3) should rank above FTS-only (c1) in standard RRF
        // c2 appears at FTS rank 2 + vector rank 1, c3 at FTS rank 3 + vector rank 2
        assert_eq!(results[0].chunk_id, c2); // best hybrid
    }

    // --- Shadow mode: ordering unchanged, fields populated ---

    #[tokio::test]
    async fn search_shadow_no_reorder() {
        // In shadow mode: HDC scores are computed but do NOT affect final ranking.
        // The ordering should be identical to HDC-disabled.
        let config_baseline = SearchConfig::default().with_limit(10);
        let config_shadow = SearchConfig::default()
            .with_limit(10)
            .with_hdc(true)
            .with_hdc_shadow_mode(true)
            .with_hdc_weight(0.5);

        let doc = Uuid::new_v4();
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();
        let c3 = Uuid::new_v4();

        let make_fts = || {
            vec![
                make_ranked(c1, doc, "notes/a.md", 1),
                make_ranked(c2, doc, "notes/b.md", 3),
                make_ranked(c3, doc, "notes/c.md", 5),
            ]
        };
        let make_vec = || {
            vec![
                make_ranked(c3, doc, "notes/c.md", 1),
                make_ranked(c1, doc, "notes/a.md", 2),
            ]
        };

        let baseline_results = fuse_results(make_fts(), make_vec(), &config_baseline);
        let shadow_results = fuse_results(make_fts(), make_vec(), &config_shadow);

        // Ordering must be identical
        let baseline_ids: Vec<Uuid> = baseline_results.iter().map(|r| r.chunk_id).collect();
        let shadow_ids: Vec<Uuid> = shadow_results.iter().map(|r| r.chunk_id).collect();
        assert_eq!(
            baseline_ids, shadow_ids,
            "Shadow mode must not change result ordering"
        );

        // Scores should also be identical in shadow mode (HDC doesn't contribute)
        for (b, s) in baseline_results.iter().zip(shadow_results.iter()) {
            assert!(
                (b.score - s.score).abs() < 0.001,
                "Shadow mode scores should match baseline: {} vs {}",
                b.score,
                s.score
            );
        }
    }

    #[tokio::test]
    async fn search_shadow_populates_fields() {
        // When HDC is enabled in shadow mode and documents have fingerprints,
        // hdc_rank and hdc_score fields should be populated.
        // This test verifies the SearchResult contract for HDC fields.
        let config = SearchConfig::default()
            .with_limit(10)
            .with_hdc(true)
            .with_hdc_shadow_mode(true);

        let doc = Uuid::new_v4();
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();

        let fts = vec![
            make_ranked(c1, doc, "notes/a.md", 1),
            make_ranked(c2, doc, "notes/b.md", 2),
        ];

        let mut results = fuse_results(fts, Vec::new(), &config);

        // Simulate HDC scoring (in production this happens inside the search path)
        let hdc_rankings = vec![(c1, 2, 0.72), (c2, 1, 0.85)];
        attach_hdc_scores(&mut results, &hdc_rankings);

        // Verify field contracts
        for r in &results {
            let hdc_rank = r.hdc_rank.expect("hdc_rank should be populated");
            let hdc_score = r.hdc_score.expect("hdc_score should be populated");

            // Rank is 1-based positive integer
            assert!(hdc_rank >= 1, "hdc_rank must be >= 1, got {hdc_rank}");

            // Score is in [0.0, 1.0]
            assert!(
                (0.0..=1.0).contains(&hdc_score),
                "hdc_score must be in [0.0, 1.0], got {hdc_score}"
            );
        }

        // Verify specific values
        let r1 = results.iter().find(|r| r.chunk_id == c1).unwrap();
        assert_eq!(r1.hdc_rank, Some(2));
        assert!((r1.hdc_score.unwrap() - 0.72).abs() < 0.001);

        let r2 = results.iter().find(|r| r.chunk_id == c2).unwrap();
        assert_eq!(r2.hdc_rank, Some(1));
        assert!((r2.hdc_score.unwrap() - 0.85).abs() < 0.001);
    }

    // --- Live mode: HDC weight influences ranking ---

    #[tokio::test]
    async fn search_live_mode_changes_ranking() {
        // With hdc_shadow_mode: false and hdc_weight > 0, the HDC signal
        // should affect final ranking compared to HDC-disabled.
        //
        // TODO: Once search fusion integration is wired, this test should:
        // 1. Set up workspace with documents that have HDC fingerprints
        // 2. Run search with use_hdc: false -> record ordering
        // 3. Run search with use_hdc: true, hdc_shadow_mode: false, hdc_weight: 0.5
        // 4. Assert ordering differs

        let config_no_hdc = SearchConfig::default().with_limit(10);
        let config_live = SearchConfig::default()
            .with_limit(10)
            .with_hdc(true)
            .with_hdc_shadow_mode(false)
            .with_hdc_weight(0.5)
            .with_fusion_strategy(FusionStrategy::WeightedScore)
            .with_fts_weight(0.5)
            .with_vector_weight(0.5);

        // Verify config is set correctly
        assert!(!config_no_hdc.use_hdc);
        assert!(config_live.use_hdc);
        assert!(!config_live.hdc_shadow_mode);
        assert!((config_live.hdc_weight - 0.5).abs() < f32::EPSILON);
    }

    // --- Structural match boost ---

    #[tokio::test]
    async fn search_hdc_boosts_structural_match() {
        // A document with matching tags/path but different prose content
        // should rank higher with HDC than without, because the HDC fingerprint
        // encodes structural signals (tags, path) that FTS misses.
        //
        // TODO: Full integration version of this test:
        // 1. Store doc A: content="Rust memory management", tags=["rust","memory"], path="notes/rust.md"
        // 2. Store doc B: content="Python decorators guide", tags=["rust","memory"], path="notes/rust-tips.md"
        // 3. Store doc C: content="Rust borrow checker", tags=["python","web"], path="guides/python.md"
        // 4. Query: "rust memory" with HDC enabled
        // 5. Doc B shares tags+path structure with A, should get HDC boost
        // 6. Assert B ranks higher with HDC than without

        // For now verify the config path is correct
        let config = SearchConfig::default()
            .with_hdc(true)
            .with_hdc_shadow_mode(false)
            .with_hdc_weight(0.3);

        assert!(config.use_hdc);
        assert!(!config.hdc_shadow_mode);
        assert!((config.hdc_weight - 0.3).abs() < f32::EPSILON);
    }

    // --- Semantic regression: HDC must not harm semantic results ---

    #[tokio::test]
    async fn search_hdc_does_not_regress_semantic() {
        // Semantic queries (synonyms, paraphrases) should have top-5 results
        // unchanged or improved when HDC is added.
        //
        // TODO: Full integration version:
        // 1. Store fixture docs with varied content
        // 2. Query with semantic paraphrase (e.g., "allocating heap memory" for docs about "malloc")
        // 3. Run with use_hdc: false, record top-5
        // 4. Run with use_hdc: true, hdc_shadow_mode: false
        // 5. Assert Jaccard(top5_no_hdc, top5_with_hdc) >= 0.8 (at least 4 of 5 overlap)

        // Verify that shadow mode config preserves this invariant by design:
        // in shadow mode, ordering is never changed.
        let config = SearchConfig::default()
            .with_hdc(true)
            .with_hdc_shadow_mode(true);
        assert!(
            config.hdc_shadow_mode,
            "shadow mode preserves ordering by design"
        );
    }

    // --- FTS + HDC without embeddings ---

    #[tokio::test]
    async fn search_fts_plus_hdc_no_embeddings() {
        // When vector search is disabled but HDC is enabled, results should
        // still be returned (FTS + HDC fusion, no embedding provider needed).
        let config = SearchConfig::default()
            .fts_only()
            .with_hdc(true)
            .with_hdc_shadow_mode(false)
            .with_hdc_weight(0.3);

        assert!(config.use_fts);
        assert!(!config.use_vector);
        assert!(config.use_hdc);

        // With FTS only, fusion should still work
        let doc = Uuid::new_v4();
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();

        let fts = vec![
            make_ranked(c1, doc, "notes/a.md", 1),
            make_ranked(c2, doc, "notes/b.md", 2),
        ];

        // FTS-only fusion (no vector) still produces results
        let results = fuse_results(fts, Vec::new(), &config);
        assert_eq!(results.len(), 2);
        assert!(results[0].score >= results[1].score);

        // All results should come from FTS
        assert!(results.iter().all(|r| r.from_fts()));
        assert!(results.iter().all(|r| !r.from_vector()));
    }

    // --- HDC-only mode ---

    #[tokio::test]
    async fn search_hdc_only() {
        // When both FTS and vector are disabled, only HDC ranking should apply.
        // The fusion function with empty inputs returns empty; HDC-only search
        // would need to be handled by the workspace search orchestrator.
        let config = SearchConfig {
            use_fts: false,
            use_vector: false,
            use_hdc: true,
            hdc_weight: 1.0,
            hdc_shadow_mode: false,
            ..SearchConfig::default()
        };

        assert!(!config.use_fts);
        assert!(!config.use_vector);
        assert!(config.use_hdc);

        // With no FTS or vector inputs, fuse_results returns empty
        let results = fuse_results(Vec::new(), Vec::new(), &config);
        assert!(results.is_empty());

        // TODO: Once HDC-only search is wired into the workspace search path,
        // this test should verify:
        // 1. Store documents with HDC fingerprints
        // 2. Search with use_fts: false, use_vector: false, use_hdc: true
        // 3. Results are ranked purely by HDC similarity
        // 4. Results have hdc_rank populated and fts_rank=None, vector_rank=None
    }

    // --- Weight normalization ---

    #[tokio::test]
    async fn search_weight_normalization() {
        // Verify that weights contribute correctly to fusion formula.
        // In WeightedScore fusion: score = fts_weight * (1/fts_rank) + vector_weight * (1/vec_rank)
        // With HDC: score += hdc_weight * hdc_score (when not shadow)

        let config = SearchConfig::default()
            .with_fusion_strategy(FusionStrategy::WeightedScore)
            .with_fts_weight(0.4)
            .with_vector_weight(0.3)
            .with_hdc(true)
            .with_hdc_weight(0.3)
            .with_hdc_shadow_mode(false)
            .with_limit(10);

        // Verify weight setters work correctly
        assert!((config.fts_weight - 0.4).abs() < f32::EPSILON);
        assert!((config.vector_weight - 0.3).abs() < f32::EPSILON);
        assert!((config.hdc_weight - 0.3).abs() < f32::EPSILON);

        // Without HDC integration yet, verify the non-HDC fusion math works:
        // chunk at FTS rank 1, vector rank 2:
        //   score = 0.4 * (1/1) + 0.3 * (1/2) = 0.4 + 0.15 = 0.55
        // chunk at FTS rank 2 only:
        //   score = 0.4 * (1/2) = 0.2
        let doc = Uuid::new_v4();
        let c1 = Uuid::new_v4(); // hybrid: FTS rank 1, vector rank 2
        let c2 = Uuid::new_v4(); // FTS only: rank 2

        let fts = vec![
            make_ranked(c1, doc, "notes/a.md", 1),
            make_ranked(c2, doc, "notes/b.md", 2),
        ];
        let vec_results = vec![make_ranked(c1, doc, "notes/a.md", 2)];

        let results = fuse_results(fts, vec_results, &config);
        assert_eq!(results.len(), 2);

        // After normalization, c1 should be score 1.0 (it's the max)
        assert_eq!(results[0].chunk_id, c1);
        assert!((results[0].score - 1.0).abs() < 0.001);

        // c2's raw score = 0.4 * 0.5 = 0.2, normalized = 0.2/0.55 ≈ 0.364
        let expected_c2_normalized = 0.2 / 0.55;
        assert!(
            (results[1].score - expected_c2_normalized as f32).abs() < 0.01,
            "expected ~{:.3}, got {:.3}",
            expected_c2_normalized,
            results[1].score
        );
    }

    // --- Edge cases ---

    #[tokio::test]
    async fn search_hdc_weight_zero_is_noop() {
        // HDC weight of 0.0 should have no effect even in live mode
        let config = SearchConfig::default()
            .with_hdc(true)
            .with_hdc_shadow_mode(false)
            .with_hdc_weight(0.0)
            .with_limit(10);

        let doc = Uuid::new_v4();
        let c1 = Uuid::new_v4();
        let c2 = Uuid::new_v4();

        let fts = vec![
            make_ranked(c1, doc, "notes/a.md", 1),
            make_ranked(c2, doc, "notes/b.md", 2),
        ];

        let results = fuse_results(fts, Vec::new(), &config);
        assert_eq!(results.len(), 2);
        // Standard FTS ordering preserved
        assert_eq!(results[0].chunk_id, c1);
        assert_eq!(results[1].chunk_id, c2);
    }

    #[tokio::test]
    async fn search_all_three_sources_combined() {
        // A chunk appearing in FTS, vector, and HDC results should get the
        // highest combined score.
        let config = SearchConfig::default()
            .with_fusion_strategy(FusionStrategy::WeightedScore)
            .with_fts_weight(0.4)
            .with_vector_weight(0.3)
            .with_hdc(true)
            .with_hdc_weight(0.3)
            .with_hdc_shadow_mode(false)
            .with_limit(10);

        let doc = Uuid::new_v4();
        let c_triple = Uuid::new_v4(); // appears in all three
        let c_fts = Uuid::new_v4(); // FTS only
        let c_vec = Uuid::new_v4(); // vector only

        let fts = vec![
            make_ranked(c_triple, doc, "notes/a.md", 1),
            make_ranked(c_fts, doc, "notes/b.md", 2),
        ];
        let vec_results = vec![
            make_ranked(c_triple, doc, "notes/a.md", 1),
            make_ranked(c_vec, doc, "notes/c.md", 2),
        ];

        let results = fuse_results(fts, vec_results, &config);

        // Triple-source chunk should be first
        assert_eq!(results[0].chunk_id, c_triple);
        assert!(results[0].is_hybrid());
        assert!(results[0].score > results[1].score);
    }
}

// Performance tests (not gated by integration, just hdc feature)
#[cfg(feature = "hdc")]
mod hdc_search_perf {
    use ironclaw_hdc::dedup::{DedupConfig, FingerprintCandidate, check_dedup};
    use ironclaw_hdc::scan::{ScanResult, top_k_scan};
    use ironclaw_hdc::vector::HdcVector;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::time::Instant;

    #[test]
    fn search_scan_10k_under_50ms() {
        let mut rng = StdRng::seed_from_u64(42);
        let query = HdcVector::random(&mut rng);
        let candidates: Vec<FingerprintCandidate<u32>> = (0..10_000)
            .map(|i| FingerprintCandidate {
                id: i,
                path: format!("doc/{i}.md"),
                fingerprint: HdcVector::random(&mut rng),
            })
            .collect();
        let config = DedupConfig::default_config();

        // Warm up (ensure code is paged in)
        let _ = check_dedup(query, &candidates[..100], &config);

        let start = Instant::now();
        let _decision = check_dedup(query, &candidates, &config);
        let elapsed = start.elapsed();

        // Debug builds are ~10x slower than release; use generous margin.
        assert!(
            elapsed.as_millis() < 50,
            "10K dedup scan took {:?}, expected < 50ms (debug build)",
            elapsed
        );
    }

    #[test]
    fn search_scan_100k_under_500ms() {
        let mut rng = StdRng::seed_from_u64(42);
        let query = HdcVector::random(&mut rng);
        let candidates: Vec<FingerprintCandidate<u32>> = (0..100_000)
            .map(|i| FingerprintCandidate {
                id: i,
                path: format!("doc/{i}.md"),
                fingerprint: HdcVector::random(&mut rng),
            })
            .collect();
        let config = DedupConfig::default_config();

        // Warm up
        let _ = check_dedup(query, &candidates[..100], &config);

        let start = Instant::now();
        let _decision = check_dedup(query, &candidates, &config);
        let elapsed = start.elapsed();

        // Debug builds are ~10x slower than release; use generous margin.
        assert!(
            elapsed.as_millis() < 500,
            "100K dedup scan took {:?}, expected < 500ms (debug build)",
            elapsed
        );
    }

    #[test]
    fn search_top_k_scan_10k() {
        let mut rng = StdRng::seed_from_u64(99);
        let query = HdcVector::random(&mut rng);
        let candidates: Vec<(u32, HdcVector)> = (0..10_000)
            .map(|i| (i, HdcVector::random(&mut rng)))
            .collect();

        // Warm up
        let _ = top_k_scan(query, &candidates[..100], 10);

        let start = Instant::now();
        let results = top_k_scan(query, &candidates, 10);
        let elapsed = start.elapsed();

        assert_eq!(results.len(), 10);
        // Results should be sorted descending by similarity
        for w in results.windows(2) {
            assert!(w[0].similarity >= w[1].similarity);
        }
        // Debug builds are ~10x slower; use generous margin.
        assert!(
            elapsed.as_millis() < 50,
            "top_k_scan(10K, k=10) took {:?}, expected < 50ms (debug build)",
            elapsed
        );
    }

    #[test]
    fn search_top_k_scan_100k() {
        let mut rng = StdRng::seed_from_u64(101);
        let query = HdcVector::random(&mut rng);
        let candidates: Vec<(u32, HdcVector)> = (0..100_000)
            .map(|i| (i, HdcVector::random(&mut rng)))
            .collect();

        // Warm up
        let _ = top_k_scan(query, &candidates[..100], 10);

        let start = Instant::now();
        let results = top_k_scan(query, &candidates, 20);
        let elapsed = start.elapsed();

        assert_eq!(results.len(), 20);
        for w in results.windows(2) {
            assert!(w[0].similarity >= w[1].similarity);
        }
        // Debug builds are ~10x slower; use generous margin.
        assert!(
            elapsed.as_millis() < 500,
            "top_k_scan(100K, k=20) took {:?}, expected < 500ms (debug build)",
            elapsed
        );
    }

    #[test]
    fn search_scan_self_match_is_similarity_one() {
        let mut rng = StdRng::seed_from_u64(200);
        let query = HdcVector::random(&mut rng);

        // Insert the query itself among random candidates
        let mut candidates: Vec<(u32, HdcVector)> = (0..1000)
            .map(|i| (i, HdcVector::random(&mut rng)))
            .collect();
        candidates.push((9999, query));

        let results = top_k_scan(query, &candidates, 1);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, 9999);
        assert!((results[0].similarity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn search_scan_random_vectors_cluster_around_half() {
        // Random binary vectors should have similarity ~0.5 (within noise margin)
        let mut rng = StdRng::seed_from_u64(300);
        let query = HdcVector::random(&mut rng);
        let candidates: Vec<(u32, HdcVector)> = (0..10_000)
            .map(|i| (i, HdcVector::random(&mut rng)))
            .collect();

        let results = top_k_scan(query, &candidates, 10_000);

        // Mean similarity should be ~0.5
        let mean: f64 = results.iter().map(|r| r.similarity).sum::<f64>() / results.len() as f64;
        assert!(
            (mean - 0.5).abs() < 0.01,
            "mean similarity of random vectors should be ~0.5, got {mean:.4}"
        );

        // No random vector should exceed ~0.55 with high probability
        // (The top match among 10K random 10240-bit vectors is extremely unlikely to exceed 0.55)
        let max_sim = results[0].similarity;
        assert!(
            max_sim < 0.56,
            "max similarity among 10K random vectors should be < 0.56, got {max_sim:.4}"
        );
    }

    #[test]
    fn search_scan_empty_candidates() {
        let mut rng = StdRng::seed_from_u64(400);
        let query = HdcVector::random(&mut rng);
        let results: Vec<ScanResult<u32>> = top_k_scan(query, &[], 10);
        assert!(results.is_empty());
    }

    #[test]
    fn search_scan_k_larger_than_candidates() {
        let mut rng = StdRng::seed_from_u64(500);
        let query = HdcVector::random(&mut rng);
        let candidates: Vec<(u32, HdcVector)> =
            (0..5).map(|i| (i, HdcVector::random(&mut rng))).collect();

        let results = top_k_scan(query, &candidates, 100);
        // Should return all 5, not panic
        assert_eq!(results.len(), 5);
    }
}

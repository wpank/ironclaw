use crate::error::HdcError;
use crate::vector::HdcVector;

/// Configuration for deduplication thresholds.
#[derive(Debug, Clone, Copy)]
pub struct DedupConfig {
    /// Threshold for "similar" warning (e.g., 0.78).
    pub similar_threshold: f64,
    /// Threshold for "duplicate" blocking (e.g., 0.92).
    pub duplicate_threshold: f64,
}

impl DedupConfig {
    /// Create and validate config. Requirements:
    /// - similar_threshold < duplicate_threshold
    /// - similar_threshold > 0.5 (above random baseline)
    /// - both in (0.0, 1.0)
    pub fn new(similar_threshold: f64, duplicate_threshold: f64) -> Result<Self, HdcError> {
        if !similar_threshold.is_finite() {
            return Err(HdcError::InvalidDedupConfig {
                reason: format!("similar_threshold ({similar_threshold}) must be finite"),
            });
        }
        if !duplicate_threshold.is_finite() {
            return Err(HdcError::InvalidDedupConfig {
                reason: format!("duplicate_threshold ({duplicate_threshold}) must be finite"),
            });
        }
        if similar_threshold >= duplicate_threshold {
            return Err(HdcError::InvalidDedupConfig {
                reason: format!(
                    "similar_threshold ({similar_threshold}) must be < duplicate_threshold ({duplicate_threshold})"
                ),
            });
        }
        if similar_threshold <= 0.5 {
            return Err(HdcError::InvalidDedupConfig {
                reason: format!("similar_threshold ({similar_threshold}) must be > 0.5"),
            });
        }
        if duplicate_threshold >= 1.0 {
            return Err(HdcError::InvalidDedupConfig {
                reason: format!("duplicate_threshold ({duplicate_threshold}) must be < 1.0"),
            });
        }
        Ok(Self {
            similar_threshold,
            duplicate_threshold,
        })
    }

    /// Default conservative config.
    pub fn default_config() -> Self {
        Self {
            similar_threshold: 0.78,
            duplicate_threshold: 0.92,
        }
    }
}

/// Result of a dedup check.
#[derive(Debug, Clone)]
pub enum DedupDecision<Id> {
    /// No similar documents found.
    Unique,
    /// Found a similar document (between similar and duplicate thresholds).
    Similar {
        id: Id,
        path: String,
        similarity: f64,
    },
    /// Found a duplicate document (above duplicate threshold).
    Duplicate {
        id: Id,
        path: String,
        similarity: f64,
    },
}

/// A candidate fingerprint for comparison.
#[derive(Debug, Clone)]
pub struct FingerprintCandidate<Id> {
    pub id: Id,
    pub path: String,
    pub fingerprint: HdcVector,
}

/// Check a query fingerprint against candidates.
/// Returns the highest-matching decision.
///
/// # Examples
///
/// ```
/// use ironclaw_hdc::{encode_text, HdcVector};
/// use ironclaw_hdc::codebook::Codebook;
/// use ironclaw_hdc::dedup::{check_dedup, DedupConfig, DedupDecision, FingerprintCandidate};
///
/// let mut cb = Codebook::new();
/// let existing_fp = encode_text("the quick brown fox", &mut cb);
/// let new_fp = encode_text("the quick brown fox jumps", &mut cb);
///
/// let config = DedupConfig::default_config();
/// let candidates = vec![FingerprintCandidate {
///     id: 42u64,
///     path: "notes/fox.md".to_string(),
///     fingerprint: existing_fp,
/// }];
///
/// match check_dedup(new_fp, &candidates, &config) {
///     DedupDecision::Unique => println!("new content"),
///     DedupDecision::Similar { similarity, .. } => println!("similar: {similarity:.2}"),
///     DedupDecision::Duplicate { similarity, .. } => println!("duplicate: {similarity:.2}"),
/// }
/// ```
pub fn check_dedup<Id: Clone>(
    query: HdcVector,
    candidates: &[FingerprintCandidate<Id>],
    config: &DedupConfig,
) -> DedupDecision<Id> {
    if candidates.is_empty() {
        return DedupDecision::Unique;
    }

    let mut best_sim = 0.0_f64;
    let mut best_idx = 0;

    for (i, candidate) in candidates.iter().enumerate() {
        let sim = query.similarity(candidate.fingerprint);
        if sim > best_sim {
            best_sim = sim;
            best_idx = i;
        }
    }

    let best = &candidates[best_idx];

    if best_sim >= config.duplicate_threshold {
        DedupDecision::Duplicate {
            id: best.id.clone(),
            path: best.path.clone(),
            similarity: best_sim,
        }
    } else if best_sim >= config.similar_threshold {
        DedupDecision::Similar {
            id: best.id.clone(),
            path: best.path.clone(),
            similarity: best_sim,
        }
    } else {
        DedupDecision::Unique
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::DIMENSION_BITS;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    /// Flip exactly `n` bits in a vector to produce a controlled similarity.
    fn flip_bits(v: HdcVector, n: usize, rng: &mut impl rand::Rng) -> HdcVector {
        let mut result = v;
        let words = result.words_mut();
        // Pick n unique bit positions to flip
        let mut positions: Vec<usize> = (0..DIMENSION_BITS).collect();
        // Fisher-Yates partial shuffle
        for i in 0..n {
            let j = i + (rng.next_u64() as usize) % (DIMENSION_BITS - i);
            positions.swap(i, j);
        }
        for &pos in &positions[..n] {
            let word_idx = pos / 64;
            let bit_idx = pos % 64;
            words[word_idx] ^= 1u64 << bit_idx;
        }
        result
    }

    #[test]
    fn config_valid() {
        let config = DedupConfig::new(0.78, 0.92).unwrap();
        assert_eq!(config.similar_threshold, 0.78);
        assert_eq!(config.duplicate_threshold, 0.92);
    }

    #[test]
    fn config_rejects_similar_gte_duplicate() {
        assert!(DedupConfig::new(0.92, 0.78).is_err());
        assert!(DedupConfig::new(0.80, 0.80).is_err());
    }

    #[test]
    fn config_rejects_similar_lte_half() {
        assert!(DedupConfig::new(0.5, 0.92).is_err());
        assert!(DedupConfig::new(0.3, 0.92).is_err());
    }

    #[test]
    fn config_rejects_duplicate_gte_one() {
        assert!(DedupConfig::new(0.78, 1.0).is_err());
    }

    #[test]
    fn config_rejects_non_finite_thresholds() {
        assert!(DedupConfig::new(f64::NAN, 0.92).is_err());
        assert!(DedupConfig::new(f64::INFINITY, 0.92).is_err());
        assert!(DedupConfig::new(0.78, f64::NAN).is_err());
        assert!(DedupConfig::new(0.78, f64::INFINITY).is_err());
    }

    #[test]
    fn dedup_config_validation() {
        // similar >= duplicate is rejected
        assert!(DedupConfig::new(0.90, 0.85).is_err());
        assert!(DedupConfig::new(0.80, 0.80).is_err());
        // similar <= 0.5 is rejected
        assert!(DedupConfig::new(0.50, 0.92).is_err());
        assert!(DedupConfig::new(0.40, 0.92).is_err());
    }

    #[test]
    fn dedup_empty_candidates() {
        let query = HdcVector::from_seed("test", b"query");
        let config = DedupConfig::default_config();
        let result: DedupDecision<u64> = check_dedup(query, &[], &config);
        assert!(matches!(result, DedupDecision::Unique));
    }

    #[test]
    fn dedup_unique() {
        // No candidates above threshold → Unique
        let mut rng = StdRng::seed_from_u64(100);
        let query = HdcVector::random(&mut rng);
        let config = DedupConfig::default_config();
        // Create candidates that are random (similarity ~0.5)
        let candidates: Vec<FingerprintCandidate<u64>> = (0..5)
            .map(|i| FingerprintCandidate {
                id: i,
                path: format!("doc_{i}.md"),
                fingerprint: HdcVector::random(&mut rng),
            })
            .collect();
        let result = check_dedup(query, &candidates, &config);
        assert!(matches!(result, DedupDecision::Unique));
    }

    #[test]
    fn dedup_similar() {
        // Two vectors with similarity ~0.80 → Similar
        let mut rng = StdRng::seed_from_u64(200);
        let query = HdcVector::random(&mut rng);
        // Flip ~20% of bits: 0.80 similarity → flip 2048 bits
        let similar_vec = flip_bits(query, 2048, &mut rng);
        let sim = query.similarity(similar_vec);
        assert!(
            sim > 0.78 && sim < 0.92,
            "expected similarity in (0.78, 0.92), got {sim}"
        );

        let config = DedupConfig::default_config();
        let candidates = vec![FingerprintCandidate {
            id: 1u64,
            path: "similar.md".to_string(),
            fingerprint: similar_vec,
        }];
        let result = check_dedup(query, &candidates, &config);
        assert!(
            matches!(result, DedupDecision::Similar { similarity, .. } if similarity > 0.78 && similarity < 0.92)
        );
    }

    #[test]
    fn dedup_duplicate() {
        // Two vectors with similarity ~0.95 → Duplicate
        let mut rng = StdRng::seed_from_u64(300);
        let query = HdcVector::random(&mut rng);
        // Flip ~5% of bits: 0.95 similarity → flip 512 bits
        let dup_vec = flip_bits(query, 512, &mut rng);
        let sim = query.similarity(dup_vec);
        assert!(sim > 0.92, "expected similarity > 0.92, got {sim}");

        let config = DedupConfig::default_config();
        let candidates = vec![FingerprintCandidate {
            id: 1u64,
            path: "dup.md".to_string(),
            fingerprint: dup_vec,
        }];
        let result = check_dedup(query, &candidates, &config);
        assert!(matches!(result, DedupDecision::Duplicate { similarity, .. } if similarity > 0.92));
    }

    #[test]
    fn dedup_exact_match() {
        // Same vector → Duplicate with similarity 1.0
        let v = HdcVector::from_seed("test", b"same");
        let config = DedupConfig::default_config();
        let candidates = vec![FingerprintCandidate {
            id: 1u64,
            path: "doc.md".to_string(),
            fingerprint: v,
        }];
        let result = check_dedup(v, &candidates, &config);
        assert!(matches!(result, DedupDecision::Duplicate { similarity, .. } if similarity == 1.0));
    }

    #[test]
    fn dedup_highest_wins() {
        // Two candidates: one ~0.79 (Similar) and one ~0.93 (Duplicate). Returns the 0.93 one.
        let mut rng = StdRng::seed_from_u64(400);
        let query = HdcVector::random(&mut rng);

        // Candidate at ~0.79: flip (1-0.79)*10240 = 2150 bits
        let similar_vec = flip_bits(query, 2150, &mut rng);
        // Candidate at ~0.93: flip (1-0.93)*10240 = 717 bits
        let dup_vec = flip_bits(query, 717, &mut rng);

        let sim_low = query.similarity(similar_vec);
        let sim_high = query.similarity(dup_vec);
        assert!(sim_low < 0.92, "low candidate sim = {sim_low}");
        assert!(sim_high > 0.92, "high candidate sim = {sim_high}");

        let config = DedupConfig::default_config();
        let candidates = vec![
            FingerprintCandidate {
                id: 1u64,
                path: "low.md".to_string(),
                fingerprint: similar_vec,
            },
            FingerprintCandidate {
                id: 2u64,
                path: "high.md".to_string(),
                fingerprint: dup_vec,
            },
        ];
        let result = check_dedup(query, &candidates, &config);
        match result {
            DedupDecision::Duplicate { id, similarity, .. } => {
                assert_eq!(id, 2);
                assert!(similarity > 0.92);
            }
            other => panic!("expected Duplicate, got {other:?}"),
        }
    }

    #[test]
    fn dedup_just_below_similar() {
        // Candidate just below 0.78 → Unique
        let mut rng = StdRng::seed_from_u64(500);
        let query = HdcVector::random(&mut rng);
        // similarity just below 0.78: flip (1-0.77)*10240 = 2355 bits
        let vec = flip_bits(query, 2355, &mut rng);
        let sim = query.similarity(vec);
        assert!(sim < 0.78, "expected sim < 0.78, got {sim}");

        let config = DedupConfig::default_config();
        let candidates = vec![FingerprintCandidate {
            id: 1u64,
            path: "below.md".to_string(),
            fingerprint: vec,
        }];
        let result = check_dedup(query, &candidates, &config);
        assert!(matches!(result, DedupDecision::Unique));
    }

    #[test]
    fn default_config_satisfies_validation() {
        let defaults = DedupConfig::default_config();
        // Verify that the hardcoded defaults would also pass new() validation
        let validated = DedupConfig::new(defaults.similar_threshold, defaults.duplicate_threshold);
        assert!(
            validated.is_ok(),
            "default_config values ({}, {}) fail validation: {:?}",
            defaults.similar_threshold,
            defaults.duplicate_threshold,
            validated.err()
        );
    }

    #[test]
    fn dedup_just_above_similar() {
        // Candidate just above 0.78 → Similar
        let mut rng = StdRng::seed_from_u64(600);
        let query = HdcVector::random(&mut rng);
        // similarity just above 0.78: flip (1-0.79)*10240 = 2150 bits
        let vec = flip_bits(query, 2150, &mut rng);
        let sim = query.similarity(vec);
        assert!(sim > 0.78, "expected sim > 0.78, got {sim}");
        assert!(sim < 0.92, "expected sim < 0.92, got {sim}");

        let config = DedupConfig::default_config();
        let candidates = vec![FingerprintCandidate {
            id: 1u64,
            path: "above.md".to_string(),
            fingerprint: vec,
        }];
        let result = check_dedup(query, &candidates, &config);
        assert!(
            matches!(result, DedupDecision::Similar { .. }),
            "expected Similar, got {result:?}"
        );
    }
}

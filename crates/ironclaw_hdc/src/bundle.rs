use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::HdcError;
use crate::vector::{DIMENSION_BITS, HdcVector};

/// Majority-vote bundle accumulator.
///
/// Tracks per-dimension vote counts using i32 (positive for 1-bits, negative for 0-bits).
/// On finalize, positive votes become 1, non-positive become 0.
///
/// # Examples
///
/// ```
/// use ironclaw_hdc::vector::HdcVector;
/// use ironclaw_hdc::bundle::BundleAccumulator;
///
/// // Bundle three vectors — the result is similar to all inputs
/// let v1 = HdcVector::from_seed("demo", b"alpha");
/// let v2 = HdcVector::from_seed("demo", b"beta");
/// let v3 = HdcVector::from_seed("demo", b"gamma");
///
/// let mut acc = BundleAccumulator::new();
/// acc.add(&v1);
/// acc.add(&v2);
/// acc.add(&v3);
/// let bundled = acc.finalize();
///
/// // The bundle is more similar to each input than random vectors would be
/// assert!(bundled.similarity(v1) > 0.55);
/// assert!(bundled.similarity(v2) > 0.55);
/// assert!(bundled.similarity(v3) > 0.55);
/// ```
pub struct BundleAccumulator {
    votes: Box<[i32; DIMENSION_BITS]>,
    count: u32,
}

impl BundleAccumulator {
    pub fn new() -> Self {
        Self {
            votes: Box::new([0i32; DIMENSION_BITS]),
            count: 0,
        }
    }

    /// Add a vector: for each bit, if 1 increment vote, if 0 decrement vote.
    pub fn add(&mut self, vector: &HdcVector) {
        let words = vector.words();
        for i in 0..DIMENSION_BITS {
            let word_idx = i / 64;
            let bit_idx = i % 64;
            if (words[word_idx] >> bit_idx) & 1 == 1 {
                self.votes[i] += 1;
            } else {
                self.votes[i] -= 1;
            }
        }
        self.count += 1;
    }

    /// Add a vector with integer weight.
    pub fn add_weighted(&mut self, vector: &HdcVector, weight: u32) {
        let words = vector.words();
        let w = weight as i32;
        for i in 0..DIMENSION_BITS {
            let word_idx = i / 64;
            let bit_idx = i % 64;
            if (words[word_idx] >> bit_idx) & 1 == 1 {
                self.votes[i] += w;
            } else {
                self.votes[i] -= w;
            }
        }
        self.count += weight;
    }

    /// Finalize: positive votes become 1, non-positive (zero or negative) become 0.
    pub fn finalize(&self) -> HdcVector {
        let mut result = HdcVector::zero();
        let words = result.words_mut();
        for i in 0..DIMENSION_BITS {
            if self.votes[i] > 0 {
                let word_idx = i / 64;
                let bit_idx = i % 64;
                words[word_idx] |= 1u64 << bit_idx;
            }
        }
        result
    }

    /// Number of vectors added (weighted adds contribute their weight).
    pub fn count(&self) -> u32 {
        self.count
    }
}

impl Default for BundleAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

/// Decaying bundle accumulator with exponential decay for recency bias.
///
/// Each time a vector is added, all existing votes are multiplied by `decay_factor`
/// before the new contribution is applied. This gives recent vectors more influence.
pub struct DecayingBundleAccumulator {
    votes: Box<[f32; DIMENSION_BITS]>,
    decay_factor: f32,
    count: u64,
}

impl Serialize for DecayingBundleAccumulator {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DecayingBundleAccumulator", 3)?;
        state.serialize_field("votes", self.votes.as_slice())?;
        state.serialize_field("decay_factor", &self.decay_factor)?;
        state.serialize_field("count", &self.count)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for DecayingBundleAccumulator {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Helper {
            votes: Vec<f32>,
            decay_factor: f32,
            count: u64,
        }
        let h = Helper::deserialize(deserializer)?;
        if h.votes.len() != DIMENSION_BITS {
            return Err(serde::de::Error::custom(format!(
                "expected {} votes, got {}",
                DIMENSION_BITS,
                h.votes.len()
            )));
        }
        if !h.decay_factor.is_finite() || h.decay_factor <= 0.0 || h.decay_factor >= 1.0 {
            return Err(serde::de::Error::custom(format!(
                "invalid decay_factor {}; expected finite value in (0.0, 1.0)",
                h.decay_factor
            )));
        }
        if h.votes.iter().any(|vote| !vote.is_finite()) {
            return Err(serde::de::Error::custom(
                "votes must contain only finite values",
            ));
        }
        let mut votes = Box::new([0.0f32; DIMENSION_BITS]);
        votes.copy_from_slice(&h.votes);
        Ok(Self {
            votes,
            decay_factor: h.decay_factor,
            count: h.count,
        })
    }
}

impl DecayingBundleAccumulator {
    /// Create new. `decay_factor` must be in (0.0, 1.0) exclusive.
    pub fn new(decay_factor: f32) -> Result<Self, HdcError> {
        if !decay_factor.is_finite() || decay_factor <= 0.0 || decay_factor >= 1.0 {
            return Err(HdcError::InvalidDecayFactor {
                value: decay_factor,
            });
        }
        Ok(Self {
            votes: Box::new([0.0f32; DIMENSION_BITS]),
            decay_factor,
            count: 0,
        })
    }

    /// Add vector: multiply existing votes by decay_factor, then add +1/-1 per bit.
    pub fn add(&mut self, vector: &HdcVector) {
        let words = vector.words();
        let decay = self.decay_factor;
        for i in 0..DIMENSION_BITS {
            self.votes[i] *= decay;
            let word_idx = i / 64;
            let bit_idx = i % 64;
            if (words[word_idx] >> bit_idx) & 1 == 1 {
                self.votes[i] += 1.0;
            } else {
                self.votes[i] -= 1.0;
            }
        }
        self.count += 1;
    }

    /// Finalize: positive votes become 1, non-positive become 0.
    pub fn finalize(&self) -> HdcVector {
        let mut result = HdcVector::zero();
        let words = result.words_mut();
        for i in 0..DIMENSION_BITS {
            if self.votes[i] > 0.0 {
                let word_idx = i / 64;
                let bit_idx = i % 64;
                words[word_idx] |= 1u64 << bit_idx;
            }
        }
        result
    }

    /// Effective half-life in cycles: ln(0.5) / ln(decay_factor).
    pub fn effective_half_life_cycles(&self) -> f64 {
        (0.5_f64).ln() / (self.decay_factor as f64).ln()
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    /// Reset all state.
    pub fn reset(&mut self) {
        self.votes.iter_mut().for_each(|v| *v = 0.0);
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    // ─── BundleAccumulator ───────────────────────────────────────────────────

    #[test]
    fn bundle_single_vector() {
        let v = HdcVector::from_seed("bundle", b"single");
        let mut acc = BundleAccumulator::new();
        acc.add(&v);
        assert_eq!(acc.finalize(), v);
    }

    #[test]
    fn bundle_majority_3v1() {
        let a = HdcVector::from_seed("bundle", b"majority-a");
        let b = HdcVector::from_seed("bundle", b"majority-b");
        let mut acc = BundleAccumulator::new();
        acc.add(&a);
        acc.add(&a);
        acc.add(&a);
        acc.add(&b);
        let result = acc.finalize();
        assert!(result.similarity(a) > result.similarity(b));
    }

    #[test]
    fn bundle_weighted() {
        let a = HdcVector::from_seed("bundle", b"weighted-a");
        let b = HdcVector::from_seed("bundle", b"weighted-b");
        let mut acc = BundleAccumulator::new();
        acc.add_weighted(&a, 5);
        acc.add_weighted(&b, 1);
        let result = acc.finalize();
        assert!(result.similarity(a) > result.similarity(b));
    }

    #[test]
    fn bundle_tie_resolves_to_zero() {
        // When a vector and its complement are added, every dimension has
        // exactly one +1 and one -1 vote, netting zero. Finalize maps
        // non-positive votes to 0-bits, producing the zero vector.
        let a = HdcVector::from_seed("bundle", b"tie-vector");
        // Build complement: flip every bit by XOR with all-ones
        let mut all_ones = HdcVector::zero();
        for word in all_ones.words_mut().iter_mut() {
            *word = u64::MAX;
        }
        let complement = a.bind(all_ones);

        let mut acc = BundleAccumulator::new();
        acc.add(&a);
        acc.add(&complement);
        assert_eq!(acc.finalize(), HdcVector::zero());
    }

    #[test]
    fn bundle_empty_finalize() {
        let acc = BundleAccumulator::new();
        assert_eq!(acc.finalize(), HdcVector::zero());
    }

    #[test]
    fn bundle_count() {
        let mut acc = BundleAccumulator::new();
        let v = HdcVector::from_seed("bundle", b"count");
        for _ in 0..7 {
            acc.add(&v);
        }
        assert_eq!(acc.count(), 7);
    }

    #[test]
    fn bundle_capacity_gradient() {
        // As K increases, bundling K random vectors dilutes similarity to any single one.
        let mut rng = StdRng::seed_from_u64(42);
        let ks = [2, 5, 10, 50, 100, 200];
        let mut prev_similarity = 1.0_f64;

        for &k in &ks {
            let vectors: Vec<HdcVector> = (0..k).map(|_| HdcVector::random(&mut rng)).collect();
            let mut acc = BundleAccumulator::new();
            for v in &vectors {
                acc.add(v);
            }
            let result = acc.finalize();
            let sim = result.similarity(vectors[0]);
            assert!(
                sim < prev_similarity,
                "K={k}: similarity {sim} should be less than previous {prev_similarity}"
            );
            prev_similarity = sim;
        }
    }

    // ─── DecayingBundleAccumulator ───────────────────────────────────────────

    #[test]
    fn decay_rejects_zero() {
        assert!(DecayingBundleAccumulator::new(0.0).is_err());
    }

    #[test]
    fn decay_rejects_one() {
        assert!(DecayingBundleAccumulator::new(1.0).is_err());
    }

    #[test]
    fn decay_rejects_negative() {
        assert!(DecayingBundleAccumulator::new(-0.5).is_err());
    }

    #[test]
    fn decay_rejects_nan() {
        assert!(DecayingBundleAccumulator::new(f32::NAN).is_err());
    }

    #[test]
    fn decay_rejects_infinity() {
        assert!(DecayingBundleAccumulator::new(f32::INFINITY).is_err());
    }

    #[test]
    fn decay_recency_bias() {
        let a = HdcVector::from_seed("decay", b"recency-a");
        let b = HdcVector::from_seed("decay", b"recency-b");

        // With decay: adding A 10 times then B once gives B more weight
        let mut decaying = DecayingBundleAccumulator::new(0.5).unwrap();
        for _ in 0..10 {
            decaying.add(&a);
        }
        decaying.add(&b);
        let decayed_result = decaying.finalize();

        // Without decay: same adds give A more weight
        let mut flat = BundleAccumulator::new();
        for _ in 0..10 {
            flat.add(&a);
        }
        flat.add(&b);
        let flat_result = flat.finalize();

        // Decaying accumulator should favor B more than the flat one
        assert!(decayed_result.similarity(b) > flat_result.similarity(b));
    }

    #[test]
    fn decay_half_life() {
        let acc = DecayingBundleAccumulator::new(0.95).unwrap();
        let hl = acc.effective_half_life_cycles();
        // ln(0.5)/ln(0.95) ≈ 13.5134
        assert!(
            (hl - 13.5134).abs() < 0.1,
            "half-life was {hl}, expected ~13.5134"
        );
    }

    #[test]
    fn decay_full_forget() {
        let a = HdcVector::from_seed("decay", b"forget-target");
        let mut rng = StdRng::seed_from_u64(42);
        let mut acc = DecayingBundleAccumulator::new(0.9).unwrap();
        acc.add(&a);
        // Flood with 200 random vectors — the initial A should be forgotten
        for _ in 0..200 {
            acc.add(&HdcVector::random(&mut rng));
        }
        let result = acc.finalize();
        let sim = result.similarity(a);
        // After aggressive forgetting, similarity should be near random (~0.5)
        assert!(
            (sim - 0.5).abs() < 0.05,
            "similarity to forgotten vector was {sim}, expected ~0.5"
        );
    }

    #[test]
    fn decay_reset() {
        let v = HdcVector::from_seed("decay", b"reset-vec");
        let mut acc = DecayingBundleAccumulator::new(0.9).unwrap();
        acc.add(&v);
        acc.add(&v);
        acc.reset();
        assert_eq!(acc.count(), 0);
        assert_eq!(acc.finalize(), HdcVector::zero());
    }

    #[test]
    fn decay_serde_roundtrip() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut acc = DecayingBundleAccumulator::new(0.85).unwrap();
        for _ in 0..5 {
            acc.add(&HdcVector::random(&mut rng));
        }
        let before = acc.finalize();

        let json = serde_json::to_string(&acc).unwrap();
        let restored: DecayingBundleAccumulator = serde_json::from_str(&json).unwrap();
        let after = restored.finalize();

        assert_eq!(before, after);
        assert_eq!(acc.count(), restored.count());
    }

    #[test]
    fn decay_serde_rejects_invalid_decay_factor() {
        let votes = serde_json::to_string(&vec![0.0f32; DIMENSION_BITS]).unwrap();
        let json = format!(r#"{{"votes":{votes},"decay_factor":1.0,"count":0}}"#);
        let restored: Result<DecayingBundleAccumulator, _> = serde_json::from_str(&json);
        assert!(restored.is_err());
    }
}

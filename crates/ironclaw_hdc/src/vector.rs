use crate::error::HdcError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Number of bits in each hypervector.
pub const DIMENSION_BITS: usize = 10_240;
/// Number of u64 words needed.
pub const WORDS: usize = 160; // 10_240 / 64
/// Byte length of serialized vector.
pub const BYTE_LEN: usize = 1_280; // 160 * 8

/// A 10,240-bit binary hyperdimensional vector stored as `[u64; 160]`.
///
/// All operations are allocation-free. The vector supports:
/// - XOR binding (associative, commutative, self-inverse)
/// - Cyclic permutation across the full bit width
/// - Hamming distance and normalized similarity
/// - Deterministic seeded construction
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct HdcVector([u64; WORDS]);

impl Serialize for HdcVector {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

impl<'de> Deserialize<'de> for HdcVector {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        Self::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

/// FNV-1a hash for deterministic seed derivation.
fn fnv1a_64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// SplitMix64 PRNG — simple, fast, deterministic.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

impl HdcVector {
    /// Create a zero vector (all bits cleared).
    pub const fn zero() -> Self {
        Self([0u64; WORDS])
    }

    /// Create a random vector from the given RNG.
    pub fn random(rng: &mut impl rand::Rng) -> Self {
        let mut words = [0u64; WORDS];
        for word in words.iter_mut() {
            *word = rng.next_u64();
        }
        Self(words)
    }

    /// Deterministic construction from a domain string and seed bytes.
    ///
    /// Concatenates domain bytes + 0x00 separator + seed bytes, hashes with
    /// FNV-1a to derive a 64-bit seed, then fills all 160 words using SplitMix64.
    /// Produces identical output across platforms and runs.
    pub fn from_seed(domain: &str, seed: &[u8]) -> Self {
        let mut input = Vec::with_capacity(domain.len() + 1 + seed.len());
        input.extend_from_slice(domain.as_bytes());
        input.push(0x00);
        input.extend_from_slice(seed);

        let mut state = fnv1a_64(&input);
        let mut words = [0u64; WORDS];
        for word in words.iter_mut() {
            *word = splitmix64(&mut state);
        }
        Self(words)
    }

    /// Deserialize from little-endian bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, HdcError> {
        if bytes.len() != BYTE_LEN {
            return Err(HdcError::InvalidBytes {
                expected: BYTE_LEN,
                got: bytes.len(),
            });
        }
        let mut words = [0u64; WORDS];
        for (i, word) in words.iter_mut().enumerate() {
            let offset = i * 8;
            *word = u64::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
        }
        Ok(Self(words))
    }

    /// Serialize to little-endian bytes.
    pub fn to_bytes(self) -> [u8; BYTE_LEN] {
        let mut out = [0u8; BYTE_LEN];
        for (i, &word) in self.0.iter().enumerate() {
            let offset = i * 8;
            let le = word.to_le_bytes();
            out[offset..offset + 8].copy_from_slice(&le);
        }
        out
    }

    /// XOR binding — associative, commutative, self-inverse.
    pub fn bind(self, other: Self) -> Self {
        let mut result = [0u64; WORDS];
        for (i, word) in result.iter_mut().enumerate() {
            *word = self.0[i] ^ other.0[i];
        }
        Self(result)
    }

    /// Cyclic left shift across the full 10,240-bit vector.
    ///
    /// The shift wraps around the entire bit space, not per-word.
    pub fn permute(self, shift: usize) -> Self {
        let effective = shift % DIMENSION_BITS;
        if effective == 0 {
            return self;
        }

        let word_shift = effective / 64;
        let bit_shift = effective % 64;

        let mut result = [0u64; WORDS];

        if bit_shift == 0 {
            // Pure word rotation (no bit shifting needed).
            for (i, word) in result.iter_mut().enumerate() {
                *word = self.0[(i + WORDS - word_shift) % WORDS];
            }
        } else {
            // Rotate words, then shift bits across word boundaries.
            for (i, word) in result.iter_mut().enumerate() {
                let src_hi = (i + WORDS - word_shift) % WORDS;
                let src_lo = (i + WORDS - word_shift + WORDS - 1) % WORDS;
                *word = (self.0[src_hi] << bit_shift) | (self.0[src_lo] >> (64 - bit_shift));
            }
        }

        Self(result)
    }

    /// Hamming distance (number of differing bits).
    pub fn hamming_distance(self, other: Self) -> u32 {
        let mut count = 0u32;
        for i in 0..WORDS {
            count += (self.0[i] ^ other.0[i]).count_ones();
        }
        count
    }

    /// Normalized similarity: `1.0 - hamming_distance / DIMENSION_BITS`.
    ///
    /// Returns 1.0 for identical vectors, ~0.5 for random independent vectors.
    pub fn similarity(self, other: Self) -> f64 {
        1.0 - self.hamming_distance(other) as f64 / DIMENSION_BITS as f64
    }

    /// Access the underlying word array (for downstream modules that need raw access).
    pub fn words(&self) -> &[u64; WORDS] {
        &self.0
    }

    /// Mutable access to the underlying word array.
    pub fn words_mut(&mut self) -> &mut [u64; WORDS] {
        &mut self.0
    }
}

impl std::fmt::Debug for HdcVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "HdcVector({:016x}_{:016x}_{:016x}_{:016x}...)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn seeded_rng() -> StdRng {
        StdRng::seed_from_u64(42)
    }

    #[test]
    fn bind_involution() {
        let mut rng = seeded_rng();
        for _ in 0..50 {
            let a = HdcVector::random(&mut rng);
            let b = HdcVector::random(&mut rng);
            assert_eq!(a.bind(b).bind(b), a);
        }
    }

    #[test]
    fn bind_commutative() {
        let mut rng = seeded_rng();
        for _ in 0..50 {
            let a = HdcVector::random(&mut rng);
            let b = HdcVector::random(&mut rng);
            assert_eq!(a.bind(b), b.bind(a));
        }
    }

    #[test]
    fn bind_self_is_zero() {
        let mut rng = seeded_rng();
        let zero = HdcVector::zero();
        for _ in 0..50 {
            let a = HdcVector::random(&mut rng);
            assert_eq!(a.bind(a), zero);
        }
    }

    #[test]
    fn bind_quasi_orthogonal() {
        let mut rng = seeded_rng();
        for _ in 0..50 {
            let a = HdcVector::random(&mut rng);
            let b = HdcVector::random(&mut rng);
            let sim = a.similarity(a.bind(b));
            assert!(
                (0.48..0.52).contains(&sim),
                "expected similarity in 0.48..0.52, got {sim}"
            );
        }
    }

    #[test]
    fn similarity_identity() {
        let mut rng = seeded_rng();
        for _ in 0..10 {
            let a = HdcVector::random(&mut rng);
            assert_eq!(a.similarity(a), 1.0);
        }
    }

    #[test]
    fn similarity_zero_zero() {
        assert_eq!(HdcVector::zero().similarity(HdcVector::zero()), 1.0);
    }

    #[test]
    fn similarity_bounds() {
        let mut rng = seeded_rng();
        for _ in 0..1000 {
            let a = HdcVector::random(&mut rng);
            let b = HdcVector::random(&mut rng);
            let sim = a.similarity(b);
            assert!(
                (0.0..=1.0).contains(&sim),
                "similarity out of bounds: {sim}"
            );
        }
    }

    #[test]
    fn similarity_random_concentration() {
        let mut rng = seeded_rng();
        let mut sims = Vec::with_capacity(1000);
        for _ in 0..1000 {
            let a = HdcVector::random(&mut rng);
            let b = HdcVector::random(&mut rng);
            sims.push(a.similarity(b));
        }
        let mean = sims.iter().sum::<f64>() / sims.len() as f64;
        let variance = sims.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / sims.len() as f64;
        let stddev = variance.sqrt();

        assert!(
            (0.495..0.505).contains(&mean),
            "mean similarity {mean} not in 0.495..0.505"
        );
        assert!(stddev < 0.008, "stddev {stddev} >= 0.008");
    }

    #[test]
    fn similarity_statistical_threshold() {
        let mut rng = seeded_rng();
        for _ in 0..1000 {
            let a = HdcVector::random(&mut rng);
            let b = HdcVector::random(&mut rng);
            let sim = a.similarity(b);
            assert!(sim <= 0.53, "random pair similarity {sim} exceeded 0.53");
        }
    }

    #[test]
    fn permute_identity() {
        let mut rng = seeded_rng();
        let a = HdcVector::random(&mut rng);
        assert_eq!(a.permute(0), a);
    }

    #[test]
    fn permute_full_cycle() {
        let mut rng = seeded_rng();
        let a = HdcVector::random(&mut rng);
        assert_eq!(a.permute(DIMENSION_BITS), a);
    }

    #[test]
    fn permute_inverse() {
        let mut rng = seeded_rng();
        let a = HdcVector::random(&mut rng);
        for k in [1, 7, 63, 64, 160, 5000, 10239] {
            assert_eq!(
                a.permute(k).permute(DIMENSION_BITS - k),
                a,
                "permute inverse failed for k={k}"
            );
        }
    }

    #[test]
    fn permute_cross_word_boundary() {
        let mut rng = seeded_rng();
        let a = HdcVector::random(&mut rng);
        assert_ne!(
            a.permute(63),
            a.permute(65),
            "permute(63) and permute(65) should differ for non-trivial vector"
        );
    }

    #[test]
    fn permute_produces_different_vector() {
        let mut rng = seeded_rng();
        let a = HdcVector::random(&mut rng);
        assert_ne!(a.permute(1), a);
    }

    #[test]
    fn from_seed_deterministic() {
        for _ in 0..10 {
            let a = HdcVector::from_seed("test_domain", b"test_seed");
            let b = HdcVector::from_seed("test_domain", b"test_seed");
            assert_eq!(a, b);
        }
    }

    #[test]
    fn from_seed_domain_separation() {
        let a = HdcVector::from_seed("tag", b"rust");
        let b = HdcVector::from_seed("path", b"rust");
        assert_ne!(a, b);
    }

    #[test]
    fn from_seed_different_seeds_orthogonal() {
        let a = HdcVector::from_seed("domain", b"seed_one");
        let b = HdcVector::from_seed("domain", b"seed_two");
        let sim = a.similarity(b);
        assert!(
            (0.48..0.52).contains(&sim),
            "expected near-orthogonal similarity, got {sim}"
        );
    }

    #[test]
    fn from_seed_empty() {
        let _ = HdcVector::from_seed("d", b"");
    }

    #[test]
    fn bytes_roundtrip() {
        let mut rng = seeded_rng();
        for _ in 0..10 {
            let v = HdcVector::random(&mut rng);
            let restored = HdcVector::from_bytes(&v.to_bytes()).unwrap();
            assert_eq!(v, restored);
        }
    }

    #[test]
    fn from_bytes_rejects_short() {
        let err = HdcVector::from_bytes(&[0u8; 100]).unwrap_err();
        match err {
            HdcError::InvalidBytes { expected, got } => {
                assert_eq!(expected, BYTE_LEN);
                assert_eq!(got, 100);
            }
            _ => panic!("expected InvalidBytes, got {err:?}"),
        }
    }

    #[test]
    fn from_bytes_rejects_long() {
        let err = HdcVector::from_bytes(&[0u8; 2000]).unwrap_err();
        match err {
            HdcError::InvalidBytes { expected, got } => {
                assert_eq!(expected, BYTE_LEN);
                assert_eq!(got, 2000);
            }
            _ => panic!("expected InvalidBytes, got {err:?}"),
        }
    }

    #[test]
    fn from_bytes_accepts_exact() {
        let result = HdcVector::from_bytes(&[0u8; BYTE_LEN]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), HdcVector::zero());
    }

    #[test]
    fn hamming_distance_self() {
        let mut rng = seeded_rng();
        for _ in 0..10 {
            let v = HdcVector::random(&mut rng);
            assert_eq!(v.hamming_distance(v), 0);
        }
    }

    #[test]
    fn hamming_distance_complement() {
        let mut rng = seeded_rng();
        let a = HdcVector::random(&mut rng);
        let mut all_ones = [0u64; WORDS];
        for word in all_ones.iter_mut() {
            *word = u64::MAX;
        }
        let complement = a.bind(HdcVector(all_ones));
        assert_eq!(a.hamming_distance(complement), DIMENSION_BITS as u32);
    }
}

use ironclaw_hdc::bundle::BundleAccumulator;
use ironclaw_hdc::dedup::{DedupConfig, DedupDecision, FingerprintCandidate, check_dedup};
use ironclaw_hdc::vector::{BYTE_LEN, DIMENSION_BITS, HdcVector};
use proptest::prelude::*;
use rand::SeedableRng;
use rand::rngs::StdRng;

proptest! {
    #[test]
    fn prop_bind_involution(seed_a in any::<u64>(), seed_b in any::<u64>()) {
        let mut rng_a = StdRng::seed_from_u64(seed_a);
        let mut rng_b = StdRng::seed_from_u64(seed_b);
        let a = HdcVector::random(&mut rng_a);
        let b = HdcVector::random(&mut rng_b);
        // bind is self-inverse: (a XOR b) XOR b == a
        prop_assert_eq!(a.bind(b).bind(b), a);
    }

    #[test]
    fn prop_similarity_in_unit(seed_a in any::<u64>(), seed_b in any::<u64>()) {
        let mut rng_a = StdRng::seed_from_u64(seed_a);
        let mut rng_b = StdRng::seed_from_u64(seed_b);
        let a = HdcVector::random(&mut rng_a);
        let b = HdcVector::random(&mut rng_b);
        let sim = a.similarity(b);
        prop_assert!((0.0..=1.0).contains(&sim), "similarity out of [0,1]: {}", sim);
    }

    #[test]
    fn prop_permute_inverse(seed in any::<u64>(), k in 0..DIMENSION_BITS) {
        let mut rng = StdRng::seed_from_u64(seed);
        let a = HdcVector::random(&mut rng);
        // permute(k) then permute(N-k) should give back original
        prop_assert_eq!(a.permute(k).permute(DIMENSION_BITS - k), a);
    }

    #[test]
    fn prop_bytes_roundtrip(seed in any::<u64>()) {
        let mut rng = StdRng::seed_from_u64(seed);
        let v = HdcVector::random(&mut rng);
        let bytes = v.to_bytes();
        prop_assert_eq!(bytes.len(), BYTE_LEN);
        let v2 = HdcVector::from_bytes(&bytes).unwrap();
        prop_assert_eq!(v, v2);
    }

    #[test]
    fn prop_bundle_similarity_monotonic(seed in any::<u64>(), extra_copies in 1u32..20) {
        let mut rng = StdRng::seed_from_u64(seed);
        let a = HdcVector::random(&mut rng);
        let b = HdcVector::random(&mut rng);

        // Bundle with equal weight: 1 copy of a, 1 copy of b
        let mut acc1 = BundleAccumulator::new();
        acc1.add(&a);
        acc1.add(&b);
        let sim1 = acc1.finalize().similarity(a);

        // Bundle with more copies of a: (1 + extra_copies) of a, 1 of b
        let mut acc2 = BundleAccumulator::new();
        for _ in 0..(1 + extra_copies) {
            acc2.add(&a);
        }
        acc2.add(&b);
        let sim2 = acc2.finalize().similarity(a);

        // Adding more copies of A should not decrease similarity to A
        // (small epsilon for floating point edge cases)
        prop_assert!(
            sim2 >= sim1 - 0.001,
            "Adding more copies of A should not decrease similarity to A: sim1={}, sim2={}",
            sim1,
            sim2
        );
    }

    #[test]
    fn prop_dedup_self_is_duplicate(seed in any::<u64>()) {
        let mut rng = StdRng::seed_from_u64(seed);
        let v = HdcVector::random(&mut rng);
        let config = DedupConfig::default_config();
        let candidates = vec![FingerprintCandidate {
            id: 0u64,
            path: "self.md".to_string(),
            fingerprint: v,
        }];
        let result = check_dedup(v, &candidates, &config);
        match result {
            DedupDecision::Duplicate { similarity, .. } => {
                prop_assert_eq!(similarity, 1.0);
            }
            other => prop_assert!(false, "expected Duplicate, got {:?}", other),
        }
    }

    #[test]
    fn prop_dedup_random_is_unique(seed_a in any::<u64>(), seed_b in any::<u64>()) {
        // Two independently random vectors should be Unique (sim ~0.5)
        let mut rng_a = StdRng::seed_from_u64(seed_a);
        let mut rng_b = StdRng::seed_from_u64(seed_b);
        let query = HdcVector::random(&mut rng_a);
        let other = HdcVector::random(&mut rng_b);
        let config = DedupConfig::default_config();
        let candidates = vec![FingerprintCandidate {
            id: 0u64,
            path: "other.md".to_string(),
            fingerprint: other,
        }];
        let result = check_dedup(query, &candidates, &config);
        prop_assert!(
            matches!(result, DedupDecision::Unique),
            "random vectors should be Unique, got {:?}",
            result
        );
    }

    #[test]
    fn prop_similarity_symmetric(seed_a in any::<u64>(), seed_b in any::<u64>()) {
        let mut rng_a = StdRng::seed_from_u64(seed_a);
        let mut rng_b = StdRng::seed_from_u64(seed_b);
        let a = HdcVector::random(&mut rng_a);
        let b = HdcVector::random(&mut rng_b);
        let sim_ab = a.similarity(b);
        let sim_ba = b.similarity(a);
        prop_assert_eq!(sim_ab, sim_ba);
    }

    #[test]
    fn prop_bind_commutative(seed_a in any::<u64>(), seed_b in any::<u64>()) {
        let mut rng_a = StdRng::seed_from_u64(seed_a);
        let mut rng_b = StdRng::seed_from_u64(seed_b);
        let a = HdcVector::random(&mut rng_a);
        let b = HdcVector::random(&mut rng_b);
        prop_assert_eq!(a.bind(b), b.bind(a));
    }
}

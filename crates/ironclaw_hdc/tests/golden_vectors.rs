//! Golden test: verifies seed vector stability.
//! Any change to the seeding algorithm breaks this test.

use ironclaw_hdc::vector::HdcVector;

#[test]
fn golden_seed_vectors_deterministic() {
    // Generate the three reference vectors
    let tag_rust = HdcVector::from_seed("tag", b"rust");
    let path_main = HdcVector::from_seed("path", b"src/main.rs");
    let content_hello = HdcVector::from_seed("content", b"hello world");

    // Verify determinism: same inputs always produce same outputs
    assert_eq!(tag_rust, HdcVector::from_seed("tag", b"rust"));
    assert_eq!(path_main, HdcVector::from_seed("path", b"src/main.rs"));
    assert_eq!(
        content_hello,
        HdcVector::from_seed("content", b"hello world")
    );

    // Verify they're all different from each other
    assert_ne!(tag_rust, path_main);
    assert_ne!(tag_rust, content_hello);
    assert_ne!(path_main, content_hello);
}

#[test]
fn golden_seed_vectors_quasi_orthogonal() {
    let tag_rust = HdcVector::from_seed("tag", b"rust");
    let path_main = HdcVector::from_seed("path", b"src/main.rs");
    let content_hello = HdcVector::from_seed("content", b"hello world");

    // All pairs should have similarity near 0.5 (quasi-orthogonal)
    let sim_tp = tag_rust.similarity(path_main);
    let sim_tc = tag_rust.similarity(content_hello);
    let sim_pc = path_main.similarity(content_hello);

    assert!(
        sim_tp > 0.47 && sim_tp < 0.53,
        "tag-path similarity: {sim_tp}"
    );
    assert!(
        sim_tc > 0.47 && sim_tc < 0.53,
        "tag-content similarity: {sim_tc}"
    );
    assert!(
        sim_pc > 0.47 && sim_pc < 0.53,
        "path-content similarity: {sim_pc}"
    );
}

#[test]
fn golden_seed_vectors_byte_stability() {
    // Pin the first 8 bytes of each reference vector.
    // If these fail, the seeding algorithm has changed and all stored fingerprints
    // in production databases are invalidated.
    let tag_rust = HdcVector::from_seed("tag", b"rust");
    let path_main = HdcVector::from_seed("path", b"src/main.rs");
    let content_hello = HdcVector::from_seed("content", b"hello world");

    let tag_bytes = tag_rust.to_bytes();
    let path_bytes = path_main.to_bytes();
    let content_bytes = content_hello.to_bytes();

    // Golden first-8-byte values (little-endian u64).
    // These were captured from the initial implementation and must never change.
    let tag_first_word = u64::from_le_bytes(tag_bytes[0..8].try_into().unwrap());
    let path_first_word = u64::from_le_bytes(path_bytes[0..8].try_into().unwrap());
    let content_first_word = u64::from_le_bytes(content_bytes[0..8].try_into().unwrap());

    // Pin golden values. Update ONLY if intentionally changing the seeding algorithm.
    assert_eq!(
        tag_first_word, 0xbb9b114ff51f3d97,
        "tag:rust first word changed — seeding algorithm was modified"
    );
    assert_eq!(
        path_first_word, 0xd35b4d5cf8c13b44,
        "path:src/main.rs first word changed — seeding algorithm was modified"
    );
    assert_eq!(
        content_first_word, 0x50e887dcafd86785,
        "content:hello world first word changed — seeding algorithm was modified"
    );
}

#[test]
fn golden_bind_is_self_inverse() {
    let a = HdcVector::from_seed("golden", b"bind-test-a");
    let b = HdcVector::from_seed("golden", b"bind-test-b");
    let bound = a.bind(b);

    // XOR binding is self-inverse
    assert_eq!(bound.bind(b), a);
    assert_eq!(bound.bind(a), b);
}

#[test]
fn golden_permute_inverse() {
    let v = HdcVector::from_seed("golden", b"permute-test");
    let dimension_bits = 10_240;

    // permute(k) composed with permute(D-k) is identity
    for k in [1, 64, 100, 5000, 10239] {
        assert_eq!(
            v.permute(k).permute(dimension_bits - k),
            v,
            "permute inverse failed for k={k}"
        );
    }
}

#[test]
fn golden_from_seed_domain_separation() {
    // Same seed bytes but different domains must produce different vectors
    let a = HdcVector::from_seed("tag", b"shared_seed");
    let b = HdcVector::from_seed("path", b"shared_seed");
    let c = HdcVector::from_seed("content", b"shared_seed");

    assert_ne!(a, b);
    assert_ne!(a, c);
    assert_ne!(b, c);
}

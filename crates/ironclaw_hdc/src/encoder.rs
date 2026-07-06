use crate::bundle::BundleAccumulator;
use crate::codebook::Codebook;
use crate::vector::HdcVector;

/// Input for document encoding.
pub struct DocumentEncodingInput<'a> {
    pub content: &'a str,
    pub tags: &'a [&'a str],
    pub path: &'a str,
}

/// Encode a full document into an HDC fingerprint.
///
/// Formula:
///   content_hv = bundle(byte_trigram(content))
///   tag_hv     = bundle(bind(role:tag, atom(tag)) for each tag)
///   path_hv    = bundle(bind(role:path, atom(segment)) for each path segment)
///   doc_hv     = weighted_bundle(content=5, tags=3, path=2)
///
/// # Examples
///
/// ```
/// use ironclaw_hdc::{encode_document, DocumentEncodingInput};
/// use ironclaw_hdc::codebook::Codebook;
///
/// let mut codebook = Codebook::new();
/// let fingerprint = encode_document(DocumentEncodingInput {
///     content: "Meeting notes from standup",
///     tags: &["meeting", "standup"],
///     path: "journal/2024/01/standup.md",
/// }, &mut codebook);
///
/// // Same input always produces the same fingerprint
/// let mut cb2 = Codebook::new();
/// let fp2 = encode_document(DocumentEncodingInput {
///     content: "Meeting notes from standup",
///     tags: &["meeting", "standup"],
///     path: "journal/2024/01/standup.md",
/// }, &mut cb2);
/// assert_eq!(fingerprint, fp2);
/// ```
pub fn encode_document(input: DocumentEncodingInput<'_>, codebook: &mut Codebook) -> HdcVector {
    let content_hv = encode_content(input.content, codebook);
    let tag_hv = encode_tags(input.tags, codebook);
    let path_hv = encode_path(input.path, codebook);

    let has_content = !input.content.is_empty();
    let has_tags = !input.tags.is_empty();
    let has_path = !input.path.is_empty();

    if !has_content && !has_tags && !has_path {
        return HdcVector::from_seed("special", b"empty-document");
    }

    let mut acc = BundleAccumulator::new();
    if has_content {
        acc.add_weighted(&content_hv, 5);
    }
    if has_tags {
        acc.add_weighted(&tag_hv, 3);
    }
    if has_path {
        acc.add_weighted(&path_hv, 2);
    }
    acc.finalize()
}

/// Encode plain text into an HDC vector (content-only encoding).
pub fn encode_text(text: &str, codebook: &mut Codebook) -> HdcVector {
    encode_document(
        DocumentEncodingInput {
            content: text,
            tags: &[],
            path: "",
        },
        codebook,
    )
}

/// Encode content using byte trigram approach.
///
/// Slides a 3-byte window over UTF-8 bytes. For each trigram [b0, b1, b2]:
///   trigram_hv = atom(b0).permute(2).bind(atom(b1).permute(1)).bind(atom(b2))
fn encode_content(content: &str, codebook: &mut Codebook) -> HdcVector {
    let bytes = content.as_bytes();
    if bytes.is_empty() {
        return HdcVector::zero();
    }

    let mut acc = BundleAccumulator::new();

    if bytes.len() < 3 {
        // Handle short content: unigrams/bigrams
        for (i, &b) in bytes.iter().enumerate() {
            let atom = codebook.get_or_create(&format!("byte:{}", b));
            acc.add(&atom.permute(i));
        }
    } else {
        for window in bytes.windows(3) {
            let b0 = codebook.get_or_create(&format!("byte:{}", window[0]));
            let b1 = codebook.get_or_create(&format!("byte:{}", window[1]));
            let b2 = codebook.get_or_create(&format!("byte:{}", window[2]));
            let trigram_hv = b0.permute(2).bind(b1.permute(1)).bind(b2);
            acc.add(&trigram_hv);
        }
    }

    acc.finalize()
}

/// Encode tags by binding each with a role vector.
fn encode_tags(tags: &[&str], codebook: &mut Codebook) -> HdcVector {
    if tags.is_empty() {
        return HdcVector::zero();
    }

    let role_tag_hv = codebook.get_or_create("role:tag");
    let mut acc = BundleAccumulator::new();

    for &tag in tags {
        let tag_atom = codebook.get_or_create(&format!("tag:{}", tag));
        let bound = role_tag_hv.bind(tag_atom);
        acc.add(&bound);
    }

    acc.finalize()
}

/// Encode path by splitting on '/' and binding segments with position.
fn encode_path(path: &str, codebook: &mut Codebook) -> HdcVector {
    if path.is_empty() {
        return HdcVector::zero();
    }

    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        return HdcVector::zero();
    }

    let role_path_hv = codebook.get_or_create("role:path");
    let mut acc = BundleAccumulator::new();

    for (i, segment) in segments.iter().enumerate() {
        let seg_atom = codebook.get_or_create(&format!("path:{}", segment));
        let bound = role_path_hv.bind(seg_atom).permute(i);
        acc.add(&bound);
    }

    acc.finalize()
}

/// Encode structured fields as role-filler bound pairs.
///
/// Each `(role, value)` pair is encoded as `bind(codebook[role], encode_text(value))`.
/// The results are bundled together. This preserves directional relationships:
/// `encode_structured([("cause", A), ("effect", B)])` differs from
/// `encode_structured([("cause", B), ("effect", A)])`.
///
/// Position encoding: each pair is permuted by `i + 1` (1-indexed) so that
/// even the first pair (index 0) gets a non-identity permutation, ensuring
/// every slot is distinguishable by position alone.
pub fn encode_structured(fields: &[(&str, &str)], codebook: &mut Codebook) -> HdcVector {
    let mut acc = BundleAccumulator::new();

    for (i, &(role, value)) in fields.iter().enumerate() {
        let role_vec = codebook.get_or_create(role);
        let value_vec = encode_text(value, codebook);
        let bound = role_vec.bind(value_vec);
        // Use i + 1 so that position 0 is not a permute(0) identity no-op.
        let permuted = bound.permute(i + 1);
        acc.add(&permuted);
    }

    acc.finalize()
}

/// Encode a causal link with distinct cause and effect roles.
///
/// Shorthand for `encode_structured([("cause", cause), ("effect", effect)])`.
/// The cause and effect are encoded with directional permutation so that
/// `encode_causal_link(A, B) != encode_causal_link(B, A)`.
pub fn encode_causal_link(cause: &str, effect: &str, codebook: &mut Codebook) -> HdcVector {
    encode_structured(&[("cause", cause), ("effect", effect)], codebook)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_text_deterministic() {
        let mut cb1 = Codebook::new();
        let mut cb2 = Codebook::new();
        let v1 = encode_text("hello world", &mut cb1);
        let v2 = encode_text("hello world", &mut cb2);
        assert_eq!(v1, v2);
    }

    #[test]
    fn encode_text_different_texts_differ() {
        let mut cb = Codebook::new();
        let v1 = encode_text("hello world", &mut cb);
        let v2 = encode_text("goodbye world", &mut cb);
        assert_ne!(v1, v2);
    }

    #[test]
    fn encode_text_similar_texts_are_closer() {
        let mut cb = Codebook::new();
        let v1 = encode_text("the quick brown fox", &mut cb);
        let v2 = encode_text("the quick brown dog", &mut cb);
        let v3 = encode_text("completely unrelated content here", &mut cb);
        // v1 and v2 share most trigrams, should be more similar than v1 and v3
        assert!(v1.similarity(v2) > v1.similarity(v3));
    }

    #[test]
    fn encode_document_empty_returns_special() {
        let mut cb = Codebook::new();
        let result = encode_document(
            DocumentEncodingInput {
                content: "",
                tags: &[],
                path: "",
            },
            &mut cb,
        );
        let expected = HdcVector::from_seed("special", b"empty-document");
        assert_eq!(result, expected);
    }

    #[test]
    fn encode_document_with_tags_influences_result() {
        let mut cb = Codebook::new();
        // Tags-only documents should differ based on tag content
        let v_tags_a = encode_document(
            DocumentEncodingInput {
                content: "",
                tags: &["rust", "hdc"],
                path: "",
            },
            &mut cb,
        );
        let v_tags_b = encode_document(
            DocumentEncodingInput {
                content: "",
                tags: &["python", "ml"],
                path: "",
            },
            &mut cb,
        );
        assert_ne!(v_tags_a, v_tags_b);
        // Both should differ from the empty-document sentinel
        let empty = HdcVector::from_seed("special", b"empty-document");
        assert_ne!(v_tags_a, empty);
    }

    #[test]
    fn encode_document_with_path_influences_result() {
        let mut cb = Codebook::new();
        // Path-only documents should differ based on path content
        let v_path_a = encode_document(
            DocumentEncodingInput {
                content: "",
                tags: &[],
                path: "src/lib.rs",
            },
            &mut cb,
        );
        let v_path_b = encode_document(
            DocumentEncodingInput {
                content: "",
                tags: &[],
                path: "tests/main.rs",
            },
            &mut cb,
        );
        assert_ne!(v_path_a, v_path_b);
        let empty = HdcVector::from_seed("special", b"empty-document");
        assert_ne!(v_path_a, empty);
    }

    #[test]
    fn encode_document_same_tags_are_similar() {
        let mut cb = Codebook::new();
        let v1 = encode_document(
            DocumentEncodingInput {
                content: "content one",
                tags: &["rust", "hdc"],
                path: "src/a.rs",
            },
            &mut cb,
        );
        let v2 = encode_document(
            DocumentEncodingInput {
                content: "content two",
                tags: &["rust", "hdc"],
                path: "src/b.rs",
            },
            &mut cb,
        );
        let v3 = encode_document(
            DocumentEncodingInput {
                content: "content three",
                tags: &["python", "ml"],
                path: "lib/c.py",
            },
            &mut cb,
        );
        // v1 and v2 share tags, should be more similar than v1 and v3
        assert!(v1.similarity(v2) > v1.similarity(v3));
    }

    #[test]
    fn encode_content_short_input() {
        let mut cb = Codebook::new();
        // Single byte
        let v1 = encode_text("a", &mut cb);
        assert_ne!(v1, HdcVector::zero());
        // Two bytes
        let v2 = encode_text("ab", &mut cb);
        assert_ne!(v2, HdcVector::zero());
        assert_ne!(v1, v2);
    }

    #[test]
    fn encode_path_order_matters() {
        let mut cb = Codebook::new();
        let v1 = encode_document(
            DocumentEncodingInput {
                content: "",
                tags: &[],
                path: "src/lib/mod.rs",
            },
            &mut cb,
        );
        let v2 = encode_document(
            DocumentEncodingInput {
                content: "",
                tags: &[],
                path: "lib/src/mod.rs",
            },
            &mut cb,
        );
        // Different path orderings should produce different vectors
        assert_ne!(v1, v2);
    }

    #[test]
    fn encode_text_unicode_works() {
        let mut cb = Codebook::new();
        let v = encode_text("cafe\u{0301}", &mut cb);
        assert_ne!(v, HdcVector::zero());
    }

    #[test]
    fn encode_document_content_dominates() {
        let mut cb = Codebook::new();
        let v1 = encode_document(
            DocumentEncodingInput {
                content: "rust async programming with tokio",
                tags: &["rust", "async"],
                path: "src/main.rs",
            },
            &mut cb,
        );
        let v2 = encode_document(
            DocumentEncodingInput {
                content: "french cooking recipes for beginners",
                tags: &["rust", "async"],
                path: "src/main.rs",
            },
            &mut cb,
        );
        // Same tags/path, different content → similarity < 0.85 (content has weight 5/10)
        let sim = v1.similarity(v2);
        assert!(
            sim < 0.85,
            "same tags/path but different content should have sim < 0.85, got {sim}"
        );
    }

    #[test]
    fn encode_document_tags_contribute() {
        let mut cb = Codebook::new();
        let v1 = encode_document(
            DocumentEncodingInput {
                content: "the quick brown fox jumps over the lazy dog",
                tags: &["rust", "hdc"],
                path: "notes/a.md",
            },
            &mut cb,
        );
        let v2 = encode_document(
            DocumentEncodingInput {
                content: "the quick brown fox jumps over the lazy dog",
                tags: &["python", "ml"],
                path: "notes/a.md",
            },
            &mut cb,
        );
        // Same content/path, different tags → similarity < 0.9
        let sim = v1.similarity(v2);
        assert!(
            sim < 0.9,
            "different tags should shift result, got sim {sim}"
        );
    }

    #[test]
    fn encode_document_path_contributes() {
        let mut cb = Codebook::new();
        let v1 = encode_document(
            DocumentEncodingInput {
                content: "the quick brown fox jumps over the lazy dog",
                tags: &["rust"],
                path: "src/lib.rs",
            },
            &mut cb,
        );
        let v2 = encode_document(
            DocumentEncodingInput {
                content: "the quick brown fox jumps over the lazy dog",
                tags: &["rust"],
                path: "tests/integration/main.rs",
            },
            &mut cb,
        );
        // Same content/tags, different path → similarity < 0.95
        let sim = v1.similarity(v2);
        assert!(
            sim < 0.95,
            "different path should shift result, got sim {sim}"
        );
    }

    #[test]
    fn encode_document_long_content() {
        let mut cb = Codebook::new();
        // Use 10KB (realistic document size) rather than 100KB
        let long_content = "the quick brown fox jumps over the lazy dog. ".repeat(225); // ~10KB
        let v = encode_document(
            DocumentEncodingInput {
                content: &long_content,
                tags: &["test"],
                path: "big.md",
            },
            &mut cb,
        );
        assert_ne!(v, HdcVector::zero());
        // Performance assertion deferred to benchmarks (debug builds are much slower)
    }

    #[test]
    fn encode_text_similar_text() {
        let mut cb = Codebook::new();
        let v1 = encode_text("rust async tokio runtime", &mut cb);
        let v2 = encode_text("rust async runtime tokio", &mut cb);
        let sim = v1.similarity(v2);
        assert!(
            sim > 0.6,
            "similar texts should have similarity > 0.6, got {sim}"
        );
    }

    #[test]
    fn encode_text_dissimilar_text() {
        let mut cb = Codebook::new();
        let v1 = encode_text("rust async tokio", &mut cb);
        let v2 = encode_text("french cooking recipes", &mut cb);
        let sim = v1.similarity(v2);
        assert!(
            (0.45..0.55).contains(&sim),
            "dissimilar texts should have similarity ~0.5, got {sim}"
        );
    }

    #[test]
    fn encode_text_empty() {
        let mut cb = Codebook::new();
        let v = encode_text("", &mut cb);
        // Empty text returns the special empty-document vector, not zero
        let expected = HdcVector::from_seed("special", b"empty-document");
        assert_eq!(v, expected);
    }

    #[test]
    fn encode_text_matches_content_component() {
        let mut cb = Codebook::new();
        let text_only = encode_text("hello world this is a test", &mut cb);
        let doc_content_only = encode_document(
            DocumentEncodingInput {
                content: "hello world this is a test",
                tags: &[],
                path: "",
            },
            &mut cb,
        );
        // encode_text with no tags/path should equal content-only document encoding
        assert_eq!(text_only, doc_content_only);
    }

    #[test]
    fn encode_structured_deterministic() {
        // Same fields in the same order must always produce the identical vector.
        let fields = &[("subject", "meeting"), ("topic", "quarterly review")];
        let mut cb1 = Codebook::new();
        let mut cb2 = Codebook::new();
        let v1 = encode_structured(fields, &mut cb1);
        let v2 = encode_structured(fields, &mut cb2);
        assert_eq!(
            v1, v2,
            "encode_structured must be deterministic across fresh codebooks"
        );
    }

    #[test]
    fn encode_structured_order_matters() {
        // Swapping role-filler pairs must produce a different vector because
        // each pair is permuted by its index (i+1).
        let fields_ab = &[("cause", "fire"), ("effect", "smoke")];
        let fields_ba = &[("effect", "smoke"), ("cause", "fire")];
        let mut cb = Codebook::new();
        let v_ab = encode_structured(fields_ab, &mut cb);
        let v_ba = encode_structured(fields_ba, &mut cb);
        assert_ne!(
            v_ab, v_ba,
            "swapping role-filler order must produce a different vector"
        );
    }

    #[test]
    fn encode_causal_link_direction_matters() {
        // encode_causal_link(A, B) must differ from encode_causal_link(B, A)
        // because cause and effect are distinct roles.
        let mut cb = Codebook::new();
        let forward = encode_causal_link("fire", "smoke", &mut cb);
        let backward = encode_causal_link("smoke", "fire", &mut cb);
        assert_ne!(
            forward, backward,
            "reversing cause/effect must produce a different vector"
        );
    }

    #[test]
    fn encode_causal_link_equals_structured() {
        // encode_causal_link(A, B) is defined as a shorthand for
        // encode_structured([("cause", A), ("effect", B)]).
        let cause = "rain";
        let effect = "flooding";
        let mut cb1 = Codebook::new();
        let mut cb2 = Codebook::new();
        let via_causal = encode_causal_link(cause, effect, &mut cb1);
        let via_structured = encode_structured(&[("cause", cause), ("effect", effect)], &mut cb2);
        assert_eq!(
            via_causal, via_structured,
            "encode_causal_link must equal encode_structured with (\"cause\", A), (\"effect\", B)"
        );
    }

    #[test]
    fn structured_encoding_differs_from_flat_text() {
        let mut cb_s = Codebook::new();
        let mut cb_t = Codebook::new();
        let structured = encode_structured(&[("cause", "X"), ("effect", "Y")], &mut cb_s);
        let flat = encode_text("cause X effect Y", &mut cb_t);
        assert_ne!(
            structured, flat,
            "structured role-filler encoding must differ from flat text concatenation"
        );
    }

    #[test]
    fn causal_direction_matters() {
        let mut cb = Codebook::new();
        let forward = encode_causal_link("A causes B", "B is effect", &mut cb);
        let reversed = encode_causal_link("B is effect", "A causes B", &mut cb);
        assert_ne!(
            forward, reversed,
            "swapping cause and effect text must produce a different vector"
        );
    }

    #[test]
    fn role_filler_unbinding() {
        // Binding a role-filler composite with the role vector should recover
        // something close to the filler vector, because bind is self-inverse:
        //   bind(role, filler).bind(role) == filler
        let mut cb = Codebook::new();
        let role_vec = cb.get_or_create("cause");
        let filler_vec = encode_text("fire started", &mut cb);
        let composite = role_vec.bind(filler_vec);
        let recovered = composite.bind(role_vec);
        assert_eq!(
            recovered, filler_vec,
            "unbinding with the role vector must exactly recover the filler"
        );
    }

    #[test]
    fn short_input_empty() {
        let mut cb = Codebook::new();
        let v = encode_text("", &mut cb);
        // Empty text produces the special empty-document sentinel, not a panic
        let expected = HdcVector::from_seed("special", b"empty-document");
        assert_eq!(
            v, expected,
            "empty input must produce the empty-document sentinel"
        );
    }

    #[test]
    fn short_input_one_byte() {
        let mut cb = Codebook::new();
        let v = encode_text("a", &mut cb);
        assert_ne!(
            v,
            HdcVector::zero(),
            "single-byte input must produce a non-zero vector"
        );
        // Must be deterministic
        let mut cb2 = Codebook::new();
        let v2 = encode_text("a", &mut cb2);
        assert_eq!(v, v2, "single-byte encoding must be deterministic");
    }

    #[test]
    fn short_input_two_bytes() {
        let mut cb = Codebook::new();
        let v = encode_text("ab", &mut cb);
        assert_ne!(
            v,
            HdcVector::zero(),
            "two-byte input must produce a non-zero vector"
        );
        // Must differ from single-byte
        let v_a = encode_text("a", &mut cb);
        assert_ne!(v, v_a, "two-byte input must differ from single-byte input");
        // Must be deterministic
        let mut cb2 = Codebook::new();
        let v2 = encode_text("ab", &mut cb2);
        assert_eq!(v, v2, "two-byte encoding must be deterministic");
    }
}

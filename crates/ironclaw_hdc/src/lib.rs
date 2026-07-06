//! Hyperdimensional computing primitives for memory deduplication and semantic similarity.
//!
//! This crate provides binary hypervector operations (bundle, bind, permute),
//! codebook management, encoding, and nearest-neighbor scanning for use in
//! IronClaw's memory deduplication pipeline.
//!
//! # Quick Start
//!
//! ```
//! use ironclaw_hdc::{encode_document, DocumentEncodingInput, HdcVector};
//! use ironclaw_hdc::codebook::Codebook;
//! use ironclaw_hdc::dedup::{check_dedup, DedupConfig, FingerprintCandidate};
//!
//! // Encode two documents
//! let mut codebook = Codebook::new();
//! let fp1 = encode_document(DocumentEncodingInput {
//!     content: "Rust is a systems programming language",
//!     tags: &["rust", "programming"],
//!     path: "notes/rust.md",
//! }, &mut codebook);
//!
//! let fp2 = encode_document(DocumentEncodingInput {
//!     content: "Rust is a systems programming language focused on safety",
//!     tags: &["rust", "programming"],
//!     path: "notes/rust-v2.md",
//! }, &mut codebook);
//!
//! // Check similarity
//! let similarity = fp1.similarity(fp2);
//! assert!(similarity > 0.7); // highly similar content
//!
//! // Deduplication check
//! let config = DedupConfig::default_config();
//! let candidates = vec![FingerprintCandidate {
//!     id: 1u64,
//!     path: "notes/rust.md".to_string(),
//!     fingerprint: fp1,
//! }];
//! let decision = check_dedup(fp2, &candidates, &config);
//! ```

pub mod bundle;
pub mod cluster;
pub mod codebook;
pub mod dedup;
pub mod encoder;
pub mod error;
pub mod scan;
pub mod vector;

pub use cluster::{Cluster, k_medoids};
pub use encoder::{
    DocumentEncodingInput, encode_causal_link, encode_document, encode_structured, encode_text,
};
pub use error::HdcError;
pub use vector::{BYTE_LEN, DIMENSION_BITS, HdcVector, WORDS};

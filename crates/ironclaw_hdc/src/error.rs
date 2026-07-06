use thiserror::Error;

#[derive(Debug, Error)]
pub enum HdcError {
    #[error("invalid byte length: expected {expected}, got {got}")]
    InvalidBytes { expected: usize, got: usize },

    #[error("invalid decay factor: {value} (must be in (0.0, 1.0) exclusive)")]
    InvalidDecayFactor { value: f32 },

    #[error("invalid dedup config: {reason}")]
    InvalidDedupConfig { reason: String },

    #[error("encoding error: {reason}")]
    EncodingError { reason: String },

    #[error("not found: {key}")]
    NotFound { key: String },

    #[error("empty input: {context}")]
    EmptyInput { context: &'static str },
}

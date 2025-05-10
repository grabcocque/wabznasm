//! Error types for the storage layer

use thiserror::Error;

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Storage layer error types
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] rmp_serde::encode::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] rmp_serde::decode::Error),

    #[error("Arrow error: {0}")]
    Arrow(#[from] arrow2::error::Error),

    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    #[error("Schema mismatch: expected {expected}, got {actual}")]
    SchemaMismatch { expected: String, actual: String },

    #[error("Invalid row index: {index} >= {max}")]
    InvalidRowIndex { index: usize, max: usize },

    #[error("Empty table")]
    EmptyTable,

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("File format error: {0}")]
    FileFormat(String),
}

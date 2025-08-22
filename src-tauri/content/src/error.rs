//! Error types for content operations

use thiserror::Error;

/// Errors that can occur during content operations
#[derive(Error, Debug)]
pub enum ContentError {
    /// Error during content hashing
    #[error("Failed to hash content: {0}")]
    HashingError(String),

    /// Error during content validation
    #[error("Content validation failed: {0}")]
    ValidationError(String),

    /// Error during content migration
    #[error("Content migration failed: {0}")]
    MigrationError(String),

    /// Error during bulk operations
    #[error("Bulk operation failed: {0}")]
    BulkOperationError(String),

    /// Error during content transformation
    #[error("Content transformation failed: {0}")]
    TransformationError(String),

    /// Generic error for unexpected situations
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

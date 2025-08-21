use thiserror::Error;

#[derive(Error, Debug)]
pub enum JsonCacheError {
    #[error("ReDB error: {0}")]
    ReDB(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cache entry not found: {0}")]
    NotFound(String),

    #[error("Cache entry expired: {0}")]
    Expired(String),

    #[error("Invalid cache key: {0}")]
    InvalidKey(String),

    #[error("Cache operation failed: {0}")]
    OperationFailed(String),
}

// Manual conversion implementations for ReDB errors
impl From<redb::Error> for JsonCacheError {
    fn from(err: redb::Error) -> Self {
        JsonCacheError::ReDB(err.to_string())
    }
}

impl From<redb::DatabaseError> for JsonCacheError {
    fn from(err: redb::DatabaseError) -> Self {
        JsonCacheError::ReDB(err.to_string())
    }
}

impl From<redb::TransactionError> for JsonCacheError {
    fn from(err: redb::TransactionError) -> Self {
        JsonCacheError::ReDB(err.to_string())
    }
}

impl From<redb::StorageError> for JsonCacheError {
    fn from(err: redb::StorageError) -> Self {
        JsonCacheError::ReDB(err.to_string())
    }
}

impl From<redb::CommitError> for JsonCacheError {
    fn from(err: redb::CommitError) -> Self {
        JsonCacheError::ReDB(err.to_string())
    }
}

impl From<redb::TableError> for JsonCacheError {
    fn from(err: redb::TableError) -> Self {
        JsonCacheError::ReDB(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, JsonCacheError>;

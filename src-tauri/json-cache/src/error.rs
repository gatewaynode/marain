use thiserror::Error;

#[derive(Error, Debug)]
pub enum JsonCacheError {
    #[error("ReDB error: {0}")]
    ReDB(#[from] redb::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] redb::DatabaseError),
    
    #[error("Transaction error: {0}")]
    Transaction(#[from] redb::TransactionError),
    
    #[error("Storage error: {0}")]
    Storage(#[from] redb::StorageError),
    
    #[error("Commit error: {0}")]
    Commit(#[from] redb::CommitError),
    
    #[error("Table error: {0}")]
    Table(#[from] redb::TableError),
    
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

pub type Result<T> = std::result::Result<T, JsonCacheError>;
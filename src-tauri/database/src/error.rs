use thiserror::Error;

pub type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Schema parsing error: {0}")]
    SchemaParsing(String),
    
    #[error("Table creation error: {0}")]
    TableCreation(String),
    
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    
    #[error("Invalid field type: {0}")]
    InvalidFieldType(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Migration error: {0}")]
    Migration(String),
    
    #[error("Other error: {0}")]
    Other(String),
}
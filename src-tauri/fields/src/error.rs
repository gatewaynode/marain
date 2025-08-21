use thiserror::Error;

pub type Result<T> = std::result::Result<T, FieldsError>;

#[derive(Error, Debug)]
pub enum FieldsError {
    #[error("Field validation error: {0}")]
    Validation(String),
    
    #[error("Invalid field type: {0}")]
    InvalidType(String),
    
    #[error("Field not found: {0}")]
    NotFound(String),
    
    #[error("Cardinality violation: {0}")]
    CardinalityViolation(String),
    
    #[error("Type conversion error: {0}")]
    TypeConversion(String),
    
    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}
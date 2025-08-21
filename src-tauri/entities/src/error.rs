use thiserror::Error;

pub type Result<T> = std::result::Result<T, EntitiesError>;

#[derive(Error, Debug)]
pub enum EntitiesError {
    #[error("Database error: {0}")]
    Database(String),

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

    #[error("YAML parsing error: {0}")]
    YamlParsing(#[from] serde_yaml::Error),

    #[error("SQL execution error: {0}")]
    SqlExecution(String),
}

// Helper to convert from sqlx errors
impl From<sqlx::Error> for EntitiesError {
    fn from(err: sqlx::Error) -> Self {
        EntitiesError::Database(err.to_string())
    }
}

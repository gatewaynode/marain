use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Secure log error: {0}")]
    SecureLogError(String),

    #[error("Hash verification failed")]
    HashVerificationFailed,

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Initialization error: {0}")]
    Initialization(String),
}

pub type Result<T> = std::result::Result<T, UserError>;

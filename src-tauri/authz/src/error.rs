//! Error types for the authorization system.
//!
//! # Security Note
//! Error messages must balance providing useful information for debugging while
//! not leaking sensitive authorization policy details to potential attackers.
//! All detailed error information should be logged securely, while external
//! error messages should be minimal.

use thiserror::Error;

/// Errors that can occur during authorization operations.
///
/// # Security Guidelines
/// - Never include policy details in error messages exposed to users
/// - Log full error details securely for debugging
/// - Return generic "Forbidden" messages externally
/// - Include enough context in logs for security auditing
#[derive(Debug, Error)]
pub enum AuthzError {
    /// Failed to parse a CEDAR policy.
    ///
    /// This typically indicates a syntax error in a `.cedar` policy file.
    #[error("Policy parsing failed: {0}")]
    PolicyParse(String),

    /// Failed to create or validate a CEDAR entity.
    ///
    /// This can occur when entity data is malformed or violates the schema.
    #[error("Entity creation failed: {0}")]
    EntityCreation(String),

    /// Failed to evaluate an authorization request.
    ///
    /// This indicates an error during the policy decision evaluation process.
    #[error("Authorization evaluation failed: {0}")]
    EvaluationError(String),

    /// The schema validation failed.
    ///
    /// This occurs when the CEDAR schema is invalid or incompatible.
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    /// A resource could not be found or accessed.
    ///
    /// This may indicate the resource doesn't exist or the principal lacks
    /// permission to know about its existence.
    #[error("Resource not found or inaccessible")]
    ResourceNotFound,

    /// The principal identity could not be validated.
    ///
    /// This indicates authentication or session validation issues.
    #[error("Invalid or unauthenticated principal")]
    InvalidPrincipal,

    /// An internal error occurred during authorization.
    ///
    /// This is a catch-all for unexpected errors that should be investigated.
    #[error("Internal authorization error: {0}")]
    Internal(String),
}

/// A specialized Result type for authorization operations.
pub type Result<T> = std::result::Result<T, AuthzError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AuthzError::PolicyParse("invalid syntax".to_string());
        assert_eq!(err.to_string(), "Policy parsing failed: invalid syntax");

        let err = AuthzError::ResourceNotFound;
        assert_eq!(err.to_string(), "Resource not found or inaccessible");

        let err = AuthzError::InvalidPrincipal;
        assert_eq!(err.to_string(), "Invalid or unauthenticated principal");
    }

    #[test]
    fn test_error_types() {
        let errors = vec![
            AuthzError::PolicyParse("test".into()),
            AuthzError::EntityCreation("test".into()),
            AuthzError::EvaluationError("test".into()),
            AuthzError::SchemaValidation("test".into()),
            AuthzError::ResourceNotFound,
            AuthzError::InvalidPrincipal,
            AuthzError::Internal("test".into()),
        ];

        // Verify all error variants can be created and formatted
        for err in errors {
            let _ = format!("{}", err);
            let _ = format!("{:?}", err);
        }
    }
}

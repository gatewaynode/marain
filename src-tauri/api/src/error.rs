use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// API Error types
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Entity not found: {0}")]
    EntityNotFound(String),
    
    #[error("Invalid entity type: {0}")]
    InvalidEntityType(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden")]
    Forbidden,
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
    
    #[error("Not implemented")]
    NotImplemented,
}

/// Error response structure for OpenAPI documentation
#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ApiErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    /// Convert error to HTTP status code
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::EntityNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::InvalidEntityType(_) => StatusCode::BAD_REQUEST,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotImplemented => StatusCode::NOT_IMPLEMENTED,
        }
    }
    
    /// Get error code for the error type
    pub fn error_code(&self) -> &str {
        match self {
            ApiError::EntityNotFound(_) => "ENTITY_NOT_FOUND",
            ApiError::InvalidEntityType(_) => "INVALID_ENTITY_TYPE",
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
            ApiError::Unauthorized => "UNAUTHORIZED",
            ApiError::Forbidden => "FORBIDDEN",
            ApiError::BadRequest(_) => "BAD_REQUEST",
            ApiError::InternalError(_) => "INTERNAL_ERROR",
            ApiError::NotImplemented => "NOT_IMPLEMENTED",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ApiErrorResponse {
            error: ErrorDetail {
                code: self.error_code().to_string(),
                message: self.to_string(),
                details: None,
            },
        };
        
        (status, Json(error_response)).into_response()
    }
}

/// Convert database errors to API errors
impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::EntityNotFound("Resource not found".to_string()),
            _ => ApiError::DatabaseError(err.to_string()),
        }
    }
}

/// Convert database errors to API errors
impl From<database::error::DatabaseError> for ApiError {
    fn from(err: database::error::DatabaseError) -> Self {
        ApiError::DatabaseError(err.to_string())
    }
}

/// Result type for API operations
pub type ApiResult<T> = Result<T, ApiError>;
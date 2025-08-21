use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Response for a single entity
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EntityResponse {
    pub id: String,
    pub entity_type: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response for listing entities
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EntityListResponse {
    pub entities: Vec<EntityResponse>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
}

/// Request to create a new entity
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateEntityRequest {
    pub data: serde_json::Value,
}

/// Request to update an existing entity
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateEntityRequest {
    pub data: serde_json::Value,
}

/// Health check response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub database: DatabaseHealth,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub message: String,
}

/// Generic success response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SuccessResponse {
    pub success: bool,
    pub message: String,
}

/// Delete response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteResponse {
    pub success: bool,
    pub id: String,
    pub message: String,
}

/// Pagination parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
        }
    }
}

/// Filter parameters for entity queries
#[derive(Debug, Deserialize)]
pub struct FilterParams {
    pub status: Option<String>,
    pub author: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
}
use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use tracing::info;

use crate::{
    error::ApiResult,
    models::{DatabaseHealth, HealthResponse},
    AppState,
};

/// Health check endpoint
///
/// GET /api/v1/health
#[utoipa::path(
    get,
    path = "/api/v1/health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse),
        (status = 503, description = "Service is unhealthy", body = HealthResponse)
    ),
    tag = "health"
)]
pub async fn health_check(State(state): State<AppState>) -> ApiResult<impl IntoResponse> {
    info!("Health check requested");

    // Check database connectivity
    let db_health = match sqlx::query("SELECT 1")
        .fetch_one(&state.db.get_pool())
        .await
    {
        Ok(_) => DatabaseHealth {
            connected: true,
            message: "Database connection successful".to_string(),
        },
        Err(e) => DatabaseHealth {
            connected: false,
            message: format!("Database connection failed: {}", e),
        },
    };

    let response = HealthResponse {
        status: if db_health.connected {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        version: "1.0.0".to_string(),
        timestamp: Utc::now(),
        database: db_health,
    };

    Ok(Json(response))
}

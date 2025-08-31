//! Authentication handlers for PassKey and Magic Link authentication

use axum::{
    extract::{Path, State},
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower_sessions::Session;
use tracing::{debug, error, info};
use webauthn_rs::prelude::*;

use crate::{error::ApiError, AppState};
use user::auth::{
    passkey::PassKeyManager,
    types::{AuthenticatedUser, AuthenticationMethod, SessionData},
};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PassKeyRegisterStartRequest {
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct PassKeyRegisterStartResponse {
    pub challenge_id: String,
    pub options: CreationChallengeResponse,
}

#[derive(Debug, Deserialize)]
pub struct PassKeyRegisterFinishRequest {
    pub challenge_id: String,
    pub credential: RegisterPublicKeyCredential,
}

#[derive(Debug, Deserialize)]
pub struct PassKeyLoginStartRequest {
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PassKeyLoginStartResponse {
    pub challenge_id: String,
    pub options: RequestChallengeResponse,
}

#[derive(Debug, Deserialize)]
pub struct PassKeyLoginFinishRequest {
    pub challenge_id: String,
    pub credential: PublicKeyCredential,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub user: Option<AuthenticatedUser>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PassKeyCredential {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================================
// PassKey Registration Handlers
// ============================================================================

/// Start PassKey registration process
/// POST /api/v1/auth/passkey/register/start
pub async fn passkey_register_start(
    State(state): State<AppState>,
    Json(req): Json<PassKeyRegisterStartRequest>,
) -> Result<Json<PassKeyRegisterStartResponse>, ApiError> {
    debug!("Starting PassKey registration for user: {}", req.user_id);

    // Get PassKey manager from user manager
    let user_manager = state
        .user_manager
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("User manager not initialized".to_string()))?;

    // Get the PassKey manager with configuration from system config
    // TODO: Get these values from actual configuration once WebAuthn config is added
    let rp_id = std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_origin =
        std::env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| "http://localhost:3030".to_string());

    let passkey_manager = PassKeyManager::new(rp_id, rp_origin)
        .map_err(|e| ApiError::InternalError(format!("Failed to create PassKey manager: {}", e)))?;

    // Start registration
    let (challenge_id, options) = passkey_manager
        .start_registration(user_manager.database(), &req.user_id, &req.username)
        .await
        .map_err(|e| {
            error!("Failed to start PassKey registration: {}", e);
            ApiError::InternalError(format!("Registration failed: {}", e))
        })?;

    info!("PassKey registration started for user: {}", req.user_id);

    Ok(Json(PassKeyRegisterStartResponse {
        challenge_id,
        options,
    }))
}

/// Complete PassKey registration process
/// POST /api/v1/auth/passkey/register/finish
pub async fn passkey_register_finish(
    State(state): State<AppState>,
    Json(req): Json<PassKeyRegisterFinishRequest>,
) -> Result<Json<Value>, ApiError> {
    debug!(
        "Completing PassKey registration with challenge: {}",
        req.challenge_id
    );

    // Get user manager
    let user_manager = state
        .user_manager
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("User manager not initialized".to_string()))?;

    // Get the PassKey manager with configuration from system config
    // TODO: Get these values from actual configuration once WebAuthn config is added
    let rp_id = std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_origin =
        std::env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| "http://localhost:3030".to_string());

    let passkey_manager = PassKeyManager::new(rp_id, rp_origin)
        .map_err(|e| ApiError::InternalError(format!("Failed to create PassKey manager: {}", e)))?;

    // Complete registration
    passkey_manager
        .complete_registration(user_manager.database(), &req.challenge_id, &req.credential)
        .await
        .map_err(|e| {
            error!("Failed to complete PassKey registration: {}", e);
            ApiError::InternalError(format!("Registration failed: {}", e))
        })?;

    info!("PassKey registration completed successfully");

    Ok(Json(json!({
        "success": true,
        "message": "PassKey registered successfully"
    })))
}

// ============================================================================
// PassKey Authentication Handlers
// ============================================================================

/// Start PassKey authentication process
/// POST /api/v1/auth/passkey/login/start
pub async fn passkey_login_start(
    State(state): State<AppState>,
    Json(req): Json<PassKeyLoginStartRequest>,
) -> Result<Json<PassKeyLoginStartResponse>, ApiError> {
    debug!("Starting PassKey authentication");

    // Get user manager
    let user_manager = state
        .user_manager
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("User manager not initialized".to_string()))?;

    // Get the PassKey manager with configuration from system config
    // TODO: Get these values from actual configuration once WebAuthn config is added
    let rp_id = std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_origin =
        std::env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| "http://localhost:3030".to_string());

    let passkey_manager = PassKeyManager::new(rp_id, rp_origin)
        .map_err(|e| ApiError::InternalError(format!("Failed to create PassKey manager: {}", e)))?;

    // Start authentication
    let (challenge_id, options) = passkey_manager
        .start_authentication(user_manager.database(), req.user_id.as_deref())
        .await
        .map_err(|e| {
            error!("Failed to start PassKey authentication: {}", e);
            ApiError::InternalError(format!("Authentication failed: {}", e))
        })?;

    info!("PassKey authentication started");

    Ok(Json(PassKeyLoginStartResponse {
        challenge_id,
        options,
    }))
}

/// Complete PassKey authentication process
/// POST /api/v1/auth/passkey/login/finish
pub async fn passkey_login_finish(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Json(req): Json<PassKeyLoginFinishRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    debug!(
        "Completing PassKey authentication with challenge: {}",
        req.challenge_id
    );

    // Get user manager
    let user_manager = state
        .user_manager
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("User manager not initialized".to_string()))?;

    // Get the PassKey manager with configuration from system config
    // TODO: Get these values from actual configuration once WebAuthn config is added
    let rp_id = std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_origin =
        std::env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| "http://localhost:3030".to_string());

    let passkey_manager = PassKeyManager::new(rp_id, rp_origin)
        .map_err(|e| ApiError::InternalError(format!("Failed to create PassKey manager: {}", e)))?;

    // Complete authentication
    let user = passkey_manager
        .complete_authentication(user_manager.database(), &req.challenge_id, &req.credential)
        .await
        .map_err(|e| {
            error!("Failed to complete PassKey authentication: {}", e);
            ApiError::InternalError(format!("Authentication failed: {}", e))
        })?;

    // Create session data
    let session_data = SessionData {
        user_id: user.id.clone(),
        username: user.username.clone(),
        email: user.email.clone(),
        auth_method: AuthenticationMethod::PassKey,
        ip_address: None, // TODO: Get from request
        user_agent: None, // TODO: Get from request
        created_at: chrono::Utc::now(),
        last_activity: chrono::Utc::now(),
    };

    // Store session data
    session.insert("user", &session_data).await.map_err(|e| {
        error!("Failed to store session data: {}", e);
        ApiError::InternalError("Session storage failed".to_string())
    })?;

    info!("PassKey authentication successful for user: {}", user.id);

    Ok(Json(LoginResponse {
        success: true,
        user: Some(user),
        message: "Authentication successful".to_string(),
    }))
}

// ============================================================================
// Credential Management Handlers
// ============================================================================

/// List user's PassKey credentials
/// GET /api/v1/auth/passkey/credentials
pub async fn list_credentials(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
) -> Result<Json<Vec<PassKeyCredential>>, ApiError> {
    // Get user from session
    let session_data: SessionData = session
        .get("user")
        .await
        .map_err(|e| {
            error!("Failed to get session data: {}", e);
            ApiError::InternalError("Session error".to_string())
        })?
        .ok_or(ApiError::Unauthorized)?;

    debug!(
        "Listing PassKey credentials for user: {}",
        session_data.user_id
    );

    // Get user manager
    let user_manager = state
        .user_manager
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("User manager not initialized".to_string()))?;

    // Query credentials from database
    let query = r#"
        SELECT id, created_at, last_used
        FROM passkey_credentials
        WHERE user_id = ?
        ORDER BY created_at DESC
    "#;

    let credentials = sqlx::query_as::<
        _,
        (
            String,
            chrono::DateTime<chrono::Utc>,
            Option<chrono::DateTime<chrono::Utc>>,
        ),
    >(query)
    .bind(&session_data.user_id)
    .fetch_all(user_manager.database().pool())
    .await
    .map_err(|e| {
        error!("Failed to fetch credentials: {}", e);
        ApiError::InternalError("Database error".to_string())
    })?
    .into_iter()
    .map(|(id, created_at, last_used)| PassKeyCredential {
        id,
        created_at,
        last_used,
    })
    .collect();

    Ok(Json(credentials))
}

/// Delete a PassKey credential
/// DELETE /api/v1/auth/passkey/credentials/{id}
pub async fn delete_credential(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(credential_id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // Get user from session
    let session_data: SessionData = session
        .get("user")
        .await
        .map_err(|e| {
            error!("Failed to get session data: {}", e);
            ApiError::InternalError("Session error".to_string())
        })?
        .ok_or(ApiError::Unauthorized)?;

    debug!(
        "Deleting PassKey credential {} for user: {}",
        credential_id, session_data.user_id
    );

    // Get user manager
    let user_manager = state
        .user_manager
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("User manager not initialized".to_string()))?;

    // Delete credential (only if it belongs to the user)
    let query = r#"
        DELETE FROM passkey_credentials
        WHERE id = ? AND user_id = ?
    "#;

    let result = sqlx::query(query)
        .bind(&credential_id)
        .bind(&session_data.user_id)
        .execute(user_manager.database().pool())
        .await
        .map_err(|e| {
            error!("Failed to delete credential: {}", e);
            ApiError::InternalError("Database error".to_string())
        })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::EntityNotFound("Credential not found".to_string()));
    }

    info!("PassKey credential {} deleted successfully", credential_id);

    Ok(Json(json!({
        "success": true,
        "message": "Credential deleted successfully"
    })))
}

// ============================================================================
// Session Management Handlers
// ============================================================================

/// Logout endpoint
/// POST /api/v1/auth/logout
pub async fn logout(Extension(session): Extension<Session>) -> Result<Json<Value>, ApiError> {
    // Clear session
    session.clear().await;

    Ok(Json(json!({
        "success": true,
        "message": "Logged out successfully"
    })))
}

/// Get current user info
/// GET /api/v1/auth/me
pub async fn get_current_user(
    Extension(session): Extension<Session>,
) -> Result<Json<SessionData>, ApiError> {
    // Get user from session
    let session_data: SessionData = session
        .get("user")
        .await
        .map_err(|e| {
            error!("Failed to get session data: {}", e);
            ApiError::InternalError("Session error".to_string())
        })?
        .ok_or(ApiError::Unauthorized)?;

    Ok(Json(session_data))
}

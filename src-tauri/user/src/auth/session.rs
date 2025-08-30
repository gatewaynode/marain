//! Session management for authentication

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use tower_sessions::Session;
use tracing::{debug, error};

use super::types::{AuthenticatedUser, AuthenticationMethod, SessionData};
use crate::error::{Result, UserError};

/// Session configuration re-export
pub use super::store::SessionConfig;

/// Session keys used for storing data
pub struct SessionKeys;

impl SessionKeys {
    pub const USER_ID: &'static str = "user_id";
    pub const USERNAME: &'static str = "username";
    pub const EMAIL: &'static str = "email";
    pub const AUTH_METHOD: &'static str = "auth_method";
    pub const IP_ADDRESS: &'static str = "ip_address";
    pub const USER_AGENT: &'static str = "user_agent";
    pub const CREATED_AT: &'static str = "created_at";
    pub const LAST_ACTIVITY: &'static str = "last_activity";
}

/// Session manager for handling user sessions
#[derive(Clone)]
pub struct SessionManager {
    #[allow(dead_code)]
    config: SessionConfig,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: SessionConfig) -> Self {
        Self { config }
    }

    /// Create a session for an authenticated user
    pub async fn create_session(
        &self,
        session: &Session,
        user: &AuthenticatedUser,
        auth_method: AuthenticationMethod,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        let now = chrono::Utc::now();

        // Store user data in session
        session
            .insert(SessionKeys::USER_ID, &user.id)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to set user_id: {}", e)))?;

        session
            .insert(SessionKeys::USERNAME, &user.username)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to set username: {}", e)))?;

        session
            .insert(SessionKeys::EMAIL, &user.email)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to set email: {}", e)))?;

        session
            .insert(SessionKeys::AUTH_METHOD, auth_method)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to set auth_method: {}", e)))?;

        if let Some(ip) = ip_address {
            session
                .insert(SessionKeys::IP_ADDRESS, ip)
                .await
                .map_err(|e| {
                    UserError::Configuration(format!("Failed to set ip_address: {}", e))
                })?;
        }

        if let Some(agent) = user_agent {
            session
                .insert(SessionKeys::USER_AGENT, agent)
                .await
                .map_err(|e| {
                    UserError::Configuration(format!("Failed to set user_agent: {}", e))
                })?;
        }

        session
            .insert(SessionKeys::CREATED_AT, now)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to set created_at: {}", e)))?;

        session
            .insert(SessionKeys::LAST_ACTIVITY, now)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to set last_activity: {}", e)))?;

        // Save the session
        session
            .save()
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to save session: {}", e)))?;

        debug!("Session created for user: {}", user.id);
        Ok(())
    }

    /// Get session data from a session
    pub async fn get_session_data(session: &Session) -> Result<Option<SessionData>> {
        // Check if user_id exists (indicates an active session)
        let user_id: Option<String> = session
            .get(SessionKeys::USER_ID)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to get user_id: {}", e)))?;

        let Some(user_id) = user_id else {
            return Ok(None);
        };

        // Get all session data
        let username: String = session
            .get(SessionKeys::USERNAME)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to get username: {}", e)))?
            .unwrap_or_default();

        let email: String = session
            .get(SessionKeys::EMAIL)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to get email: {}", e)))?
            .unwrap_or_default();

        let auth_method: AuthenticationMethod = session
            .get(SessionKeys::AUTH_METHOD)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to get auth_method: {}", e)))?
            .unwrap_or(AuthenticationMethod::PassKey);

        let ip_address: Option<String> = session
            .get(SessionKeys::IP_ADDRESS)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to get ip_address: {}", e)))?;

        let user_agent: Option<String> = session
            .get(SessionKeys::USER_AGENT)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to get user_agent: {}", e)))?;

        let created_at = session
            .get(SessionKeys::CREATED_AT)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to get created_at: {}", e)))?
            .unwrap_or_else(chrono::Utc::now);

        let last_activity = session
            .get(SessionKeys::LAST_ACTIVITY)
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to get last_activity: {}", e)))?
            .unwrap_or_else(chrono::Utc::now);

        Ok(Some(SessionData {
            user_id,
            username,
            email,
            auth_method,
            ip_address,
            user_agent,
            created_at,
            last_activity,
        }))
    }

    /// Update last activity timestamp
    pub async fn update_activity(session: &Session) -> Result<()> {
        session
            .insert(SessionKeys::LAST_ACTIVITY, chrono::Utc::now())
            .await
            .map_err(|e| {
                UserError::Configuration(format!("Failed to update last_activity: {}", e))
            })?;

        session
            .save()
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to save session: {}", e)))?;

        Ok(())
    }

    /// Destroy a session (logout)
    pub async fn destroy_session(session: &Session) -> Result<()> {
        session
            .flush()
            .await
            .map_err(|e| UserError::Configuration(format!("Failed to flush session: {}", e)))?;

        debug!("Session destroyed");
        Ok(())
    }

    /// Check if a session is authenticated
    pub async fn is_authenticated(session: &Session) -> bool {
        session
            .get::<String>(SessionKeys::USER_ID)
            .await
            .unwrap_or(None)
            .is_some()
    }
}

/// Extractor for optional session data
pub struct OptionalSessionData(pub Option<SessionData>);

#[axum::async_trait]
impl<S> FromRequestParts<S> for OptionalSessionData
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        // Extract session using the correct method for tower-sessions 0.14
        use axum::Extension;

        let Extension(session): Extension<Session> = Extension::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let session_data = SessionManager::get_session_data(&session)
            .await
            .map_err(|e| {
                error!("Failed to get session data: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        Ok(OptionalSessionData(session_data))
    }
}

/// Extractor for required session data (returns 401 if not authenticated)
pub struct RequiredSessionData(pub SessionData);

#[axum::async_trait]
impl<S> FromRequestParts<S> for RequiredSessionData
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let OptionalSessionData(session_data) =
            OptionalSessionData::from_request_parts(parts, state).await?;

        match session_data {
            Some(data) => Ok(RequiredSessionData(data)),
            None => Err(StatusCode::UNAUTHORIZED),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_sessions::MemoryStore;

    #[tokio::test]
    async fn test_session_manager() {
        let config = SessionConfig::default();
        let manager = SessionManager::new(config);

        // Create a test session with memory store
        let store = MemoryStore::default();
        let session = Session::new(None, std::sync::Arc::new(store), None);

        let user = AuthenticatedUser {
            id: "test_user".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Create session
        manager
            .create_session(
                &session,
                &user,
                AuthenticationMethod::MagicLink,
                Some("127.0.0.1".to_string()),
                Some("TestAgent/1.0".to_string()),
            )
            .await
            .unwrap();

        // Verify session data
        let session_data = SessionManager::get_session_data(&session).await.unwrap();
        assert!(session_data.is_some());

        let data = session_data.unwrap();
        assert_eq!(data.user_id, "test_user");
        assert_eq!(data.username, "testuser");
        assert_eq!(data.email, "test@example.com");
        assert_eq!(data.auth_method, AuthenticationMethod::MagicLink);
        assert_eq!(data.ip_address, Some("127.0.0.1".to_string()));

        // Test is_authenticated
        assert!(SessionManager::is_authenticated(&session).await);

        // Destroy session
        SessionManager::destroy_session(&session).await.unwrap();
        assert!(!SessionManager::is_authenticated(&session).await);
    }
}

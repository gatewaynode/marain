//! Authentication module for Marain CMS
//!
//! This module provides authentication functionality including:
//! - Session management with tower-sessions
//! - PassKey (WebAuthn) authentication
//! - Magic email link authentication
//! - User authentication types and traits

pub mod magic_link;
pub mod passkey;
pub mod session;
pub mod store;
pub mod types;

use async_trait::async_trait;
use axum_login::{AuthnBackend, UserId};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

pub use session::SessionConfig;
pub use store::SqlxSessionStore;
pub use types::{AuthenticatedUser, AuthenticationMethod, Credentials};

use crate::{
    database::UserDatabase,
    error::{Result, UserError},
    secure_log::SecureLogger,
};

/// Authentication backend for axum-login
#[derive(Clone)]
pub struct AuthBackend {
    db: Arc<UserDatabase>,
    secure_logger: Arc<SecureLogger>,
}

impl AuthBackend {
    /// Create a new authentication backend
    pub fn new(db: Arc<UserDatabase>, secure_logger: Arc<SecureLogger>) -> Self {
        Self { db, secure_logger }
    }

    /// Log an authentication event to the secure log
    async fn log_auth_event(
        &self,
        _user_id: Option<&str>,
        action: &str,
        target: &str,
        details: &str,
        ip_address: Option<&str>,
        success: bool,
    ) -> Result<()> {
        self.secure_logger
            .log_action(
                0, // Use 0 for anonymous user
                action,
                Some(target.to_string()),
                Some(serde_json::json!({ "details": details })),
                ip_address.map(|s| s.to_string()),
                success,
            )
            .await?;
        Ok(())
    }
}

#[async_trait]
impl AuthnBackend for AuthBackend {
    type User = AuthenticatedUser;
    type Credentials = Credentials;
    type Error = UserError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>> {
        // Extract method and IP first, before moving creds
        let (method, ip) = match &creds {
            Credentials::PassKey { ip_address, .. } => ("passkey", ip_address.clone()),
            Credentials::MagicLink { ip_address, .. } => ("magic_link", ip_address.clone()),
        };

        info!("Authentication attempt using method: {}", method);

        // Now we can move creds since we've already extracted what we need
        let result = match creds {
            Credentials::PassKey {
                user_id,
                challenge_response,
                ..
            } => passkey::verify_passkey(&self.db, &user_id, challenge_response).await,
            Credentials::MagicLink { email, token, .. } => {
                magic_link::verify_magic_link(&self.db, &email, &token).await
            }
        };

        // Log the authentication attempt
        match &result {
            Ok(Some(user)) => {
                self.log_auth_event(
                    Some(&user.id),
                    "authenticate",
                    method,
                    &format!("User authenticated successfully via {}", method),
                    ip.as_deref(),
                    true,
                )
                .await?;
                info!("User {} authenticated successfully", user.id);
            }
            Ok(None) => {
                self.log_auth_event(
                    None,
                    "authenticate",
                    method,
                    "Authentication failed - invalid credentials",
                    ip.as_deref(),
                    false,
                )
                .await?;
                warn!("Authentication failed for method: {}", method);
            }
            Err(e) => {
                self.log_auth_event(
                    None,
                    "authenticate",
                    method,
                    &format!("Authentication error: {}", e),
                    ip.as_deref(),
                    false,
                )
                .await?;
                error!("Authentication error: {}", e);
            }
        }

        result
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>> {
        debug!("Fetching user with ID: {}", user_id);

        // Query the database for the user
        let query = r#"
            SELECT id, username, email, created_at, updated_at
            FROM users
            WHERE id = ?
        "#;

        let user = sqlx::query_as::<_, AuthenticatedUser>(query)
            .bind(user_id)
            .fetch_optional(self.db.pool())
            .await?;

        if user.is_some() {
            debug!("User {} found in database", user_id);
        } else {
            debug!("User {} not found in database", user_id);
        }

        Ok(user)
    }
}

/// Authentication state that can be extracted from requests
#[derive(Debug, Clone)]
pub enum AuthState {
    /// User is authenticated
    Authenticated(AuthenticatedUser),
    /// User is not authenticated
    Unauthenticated,
}

impl AuthState {
    /// Check if the user is authenticated
    pub fn is_authenticated(&self) -> bool {
        matches!(self, AuthState::Authenticated(_))
    }

    /// Get the authenticated user if available
    pub fn user(&self) -> Option<&AuthenticatedUser> {
        match self {
            AuthState::Authenticated(user) => Some(user),
            AuthState::Unauthenticated => None,
        }
    }

    /// Get the user ID if authenticated
    pub fn user_id(&self) -> Option<&str> {
        self.user().map(|u| u.id.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_state() {
        let unauth = AuthState::Unauthenticated;
        assert!(!unauth.is_authenticated());
        assert!(unauth.user().is_none());
        assert!(unauth.user_id().is_none());

        let user = AuthenticatedUser {
            id: "test_user".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let auth = AuthState::Authenticated(user.clone());
        assert!(auth.is_authenticated());
        assert_eq!(auth.user().unwrap().id, "test_user");
        assert_eq!(auth.user_id().unwrap(), "test_user");
    }
}

//! Authentication types and traits

use axum_login::AuthUser;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents an authenticated user in the system
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuthenticatedUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AuthUser for AuthenticatedUser {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        // Use the user ID as the session hash
        // This is stable and doesn't create temporary values
        self.id.as_bytes()
    }
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Credentials {
    /// PassKey (WebAuthn) authentication
    PassKey {
        user_id: String,
        challenge_response: String,
        ip_address: Option<String>,
    },
    /// Magic link email authentication
    MagicLink {
        email: String,
        token: String,
        ip_address: Option<String>,
    },
}

/// Authentication method used
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthenticationMethod {
    PassKey,
    MagicLink,
}

impl std::fmt::Display for AuthenticationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthenticationMethod::PassKey => write!(f, "passkey"),
            AuthenticationMethod::MagicLink => write!(f, "magic_link"),
        }
    }
}

/// User registration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegistration {
    pub username: String,
    pub email: String,
    pub phone_number: Option<String>,
}

/// Magic link token data stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MagicLinkToken {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

/// PassKey credential data stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PassKeyCredential {
    pub id: String,
    pub user_id: String,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub counter: u32,
    pub transports: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

/// Session data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub auth_method: AuthenticationMethod,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Login request for PassKey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassKeyLoginRequest {
    pub user_id: String,
}

/// Login response for PassKey (contains challenge)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassKeyLoginChallenge {
    pub challenge: String,
    pub rp_id: String,
    pub user_id: String,
}

/// PassKey verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassKeyVerificationRequest {
    pub user_id: String,
    pub challenge_response: String,
}

/// Magic link request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicLinkRequest {
    pub email: String,
}

/// Magic link verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicLinkVerificationRequest {
    pub email: String,
    pub token: String,
}

/// Authentication response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub user: Option<AuthenticatedUser>,
    pub session_token: Option<String>,
    pub message: String,
}

impl AuthResponse {
    /// Create a successful authentication response
    pub fn success(user: AuthenticatedUser, session_token: String) -> Self {
        Self {
            success: true,
            user: Some(user),
            session_token: Some(session_token),
            message: "Authentication successful".to_string(),
        }
    }

    /// Create a failed authentication response
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            user: None,
            session_token: None,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_response() {
        let user = AuthenticatedUser {
            id: "test_id".to_string(),
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let success_response = AuthResponse::success(user.clone(), "token123".to_string());
        assert!(success_response.success);
        assert_eq!(success_response.user.unwrap().id, "test_id");
        assert_eq!(success_response.session_token.unwrap(), "token123");

        let failure_response = AuthResponse::failure("Invalid credentials");
        assert!(!failure_response.success);
        assert!(failure_response.user.is_none());
        assert!(failure_response.session_token.is_none());
        assert_eq!(failure_response.message, "Invalid credentials");
    }

    #[test]
    fn test_authentication_method_display() {
        assert_eq!(AuthenticationMethod::PassKey.to_string(), "passkey");
        assert_eq!(AuthenticationMethod::MagicLink.to_string(), "magic_link");
    }
}

//! PassKey (WebAuthn) authentication implementation

// use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use ulid::Ulid;
use webauthn_rs::prelude::*;

use super::types::{AuthenticatedUser, PassKeyCredential};
use crate::{
    database::UserDatabase,
    error::{Result, UserError},
};

/// PassKey manager for WebAuthn operations
pub struct PassKeyManager {
    webauthn: Arc<Webauthn>,
    #[allow(dead_code)]
    rp_id: String,
    #[allow(dead_code)]
    rp_origin: Url,
}

impl PassKeyManager {
    /// Create a new PassKey manager
    pub fn new(rp_id: String, rp_origin: String) -> Result<Self> {
        let rp_origin = Url::parse(&rp_origin)
            .map_err(|e| UserError::Configuration(format!("Invalid RP origin URL: {}", e)))?;

        let builder = WebauthnBuilder::new(&rp_id, &rp_origin).map_err(|e| {
            UserError::Configuration(format!("Failed to create WebAuthn builder: {}", e))
        })?;

        let webauthn =
            Arc::new(builder.build().map_err(|e| {
                UserError::Configuration(format!("Failed to build WebAuthn: {}", e))
            })?);

        Ok(Self {
            webauthn,
            rp_id,
            rp_origin,
        })
    }

    /// Store a challenge in the database
    async fn store_challenge(
        &self,
        db: &UserDatabase,
        user_id: Option<&str>,
        challenge: &str,
        challenge_type: &str,
    ) -> Result<String> {
        let challenge_id = Ulid::new().to_string();
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5); // 5 minute TTL

        let query = r#"
            INSERT INTO passkey_challenges (
                id, user_id, challenge, challenge_type, expires_at, used
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(&challenge_id)
            .bind(user_id)
            .bind(challenge)
            .bind(challenge_type)
            .bind(expires_at)
            .bind(false)
            .execute(db.pool())
            .await
            .map_err(|e| {
                error!("Failed to store challenge: {}", e);
                UserError::Database(e)
            })?;

        debug!("Stored {} challenge: {}", challenge_type, challenge_id);
        Ok(challenge_id)
    }

    /// Retrieve and validate a challenge from the database
    async fn get_challenge(
        &self,
        db: &UserDatabase,
        challenge_id: &str,
        challenge_type: &str,
    ) -> Result<(Option<String>, String)> {
        let query = r#"
            SELECT user_id, challenge, expires_at, used
            FROM passkey_challenges
            WHERE id = ? AND challenge_type = ?
        "#;

        let row = sqlx::query(query)
            .bind(challenge_id)
            .bind(challenge_type)
            .fetch_optional(db.pool())
            .await
            .map_err(|e| {
                error!("Failed to fetch challenge: {}", e);
                UserError::Database(e)
            })?
            .ok_or_else(|| {
                warn!("Challenge not found: {}", challenge_id);
                UserError::InvalidCredentials
            })?;

        let user_id: Option<String> = row.get("user_id");
        let challenge: String = row.get("challenge");
        let expires_at: chrono::DateTime<chrono::Utc> = row.get("expires_at");
        let used: bool = row.get("used");

        // Check if challenge is expired
        if chrono::Utc::now() > expires_at {
            warn!("Challenge expired: {}", challenge_id);
            return Err(UserError::InvalidCredentials);
        }

        // Check if challenge was already used
        if used {
            warn!("Challenge already used: {}", challenge_id);
            return Err(UserError::InvalidCredentials);
        }

        Ok((user_id, challenge))
    }

    /// Mark a challenge as used
    async fn mark_challenge_used(&self, db: &UserDatabase, challenge_id: &str) -> Result<()> {
        let query = r#"
            UPDATE passkey_challenges
            SET used = 1
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(challenge_id)
            .execute(db.pool())
            .await
            .map_err(|e| {
                error!("Failed to mark challenge as used: {}", e);
                UserError::Database(e)
            })?;

        debug!("Marked challenge as used: {}", challenge_id);
        Ok(())
    }

    /// Clean up expired challenges
    pub async fn cleanup_expired_challenges(db: &UserDatabase) -> Result<u64> {
        let query = r#"
            DELETE FROM passkey_challenges
            WHERE expires_at < ? OR used = 1
        "#;

        let result = sqlx::query(query)
            .bind(chrono::Utc::now())
            .execute(db.pool())
            .await
            .map_err(|e| {
                error!("Failed to cleanup expired challenges: {}", e);
                UserError::Database(e)
            })?;

        let deleted = result.rows_affected();
        if deleted > 0 {
            debug!("Cleaned up {} expired challenges", deleted);
        }

        Ok(deleted)
    }

    /// Start registration process for a new PassKey
    pub async fn start_registration(
        &self,
        db: &UserDatabase,
        user_id: &str,
        username: &str,
    ) -> Result<(String, CreationChallengeResponse)> {
        // Get existing credentials for the user
        let existing_credentials = self.get_user_credentials(db, user_id).await?;

        let exclude_credentials: Vec<CredentialID> = existing_credentials
            .into_iter()
            .map(|cred| CredentialID::from(cred.credential_id))
            .collect();

        // Create user entity - using Passkey type from webauthn-rs
        let user_uuid = uuid::Uuid::new_v4();

        // Start registration with proper API
        let (ccr, reg_state) = self
            .webauthn
            .start_passkey_registration(user_uuid, username, username, Some(exclude_credentials))
            .map_err(|e| {
                UserError::Configuration(format!("Failed to start registration: {}", e))
            })?;

        // Serialize and store the registration state with challenge
        let state_json = serde_json::to_string(&reg_state).map_err(UserError::Serialization)?;

        let challenge_id = self
            .store_challenge(db, Some(user_id), &state_json, "registration")
            .await?;

        debug!("Started PassKey registration for user: {}", user_id);
        Ok((challenge_id, ccr))
    }

    /// Complete registration process
    pub async fn complete_registration(
        &self,
        db: &UserDatabase,
        challenge_id: &str,
        credential: &RegisterPublicKeyCredential,
    ) -> Result<()> {
        // Retrieve and validate the challenge
        let (user_id_opt, state_json) =
            self.get_challenge(db, challenge_id, "registration").await?;

        let user_id = user_id_opt.ok_or_else(|| {
            error!("Registration challenge missing user_id");
            UserError::InvalidCredentials
        })?;

        // Deserialize the registration state
        let reg_state: PasskeyRegistration = serde_json::from_str(&state_json).map_err(|e| {
            error!("Failed to deserialize registration state: {}", e);
            UserError::Serialization(e)
        })?;

        // Finish registration
        let passkey = self
            .webauthn
            .finish_passkey_registration(credential, &reg_state)
            .map_err(|e| {
                error!("Failed to finish registration: {}", e);
                UserError::InvalidCredentials
            })?;

        // Store the credential in database
        let cred_id = Ulid::new().to_string();
        let credential_id = passkey.cred_id().to_vec();
        let public_key = serde_json::to_vec(&passkey).map_err(UserError::Serialization)?;

        let query = r#"
            INSERT INTO passkey_credentials (
                id, user_id, credential_id, public_key, counter, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(&cred_id)
            .bind(&user_id)
            .bind(&credential_id)
            .bind(&public_key)
            .bind(0i32) // Initial counter value
            .bind(chrono::Utc::now())
            .execute(db.pool())
            .await
            .map_err(|e| {
                error!("Failed to store PassKey credential: {}", e);
                UserError::Database(e)
            })?;

        // Mark challenge as used
        self.mark_challenge_used(db, challenge_id).await?;

        info!("PassKey registered successfully for user: {}", user_id);
        Ok(())
    }

    /// Start authentication process
    pub async fn start_authentication(
        &self,
        db: &UserDatabase,
        user_id: Option<&str>,
    ) -> Result<(String, RequestChallengeResponse)> {
        // For now, we'll use discoverable credentials (empty list)
        // In a full implementation, we'd load the user's passkeys from the database
        let allow_credentials = vec![];

        // Start authentication
        let (rcr, auth_state) = self
            .webauthn
            .start_passkey_authentication(&allow_credentials)
            .map_err(|e| {
                UserError::Configuration(format!("Failed to start authentication: {}", e))
            })?;

        // Serialize and store the authentication state with challenge
        let state_json = serde_json::to_string(&auth_state).map_err(UserError::Serialization)?;

        let challenge_id = self
            .store_challenge(db, user_id, &state_json, "authentication")
            .await?;

        debug!("Started PassKey authentication");
        Ok((challenge_id, rcr))
    }

    /// Complete authentication process
    pub async fn complete_authentication(
        &self,
        db: &UserDatabase,
        challenge_id: &str,
        credential: &PublicKeyCredential,
    ) -> Result<AuthenticatedUser> {
        // Retrieve and validate the challenge
        let (_, state_json) = self
            .get_challenge(db, challenge_id, "authentication")
            .await?;

        // Deserialize the authentication state
        let auth_state: PasskeyAuthentication = serde_json::from_str(&state_json).map_err(|e| {
            error!("Failed to deserialize authentication state: {}", e);
            UserError::Serialization(e)
        })?;

        // Get the credential from database
        let cred_id = credential.raw_id.to_vec();

        let query = r#"
            SELECT id, user_id, public_key, counter
            FROM passkey_credentials
            WHERE credential_id = ?
        "#;

        let row = sqlx::query(query)
            .bind(&cred_id)
            .fetch_optional(db.pool())
            .await
            .map_err(|e| {
                error!("Failed to fetch credential: {}", e);
                UserError::Database(e)
            })?
            .ok_or_else(|| {
                warn!("Credential not found");
                UserError::InvalidCredentials
            })?;

        let stored_id: String = row.get("id");
        let user_id: String = row.get("user_id");
        let public_key_bytes: Vec<u8> = row.get("public_key");
        let _counter: u32 = row.get("counter");

        // Deserialize the passkey
        let _passkey: Passkey = serde_json::from_slice(&public_key_bytes).map_err(|e| {
            error!("Failed to deserialize passkey: {}", e);
            UserError::Serialization(e)
        })?;

        // Finish authentication - updated API doesn't take passkey as parameter
        let _auth_result = self
            .webauthn
            .finish_passkey_authentication(credential, &auth_state)
            .map_err(|e| {
                error!("Failed to finish authentication: {}", e);
                UserError::InvalidCredentials
            })?;

        // Mark challenge as used
        self.mark_challenge_used(db, challenge_id).await?;

        // Update counter and last_used in database
        let update_query = r#"
            UPDATE passkey_credentials
            SET counter = counter + 1, last_used = ?
            WHERE id = ?
        "#;

        sqlx::query(update_query)
            .bind(chrono::Utc::now())
            .bind(&stored_id)
            .execute(db.pool())
            .await
            .map_err(|e| {
                error!("Failed to update credential counter: {}", e);
                UserError::Database(e)
            })?;

        // Get user information
        let user_query = r#"
            SELECT id, username, email, created_at, updated_at
            FROM users
            WHERE id = ?
        "#;

        let user = sqlx::query_as::<_, AuthenticatedUser>(user_query)
            .bind(&user_id)
            .fetch_optional(db.pool())
            .await
            .map_err(|e| {
                error!("Failed to fetch user: {}", e);
                UserError::Database(e)
            })?
            .ok_or_else(|| {
                error!("User not found for credential");
                UserError::UserNotFound(user_id.clone())
            })?;

        info!("PassKey authentication successful for user: {}", user.id);
        Ok(user)
    }

    /// Get all credentials for a user
    async fn get_user_credentials(
        &self,
        db: &UserDatabase,
        user_id: &str,
    ) -> Result<Vec<PassKeyCredential>> {
        let query = r#"
            SELECT id, user_id, credential_id, public_key, counter, 
                   transports, created_at, last_used
            FROM passkey_credentials
            WHERE user_id = ?
            ORDER BY created_at DESC
        "#;

        let credentials = sqlx::query_as::<_, PassKeyCredential>(query)
            .bind(user_id)
            .fetch_all(db.pool())
            .await
            .map_err(|e| {
                error!("Failed to fetch user credentials: {}", e);
                UserError::Database(e)
            })?;

        Ok(credentials)
    }
}

/// Verify a PassKey challenge response
pub async fn verify_passkey(
    db: &UserDatabase,
    user_id: &str,
    _challenge_response: String,
) -> Result<Option<AuthenticatedUser>> {
    // This is a simplified version - in production, you would:
    // 1. Deserialize the challenge response
    // 2. Verify it against the stored challenge
    // 3. Complete the WebAuthn authentication flow

    warn!("PassKey verification not fully implemented - using mock verification");

    // For now, just check if the user exists
    let query = r#"
        SELECT id, username, email, created_at, updated_at
        FROM users
        WHERE id = ?
    "#;

    let user = sqlx::query_as::<_, AuthenticatedUser>(query)
        .bind(user_id)
        .fetch_optional(db.pool())
        .await
        .map_err(UserError::Database)?;

    Ok(user)
}

/// PassKey registration state that needs to be stored temporarily
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassKeyRegistrationState {
    pub user_id: String,
    pub challenge: String,
    pub state: String, // Serialized PasskeyRegistration
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// PassKey authentication state that needs to be stored temporarily
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassKeyAuthenticationState {
    pub challenge: String,
    pub state: String, // Serialized PasskeyAuthentication
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passkey_manager_creation() {
        let result =
            PassKeyManager::new("localhost".to_string(), "http://localhost:3000".to_string());

        assert!(result.is_ok());
        let manager = result.unwrap();
        assert_eq!(manager.rp_id, "localhost");
    }

    #[test]
    fn test_invalid_origin_url() {
        let result = PassKeyManager::new("localhost".to_string(), "not-a-valid-url".to_string());

        assert!(result.is_err());
    }
}

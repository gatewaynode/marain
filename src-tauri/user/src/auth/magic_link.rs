//! Magic email link authentication implementation

use base64::{engine::general_purpose::URL_SAFE_NO_PAD as BASE64, Engine};
use chrono::{Duration, Utc};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tracing::{debug, error, info, warn};
use ulid::Ulid;

use super::types::AuthenticatedUser;
use crate::{
    database::UserDatabase,
    error::{Result, UserError},
};

/// Magic link email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicLinkConfig {
    /// SMTP server host
    pub smtp_host: String,
    /// SMTP server port
    pub smtp_port: u16,
    /// SMTP username
    pub smtp_username: String,
    /// SMTP password
    pub smtp_password: String,
    /// From email address
    pub from_email: String,
    /// From name
    pub from_name: String,
    /// Token expiry in minutes
    pub token_expiry_minutes: i64,
    /// Base URL for magic links
    pub base_url: String,
}

impl Default for MagicLinkConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 1025, // MailHog default port for development
            smtp_username: String::new(),
            smtp_password: String::new(),
            from_email: "noreply@marain-cms.local".to_string(),
            from_name: "Marain CMS".to_string(),
            token_expiry_minutes: 15,
            base_url: "http://localhost:3000".to_string(),
        }
    }
}

/// Magic link manager for email-based authentication
pub struct MagicLinkManager {
    config: MagicLinkConfig,
}

impl MagicLinkManager {
    /// Create a new magic link manager
    pub fn new(config: MagicLinkConfig) -> Self {
        Self { config }
    }

    /// Generate a secure random token
    fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill(&mut bytes);
        BASE64.encode(bytes)
    }

    /// Send a magic link email
    pub async fn send_magic_link(&self, db: &UserDatabase, email: &str) -> Result<()> {
        // Check if user exists
        let user_query = r#"
            SELECT id, username
            FROM users
            WHERE email = ?
        "#;

        let user = sqlx::query(user_query)
            .bind(email)
            .fetch_optional(db.pool())
            .await
            .map_err(UserError::Database)?;

        let Some(user) = user else {
            // Don't reveal whether the email exists or not
            info!("Magic link requested for non-existent email: {}", email);
            return Ok(());
        };

        let user_id: String = user.get("id");
        let username: String = user.get("username");

        // Generate token
        let token = Self::generate_token();
        let token_id = Ulid::new().to_string();
        let expires_at = Utc::now() + Duration::minutes(self.config.token_expiry_minutes);

        // Store token in database
        let insert_query = r#"
            INSERT INTO magic_link_tokens (
                id, user_id, token, email, expires_at, used, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(insert_query)
            .bind(&token_id)
            .bind(&user_id)
            .bind(&token)
            .bind(email)
            .bind(expires_at)
            .bind(false)
            .bind(Utc::now())
            .execute(db.pool())
            .await
            .map_err(|e| {
                error!("Failed to store magic link token: {}", e);
                UserError::Database(e)
            })?;

        // Send email
        self.send_email(email, &username, &token).await?;

        info!("Magic link sent to: {}", email);
        Ok(())
    }

    /// Send the actual email
    async fn send_email(&self, to_email: &str, username: &str, token: &str) -> Result<()> {
        let magic_link = format!(
            "{}/auth/verify-magic-link?token={}&email={}",
            self.config.base_url,
            urlencoding::encode(token),
            urlencoding::encode(to_email)
        );

        let email_body = format!(
            r#"
            <html>
            <body>
                <h2>Hello {username},</h2>
                <p>You requested a magic link to sign in to Marain CMS.</p>
                <p>Click the link below to sign in:</p>
                <p><a href="{magic_link}" style="display: inline-block; padding: 10px 20px; background-color: #007bff; color: white; text-decoration: none; border-radius: 5px;">Sign In</a></p>
                <p>Or copy and paste this URL into your browser:</p>
                <p>{magic_link}</p>
                <p>This link will expire in {} minutes.</p>
                <p>If you didn't request this link, you can safely ignore this email.</p>
                <br>
                <p>Best regards,<br>The Marain CMS Team</p>
            </body>
            </html>
            "#,
            self.config.token_expiry_minutes
        );

        let email = Message::builder()
            .from(
                format!("{} <{}>", self.config.from_name, self.config.from_email)
                    .parse()
                    .map_err(|e| UserError::Configuration(format!("Invalid from email: {}", e)))?,
            )
            .to(to_email
                .parse()
                .map_err(|e| UserError::Configuration(format!("Invalid to email: {}", e)))?)
            .subject("Your Marain CMS Magic Link")
            .header(ContentType::TEXT_HTML)
            .body(email_body)
            .map_err(|e| UserError::Configuration(format!("Failed to build email: {}", e)))?;

        // Create SMTP transport
        let mailer = if self.config.smtp_username.is_empty() {
            // No authentication (for development with MailHog)
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.config.smtp_host)
                .port(self.config.smtp_port)
                .build()
        } else {
            // With authentication (for production)
            let creds = Credentials::new(
                self.config.smtp_username.clone(),
                self.config.smtp_password.clone(),
            );

            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.smtp_host)
                .map_err(|e| UserError::Configuration(format!("Invalid SMTP host: {}", e)))?
                .port(self.config.smtp_port)
                .credentials(creds)
                .build()
        };

        // Send email
        mailer.send(email).await.map_err(|e| {
            error!("Failed to send email: {}", e);
            UserError::Configuration(format!("Failed to send email: {}", e))
        })?;

        debug!("Magic link email sent to: {}", to_email);
        Ok(())
    }

    /// Verify a magic link token
    pub async fn verify_token(
        db: &UserDatabase,
        email: &str,
        token: &str,
    ) -> Result<Option<AuthenticatedUser>> {
        // Get token from database
        let query = r#"
            SELECT id, user_id, expires_at, used
            FROM magic_link_tokens
            WHERE email = ? AND token = ?
        "#;

        let token_row = sqlx::query(query)
            .bind(email)
            .bind(token)
            .fetch_optional(db.pool())
            .await
            .map_err(UserError::Database)?;

        let Some(token_row) = token_row else {
            warn!("Invalid magic link token for email: {}", email);
            return Ok(None);
        };

        let token_id: String = token_row.get("id");
        let user_id: String = token_row.get("user_id");
        let expires_at: chrono::DateTime<Utc> = token_row.get("expires_at");
        let used: bool = token_row.get("used");

        // Check if token is expired
        if expires_at < Utc::now() {
            warn!("Expired magic link token for email: {}", email);
            return Ok(None);
        }

        // Check if token was already used
        if used {
            warn!("Already used magic link token for email: {}", email);
            return Ok(None);
        }

        // Mark token as used
        let update_query = r#"
            UPDATE magic_link_tokens
            SET used = true
            WHERE id = ?
        "#;

        sqlx::query(update_query)
            .bind(&token_id)
            .execute(db.pool())
            .await
            .map_err(UserError::Database)?;

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
            .map_err(UserError::Database)?;

        if let Some(ref user) = user {
            info!("Magic link authentication successful for user: {}", user.id);
        }

        Ok(user)
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired_tokens(db: &UserDatabase) -> Result<()> {
        let query = r#"
            DELETE FROM magic_link_tokens
            WHERE expires_at < ? OR used = true
        "#;

        let deleted = sqlx::query(query)
            .bind(Utc::now())
            .execute(db.pool())
            .await
            .map_err(UserError::Database)?;

        if deleted.rows_affected() > 0 {
            debug!(
                "Cleaned up {} expired/used magic link tokens",
                deleted.rows_affected()
            );
        }

        Ok(())
    }
}

/// Verify a magic link
pub async fn verify_magic_link(
    db: &UserDatabase,
    email: &str,
    token: &str,
) -> Result<Option<AuthenticatedUser>> {
    MagicLinkManager::verify_token(db, email, token).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let token1 = MagicLinkManager::generate_token();
        let token2 = MagicLinkManager::generate_token();

        // Tokens should be unique
        assert_ne!(token1, token2);

        // Tokens should be URL-safe base64
        assert!(token1
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));

        // Tokens should be reasonably long
        assert!(token1.len() >= 32);
    }

    #[test]
    fn test_magic_link_config_default() {
        let config = MagicLinkConfig::default();

        assert_eq!(config.smtp_host, "localhost");
        assert_eq!(config.smtp_port, 1025);
        assert_eq!(config.token_expiry_minutes, 15);
        assert_eq!(config.base_url, "http://localhost:3000");
    }
}

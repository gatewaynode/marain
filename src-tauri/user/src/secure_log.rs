use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::error::{Result, UserError};

/// Represents a single log entry in the secure audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureLogEntry {
    /// Unique ID for this log entry
    pub id: String,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// User ID who performed the action (0 for system)
    pub user_id: i64,
    /// Type of action performed
    pub action: String,
    /// Target of the action (e.g., user ID being modified)
    pub target: Option<String>,
    /// Additional details about the action
    pub details: Option<serde_json::Value>,
    /// IP address of the request
    pub ip_address: Option<String>,
    /// Result of the action (success/failure)
    pub result: String,
    /// Hash of the previous log entry for chain verification
    pub previous_hash: String,
    /// Hash of this log entry
    pub entry_hash: String,
}

impl SecureLogEntry {
    /// Create a new log entry
    pub fn new(
        user_id: i64,
        action: String,
        target: Option<String>,
        details: Option<serde_json::Value>,
        ip_address: Option<String>,
        result: String,
        previous_hash: String,
    ) -> Self {
        let id = ulid::Ulid::new().to_string();
        let timestamp = Utc::now();

        let mut entry = Self {
            id: id.clone(),
            timestamp,
            user_id,
            action,
            target,
            details,
            ip_address,
            result,
            previous_hash,
            entry_hash: String::new(),
        };

        // Calculate the hash for this entry
        entry.entry_hash = entry.calculate_hash();
        entry
    }

    /// Calculate the SHA256 hash of this log entry
    fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();

        // Include all fields except the entry_hash itself
        hasher.update(self.id.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(self.user_id.to_string().as_bytes());
        hasher.update(self.action.as_bytes());

        if let Some(ref target) = self.target {
            hasher.update(target.as_bytes());
        }

        if let Some(ref details) = self.details {
            hasher.update(details.to_string().as_bytes());
        }

        if let Some(ref ip) = self.ip_address {
            hasher.update(ip.as_bytes());
        }

        hasher.update(self.result.as_bytes());
        hasher.update(self.previous_hash.as_bytes());

        hex::encode(hasher.finalize())
    }

    /// Verify the hash of this log entry
    pub fn verify_hash(&self) -> bool {
        self.entry_hash == self.calculate_hash()
    }
}

/// Configuration for the secure logger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureLogConfig {
    /// Path to the secure log file
    pub log_path: PathBuf,
    /// Maximum size of the log file before rotation (in MB)
    pub max_size_mb: u64,
    /// Number of rotated log files to keep
    pub max_rotations: u32,
    /// Whether to enable real-time hash verification
    pub enable_verification: bool,
}

impl Default for SecureLogConfig {
    fn default() -> Self {
        Self {
            log_path: PathBuf::from("data/user-backend/secure.log"),
            max_size_mb: 100,
            max_rotations: 10,
            enable_verification: true,
        }
    }
}

/// Secure logger for user actions with cryptographic verification
pub struct SecureLogger {
    config: SecureLogConfig,
    last_hash: Arc<RwLock<String>>,
    file_lock: Arc<RwLock<()>>,
}

impl SecureLogger {
    /// Create a new secure logger
    pub fn new(config: SecureLogConfig) -> Result<Self> {
        // Ensure the directory exists
        if let Some(parent) = config.log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Get the last hash from the existing log file if it exists
        let last_hash = if config.log_path.exists() {
            Self::get_last_hash(&config.log_path)?
        } else {
            // Genesis hash for the first entry
            String::from("0000000000000000000000000000000000000000000000000000000000000000")
        };

        Ok(Self {
            config,
            last_hash: Arc::new(RwLock::new(last_hash)),
            file_lock: Arc::new(RwLock::new(())),
        })
    }

    /// Log a user action
    pub async fn log_action(
        &self,
        user_id: i64,
        action: &str,
        target: Option<String>,
        details: Option<serde_json::Value>,
        ip_address: Option<String>,
        success: bool,
    ) -> Result<()> {
        let result = if success { "success" } else { "failure" };

        // Get the previous hash
        let previous_hash = {
            let hash_guard = self.last_hash.read().await;
            hash_guard.clone()
        };

        // Create the log entry
        let entry = SecureLogEntry::new(
            user_id,
            action.to_string(),
            target,
            details,
            ip_address,
            result.to_string(),
            previous_hash,
        );

        // Write to file
        self.write_entry(&entry).await?;

        // Update the last hash
        {
            let mut hash_guard = self.last_hash.write().await;
            *hash_guard = entry.entry_hash.clone();
        }

        info!(
            "Secure log entry created: action={}, user={}, result={}",
            action, user_id, result
        );

        Ok(())
    }

    /// Write an entry to the log file
    async fn write_entry(&self, entry: &SecureLogEntry) -> Result<()> {
        let _lock = self.file_lock.write().await;

        // Check if rotation is needed
        self.rotate_if_needed().await?;

        // Open file in append mode
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_path)?;

        // Write the JSON entry with a newline
        let json = serde_json::to_string(entry)?;
        writeln!(file, "{}", json)?;
        file.flush()?;

        Ok(())
    }

    /// Rotate log files if needed
    async fn rotate_if_needed(&self) -> Result<()> {
        if !self.config.log_path.exists() {
            return Ok(());
        }

        let metadata = std::fs::metadata(&self.config.log_path)?;
        let size_mb = metadata.len() / (1024 * 1024);

        if size_mb >= self.config.max_size_mb {
            self.rotate_logs()?;
        }

        Ok(())
    }

    /// Rotate log files
    fn rotate_logs(&self) -> Result<()> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let rotated_path = self
            .config
            .log_path
            .with_extension(format!("{}.log", timestamp));

        std::fs::rename(&self.config.log_path, &rotated_path)?;

        info!("Rotated secure log to: {:?}", rotated_path);

        // Clean up old rotations if needed
        self.cleanup_old_rotations()?;

        Ok(())
    }

    /// Clean up old rotation files
    fn cleanup_old_rotations(&self) -> Result<()> {
        if let Some(parent) = self.config.log_path.parent() {
            let base_name = self
                .config
                .log_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("secure");

            let mut rotated_files: Vec<_> = std::fs::read_dir(parent)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    if let Some(name) = entry.file_name().to_str() {
                        name.starts_with(base_name) && name != "secure.log"
                    } else {
                        false
                    }
                })
                .collect();

            // Sort by modification time
            rotated_files.sort_by_key(|entry| {
                entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            });

            // Remove old files if we have too many
            while rotated_files.len() > self.config.max_rotations as usize {
                if let Some(old_file) = rotated_files.first() {
                    std::fs::remove_file(old_file.path())?;
                    info!("Removed old rotation: {:?}", old_file.path());
                    rotated_files.remove(0);
                }
            }
        }

        Ok(())
    }

    /// Get the last hash from an existing log file
    fn get_last_hash(path: &Path) -> Result<String> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut last_hash =
            String::from("0000000000000000000000000000000000000000000000000000000000000000");

        for line in reader.lines().map_while(|r| r.ok()) {
            if let Ok(entry) = serde_json::from_str::<SecureLogEntry>(&line) {
                last_hash = entry.entry_hash;
            }
        }

        Ok(last_hash)
    }

    /// Verify the integrity of the entire log chain
    pub async fn verify_log_chain(&self) -> Result<bool> {
        if !self.config.log_path.exists() {
            return Ok(true); // Empty log is valid
        }

        let file = File::open(&self.config.log_path)?;
        let reader = BufReader::new(file);

        let mut expected_previous_hash =
            String::from("0000000000000000000000000000000000000000000000000000000000000000");
        let mut line_number = 0;

        for line in reader.lines() {
            line_number += 1;
            let line = line?;

            let entry: SecureLogEntry = serde_json::from_str(&line).map_err(|e| {
                UserError::SecureLogError(format!("Failed to parse line {}: {}", line_number, e))
            })?;

            // Verify the entry's hash
            if !entry.verify_hash() {
                error!(
                    "Hash verification failed at line {}: entry_id={}",
                    line_number, entry.id
                );
                return Ok(false);
            }

            // Verify the chain
            if entry.previous_hash != expected_previous_hash {
                error!(
                    "Chain verification failed at line {}: expected_previous={}, got={}",
                    line_number, expected_previous_hash, entry.previous_hash
                );
                return Ok(false);
            }

            expected_previous_hash = entry.entry_hash;
        }

        info!(
            "Log chain verification successful: {} entries verified",
            line_number
        );
        Ok(true)
    }

    /// Replay the log from a backup to verify against current state
    pub async fn replay_from_backup(&self, backup_path: &Path) -> Result<Vec<SecureLogEntry>> {
        if !backup_path.exists() {
            return Err(UserError::SecureLogError(format!(
                "Backup file not found: {:?}",
                backup_path
            )));
        }

        let file = File::open(backup_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let entry: SecureLogEntry = serde_json::from_str(&line)?;

            if !entry.verify_hash() {
                return Err(UserError::HashVerificationFailed);
            }

            entries.push(entry);
        }

        info!("Replayed {} entries from backup", entries.len());
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_secure_log_entry_hash() {
        let entry = SecureLogEntry::new(
            1,
            "login".to_string(),
            Some("user123".to_string()),
            None,
            Some("192.168.1.1".to_string()),
            "success".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        );

        assert!(entry.verify_hash());
    }

    #[tokio::test]
    async fn test_secure_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("secure.log");

        let config = SecureLogConfig {
            log_path: log_path.clone(),
            max_size_mb: 10,
            max_rotations: 5,
            enable_verification: true,
        };

        let logger = SecureLogger::new(config).unwrap();

        // Log an action
        logger
            .log_action(1, "test_action", None, None, None, true)
            .await
            .unwrap();

        assert!(log_path.exists());
    }

    #[tokio::test]
    async fn test_log_chain_verification() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("secure.log");

        let config = SecureLogConfig {
            log_path: log_path.clone(),
            max_size_mb: 10,
            max_rotations: 5,
            enable_verification: true,
        };

        let logger = SecureLogger::new(config).unwrap();

        // Log multiple actions
        for i in 0..5 {
            logger
                .log_action(i, &format!("action_{}", i), None, None, None, true)
                .await
                .unwrap();
        }

        // Verify the chain
        assert!(logger.verify_log_chain().await.unwrap());
    }
}

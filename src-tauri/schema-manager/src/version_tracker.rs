use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
#[allow(unused_imports)]
use std::collections::HashMap;
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

/// Version information for a tracked file
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FileVersion {
    pub id: i64,
    pub file_path: String,
    pub version: i32,
    pub file_hash: String,
    pub action_id: String,
    pub applied_at: DateTime<Utc>,
    pub applied_by: Option<String>,
    pub actions_executed: String, // JSON string of executed actions
    pub rollback_actions: Option<String>, // JSON string of rollback actions
    pub status: String, // pending, applied, rolled_back
}

/// Version tracker for configuration files
pub struct VersionTracker {
    pool: SqlitePool,
}

impl VersionTracker {
    /// Create a new version tracker
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    /// Initialize the version tracking table
    pub async fn init_table(&self) -> Result<(), sqlx::Error> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS file_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                version INTEGER NOT NULL,
                file_hash TEXT NOT NULL,
                action_id TEXT NOT NULL,
                applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                applied_by TEXT,
                actions_executed TEXT NOT NULL,
                rollback_actions TEXT,
                status TEXT CHECK(status IN ('pending', 'applied', 'rolled_back')) DEFAULT 'pending',
                UNIQUE(file_path, version)
            )
        "#;
        
        sqlx::query(sql).execute(&self.pool).await?;
        
        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_file_versions_path ON file_versions(file_path)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_file_versions_status ON file_versions(status)")
            .execute(&self.pool)
            .await?;
        
        info!("Version tracking table initialized");
        Ok(())
    }
    
    /// Record a new version for a file
    pub async fn record_version(
        &self,
        file_path: &str,
        file_hash: &str,
        actions: &[crate::action_generator::Action],
        applied_by: Option<String>,
    ) -> Result<i64, sqlx::Error> {
        // Get the next version number for this file
        let next_version = self.get_next_version(file_path).await?;
        
        // Generate a unique action ID
        let action_id = format!("{}-{}-{}", 
            file_path.replace('/', "_").replace('.', "_"),
            next_version,
            chrono::Utc::now().timestamp()
        );
        
        // Serialize actions to JSON
        let actions_json = serde_json::to_string(actions)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        
        // Generate rollback actions
        let rollback_actions: Vec<_> = actions.iter()
            .filter_map(|a| a.rollback_action())
            .collect();
        
        let rollback_json = if !rollback_actions.is_empty() {
            Some(serde_json::to_string(&rollback_actions)
                .map_err(|e| sqlx::Error::Protocol(e.to_string()))?)
        } else {
            None
        };
        
        // Insert the version record
        let sql = r#"
            INSERT INTO file_versions (
                file_path, version, file_hash, action_id, 
                applied_by, actions_executed, rollback_actions, status
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 'applied')
        "#;
        
        let result = sqlx::query(sql)
            .bind(file_path)
            .bind(next_version)
            .bind(file_hash)
            .bind(&action_id)
            .bind(applied_by)
            .bind(actions_json)
            .bind(rollback_json)
            .execute(&self.pool)
            .await?;
        
        info!("Recorded version {} for file {}", next_version, file_path);
        Ok(result.last_insert_rowid())
    }
    
    /// Get the next version number for a file
    async fn get_next_version(&self, file_path: &str) -> Result<i32, sqlx::Error> {
        let sql = "SELECT MAX(version) FROM file_versions WHERE file_path = ?";
        let result: Option<(Option<i32>,)> = sqlx::query_as(sql)
            .bind(file_path)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(result.and_then(|(v,)| v).unwrap_or(0) + 1)
    }
    
    /// Get the current version of a file
    pub async fn get_current_version(&self, file_path: &str) -> Result<Option<FileVersion>, sqlx::Error> {
        let sql = r#"
            SELECT * FROM file_versions 
            WHERE file_path = ? AND status = 'applied'
            ORDER BY version DESC 
            LIMIT 1
        "#;
        
        sqlx::query_as(sql)
            .bind(file_path)
            .fetch_optional(&self.pool)
            .await
    }
    
    /// Get version history for a file
    pub async fn get_version_history(
        &self,
        file_path: &str,
        limit: Option<i32>,
    ) -> Result<Vec<FileVersion>, sqlx::Error> {
        let mut sql = String::from(
            "SELECT * FROM file_versions WHERE file_path = ? ORDER BY version DESC"
        );
        
        if let Some(limit) = limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        sqlx::query_as(&sql)
            .bind(file_path)
            .fetch_all(&self.pool)
            .await
    }
    
    /// Get all versions across all files
    pub async fn get_all_versions(
        &self,
        limit: Option<i32>,
    ) -> Result<Vec<FileVersion>, sqlx::Error> {
        let mut sql = String::from(
            "SELECT * FROM file_versions ORDER BY applied_at DESC"
        );
        
        if let Some(limit) = limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        sqlx::query_as(&sql)
            .fetch_all(&self.pool)
            .await
    }
    
    /// Rollback to a specific version
    pub async fn rollback_to_version(
        &self,
        file_path: &str,
        target_version: i32,
    ) -> Result<Vec<crate::action_generator::Action>, sqlx::Error> {
        // Get all versions after the target version
        let sql = r#"
            SELECT * FROM file_versions 
            WHERE file_path = ? AND version > ? AND status = 'applied'
            ORDER BY version DESC
        "#;
        
        let versions: Vec<FileVersion> = sqlx::query_as(sql)
            .bind(file_path)
            .bind(target_version)
            .fetch_all(&self.pool)
            .await?;
        
        let mut rollback_actions = Vec::new();
        
        // Collect rollback actions in reverse order
        for version in versions {
            if let Some(rollback_json) = version.rollback_actions {
                match serde_json::from_str::<Vec<crate::action_generator::Action>>(&rollback_json) {
                    Ok(actions) => rollback_actions.extend(actions),
                    Err(e) => {
                        warn!("Failed to parse rollback actions for version {}: {}", version.id, e);
                    }
                }
            }
            
            // Mark the version as rolled back
            sqlx::query("UPDATE file_versions SET status = 'rolled_back' WHERE id = ?")
                .bind(version.id)
                .execute(&self.pool)
                .await?;
        }
        
        info!("Prepared {} rollback actions for file {} to version {}", 
              rollback_actions.len(), file_path, target_version);
        
        Ok(rollback_actions)
    }
    
    /// Calculate file hash for change detection
    pub fn calculate_file_hash(content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    /// Check if a file has changed based on its hash
    pub async fn has_file_changed(
        &self,
        file_path: &str,
        current_hash: &str,
    ) -> Result<bool, sqlx::Error> {
        if let Some(version) = self.get_current_version(file_path).await? {
            Ok(version.file_hash != current_hash)
        } else {
            // No version recorded, so it's a new file
            Ok(true)
        }
    }
    
    /// Get statistics about version tracking
    pub async fn get_statistics(&self) -> Result<VersionStatistics, sqlx::Error> {
        let total_versions: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM file_versions"
        )
        .fetch_one(&self.pool)
        .await?;
        
        let total_files: (i64,) = sqlx::query_as(
            "SELECT COUNT(DISTINCT file_path) FROM file_versions"
        )
        .fetch_one(&self.pool)
        .await?;
        
        let applied_versions: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM file_versions WHERE status = 'applied'"
        )
        .fetch_one(&self.pool)
        .await?;
        
        let rolled_back_versions: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM file_versions WHERE status = 'rolled_back'"
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(VersionStatistics {
            total_versions: total_versions.0,
            total_files: total_files.0,
            applied_versions: applied_versions.0,
            rolled_back_versions: rolled_back_versions.0,
        })
    }
}

/// Statistics about version tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionStatistics {
    pub total_versions: i64,
    pub total_files: i64,
    pub applied_versions: i64,
    pub rolled_back_versions: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    async fn create_test_pool() -> SqlitePool {
        SqlitePool::connect(":memory:").await.unwrap()
    }
    
    #[tokio::test]
    async fn test_version_tracking() {
        let pool = create_test_pool().await;
        let tracker = VersionTracker::new(pool);
        
        // Initialize table
        tracker.init_table().await.unwrap();
        
        // Record a version
        let actions = vec![
            crate::action_generator::Action::CreateTable {
                entity_id: "test".to_string(),
                table_name: "content_test".to_string(),
                sql: "CREATE TABLE content_test (id TEXT)".to_string(),
            },
        ];
        
        let version_id = tracker.record_version(
            "schemas/test.yaml",
            "abc123",
            &actions,
            Some("test_user".to_string()),
        ).await.unwrap();
        
        assert!(version_id > 0);
        
        // Get current version
        let current = tracker.get_current_version("schemas/test.yaml").await.unwrap();
        assert!(current.is_some());
        assert_eq!(current.unwrap().version, 1);
        
        // Check if file has changed
        let changed = tracker.has_file_changed("schemas/test.yaml", "xyz789").await.unwrap();
        assert!(changed);
        
        let not_changed = tracker.has_file_changed("schemas/test.yaml", "abc123").await.unwrap();
        assert!(!not_changed);
    }
    
    #[test]
    fn test_file_hash() {
        let content1 = "test content";
        let content2 = "test content";
        let content3 = "different content";
        
        let hash1 = VersionTracker::calculate_file_hash(content1);
        let hash2 = VersionTracker::calculate_file_hash(content2);
        let hash3 = VersionTracker::calculate_file_hash(content3);
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
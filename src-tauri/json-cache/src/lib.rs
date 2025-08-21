pub mod error;

use chrono::{DateTime, Utc};
use error::Result;
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// Define the table for storing cached JSON content
const CACHE_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("json_cache");
const METADATA_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("cache_metadata");

/// Metadata for cached entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub key: String,
    pub entity_type: String,
    pub cached_at: DateTime<Utc>,
    pub ttl_seconds: i64,
    pub content_hash: String,
    pub size_bytes: usize,
}

/// Entry in the cache combining content and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub content: serde_json::Value,
    pub metadata: CacheMetadata,
}

/// JSON Cache using ReDB for persistent key-value storage
pub struct JsonCache {
    db: Arc<Database>,
    path: PathBuf,
}

impl JsonCache {
    /// Create a new JSON cache instance
    pub fn new(cache_path: impl AsRef<Path>) -> Result<Self> {
        let path = cache_path.as_ref().to_path_buf();

        // Ensure the parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!("Opening JSON cache database at: {:?}", path);
        let db = Database::create(&path)?;

        // Initialize tables
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(CACHE_TABLE)?;
            let _ = write_txn.open_table(METADATA_TABLE)?;
        }
        write_txn.commit()?;

        info!("JSON cache initialized successfully");

        Ok(Self {
            db: Arc::new(db),
            path,
        })
    }

    /// Get the path to the cache database
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Store a JSON value in the cache
    pub fn set(
        &self,
        key: &str,
        value: &serde_json::Value,
        entity_type: &str,
        ttl_seconds: i64,
        content_hash: &str,
    ) -> Result<()> {
        debug!(
            "Caching entity: key={}, type={}, ttl={}",
            key, entity_type, ttl_seconds
        );

        // Serialize the content
        let content_bytes = serde_json::to_vec(value)?;
        let size_bytes = content_bytes.len();

        // Create metadata
        let metadata = CacheMetadata {
            key: key.to_string(),
            entity_type: entity_type.to_string(),
            cached_at: Utc::now(),
            ttl_seconds,
            content_hash: content_hash.to_string(),
            size_bytes,
        };

        let metadata_bytes = serde_json::to_vec(&metadata)?;

        // Store in database
        let write_txn = self.db.begin_write()?;
        {
            let mut cache_table = write_txn.open_table(CACHE_TABLE)?;
            cache_table.insert(key, content_bytes.as_slice())?;

            let mut metadata_table = write_txn.open_table(METADATA_TABLE)?;
            metadata_table.insert(key, metadata_bytes.as_slice())?;
        }
        write_txn.commit()?;

        debug!(
            "Successfully cached entity: key={}, size={} bytes",
            key, size_bytes
        );
        Ok(())
    }

    /// Get a JSON value from the cache
    pub fn get(&self, key: &str) -> Result<Option<CacheEntry>> {
        debug!("Retrieving from cache: key={}", key);

        let read_txn = self.db.begin_read()?;
        let cache_table = read_txn.open_table(CACHE_TABLE)?;
        let metadata_table = read_txn.open_table(METADATA_TABLE)?;

        // Get content
        let content_bytes = match cache_table.get(key)? {
            Some(bytes) => bytes.value().to_vec(),
            None => {
                debug!("Cache miss: key={}", key);
                return Ok(None);
            }
        };

        // Get metadata
        let metadata_bytes = match metadata_table.get(key)? {
            Some(bytes) => bytes.value().to_vec(),
            None => {
                warn!("Cache metadata missing for key={}", key);
                return Ok(None);
            }
        };

        // Deserialize
        let content: serde_json::Value = serde_json::from_slice(&content_bytes)?;
        let metadata: CacheMetadata = serde_json::from_slice(&metadata_bytes)?;

        // Check if expired
        let age_seconds = (Utc::now() - metadata.cached_at).num_seconds();
        // TTL of 0 means expired immediately, positive TTL means check against age
        if metadata.ttl_seconds == 0
            || (metadata.ttl_seconds > 0 && age_seconds > metadata.ttl_seconds)
        {
            debug!(
                "Cache entry expired: key={}, age={}, ttl={}",
                key, age_seconds, metadata.ttl_seconds
            );
            return Ok(None);
        }

        debug!("Cache hit: key={}, age={} seconds", key, age_seconds);
        Ok(Some(CacheEntry { content, metadata }))
    }

    /// Check if a key exists and is not expired
    pub fn exists(&self, key: &str) -> Result<bool> {
        match self.get(key)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// Delete a value from the cache
    pub fn delete(&self, key: &str) -> Result<bool> {
        debug!("Deleting from cache: key={}", key);

        let write_txn = self.db.begin_write()?;
        let deleted = {
            let mut cache_table = write_txn.open_table(CACHE_TABLE)?;
            let was_deleted = cache_table.remove(key)?.is_some();

            let mut metadata_table = write_txn.open_table(METADATA_TABLE)?;
            metadata_table.remove(key)?;

            was_deleted
        };
        write_txn.commit()?;

        if deleted {
            debug!("Successfully deleted from cache: key={}", key);
        } else {
            debug!("Key not found in cache: key={}", key);
        }

        Ok(deleted)
    }

    /// Clear all entries from the cache
    pub fn clear(&self) -> Result<usize> {
        info!("Clearing all cache entries");

        let write_txn = self.db.begin_write()?;
        // Clear tables by iterating and removing all entries
        let mut count = 0;
        {
            let mut cache_table = write_txn.open_table(CACHE_TABLE)?;
            let mut metadata_table = write_txn.open_table(METADATA_TABLE)?;

            // Collect all keys first
            let mut keys_to_remove = Vec::new();
            for result in cache_table.iter()? {
                let (key, _) = result?;
                keys_to_remove.push(key.value().to_string());
                count += 1;
            }

            // Remove all entries
            for key in keys_to_remove {
                cache_table.remove(key.as_str())?;
                metadata_table.remove(key.as_str())?;
            }
        }
        write_txn.commit()?;

        info!("Cleared {} cache entries", count);
        Ok(count as usize)
    }

    /// Get statistics about the cache
    pub fn stats(&self) -> Result<CacheStats> {
        let read_txn = self.db.begin_read()?;
        let cache_table = read_txn.open_table(CACHE_TABLE)?;
        let metadata_table = read_txn.open_table(METADATA_TABLE)?;

        let mut total_entries = 0usize;
        for _ in cache_table.iter()? {
            total_entries += 1;
        }
        let mut total_size = 0usize;
        let mut expired_count = 0usize;
        let mut entity_types = std::collections::HashMap::new();

        // Iterate through metadata to gather stats
        for result in metadata_table.iter()? {
            let (_, metadata_bytes) = result?;
            let metadata: CacheMetadata = serde_json::from_slice(metadata_bytes.value())?;

            total_size += metadata.size_bytes;

            // Check if expired
            let age_seconds = (Utc::now() - metadata.cached_at).num_seconds();
            if metadata.ttl_seconds == 0
                || (metadata.ttl_seconds > 0 && age_seconds > metadata.ttl_seconds)
            {
                expired_count += 1;
            }

            // Count by entity type
            *entity_types
                .entry(metadata.entity_type.clone())
                .or_insert(0) += 1;
        }

        Ok(CacheStats {
            total_entries,
            total_size_bytes: total_size,
            expired_entries: expired_count,
            active_entries: total_entries - expired_count,
            entries_by_type: entity_types,
        })
    }

    /// Remove expired entries from the cache
    pub fn evict_expired(&self) -> Result<usize> {
        info!("Evicting expired cache entries");

        let read_txn = self.db.begin_read()?;
        let metadata_table = read_txn.open_table(METADATA_TABLE)?;

        let mut expired_keys = Vec::new();

        // Find expired entries
        for result in metadata_table.iter()? {
            let (key, metadata_bytes) = result?;
            let metadata: CacheMetadata = serde_json::from_slice(metadata_bytes.value())?;

            let age_seconds = (Utc::now() - metadata.cached_at).num_seconds();
            if metadata.ttl_seconds == 0
                || (metadata.ttl_seconds > 0 && age_seconds > metadata.ttl_seconds)
            {
                expired_keys.push(key.value().to_string());
            }
        }

        drop(read_txn);

        // Delete expired entries
        let count = expired_keys.len();
        for key in expired_keys {
            self.delete(&key)?;
        }

        info!("Evicted {} expired cache entries", count);
        Ok(count)
    }
}

/// Statistics about the cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub expired_entries: usize,
    pub active_entries: usize,
    pub entries_by_type: std::collections::HashMap<String, usize>,
}

/// Thread-safe cache manager
pub struct CacheManager {
    cache: Arc<RwLock<JsonCache>>,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(cache_path: impl AsRef<Path>) -> Result<Self> {
        let cache = JsonCache::new(cache_path)?;
        Ok(Self {
            cache: Arc::new(RwLock::new(cache)),
        })
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &str) -> Result<Option<CacheEntry>> {
        let cache = self.cache.read().await;
        cache.get(key)
    }

    /// Set a value in the cache
    pub async fn set(
        &self,
        key: &str,
        value: &serde_json::Value,
        entity_type: &str,
        ttl_seconds: i64,
        content_hash: &str,
    ) -> Result<()> {
        let cache = self.cache.read().await;
        cache.set(key, value, entity_type, ttl_seconds, content_hash)
    }

    /// Delete a value from the cache
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let cache = self.cache.write().await;
        cache.delete(key)
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let cache = self.cache.read().await;
        cache.exists(key)
    }

    /// Clear the cache
    pub async fn clear(&self) -> Result<usize> {
        let cache = self.cache.write().await;
        cache.clear()
    }

    /// Get cache statistics
    pub async fn stats(&self) -> Result<CacheStats> {
        let cache = self.cache.read().await;
        cache.stats()
    }

    /// Evict expired entries
    pub async fn evict_expired(&self) -> Result<usize> {
        let cache = self.cache.write().await;
        cache.evict_expired()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let dir = std::env::temp_dir();
        let cache_path = dir.join(format!("test_cache_{}.db", std::process::id()));
        let cache = JsonCache::new(&cache_path).unwrap();

        // Test set and get
        let value = serde_json::json!({
            "id": "test1",
            "title": "Test Entity",
            "content": "This is a test"
        });

        cache
            .set("test1", &value, "test_entity", 3600, "hash123")
            .unwrap();

        let entry = cache.get("test1").unwrap().unwrap();
        assert_eq!(entry.content, value);
        assert_eq!(entry.metadata.entity_type, "test_entity");

        // Test exists
        assert!(cache.exists("test1").unwrap());
        assert!(!cache.exists("nonexistent").unwrap());

        // Test delete
        assert!(cache.delete("test1").unwrap());
        assert!(!cache.exists("test1").unwrap());
    }

    #[test]
    fn test_cache_expiration() {
        let dir = std::env::temp_dir();
        let cache_path = dir.join(format!("test_cache_exp_{}.db", std::process::id()));
        let cache = JsonCache::new(&cache_path).unwrap();

        let value = serde_json::json!({"test": "data"});

        // Set with 0 TTL (expired immediately)
        cache.set("expired", &value, "test", 0, "hash").unwrap();

        // Should not return expired entry
        assert!(cache.get("expired").unwrap().is_none());

        // Clean up
        let _ = std::fs::remove_file(&cache_path);
    }
}

use crate::{Database, DatabaseError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{Column, Row};
use std::collections::HashMap;
use tracing::{debug, info};

/// Represents a content item stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    pub id: String,
    pub entity_type: String,
    pub fields: HashMap<String, JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Entity content storage operations
pub struct EntityStorage<'a> {
    db: &'a Database,
    entity_type: String,
    table_name: String,
    versioned: bool,
}

impl<'a> EntityStorage<'a> {
    /// Create a new EntityStorage instance
    pub fn new(db: &'a Database, entity_type: &str) -> Self {
        let table_name = format!("content_{}", entity_type);
        Self {
            db,
            entity_type: entity_type.to_string(),
            table_name,
            versioned: false, // Default to false, should be set based on entity definition
        }
    }
    
    /// Create a new EntityStorage instance with versioning support
    pub fn new_versioned(db: &'a Database, entity_type: &str, versioned: bool) -> Self {
        let table_name = format!("content_{}", entity_type);
        Self {
            db,
            entity_type: entity_type.to_string(),
            table_name,
            versioned,
        }
    }
    
    /// Create a new content item
    pub async fn create(&self, fields: HashMap<String, JsonValue>) -> Result<String> {
        let id = generate_id();
        let uuid = generate_uuid();
        let mut columns = vec!["id".to_string(), "uuid".to_string()];
        let mut placeholders = vec!["?".to_string(), "?".to_string()];
        let mut values: Vec<JsonValue> = vec![
            JsonValue::String(id.clone()),
            JsonValue::String(uuid),
        ];
        
        // Build dynamic SQL based on provided fields
        for (field_name, field_value) in &fields {
            columns.push(field_name.clone());
            placeholders.push("?".to_string());
            values.push(field_value.clone());
        }
        
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.table_name,
            columns.join(", "),
            placeholders.join(", ")
        );
        
        debug!("Executing SQL: {}", sql);
        
        // Build the query dynamically
        let mut query = sqlx::query(&sql);
        for value in values {
            query = match value {
                JsonValue::String(s) => query.bind(s),
                JsonValue::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        query.bind(i)
                    } else if let Some(f) = n.as_f64() {
                        query.bind(f)
                    } else {
                        query.bind(n.to_string())
                    }
                },
                JsonValue::Bool(b) => query.bind(b as i32),
                JsonValue::Null => query.bind(None::<String>),
                _ => query.bind(value.to_string()),
            };
        }
        
        query.execute(self.db.pool()).await?;
        
        info!("Created {} with id: {}", self.entity_type, id);
        
        Ok(id)
    }
    
    /// Get a content item by ID
    pub async fn get(&self, id: &str) -> Result<Option<ContentItem>> {
        let sql = format!("SELECT * FROM {} WHERE id = ?", self.table_name);
        
        let row = match sqlx::query(&sql)
            .bind(id)
            .fetch_optional(self.db.pool())
            .await?
        {
            Some(row) => row,
            None => return Ok(None),
        };
        
        // Extract all columns into a HashMap
        let mut fields = HashMap::new();
        let columns = row.columns();
        
        for (i, column) in columns.iter().enumerate() {
            let name = column.name();
            
            // Skip system fields
            if name == "id" || name == "created_at" || name == "updated_at" {
                continue;
            }
            
            // Try to get value as different types
            let value = if let Ok(v) = row.try_get::<String, _>(i) {
                JsonValue::String(v)
            } else if let Ok(v) = row.try_get::<i64, _>(i) {
                JsonValue::Number(serde_json::Number::from(v))
            } else if let Ok(v) = row.try_get::<f64, _>(i) {
                JsonValue::Number(serde_json::Number::from_f64(v).unwrap())
            } else if let Ok(v) = row.try_get::<bool, _>(i) {
                JsonValue::Bool(v)
            } else {
                JsonValue::Null
            };
            
            fields.insert(name.to_string(), value);
        }
        
        let content = ContentItem {
            id: row.try_get("id")?,
            entity_type: self.entity_type.clone(),
            fields,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        
        Ok(Some(content))
    }
    
    /// Update a content item with revision support
    pub async fn update(&self, id: &str, fields: HashMap<String, JsonValue>) -> Result<()> {
        if fields.is_empty() {
            return Ok(());
        }
        
        // If versioned, create a revision before updating
        if self.versioned {
            self.create_revision(id).await?;
        }
        
        let mut set_clauses = vec!["updated_at = CURRENT_TIMESTAMP".to_string()];
        
        // If versioned, increment the revision ID
        if self.versioned {
            set_clauses.push("rid = rid + 1".to_string());
        }
        
        let mut values: Vec<JsonValue> = Vec::new();
        
        for (field_name, field_value) in &fields {
            set_clauses.push(format!("{} = ?", field_name));
            values.push(field_value.clone());
        }
        
        let sql = format!(
            "UPDATE {} SET {} WHERE id = ?",
            self.table_name,
            set_clauses.join(", ")
        );
        
        debug!("Executing SQL: {}", sql);
        
        // Build the query dynamically
        let mut query = sqlx::query(&sql);
        for value in values {
            query = match value {
                JsonValue::String(s) => query.bind(s),
                JsonValue::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        query.bind(i)
                    } else if let Some(f) = n.as_f64() {
                        query.bind(f)
                    } else {
                        query.bind(n.to_string())
                    }
                },
                JsonValue::Bool(b) => query.bind(b as i32),
                JsonValue::Null => query.bind(None::<String>),
                _ => query.bind(value.to_string()),
            };
        }
        query = query.bind(id);
        
        let result = query.execute(self.db.pool()).await?;
        
        if result.rows_affected() == 0 {
            return Err(DatabaseError::EntityNotFound(format!("{} with id: {}", self.entity_type, id)));
        }
        
        info!("Updated {} with id: {}", self.entity_type, id);
        
        Ok(())
    }
    
    /// Create a revision of the current entity state
    async fn create_revision(&self, id: &str) -> Result<()> {
        let revision_table = format!("content_revisions_{}", self.entity_type);
        
        // The revision table has the same columns as the main table plus revision_created_at
        // We need to copy all data from the main table including the current rid value
        // The rid in the revision table represents the revision number at the time of the snapshot
        
        // Simple approach: copy all columns from main table, add revision_created_at
        let sql = format!(
            r#"
            INSERT INTO {}
            SELECT
                id,
                uuid,
                user,
                rid,  -- Copy the current rid value
                created_at,
                updated_at,
                CURRENT_TIMESTAMP as revision_created_at,
                title,
                body,
                author,
                published_at,
                status
            FROM {}
            WHERE id = ?
            "#,
            revision_table,
            self.table_name
        );
        
        sqlx::query(&sql)
            .bind(id)
            .execute(self.db.pool())
            .await
            .map_err(|e| {
                tracing::error!("Failed to create revision: {}", e);
                // If the specific column list doesn't work (e.g., for entities with different fields),
                // we need a more dynamic approach
                DatabaseError::Other(format!("Failed to create revision: {}", e))
            })?;
        
        debug!("Created revision for {} with id: {}", self.entity_type, id);
        
        // Also create revisions for multi-value fields if they exist
        // This would need to be implemented based on entity definition
        
        Ok(())
    }
    
    /// Get a specific revision of an entity
    pub async fn get_revision(&self, id: &str, rid: i64) -> Result<Option<ContentItem>> {
        if !self.versioned {
            return Err(DatabaseError::Validation("Entity is not versioned".to_string()));
        }
        
        let revision_table = format!("content_revisions_{}", self.entity_type);
        let sql = format!("SELECT * FROM {} WHERE id = ? AND rid = ?", revision_table);
        
        let row = match sqlx::query(&sql)
            .bind(id)
            .bind(rid)
            .fetch_optional(self.db.pool())
            .await?
        {
            Some(row) => row,
            None => return Ok(None),
        };
        
        // Extract all columns into a HashMap (similar to get method)
        let mut fields = HashMap::new();
        let columns = row.columns();
        
        for (i, column) in columns.iter().enumerate() {
            let name = column.name();
            
            // Skip system fields
            if name == "id" || name == "created_at" || name == "updated_at" ||
               name == "rid" || name == "revision_created_at" {
                continue;
            }
            
            // Try to get value as different types
            let value = if let Ok(v) = row.try_get::<String, _>(i) {
                JsonValue::String(v)
            } else if let Ok(v) = row.try_get::<i64, _>(i) {
                JsonValue::Number(serde_json::Number::from(v))
            } else if let Ok(v) = row.try_get::<f64, _>(i) {
                JsonValue::Number(serde_json::Number::from_f64(v).unwrap())
            } else if let Ok(v) = row.try_get::<bool, _>(i) {
                JsonValue::Bool(v)
            } else {
                JsonValue::Null
            };
            
            fields.insert(name.to_string(), value);
        }
        
        let content = ContentItem {
            id: row.try_get("id")?,
            entity_type: self.entity_type.clone(),
            fields,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };
        
        Ok(Some(content))
    }
    
    /// List all revisions for an entity
    pub async fn list_revisions(&self, id: &str) -> Result<Vec<i64>> {
        if !self.versioned {
            return Err(DatabaseError::Validation("Entity is not versioned".to_string()));
        }
        
        let revision_table = format!("content_revisions_{}", self.entity_type);
        let sql = format!(
            "SELECT rid FROM {} WHERE id = ? ORDER BY rid DESC",
            revision_table
        );
        
        let rows = sqlx::query(&sql)
            .bind(id)
            .fetch_all(self.db.pool())
            .await?;
        
        let mut revisions = Vec::new();
        for row in rows {
            revisions.push(row.try_get("rid")?);
        }
        
        Ok(revisions)
    }
    
    /// Delete a content item
    pub async fn delete(&self, id: &str) -> Result<()> {
        let sql = format!("DELETE FROM {} WHERE id = ?", self.table_name);
        
        let result = sqlx::query(&sql)
            .bind(id)
            .execute(self.db.pool())
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(DatabaseError::EntityNotFound(format!("{} with id: {}", self.entity_type, id)));
        }
        
        info!("Deleted {} with id: {}", self.entity_type, id);
        
        Ok(())
    }
    
    /// List all content items
    pub async fn list(&self, limit: Option<i64>, offset: Option<i64>) -> Result<Vec<ContentItem>> {
        let mut sql = format!("SELECT * FROM {} ORDER BY created_at DESC", self.table_name);
        
        if let Some(limit) = limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }
        
        let rows = sqlx::query(&sql)
            .fetch_all(self.db.pool())
            .await?;
        
        let mut items = Vec::new();
        
        for row in rows {
            // Extract all columns into a HashMap
            let mut fields = HashMap::new();
            let columns = row.columns();
            
            for (i, column) in columns.iter().enumerate() {
                let name = column.name();
                
                // Skip system fields
                if name == "id" || name == "created_at" || name == "updated_at" {
                    continue;
                }
                
                // Try to get value as different types
                let value = if let Ok(v) = row.try_get::<Option<String>, _>(i) {
                    v.map(JsonValue::String).unwrap_or(JsonValue::Null)
                } else if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
                    v.map(|n| JsonValue::Number(serde_json::Number::from(n))).unwrap_or(JsonValue::Null)
                } else if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
                    v.and_then(serde_json::Number::from_f64)
                        .map(JsonValue::Number)
                        .unwrap_or(JsonValue::Null)
                } else if let Ok(v) = row.try_get::<Option<bool>, _>(i) {
                    v.map(JsonValue::Bool).unwrap_or(JsonValue::Null)
                } else {
                    JsonValue::Null
                };
                
                fields.insert(name.to_string(), value);
            }
            
            let content = ContentItem {
                id: row.try_get("id")?,
                entity_type: self.entity_type.clone(),
                fields,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            };
            
            items.push(content);
        }
        
        Ok(items)
    }
}

/// Generate a unique ID for content items
fn generate_id() -> String {
    // Simple UUID v4 generation
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let random: u32 = rand::random();
    format!("{:x}-{:x}", timestamp, random)
}

/// Generate a UUID for content items
fn generate_uuid() -> String {
    // Generate a proper UUID v4
    use uuid::Uuid;
    Uuid::new_v4().to_string()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use std::sync::atomic::{AtomicU32, Ordering};
    
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);
    
    fn get_test_db_path() -> String {
        // Get the project root by navigating up from CARGO_MANIFEST_DIR
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let database_dir = std::path::Path::new(manifest_dir); // src-tauri/database
        let src_tauri = database_dir.parent().unwrap(); // src-tauri
        let project_root = src_tauri.parent().unwrap(); // project root
        
        // Use /data/test_databases/ at project root
        let test_db_dir = project_root.join("data").join("test_databases");
        std::fs::create_dir_all(&test_db_dir).unwrap();
        
        // Use a unique database for each test to avoid conflicts
        // Use atomic counter for thread safety
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        // Include timestamp to ensure uniqueness even across test runs
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        let db_path = test_db_dir.join(format!("test_storage_{}_{}.db", test_id, timestamp));
        
        // Clean up any existing test database with this name (shouldn't exist with timestamp)
        let _ = std::fs::remove_file(&db_path);
        
        db_path.to_string_lossy().to_string()
    }
    
    async fn setup_test_db() -> Database {
        let db_path = get_test_db_path();
        
        // Remove existing test database if it exists
        let _ = std::fs::remove_file(&db_path);
        
        // Create the database file first (SQLite requires this)
        std::fs::File::create(&db_path).unwrap();
        
        let db = Database::new(&db_path).await.unwrap();
        
        // Create a test table (drop if exists first to ensure clean state)
        // Properly await the DROP TABLE command to ensure it completes
        let _ = db.execute_raw("DROP TABLE IF EXISTS content_test").await;
        
        // Use CREATE TABLE IF NOT EXISTS to be extra safe
        db.execute_raw(
            r#"
            CREATE TABLE IF NOT EXISTS content_test (
                id TEXT PRIMARY KEY,
                uuid TEXT NOT NULL UNIQUE,
                user INTEGER DEFAULT 0,
                rid INTEGER DEFAULT 1,
                title TEXT,
                count INTEGER,
                active INTEGER,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#
        ).await.unwrap();
        
        db
    }
    
    #[tokio::test]
    async fn test_create_and_get() {
        let db = setup_test_db().await;
        let storage = EntityStorage::new(&db, "test");
        
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), JsonValue::String("Test Title".to_string()));
        fields.insert("count".to_string(), JsonValue::Number(serde_json::Number::from(42)));
        fields.insert("active".to_string(), JsonValue::Bool(true));
        
        let id = storage.create(fields).await.unwrap();
        
        let item = storage.get(&id).await.unwrap().unwrap();
        assert_eq!(item.id, id);
        assert_eq!(item.entity_type, "test");
        assert_eq!(item.fields.get("title").unwrap(), &JsonValue::String("Test Title".to_string()));
        assert_eq!(item.fields.get("count").unwrap(), &JsonValue::Number(serde_json::Number::from(42)));
    }
    
    #[tokio::test]
    async fn test_update() {
        let db = setup_test_db().await;
        let storage = EntityStorage::new(&db, "test");
        
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), JsonValue::String("Original".to_string()));
        
        let id = storage.create(fields).await.unwrap();
        
        let mut update_fields = HashMap::new();
        update_fields.insert("title".to_string(), JsonValue::String("Updated".to_string()));
        
        storage.update(&id, update_fields).await.unwrap();
        
        let item = storage.get(&id).await.unwrap().unwrap();
        assert_eq!(item.fields.get("title").unwrap(), &JsonValue::String("Updated".to_string()));
    }
    
    #[tokio::test]
    async fn test_delete() {
        let db = setup_test_db().await;
        let storage = EntityStorage::new(&db, "test");
        
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), JsonValue::String("To Delete".to_string()));
        
        let id = storage.create(fields).await.unwrap();
        
        storage.delete(&id).await.unwrap();
        
        let item = storage.get(&id).await.unwrap();
        assert!(item.is_none());
    }
}
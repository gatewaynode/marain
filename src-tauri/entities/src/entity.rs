use crate::{EntitiesError, Result};
use async_trait::async_trait;
use fields::{Field, FieldType};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

/// Entity definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDefinition {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub versioned: bool,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default = "default_cacheable")]
    pub cacheable: bool,
    pub fields: Vec<Field>,
}

pub fn default_cacheable() -> bool {
    true
}

impl EntityDefinition {
    /// Get the table name for this entity
    pub fn table_name(&self) -> String {
        format!("content_{}", self.id)
    }

    /// Get the field table name for a multi-value field
    pub fn field_table_name(&self, field_id: &str) -> String {
        format!("field_{}_{}", self.id, field_id)
    }
}

/// Trait for entity operations
#[async_trait]
pub trait Entity: Send + Sync {
    /// Get the entity definition
    fn definition(&self) -> &EntityDefinition;

    /// Create the database tables for this entity
    async fn create_tables(&self, pool: &Pool<Sqlite>) -> Result<()>;

    /// Drop the database tables for this entity
    async fn drop_tables(&self, pool: &Pool<Sqlite>) -> Result<()>;

    /// Check if the entity tables exist
    async fn tables_exist(&self, pool: &Pool<Sqlite>) -> Result<bool>;
}

/// Generic entity implementation
pub struct GenericEntity {
    definition: EntityDefinition,
}

impl GenericEntity {
    pub fn new(definition: EntityDefinition) -> Self {
        Self { definition }
    }

    /// Generate SQL for creating the main entity table
    fn generate_create_table_sql(&self) -> String {
        let mut columns = vec![
            "id TEXT PRIMARY KEY".to_string(), // ULID will be used for this field
            "user INTEGER DEFAULT 0".to_string(), // Add user field with default 0
            "rid INTEGER DEFAULT 1".to_string(), // Add revision ID field
            "last_cached TIMESTAMP".to_string(), // When entity was last cached to JSON cache
            "cache_ttl INTEGER DEFAULT 86400".to_string(), // Cache time-to-live in seconds (default 24 hours)
            "content_hash TEXT".to_string(), // Hash of all field values for change detection
            "created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP".to_string(),
            "updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP".to_string(),
        ];

        // Add columns for single-value fields
        for field in &self.definition.fields {
            if field.cardinality == 1 {
                let column = self.generate_column_sql(field);
                columns.push(column);
            } else {
                // Add field_reference column for multi-value fields
                let table_name = self.definition.field_table_name(&field.id);
                let reference_column =
                    format!("field_reference_{} TEXT DEFAULT '{}'", field.id, table_name);
                columns.push(reference_column);
            }
        }

        format!(
            "CREATE TABLE IF NOT EXISTS {} (\n    {}\n)",
            self.definition.table_name(),
            columns.join(",\n    ")
        )
    }

    /// Generate SQL for a field column
    fn generate_column_sql(&self, field: &Field) -> String {
        field.to_sql_column()
    }

    /// Generate SQL for creating a multi-value field table
    fn generate_field_table_sql(&self, field: &Field) -> String {
        let table_name = self.definition.field_table_name(&field.id);

        format!(
            r#"CREATE TABLE IF NOT EXISTS {} (
    id TEXT PRIMARY KEY,
    user INTEGER DEFAULT 0,
    rid INTEGER DEFAULT 1,
    parent_id TEXT NOT NULL,
    value TEXT NOT NULL,
    sort_order INTEGER,
    FOREIGN KEY (parent_id) REFERENCES {}(id) ON DELETE CASCADE
)"#,
            table_name,
            self.definition.table_name()
        )
    }

    /// Generate SQL for creating revision table for versioned entities
    fn generate_revision_table_sql(&self) -> Option<String> {
        if !self.definition.versioned {
            return None;
        }

        let mut columns = vec![
            "id TEXT".to_string(), // Not primary key in revision table
            "user INTEGER DEFAULT 0".to_string(),
            "rid INTEGER NOT NULL".to_string(),  // Revision ID
            "last_cached TIMESTAMP".to_string(), // When entity was last cached to JSON cache
            "cache_ttl INTEGER DEFAULT 86400".to_string(), // Cache time-to-live in seconds
            "content_hash TEXT".to_string(),     // Hash of all field values for change detection
            "created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP".to_string(),
            "updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP".to_string(),
            "revision_created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP".to_string(), // When this revision was created
        ];

        // Add columns for single-value fields
        for field in &self.definition.fields {
            if field.cardinality == 1 {
                let column = self.generate_column_sql(field);
                // Remove NOT NULL constraints for revision table
                let column = column.replace(" NOT NULL", "");
                columns.push(column);
            } else {
                // Add field_reference column for multi-value fields
                let table_name = self.definition.field_table_name(&field.id);
                let reference_column =
                    format!("field_reference_{} TEXT DEFAULT '{}'", field.id, table_name);
                columns.push(reference_column);
            }
        }

        Some(format!(
            "CREATE TABLE IF NOT EXISTS content_revisions_{} (\n    {},\n    PRIMARY KEY (id, rid)\n)",
            self.definition.id,
            columns.join(",\n    ")
        ))
    }

    /// Generate SQL for creating revision table for multi-value fields
    fn generate_field_revision_table_sql(&self, field: &Field) -> Option<String> {
        if !self.definition.versioned {
            return None;
        }

        let table_name = format!("field_revisions_{}_{}", self.definition.id, field.id);

        Some(format!(
            r#"CREATE TABLE IF NOT EXISTS {} (
    id TEXT,
    user INTEGER DEFAULT 0,
    rid INTEGER NOT NULL,
    parent_id TEXT NOT NULL,
    value TEXT NOT NULL,
    sort_order INTEGER,
    revision_created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id, rid),
    FOREIGN KEY (parent_id) REFERENCES {}(id) ON DELETE CASCADE
)"#,
            table_name,
            self.definition.table_name()
        ))
    }

    /// Generate SQL for creating indexes
    fn generate_index_sql(&self) -> Vec<String> {
        let mut indexes = Vec::new();

        // Index for ID field (ULID) for performance
        indexes.push(format!(
            "CREATE INDEX IF NOT EXISTS idx_{}_id ON {}(id)",
            self.definition.id,
            self.definition.table_name()
        ));

        // Index for slug fields
        for field in &self.definition.fields {
            if field.field_type == FieldType::Slug && field.cardinality == 1 {
                indexes.push(format!(
                    "CREATE INDEX IF NOT EXISTS idx_{}_{} ON {}({})",
                    self.definition.id,
                    field.id,
                    self.definition.table_name(),
                    field.id
                ));
            }

            // Index for multi-value field tables
            if field.cardinality != 1 {
                let table_name = self.definition.field_table_name(&field.id);
                indexes.push(format!(
                    "CREATE INDEX IF NOT EXISTS idx_{}_parent ON {}(parent_id)",
                    table_name, table_name
                ));
                // Also add ID index for multi-value tables
                indexes.push(format!(
                    "CREATE INDEX IF NOT EXISTS idx_{}_id ON {}(id)",
                    table_name, table_name
                ));
            }
        }

        // Add indexes for revision tables if versioned
        if self.definition.versioned {
            // Index for main revision table
            indexes.push(format!(
                "CREATE INDEX IF NOT EXISTS idx_revisions_{}_id ON content_revisions_{}(id)",
                self.definition.id, self.definition.id
            ));
            indexes.push(format!(
                "CREATE INDEX IF NOT EXISTS idx_revisions_{}_rid ON content_revisions_{}(rid)",
                self.definition.id, self.definition.id
            ));

            // Indexes for multi-value field revision tables
            for field in &self.definition.fields {
                if field.cardinality != 1 {
                    let table_name = format!("field_revisions_{}_{}", self.definition.id, field.id);
                    indexes.push(format!(
                        "CREATE INDEX IF NOT EXISTS idx_{}_parent ON {}(parent_id)",
                        table_name, table_name
                    ));
                    indexes.push(format!(
                        "CREATE INDEX IF NOT EXISTS idx_{}_rid ON {}(rid)",
                        table_name, table_name
                    ));
                }
            }
        }

        indexes
    }

    /// Execute raw SQL on the pool
    async fn execute_raw(pool: &Pool<Sqlite>, sql: &str) -> Result<()> {
        sqlx::query(sql)
            .execute(pool)
            .await
            .map_err(|e| EntitiesError::SqlExecution(e.to_string()))?;
        Ok(())
    }

    /// Check if a table exists
    async fn table_exists(pool: &Pool<Sqlite>, table_name: &str) -> Result<bool> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM sqlite_master
            WHERE type='table' AND name=?
        "#;

        let result: (i32,) = sqlx::query_as(query)
            .bind(table_name)
            .fetch_one(pool)
            .await?;

        Ok(result.0 > 0)
    }
}

#[async_trait]
impl Entity for GenericEntity {
    fn definition(&self) -> &EntityDefinition {
        &self.definition
    }

    async fn create_tables(&self, pool: &Pool<Sqlite>) -> Result<()> {
        // Create main entity table
        let create_sql = self.generate_create_table_sql();
        Self::execute_raw(pool, &create_sql).await?;

        // Create revision table for versioned entities
        if let Some(revision_sql) = self.generate_revision_table_sql() {
            Self::execute_raw(pool, &revision_sql).await?;
        }

        // Create multi-value field tables
        for field in &self.definition.fields {
            if field.cardinality != 1 {
                let field_sql = self.generate_field_table_sql(field);
                Self::execute_raw(pool, &field_sql).await?;

                // Create revision table for multi-value fields if entity is versioned
                if let Some(field_revision_sql) = self.generate_field_revision_table_sql(field) {
                    Self::execute_raw(pool, &field_revision_sql).await?;
                }
            }
        }

        // Create indexes
        for index_sql in self.generate_index_sql() {
            Self::execute_raw(pool, &index_sql).await?;
        }

        Ok(())
    }

    async fn drop_tables(&self, pool: &Pool<Sqlite>) -> Result<()> {
        // Drop multi-value field tables first (due to foreign key constraints)
        for field in &self.definition.fields {
            if field.cardinality != 1 {
                let table_name = self.definition.field_table_name(&field.id);
                let drop_sql = format!("DROP TABLE IF EXISTS {}", table_name);
                Self::execute_raw(pool, &drop_sql).await?;
            }
        }

        // Drop main entity table
        let drop_sql = format!("DROP TABLE IF EXISTS {}", self.definition.table_name());
        Self::execute_raw(pool, &drop_sql).await?;

        Ok(())
    }

    async fn tables_exist(&self, pool: &Pool<Sqlite>) -> Result<bool> {
        Self::table_exists(pool, &self.definition.table_name()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_table_name() {
        let def = EntityDefinition {
            id: "article".to_string(),
            name: "Article".to_string(),
            description: None,
            versioned: false,
            recursive: false,
            cacheable: true,
            fields: vec![],
        };

        assert_eq!(def.table_name(), "content_article");
        assert_eq!(def.field_table_name("tags"), "field_article_tags");
    }

    #[test]
    fn test_generate_column_sql() {
        let entity = GenericEntity::new(EntityDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            versioned: false,
            recursive: false,
            cacheable: true,
            fields: vec![],
        });

        let text_field = Field {
            id: "title".to_string(),
            field_type: FieldType::Text,
            label: "Title".to_string(),
            required: true,
            description: None,
            cardinality: 1,
            target_entity: None,
            fields: None,
        };

        assert_eq!(
            entity.generate_column_sql(&text_field),
            "title TEXT NOT NULL"
        );

        let slug_field = Field {
            id: "slug".to_string(),
            field_type: FieldType::Slug,
            label: "Slug".to_string(),
            required: false,
            description: None,
            cardinality: 1,
            target_entity: None,
            fields: None,
        };

        assert_eq!(entity.generate_column_sql(&slug_field), "slug TEXT UNIQUE");
    }
}

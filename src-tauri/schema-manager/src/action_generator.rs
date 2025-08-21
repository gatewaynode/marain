use crate::diff_engine::ConfigDiff;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

/// Represents an action that needs to be executed based on a configuration change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    /// Create a new database table for an entity
    CreateTable {
        entity_id: String,
        table_name: String,
        sql: String,
    },
    /// Drop an existing database table
    DropTable {
        entity_id: String,
        table_name: String,
    },
    /// Add a column to an existing table
    AddColumn {
        entity_id: String,
        table_name: String,
        column_name: String,
        sql: String,
    },
    /// Drop a column from an existing table
    DropColumn {
        entity_id: String,
        table_name: String,
        column_name: String,
    },
    /// Modify a column in an existing table
    ModifyColumn {
        entity_id: String,
        table_name: String,
        column_name: String,
        old_type: String,
        new_type: String,
        sql: String,
    },
    /// Update in-memory configuration
    UpdateConfig { key: String, value: Value },
    /// Invalidate cache for an entity
    InvalidateCache { entity_id: String },
    /// Create an index on a table
    CreateIndex {
        index_name: String,
        table_name: String,
        columns: Vec<String>,
        sql: String,
    },
    /// Drop an index from a table
    DropIndex {
        index_name: String,
        table_name: String,
    },
    /// Reload entity definitions
    ReloadEntityDefinitions,
}

impl Action {
    /// Check if this action is reversible
    pub fn is_reversible(&self) -> bool {
        match self {
            Action::CreateTable { .. } => true,
            Action::DropTable { .. } => false, // Can't reverse without data
            Action::AddColumn { .. } => true,
            Action::DropColumn { .. } => false, // Can't reverse without data
            Action::ModifyColumn { .. } => false, // May lose data
            Action::UpdateConfig { .. } => true,
            Action::InvalidateCache { .. } => true,
            Action::CreateIndex { .. } => true,
            Action::DropIndex { .. } => true,
            Action::ReloadEntityDefinitions => true,
        }
    }

    /// Generate a rollback action for this action
    pub fn rollback_action(&self) -> Option<Action> {
        match self {
            Action::CreateTable {
                entity_id,
                table_name,
                ..
            } => Some(Action::DropTable {
                entity_id: entity_id.clone(),
                table_name: table_name.clone(),
            }),
            Action::AddColumn {
                entity_id,
                table_name,
                column_name,
                ..
            } => Some(Action::DropColumn {
                entity_id: entity_id.clone(),
                table_name: table_name.clone(),
                column_name: column_name.clone(),
            }),
            Action::CreateIndex {
                index_name,
                table_name,
                ..
            } => Some(Action::DropIndex {
                index_name: index_name.clone(),
                table_name: table_name.clone(),
            }),
            Action::DropIndex {
                index_name: _,
                table_name: _,
            } => {
                // We'd need to store the original index definition to recreate it
                None
            }
            _ => None,
        }
    }
}

/// Generates actions based on configuration differences
pub struct ActionGenerator;

impl ActionGenerator {
    /// Generate actions from a configuration diff
    pub fn generate_actions(file_type: FileType, diff: &ConfigDiff) -> Result<Vec<Action>, String> {
        match file_type {
            FileType::EntitySchema => Self::generate_entity_actions(diff),
            FileType::SystemConfig => Self::generate_config_actions(diff),
            FileType::FieldGroup => Self::generate_field_group_actions(diff),
            FileType::Unknown => Ok(vec![]),
        }
    }

    /// Generate actions for entity schema changes
    fn generate_entity_actions(diff: &ConfigDiff) -> Result<Vec<Action>, String> {
        let mut actions = Vec::new();

        // Handle new entities
        for (entity_id, entity_def) in &diff.added {
            if let Some(entity_map) = entity_def.as_mapping() {
                let table_name = format!("content_{}", entity_id);
                let sql = Self::generate_create_table_sql(entity_id, entity_map)?;

                actions.push(Action::CreateTable {
                    entity_id: entity_id.clone(),
                    table_name: table_name.clone(),
                    sql,
                });

                // Generate index actions for the new entity
                if let Some(fields) = entity_map.get(Value::String("fields".to_string())) {
                    if let Some(fields_seq) = fields.as_sequence() {
                        for field in fields_seq {
                            if let Some(field_map) = field.as_mapping() {
                                if let Some(field_id) =
                                    field_map.get(Value::String("id".to_string()))
                                {
                                    if let Some(field_type) =
                                        field_map.get(Value::String("type".to_string()))
                                    {
                                        if field_type == &Value::String("slug".to_string()) {
                                            let index_name = format!(
                                                "idx_{}_{}",
                                                entity_id,
                                                field_id.as_str().unwrap_or("")
                                            );
                                            let sql = format!(
                                                "CREATE INDEX IF NOT EXISTS {} ON {}({})",
                                                index_name,
                                                table_name,
                                                field_id.as_str().unwrap_or("")
                                            );
                                            actions.push(Action::CreateIndex {
                                                index_name,
                                                table_name: table_name.clone(),
                                                columns: vec![field_id
                                                    .as_str()
                                                    .unwrap_or("")
                                                    .to_string()],
                                                sql,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Handle removed entities
        for entity_id in diff.removed.keys() {
            actions.push(Action::DropTable {
                entity_id: entity_id.clone(),
                table_name: format!("content_{}", entity_id),
            });
        }

        // Handle modified entities
        for (entity_id, changes) in &diff.modified {
            let table_name = format!("content_{}", entity_id);

            // Check for field additions
            if let Some(field_changes) = changes.get("fields") {
                if let Some(field_diff) = field_changes.as_mapping() {
                    // Handle added fields
                    if let Some(added_fields) = field_diff.get(Value::String("added".to_string())) {
                        if let Some(fields_map) = added_fields.as_mapping() {
                            for (field_id, field_def) in fields_map {
                                if let Some(field_id_str) = field_id.as_str() {
                                    let sql = Self::generate_add_column_sql(
                                        &table_name,
                                        field_id_str,
                                        field_def,
                                    )?;
                                    actions.push(Action::AddColumn {
                                        entity_id: entity_id.clone(),
                                        table_name: table_name.clone(),
                                        column_name: field_id_str.to_string(),
                                        sql,
                                    });
                                }
                            }
                        }
                    }

                    // Handle removed fields
                    if let Some(removed_fields) =
                        field_diff.get(Value::String("removed".to_string()))
                    {
                        if let Some(fields_map) = removed_fields.as_mapping() {
                            for (field_id, _) in fields_map {
                                if let Some(field_id_str) = field_id.as_str() {
                                    actions.push(Action::DropColumn {
                                        entity_id: entity_id.clone(),
                                        table_name: table_name.clone(),
                                        column_name: field_id_str.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Always reload entity definitions after schema changes
        if !actions.is_empty() {
            actions.push(Action::ReloadEntityDefinitions);
        }

        Ok(actions)
    }

    /// Generate actions for system configuration changes
    fn generate_config_actions(diff: &ConfigDiff) -> Result<Vec<Action>, String> {
        let mut actions = Vec::new();

        // Handle added configuration keys
        for (key, value) in &diff.added {
            actions.push(Action::UpdateConfig {
                key: key.clone(),
                value: value.clone(),
            });
        }

        // Handle modified configuration values
        for (key, changes) in &diff.modified {
            if let Some(new_value) = changes.get("new") {
                actions.push(Action::UpdateConfig {
                    key: key.clone(),
                    value: new_value.clone(),
                });
            }
        }

        // Handle removed configuration keys
        for key in diff.removed.keys() {
            actions.push(Action::UpdateConfig {
                key: key.clone(),
                value: Value::Null,
            });
        }

        Ok(actions)
    }

    /// Generate actions for field group changes
    fn generate_field_group_actions(diff: &ConfigDiff) -> Result<Vec<Action>, String> {
        // Field groups don't directly affect database structure
        // They are used as templates when creating entities
        // So we just need to reload entity definitions
        if !diff.added.is_empty() || !diff.removed.is_empty() || !diff.modified.is_empty() {
            Ok(vec![Action::ReloadEntityDefinitions])
        } else {
            Ok(vec![])
        }
    }

    /// Generate SQL for creating a table from an entity definition
    fn generate_create_table_sql(
        entity_id: &str,
        entity_def: &serde_yaml::Mapping,
    ) -> Result<String, String> {
        let mut columns = vec![
            "id TEXT PRIMARY KEY".to_string(),
            "created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP".to_string(),
            "updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP".to_string(),
        ];

        // Add columns for fields
        if let Some(fields) = entity_def.get(Value::String("fields".to_string())) {
            if let Some(fields_seq) = fields.as_sequence() {
                for field in fields_seq {
                    if let Some(field_map) = field.as_mapping() {
                        let field_id = field_map
                            .get(Value::String("id".to_string()))
                            .and_then(|v| v.as_str())
                            .ok_or("Field missing 'id'")?;

                        let field_type = field_map
                            .get(Value::String("type".to_string()))
                            .and_then(|v| v.as_str())
                            .ok_or("Field missing 'type'")?;

                        let cardinality = field_map
                            .get(Value::String("cardinality".to_string()))
                            .and_then(|v| v.as_i64())
                            .unwrap_or(1);

                        // Only add column for single-value fields
                        if cardinality == 1 {
                            let sql_type = Self::field_type_to_sql(field_type);
                            let required = field_map
                                .get(Value::String("required".to_string()))
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);

                            let mut column_def = format!("{} {}", field_id, sql_type);
                            if required {
                                column_def.push_str(" NOT NULL");
                            }
                            if field_type == "slug" {
                                column_def.push_str(" UNIQUE");
                            }

                            columns.push(column_def);
                        }
                    }
                }
            }
        }

        Ok(format!(
            "CREATE TABLE IF NOT EXISTS content_{} (\n    {}\n)",
            entity_id,
            columns.join(",\n    ")
        ))
    }

    /// Generate SQL for adding a column to a table
    fn generate_add_column_sql(
        table_name: &str,
        field_id: &str,
        field_def: &Value,
    ) -> Result<String, String> {
        if let Some(field_map) = field_def.as_mapping() {
            let field_type = field_map
                .get(Value::String("type".to_string()))
                .and_then(|v| v.as_str())
                .ok_or("Field missing 'type'")?;

            let sql_type = Self::field_type_to_sql(field_type);
            let required = field_map
                .get(Value::String("required".to_string()))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let mut column_def = format!(
                "ALTER TABLE {} ADD COLUMN {} {}",
                table_name, field_id, sql_type
            );
            if required {
                column_def.push_str(" NOT NULL DEFAULT ''"); // Need default for NOT NULL columns
            }

            Ok(column_def)
        } else {
            Err("Invalid field definition".to_string())
        }
    }

    /// Convert field type to SQL type
    fn field_type_to_sql(field_type: &str) -> &'static str {
        match field_type {
            "text" | "long_text" | "rich_text" | "slug" => "TEXT",
            "integer" => "INTEGER",
            "float" => "REAL",
            "boolean" => "INTEGER",
            "datetime" => "TIMESTAMP",
            "entity_reference" => "TEXT",
            "component" => "TEXT",
            _ => "TEXT",
        }
    }
}

/// Type of configuration file
#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    EntitySchema,
    SystemConfig,
    FieldGroup,
    Unknown,
}

impl FileType {
    /// Determine file type from path
    pub fn from_path(path: &std::path::Path) -> Self {
        if let Some(parent) = path.parent() {
            if let Some(parent_name) = parent.file_name() {
                let parent_str = parent_name.to_string_lossy();
                if parent_str == "schemas" {
                    if let Some(stem) = path.file_stem() {
                        let filename = stem.to_string_lossy();
                        if filename.ends_with(".schema") {
                            return FileType::EntitySchema;
                        } else if filename.starts_with("field_group") {
                            return FileType::FieldGroup;
                        }
                    }
                } else if parent_str == "config" {
                    return FileType::SystemConfig;
                }
            }
        }
        FileType::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_detection() {
        let schema_path = std::path::Path::new("schemas/article.schema.yaml");
        assert_eq!(FileType::from_path(schema_path), FileType::EntitySchema);

        let config_path = std::path::Path::new("config/system.dev.yaml");
        assert_eq!(FileType::from_path(config_path), FileType::SystemConfig);

        let unknown_path = std::path::Path::new("data/test.yaml");
        assert_eq!(FileType::from_path(unknown_path), FileType::Unknown);
    }

    #[test]
    fn test_action_reversibility() {
        let create_action = Action::CreateTable {
            entity_id: "test".to_string(),
            table_name: "content_test".to_string(),
            sql: "CREATE TABLE content_test (id TEXT)".to_string(),
        };
        assert!(create_action.is_reversible());
        assert!(create_action.rollback_action().is_some());

        let drop_action = Action::DropTable {
            entity_id: "test".to_string(),
            table_name: "content_test".to_string(),
        };
        assert!(!drop_action.is_reversible());
        assert!(drop_action.rollback_action().is_none());
    }
}

use crate::entity::{EntityDefinition, GenericEntity, Entity};
use crate::error::{EntitiesError, Result};
use fields::{Field, FieldType};
use std::path::Path;
use tracing::{debug, info};

/// Load entity definitions from YAML files
pub struct SchemaLoader;

impl SchemaLoader {
    /// Load a single entity definition from a YAML file
    pub async fn load_entity_from_file(path: &Path) -> Result<Box<dyn Entity>> {
        debug!("Loading entity schema from: {:?}", path);
        
        let content = std::fs::read_to_string(path)
            .map_err(|e| EntitiesError::SchemaParsing(format!("Failed to read file: {}", e)))?;
        
        let definition: EntityDefinition = serde_yaml::from_str(&content)
            .map_err(|e| EntitiesError::SchemaParsing(format!("Failed to parse YAML: {}", e)))?;
        
        info!("Loaded entity '{}' from {:?}", definition.id, path);
        
        Ok(Box::new(GenericEntity::new(definition)))
    }
    
    /// Load all entity definitions from a directory
    pub async fn load_entities_from_directory(dir: &Path) -> Result<Vec<Box<dyn Entity>>> {
        info!("Loading entity schemas from directory: {:?}", dir);
        
        if !dir.exists() {
            return Err(EntitiesError::SchemaParsing(format!(
                "Schema directory does not exist: {:?}",
                dir
            )));
        }
        
        let mut entities = Vec::new();
        
        let entries = std::fs::read_dir(dir)
            .map_err(|e| EntitiesError::SchemaParsing(format!("Failed to read directory: {}", e)))?;
        
        for entry in entries {
            let entry = entry
                .map_err(|e| EntitiesError::SchemaParsing(format!("Failed to read directory entry: {}", e)))?;
            
            let path = entry.path();
            
            // Only process .yaml and .yml files
            if let Some(extension) = path.extension() {
                if extension == "yaml" || extension == "yml" {
                    // Skip field group schemas (they don't have _entity suffix)
                    if let Some(stem) = path.file_stem() {
                        let filename = stem.to_string_lossy();
                        if filename.ends_with(".schema") {
                            match Self::load_entity_from_file(&path).await {
                                Ok(entity) => entities.push(entity),
                                Err(e) => {
                                    // Log error but continue loading other schemas
                                    tracing::error!("Failed to load schema from {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        info!("Loaded {} entity schemas", entities.len());
        
        Ok(entities)
    }
    
    /// Validate an entity definition
    pub fn validate_definition(definition: &EntityDefinition) -> Result<()> {
        // Validate entity ID
        if definition.id.is_empty() {
            return Err(EntitiesError::Validation("Entity ID cannot be empty".to_string()));
        }
        
        // Validate entity name
        if definition.name.is_empty() {
            return Err(EntitiesError::Validation("Entity name cannot be empty".to_string()));
        }
        
        // Validate fields
        for field in &definition.fields {
            Self::validate_field(field)?;
        }
        
        Ok(())
    }
    
    /// Validate a field definition
    fn validate_field(field: &Field) -> Result<()> {
        // Validate field ID
        if field.id.is_empty() {
            return Err(EntitiesError::Validation("Field ID cannot be empty".to_string()));
        }
        
        // Validate field label
        if field.label.is_empty() {
            return Err(EntitiesError::Validation(format!(
                "Field '{}' label cannot be empty",
                field.id
            )));
        }
        
        // Validate entity reference fields
        if field.field_type == FieldType::EntityReference && field.target_entity.is_none() {
            return Err(EntitiesError::Validation(format!(
                "Field '{}' is an entity_reference but has no target_entity",
                field.id
            )));
        }
        
        // Validate component fields
        if field.field_type == FieldType::Component && field.fields.is_none() {
            return Err(EntitiesError::Validation(format!(
                "Field '{}' is a component but has no nested fields",
                field.id
            )));
        }
        
        // Validate nested fields in components
        if let Some(nested_fields) = &field.fields {
            for nested_field in nested_fields {
                Self::validate_field(nested_field)?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fields::{Field, FieldType};
    use std::fs;
    
    fn get_test_schema_path() -> std::path::PathBuf {
        // Get the project root by going up from the test binary location
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let project_root = std::path::Path::new(manifest_dir)
            .parent() // src-tauri
            .unwrap()
            .parent() // project root
            .unwrap();
        
        let schema_dir = project_root.join("data").join("test_schemas");
        std::fs::create_dir_all(&schema_dir).unwrap();
        
        schema_dir.join("test.schema.yaml")
    }
    
    #[tokio::test]
    async fn test_load_entity_from_file() {
        let schema_path = get_test_schema_path();
        
        let yaml_content = r#"
id: test_entity
name: Test Entity
description: A test entity
fields:
  - id: title
    type: text
    label: Title
    required: true
  - id: count
    type: integer
    label: Count
"#;
        
        fs::write(&schema_path, yaml_content).unwrap();
        
        let entity = SchemaLoader::load_entity_from_file(&schema_path).await.unwrap();
        let definition = entity.definition();
        
        assert_eq!(definition.id, "test_entity");
        assert_eq!(definition.name, "Test Entity");
        assert_eq!(definition.fields.len(), 2);
        assert_eq!(definition.fields[0].id, "title");
        assert_eq!(definition.fields[0].field_type, FieldType::Text);
        assert!(definition.fields[0].required);
    }
    
    #[test]
    fn test_validate_definition() {
        let valid_definition = EntityDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: None,
            versioned: false,
            recursive: false,
            cacheable: true,
            fields: vec![
                Field {
                    id: "title".to_string(),
                    field_type: FieldType::Text,
                    label: "Title".to_string(),
                    required: true,
                    description: None,
                    cardinality: 1,
                    target_entity: None,
                    fields: None,
                },
            ],
        };
        
        assert!(SchemaLoader::validate_definition(&valid_definition).is_ok());
        
        // Test invalid entity ID
        let mut invalid_definition = valid_definition.clone();
        invalid_definition.id = "".to_string();
        assert!(SchemaLoader::validate_definition(&invalid_definition).is_err());
        
        // Test invalid field
        let mut invalid_field_definition = valid_definition.clone();
        invalid_field_definition.fields[0].id = "".to_string();
        assert!(SchemaLoader::validate_definition(&invalid_field_definition).is_err());
        
        // Test entity reference without target
        let mut invalid_ref_definition = valid_definition.clone();
        invalid_ref_definition.fields[0].field_type = FieldType::EntityReference;
        invalid_ref_definition.fields[0].target_entity = None;
        assert!(SchemaLoader::validate_definition(&invalid_ref_definition).is_err());
    }
}
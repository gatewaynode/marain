use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub mod field_types;
pub mod validation;
pub mod error;

pub use field_types::{FieldType, Field};
pub use validation::FieldValidator;
pub use error::{FieldsError, Result};

/// Default cardinality value for fields
pub fn default_cardinality() -> i32 {
    1
}

/// Trait for field value conversion and validation
pub trait FieldValue: Serialize + for<'de> Deserialize<'de> {
    /// Convert the field value to JSON
    fn to_json(&self) -> JsonValue;
    
    /// Create from JSON value
    fn from_json(value: &JsonValue) -> Result<Self> where Self: Sized;
    
    /// Validate the field value
    fn validate(&self) -> Result<()>;
}

/// Field metadata for runtime field information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMetadata {
    pub id: String,
    pub field_type: FieldType,
    pub label: String,
    pub required: bool,
    pub description: Option<String>,
    pub cardinality: i32,
    pub constraints: HashMap<String, JsonValue>,
}

impl FieldMetadata {
    /// Check if this field allows multiple values
    pub fn is_multi_value(&self) -> bool {
        self.cardinality != 1
    }
    
    /// Check if this field is unlimited cardinality
    pub fn is_unlimited(&self) -> bool {
        self.cardinality == -1
    }
    
    /// Get the maximum number of values allowed
    pub fn max_values(&self) -> Option<usize> {
        if self.cardinality == -1 {
            None
        } else if self.cardinality > 0 {
            Some(self.cardinality as usize)
        } else {
            Some(1)
        }
    }
}

/// Field group for reusable field collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldGroup {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<Field>,
}

/// Field collection for managing multiple fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCollection {
    fields: Vec<Field>,
    metadata: HashMap<String, FieldMetadata>,
}

impl FieldCollection {
    /// Create a new field collection
    pub fn new(fields: Vec<Field>) -> Self {
        let mut metadata = HashMap::new();
        for field in &fields {
            metadata.insert(
                field.id.clone(),
                FieldMetadata {
                    id: field.id.clone(),
                    field_type: field.field_type.clone(),
                    label: field.label.clone(),
                    required: field.required,
                    description: field.description.clone(),
                    cardinality: field.cardinality,
                    constraints: HashMap::new(),
                },
            );
        }
        
        Self { fields, metadata }
    }
    
    /// Get all fields
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
    
    /// Get field by ID
    pub fn get_field(&self, id: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.id == id)
    }
    
    /// Get field metadata by ID
    pub fn get_metadata(&self, id: &str) -> Option<&FieldMetadata> {
        self.metadata.get(id)
    }
    
    /// Get all single-value fields
    pub fn single_value_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.cardinality == 1).collect()
    }
    
    /// Get all multi-value fields
    pub fn multi_value_fields(&self) -> Vec<&Field> {
        self.fields.iter().filter(|f| f.cardinality != 1).collect()
    }
    
    /// Validate a set of field values
    pub fn validate_values(&self, values: &HashMap<String, JsonValue>) -> Result<()> {
        for field in &self.fields {
            if field.required && !values.contains_key(&field.id) {
                return Err(FieldsError::Validation(format!(
                    "Required field '{}' is missing",
                    field.id
                )));
            }
            
            if let Some(value) = values.get(&field.id) {
                FieldValidator::validate_field_value(field, value)?;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_field_metadata() {
        let metadata = FieldMetadata {
            id: "test".to_string(),
            field_type: FieldType::Text,
            label: "Test Field".to_string(),
            required: true,
            description: None,
            cardinality: -1,
            constraints: HashMap::new(),
        };
        
        assert!(metadata.is_multi_value());
        assert!(metadata.is_unlimited());
        assert_eq!(metadata.max_values(), None);
        
        let single_metadata = FieldMetadata {
            id: "single".to_string(),
            field_type: FieldType::Text,
            label: "Single Field".to_string(),
            required: false,
            description: None,
            cardinality: 1,
            constraints: HashMap::new(),
        };
        
        assert!(!single_metadata.is_multi_value());
        assert!(!single_metadata.is_unlimited());
        assert_eq!(single_metadata.max_values(), Some(1));
    }
}
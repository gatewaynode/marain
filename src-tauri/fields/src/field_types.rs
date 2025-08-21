use serde::{Deserialize, Serialize};

/// Field types supported by the field system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    LongText,
    RichText,
    Integer,
    Float,
    Boolean,
    Datetime,
    Slug,
    EntityReference,
    Component,
}

impl FieldType {
    /// Get the SQL type for this field type
    pub fn sql_type(&self) -> &'static str {
        match self {
            FieldType::Text | FieldType::LongText | FieldType::RichText | FieldType::Slug => "TEXT",
            FieldType::Integer => "INTEGER",
            FieldType::Float => "REAL",
            FieldType::Boolean => "INTEGER", // SQLite uses 0/1 for boolean
            FieldType::Datetime => "TIMESTAMP",
            FieldType::EntityReference => "TEXT", // Store reference ID
            FieldType::Component => "TEXT",       // Store as JSON
        }
    }

    /// Check if this field type requires additional configuration
    pub fn requires_config(&self) -> bool {
        matches!(self, FieldType::EntityReference | FieldType::Component)
    }

    /// Check if this field type should have a unique constraint by default
    pub fn is_unique_by_default(&self) -> bool {
        matches!(self, FieldType::Slug)
    }

    /// Check if this field type stores structured data
    pub fn is_structured(&self) -> bool {
        matches!(self, FieldType::Component)
    }

    /// Check if this field type is a reference to another entity
    pub fn is_reference(&self) -> bool {
        matches!(self, FieldType::EntityReference)
    }
}

/// Field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub id: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    pub label: String,
    #[serde(default)]
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_cardinality")]
    pub cardinality: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_entity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<Field>>,
}

fn default_cardinality() -> i32 {
    1
}

impl Field {
    /// Create a new field with minimal configuration
    pub fn new(id: impl Into<String>, field_type: FieldType, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            field_type,
            label: label.into(),
            required: false,
            description: None,
            cardinality: 1,
            target_entity: None,
            fields: None,
        }
    }

    /// Set the field as required
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set the field description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the field cardinality
    pub fn with_cardinality(mut self, cardinality: i32) -> Self {
        self.cardinality = cardinality;
        self
    }

    /// Set the target entity for entity reference fields
    pub fn with_target_entity(mut self, target: impl Into<String>) -> Self {
        self.target_entity = Some(target.into());
        self
    }

    /// Set nested fields for component fields
    pub fn with_fields(mut self, fields: Vec<Field>) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Check if this field allows multiple values
    pub fn is_multi_value(&self) -> bool {
        self.cardinality != 1
    }

    /// Check if this field has unlimited cardinality
    pub fn is_unlimited(&self) -> bool {
        self.cardinality == -1
    }

    /// Get the SQL column definition for this field
    pub fn to_sql_column(&self) -> String {
        let mut column_def = format!("{} {}", self.id, self.field_type.sql_type());

        if self.required {
            column_def.push_str(" NOT NULL");
        }

        if self.field_type.is_unique_by_default() {
            column_def.push_str(" UNIQUE");
        }

        column_def
    }

    /// Validate the field configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate field ID
        if self.id.is_empty() {
            return Err("Field ID cannot be empty".to_string());
        }

        // Validate field label
        if self.label.is_empty() {
            return Err(format!("Field '{}' label cannot be empty", self.id));
        }

        // Validate entity reference fields
        if self.field_type == FieldType::EntityReference && self.target_entity.is_none() {
            return Err(format!(
                "Field '{}' is an entity_reference but has no target_entity",
                self.id
            ));
        }

        // Validate component fields
        if self.field_type == FieldType::Component && self.fields.is_none() {
            return Err(format!(
                "Field '{}' is a component but has no nested fields",
                self.id
            ));
        }

        // Validate nested fields in components
        if let Some(nested_fields) = &self.fields {
            for nested_field in nested_fields {
                nested_field.validate()?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_type_properties() {
        assert_eq!(FieldType::Text.sql_type(), "TEXT");
        assert_eq!(FieldType::Integer.sql_type(), "INTEGER");
        assert_eq!(FieldType::Boolean.sql_type(), "INTEGER");

        assert!(FieldType::EntityReference.requires_config());
        assert!(FieldType::Component.requires_config());
        assert!(!FieldType::Text.requires_config());

        assert!(FieldType::Slug.is_unique_by_default());
        assert!(!FieldType::Text.is_unique_by_default());

        assert!(FieldType::Component.is_structured());
        assert!(!FieldType::Text.is_structured());

        assert!(FieldType::EntityReference.is_reference());
        assert!(!FieldType::Text.is_reference());
    }

    #[test]
    fn test_field_builder() {
        let field = Field::new("test_field", FieldType::Text, "Test Field")
            .required(true)
            .with_description("A test field")
            .with_cardinality(5);

        assert_eq!(field.id, "test_field");
        assert_eq!(field.field_type, FieldType::Text);
        assert_eq!(field.label, "Test Field");
        assert!(field.required);
        assert_eq!(field.description, Some("A test field".to_string()));
        assert_eq!(field.cardinality, 5);
        assert!(field.is_multi_value());
        assert!(!field.is_unlimited());
    }

    #[test]
    fn test_field_validation() {
        let valid_field = Field::new("test", FieldType::Text, "Test");
        assert!(valid_field.validate().is_ok());

        let mut invalid_field = Field::new("", FieldType::Text, "Test");
        assert!(invalid_field.validate().is_err());

        invalid_field.id = "test".to_string();
        invalid_field.label = "".to_string();
        assert!(invalid_field.validate().is_err());

        let invalid_ref = Field::new("ref", FieldType::EntityReference, "Reference");
        assert!(invalid_ref.validate().is_err());

        let valid_ref =
            Field::new("ref", FieldType::EntityReference, "Reference").with_target_entity("user");
        assert!(valid_ref.validate().is_ok());
    }

    #[test]
    fn test_sql_column_generation() {
        let text_field = Field::new("title", FieldType::Text, "Title").required(true);
        assert_eq!(text_field.to_sql_column(), "title TEXT NOT NULL");

        let slug_field = Field::new("slug", FieldType::Slug, "Slug");
        assert_eq!(slug_field.to_sql_column(), "slug TEXT UNIQUE");

        let int_field = Field::new("count", FieldType::Integer, "Count");
        assert_eq!(int_field.to_sql_column(), "count INTEGER");
    }
}

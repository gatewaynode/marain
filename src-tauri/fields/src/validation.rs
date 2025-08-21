use crate::{Field, FieldType, FieldsError, Result};
use serde_json::Value as JsonValue;

/// Field validator for validating field values
pub struct FieldValidator;

impl FieldValidator {
    /// Validate a field value against its field definition
    pub fn validate_field_value(field: &Field, value: &JsonValue) -> Result<()> {
        // Check for null values
        if value.is_null() {
            if field.required {
                return Err(FieldsError::Validation(format!(
                    "Required field '{}' cannot be null",
                    field.id
                )));
            }
            return Ok(());
        }

        // For multi-value fields, handle arrays specially
        if field.cardinality != 1 {
            if let Some(arr) = value.as_array() {
                // Validate cardinality constraints
                if field.cardinality > 0 {
                    let max_values = field.cardinality as usize;
                    if arr.len() > max_values {
                        return Err(FieldsError::CardinalityViolation(format!(
                            "Field '{}' allows maximum {} values, but {} were provided",
                            field.id,
                            max_values,
                            arr.len()
                        )));
                    }
                }

                // Validate each element in the array
                for element in arr {
                    Self::validate_single_value(field, element)?;
                }
                return Ok(());
            } else {
                // For multi-value fields, accept single values and treat them as array of one
                Self::validate_single_value(field, value)?;
                return Ok(());
            }
        }

        // For single-value fields, validate normally
        if value.is_array() {
            return Err(FieldsError::CardinalityViolation(format!(
                "Field '{}' expects a single value, not an array",
                field.id
            )));
        }

        Self::validate_single_value(field, value)?;

        Ok(())
    }

    /// Validate a single value based on field type
    fn validate_single_value(field: &Field, value: &JsonValue) -> Result<()> {
        // Validate based on field type
        match field.field_type {
            FieldType::Text | FieldType::LongText | FieldType::RichText => {
                Self::validate_text(field, value)?;
            }
            FieldType::Slug => {
                Self::validate_slug(field, value)?;
            }
            FieldType::Integer => {
                Self::validate_integer(field, value)?;
            }
            FieldType::Float => {
                Self::validate_float(field, value)?;
            }
            FieldType::Boolean => {
                Self::validate_boolean(field, value)?;
            }
            FieldType::Datetime => {
                Self::validate_datetime(field, value)?;
            }
            FieldType::EntityReference => {
                Self::validate_entity_reference(field, value)?;
            }
            FieldType::Component => {
                Self::validate_component(field, value)?;
            }
        }

        Ok(())
    }

    /// Validate text field value
    fn validate_text(field: &Field, value: &JsonValue) -> Result<()> {
        if !value.is_string() {
            return Err(FieldsError::TypeConversion(format!(
                "Field '{}' expects a string value",
                field.id
            )));
        }
        Ok(())
    }

    /// Validate slug field value
    fn validate_slug(field: &Field, value: &JsonValue) -> Result<()> {
        if let Some(slug) = value.as_str() {
            // Validate slug format (lowercase, alphanumeric, hyphens, underscores)
            if !slug
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
            {
                return Err(FieldsError::Validation(format!(
                    "Field '{}' contains invalid slug format: '{}'",
                    field.id, slug
                )));
            }
        } else {
            return Err(FieldsError::TypeConversion(format!(
                "Field '{}' expects a string value for slug",
                field.id
            )));
        }
        Ok(())
    }

    /// Validate integer field value
    fn validate_integer(field: &Field, value: &JsonValue) -> Result<()> {
        if !value.is_i64() && !value.is_u64() {
            // Try to parse from string
            if let Some(s) = value.as_str() {
                if s.parse::<i64>().is_err() {
                    return Err(FieldsError::TypeConversion(format!(
                        "Field '{}' expects an integer value",
                        field.id
                    )));
                }
            } else {
                return Err(FieldsError::TypeConversion(format!(
                    "Field '{}' expects an integer value",
                    field.id
                )));
            }
        }
        Ok(())
    }

    /// Validate float field value
    fn validate_float(field: &Field, value: &JsonValue) -> Result<()> {
        if !value.is_f64() && !value.is_i64() && !value.is_u64() {
            // Try to parse from string
            if let Some(s) = value.as_str() {
                if s.parse::<f64>().is_err() {
                    return Err(FieldsError::TypeConversion(format!(
                        "Field '{}' expects a numeric value",
                        field.id
                    )));
                }
            } else {
                return Err(FieldsError::TypeConversion(format!(
                    "Field '{}' expects a numeric value",
                    field.id
                )));
            }
        }
        Ok(())
    }

    /// Validate boolean field value
    fn validate_boolean(field: &Field, value: &JsonValue) -> Result<()> {
        if !value.is_boolean() {
            // Accept 0/1 as boolean values
            if let Some(n) = value.as_i64() {
                if n != 0 && n != 1 {
                    return Err(FieldsError::TypeConversion(format!(
                        "Field '{}' expects a boolean value (true/false or 0/1)",
                        field.id
                    )));
                }
            } else {
                return Err(FieldsError::TypeConversion(format!(
                    "Field '{}' expects a boolean value",
                    field.id
                )));
            }
        }
        Ok(())
    }

    /// Validate datetime field value
    fn validate_datetime(field: &Field, value: &JsonValue) -> Result<()> {
        if let Some(s) = value.as_str() {
            // Try to parse as ISO 8601 datetime
            if chrono::DateTime::parse_from_rfc3339(s).is_err() {
                return Err(FieldsError::Validation(format!(
                    "Field '{}' expects a valid ISO 8601 datetime string",
                    field.id
                )));
            }
        } else {
            return Err(FieldsError::TypeConversion(format!(
                "Field '{}' expects a datetime string",
                field.id
            )));
        }
        Ok(())
    }

    /// Validate entity reference field value
    fn validate_entity_reference(field: &Field, value: &JsonValue) -> Result<()> {
        if field.target_entity.is_none() {
            return Err(FieldsError::Validation(format!(
                "Field '{}' is missing target_entity configuration",
                field.id
            )));
        }

        // Entity references should be strings (IDs)
        if !value.is_string() {
            return Err(FieldsError::TypeConversion(format!(
                "Field '{}' expects a string ID for entity reference",
                field.id
            )));
        }

        Ok(())
    }

    /// Validate component field value
    fn validate_component(field: &Field, value: &JsonValue) -> Result<()> {
        let nested_fields = field.fields.as_ref().ok_or_else(|| {
            FieldsError::Validation(format!(
                "Field '{}' is missing nested fields configuration",
                field.id
            ))
        })?;

        // Component values should be objects
        let obj = value.as_object().ok_or_else(|| {
            FieldsError::TypeConversion(format!(
                "Field '{}' expects an object for component value",
                field.id
            ))
        })?;

        // Validate each nested field
        for nested_field in nested_fields {
            if let Some(nested_value) = obj.get(&nested_field.id) {
                Self::validate_field_value(nested_field, nested_value)?;
            } else if nested_field.required {
                return Err(FieldsError::Validation(format!(
                    "Required nested field '{}.{}' is missing",
                    field.id, nested_field.id
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_text_field() {
        let field = Field::new("title", FieldType::Text, "Title");

        assert!(FieldValidator::validate_field_value(&field, &json!("Test")).is_ok());
        assert!(FieldValidator::validate_field_value(&field, &json!(123)).is_err());
        assert!(FieldValidator::validate_field_value(&field, &json!(null)).is_ok());

        let required_field = Field::new("title", FieldType::Text, "Title").required(true);
        assert!(FieldValidator::validate_field_value(&required_field, &json!(null)).is_err());
    }

    #[test]
    fn test_validate_slug_field() {
        let field = Field::new("slug", FieldType::Slug, "Slug");

        assert!(FieldValidator::validate_field_value(&field, &json!("valid-slug")).is_ok());
        assert!(FieldValidator::validate_field_value(&field, &json!("valid_slug_123")).is_ok());
        assert!(FieldValidator::validate_field_value(&field, &json!("Invalid Slug")).is_err());
        assert!(FieldValidator::validate_field_value(&field, &json!("UPPERCASE")).is_err());
    }

    #[test]
    fn test_validate_integer_field() {
        let field = Field::new("count", FieldType::Integer, "Count");

        assert!(FieldValidator::validate_field_value(&field, &json!(42)).is_ok());
        assert!(FieldValidator::validate_field_value(&field, &json!("123")).is_ok());
        assert!(FieldValidator::validate_field_value(&field, &json!(3.5)).is_err());
        assert!(FieldValidator::validate_field_value(&field, &json!("not a number")).is_err());
    }

    #[test]
    fn test_validate_boolean_field() {
        let field = Field::new("active", FieldType::Boolean, "Active");

        assert!(FieldValidator::validate_field_value(&field, &json!(true)).is_ok());
        assert!(FieldValidator::validate_field_value(&field, &json!(false)).is_ok());
        assert!(FieldValidator::validate_field_value(&field, &json!(1)).is_ok());
        assert!(FieldValidator::validate_field_value(&field, &json!(0)).is_ok());
        assert!(FieldValidator::validate_field_value(&field, &json!(2)).is_err());
        assert!(FieldValidator::validate_field_value(&field, &json!("true")).is_err());
    }

    #[test]
    fn test_validate_datetime_field() {
        let field = Field::new("created_at", FieldType::Datetime, "Created At");

        assert!(
            FieldValidator::validate_field_value(&field, &json!("2023-01-01T00:00:00Z")).is_ok()
        );
        assert!(
            FieldValidator::validate_field_value(&field, &json!("2023-01-01T00:00:00+00:00"))
                .is_ok()
        );
        assert!(FieldValidator::validate_field_value(&field, &json!("invalid date")).is_err());
        assert!(FieldValidator::validate_field_value(&field, &json!(123456789)).is_err());
    }

    #[test]
    fn test_validate_cardinality() {
        // Test single-value field
        let single_field = Field::new("title", FieldType::Text, "Title");
        assert!(FieldValidator::validate_field_value(&single_field, &json!("Test")).is_ok());
        assert!(
            FieldValidator::validate_field_value(&single_field, &json!(["Test1", "Test2"]))
                .is_err()
        );

        // Test multi-value field with limit
        let multi_field = Field::new("tags", FieldType::Text, "Tags").with_cardinality(3);
        assert!(
            FieldValidator::validate_field_value(&multi_field, &json!(["Tag1", "Tag2"])).is_ok()
        );
        assert!(FieldValidator::validate_field_value(
            &multi_field,
            &json!(["Tag1", "Tag2", "Tag3"])
        )
        .is_ok());
        assert!(FieldValidator::validate_field_value(
            &multi_field,
            &json!(["Tag1", "Tag2", "Tag3", "Tag4"])
        )
        .is_err());
        // Test single value for multi-value field (should be accepted)
        assert!(FieldValidator::validate_field_value(&multi_field, &json!("SingleTag")).is_ok());

        // Test unlimited cardinality field
        let unlimited_field = Field::new("items", FieldType::Text, "Items").with_cardinality(-1);
        assert!(FieldValidator::validate_field_value(
            &unlimited_field,
            &json!(["Item1", "Item2", "Item3", "Item4", "Item5"])
        )
        .is_ok());
        assert!(
            FieldValidator::validate_field_value(&unlimited_field, &json!("SingleItem")).is_ok()
        );
    }
}

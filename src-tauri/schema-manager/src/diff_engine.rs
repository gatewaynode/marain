use serde_yaml::Value;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Categories of changes based on their impact
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeCategory {
    /// Non-breaking changes (e.g., adding optional fields)
    Safe,
    /// Potentially breaking changes (e.g., changing field types)
    Warning,
    /// Definitely breaking changes (e.g., removing required fields)
    Breaking,
}

/// Represents a change in configuration
#[derive(Debug, Clone)]
pub struct Change {
    pub path: Vec<String>,
    pub category: ChangeCategory,
    pub description: String,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
}

/// Engine for detecting differences between YAML configurations
pub struct DiffEngine;

impl DiffEngine {
    /// Compare two YAML values and generate a diff
    pub fn compare(old_state: &Value, new_state: &Value) -> ConfigDiff {
        let mut diff = ConfigDiff::new();
        
        match (old_state, new_state) {
            (Value::Mapping(old_map), Value::Mapping(new_map)) => {
                Self::compare_mappings(old_map, new_map, &mut diff, vec![]);
            }
            _ => {
                // If root types differ, treat as complete replacement
                diff.modified.insert("_root".to_string(), Value::Mapping(serde_yaml::Mapping::new()));
            }
        }
        
        diff
    }
    
    /// Compare two YAML mappings recursively
    fn compare_mappings(
        old_map: &serde_yaml::Mapping,
        new_map: &serde_yaml::Mapping,
        diff: &mut ConfigDiff,
        path: Vec<String>,
    ) {
        // Check for added keys
        for (key, value) in new_map {
            if let Some(key_str) = key.as_str() {
                if !old_map.contains_key(key) {
                    let full_path = Self::build_path(&path, key_str);
                    diff.added.insert(full_path, value.clone());
                }
            }
        }
        
        // Check for removed keys
        for (key, value) in old_map {
            if let Some(key_str) = key.as_str() {
                if !new_map.contains_key(key) {
                    let full_path = Self::build_path(&path, key_str);
                    diff.removed.insert(full_path, value.clone());
                }
            }
        }
        
        // Check for modified values
        for (key, new_value) in new_map {
            if let Some(key_str) = key.as_str() {
                if let Some(old_value) = old_map.get(key) {
                    if !Self::values_equal(old_value, new_value) {
                        let full_path = Self::build_path(&path, key_str);
                        
                        // Create a change record
                        let mut change_record = serde_yaml::Mapping::new();
                        change_record.insert(Value::String("old".to_string()), old_value.clone());
                        change_record.insert(Value::String("new".to_string()), new_value.clone());
                        
                        // For nested mappings, recurse
                        if let (Value::Mapping(old_nested), Value::Mapping(new_nested)) = (old_value, new_value) {
                            let mut nested_path = path.clone();
                            nested_path.push(key_str.to_string());
                            Self::compare_mappings(old_nested, new_nested, diff, nested_path);
                        } else {
                            diff.modified.insert(full_path, Value::Mapping(change_record));
                        }
                    }
                }
            }
        }
    }
    
    /// Check if two values are equal
    fn values_equal(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Sequence(a), Value::Sequence(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| Self::values_equal(x, y))
            }
            (Value::Mapping(a), Value::Mapping(b)) => {
                a.len() == b.len() && a.iter().all(|(k, v)| {
                    b.get(k).map_or(false, |v2| Self::values_equal(v, v2))
                })
            }
            _ => false,
        }
    }
    
    /// Build a path string from path components
    fn build_path(path: &[String], key: &str) -> String {
        if path.is_empty() {
            key.to_string()
        } else {
            format!("{}.{}", path.join("."), key)
        }
    }
    
    /// Categorize changes based on their impact
    pub fn categorize_changes(diff: &ConfigDiff) -> Vec<Change> {
        let mut changes = Vec::new();
        
        // Categorize additions
        for (path, value) in &diff.added {
            let category = Self::categorize_addition(path, value);
            changes.push(Change {
                path: path.split('.').map(String::from).collect(),
                category,
                description: format!("Added: {}", path),
                old_value: None,
                new_value: Some(value.clone()),
            });
        }
        
        // Categorize removals
        for (path, value) in &diff.removed {
            let category = Self::categorize_removal(path, value);
            changes.push(Change {
                path: path.split('.').map(String::from).collect(),
                category,
                description: format!("Removed: {}", path),
                old_value: Some(value.clone()),
                new_value: None,
            });
        }
        
        // Categorize modifications
        for (path, change_record) in &diff.modified {
            if let Some(mapping) = change_record.as_mapping() {
                let old_value = mapping.get(&Value::String("old".to_string()));
                let new_value = mapping.get(&Value::String("new".to_string()));
                
                let category = Self::categorize_modification(path, old_value, new_value);
                changes.push(Change {
                    path: path.split('.').map(String::from).collect(),
                    category,
                    description: format!("Modified: {}", path),
                    old_value: old_value.cloned(),
                    new_value: new_value.cloned(),
                });
            }
        }
        
        changes
    }
    
    /// Categorize an addition based on its impact
    fn categorize_addition(path: &str, _value: &Value) -> ChangeCategory {
        // Adding new fields is generally safe
        if path.contains("fields") && !path.contains("required") {
            ChangeCategory::Safe
        } else if path.contains("description") || path.contains("label") {
            ChangeCategory::Safe
        } else {
            ChangeCategory::Warning
        }
    }
    
    /// Categorize a removal based on its impact
    fn categorize_removal(path: &str, value: &Value) -> ChangeCategory {
        // Removing fields is breaking
        if path.contains("fields") {
            ChangeCategory::Breaking
        }
        // Removing required fields is definitely breaking
        else if path.contains("required") && value.as_bool() == Some(true) {
            ChangeCategory::Breaking
        }
        // Removing entities is breaking
        else if !path.contains('.') {
            ChangeCategory::Breaking
        } else {
            ChangeCategory::Warning
        }
    }
    
    /// Categorize a modification based on its impact
    fn categorize_modification(path: &str, old_value: Option<&Value>, new_value: Option<&Value>) -> ChangeCategory {
        // Type changes are breaking
        if let (Some(old), Some(new)) = (old_value, new_value) {
            if std::mem::discriminant(old) != std::mem::discriminant(new) {
                return ChangeCategory::Breaking;
            }
        }
        
        // Field type changes are breaking
        if path.contains("type") && path.contains("fields") {
            ChangeCategory::Breaking
        }
        // Cardinality changes are breaking
        else if path.contains("cardinality") {
            ChangeCategory::Breaking
        }
        // Making a field required is breaking for existing data
        else if path.contains("required") {
            if let (Some(Value::Bool(false)), Some(Value::Bool(true))) = (old_value, new_value) {
                ChangeCategory::Breaking
            } else {
                ChangeCategory::Safe
            }
        } else {
            ChangeCategory::Safe
        }
    }
    
    /// Generate a human-readable summary of changes
    pub fn summarize_changes(changes: &[Change]) -> String {
        let safe_count = changes.iter().filter(|c| c.category == ChangeCategory::Safe).count();
        let warning_count = changes.iter().filter(|c| c.category == ChangeCategory::Warning).count();
        let breaking_count = changes.iter().filter(|c| c.category == ChangeCategory::Breaking).count();
        
        let mut summary = String::new();
        summary.push_str(&format!("Total changes: {}\n", changes.len()));
        summary.push_str(&format!("  Safe: {}\n", safe_count));
        summary.push_str(&format!("  Warnings: {}\n", warning_count));
        summary.push_str(&format!("  Breaking: {}\n", breaking_count));
        
        if breaking_count > 0 {
            summary.push_str("\nBreaking changes:\n");
            for change in changes.iter().filter(|c| c.category == ChangeCategory::Breaking) {
                summary.push_str(&format!("  - {}\n", change.description));
            }
        }
        
        summary
    }
}

/// Represents differences between two configuration states
#[derive(Debug, Clone)]
pub struct ConfigDiff {
    pub added: HashMap<String, Value>,
    pub removed: HashMap<String, Value>,
    pub modified: HashMap<String, Value>,
}

impl ConfigDiff {
    pub fn new() -> Self {
        Self {
            added: HashMap::new(),
            removed: HashMap::new(),
            modified: HashMap::new(),
        }
    }
    
    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.modified.is_empty()
    }
    
    /// Count total number of changes
    pub fn change_count(&self) -> usize {
        self.added.len() + self.removed.len() + self.modified.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compare_identical_values() {
        let yaml1 = r#"
        id: test
        name: Test Entity
        fields:
          - id: title
            type: text
            required: true
        "#;
        
        let value1: Value = serde_yaml::from_str(yaml1).unwrap();
        let diff = DiffEngine::compare(&value1, &value1);
        
        assert!(!diff.has_changes());
    }
    
    #[test]
    fn test_detect_added_fields() {
        let yaml1 = r#"
        id: test
        fields:
          - id: title
            type: text
        "#;
        
        let yaml2 = r#"
        id: test
        fields:
          - id: title
            type: text
          - id: description
            type: text
        "#;
        
        let value1: Value = serde_yaml::from_str(yaml1).unwrap();
        let value2: Value = serde_yaml::from_str(yaml2).unwrap();
        let diff = DiffEngine::compare(&value1, &value2);
        
        assert!(diff.has_changes());
        assert!(!diff.added.is_empty() || !diff.modified.is_empty());
    }
    
    #[test]
    fn test_detect_removed_fields() {
        let yaml1 = r#"
        id: test
        fields:
          - id: title
            type: text
          - id: description
            type: text
        "#;
        
        let yaml2 = r#"
        id: test
        fields:
          - id: title
            type: text
        "#;
        
        let value1: Value = serde_yaml::from_str(yaml1).unwrap();
        let value2: Value = serde_yaml::from_str(yaml2).unwrap();
        let diff = DiffEngine::compare(&value1, &value2);
        
        assert!(diff.has_changes());
        assert!(!diff.removed.is_empty() || !diff.modified.is_empty());
    }
    
    #[test]
    fn test_categorize_breaking_changes() {
        let mut diff = ConfigDiff::new();
        diff.removed.insert("fields.title".to_string(), Value::String("test".to_string()));
        
        let changes = DiffEngine::categorize_changes(&diff);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].category, ChangeCategory::Breaking);
    }
    
    #[test]
    fn test_categorize_safe_changes() {
        let mut diff = ConfigDiff::new();
        diff.added.insert("description".to_string(), Value::String("A description".to_string()));
        
        let changes = DiffEngine::categorize_changes(&diff);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].category, ChangeCategory::Safe);
    }
}
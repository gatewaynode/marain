//! Bulk operations and content workflow utilities

use crate::error::ContentError;
use crate::hashing::generate_content_hash;
use serde_json::Value;
use std::collections::HashMap;

/// Result of a bulk operation
#[derive(Debug, Default)]
pub struct BulkOperationResult {
    /// Number of items successfully processed
    pub success_count: usize,
    /// Number of items that failed
    pub failure_count: usize,
    /// Details of failures (item ID -> error message)
    pub failures: HashMap<String, String>,
}

impl BulkOperationResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful operation
    pub fn add_success(&mut self) {
        self.success_count += 1;
    }

    /// Record a failed operation
    pub fn add_failure(&mut self, id: String, error: String) {
        self.failure_count += 1;
        self.failures.insert(id, error);
    }

    /// Check if all operations were successful
    pub fn all_successful(&self) -> bool {
        self.failure_count == 0
    }
}

/// Process multiple content items in bulk
///
/// # Arguments
///
/// * `items` - Vector of content items to process
/// * `processor` - Function to process each item
///
/// # Returns
///
/// A `BulkOperationResult` summarizing the operation
pub async fn process_bulk<F, Fut>(
    items: Vec<HashMap<String, Value>>,
    processor: F,
) -> BulkOperationResult
where
    F: Fn(HashMap<String, Value>) -> Fut,
    Fut: std::future::Future<Output = Result<(), ContentError>>,
{
    let mut result = BulkOperationResult::new();

    for item in items {
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        match processor(item).await {
            Ok(_) => result.add_success(),
            Err(e) => result.add_failure(id, e.to_string()),
        }
    }

    result
}

/// Batch update content hashes for multiple items
///
/// # Arguments
///
/// * `items` - Mutable vector of content items
///
/// # Returns
///
/// Number of items updated
pub fn batch_update_hashes(items: &mut [HashMap<String, Value>]) -> usize {
    let mut updated = 0;

    for item in items.iter_mut() {
        let hash = generate_content_hash(item);
        item.insert("content_hash".to_string(), Value::String(hash));
        updated += 1;
    }

    updated
}

/// Filter content items based on a predicate
///
/// # Arguments
///
/// * `items` - Vector of content items to filter
/// * `predicate` - Function that returns true for items to keep
///
/// # Returns
///
/// Filtered vector of content items
pub fn filter_content<F>(
    items: Vec<HashMap<String, Value>>,
    predicate: F,
) -> Vec<HashMap<String, Value>>
where
    F: Fn(&HashMap<String, Value>) -> bool,
{
    items.into_iter().filter(|item| predicate(item)).collect()
}

/// Transform content items using a mapping function
///
/// # Arguments
///
/// * `items` - Vector of content items to transform
/// * `transformer` - Function to transform each item
///
/// # Returns
///
/// Vector of transformed items
pub fn transform_content<F>(
    items: Vec<HashMap<String, Value>>,
    transformer: F,
) -> Vec<HashMap<String, Value>>
where
    F: Fn(HashMap<String, Value>) -> HashMap<String, Value>,
{
    items.into_iter().map(transformer).collect()
}

/// Validate multiple content items
///
/// # Arguments
///
/// * `items` - Vector of content items to validate
/// * `validator` - Function to validate each item
///
/// # Returns
///
/// A `BulkOperationResult` with validation results
pub fn validate_bulk<F>(items: &[HashMap<String, Value>], validator: F) -> BulkOperationResult
where
    F: Fn(&HashMap<String, Value>) -> Result<(), ContentError>,
{
    let mut result = BulkOperationResult::new();

    for item in items {
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        match validator(item) {
            Ok(_) => result.add_success(),
            Err(e) => result.add_failure(id, e.to_string()),
        }
    }

    result
}

/// Content migration helper for moving content between entity types
#[derive(Default)]
pub struct ContentMigrator {
    /// Field mappings from source to target
    field_mappings: HashMap<String, String>,
    /// Default values for missing fields
    defaults: HashMap<String, Value>,
}

impl ContentMigrator {
    /// Create a new content migrator
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a field mapping
    pub fn map_field(mut self, from: &str, to: &str) -> Self {
        self.field_mappings.insert(from.to_string(), to.to_string());
        self
    }

    /// Add a default value for a field
    pub fn with_default(mut self, field: &str, value: Value) -> Self {
        self.defaults.insert(field.to_string(), value);
        self
    }

    /// Migrate a single content item
    pub fn migrate(&self, source: HashMap<String, Value>) -> HashMap<String, Value> {
        let mut target = HashMap::new();

        // Apply field mappings
        for (from_field, to_field) in &self.field_mappings {
            if let Some(value) = source.get(from_field) {
                target.insert(to_field.clone(), value.clone());
            }
        }

        // Apply defaults for missing fields
        for (field, default_value) in &self.defaults {
            target
                .entry(field.clone())
                .or_insert_with(|| default_value.clone());
        }

        // Generate new content hash
        let hash = generate_content_hash(&target);
        target.insert("content_hash".to_string(), Value::String(hash));

        target
    }

    /// Migrate multiple content items
    pub fn migrate_bulk(&self, items: Vec<HashMap<String, Value>>) -> Vec<HashMap<String, Value>> {
        items.into_iter().map(|item| self.migrate(item)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_bulk_operation_result() {
        let mut result = BulkOperationResult::new();
        assert!(result.all_successful());

        result.add_success();
        result.add_success();
        assert_eq!(result.success_count, 2);
        assert!(result.all_successful());

        result.add_failure("item1".to_string(), "Error message".to_string());
        assert_eq!(result.failure_count, 1);
        assert!(!result.all_successful());
    }

    #[test]
    fn test_batch_update_hashes() {
        let mut items = vec![
            {
                let mut item = HashMap::new();
                item.insert("title".to_string(), json!("Item 1"));
                item.insert("body".to_string(), json!("Content 1"));
                item
            },
            {
                let mut item = HashMap::new();
                item.insert("title".to_string(), json!("Item 2"));
                item.insert("body".to_string(), json!("Content 2"));
                item
            },
        ];

        let updated = batch_update_hashes(&mut items);
        assert_eq!(updated, 2);

        for item in &items {
            assert!(item.contains_key("content_hash"));
        }
    }

    #[test]
    fn test_filter_content() {
        let items = vec![
            {
                let mut item = HashMap::new();
                item.insert("status".to_string(), json!("published"));
                item
            },
            {
                let mut item = HashMap::new();
                item.insert("status".to_string(), json!("draft"));
                item
            },
        ];

        let filtered = filter_content(items, |item| {
            item.get("status")
                .and_then(|v| v.as_str())
                .map(|s| s == "published")
                .unwrap_or(false)
        });

        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_content_migrator() {
        let migrator = ContentMigrator::new()
            .map_field("old_title", "title")
            .map_field("old_body", "body")
            .with_default("status", json!("migrated"));

        let mut source = HashMap::new();
        source.insert("old_title".to_string(), json!("Test Title"));
        source.insert("old_body".to_string(), json!("Test Content"));
        source.insert("extra_field".to_string(), json!("Extra"));

        let target = migrator.migrate(source);

        assert_eq!(target.get("title"), Some(&json!("Test Title")));
        assert_eq!(target.get("body"), Some(&json!("Test Content")));
        assert_eq!(target.get("status"), Some(&json!("migrated")));
        assert!(target.contains_key("content_hash"));
        assert!(!target.contains_key("old_title"));
        assert!(!target.contains_key("extra_field"));
    }
}

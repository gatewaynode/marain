//! Content hashing utilities for change detection and caching

use crate::error::ContentError;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// List of metadata fields that should be excluded from content hashing
const METADATA_FIELDS: &[&str] = &[
    "id",
    "user",
    "rid",
    "created_at",
    "updated_at",
    "last_cached",
    "cache_ttl",
    "content_hash",
];

/// Generate a SHA256 content hash from field values
///
/// This function creates a deterministic hash of content by:
/// 1. Excluding metadata fields that don't represent actual content
/// 2. Sorting fields alphabetically for consistent hashing
/// 3. Concatenating field names and values before hashing
///
/// # Arguments
///
/// * `data` - A HashMap containing field names and their JSON values
///
/// # Returns
///
/// A hexadecimal string representation of the SHA256 hash
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use serde_json::json;
/// use content::generate_content_hash;
///
/// let mut data = HashMap::new();
/// data.insert("title".to_string(), json!("My Article"));
/// data.insert("body".to_string(), json!("Article content..."));
/// data.insert("id".to_string(), json!("article_123")); // Will be excluded
///
/// let hash = generate_content_hash(&data);
/// assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
/// ```
pub fn generate_content_hash(data: &HashMap<String, Value>) -> String {
    let mut hasher = Sha256::new();
    let mut sorted_data: Vec<_> = data.iter().collect();
    sorted_data.sort_by_key(|&(k, _)| k);

    for (key, value) in sorted_data {
        // Skip metadata fields when generating content hash
        if METADATA_FIELDS.contains(&key.as_str()) {
            continue;
        }
        hasher.update(key.as_bytes());
        hasher.update(value.to_string().as_bytes());
    }

    format!("{:x}", hasher.finalize())
}

/// Calculate a content hash with custom field exclusions
///
/// This is a more flexible version of `generate_content_hash` that allows
/// specifying which fields to exclude from the hash calculation.
///
/// # Arguments
///
/// * `data` - A HashMap containing field names and their JSON values
/// * `exclude_fields` - A slice of field names to exclude from hashing
///
/// # Returns
///
/// A Result containing the hexadecimal hash string or an error
pub fn calculate_content_hash(
    data: &HashMap<String, Value>,
    exclude_fields: &[&str],
) -> Result<String, ContentError> {
    let mut hasher = Sha256::new();
    let mut sorted_data: Vec<_> = data.iter().collect();
    sorted_data.sort_by_key(|&(k, _)| k);

    for (key, value) in sorted_data {
        if exclude_fields.contains(&key.as_str()) {
            continue;
        }
        hasher.update(key.as_bytes());

        // Handle serialization errors
        let value_str = serde_json::to_string(value)
            .map_err(|e| ContentError::HashingError(format!("Failed to serialize value: {}", e)))?;
        hasher.update(value_str.as_bytes());
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Verify if content has changed by comparing hashes
///
/// # Arguments
///
/// * `old_data` - The original content data
/// * `new_data` - The new content data to compare
///
/// # Returns
///
/// `true` if the content has changed, `false` otherwise
pub fn has_content_changed(
    old_data: &HashMap<String, Value>,
    new_data: &HashMap<String, Value>,
) -> bool {
    generate_content_hash(old_data) != generate_content_hash(new_data)
}

/// Generate a hash for a single value
///
/// Useful for hashing individual fields or simple values.
///
/// # Arguments
///
/// * `value` - The value to hash
///
/// # Returns
///
/// A hexadecimal string representation of the SHA256 hash
pub fn hash_value(value: &Value) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.to_string().as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_content_hash() {
        let mut data = HashMap::new();
        data.insert("title".to_string(), json!("Test Title"));
        data.insert("body".to_string(), json!("Test content"));
        data.insert("id".to_string(), json!("test_id")); // Should be excluded

        let hash = generate_content_hash(&data);
        assert_eq!(hash.len(), 64);

        // Hash should be deterministic
        let hash2 = generate_content_hash(&data);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_metadata_fields_excluded() {
        let mut data1 = HashMap::new();
        data1.insert("title".to_string(), json!("Test"));
        data1.insert("body".to_string(), json!("Content"));

        let mut data2 = data1.clone();
        // Add metadata fields that should be excluded
        data2.insert("id".to_string(), json!("123"));
        data2.insert("user".to_string(), json!(0));
        data2.insert("created_at".to_string(), json!("2024-01-01"));

        let hash1 = generate_content_hash(&data1);
        let hash2 = generate_content_hash(&data2);

        // Hashes should be the same since metadata is excluded
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_calculate_content_hash_with_custom_exclusions() {
        let mut data = HashMap::new();
        data.insert("title".to_string(), json!("Test"));
        data.insert("body".to_string(), json!("Content"));
        data.insert("custom_field".to_string(), json!("Value"));

        let hash1 = calculate_content_hash(&data, &["custom_field"]).unwrap();
        let hash2 = calculate_content_hash(&data, &[]).unwrap();

        // Hashes should be different when different fields are excluded
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_has_content_changed() {
        let mut old_data = HashMap::new();
        old_data.insert("title".to_string(), json!("Original"));
        old_data.insert("body".to_string(), json!("Content"));

        let mut new_data = old_data.clone();
        assert!(!has_content_changed(&old_data, &new_data));

        new_data.insert("body".to_string(), json!("Modified Content"));
        assert!(has_content_changed(&old_data, &new_data));
    }

    #[test]
    fn test_hash_value() {
        let value = json!("test string");
        let hash = hash_value(&value);
        assert_eq!(hash.len(), 64);

        // Should be deterministic
        let hash2 = hash_value(&value);
        assert_eq!(hash, hash2);
    }
}

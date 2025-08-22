//! # Content Crate
//!
//! This crate provides common content-related functions and utilities that are needed
//! across the Marain CMS application. It serves as a centralized location for:
//!
//! - Content hashing and change detection
//! - Content bulk operations
//! - Publishing workflows
//! - Content reorganization and migration
//! - Content validation and transformation utilities
//!
//! ## Key Features
//!
//! - **Content Hashing**: Generate SHA256 hashes of content for change detection and caching
//! - **ID Generation**: Create URL-safe IDs from content titles
//! - **Bulk Operations**: Utilities for processing multiple content items efficiently
//! - **Migration Support**: Tools for migrating content between entity types
//!
//! ## Usage
//!
//! ```rust
//! use content::{generate_content_hash, generate_id_from_title};
//! use std::collections::HashMap;
//! use serde_json::json;
//!
//! let mut data = HashMap::new();
//! data.insert("title".to_string(), json!("My Article"));
//! data.insert("body".to_string(), json!("Article content..."));
//!
//! let hash = generate_content_hash(&data);
//! let id = generate_id_from_title("My Article");
//! ```

pub mod error;
pub mod hashing;
pub mod operations;
pub mod utils;

// Re-export commonly used functions at the crate root
pub use error::ContentError;
pub use hashing::{calculate_content_hash, generate_content_hash, has_content_changed, hash_value};
pub use utils::{generate_id_from_title, sanitize_slug};

/// Result type for content operations
pub type Result<T> = std::result::Result<T, ContentError>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_content_hash_generation() {
        let mut data = HashMap::new();
        data.insert("title".to_string(), json!("Test Title"));
        data.insert("body".to_string(), json!("Test content"));

        let hash = generate_content_hash(&data);
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
    }

    #[test]
    fn test_id_from_title() {
        assert_eq!(generate_id_from_title("Hello World"), "hello_world");
        assert_eq!(
            generate_id_from_title("Test: With Punctuation!"),
            "test_with_punctuation"
        );
        assert_eq!(
            generate_id_from_title("Multiple   Spaces"),
            "multiple_spaces"
        );
    }
}

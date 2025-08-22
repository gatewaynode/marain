use chrono::Utc;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use ulid::Ulid;

/// Helper function to generate ID from title
fn generate_id_from_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' {
                c
            } else {
                ' ' // Replace punctuation with space
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("_")
}

/// Generate a content hash from the field values
fn generate_content_hash(data: &HashMap<String, serde_json::Value>) -> String {
    let mut hasher = Sha256::new();
    let mut sorted_data: Vec<_> = data.iter().collect();
    sorted_data.sort_by_key(|&(k, _)| k);

    for (key, value) in sorted_data {
        // Skip metadata fields when generating content hash
        if key == "id"
            || key == "user"
            || key == "rid"
            || key == "created_at"
            || key == "updated_at"
            || key == "last_cached"
            || key == "cache_ttl"
            || key == "content_hash"
        {
            continue;
        }
        hasher.update(key.as_bytes());
        hasher.update(value.to_string().as_bytes());
    }

    format!("{:x}", hasher.finalize())
}

/// Generate lorem ipsum test data for the snippet entity
pub fn generate_snippet_test_data() -> Vec<HashMap<String, serde_json::Value>> {
    vec![
        {
            let mut data = HashMap::new();
            let title = "Getting Started with Rust";
            data.insert("id".to_string(), json!(generate_id_from_title(title)));
            data.insert("user".to_string(), json!(0)); // Default system user
            data.insert("title".to_string(), json!(title));
            data.insert("body".to_string(), json!("Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety. Lorem ipsum dolor sit amet, consectetur adipiscing elit."));
            data.insert("author".to_string(), json!("user-001"));
            data.insert("published_at".to_string(), json!(Utc::now().to_rfc3339()));
            data.insert("status".to_string(), json!("published"));
            // Add cache fields
            data.insert("last_cached".to_string(), json!(null)); // Not cached yet
            data.insert("cache_ttl".to_string(), json!(86400)); // 24 hours default
            let hash = generate_content_hash(&data);
            data.insert("content_hash".to_string(), json!(hash));
            data
        },
        {
            let mut data = HashMap::new();
            let title = "Understanding Async/Await";
            data.insert("id".to_string(), json!(generate_id_from_title(title)));
            data.insert("user".to_string(), json!(0)); // Default system user
            data.insert("title".to_string(), json!(title));
            data.insert("body".to_string(), json!("Asynchronous programming in Rust allows you to write efficient concurrent code. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."));
            data.insert("author".to_string(), json!("user-002"));
            data.insert("published_at".to_string(), json!(Utc::now().to_rfc3339()));
            data.insert("status".to_string(), json!("published"));
            // Add cache fields
            data.insert("last_cached".to_string(), json!(null)); // Not cached yet
            data.insert("cache_ttl".to_string(), json!(86400)); // 24 hours default
            let hash = generate_content_hash(&data);
            data.insert("content_hash".to_string(), json!(hash));
            data
        },
        {
            let mut data = HashMap::new();
            let title = "Memory Management Tips";
            data.insert("id".to_string(), json!(generate_id_from_title(title)));
            data.insert("user".to_string(), json!(0)); // Default system user
            data.insert("title".to_string(), json!(title));
            data.insert("body".to_string(), json!("Learn about ownership, borrowing, and lifetimes in Rust. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris."));
            data.insert("author".to_string(), json!("user-001"));
            data.insert("published_at".to_string(), json!(Utc::now().to_rfc3339()));
            data.insert("status".to_string(), json!("draft"));
            // Add cache fields
            data.insert("last_cached".to_string(), json!(null)); // Not cached yet
            data.insert("cache_ttl".to_string(), json!(86400)); // 24 hours default
            let hash = generate_content_hash(&data);
            data.insert("content_hash".to_string(), json!(hash));
            data
        },
    ]
}

/// Generate lorem ipsum test data for the all_fields entity
pub fn generate_all_fields_test_data() -> Vec<HashMap<String, serde_json::Value>> {
    vec![
        {
            let mut data = HashMap::new();
            let title = "Complete Field Test Entry";
            data.insert("id".to_string(), json!(generate_id_from_title(title)));
            data.insert("user".to_string(), json!(0)); // Default system user
            data.insert("title".to_string(), json!(title));
            data.insert("splash".to_string(), json!("<h1>Welcome to the Test</h1><p>This is rich text content with <strong>bold</strong> and <em>italic</em> formatting.</p>"));
            data.insert("body".to_string(), json!("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat."));
            data.insert("count".to_string(), json!(42));
            data.insert("float".to_string(), json!(3.5));
            data.insert("bool".to_string(), json!(true));
            data.insert("sluggo".to_string(), json!("complete-field-test-entry"));
            data.insert("author".to_string(), json!("user-003"));
            data.insert("published_at".to_string(), json!(Utc::now().to_rfc3339()));
            data.insert("status".to_string(), json!("published"));
            // Add cache fields
            data.insert("last_cached".to_string(), json!(null)); // Not cached yet
            data.insert("cache_ttl".to_string(), json!(86400)); // 24 hours default
            let hash = generate_content_hash(&data);
            data.insert("content_hash".to_string(), json!(hash));
            data
        },
        {
            let mut data = HashMap::new();
            let title = "Minimal Field Entry";
            data.insert("id".to_string(), json!(generate_id_from_title(title)));
            data.insert("user".to_string(), json!(0)); // Default system user
            data.insert("title".to_string(), json!(title));
            data.insert("splash".to_string(), json!("<p>Simple HTML content</p>"));
            data.insert("body".to_string(), json!("Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur."));
            data.insert("count".to_string(), json!(7));
            data.insert("float".to_string(), json!(2.5));
            data.insert("bool".to_string(), json!(false));
            data.insert("sluggo".to_string(), json!("minimal-field-entry"));
            data.insert("author".to_string(), json!("user-002"));
            data.insert("published_at".to_string(), json!(Utc::now().to_rfc3339()));
            data.insert("status".to_string(), json!("draft"));
            // Add cache fields
            data.insert("last_cached".to_string(), json!(null)); // Not cached yet
            data.insert("cache_ttl".to_string(), json!(86400)); // 24 hours default
            let hash = generate_content_hash(&data);
            data.insert("content_hash".to_string(), json!(hash));
            data
        },
    ]
}

/// Generate lorem ipsum test data for the multi entity (without multi-value fields)
pub fn generate_multi_test_data() -> Vec<HashMap<String, serde_json::Value>> {
    vec![
        {
            let mut data = HashMap::new();
            let title = "Multi-Value Test Entry";
            data.insert("id".to_string(), json!(generate_id_from_title(title)));
            data.insert("user".to_string(), json!(0)); // Default system user
            data.insert("title".to_string(), json!(title));
            // Note: multi-value fields (two, infinite) are stored in separate tables
            data.insert("author".to_string(), json!("user-001"));
            data.insert("published_at".to_string(), json!(Utc::now().to_rfc3339()));
            data.insert("status".to_string(), json!("published"));
            // Add cache fields
            data.insert("last_cached".to_string(), json!(null)); // Not cached yet
            data.insert("cache_ttl".to_string(), json!(86400)); // 24 hours default
            let hash = generate_content_hash(&data);
            data.insert("content_hash".to_string(), json!(hash));
            data
        },
        {
            let mut data = HashMap::new();
            let title = "Another Multi Entry";
            data.insert("id".to_string(), json!(generate_id_from_title(title)));
            data.insert("user".to_string(), json!(0)); // Default system user
            data.insert("title".to_string(), json!(title));
            data.insert("author".to_string(), json!("user-003"));
            data.insert("published_at".to_string(), json!(Utc::now().to_rfc3339()));
            data.insert("status".to_string(), json!("archived"));
            // Add cache fields
            data.insert("last_cached".to_string(), json!(null)); // Not cached yet
            data.insert("cache_ttl".to_string(), json!(86400)); // 24 hours default
            let hash = generate_content_hash(&data);
            data.insert("content_hash".to_string(), json!(hash));
            data
        },
        {
            let mut data = HashMap::new();
            let title = "Minimal Multi Entry";
            data.insert("id".to_string(), json!(generate_id_from_title(title)));
            data.insert("user".to_string(), json!(0)); // Default system user
            data.insert("title".to_string(), json!(title));
            data.insert("author".to_string(), json!("user-002"));
            data.insert("published_at".to_string(), json!(Utc::now().to_rfc3339()));
            data.insert("status".to_string(), json!("draft"));
            // Add cache fields
            data.insert("last_cached".to_string(), json!(null)); // Not cached yet
            data.insert("cache_ttl".to_string(), json!(86400)); // 24 hours default
            let hash = generate_content_hash(&data);
            data.insert("content_hash".to_string(), json!(hash));
            data
        },
    ]
}

/// Multi-value field data for the 'two' field (cardinality: 2)
pub fn get_multi_two_values() -> Vec<Vec<&'static str>> {
    vec![
        vec!["First value", "Second value"],
        vec!["Alpha", "Beta"],
        vec!["One", "Two"],
    ]
}

/// Multi-value field data for the 'infinite' field (cardinality: -1)
pub fn get_multi_infinite_values() -> Vec<Vec<&'static str>> {
    vec![
        vec!["Tag1", "Tag2", "Tag3", "Tag4", "Tag5"],
        vec![
            "Lorem",
            "Ipsum",
            "Dolor",
            "Sit",
            "Amet",
            "Consectetur",
            "Adipiscing",
        ],
        vec!["Single"],
    ]
}

/// Initialize test data in the database (only if database is empty)
pub async fn init_test_data(db: &database::Database) -> Result<(), Box<dyn std::error::Error>> {
    use database::storage::EntityStorage;
    use tracing::info;

    // Check if any data already exists in the database
    let snippet_storage = EntityStorage::new(db, "snippet");
    let existing_snippets = snippet_storage.list(Some(1), None).await?;

    if !existing_snippets.is_empty() {
        info!("Database already contains data, skipping test data initialization");
        return Ok(());
    }

    // Check other entities as well to be thorough
    let all_fields_storage = EntityStorage::new(db, "all_fields");
    let existing_all_fields = all_fields_storage.list(Some(1), None).await?;

    if !existing_all_fields.is_empty() {
        info!("Database already contains data, skipping test data initialization");
        return Ok(());
    }

    info!("Database is empty, initializing test data for entities");

    // Create snippet test data
    for data in generate_snippet_test_data() {
        match snippet_storage.create(data.clone()).await {
            Ok(id) => info!("Created snippet test data with id: {}", id),
            Err(e) => info!("Failed to create snippet test data: {}", e),
        }
    }

    // Create all_fields test data
    for data in generate_all_fields_test_data() {
        match all_fields_storage.create(data.clone()).await {
            Ok(id) => info!("Created all_fields test data with id: {}", id),
            Err(e) => info!("Failed to create all_fields test data: {}", e),
        }
    }

    // Create multi test data with multi-value fields
    let multi_storage = EntityStorage::new(db, "multi");
    let mut multi_ids = Vec::new();

    for data in generate_multi_test_data() {
        match multi_storage.create(data.clone()).await {
            Ok(id) => {
                info!("Created multi test data with id: {}", id);
                multi_ids.push(id);
            }
            Err(e) => info!("Failed to create multi test data: {}", e),
        }
    }

    // Now insert multi-value field data
    if !multi_ids.is_empty() {
        insert_multi_value_fields(db, &multi_ids).await?;
    }

    info!("Test data initialization complete");
    Ok(())
}

/// Insert multi-value field data for multi entities
async fn insert_multi_value_fields(
    db: &database::Database,
    parent_ids: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    use tracing::info;

    let two_values = get_multi_two_values();
    let infinite_values = get_multi_infinite_values();

    for (idx, parent_id) in parent_ids.iter().enumerate() {
        // Insert 'two' field values (cardinality: 2)
        if idx < two_values.len() {
            for (sort_order, value) in two_values[idx].iter().enumerate() {
                let field_id = generate_field_id();
                sqlx::query(
                    "INSERT INTO field_multi_two (id, user, parent_id, value, sort_order) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(&field_id)
                .bind(0) // Default system user
                .bind(parent_id)
                .bind(value)
                .bind(sort_order as i32)
                .execute(db.pool())
                .await?;
            }
            info!(
                "Inserted {} values for field 'two' of parent {}",
                two_values[idx].len(),
                parent_id
            );
        }

        // Insert 'infinite' field values (cardinality: -1)
        if idx < infinite_values.len() {
            for (sort_order, value) in infinite_values[idx].iter().enumerate() {
                let field_id = generate_field_id();
                sqlx::query(
                    "INSERT INTO field_multi_infinite (id, user, parent_id, value, sort_order) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(&field_id)
                .bind(0) // Default system user
                .bind(parent_id)
                .bind(value)
                .bind(sort_order as i32)
                .execute(db.pool())
                .await?;
            }
            info!(
                "Inserted {} values for field 'infinite' of parent {}",
                infinite_values[idx].len(),
                parent_id
            );
        }
    }

    Ok(())
}

/// Generate a unique ID for field entries using ULID
fn generate_field_id() -> String {
    format!("field_{}", Ulid::new())
}

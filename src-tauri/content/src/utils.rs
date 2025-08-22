//! Utility functions for content processing

use crate::error::ContentError;

/// Generate a URL-safe ID from a title string
///
/// This function converts a title into a valid ID by:
/// 1. Converting to lowercase
/// 2. Replacing punctuation with spaces
/// 3. Collapsing multiple spaces
/// 4. Replacing spaces with underscores
///
/// # Arguments
///
/// * `title` - The title string to convert
///
/// # Returns
///
/// A URL-safe ID string
///
/// # Example
///
/// ```rust
/// use content::generate_id_from_title;
///
/// assert_eq!(generate_id_from_title("Hello World!"), "hello_world");
/// assert_eq!(generate_id_from_title("Test: With Punctuation"), "test_with_punctuation");
/// ```
pub fn generate_id_from_title(title: &str) -> String {
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

/// Sanitize a string to create a valid URL slug
///
/// Similar to `generate_id_from_title` but uses hyphens instead of underscores,
/// which is more common for URL slugs.
///
/// # Arguments
///
/// * `text` - The text to convert to a slug
///
/// # Returns
///
/// A URL-safe slug string
pub fn sanitize_slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

/// Truncate a string to a maximum length, adding ellipsis if truncated
///
/// # Arguments
///
/// * `text` - The text to truncate
/// * `max_length` - Maximum length of the resulting string
///
/// # Returns
///
/// The truncated string with ellipsis if it was truncated
pub fn truncate_with_ellipsis(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else if max_length <= 3 {
        text.chars().take(max_length).collect()
    } else {
        format!(
            "{}...",
            text.chars().take(max_length - 3).collect::<String>()
        )
    }
}

/// Extract a summary from content text
///
/// Extracts the first paragraph or a specified number of characters as a summary.
///
/// # Arguments
///
/// * `content` - The full content text
/// * `max_length` - Maximum length of the summary
///
/// # Returns
///
/// A summary string
pub fn extract_summary(content: &str, max_length: usize) -> String {
    // Try to find the first paragraph
    let first_paragraph = content.split("\n\n").next().unwrap_or(content).trim();

    truncate_with_ellipsis(first_paragraph, max_length)
}

/// Strip HTML tags from content
///
/// Basic HTML tag removal for plain text extraction.
/// Note: This is a simple implementation and may not handle all edge cases.
///
/// # Arguments
///
/// * `html` - HTML content to strip
///
/// # Returns
///
/// Plain text with HTML tags removed
pub fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    // Clean up extra whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Validate that a string is a valid ID format
///
/// Valid IDs contain only lowercase letters, numbers, and underscores.
///
/// # Arguments
///
/// * `id` - The ID string to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err` with description if invalid
pub fn validate_id_format(id: &str) -> Result<(), ContentError> {
    if id.is_empty() {
        return Err(ContentError::ValidationError(
            "ID cannot be empty".to_string(),
        ));
    }

    if !id
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(ContentError::ValidationError(
            "ID can only contain lowercase letters, numbers, and underscores".to_string(),
        ));
    }

    if id.starts_with('_') || id.ends_with('_') {
        return Err(ContentError::ValidationError(
            "ID cannot start or end with underscore".to_string(),
        ));
    }

    Ok(())
}

/// Validate that a string is a valid slug format
///
/// Valid slugs contain only lowercase letters, numbers, and hyphens.
///
/// # Arguments
///
/// * `slug` - The slug string to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err` with description if invalid
pub fn validate_slug_format(slug: &str) -> Result<(), ContentError> {
    if slug.is_empty() {
        return Err(ContentError::ValidationError(
            "Slug cannot be empty".to_string(),
        ));
    }

    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ContentError::ValidationError(
            "Slug can only contain lowercase letters, numbers, and hyphens".to_string(),
        ));
    }

    if slug.starts_with('-') || slug.ends_with('-') {
        return Err(ContentError::ValidationError(
            "Slug cannot start or end with hyphen".to_string(),
        ));
    }

    if slug.contains("--") {
        return Err(ContentError::ValidationError(
            "Slug cannot contain consecutive hyphens".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id_from_title() {
        assert_eq!(generate_id_from_title("Hello World"), "hello_world");
        assert_eq!(
            generate_id_from_title("Test: With Punctuation!"),
            "test_with_punctuation"
        );
        assert_eq!(
            generate_id_from_title("Multiple   Spaces"),
            "multiple_spaces"
        );
        assert_eq!(
            generate_id_from_title("Numbers 123 Test"),
            "numbers_123_test"
        );
        assert_eq!(generate_id_from_title(""), "");
    }

    #[test]
    fn test_sanitize_slug() {
        assert_eq!(sanitize_slug("Hello World"), "hello-world");
        assert_eq!(
            sanitize_slug("Test: With Punctuation!"),
            "test-with-punctuation"
        );
        assert_eq!(sanitize_slug("Multiple   Spaces"), "multiple-spaces");
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        assert_eq!(truncate_with_ellipsis("Short", 10), "Short");
        assert_eq!(
            truncate_with_ellipsis("This is a long string", 10),
            "This is..."
        );
        assert_eq!(truncate_with_ellipsis("Test", 4), "Test");
        assert_eq!(truncate_with_ellipsis("Test", 3), "Tes");
    }

    #[test]
    fn test_extract_summary() {
        let content = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        assert_eq!(extract_summary(content, 20), "First paragraph.");

        let long_first = "This is a very long first paragraph that should be truncated.";
        assert_eq!(extract_summary(long_first, 20), "This is a very lo...");
    }

    #[test]
    fn test_strip_html_tags() {
        assert_eq!(strip_html_tags("<p>Hello <b>World</b></p>"), "Hello World");
        assert_eq!(strip_html_tags("No tags here"), "No tags here");
        assert_eq!(
            strip_html_tags("<div>  Multiple  spaces  </div>"),
            "Multiple spaces"
        );
    }

    #[test]
    fn test_validate_id_format() {
        assert!(validate_id_format("valid_id_123").is_ok());
        assert!(validate_id_format("").is_err());
        assert!(validate_id_format("_starts_with_underscore").is_err());
        assert!(validate_id_format("ends_with_underscore_").is_err());
        assert!(validate_id_format("has-hyphen").is_err());
        assert!(validate_id_format("UPPERCASE").is_err());
    }

    #[test]
    fn test_validate_slug_format() {
        assert!(validate_slug_format("valid-slug-123").is_ok());
        assert!(validate_slug_format("").is_err());
        assert!(validate_slug_format("-starts-with-hyphen").is_err());
        assert!(validate_slug_format("ends-with-hyphen-").is_err());
        assert!(validate_slug_format("has--double--hyphen").is_err());
        assert!(validate_slug_format("has_underscore").is_err());
        assert!(validate_slug_format("UPPERCASE").is_err());
    }
}

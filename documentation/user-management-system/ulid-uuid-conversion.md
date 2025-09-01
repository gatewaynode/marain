# ULID to UUID Conversion Strategy

## Overview

The Marain CMS project standardizes on ULIDs (Universally Unique Lexicographically Sortable Identifiers) for all entity and user identifiers. However, some external dependencies, particularly the `webauthn-rs` library used for PassKey authentication, require UUIDs. This document describes our conversion strategy to maintain ULID consistency while supporting UUID-dependent libraries.

## The Problem

- **Project Standard**: ULIDs are used throughout the system for their sortability and time-ordering properties
- **External Dependency**: The `webauthn-rs` library requires UUID v4 for user handles in WebAuthn operations
- **Solution**: A thin conversion layer that translates between ULIDs and UUIDs without data loss

## Implementation

### Conversion Module

The `ulid_uuid_bridge` module in the `user` crate provides bidirectional conversion functions:

```rust
// Located at: src-tauri/user/src/ulid_uuid_bridge.rs

/// Convert a ULID to a UUID (byte-for-byte identical)
pub fn ulid_to_uuid(ulid: &Ulid) -> Uuid

/// Convert a UUID back to a ULID
pub fn uuid_to_ulid(uuid: &Uuid) -> Ulid

/// Generate a new ULID and return both representations
pub fn generate_ulid_uuid_pair() -> (Ulid, Uuid)

/// Convert a ULID string to a UUID
pub fn ulid_string_to_uuid(ulid_str: &str) -> Option<Uuid>

/// Convert a UUID string to a ULID
pub fn uuid_string_to_ulid(uuid_str: &str) -> Option<Ulid>
```

### How It Works

Both ULIDs and UUIDs are 128-bit (16-byte) values. The conversion is a straightforward byte-for-byte copy that preserves the underlying value while changing the type:

1. **ULID to UUID**: Extract the 128-bit value from the ULID and create a UUID from it
2. **UUID to ULID**: Extract the 128-bit value from the UUID and create a ULID from it
3. **No Data Loss**: The conversion is lossless and reversible

### Usage Example

In the PassKey authentication flow:

```rust
// When starting PassKey registration
pub async fn start_registration(
    &self,
    db: &UserDatabase,
    user_id: &str,  // This is a ULID string
    username: &str,
) -> Result<(String, CreationChallengeResponse)> {
    // Convert user's ULID to UUID for webauthn-rs
    let user_uuid = ulid_string_to_uuid(user_id)
        .ok_or_else(|| UserError::Configuration(
            format!("Invalid user ID format: {}", user_id)
        ))?;
    
    // Use the UUID with webauthn-rs
    let (ccr, reg_state) = self.webauthn
        .start_passkey_registration(
            user_uuid,  // webauthn-rs expects UUID
            username,
            username,
            Some(exclude_credentials)
        )?;
    
    // Continue with registration...
}
```

## Guidelines for Developers

### When to Use the Conversion Layer

1. **External Libraries**: When calling external libraries that require UUIDs
2. **API Boundaries**: When interfacing with external systems that expect UUIDs
3. **Legacy Code**: When integrating with older code that uses UUIDs

### When NOT to Use the Conversion Layer

1. **Internal Code**: All new internal code should use ULIDs directly
2. **Database Storage**: Always store ULIDs in the database, never UUIDs
3. **API Responses**: Return ULIDs in API responses unless specifically required otherwise

### Best Practices

1. **Document Usage**: Always add a comment explaining why UUID conversion is needed
2. **Minimize Scope**: Convert at the boundary, not throughout the codebase
3. **Validate Input**: Always validate ULID strings before conversion
4. **Error Handling**: Handle conversion failures gracefully with meaningful error messages

## Database Considerations

The users table and all related tables use TEXT fields that store ULID strings:

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,  -- Stores ULID as string
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    -- ... other fields
)
```

This allows us to:
- Store ULIDs in their string representation
- Maintain sortability in database queries
- Easily identify records by their time-ordered nature

## Future Considerations

If the `webauthn-rs` library or other dependencies eventually support ULIDs natively, we can:
1. Remove the conversion layer
2. Update the calling code to pass ULIDs directly
3. Maintain backward compatibility through the existing database schema

## Testing

The conversion functions are thoroughly tested in `src-tauri/user/src/ulid_uuid_bridge.rs`:

- Round-trip conversion tests ensure no data loss
- String conversion tests validate parsing
- Invalid input tests ensure proper error handling

Run tests with:
```bash
cd src-tauri && cargo test -p user
```

## References

- [ULID Specification](https://github.com/ulid/spec)
- [UUID RFC 4122](https://tools.ietf.org/html/rfc4122)
- [webauthn-rs Documentation](https://docs.rs/webauthn-rs/)
- Reference implementation: `experiments/ulid_to_uuid/src/main.rs`
//! ULID to UUID conversion bridge for webauthn-rs compatibility
//!
//! This module provides a thin conversion layer between our system's ULIDs
//! and the UUID requirements of the webauthn-rs library. The conversion is
//! byte-for-byte identical, preserving the 128-bit value while changing the type.

use ulid::Ulid;
use uuid::Uuid;

/// Convert a ULID to a UUID for webauthn-rs compatibility
///
/// This performs a byte-for-byte conversion from ULID to UUID.
/// The underlying 128-bit value remains identical.
///
/// # Example
/// ```
/// use ulid::Ulid;
/// use user::ulid_uuid_bridge::ulid_to_uuid;
///
/// let ulid = Ulid::new();
/// let uuid = ulid_to_uuid(&ulid);
/// ```
pub fn ulid_to_uuid(ulid: &Ulid) -> Uuid {
    // Convert ULID's u128 representation directly to UUID
    // This is a straightforward byte-for-byte copy
    Uuid::from_u128(ulid.0)
}

/// Convert a UUID back to a ULID
///
/// This performs the reverse conversion from UUID to ULID.
/// Used when retrieving data from webauthn-rs that contains UUIDs.
///
/// # Example
/// ```
/// use uuid::Uuid;
/// use user::ulid_uuid_bridge::uuid_to_ulid;
///
/// let uuid = Uuid::new_v4();
/// let ulid = uuid_to_ulid(&uuid);
/// ```
pub fn uuid_to_ulid(uuid: &Uuid) -> Ulid {
    // Convert UUID's 128-bit representation to ULID
    Ulid::from(uuid.as_u128())
}

/// Generate a new ULID and return it as both ULID and UUID
///
/// This is a convenience function for when you need both representations
/// of the same identifier, commonly used in PassKey registration.
///
/// # Returns
/// A tuple of (ULID, UUID) representing the same 128-bit value
pub fn generate_ulid_uuid_pair() -> (Ulid, Uuid) {
    let ulid = Ulid::new();
    let uuid = ulid_to_uuid(&ulid);
    (ulid, uuid)
}

/// Convert a ULID string to a UUID
///
/// Parses a ULID from a string and converts it to UUID.
/// Returns None if the string is not a valid ULID.
pub fn ulid_string_to_uuid(ulid_str: &str) -> Option<Uuid> {
    ulid_str
        .parse::<Ulid>()
        .ok()
        .map(|ulid| ulid_to_uuid(&ulid))
}

/// Convert a UUID string to a ULID
///
/// Parses a UUID from a string and converts it to ULID.
/// Returns None if the string is not a valid UUID.
pub fn uuid_string_to_ulid(uuid_str: &str) -> Option<Ulid> {
    uuid_str
        .parse::<Uuid>()
        .ok()
        .map(|uuid| uuid_to_ulid(&uuid))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ulid_to_uuid_conversion() {
        let ulid = Ulid::new();
        let uuid = ulid_to_uuid(&ulid);

        // Verify the byte representations are identical
        assert_eq!(ulid.0.to_be_bytes(), *uuid.as_bytes());
    }

    #[test]
    fn test_uuid_to_ulid_conversion() {
        let original_ulid = Ulid::new();
        let uuid = ulid_to_uuid(&original_ulid);
        let converted_ulid = uuid_to_ulid(&uuid);

        // Verify round-trip conversion preserves the value
        assert_eq!(original_ulid, converted_ulid);
    }

    #[test]
    fn test_generate_ulid_uuid_pair() {
        let (ulid, uuid) = generate_ulid_uuid_pair();

        // Verify the pair represents the same value
        assert_eq!(ulid.0.to_be_bytes(), *uuid.as_bytes());

        // Verify conversion consistency
        assert_eq!(uuid, ulid_to_uuid(&ulid));
    }

    #[test]
    fn test_string_conversions() {
        let ulid = Ulid::new();
        let ulid_str = ulid.to_string();

        // Test ULID string to UUID
        let uuid = ulid_string_to_uuid(&ulid_str);
        assert!(uuid.is_some());
        assert_eq!(uuid.unwrap(), ulid_to_uuid(&ulid));

        // Test UUID string to ULID
        let uuid = ulid_to_uuid(&ulid);
        let uuid_str = uuid.to_string();
        let converted_ulid = uuid_string_to_ulid(&uuid_str);
        assert!(converted_ulid.is_some());
        assert_eq!(converted_ulid.unwrap(), ulid);
    }

    #[test]
    fn test_invalid_string_conversions() {
        assert!(ulid_string_to_uuid("invalid").is_none());
        assert!(uuid_string_to_ulid("invalid").is_none());
    }
}

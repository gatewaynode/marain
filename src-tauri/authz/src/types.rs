//! Core authorization types for the CEDAR-based authorization system.
//!
//! # Security Considerations for Authorization System
//!
//! This module contains the foundational types for authorization. As we build out
//! the full authorization system, we must carefully address the following security concerns:
//!
//! ## 1. Principal Identity & Authentication
//! - Ensure principals are properly authenticated before authorization checks
//! - Validate that principal IDs match authenticated session identities
//! - Prevent principal spoofing or injection attacks
//! - Handle session expiration and token validation
//!
//! ## 2. Policy Management & Integrity
//! - Validate policy syntax and semantics before loading
//! - Implement policy versioning and rollback capabilities
//! - Audit all policy changes with timestamps and authors
//! - Prevent unauthorized policy modifications
//! - Use cryptographic signatures for policy files in production
//!
//! ## 3. Resource Access Control
//! - Validate resource IDs exist and are accessible
//! - Check resource ownership and hierarchical permissions
//! - Prevent path traversal and resource enumeration attacks
//! - Implement proper resource isolation between tenants/users
//!
//! ## 4. Action Validation
//! - Whitelist allowed actions per resource type
//! - Prevent privilege escalation through action manipulation
//! - Validate action parameters and side effects
//! - Log all privileged actions for audit trails
//!
//! ## 5. Attribute-Based Access Control (ABAC)
//! - Validate all attributes used in policy decisions
//! - Ensure attribute freshness (prevent stale data attacks)
//! - Protect sensitive attributes in transit and at rest
//! - Implement attribute caching with proper TTLs
//!
//! ## 6. Performance & Denial of Service
//! - Implement rate limiting on authorization checks
//! - Cache authorization decisions appropriately
//! - Set timeouts on policy evaluation
//! - Prevent policy bombs (exponential complexity policies)
//! - Monitor for performance degradation attacks
//!
//! ## 7. Logging & Auditing
//! - Log all authorization decisions (allow/deny) with context
//! - Include principal, action, resource, and decision in logs
//! - Protect audit logs from tampering
//! - Implement log retention and analysis policies
//! - Alert on suspicious authorization patterns
//!
//! ## 8. Error Handling & Information Disclosure
//! - Never leak policy details in error messages
//! - Provide minimal information on deny decisions
//! - Log detailed errors securely for debugging
//! - Implement proper error recovery without bypassing authorization
//!
//! ## 9. Hierarchical Permissions & Groups
//! - Validate group membership chains
//! - Prevent circular group dependencies
//! - Cache group hierarchies with proper invalidation
//! - Implement depth limits on hierarchy traversal
//!
//! ## 10. Integration Security
//! - Validate all data from external sources (DB, cache, etc.)
//! - Use parameterized queries to prevent injection
//! - Implement proper database access controls
//! - Secure inter-service communication
//!
//! ## 11. Testing & Validation
//! - Test authorization with boundary conditions
//! - Include negative test cases (should deny)
//! - Test policy conflicts and precedence rules
//! - Perform security code reviews
//! - Run regular penetration tests
//!
//! ## 12. Compliance & Standards
//! - Follow principle of least privilege
//! - Implement separation of duties where applicable
//! - Support compliance requirements (GDPR, HIPAA, etc.)
//! - Document authorization architecture and decisions

use serde::{Deserialize, Serialize};

/// Represents a principal (user, service, or entity) making an authorization request.
///
/// In CEDAR terminology, a Principal is the "who" in "who can do what to which resource".
/// This will eventually contain user IDs (ULIDs), roles, and group memberships.
///
/// # Security Note
/// In the full implementation, principals must be derived from authenticated sessions only.
/// Never trust principal data from untrusted sources.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Principal {
    /// The unique identifier for this principal (e.g., ULID for users)
    pub id: String,

    /// The type of principal (e.g., "User", "Service", "Group")
    pub entity_type: String,
}

impl Principal {
    /// Creates a new Principal with the given ID and type.
    pub fn new(id: impl Into<String>, entity_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            entity_type: entity_type.into(),
        }
    }

    /// Creates a Principal representing a user with the given ULID.
    pub fn user(ulid: impl Into<String>) -> Self {
        Self::new(ulid, "User")
    }

    /// Creates a Principal representing an anonymous/unauthenticated user.
    pub fn anonymous() -> Self {
        Self::new("anonymous", "User")
    }
}

/// Represents an action being performed in an authorization request.
///
/// In CEDAR terminology, an Action is the "what" in "who can do what to which resource".
/// Examples: "read", "write", "delete", "update", "create"
///
/// # Security Note
/// Actions should be validated against a whitelist of allowed operations for each resource type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Action {
    /// The name of the action being performed (e.g., "read", "write", "delete")
    pub name: String,
}

impl Action {
    /// Creates a new Action with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    /// Creates an Action representing a read operation.
    pub fn read() -> Self {
        Self::new("read")
    }

    /// Creates an Action representing a write/update operation.
    pub fn write() -> Self {
        Self::new("write")
    }

    /// Creates an Action representing a create operation.
    pub fn create() -> Self {
        Self::new("create")
    }

    /// Creates an Action representing a delete operation.
    pub fn delete() -> Self {
        Self::new("delete")
    }
}

/// Represents a resource being accessed in an authorization request.
///
/// In CEDAR terminology, a Resource is the "which" in "who can do what to which resource".
/// This represents the entity being accessed (e.g., a specific content item, user profile, etc.)
///
/// # Security Note
/// Resource IDs must be validated to exist and be accessible before authorization checks.
/// Implement proper resource isolation to prevent cross-tenant access.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resource {
    /// The unique identifier for this resource
    pub id: String,

    /// The type of resource (e.g., "Snippet", "User", "Link")
    pub entity_type: String,
}

impl Resource {
    /// Creates a new Resource with the given ID and type.
    pub fn new(id: impl Into<String>, entity_type: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            entity_type: entity_type.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_principal_creation() {
        let principal = Principal::new("user123", "User");
        assert_eq!(principal.id, "user123");
        assert_eq!(principal.entity_type, "User");
    }

    #[test]
    fn test_principal_user_helper() {
        let principal = Principal::user("01H8XGJWBWBAQ4Z4M9D5K4Z3E1");
        assert_eq!(principal.entity_type, "User");
    }

    #[test]
    fn test_principal_anonymous() {
        let principal = Principal::anonymous();
        assert_eq!(principal.id, "anonymous");
        assert_eq!(principal.entity_type, "User");
    }

    #[test]
    fn test_action_creation() {
        let action = Action::new("read");
        assert_eq!(action.name, "read");
    }

    #[test]
    fn test_action_helpers() {
        assert_eq!(Action::read().name, "read");
        assert_eq!(Action::write().name, "write");
        assert_eq!(Action::create().name, "create");
        assert_eq!(Action::delete().name, "delete");
    }

    #[test]
    fn test_resource_creation() {
        let resource = Resource::new("snippet123", "Snippet");
        assert_eq!(resource.id, "snippet123");
        assert_eq!(resource.entity_type, "Snippet");
    }
}

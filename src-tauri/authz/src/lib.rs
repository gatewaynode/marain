//! CEDAR-based authorization engine for the Marain CMS.
//!
//! This crate provides fine-grained authorization capabilities using the
//! [CEDAR policy language](https://www.cedarpolicy.com/). It defines core
//! authorization types (Principal, Action, Resource) and an `AuthzEngine`
//! that evaluates authorization requests against CEDAR policies.
//!
//! # Architecture Overview
//!
//! The authorization flow follows this pattern:
//!
//! 1. **Request arrives** at the API layer
//! 2. **Authentication** verifies the user's identity
//! 3. **Authorization middleware** extracts Principal, Action, Resource
//! 4. **AuthzEngine** evaluates the request against policies
//! 5. **Decision** is made: Allow or Deny
//!
//! # Implementation Stages
//!
//! This crate is being built in stages:
//!
//! - **Stage 1 (Current)**: Static proof-of-concept with hardcoded policies and entities
//! - **Stage 2**: Integration with API middleware using static PoC
//! - **Stage 3**: Dynamic policy loading from files and database
//! - **Stage 4**: Full testing, documentation, and production readiness
//!
//! # Security Architecture
//!
//! See `types.rs` for comprehensive security considerations covering:
//! - Principal identity & authentication
//! - Policy management & integrity
//! - Resource access control
//! - Action validation
//! - Attribute-based access control (ABAC)
//! - Performance & denial of service prevention
//! - Logging & auditing
//! - Error handling & information disclosure
//! - Hierarchical permissions & groups
//! - Integration security
//! - Testing & validation
//! - Compliance & standards

pub mod error;
pub mod types;

use cedar_policy::{
    Authorizer, Context, Decision, Entities, EntityId, EntityTypeName, EntityUid, PolicySet,
    Request,
};
use error::{AuthzError, Result};
use std::str::FromStr;
use types::{Action, Principal, Resource};

/// The core authorization engine for evaluating CEDAR policies.
///
/// This engine manages the policy set, entity store, and evaluation context
/// needed to make authorization decisions.
///
/// # Current Implementation (Stage 1)
///
/// This is a static proof-of-concept that uses hardcoded policies and entities
/// to validate the CEDAR integration. It will be replaced with dynamic loading
/// in later stages.
///
/// # Example
///
/// ```rust
/// use authz::{AuthzEngine, types::{Principal, Action, Resource}};
///
/// let engine = AuthzEngine::new();
/// let principal = Principal::user("test_user");
/// let action = Action::read();
/// let resource = Resource::new("doc123", "Document");
///
/// match engine.is_authorized_static_poc(&principal, &action, &resource) {
///     Ok(true) => println!("Access granted"),
///     Ok(false) => println!("Access denied"),
///     Err(e) => eprintln!("Authorization error: {}", e),
/// }
/// ```
pub struct AuthzEngine {
    authorizer: Authorizer,
}

impl AuthzEngine {
    /// Creates a new authorization engine.
    ///
    /// This initializes the CEDAR authorizer that will be used for all
    /// authorization decisions.
    pub fn new() -> Self {
        Self {
            authorizer: Authorizer::new(),
        }
    }

    /// Static proof-of-concept authorization check with hardcoded policies and entities.
    ///
    /// This method demonstrates the core CEDAR authorization flow using hardcoded
    /// policies and entities. It serves as a foundational proof-of-concept before
    /// implementing dynamic policy and entity loading.
    ///
    /// # Hardcoded Policy Logic
    ///
    /// The current hardcoded policy allows:
    /// - User "test_user" can perform "read" action on any resource
    /// - User "admin_user" can perform any action on any resource
    /// - All other combinations are denied by default
    ///
    /// # Arguments
    ///
    /// * `principal` - The entity making the request (user, service, etc.)
    /// * `action` - The action being performed (read, write, delete, etc.)
    /// * `resource` - The resource being accessed
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the action is explicitly allowed by policy
    /// - `Ok(false)` if the action is denied (no explicit allow)
    /// - `Err(AuthzError)` if there was an error evaluating the policy
    ///
    /// # Security Note
    ///
    /// This is a proof-of-concept only. In production:
    /// - Policies must be loaded from secure, version-controlled files
    /// - Entities must be constructed from authenticated database data
    /// - All authorization decisions must be logged for audit trails
    /// - Deny-by-default should be strictly enforced
    pub fn is_authorized_static_poc(
        &self,
        principal: &Principal,
        action: &Action,
        resource: &Resource,
    ) -> Result<bool> {
        // Create hardcoded policy set
        let policy_set = self.create_hardcoded_policies()?;

        // Create hardcoded entities
        let entities = self.create_hardcoded_entities()?;

        // Build the CEDAR request
        let request = self.build_cedar_request(principal, action, resource)?;

        // Evaluate the authorization request
        let response = self
            .authorizer
            .is_authorized(&request, &policy_set, &entities);

        // Return the decision (Allow = true, Deny = false)
        Ok(response.decision() == Decision::Allow)
    }

    /// Creates a hardcoded policy set for the proof-of-concept.
    ///
    /// # Policies
    ///
    /// 1. Allow test_user to read any resource
    /// 2. Allow admin_user to perform any action on any resource
    ///
    /// # Security Note
    ///
    /// In production, policies will be loaded from `.cedar` files in the config directory,
    /// validated, and potentially signed for integrity.
    fn create_hardcoded_policies(&self) -> Result<PolicySet> {
        let policy_src = r#"
            permit(
                principal == User::"test_user",
                action == Action::"read",
                resource
            );

            permit(
                principal == User::"admin_user",
                action,
                resource
            );
        "#;

        // Parse the policy text into a PolicySet
        // Cedar 3.x uses parse() method on the policy string
        PolicySet::from_str(policy_src).map_err(|e| AuthzError::PolicyParse(e.to_string()))
    }

    /// Creates a hardcoded entity store for the proof-of-concept.
    ///
    /// # Entities
    ///
    /// - User::"test_user" - A standard test user
    /// - User::"admin_user" - An admin user with full permissions
    /// - Action::"read", Action::"write", Action::"delete" - Standard actions
    ///
    /// # Security Note
    ///
    /// In production, entities will be dynamically constructed from:
    /// - User database records (with ULIDs)
    /// - Content entity data
    /// - Group membership hierarchies
    /// - Resource attributes from schemas
    fn create_hardcoded_entities(&self) -> Result<Entities> {
        let entities_json = serde_json::json!([
            {
                "uid": { "type": "User", "id": "test_user" },
                "attrs": {},
                "parents": []
            },
            {
                "uid": { "type": "User", "id": "admin_user" },
                "attrs": {},
                "parents": []
            },
            {
                "uid": { "type": "User", "id": "anonymous" },
                "attrs": {},
                "parents": []
            },
            {
                "uid": { "type": "Action", "id": "read" },
                "attrs": {},
                "parents": []
            },
            {
                "uid": { "type": "Action", "id": "write" },
                "attrs": {},
                "parents": []
            },
            {
                "uid": { "type": "Action", "id": "delete" },
                "attrs": {},
                "parents": []
            }
        ]);

        Entities::from_json_value(entities_json, None)
            .map_err(|e| AuthzError::EntityCreation(e.to_string()))
    }

    /// Builds a CEDAR authorization request from our types.
    ///
    /// This converts our application's Principal, Action, and Resource types
    /// into CEDAR's internal Request format.
    ///
    /// # Security Note
    ///
    /// The principal ID must be validated against the authenticated session.
    /// Resource IDs should be validated to exist before authorization.
    fn build_cedar_request(
        &self,
        principal: &Principal,
        action: &Action,
        resource: &Resource,
    ) -> Result<Request> {
        // Create CEDAR EntityUid for principal using the new() constructor
        let principal_uid = EntityUid::from_type_name_and_id(
            EntityTypeName::from_str(&principal.entity_type).map_err(|e| {
                AuthzError::EntityCreation(format!("Invalid principal type: {}", e))
            })?,
            EntityId::new(&principal.id),
        );

        // Create CEDAR EntityUid for action
        let action_uid = EntityUid::from_type_name_and_id(
            EntityTypeName::from_str("Action")
                .map_err(|e| AuthzError::EntityCreation(format!("Invalid action type: {}", e)))?,
            EntityId::new(&action.name),
        );

        // Create CEDAR EntityUid for resource
        let resource_uid = EntityUid::from_type_name_and_id(
            EntityTypeName::from_str(&resource.entity_type)
                .map_err(|e| AuthzError::EntityCreation(format!("Invalid resource type: {}", e)))?,
            EntityId::new(&resource.id),
        );

        // Build the request with empty context and no schema (will be added in later stages)
        Request::new(
            Some(principal_uid),
            Some(action_uid),
            Some(resource_uid),
            Context::empty(),
            None, // No schema in this static PoC
        )
        .map_err(|e| AuthzError::EvaluationError(e.to_string()))
    }
}

impl Default for AuthzEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that test_user can read a document
    #[test]
    fn test_user_can_read() {
        let engine = AuthzEngine::new();
        let principal = Principal::user("test_user");
        let action = Action::read();
        let resource = Resource::new("doc123", "Document");

        let result = engine.is_authorized_static_poc(&principal, &action, &resource);
        assert!(result.is_ok());
        assert!(result.unwrap(), "test_user should be allowed to read");
    }

    /// Test that test_user cannot write to a document
    #[test]
    fn test_user_cannot_write() {
        let engine = AuthzEngine::new();
        let principal = Principal::user("test_user");
        let action = Action::write();
        let resource = Resource::new("doc123", "Document");

        let result = engine.is_authorized_static_poc(&principal, &action, &resource);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "test_user should not be allowed to write");
    }

    /// Test that test_user cannot delete a document
    #[test]
    fn test_user_cannot_delete() {
        let engine = AuthzEngine::new();
        let principal = Principal::user("test_user");
        let action = Action::delete();
        let resource = Resource::new("doc123", "Document");

        let result = engine.is_authorized_static_poc(&principal, &action, &resource);
        assert!(result.is_ok());
        assert!(
            !result.unwrap(),
            "test_user should not be allowed to delete"
        );
    }

    /// Test that admin_user can perform any action
    #[test]
    fn test_admin_can_do_anything() {
        let engine = AuthzEngine::new();
        let principal = Principal::user("admin_user");
        let resource = Resource::new("doc123", "Document");

        // Test all actions
        for action in [Action::read(), Action::write(), Action::delete()] {
            let result = engine.is_authorized_static_poc(&principal, &action, &resource);
            assert!(result.is_ok());
            assert!(
                result.unwrap(),
                "admin_user should be allowed to perform {}",
                action.name
            );
        }
    }

    /// Test that anonymous user is denied by default
    #[test]
    fn test_anonymous_denied() {
        let engine = AuthzEngine::new();
        let principal = Principal::anonymous();
        let action = Action::read();
        let resource = Resource::new("doc123", "Document");

        let result = engine.is_authorized_static_poc(&principal, &action, &resource);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "anonymous should be denied by default");
    }

    /// Test that unknown user is denied
    #[test]
    fn test_unknown_user_denied() {
        let engine = AuthzEngine::new();
        let principal = Principal::user("unknown_user");
        let action = Action::read();
        let resource = Resource::new("doc123", "Document");

        let result = engine.is_authorized_static_poc(&principal, &action, &resource);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "unknown user should be denied");
    }

    /// Test that hardcoded policies parse correctly
    #[test]
    fn test_policy_parsing() {
        let engine = AuthzEngine::new();
        let result = engine.create_hardcoded_policies();
        assert!(
            result.is_ok(),
            "Hardcoded policies should parse successfully"
        );
    }

    /// Test that hardcoded entities are created correctly
    #[test]
    fn test_entity_creation() {
        let engine = AuthzEngine::new();
        let result = engine.create_hardcoded_entities();
        assert!(
            result.is_ok(),
            "Hardcoded entities should be created successfully"
        );
    }

    /// Test request building with various input types
    #[test]
    fn test_request_building() {
        let engine = AuthzEngine::new();

        let test_cases = vec![
            (
                Principal::user("test"),
                Action::read(),
                Resource::new("id", "Type"),
            ),
            (
                Principal::anonymous(),
                Action::write(),
                Resource::new("123", "Document"),
            ),
            (
                Principal::user("admin"),
                Action::delete(),
                Resource::new("xyz", "Snippet"),
            ),
        ];

        for (principal, action, resource) in test_cases {
            let result = engine.build_cedar_request(&principal, &action, &resource);
            assert!(
                result.is_ok(),
                "Request building should succeed for valid inputs"
            );
        }
    }

    /// Test deny-by-default security principle
    #[test]
    fn test_deny_by_default() {
        let engine = AuthzEngine::new();

        // Create a user and action that aren't explicitly allowed
        let principal = Principal::user("random_user");
        let action = Action::create();
        let resource = Resource::new("new_doc", "Document");

        let result = engine.is_authorized_static_poc(&principal, &action, &resource);
        assert!(result.is_ok());
        assert!(
            !result.unwrap(),
            "Should deny by default when no policy matches"
        );
    }

    /// Test multiple authorization checks in sequence
    #[test]
    fn test_multiple_checks() {
        let engine = AuthzEngine::new();

        // First check - should succeed
        let result1 = engine.is_authorized_static_poc(
            &Principal::user("test_user"),
            &Action::read(),
            &Resource::new("doc1", "Document"),
        );
        assert!(result1.is_ok() && result1.unwrap());

        // Second check - should fail
        let result2 = engine.is_authorized_static_poc(
            &Principal::user("test_user"),
            &Action::write(),
            &Resource::new("doc1", "Document"),
        );
        assert!(result2.is_ok() && !result2.unwrap());

        // Third check - should succeed
        let result3 = engine.is_authorized_static_poc(
            &Principal::user("admin_user"),
            &Action::write(),
            &Resource::new("doc1", "Document"),
        );
        assert!(result3.is_ok() && result3.unwrap());
    }
}

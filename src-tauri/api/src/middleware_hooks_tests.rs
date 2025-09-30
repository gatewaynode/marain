//! Integration tests for authorization middleware
//!
//! These tests verify the authorization logic by testing the helper functions
//! that extract Principal, Action, and Resource from requests, and by testing
//! the authorization engine directly.

#[cfg(test)]
mod tests {
    use super::super::middleware_hooks::*;
    use authz::{
        types::{Action, Principal, Resource},
        AuthzEngine,
    };
    use axum::{
        body::Body,
        http::{Method, Request},
    };

    #[test]
    fn test_extract_principal_from_admin_path() {
        let request = Request::builder()
            .uri("/admin/users")
            .body(Body::empty())
            .unwrap();

        let principal = extract_principal_from_request(&request);
        assert_eq!(principal.id, "admin_user");
        assert_eq!(principal.entity_type, "User");
    }

    #[test]
    fn test_extract_principal_from_health_path() {
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let principal = extract_principal_from_request(&request);
        assert_eq!(principal.id, "test_user");
    }

    #[test]
    fn test_extract_principal_from_regular_path() {
        let request = Request::builder()
            .uri("/api/entity")
            .body(Body::empty())
            .unwrap();

        let principal = extract_principal_from_request(&request);
        assert_eq!(principal.id, "test_user");
    }

    #[test]
    fn test_extract_action_from_get_method() {
        let method = Method::GET;
        let action = extract_action_from_method(&method);
        assert_eq!(action.name, "read");
    }

    #[test]
    fn test_extract_action_from_post_method() {
        let method = Method::POST;
        let action = extract_action_from_method(&method);
        assert_eq!(action.name, "write");
    }

    #[test]
    fn test_extract_action_from_delete_method() {
        let method = Method::DELETE;
        let action = extract_action_from_method(&method);
        assert_eq!(action.name, "delete");
    }

    #[test]
    fn test_extract_resource_from_entity_path() {
        let resource = extract_resource_from_path("/api/v1/entity/read/snippet");
        assert_eq!(resource.entity_type, "snippet");
    }

    #[test]
    fn test_extract_resource_from_health_path() {
        let resource = extract_resource_from_path("/health");
        assert_eq!(resource.entity_type, "HealthCheck");
        assert_eq!(resource.id, "health");
    }

    #[test]
    fn test_authorization_engine_test_user_read() {
        let engine = AuthzEngine::new();
        let principal = Principal::user("test_user");
        let action = Action::read();
        let resource = Resource::new("doc1", "Document");

        let result = engine.is_authorized_static_poc(&principal, &action, &resource);
        assert!(result.is_ok());
        assert!(result.unwrap(), "test_user should be allowed to read");
    }

    #[test]
    fn test_authorization_engine_test_user_write_denied() {
        let engine = AuthzEngine::new();
        let principal = Principal::user("test_user");
        let action = Action::write();
        let resource = Resource::new("doc1", "Document");

        let result = engine.is_authorized_static_poc(&principal, &action, &resource);
        assert!(result.is_ok());
        assert!(!result.unwrap(), "test_user should be denied write");
    }

    #[test]
    fn test_authorization_engine_admin_user_any_action() {
        let engine = AuthzEngine::new();
        let principal = Principal::user("admin_user");
        let resource = Resource::new("doc1", "Document");

        // Test multiple actions
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

    #[test]
    fn test_authorization_engine_deny_by_default() {
        let engine = AuthzEngine::new();
        let principal = Principal::user("unknown_user");
        let action = Action::read();
        let resource = Resource::new("doc1", "Document");

        let result = engine.is_authorized_static_poc(&principal, &action, &resource);
        assert!(result.is_ok());
        assert!(
            !result.unwrap(),
            "unknown users should be denied by default"
        );
    }

    #[test]
    fn test_end_to_end_authorization_flow() {
        // Simulate the full middleware flow without actually creating a server

        // Test case 1: test_user reading health endpoint (should be allowed)
        let request1 = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let principal1 = extract_principal_from_request(&request1);
        let action1 = extract_action_from_method(request1.method());
        let resource1 = extract_resource_from_path(request1.uri().path());

        let engine = AuthzEngine::new();
        let result1 = engine.is_authorized_static_poc(&principal1, &action1, &resource1);
        assert!(result1.unwrap(), "GET /health should be allowed");

        // Test case 2: test_user posting to entity endpoint (should be denied)
        let request2 = Request::builder()
            .uri("/api/v1/entity/create/snippet")
            .method("POST")
            .body(Body::empty())
            .unwrap();

        let principal2 = extract_principal_from_request(&request2);
        let action2 = extract_action_from_method(request2.method());
        let resource2 = extract_resource_from_path(request2.uri().path());

        let result2 = engine.is_authorized_static_poc(&principal2, &action2, &resource2);
        assert!(
            !result2.unwrap(),
            "POST /entity should be denied for test_user"
        );

        // Test case 3: admin_user posting to admin endpoint (should be allowed)
        let request3 = Request::builder()
            .uri("/admin/settings")
            .method("POST")
            .body(Body::empty())
            .unwrap();

        let principal3 = extract_principal_from_request(&request3);
        let action3 = extract_action_from_method(request3.method());
        let resource3 = extract_resource_from_path(request3.uri().path());

        let result3 = engine.is_authorized_static_poc(&principal3, &action3, &resource3);
        assert!(
            result3.unwrap(),
            "POST /admin should be allowed for admin_user"
        );
    }
}

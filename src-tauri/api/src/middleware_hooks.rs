use authz::{
    types::{Action, Principal, Resource},
    AuthzEngine,
};
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{debug, info, warn};

use crate::AppState;

/// Authorization middleware using CEDAR policies
///
/// This middleware integrates the AuthzEngine to enforce authorization policies
/// on incoming requests. Currently uses the static proof-of-concept with hardcoded
/// policies and entities from Stage 1.
///
/// # Authorization Flow
///
/// 1. Extract Principal from request (placeholder user for now)
/// 2. Extract Action from HTTP method
/// 3. Extract Resource from URI path
/// 4. Call AuthzEngine::is_authorized_static_poc()
/// 5. Return 403 Forbidden on Deny, pass through on Allow
///
/// # Future Enhancements (Stage 3)
///
/// - Extract actual user from authenticated session
/// - Extract resource ID and type from path parameters
/// - Use dynamic policies and entities from database
/// - Add request context attributes for more nuanced decisions
///
/// # Security Notes
///
/// - Currently uses placeholder principals for testing
/// - In production, principal must come from authenticated session only
/// - All authorization decisions are logged for audit trails
/// - Deny-by-default is enforced: any error results in 403
pub async fn authorization_middleware(
    State(_state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();

    info!("AUTHZ MIDDLEWARE: Processing {} request to {}", method, uri);

    // Create the authorization engine
    let engine = AuthzEngine::new();

    // Extract Principal from request
    // TODO (Stage 3): Extract from authenticated session
    // For now, use placeholder principals for testing the middleware flow
    let principal = extract_principal_from_request(&request);

    // Extract Action from HTTP method
    let action = extract_action_from_method(&method);

    // Extract Resource from URI path
    // TODO (Stage 3): Extract actual resource ID and type from path parameters
    let resource = extract_resource_from_path(path);

    debug!(
        "AUTHZ MIDDLEWARE: Checking authorization for principal={:?}, action={:?}, resource={:?}",
        principal, action, resource
    );

    // Make authorization decision using static PoC
    match engine.is_authorized_static_poc(&principal, &action, &resource) {
        Ok(true) => {
            info!(
                "AUTHZ MIDDLEWARE: Access ALLOWED for {} {} {}",
                principal.id, action.name, resource.entity_type
            );
            // Authorization passed, continue to next middleware/handler
            Ok(next.run(request).await)
        }
        Ok(false) => {
            warn!(
                "AUTHZ MIDDLEWARE: Access DENIED for {} {} {}",
                principal.id, action.name, resource.entity_type
            );
            // Authorization denied, return 403 Forbidden
            Err(StatusCode::FORBIDDEN)
        }
        Err(e) => {
            // Authorization error occurred, log and deny access
            warn!(
                "AUTHZ MIDDLEWARE: Authorization error for {} {} {}: {}",
                principal.id, action.name, resource.entity_type, e
            );
            // On error, fail closed (deny access)
            Err(StatusCode::FORBIDDEN)
        }
    }
}

/// Extract principal from the request
///
/// # Current Implementation (Stage 2)
///
/// Uses placeholder principals for testing:
/// - Requests to /admin/* paths use "admin_user"
/// - Other requests use "test_user"
///
/// # Future Implementation (Stage 3)
///
/// Will extract the actual user from:
/// - Session cookies
/// - JWT tokens
/// - API keys
/// - Or return anonymous principal if unauthenticated
pub fn extract_principal_from_request(request: &Request<Body>) -> Principal {
    // TODO (Stage 3): Extract from authenticated session
    // For now, use simple path-based logic for testing

    let path = request.uri().path();

    if path.contains("/admin") {
        Principal::user("admin_user")
    } else if path.contains("/health") {
        // Health checks can use test_user
        Principal::user("test_user")
    } else {
        // Default to test_user for other paths
        Principal::user("test_user")
    }
}

/// Extract action from HTTP method
///
/// Maps HTTP methods to authorization actions:
/// - GET, HEAD -> "read"
/// - POST -> "write" (covers both create and update)
/// - PUT, PATCH -> "write"
/// - DELETE -> "delete"
pub fn extract_action_from_method(method: &axum::http::Method) -> Action {
    match method.as_str() {
        "GET" | "HEAD" => Action::read(),
        "POST" | "PUT" | "PATCH" => Action::write(),
        "DELETE" => Action::delete(),
        _ => Action::read(), // Default to read for unknown methods
    }
}

/// Extract resource from URI path
///
/// # Current Implementation (Stage 2)
///
/// Creates a generic resource based on the path:
/// - Extracts resource type from path segments
/// - Uses placeholder ID
///
/// # Future Implementation (Stage 3)
///
/// Will extract:
/// - Actual resource ID from path parameters
/// - Resource type from route definition
/// - Load resource attributes from database for ABAC
pub fn extract_resource_from_path(path: &str) -> Resource {
    // TODO (Stage 3): Extract actual resource ID and type from path parameters

    // Simple path-based resource type extraction for testing
    if path.contains("/entity") {
        // Extract entity type from path if possible
        let segments: Vec<&str> = path.split('/').collect();
        for (i, segment) in segments.iter().enumerate() {
            if *segment == "entity" && i + 1 < segments.len() {
                let operation = segments[i + 1];
                if i + 2 < segments.len() {
                    let entity_type = segments[i + 2];
                    return Resource::new("placeholder_id", entity_type);
                } else {
                    return Resource::new("placeholder_id", operation);
                }
            }
        }
        Resource::new("placeholder_id", "Entity")
    } else if path.contains("/health") {
        Resource::new("health", "HealthCheck")
    } else {
        Resource::new("unknown", "Unknown")
    }
}

/// Request processing middleware hook
/// This runs after authentication to modify/process incoming requests
pub async fn request_middleware(
    State(_state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    // Log incoming request processing
    info!(
        "REQUEST MIDDLEWARE: Processing incoming {} request to {}",
        method, uri
    );

    // TODO: Add any request preprocessing logic here
    // Examples:
    // - Request ID injection
    // - Rate limiting checks
    // - Request validation
    // - Header manipulation

    debug!("REQUEST MIDDLEWARE: Request preprocessing complete");

    // Continue to next middleware/handler
    let response = next.run(request).await;

    let duration = start.elapsed();
    debug!("REQUEST MIDDLEWARE: Request processed in {:?}", duration);

    Ok(response)
}

/// Response processing middleware hook
/// This runs before sending the response to modify/process outgoing responses
pub async fn response_middleware(
    State(_state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();

    // Process the request through the handler chain
    let mut response = next.run(request).await;

    // Log response processing
    info!(
        "RESPONSE MIDDLEWARE: Processing response for {} {}",
        method, uri
    );

    // TODO: Add any response postprocessing logic here
    // Examples:
    // - Response caching
    // - Response compression
    // - Header injection (CORS, security headers, etc.)
    // - Response transformation
    // - Metrics collection

    // Add custom headers to demonstrate the middleware is working
    let headers = response.headers_mut();
    headers.insert("X-Marain-Version", "1.0.0".parse().unwrap());
    headers.insert("X-Marain-Processed", "true".parse().unwrap());

    debug!("RESPONSE MIDDLEWARE: Response postprocessing complete");

    Ok(response)
}

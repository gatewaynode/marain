use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    body::Body,
};
use tracing::{info, debug};
use std::time::Instant;

use crate::AppState;

/// Authentication middleware hook (stubbed - passes through for now)
/// This will be the first middleware to process incoming requests
pub async fn auth_middleware(
    State(_state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    // Log authentication check (stubbed for now)
    info!(
        "AUTH MIDDLEWARE: Processing {} request to {}",
        method, uri
    );
    
    // TODO: Implement actual authentication logic here
    // For now, just pass through all requests
    debug!("AUTH MIDDLEWARE: Authentication check passed (stub mode)");
    
    // Continue to next middleware
    Ok(next.run(request).await)
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
    debug!(
        "REQUEST MIDDLEWARE: Request processed in {:?}",
        duration
    );
    
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
    headers.insert(
        "X-Marain-Version",
        "1.0.0".parse().unwrap(),
    );
    headers.insert(
        "X-Marain-Processed",
        "true".parse().unwrap(),
    );
    
    debug!("RESPONSE MIDDLEWARE: Response postprocessing complete");
    
    Ok(response)
}
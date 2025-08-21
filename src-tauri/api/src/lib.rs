use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod error;
pub mod handlers;
pub mod middleware_hooks;
pub mod models;
pub mod server;
pub mod test_data;

// Re-export server functions for convenience
pub use server::{
    spawn_server, spawn_server_with_config, start_server, start_server_with_config, ApiConfig,
};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<database::Database>,
    pub cache: Arc<json_cache::CacheManager>,
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::entity::read_entity,
        handlers::entity::list_entities,
        handlers::entity::create_entity,
        handlers::entity::update_entity,
        handlers::entity::delete_entity,
        handlers::entity::read_entity_version,
        handlers::entity::list_entity_versions,
        handlers::health::health_check,
    ),
    components(
        schemas(
            models::EntityResponse,
            models::EntityListResponse,
            models::CreateEntityRequest,
            models::UpdateEntityRequest,
            models::HealthResponse,
            error::ApiErrorResponse,
        )
    ),
    tags(
        (name = "entities", description = "Entity CRUD operations"),
        (name = "health", description = "Health check endpoints"),
    ),
    info(
        title = "Marain CMS API",
        version = "1.0.0",
        description = "RESTful API for Marain CMS",
        contact(
            name = "Marain CMS Team",
        ),
    ),
)]
pub struct ApiDoc;

/// Create the main API router with all routes and middleware
pub fn create_router(state: AppState) -> Router {
    // API v1 routes
    let api_v1 = Router::new()
        // Entity CRUD endpoints
        .route(
            "/entity/read/:entity_type/:content_id",
            get(handlers::entity::read_entity),
        )
        .route(
            "/entity/list/:entity_type",
            get(handlers::entity::list_entities),
        )
        .route(
            "/entity/create/:entity_type",
            post(handlers::entity::create_entity),
        )
        .route(
            "/entity/update/:entity_type/:content_id",
            post(handlers::entity::update_entity),
        )
        .route(
            "/entity/delete/:entity_type/:content_id",
            post(handlers::entity::delete_entity),
        )
        // Entity version endpoints
        .route(
            "/entity/version/read/:entity_type/:content_id/:version_id",
            get(handlers::entity::read_entity_version),
        )
        .route(
            "/entity/version/list/:entity_type/:content_id",
            get(handlers::entity::list_entity_versions),
        )
        // Health check
        .route("/health", get(handlers::health::health_check))
        // Apply middleware to all API routes
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middleware_hooks::auth_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middleware_hooks::request_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            middleware_hooks::response_middleware,
        ));

    // Main router
    Router::new()
        .nest("/api/v1", api_v1)
        // Swagger UI - also goes through middleware
        .merge(SwaggerUi::new("/api/v1/swagger").url("/api/v1/openapi.json", ApiDoc::openapi()))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state)
}

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde_json::json;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    models::{
        CreateEntityRequest, DeleteResponse, EntityListResponse, EntityResponse,
        FilterParams, PaginationParams, UpdateEntityRequest,
    },
    AppState,
};

/// Read a single entity by ID
/// 
/// GET /api/v1/entity/read/{entity_type}/{content_id}
#[utoipa::path(
    get,
    path = "/api/v1/entity/read/{entity_type}/{content_id}",
    params(
        ("entity_type" = String, Path, description = "Type of entity (e.g., snippet, all_fields, multi)"),
        ("content_id" = String, Path, description = "Unique identifier of the content")
    ),
    responses(
        (status = 200, description = "Entity retrieved successfully", body = EntityResponse),
        (status = 404, description = "Entity not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "entities"
)]
pub async fn read_entity(
    State(state): State<AppState>,
    Path((entity_type, content_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    info!("Reading entity: type={}, id={}", entity_type, content_id);
    
    // Validate entity type against hot-loaded schemas
    let entity_definitions = schema_manager::get_entity_definitions();
    let entity_exists = entity_definitions.iter()
        .any(|entity| entity.definition().id == entity_type);
    if !entity_exists {
        return Err(ApiError::InvalidEntityType(entity_type));
    }
    // Use EntityStorage to read from database
    use database::storage::EntityStorage;
    
    let storage = EntityStorage::new(&state.db, &entity_type);
    
    match storage.get(&content_id).await {
        Ok(Some(item)) => {
            // Convert the ContentItem to our API response format
            let data = serde_json::to_value(item.fields).unwrap_or(json!({}));
            
            let response = EntityResponse {
                id: item.id,
                entity_type,
                data,
                created_at: item.created_at,
                updated_at: item.updated_at,
            };
            
            Ok(Json(response))
        }
        Ok(None) => {
            Err(ApiError::EntityNotFound(format!("Entity {} with id {} not found", entity_type, content_id)))
        }
        Err(e) => {
            error!("Database error reading entity: {}", e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
    
}

/// List entities of a specific type with pagination
/// 
/// GET /api/v1/entity/list/{entity_type}
#[utoipa::path(
    get,
    path = "/api/v1/entity/list/{entity_type}",
    params(
        ("entity_type" = String, Path, description = "Type of entity to list"),
        ("page" = Option<usize>, Query, description = "Page number (default: 1)"),
        ("page_size" = Option<usize>, Query, description = "Items per page (default: 20)")
    ),
    responses(
        (status = 200, description = "Entities listed successfully", body = EntityListResponse),
        (status = 400, description = "Invalid entity type", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "entities"
)]
pub async fn list_entities(
    State(state): State<AppState>,
    Path(entity_type): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> ApiResult<impl IntoResponse> {
    info!("Listing entities: type={}", entity_type);
    
    // Validate entity type against hot-loaded schemas
    let entity_definitions = schema_manager::get_entity_definitions();
    let entity_exists = entity_definitions.iter()
        .any(|entity| entity.definition().id == entity_type);
    if !entity_exists {
        return Err(ApiError::InvalidEntityType(entity_type));
    }
    
    let page = pagination.page.unwrap_or(1);
    let page_size = pagination.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;
    
    // Use EntityStorage to list from database
    use database::storage::EntityStorage;
    
    let storage = EntityStorage::new(&state.db, &entity_type);
    
    // Get items with pagination (convert usize to i64)
    match storage.list(Some(page_size as i64), Some(offset as i64)).await {
        Ok(items) => {
            // Convert ContentItems to EntityResponses
            let entities: Vec<EntityResponse> = items.into_iter().map(|item| {
                EntityResponse {
                    id: item.id,
                    entity_type: entity_type.clone(),
                    data: serde_json::to_value(item.fields).unwrap_or(json!({})),
                    created_at: item.created_at,
                    updated_at: item.updated_at,
                }
            }).collect();
            
            // Get total count
            let total = entities.len(); // In production, you'd want a separate count query
            
            let response = EntityListResponse {
                entities,
                total,
                page,
                page_size,
            };
            
            Ok(Json(response))
        }
        Err(e) => {
            error!("Database error listing entities: {}", e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
}

/// Create a new entity
/// 
/// POST /api/v1/entity/create/{entity_type}
#[utoipa::path(
    post,
    path = "/api/v1/entity/create/{entity_type}",
    params(
        ("entity_type" = String, Path, description = "Type of entity to create")
    ),
    request_body = CreateEntityRequest,
    responses(
        (status = 201, description = "Entity created successfully", body = EntityResponse),
        (status = 400, description = "Invalid request", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "entities"
)]
pub async fn create_entity(
    State(state): State<AppState>,
    Path(entity_type): Path<String>,
    Json(request): Json<CreateEntityRequest>,
) -> ApiResult<impl IntoResponse> {
    info!("Creating entity: type={}", entity_type);
    
    // Validate entity type against hot-loaded schemas
    let entity_definitions = schema_manager::get_entity_definitions();
    let entity_exists = entity_definitions.iter()
        .any(|entity| entity.definition().id == entity_type);
    if !entity_exists {
        return Err(ApiError::InvalidEntityType(entity_type));
    }
    
    // Use EntityStorage to create in database
    use database::storage::EntityStorage;
    use std::collections::HashMap;
    
    let storage = EntityStorage::new(&state.db, &entity_type);
    
    // Convert JSON data to HashMap for storage
    let fields: HashMap<String, serde_json::Value> = if let serde_json::Value::Object(map) = request.data {
        map.into_iter().collect()
    } else {
        return Err(ApiError::BadRequest("Invalid data format".to_string()));
    };
    
    match storage.create(fields).await {
        Ok(id) => {
            // Fetch the created item to return full data
            match storage.get(&id).await {
                Ok(Some(item)) => {
                    let response = EntityResponse {
                        id: item.id,
                        entity_type,
                        data: serde_json::to_value(item.fields).unwrap_or(json!({})),
                        created_at: item.created_at,
                        updated_at: item.updated_at,
                    };
                    Ok((StatusCode::CREATED, Json(response)))
                }
                Ok(None) => {
                    Err(ApiError::InternalError("Created entity not found".to_string()))
                }
                Err(e) => {
                    error!("Database error fetching created entity: {}", e);
                    Err(ApiError::DatabaseError(e.to_string()))
                }
            }
        }
        Err(e) => {
            error!("Database error creating entity: {}", e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
}

/// Update an existing entity
/// 
/// POST /api/v1/entity/update/{entity_type}/{content_id}
#[utoipa::path(
    post,
    path = "/api/v1/entity/update/{entity_type}/{content_id}",
    params(
        ("entity_type" = String, Path, description = "Type of entity"),
        ("content_id" = String, Path, description = "ID of entity to update")
    ),
    request_body = UpdateEntityRequest,
    responses(
        (status = 200, description = "Entity updated successfully", body = EntityResponse),
        (status = 404, description = "Entity not found", body = ApiErrorResponse),
        (status = 400, description = "Invalid request", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "entities"
)]
pub async fn update_entity(
    State(state): State<AppState>,
    Path((entity_type, content_id)): Path<(String, String)>,
    Json(request): Json<UpdateEntityRequest>,
) -> ApiResult<impl IntoResponse> {
    info!("Updating entity: type={}, id={}", entity_type, content_id);
    
    // Validate entity type against hot-loaded schemas
    let entity_definitions = schema_manager::get_entity_definitions();
    let entity_exists = entity_definitions.iter()
        .any(|entity| entity.definition().id == entity_type);
    if !entity_exists {
        return Err(ApiError::InvalidEntityType(entity_type));
    }
    
    // Use EntityStorage to update in database
    use database::storage::EntityStorage;
    use std::collections::HashMap;
    
    // Check if entity is versioned
    let is_versioned = entity_definitions.iter()
        .find(|entity| entity.definition().id == entity_type)
        .map(|entity| entity.definition().versioned)
        .unwrap_or(false);
    
    let storage = EntityStorage::new_versioned(&state.db, &entity_type, is_versioned);
    
    // Convert JSON data to HashMap for storage
    let fields: HashMap<String, serde_json::Value> = if let serde_json::Value::Object(map) = request.data {
        map.into_iter().collect()
    } else {
        return Err(ApiError::BadRequest("Invalid data format".to_string()));
    };
    
    match storage.update(&content_id, fields).await {
        Ok(_) => {
            // Fetch the updated item to return full data
            match storage.get(&content_id).await {
                Ok(Some(item)) => {
                    let response = EntityResponse {
                        id: item.id,
                        entity_type,
                        data: serde_json::to_value(item.fields).unwrap_or(json!({})),
                        created_at: item.created_at,
                        updated_at: item.updated_at,
                    };
                    Ok(Json(response))
                }
                Ok(None) => {
                    Err(ApiError::EntityNotFound(format!("Entity {} with id {} not found", entity_type, content_id)))
                }
                Err(e) => {
                    error!("Database error fetching updated entity: {}", e);
                    Err(ApiError::DatabaseError(e.to_string()))
                }
            }
        }
        Err(e) => {
            error!("Database error updating entity: {}", e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
}

/// Delete an entity
/// 
/// POST /api/v1/entity/delete/{entity_type}/{content_id}
#[utoipa::path(
    post,
    path = "/api/v1/entity/delete/{entity_type}/{content_id}",
    params(
        ("entity_type" = String, Path, description = "Type of entity"),
        ("content_id" = String, Path, description = "ID of entity to delete")
    ),
    responses(
        (status = 200, description = "Entity deleted successfully", body = DeleteResponse),
        (status = 404, description = "Entity not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "entities"
)]
pub async fn delete_entity(
    State(state): State<AppState>,
    Path((entity_type, content_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    info!("Deleting entity: type={}, id={}", entity_type, content_id);
    
    // Validate entity type against hot-loaded schemas
    let entity_definitions = schema_manager::get_entity_definitions();
    let entity_exists = entity_definitions.iter()
        .any(|entity| entity.definition().id == entity_type);
    if !entity_exists {
        return Err(ApiError::InvalidEntityType(entity_type));
    }
    
    // Use EntityStorage to delete from database
    use database::storage::EntityStorage;
    
    let storage = EntityStorage::new(&state.db, &entity_type);
    
    match storage.delete(&content_id).await {
        Ok(_) => {
            let response = DeleteResponse {
                success: true,
                id: content_id,
                message: format!("{} deleted successfully", entity_type),
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("Database error deleting entity: {}", e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
}

/// Read a specific revision of an entity
///
/// GET /api/v1/entity/version/read/{entity_type}/{content_id}/{version_id}
#[utoipa::path(
    get,
    path = "/api/v1/entity/version/read/{entity_type}/{content_id}/{version_id}",
    params(
        ("entity_type" = String, Path, description = "Type of entity (e.g., snippet, all_fields, multi)"),
        ("content_id" = String, Path, description = "Unique identifier of the content"),
        ("version_id" = i64, Path, description = "Revision ID to retrieve")
    ),
    responses(
        (status = 200, description = "Entity revision retrieved successfully", body = EntityResponse),
        (status = 404, description = "Entity or revision not found", body = ApiErrorResponse),
        (status = 400, description = "Entity is not versioned", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "entities"
)]
pub async fn read_entity_version(
    State(state): State<AppState>,
    Path((entity_type, content_id, version_id)): Path<(String, String, i64)>,
) -> ApiResult<impl IntoResponse> {
    info!("Reading entity revision: type={}, id={}, version={}", entity_type, content_id, version_id);
    
    // Validate entity type and check if it's versioned
    let entity_definitions = schema_manager::get_entity_definitions();
    let entity_def = entity_definitions.iter()
        .find(|entity| entity.definition().id == entity_type);
    
    match entity_def {
        None => return Err(ApiError::InvalidEntityType(entity_type)),
        Some(def) => {
            if !def.definition().versioned {
                return Err(ApiError::BadRequest(format!("Entity type '{}' is not versioned", entity_type)));
            }
        }
    }
    
    // Use EntityStorage with versioning to read from database
    use database::storage::EntityStorage;
    
    let storage = EntityStorage::new_versioned(&state.db, &entity_type, true);
    
    match storage.get_revision(&content_id, version_id).await {
        Ok(Some(item)) => {
            // Convert the ContentItem to our API response format
            let data = serde_json::to_value(item.fields).unwrap_or(json!({}));
            
            let response = EntityResponse {
                id: item.id,
                entity_type,
                data,
                created_at: item.created_at,
                updated_at: item.updated_at,
            };
            
            Ok(Json(response))
        }
        Ok(None) => {
            Err(ApiError::EntityNotFound(format!("Entity {} with id {} and version {} not found", entity_type, content_id, version_id)))
        }
        Err(e) => {
            error!("Database error reading entity revision: {}", e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
}

/// List all revisions for an entity
///
/// GET /api/v1/entity/version/list/{entity_type}/{content_id}
#[utoipa::path(
    get,
    path = "/api/v1/entity/version/list/{entity_type}/{content_id}",
    params(
        ("entity_type" = String, Path, description = "Type of entity"),
        ("content_id" = String, Path, description = "Unique identifier of the content")
    ),
    responses(
        (status = 200, description = "Revisions listed successfully", body = Vec<i64>),
        (status = 404, description = "Entity not found", body = ApiErrorResponse),
        (status = 400, description = "Entity is not versioned", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    ),
    tag = "entities"
)]
pub async fn list_entity_versions(
    State(state): State<AppState>,
    Path((entity_type, content_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    info!("Listing entity revisions: type={}, id={}", entity_type, content_id);
    
    // Validate entity type and check if it's versioned
    let entity_definitions = schema_manager::get_entity_definitions();
    let entity_def = entity_definitions.iter()
        .find(|entity| entity.definition().id == entity_type);
    
    match entity_def {
        None => return Err(ApiError::InvalidEntityType(entity_type)),
        Some(def) => {
            if !def.definition().versioned {
                return Err(ApiError::BadRequest(format!("Entity type '{}' is not versioned", entity_type)));
            }
        }
    }
    
    // Use EntityStorage with versioning to list revisions
    use database::storage::EntityStorage;
    
    let storage = EntityStorage::new_versioned(&state.db, &entity_type, true);
    
    match storage.list_revisions(&content_id).await {
        Ok(revisions) => {
            Ok(Json(revisions))
        }
        Err(e) => {
            error!("Database error listing entity revisions: {}", e);
            Err(ApiError::DatabaseError(e.to_string()))
        }
    }
}
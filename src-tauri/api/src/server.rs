use crate::{create_router, AppState};
use std::sync::Arc;
use tracing::{info, warn};
use tokio::task::JoinHandle;

/// API server configuration
pub struct ApiConfig {
    /// Port to listen on
    pub port: u16,
    /// Whether to initialize test data (development only)
    pub init_test_data: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            port: 3030,
            #[cfg(debug_assertions)]
            init_test_data: true,
            #[cfg(not(debug_assertions))]
            init_test_data: false,
        }
    }
}

impl ApiConfig {
    /// Create a new API configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    /// Set whether to initialize test data
    pub fn with_test_data(mut self, init: bool) -> Self {
        self.init_test_data = init;
        self
    }
}

/// Start the API server with the given configuration
pub async fn start_server_with_config(
    db: Arc<database::Database>,
    config: ApiConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize test data if requested
    #[cfg(debug_assertions)]
    if config.init_test_data {
        info!("Initializing test data for development");
        if let Err(e) = crate::test_data::init_test_data(&db).await {
            warn!("Failed to initialize test data: {}", e);
        }
    }
    
    let state = AppState { db };
    let app = create_router(state);
    
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("API server listening on {}", addr);
    info!("Swagger UI available at http://localhost:{}/api/v1/swagger", config.port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Start the API server with default configuration
pub async fn start_server(db: Arc<database::Database>) -> Result<(), Box<dyn std::error::Error>> {
    let config = ApiConfig::default();
    start_server_with_config(db, config).await
}

/// Start the API server in a background task
pub fn spawn_server(db: Arc<database::Database>) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(e) = start_server(db).await {
            tracing::error!("API server error: {}", e);
        }
    })
}

/// Start the API server in a background task with custom configuration
pub fn spawn_server_with_config(db: Arc<database::Database>, config: ApiConfig) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(e) = start_server_with_config(db, config).await {
            tracing::error!("API server error: {}", e);
        }
    })
}
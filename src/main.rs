use anyhow::Result;
use reprime_backend::{
    config::Config,
    handlers::Handlers,
    middleware::{cors_layer, logging_layer},
    repositories::Repositories,
    routes::create_routes,
    services::Services,
    utils::{create_database_pool, init_tracing},
};
use std::sync::Arc;
use tokio::net::TcpListener;


#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::new().unwrap_or_else(|_| {
        eprintln!("Failed to load configuration, using defaults");
        Config::default()
    });

    // Initialize tracing
    init_tracing(&config);

    tracing::info!("Starting reprime-backend server...");
    tracing::info!("Configuration loaded: {:?}", config);

    // Create database connection pool
    let pool = create_database_pool(&config).await?;

    // Initialize layers
    let repositories = Arc::new(Repositories::new(pool));
    let services = Arc::new(Services::new(repositories));
    let handlers = Handlers::new(services);

    // Create router with middleware
    let app = create_routes(handlers)
        .layer(cors_layer())
        .layer(logging_layer());

    // Start server
    let listener = TcpListener::bind(&config.server_address()).await?;
    tracing::info!("Server listening on {}", config.server_address());

    axum::serve(listener, app).await?;

    Ok(())
}

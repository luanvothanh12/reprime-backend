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
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;


#[derive(OpenApi)]
#[allow(dead_code)]
#[openapi(
    paths(
        reprime_backend::handlers::health::health_check,
        reprime_backend::handlers::user::create_user,
        reprime_backend::handlers::user::get_users,
        reprime_backend::handlers::user::get_user,
        reprime_backend::handlers::user::update_user,
        reprime_backend::handlers::user::delete_user,
    ),
    components(
        schemas(
            reprime_backend::models::User,
            reprime_backend::models::UserResponse,
            reprime_backend::models::CreateUserRequest,
            reprime_backend::models::UpdateUserRequest,
            reprime_backend::models::ApiResponse<reprime_backend::models::UserResponse>,
            reprime_backend::models::PaginatedResponse<reprime_backend::models::UserResponse>,
            reprime_backend::models::PaginationParams,
            reprime_backend::models::DeleteResponse,
            reprime_backend::handlers::HealthResponse,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "users", description = "User management endpoints")
    ),
    info(
        title = "Reprime Backend API",
        version = "0.1.0",
        description = "A modern Rust backend API with OpenAPI documentation",
        contact(
            name = "API Support",
            email = "support@reprime.com"
        )
    )
)]
struct ApiDoc;

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

    // Create a database connection pool
    let pool = create_database_pool(&config).await?;

    // Initialize layers
    let repositories = Arc::new(Repositories::new(pool));
    let services = Arc::new(Services::new(repositories));
    let handlers = Handlers::new(services);

    // Create OpenAPI documentation
    let openapi = ApiDoc::openapi();

    let app = create_routes(handlers)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .layer(cors_layer())
        .layer(logging_layer());

    // Start server
    let listener = TcpListener::bind(&config.server_address()).await?;
    tracing::info!("Server listening on {}", config.server_address());
    tracing::info!("Swagger UI available at http://{}/swagger-ui/", config.server_address());

    axum::serve(listener, app).await?;

    Ok(())
}

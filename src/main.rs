use anyhow::Result;
use reprime_backend::{
    auth::{jwt::JwtService, openfga::OpenFgaService},
    config::Config,
    handlers::{Handlers, metrics::metrics_handler},
    middleware::{cors_layer, logging_layer, prometheus::prometheus_middleware},
    repositories::Repositories,
    routes::create_routes,
    services::Services,
    utils::create_database_pool,
    metrics::AppMetrics,
    database::InstrumentedDatabase,
};
use std::{sync::Arc, time::Duration};
use tokio::{net::TcpListener, time::interval};
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
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
        reprime_backend::auth::handlers::register,
        reprime_backend::auth::handlers::login,
        reprime_backend::auth::handlers::logout,
        reprime_backend::auth::handlers::me,
        reprime_backend::auth::handlers::refresh_token,
        reprime_backend::auth::handlers::check_permission,
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
            reprime_backend::auth::models::LoginRequest,
            reprime_backend::auth::models::LoginResponse,
            reprime_backend::auth::models::RegisterRequest,
            reprime_backend::auth::models::UserInfo,
            reprime_backend::auth::models::PermissionCheck,
            reprime_backend::models::ApiResponse<reprime_backend::auth::models::LoginResponse>,
            reprime_backend::models::ApiResponse<reprime_backend::auth::models::UserInfo>,
            reprime_backend::models::ApiResponse<bool>,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "authentication", description = "Authentication and authorization endpoints"),
    ),
    info(
        title = "Reprime Backend API",
        version = "0.1.0",
        description = "A modern Rust backend API with OpenAPI documentation and ABAC authorization using OpenFGA",
        contact(
            name = "API Support",
            email = "support@reprime.com"
        )
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::new().unwrap_or_else(|_| {
        eprintln!("Failed to load configuration, using defaults");
        Config::default()
    });

    // Initialize comprehensive telemetry with OpenTelemetry, Loki, and structured logging
    reprime_backend::telemetry::init_telemetry_with_loki(&config).await?;

    tracing::info!("Starting reprime-backend server...");
    tracing::info!("Configuration loaded: {:?}", config);

    // Create a database connection pool
    let pool = create_database_pool(&config).await?;

    // Initialize custom metrics
    let metrics = AppMetrics::new().expect("Failed to create metrics");

    // Create instrumented database
    let instrumented_db = Arc::new(InstrumentedDatabase::new((*pool).clone(), Some(metrics.clone())));

    // Initialize auth services
    let jwt_service = Arc::new(JwtService::new(&config));
    let openfga_service = Arc::new(OpenFgaService::new(&config).await?);

    // Initialize layers
    let repositories = Arc::new(Repositories::new(instrumented_db.clone()));
    let services = Arc::new(Services::new(
        repositories,
        jwt_service.clone(),
        openfga_service.clone(),
    ));

    let handlers = Handlers::new(services, jwt_service.clone(), openfga_service);

    // Create OpenAPI documentation
    let openapi = ApiDoc::openapi();

    // Start background task for connection pool monitoring
    let instrumented_db_clone = instrumented_db.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            instrumented_db_clone.get_pool_metrics();
        }
    });

    let metrics_router = axum::Router::new()
        .route("/metrics", axum::routing::get(metrics_handler))
        .with_state(metrics.clone());

    let app = create_routes(handlers, jwt_service)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi))
        .merge(metrics_router)
        .layer(axum::middleware::from_fn_with_state(metrics.clone(), prometheus_middleware))
        .layer(cors_layer())
        .layer(logging_layer());

    // Start server
    let listener = TcpListener::bind(&config.server_address()).await?;

    tracing::info!(
        address = %config.server_address(),
        swagger_ui = %format!("http://{}/swagger-ui/", config.server_address()),
        metrics = %format!("http://{}/metrics", config.server_address()),
        "Server started successfully"
    );

    // Set up graceful shutdown
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        tracing::info!("Shutdown signal received, starting graceful shutdown...");
        reprime_backend::telemetry::shutdown_telemetry();
    };

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

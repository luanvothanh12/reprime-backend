use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use reprime_backend::{
    config::Config, handlers::Handlers, repositories::Repositories,
    routes::create_routes, services::Services, utils::create_database_pool,
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder().uri("/health").body(Body::empty()).unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body =
        axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body["status"], "ok");
    assert!(body["timestamp"].is_string());
    assert_eq!(body["service"], "reprime-backend");
}

async fn create_test_app() -> axum::Router {
    let config = Config::default();

    // For testing, you might want to use an in-memory database or test database
    // This is a simplified version - in real tests, you'd set up a test database
    let pool = create_database_pool(&config).await.unwrap();
    let repositories = Arc::new(Repositories::new(pool));
    let services = Arc::new(Services::new(repositories));
    let handlers = Handlers::new(services);

    create_routes(handlers)
}

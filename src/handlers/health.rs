use axum::{response::Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct HealthResponse {
    #[schema(example = "ok")]
    pub status: String,
    pub timestamp: String,
    #[schema(example = "reprime-backend")]
    pub service: String,
    #[schema(example = "0.1.0")]
    pub version: String,
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_check() -> Result<Json<HealthResponse>, StatusCode> {
    let health_response = HealthResponse {
        status: "ok".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        service: "reprime-backend".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Ok(Json(health_response))
}

use axum::{http::StatusCode, response::Json};
use serde_json::{json, Value};

pub async fn health_check() -> Result<(StatusCode, Json<Value>), StatusCode> {
    let health_check = json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "reprime-backend",
        "version": env!("CARGO_PKG_VERSION")
    });

    Ok((StatusCode::OK, Json(health_check)))
}

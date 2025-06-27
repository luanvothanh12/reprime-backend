use crate::metrics::AppMetrics;
use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use prometheus::{Encoder, TextEncoder};

/// Metrics endpoint for Prometheus scraping
pub async fn metrics_handler(State(metrics): State<AppMetrics>) -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = metrics.registry.gather();
    
    match encoder.encode_to_string(&metric_families) {
        Ok(output) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, encoder.format_type())
            .body(output)
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Failed to encode metrics".to_string())
            .unwrap(),
    }
}

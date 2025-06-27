use crate::metrics::AppMetrics;
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;

/// Middleware to collect Prometheus metrics for HTTP requests
pub async fn prometheus_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Get metrics from request extensions (will be set in main.rs)
    let metrics = request
        .extensions()
        .get::<AppMetrics>()
        .cloned()
        .unwrap_or_default();

    // Increment in-flight requests
    metrics.http_requests_in_flight
        .with_label_values(&[&method, &path])
        .inc();

    // Process the request
    let response = next.run(request).await;

    // Decrement in-flight requests
    metrics.http_requests_in_flight
        .with_label_values(&[&method, &path])
        .dec();

    // Record metrics
    let duration = start.elapsed().as_secs_f64();
    let status_code = response.status().as_u16();

    metrics.record_http_request(&method, &path, status_code, duration);

    response
}



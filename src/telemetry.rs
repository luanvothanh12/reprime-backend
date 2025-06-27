use anyhow::Result;
use std::collections::HashMap;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};
use uuid::Uuid;
use crate::config::Config;

/// Initialize comprehensive telemetry with Loki and structured logging
pub async fn init_telemetry_with_loki(config: &Config) -> Result<()> {
    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.logging.level));

    // Try to create Loki layer
    let loki_url = std::env::var("LOKI_URL").unwrap_or_else(|_| "http://localhost:3100".to_string());

    let environment = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string());
    let region = std::env::var("REGION").unwrap_or_else(|_| "local".to_string());
    let instance_id = std::env::var("INSTANCE_ID").unwrap_or_else(|_| {
        format!("reprime-{}", uuid::Uuid::new_v4().to_string()[..8].to_string())
    });

    match tracing_loki::builder()
        .label("service", "reprime-backend")?
        .label("version", env!("CARGO_PKG_VERSION"))?
        .label("environment", &environment)?
        .label("region", &region)?
        .label("instance", &instance_id)?
        .build_url(loki_url.parse()?)
    {
        Ok((loki_layer, task)) => {
            // Spawn the background task for Loki
            tokio::spawn(task);

            // Create structured JSON formatter with trace correlation
            let fmt_layer = fmt::layer()
                .json()
                .flatten_event(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_current_span(true)
                .with_span_list(true);

            // Initialize subscriber with all layers
            Registry::default()
                .with(env_filter)
                .with(fmt_layer)
                .with(loki_layer)
                .init();

            tracing::info!(
                loki_url = %loki_url,
                service = "reprime-backend",
                version = env!("CARGO_PKG_VERSION"),
                "Telemetry initialized with Loki and structured logging"
            );
        }
        Err(e) => {
            // Fall back to console only
            tracing::warn!("Failed to initialize Loki layer: {}. Using console logging only.", e);

            let fmt_layer = fmt::layer()
                .json()
                .flatten_event(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_current_span(true)
                .with_span_list(true);

            Registry::default()
                .with(env_filter)
                .with(fmt_layer)
                .init();

            tracing::info!("Telemetry initialized with console logging");
        }
    }

    Ok(())
}



/// Shutdown telemetry gracefully
pub fn shutdown_telemetry() {
    tracing::info!("Shutting down telemetry...");
    // TODO: Add OpenTelemetry shutdown when implemented
    tracing::info!("Telemetry shutdown complete");
}

thread_local! {
    static TRACE_CONTEXT: std::cell::RefCell<HashMap<String, String>> = std::cell::RefCell::new(HashMap::new());
}

/// Generate a new trace ID
pub fn generate_trace_id() -> String {
    Uuid::new_v4().to_string().replace("-", "")[..16].to_string()
}

/// Generate a new span ID
pub fn generate_span_id() -> String {
    Uuid::new_v4().to_string().replace("-", "")[..8].to_string()
}

/// Set the current trace context
pub fn set_trace_context(trace_id: String, span_id: String) {
    TRACE_CONTEXT.with(|ctx| {
        let mut context = ctx.borrow_mut();
        context.insert("trace_id".to_string(), trace_id);
        context.insert("span_id".to_string(), span_id);
    });
}

/// Helper to get current trace ID as string for correlation
pub fn current_trace_id() -> Option<String> {
    TRACE_CONTEXT.with(|ctx| {
        ctx.borrow().get("trace_id").cloned()
    })
}

/// Helper to get current span ID as string for correlation
pub fn current_span_id() -> Option<String> {
    TRACE_CONTEXT.with(|ctx| {
        ctx.borrow().get("span_id").cloned()
    })
}

/// Initialize a new trace for the current request
pub fn init_request_trace() -> (String, String) {
    let trace_id = generate_trace_id();
    let span_id = generate_span_id();
    set_trace_context(trace_id.clone(), span_id.clone());
    (trace_id, span_id)
}

/// Create a child span within the current trace
pub fn create_child_span() -> String {
    let span_id = generate_span_id();
    TRACE_CONTEXT.with(|ctx| {
        let mut context = ctx.borrow_mut();
        context.insert("span_id".to_string(), span_id.clone());
    });
    span_id
}

/// Enhanced macro for structured logging with automatic trace correlation
#[macro_export]
macro_rules! log_with_trace {
    ($level:ident, $($arg:tt)*) => {
        {
            let trace_id = $crate::telemetry::current_trace_id()
                .unwrap_or_else(|| "no-trace".to_string());
            let span_id = $crate::telemetry::current_span_id()
                .unwrap_or_else(|| "no-span".to_string());
            let environment = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".to_string());

            tracing::$level!(
                trace_id = %trace_id,
                span_id = %span_id,
                service = "reprime-backend",
                version = env!("CARGO_PKG_VERSION"),
                environment = %environment,
                $($arg)*
            );
        }
    };
}



/// Helper struct for timing operations with trace correlation
pub struct TracedTimer {
    start: std::time::Instant,
    operation: String,
}

impl TracedTimer {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            start: std::time::Instant::now(),
            operation: operation.into(),
        }
    }
    
    pub fn finish(self) -> f64 {
        let duration = self.start.elapsed().as_secs_f64();
        let duration_ms = self.start.elapsed().as_millis() as f64;

        log_with_trace!(info,
            operation = %self.operation,
            duration_seconds = duration,
            duration_ms = duration_ms,
            "Operation completed"
        );

        duration
    }
}

/// Middleware helper for HTTP request tracing
pub fn extract_or_generate_trace_id(headers: &axum::http::HeaderMap) -> (String, String) {
    // Try to extract trace ID from headers (for distributed tracing)
    let trace_id = headers
        .get("x-trace-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| generate_trace_id());

    let span_id = generate_span_id();
    set_trace_context(trace_id.clone(), span_id.clone());

    (trace_id, span_id)
}

/// Helper to add trace headers to HTTP responses
pub fn add_trace_headers(
    mut response: axum::response::Response,
    trace_id: &str,
    span_id: &str,
) -> axum::response::Response {
    let headers = response.headers_mut();

    if let Ok(trace_header) = axum::http::HeaderValue::from_str(trace_id) {
        headers.insert("x-trace-id", trace_header);
    }

    if let Ok(span_header) = axum::http::HeaderValue::from_str(span_id) {
        headers.insert("x-span-id", span_header);
    }

    response
}


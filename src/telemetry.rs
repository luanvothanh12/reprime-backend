use anyhow::Result;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};
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

/// Helper to get current trace ID as string for correlation
pub fn current_trace_id() -> Option<String> {
    // For now, return None - this will be populated when OpenTelemetry is properly configured
    // In a real implementation, this would extract the trace_id from the current span context
    None
}

/// Helper to get current span ID as string for correlation
pub fn current_span_id() -> Option<String> {
    // For now, return None - this will be populated when OpenTelemetry is properly configured
    // In a real implementation, this would extract the span_id from the current span context
    None
}

/// Macro for structured logging with automatic trace correlation
#[macro_export]
macro_rules! log_with_trace {
    ($level:ident, $($field:ident = $value:expr),* $(,)? ; $($arg:tt)*) => {
        {
            let trace_id = $crate::telemetry::current_trace_id();
            let span_id = $crate::telemetry::current_span_id();
            
            tracing::$level!(
                trace_id = ?trace_id,
                span_id = ?span_id,
                $($field = $value,)*
                $($arg)*
            );
        }
    };
    ($level:ident, $($arg:tt)*) => {
        {
            let trace_id = $crate::telemetry::current_trace_id();
            let span_id = $crate::telemetry::current_span_id();
            
            tracing::$level!(
                trace_id = ?trace_id,
                span_id = ?span_id,
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
        
        log_with_trace!(info,
            operation = %self.operation,
            duration_seconds = duration,
            "Operation completed"
        );
        
        duration
    }
}

use crate::config::Config;
use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub fn init_tracing(config: &Config) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.logging.level));

    match config.logging.format.as_str() {
        "json" => {
            Registry::default()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().json().flatten_event(true))
                .init();
        }
        _ => {
            Registry::default()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().pretty())
                .init();
        }
    }

    tracing::info!("Tracing initialized with level: {}", config.logging.level);
}

pub async fn init_tracing_with_loki(config: &Config) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.logging.level));

    // Try to create Loki layer, fall back to console if Loki is not available
    let loki_url = std::env::var("LOKI_URL").unwrap_or_else(|_| "http://localhost:3100".to_string());

    match tracing_loki::builder()
        .label("service", "reprime-backend")?
        .label("version", env!("CARGO_PKG_VERSION"))?
        .build_url(loki_url.parse()?)
    {
        Ok((loki_layer, task)) => {
            // Spawn the background task
            tokio::spawn(task);

            // Initialize with both console and Loki layers
            Registry::default()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().json().flatten_event(true))
                .with(loki_layer)
                .init();

            tracing::info!("Tracing initialized with Loki integration at: {}", loki_url);
        }
        Err(e) => {
            // Fall back to console-only logging
            tracing::warn!("Failed to initialize Loki layer: {}. Falling back to console logging.", e);
            init_tracing(config);
        }
    }

    Ok(())
}

use crate::config::Config;
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

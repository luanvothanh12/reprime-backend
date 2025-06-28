use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub telemetry: TelemetryConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelemetryConfig {
    pub otlp_endpoint: String,
    pub loki_endpoint: String,
    pub service_name: String,
    pub enable_tracing: bool,
    pub enable_metrics: bool,
    pub enable_logging: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub openfga: OpenFgaConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OpenFgaConfig {
    pub endpoint: String,
    pub store_id: String,
    pub auth_model_id: Option<String>,
    pub api_token: Option<String>,
    pub cache_enabled: bool,
    pub cache_ttl_seconds: u64,
    pub cache_max_entries: usize,
    pub request_timeout_seconds: u64,
}

impl Config {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let config = ConfigBuilder::builder()
            // Start with default configuration
            .add_source(File::with_name("config/default"))
            // Add environment-specific configuration
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add local configuration (for development)
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables with prefix "APP"
            .add_source(Environment::with_prefix("APP").separator("_"))
            .build()?;

        config.try_deserialize()
    }

    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/reprime_backend".to_string(),
                max_connections: 10,
                min_connections: 1,
                acquire_timeout: 30,
                idle_timeout: 600,
                max_lifetime: 1800,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
            telemetry: TelemetryConfig {
                otlp_endpoint: "http://localhost:4317".to_string(),
                loki_endpoint: "http://localhost:3100".to_string(),
                service_name: "reprime-backend".to_string(),
                enable_tracing: true,
                enable_metrics: true,
                enable_logging: true,
            },
            auth: AuthConfig {
                jwt_secret: "your-secret-key-change-in-production".to_string(),
                jwt_expiration_hours: 24,
                openfga: OpenFgaConfig {
                    endpoint: "http://localhost:8080".to_string(),
                    store_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
                    auth_model_id: None,
                    api_token: None,
                    cache_enabled: true,
                    cache_ttl_seconds: 300,
                    cache_max_entries: 50000,
                    request_timeout_seconds: 30,
                },
            },
        }
    }
}

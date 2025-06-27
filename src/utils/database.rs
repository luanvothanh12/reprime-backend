use crate::config::Config;
use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{sync::Arc, time::Duration};

pub async fn create_database_pool(config: &Config) -> Result<Arc<PgPool>> {
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .acquire_timeout(Duration::from_secs(config.database.acquire_timeout))
        .idle_timeout(Duration::from_secs(config.database.idle_timeout))
        .max_lifetime(Duration::from_secs(config.database.max_lifetime))
        .connect(&config.database.url)
        .await?;

    // Run migrations if available
    sqlx::migrate!("./migrations").run(&pool).await?;

    tracing::info!("Database connection pool created successfully");

    Ok(Arc::new(pool))
}

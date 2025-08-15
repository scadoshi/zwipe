use anyhow::{anyhow, Error as AnyhowError};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tracing::info;

pub mod card;

pub async fn new_pool(database_url: &str) -> Result<PgPool, AnyhowError> {
    info!("Generating database connection pool");
    PgPoolOptions::new()
        .min_connections(2)
        .max_connections(10)
        .idle_timeout(Some(Duration::from_secs(300)))
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
        .map_err(|e| anyhow!("Failed to build connection. Error: {:?}", e))
}

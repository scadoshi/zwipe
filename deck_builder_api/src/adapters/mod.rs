pub mod database;
pub mod external;
pub mod http;

use crate::adapters::database::postgres::Postgres;
use crate::domain::auth::jwt::JwtConfig;
use anyhow::Context;
use sqlx::PgPool;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_config: JwtConfig,
}

impl AppState {
    pub async fn initialize() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL environment variable must be set")?;
        info!("Extracted database URL from environment");

        let postgresql = Postgres::new(&database_url).await?;
        let jwt_config = JwtConfig::from_env()?;
        Ok(Self {
            db_pool: postgresql.pool,
            jwt_config,
        })
    }
}

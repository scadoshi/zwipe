pub mod auth;
pub mod database;
pub mod external;
pub mod http;

use crate::adapters::auth::jwt::JwtConfig;
use crate::adapters::database::new_pool;
use anyhow::anyhow;
use sqlx::PgPool;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_config: JwtConfig,
}

impl AppState {
    pub async fn initialize() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL").map_err(|e| {
            anyhow!(
                "DATABASE_URL environment variable must be set. Error: {:?}",
                e
            )
        })?;
        info!("Extracted database URL from environment");

        let db_pool = new_pool(&database_url).await?;
        let jwt_config = JwtConfig::from_env()?;
        Ok(Self {
            db_pool,
            jwt_config,
        })
    }
}

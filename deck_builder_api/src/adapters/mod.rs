pub mod auth;
pub mod database;
pub mod external;

use crate::adapters::auth::jwt::JwtConfig;
use anyhow;
use sqlx::PgPool;

#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
    jwt_config: JwtConfig,
}

impl AppState {
    async fn initialize() -> Result<Self, anyhow::Error> {
        let database_url = std::env::var("DATABASE_URL").map_err(|e| {
            anyhow!(
                "DATABASE_URL environment variable must be set. Error: {:?}",
                e
            )
        })?;
        info!("Extracted database URL from environment");

        Ok(Self {
            db_pool: database::new_pool()?,
            jwt_config: JwtConfig::from_env()?,
        })
    }
}

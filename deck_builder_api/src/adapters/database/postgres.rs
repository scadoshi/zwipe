use anyhow::Context;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Postgres {
    pub pool: sqlx::PgPool,
}

impl Postgres {
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        info!("Generating connection pool to database");

        let pool = crate::config::PostgresPoolOptions::new()
            .connect(path)
            .await
            .with_context(|| format!("Failed to open database with at {}", path))?;

        Ok(Self { pool })
    }
}

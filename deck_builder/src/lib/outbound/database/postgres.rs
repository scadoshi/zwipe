use anyhow::Context;
use tracing::info;

pub struct PostgresPoolOptions(sqlx::postgres::PgPoolOptions);

impl PostgresPoolOptions {
    pub fn new() -> Self {
        Self(
            sqlx::postgres::PgPoolOptions::new()
                .min_connections(2)
                .max_connections(10)
                .idle_timeout(Some(std::time::Duration::from_secs(300)))
                .acquire_timeout(std::time::Duration::from_secs(5)),
        )
    }

    pub async fn connect(self, path: &str) -> Result<sqlx::postgres::PgPool, sqlx::Error> {
        self.0.connect(path).await
    }
}

#[derive(Debug, Clone)]
pub struct Postgres {
    pub pool: sqlx::PgPool,
}

impl Postgres {
    pub async fn new(path: &str) -> anyhow::Result<Self> {
        info!("Generating connection pool to database");

        let pool = PostgresPoolOptions::new()
            .connect(path)
            .await
            .with_context(|| format!("Failed to open database with at {}", path))?;

        Ok(Self { pool })
    }
}

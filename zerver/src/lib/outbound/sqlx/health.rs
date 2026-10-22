//! Health check repository implementation (`SELECT 1` connectivity test).

use anyhow::anyhow;
use sqlx::query;

use crate::domain::health::models::HealthCheckFailed;
use crate::domain::health::ports::HealthRepository;
use crate::outbound::sqlx::postgres::Postgres;

impl HealthRepository for Postgres {
    async fn check_database(&self) -> Result<(), HealthCheckFailed> {
        query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| HealthCheckFailed(anyhow!("{e}")))?;

        Ok(())
    }
}

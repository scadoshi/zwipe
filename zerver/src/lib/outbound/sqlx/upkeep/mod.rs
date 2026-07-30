//! PostgreSQL adapter for upkeep (nightly maintenance) deletes.
//!
//! Runs under the scoped `zervice` role in production — every statement here
//! must be covered by `zcripts/server/sql/zervice_role.sql`. Uses runtime
//! queries (not the `query!` macro) deliberately: the session prune's grant is
//! destruction-only (`DELETE` + column-scoped `SELECT (expires_at)`), and
//! these are three trivial deletes.

use crate::{domain::upkeep::ports::UpkeepRepository, outbound::sqlx::postgres::Postgres};
use anyhow::Context;

impl UpkeepRepository for Postgres {
    async fn prune_client_errors(&self, retention_days: i32) -> anyhow::Result<u64> {
        let result = sqlx::query(
            "DELETE FROM client_errors WHERE received_at < NOW() - make_interval(days => $1)",
        )
        .bind(retention_days)
        .execute(&self.pool)
        .await
        .context("pruning client_errors")?;
        Ok(result.rows_affected())
    }

    async fn prune_crash_reports(&self, retention_days: i32) -> anyhow::Result<u64> {
        let result = sqlx::query(
            "DELETE FROM crash_reports WHERE received_at < NOW() - make_interval(days => $1)",
        )
        .bind(retention_days)
        .execute(&self.pool)
        .await
        .context("pruning crash_reports")?;
        Ok(result.rows_affected())
    }

    async fn prune_expired_sessions(&self) -> anyhow::Result<u64> {
        let result = sqlx::query("DELETE FROM refresh_tokens WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await
            .context("pruning expired refresh tokens")?;
        Ok(result.rows_affected())
    }
}

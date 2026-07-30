//! Port traits for nightly upkeep (maintenance) operations.
//!
//! Constructed ONLY by the zervice bin — zerver's HTTP wiring never straps
//! this on. The scoped `zervice` Postgres role's grants
//! (`zcripts/server/sql/zervice_role.sql`) define exactly what upkeep may
//! touch; a method added here needs a matching grant there.

use std::future::Future;

/// Database port for upkeep deletes.
pub trait UpkeepRepository: Clone + Send + Sync + 'static {
    /// Deletes `client_errors` rows older than the retention window.
    /// Returns the number of rows removed.
    fn prune_client_errors(
        &self,
        retention_days: i32,
    ) -> impl Future<Output = anyhow::Result<u64>> + Send;

    /// Deletes `crash_reports` rows older than the retention window.
    /// Returns the number of rows removed.
    fn prune_crash_reports(
        &self,
        retention_days: i32,
    ) -> impl Future<Output = anyhow::Result<u64>> + Send;

    /// Deletes expired refresh tokens across all users — the final dusting
    /// for dormant users the insert-time drive-by never revisits. Returns the
    /// number of rows removed. (The grant behind this is deliberately
    /// destruction-only: `DELETE` + column-scoped `SELECT (expires_at)`.)
    fn prune_expired_sessions(&self) -> impl Future<Output = anyhow::Result<u64>> + Send;
}

/// Service port for the nightly upkeep step.
pub trait UpkeepService: Clone + Send + Sync + 'static {
    /// Prunes `client_errors` past the standard retention window.
    fn prune_client_errors(&self) -> impl Future<Output = anyhow::Result<u64>> + Send;

    /// Prunes `crash_reports` past the standard retention window.
    fn prune_crash_reports(&self) -> impl Future<Output = anyhow::Result<u64>> + Send;

    /// Prunes expired refresh tokens across all users.
    fn prune_expired_sessions(&self) -> impl Future<Output = anyhow::Result<u64>> + Send;
}

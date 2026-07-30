//! Upkeep service — the nightly maintenance step's business logic.

use crate::domain::upkeep::ports::{UpkeepRepository, UpkeepService};

/// Days of diagnostic data (`client_errors`, `crash_reports`) kept before the
/// nightly prune removes them.
pub const DIAGNOSTIC_RETENTION_DAYS: i32 = 90;

/// Canonical [`UpkeepService`] implementation.
#[derive(Debug, Clone)]
pub struct Service<R: UpkeepRepository> {
    repo: R,
}

impl<R: UpkeepRepository> Service<R> {
    /// Creates the service over the given repository.
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: UpkeepRepository> UpkeepService for Service<R> {
    async fn prune_client_errors(&self) -> anyhow::Result<u64> {
        self.repo
            .prune_client_errors(DIAGNOSTIC_RETENTION_DAYS)
            .await
    }

    async fn prune_crash_reports(&self) -> anyhow::Result<u64> {
        self.repo
            .prune_crash_reports(DIAGNOSTIC_RETENTION_DAYS)
            .await
    }

    async fn prune_expired_sessions(&self) -> anyhow::Result<u64> {
        self.repo.prune_expired_sessions().await
    }
}

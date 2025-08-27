// internal
use crate::domain::health::ports::{HealthRepository, HealthService};

/// structure which implements `HealthService`
#[derive(Debug, Clone)]
pub struct Service<R: HealthRepository> {
    repo: R,
}

impl<R: HealthRepository> Service<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: HealthRepository> HealthService for Service<R> {
    async fn check_database(&self) -> Result<(), super::models::HealthCheckFailed> {
        self.repo.check_database().await
    }
}

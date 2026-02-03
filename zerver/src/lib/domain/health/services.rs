use crate::domain::health::ports::{HealthRepository, HealthService};

/// Health check service implementation for monitoring system status.
///
/// Provides health check endpoints for:
/// - **Database connectivity**: Verifies PostgreSQL connection is alive
/// - **API liveness**: Basic health check for load balancers/orchestrators
///
/// Used by Kubernetes/Docker health probes and monitoring systems.
#[derive(Debug, Clone)]
pub struct Service<R: HealthRepository> {
    repo: R,
}

impl<R: HealthRepository> Service<R> {
    /// Creates a new health service with the provided repository.
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: HealthRepository> HealthService for Service<R> {
    async fn check_database(&self) -> Result<(), super::models::HealthCheckFailed> {
        self.repo.check_database().await
    }
}

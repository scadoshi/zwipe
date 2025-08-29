use crate::domain::health::models::HealthCheckFailed;
use std::future::Future;

/// enables health check related database operations
pub trait HealthRepository: Clone + Send + Sync + 'static {
    fn check_database(&self) -> impl Future<Output = Result<(), HealthCheckFailed>> + Send;
}

/// orchestrates health check related operations
pub trait HealthService: Clone + Send + Sync + 'static {
    fn check_database(&self) -> impl Future<Output = Result<(), HealthCheckFailed>> + Send;
}

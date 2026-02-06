//! Port traits for health check operations.
//!
//! This module defines the interfaces (ports) for service health monitoring.
//! Health checks verify the service is operational and can communicate with dependencies.

use crate::domain::health::models::HealthCheckFailed;
use std::future::Future;

/// Database port for health check operations.
pub trait HealthRepository: Clone + Send + Sync + 'static {
    /// Checks database connectivity and basic query operations.
    fn check_database(&self) -> impl Future<Output = Result<(), HealthCheckFailed>> + Send;
}

/// Service port for health check business logic.
pub trait HealthService: Clone + Send + Sync + 'static {
    /// Checks all service dependencies.
    fn check_database(&self) -> impl Future<Output = Result<(), HealthCheckFailed>> + Send;
}

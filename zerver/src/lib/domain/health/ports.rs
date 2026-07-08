//! Port traits for health check operations.
//!
//! This module defines the interfaces (ports) for service health monitoring.
//! Health checks verify the service is operational and can communicate with dependencies.

use crate::domain::{BoxFuture, health::models::HealthCheckFailed};
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

/// Object-safe wrapper used by `AppState` so the concrete service type stays
/// out of the generic parameter list. Auto-implemented for any `HealthService`.
pub trait ErasedHealthService: Send + Sync + 'static {
    /// See [`HealthService::check_database`].
    fn check_database<'a>(&'a self) -> BoxFuture<'a, Result<(), HealthCheckFailed>>;
}

impl<T> ErasedHealthService for T
where
    T: HealthService,
{
    fn check_database<'a>(&'a self) -> BoxFuture<'a, Result<(), HealthCheckFailed>> {
        Box::pin(HealthService::check_database(self))
    }
}

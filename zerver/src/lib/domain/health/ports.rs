//! Port traits for health check operations.
//!
//! This module defines the interfaces (ports) for service health monitoring.
//! Health checks verify the service is operational and can communicate with dependencies.
//!
//! # Purpose
//!
//! Health checks enable:
//! - **Monitoring**: External systems verify service liveness
//! - **Load Balancing**: Unhealthy instances removed from rotation
//! - **Kubernetes**: Liveness and readiness probes
//! - **Debugging**: Quick verification of database connectivity
//!
//! # Implementation
//!
//! - Repository: Executes simple database query (SELECT 1)
//! - Service: Orchestrates health checks (currently just database)

use crate::domain::health::models::HealthCheckFailed;
use std::future::Future;

/// Database port for health check operations.
///
/// Executes simple database query to verify connectivity and basic operations.
pub trait HealthRepository: Clone + Send + Sync + 'static {
    /// Checks database connectivity and basic query operations.
    ///
    /// Executes `SELECT 1` to verify database is reachable and responding.
    fn check_database(&self) -> impl Future<Output = Result<(), HealthCheckFailed>> + Send;
}

/// Service port for health check business logic.
///
/// Orchestrates health checks for all dependencies.
/// Currently only checks database, but could expand to check external APIs,
/// cache servers, message queues, etc.
pub trait HealthService: Clone + Send + Sync + 'static {
    /// Checks all service dependencies (currently just database).
    ///
    /// Returns Ok if all dependencies are healthy, Err otherwise.
    /// Used by HTTP health check endpoint (GET /health).
    fn check_database(&self) -> impl Future<Output = Result<(), HealthCheckFailed>> + Send;
}

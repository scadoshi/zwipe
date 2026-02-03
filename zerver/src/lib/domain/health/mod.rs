//! Health check operations for service monitoring.
//!
//! This module provides simple health check functionality to verify the service
//! is operational and can communicate with its dependencies (database).
//!
//! # Purpose
//!
//! Health checks enable:
//! - **Monitoring**: External systems can verify service liveness
//! - **Load Balancing**: Load balancers can remove unhealthy instances
//! - **Debugging**: Quick verification that dependencies are accessible
//! - **Kubernetes**: Liveness and readiness probes
//!
//! # Health Check Operation
//!
//! Performs a simple database query to verify:
//! 1. Database is reachable
//! 2. Connection pool is functioning
//! 3. Basic query operations work
//!
//! # Example
//!
//! ```rust,ignore
//! // Health check endpoint handler
//! async fn health_check(
//!     State(health_service): State<HealthService>
//! ) -> Result<StatusCode, HealthCheckFailed> {
//!     health_service.check().await?;
//!     Ok(StatusCode::OK)
//! }
//! ```

/// Health check models and error types.
pub mod models;

/// Port traits (interfaces) for health check operations.
#[cfg(feature = "zerver")]
pub mod ports;

/// Service implementations for health check logic.
#[cfg(feature = "zerver")]
pub mod services;

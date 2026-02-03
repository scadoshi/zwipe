#[cfg(feature = "zerver")]
use thiserror::Error;

#[cfg(feature = "zerver")]
/// Error returned when a health check fails.
///
/// Wraps the underlying error (typically database connection failure).
/// Used by monitoring systems to detect service degradation.
#[derive(Debug, Error)]
#[error("failed health check: {0}")]
pub struct HealthCheckFailed(pub anyhow::Error);

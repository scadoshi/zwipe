#[cfg(feature = "zerver")]
use thiserror::Error;

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
#[error("failed health check: {0}")]
pub struct HealthCheckFailed(pub anyhow::Error);

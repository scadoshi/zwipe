// external
use thiserror::Error;

/// wraps `anyhow::Error` if health check fails for some reason
#[derive(Debug, Error)]
#[error("Failed health check: {0}")]
pub struct HealthCheckFailed(pub anyhow::Error);

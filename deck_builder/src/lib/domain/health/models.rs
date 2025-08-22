use thiserror::Error;

#[derive(Debug, Error)]
#[error("Failed health check: {0}")]
pub struct HealthCheckFailed(pub anyhow::Error);

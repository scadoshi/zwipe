//! Service-layer error types for metrics operations.

use thiserror::Error;

/// Errors that can occur on any metrics persistence call.
#[derive(Debug, Error)]
pub enum MetricsError {
    /// Owning user row missing for a lifetime fetch.
    #[error("metrics row not found for user")]
    NotFound,

    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

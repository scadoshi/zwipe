//! User display settings.
//!
//! Service-layer error types for setting operations. Domain types live in zwipe-core.

use thiserror::Error;
pub use zwipe_core::domain::user::settings::InvalidUpdateSettings;

/// Error from the update settings operation.
#[derive(Debug, Error)]
pub enum UpdateSettingsError {
    /// Validation failed.
    #[error(transparent)]
    Invalid(#[from] InvalidUpdateSettings),
    /// Database error.
    #[error("database error")]
    Database(#[from] anyhow::Error),
}

/// Error from the fetch settings operation.
#[derive(Debug, Error)]
pub enum FetchSettingsError {
    /// Database error.
    #[error("database error")]
    Database(#[from] anyhow::Error),
}

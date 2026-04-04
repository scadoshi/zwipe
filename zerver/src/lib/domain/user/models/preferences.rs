//! User display preferences.
//!
//! Service-layer error types for preference operations. Domain types live in zwipe-core.

use thiserror::Error;
pub use zwipe_core::domain::user::preferences::InvalidUpdatePreferences;

/// Error from the update preferences operation.
#[derive(Debug, Error)]
pub enum UpdatePreferencesError {
    /// Validation failed.
    #[error(transparent)]
    Invalid(#[from] InvalidUpdatePreferences),
    /// Database error.
    #[error("database error")]
    Database(#[from] anyhow::Error),
}

/// Error from the get preferences operation.
#[derive(Debug, Error)]
pub enum GetPreferencesError {
    /// Database error.
    #[error("database error")]
    Database(#[from] anyhow::Error),
}

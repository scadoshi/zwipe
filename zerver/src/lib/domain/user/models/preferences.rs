//! User display preferences.
//!
//! Domain types re-exported from `zwipe_core`. Service-layer error types remain here.

pub use zwipe_core::domain::user::preferences::*;

use thiserror::Error;

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

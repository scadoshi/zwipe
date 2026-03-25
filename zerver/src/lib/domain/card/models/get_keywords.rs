//! Get keywords operation.
//!
//! Retrieves list of all unique keyword abilities (Flying, Trample, Deathtouch, etc.).
//! Used for keyword filter autocomplete in card search.

use thiserror::Error;

/// Errors that can occur when retrieving keyword list.
#[derive(Debug, Error)]
pub enum GetKeywordsError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

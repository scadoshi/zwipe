//! Get languages operation.
//!
//! Retrieves list of all unique card languages in the database.
//! Used for language filter in card search (e.g., "en", "ja", "es").

use thiserror::Error;

/// Errors that can occur when retrieving language list.
#[derive(Debug, Error)]
pub enum GetLanguagesError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

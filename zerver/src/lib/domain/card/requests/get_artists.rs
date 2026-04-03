//! Get artists operation.
//!
//! Retrieves list of all unique card artists in the database.
//! Used for artist filter autocomplete in card search.

use thiserror::Error;

/// Errors that can occur when retrieving artist list.
#[derive(Debug, Error)]
pub enum GetArtistsError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

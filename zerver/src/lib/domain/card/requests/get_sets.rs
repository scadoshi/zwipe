//! Get sets operation.
//!
//! Retrieves list of all unique MTG set codes in the database.
//! Used for set filter autocomplete (e.g., "MH2", "ONE", "BRO").

use thiserror::Error;

/// Errors that can occur when retrieving set list.
#[derive(Debug, Error)]
pub enum GetSetsError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

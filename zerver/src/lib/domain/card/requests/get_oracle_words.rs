//! Get oracle words operation.
//!
//! Retrieves list of all unique words from oracle text, normalized and filtered of grammatical
//! noise. Used for oracle text word-picker filter autocomplete in card search.

use thiserror::Error;

/// Errors that can occur when retrieving oracle word list.
#[derive(Debug, Error)]
pub enum GetOracleWordsError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

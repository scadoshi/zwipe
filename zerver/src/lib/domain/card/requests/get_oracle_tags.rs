//! Get oracle tags operation.
//!
//! Retrieves the full oracle tag catalog (slug, label, description, parent slugs).
//! Used to populate the otag filter picker in card search. See
//! `context/plans/otags/`.

use thiserror::Error;

/// Errors that can occur when retrieving the oracle tag catalog.
#[derive(Debug, Error)]
pub enum GetOracleTagsError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

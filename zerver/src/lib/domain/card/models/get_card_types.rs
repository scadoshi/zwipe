//! Get card types operation.
//!
//! Retrieves list of all unique card types (Creature, Instant, Sorcery, etc.).
//! Used for type filter autocomplete in card search.

use thiserror::Error;

/// Errors that can occur when retrieving card type list.
#[derive(Debug, Error)]
pub enum GetCardTypesError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

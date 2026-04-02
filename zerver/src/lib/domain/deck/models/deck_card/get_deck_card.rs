//! Get deck card operation.
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.

pub use zwipe_core::domain::deck::requests::get_deck_card::*;

#[cfg(feature = "zerver")]
use thiserror::Error;

#[allow(missing_docs)]
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetDeckCardError {
    #[error("deck card not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("deck card found but database returned invalid object: {0}")]
    DeckCardFromDb(anyhow::Error),
    #[error("deck does not belong to requesting user")]
    Forbidden,
}

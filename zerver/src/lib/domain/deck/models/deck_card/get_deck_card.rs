//! Get deck card operation.
//!
//! Re-exported from `zwipe_core`. Service-layer error type remains here.

use thiserror::Error;

#[allow(missing_docs)]
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

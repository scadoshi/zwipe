//! Get deck card operation; validation error only.

use thiserror::Error;

#[expect(
    missing_docs,
    reason = "each variant names the id that failed to parse"
)]
#[derive(Debug, Error)]
pub enum InvalidGetDeckCard {
    #[error(transparent)]
    DeckId(uuid::Error),
    #[error(transparent)]
    ScryfallDataId(uuid::Error),
}

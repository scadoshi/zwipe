//! Get deck card operation — validation error only.

use thiserror::Error;

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum InvalidGetDeckCard {
    #[error(transparent)]
    DeckId(uuid::Error),
    #[error(transparent)]
    ScryfallDataId(uuid::Error),
}

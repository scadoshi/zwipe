use thiserror::Error;

#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum InvalidGetDeckCard {
    #[error(transparent)]
    DeckId(uuid::Error),
    #[error(transparent)]
    ScryfallDataId(uuid::Error),
}

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

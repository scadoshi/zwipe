use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum InvalidDeleteDeck {
    #[error(transparent)]
    UserId(uuid::Error),
    #[error(transparent)]
    DeckId(uuid::Error),
}

#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum DeleteDeckError {
    #[error("deck not found")]
    NotFound,
    #[error(transparent)]
    Database(anyhow::Error),
    #[error("deck does not belong to requesting user")]
    Forbidden,
}

#[derive(Debug, Clone)]
pub struct DeleteDeck {
    pub user_id: Uuid,
    pub deck_id: Uuid,
}

impl DeleteDeck {
    pub fn new(user_id: Uuid, deck_id: &str) -> Result<Self, InvalidDeleteDeck> {
        let deck_id = Uuid::try_parse(deck_id.trim()).map_err(InvalidDeleteDeck::DeckId)?;
        Ok(Self { user_id, deck_id })
    }
}

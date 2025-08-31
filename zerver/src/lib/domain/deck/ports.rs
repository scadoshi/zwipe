use std::future::Future;

use crate::domain::deck::models::deck::{CreateDeckError, Deck};

/// enables deck related database operations
pub trait DeckRepository: Clone + Send + Sync + 'static {
    fn create_deck(&self, deck: Deck) -> impl Future<Output = Result<Deck, CreateDeckError>>;
}

/// orchestrates deck related operations
pub trait DeckService: Clone + Send + Sync + 'static {}

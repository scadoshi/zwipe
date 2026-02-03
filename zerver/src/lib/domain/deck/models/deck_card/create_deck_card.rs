//! Add card to deck operation.
//!
//! Adds a card to a deck with specified quantity. If the card already exists
//! in the deck, this operation will fail with a duplicate error (use update instead).
//!
//! # Quantity Validation
//!
//! Quantity must respect deck copy limits:
//! - **Singleton decks** (copy_max = 1): 1 copy only (except basic lands)
//! - **Standard decks** (copy_max = 4): 1-4 copies (except basic lands)
//! - **Basic lands**: Unlimited (1-99) in any format
//!
//! # Authorization
//!
//! Only the deck owner can add cards to their deck.

#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
use crate::domain::deck::models::deck_card::quantity::{InvalidQuantity, Quantity};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing a [`CreateDeckCard`] request.
#[derive(Debug, Error)]
pub enum InvalidCreateDeckCard {
    /// Invalid deck ID format.
    #[error(transparent)]
    DeckId(uuid::Error),
    /// Invalid card ID format.
    #[error(transparent)]
    ScryfallDataId(uuid::Error),
    /// Quantity is invalid (violates copy limits or out of range).
    #[error(transparent)]
    Quantity(InvalidQuantity),
}

impl From<InvalidQuantity> for InvalidCreateDeckCard {
    fn from(value: InvalidQuantity) -> Self {
        Self::Quantity(value)
    }
}

/// Errors that can occur during deck card creation execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CreateDeckCardError {
    /// Card already exists in this deck (use update to change quantity).
    #[error("card and deck combination already exists")]
    Duplicate,
    /// Database returned invalid data after creation.
    #[error("deck card created but database returned invalid object {0}")]
    DeckCardFromDb(anyhow::Error),
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Error retrieving deck for authorization check.
    #[error(transparent)]
    GetDeckProfileError(#[from] GetDeckProfileError),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
}

/// Request to add a card to a deck.
///
/// Creates a new deck_card entry with the specified quantity.
/// If the card already exists in the deck, returns duplicate error.
///
/// # Example
///
/// ```rust,ignore
/// let add_card = CreateDeckCard::new(
///     user_id,
///     "deck-uuid",
///     "card-uuid",
///     4  // 4 copies
/// )?;
/// deck_service.create_deck_card(&add_card).await?;
/// ```
#[derive(Debug, Clone)]
pub struct CreateDeckCard {
    /// Requesting user (for authorization).
    pub user_id: Uuid,
    /// Deck to add card to.
    pub deck_id: Uuid,
    /// Card to add (Scryfall data ID).
    pub scryfall_data_id: Uuid,
    /// How many copies (validated against deck copy limits).
    pub quantity: Quantity,
}

impl CreateDeckCard {
    /// Creates a new deck card addition request with validation.
    ///
    /// # Parameters
    ///
    /// - `user_id`: Requesting user's ID
    /// - `deck_id`: Deck ID as string (will be parsed)
    /// - `scryfall_data_id`: Card ID as string (will be parsed)
    /// - `quantity`: Number of copies (will be validated)
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCreateDeckCard`] if:
    /// - Deck ID is not a valid UUID
    /// - Card ID is not a valid UUID
    /// - Quantity is invalid (0, negative, or violates copy limits)
    pub fn new(
        user_id: Uuid,
        deck_id: &str,
        scryfall_data_id: &str,
        quantity: i32,
    ) -> Result<Self, InvalidCreateDeckCard> {
        let deck_id = Uuid::try_parse(deck_id).map_err(InvalidCreateDeckCard::DeckId)?;
        let scryfall_data_id =
            Uuid::try_parse(scryfall_data_id).map_err(InvalidCreateDeckCard::ScryfallDataId)?;
        let quantity = Quantity::new(quantity)?;

        Ok(Self {
            deck_id,
            scryfall_data_id,
            quantity,
            user_id,
        })
    }
}

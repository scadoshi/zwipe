/// Add card to deck operation.
pub mod create_deck_card;
/// Remove card from deck operation.
pub mod delete_deck_card;
/// Get deck card operation.
pub mod get_deck_card;
/// Quantity validation (1-99 cards, respects deck copy limits).
pub mod quantity;
/// Update card quantity in deck operation.
pub mod update_deck_card;

use crate::domain::deck::models::deck_card::quantity::Quantity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A card in a deck (join table entity).
///
/// Represents the many-to-many relationship between decks and cards.
/// Each entry specifies which card, in which deck, and how many copies.
///
/// # Quantity Limits
///
/// Quantity is validated against deck copy limits:
/// - **Singleton decks** (copy_max = 1): Only 1 copy allowed (except basic lands)
/// - **Standard decks** (copy_max = 4): Up to 4 copies allowed (except basic lands)
/// - **Basic lands**: Unlimited copies in either format
#[derive(Debug, Serialize, Deserialize)]
pub struct DeckCard {
    /// The deck containing this card.
    pub deck_id: Uuid,
    /// The card (Scryfall data ID).
    pub scryfall_data_id: Uuid,
    /// How many copies (1-99, validated against deck copy limit).
    pub quantity: Quantity,
}

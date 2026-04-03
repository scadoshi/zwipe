//! Deck-card join entity.

use crate::domain::deck::Quantity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A card in a deck (join table entity).
///
/// Represents the many-to-many relationship between decks and cards.
/// Each entry specifies which card, in which deck, and how many copies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeckCard {
    /// The deck containing this card.
    pub deck_id: Uuid,
    /// The card (Scryfall data ID).
    pub scryfall_data_id: Uuid,
    /// How many copies (1-99, validated against deck copy limit).
    pub quantity: Quantity,
}

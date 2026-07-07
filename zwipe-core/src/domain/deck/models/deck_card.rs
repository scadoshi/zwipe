//! Deck-card join entity.

use super::board::Board;
use crate::domain::deck::Quantity;
use chrono::{DateTime, Utc};
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
    /// The selected printing (Scryfall data ID).
    pub scryfall_data_id: Uuid,
    /// The logical card identity (shared across all printings).
    pub oracle_id: Uuid,
    /// How many copies (1-99, validated against deck copy limit).
    pub quantity: Quantity,
    /// Which board this card belongs to (deck, maybeboard, or sideboard).
    pub board: Board,
    /// When this card was starred as a deck MVP; `None` = not an MVP.
    /// The timestamp is the vesting clock (global signal counts a star after
    /// 3 days). Mainboard-only; up to 3 per deck (enforced server-side).
    /// `#[serde(default)]` so new clients parse old servers and vice versa.
    #[serde(default)]
    pub mvp_at: Option<DateTime<Utc>>,
}

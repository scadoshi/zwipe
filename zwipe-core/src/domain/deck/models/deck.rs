//! Deck aggregate entity combining profile, cards, and warnings.

use crate::domain::{
    card::Card,
    deck::{DeckCard, DeckProfile, DeckWarning},
};
use serde::{Deserialize, Serialize};

/// A card paired with its deck membership data (quantity, IDs).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeckEntry {
    /// Full card data (Scryfall + profile).
    pub card: Card,
    /// Deck-card join data (quantity, deck_id, scryfall_data_id).
    pub deck_card: DeckCard,
}

/// A complete Magic: The Gathering deck with metadata and card list.
///
/// This aggregate entity combines deck metadata ([`DeckProfile`]) with the
/// actual card inventory. It represents the full state of a user's deck.
///
/// # Structure
///
/// - **DeckProfile**: Name, commander, owner ID
/// - **Entries**: Each entry pairs a card with its deck membership data (quantity, etc.)
///
/// # Use Cases
///
/// This complete view is used when:
/// - Displaying a deck to the user
/// - Validating deck construction rules
/// - Exporting deck to external formats
/// - Analyzing deck composition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Deck {
    /// Deck metadata (name, commander, format, owner).
    pub deck_profile: DeckProfile,

    /// Card entries, each pairing a [`Card`] with its [`DeckCard`] join data.
    pub entries: Vec<DeckEntry>,

    /// Deck-building warnings (informational, not blocking).
    pub warnings: Vec<DeckWarning>,

    /// Command-zone cards (commander, partner, background, signature spell) that
    /// live on the profile rather than in `entries`. Carried here so clients can
    /// fold them into price and card-count calcs. `#[serde(default)]` keeps older
    /// servers (which omit the field) and older clients (which ignore it) working.
    #[serde(default)]
    pub command_zone_cards: Vec<Card>,
}

impl Deck {
    /// Creates a new deck from profile, entries, and warnings. The command zone
    /// starts empty; attach it with [`with_command_zone_cards`](Self::with_command_zone_cards).
    pub fn new(
        deck_profile: DeckProfile,
        entries: Vec<DeckEntry>,
        warnings: Vec<DeckWarning>,
    ) -> Self {
        Self {
            deck_profile,
            entries,
            warnings,
            command_zone_cards: Vec::new(),
        }
    }

    /// Attaches the command-zone cards (commander, partner, background, signature
    /// spell) so clients can include them in price and card-count calcs.
    pub fn with_command_zone_cards(mut self, cards: Vec<Card>) -> Self {
        self.command_zone_cards = cards;
        self
    }
}

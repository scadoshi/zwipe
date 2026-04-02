//! Deck entity and related operations.
//!
//! This module provides the complete deck entity (profile + cards) and all
//! deck profile management operations.

/// Create deck profile operation.
pub mod create_deck_profile;
/// Deck warning value object.
pub mod deck_warning;
/// Deck name validation (1-64 chars, no profanity).
pub mod deck_name;
/// Deck profile entity (deck metadata).
pub mod deck_profile;
/// Delete deck operation.
pub mod delete_deck;
/// Deck format classification (Commander, Standard, Modern, etc.).
pub mod format;
/// Get complete deck operation (profile + cards).
pub mod get_deck;
/// Get tokens produced by all cards in a deck.
pub mod get_deck_tokens;
/// Get single deck profile operation.
pub mod get_deck_profile;
/// Deck validation logic (generates warnings).
pub mod validate_deck;
/// Get multiple deck profiles operation (list user's decks).
pub mod get_deck_profiles;
/// Update deck profile operation.
pub mod update_deck_profile;

use crate::domain::{
    card::models::Card,
    deck::models::{
        deck::{deck_profile::DeckProfile, deck_warning::DeckWarning},
        deck_card::DeckCard,
    },
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
///
/// # Example
///
/// ```rust,ignore
/// use zwipe::domain::deck::models::deck::Deck;
///
/// // Fetch complete deck
/// let deck: Deck = deck_service.get_deck(user_id, deck_id).await?;
///
/// println!("Deck: {}", deck.deck_profile.name);
/// println!("Entries: {}", deck.entries.len());
/// println!("Commander: {:?}", deck.deck_profile.commander_id);
///
/// // Display cards with quantities
/// for entry in &deck.entries {
///     println!("  - {} x{}", entry.card.scryfall_data.name, *entry.deck_card.quantity);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Deck {
    /// Deck metadata (name, commander, format, owner).
    pub deck_profile: DeckProfile,

    /// Card entries, each pairing a [`Card`] with its [`DeckCard`] join data.
    pub entries: Vec<DeckEntry>,

    /// Deck-building warnings (informational, not blocking).
    pub warnings: Vec<DeckWarning>,
}

impl Deck {
    /// Creates a new deck from profile and entries.
    ///
    /// Typically called by the service layer after fetching deck profile
    /// and joining card data with deck-card membership data.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let deck = Deck::new(deck_profile, entries);
    /// ```
    pub fn new(deck_profile: DeckProfile, entries: Vec<DeckEntry>, warnings: Vec<DeckWarning>) -> Self {
        Self {
            deck_profile,
            entries,
            warnings,
        }
    }
}

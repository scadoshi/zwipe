//! Archidekt deck import operation — server-side types.
//!
//! The parsed, source-neutral deck (`ArchidektDeck`) is produced by the
//! Archidekt outbound adapter and consumed by the deck service, which resolves
//! each printing against the card database and creates a new deck. Card *data*
//! is never taken from Archidekt — only identity (Scryfall printing id),
//! quantity, and command-zone membership.

#[cfg(feature = "zerver")]
use thiserror::Error;
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::requests::create_deck_profile::InvalidCreateDeckProfile;
#[cfg(feature = "zerver")]
use zwipe_core::domain::deck::requests::import_deck_cards::{ImportedCard, UnresolvedCard};

#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::create_deck_profile::CreateDeckProfileError;
#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck_card::import_deck_cards::ImportDeckCardsError;

/// A deck fetched from Archidekt and reduced to what Zwipe needs to import it.
#[cfg(feature = "zerver")]
#[derive(Debug, Clone)]
pub struct ArchidektDeck {
    /// Deck name as set on Archidekt.
    pub name: String,
    /// Zwipe format string (e.g. `"commander"`), if Archidekt's numeric format
    /// mapped to a known one. `None` leaves the new deck without a format.
    pub format: Option<String>,
    /// Every card in the deck (command zone included).
    pub cards: Vec<ArchidektCard>,
}

/// A single card entry from an Archidekt deck.
#[cfg(feature = "zerver")]
#[derive(Debug, Clone)]
pub struct ArchidektCard {
    /// Scryfall printing id (`card.uid`), resolved against `scryfall_data.id`.
    /// `Uuid::nil()` if Archidekt gave an unparseable id — surfaces as unresolved.
    pub scryfall_id: Uuid,
    /// Card name (Archidekt's oracle name), used only for the unresolved report.
    pub name: String,
    /// Number of copies.
    pub quantity: i32,
    /// Whether Archidekt tags this card as in the command zone (premier category).
    pub command_zone: bool,
}

/// Outcome of a successful Archidekt import.
#[cfg(feature = "zerver")]
#[derive(Debug, Clone)]
pub struct ArchidektImportResult {
    /// Id of the newly created deck.
    pub deck_id: Uuid,
    /// Validated name the deck was created with.
    pub deck_name: String,
    /// Format the deck was created with, if any.
    pub format: Option<String>,
    /// Names of cards placed in the command zone.
    pub command_zone: Vec<String>,
    /// Cards inserted into the deck.
    pub imported: Vec<ImportedCard>,
    /// Cards that didn't resolve against the card database.
    pub unresolved: Vec<UnresolvedCard>,
}

/// Errors that can occur while importing an Archidekt deck.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum ImportArchidektError {
    /// The chosen deck name failed validation (length, profanity) or the format
    /// string was unrecognized.
    #[error(transparent)]
    InvalidProfile(#[from] InvalidCreateDeckProfile),
    /// Creating the deck profile failed (e.g. deck-count limit reached).
    #[error(transparent)]
    CreateDeck(#[from] CreateDeckProfileError),
    /// Inserting the resolved cards failed (e.g. card limit reached).
    #[error(transparent)]
    Insert(#[from] ImportDeckCardsError),
    /// Database operation failed during resolution.
    #[error(transparent)]
    Database(anyhow::Error),
}

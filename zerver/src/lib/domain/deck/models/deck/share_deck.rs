//! Deck share operations (create/revoke a share token, public shared read).
//!
//! The share token is a capability: an unguessable UUID that makes the deck
//! readable at a public URL. NULL = private. Server-side types only.

#[cfg(feature = "zerver")]
use crate::domain::deck::models::deck::get_deck::GetDeckError;
#[cfg(feature = "zerver")]
use thiserror::Error;
#[cfg(feature = "zerver")]
use zwipe_core::domain::{card::Card, deck::Deck};

/// Errors that can occur while sharing or unsharing a deck.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum ShareDeckError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// Requesting user doesn't own this deck.
    #[error("deck does not belong to requesting user")]
    Forbidden,
    /// Deck does not exist.
    #[error("deck not found")]
    NotFound,
}

/// Errors that can occur while resolving a public shared-deck read.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum GetSharedDeckError {
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
    /// No deck carries this token (never shared, or sharing was stopped).
    #[error("no deck is shared under this token")]
    NotFound,
    /// Error assembling the deck aggregate.
    #[error(transparent)]
    GetDeck(#[from] GetDeckError),
}

/// A deck resolved through its share token, with command zone cards attached
/// for the public page. The handler strips owner identity before responding.
#[cfg(feature = "zerver")]
#[derive(Debug, Clone)]
pub struct SharedDeck {
    /// The full deck aggregate (profile + entries).
    pub deck: Deck,
    /// Commander card, if set.
    pub commander: Option<Card>,
    /// Partner commander card, if set.
    pub partner_commander: Option<Card>,
    /// Background enchantment card, if set.
    pub background: Option<Card>,
    /// Signature spell card, if set.
    pub signature_spell: Option<Card>,
    /// Token cards the deck's cards produce (same derivation as the app's token
    /// list), so the shared page can show them without an authed call.
    pub tokens: Vec<Card>,
}

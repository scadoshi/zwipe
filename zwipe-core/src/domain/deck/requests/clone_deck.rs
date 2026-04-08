//! Deck cloning request.
//!
//! Copies an existing deck (profile + all entries on every board) under a
//! new name, owned by the same caller. Carries everything the service needs
//! to enforce authorization and the deck-count limit.

use crate::domain::deck::models::deck_name::{DeckName, InvalidDeckname};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Request to clone an existing deck into a new one with a caller-chosen name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneDeck {
    /// Deck to copy from. Must be owned by `user_id`.
    pub source_deck_id: Uuid,
    /// Validated name for the new deck.
    pub new_name: DeckName,
    /// Authenticated caller (new deck owner + source ownership check).
    pub user_id: Uuid,
    /// Whether the caller's email is verified (affects deck count limit).
    pub email_verified: bool,
}

impl CloneDeck {
    /// Creates a new clone request, validating the new deck name.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCloneDeck::Name`] if `new_name` fails
    /// [`DeckName`] validation (empty, too long, or profanity).
    pub fn new(
        source_deck_id: Uuid,
        new_name: impl Into<String>,
        user_id: Uuid,
        email_verified: bool,
    ) -> Result<Self, InvalidCloneDeck> {
        Ok(Self {
            source_deck_id,
            new_name: DeckName::new(new_name)?,
            user_id,
            email_verified,
        })
    }
}

/// Errors that occur when constructing an invalid [`CloneDeck`] request.
#[derive(Debug, Error)]
pub enum InvalidCloneDeck {
    /// The supplied new name failed validation.
    #[error(transparent)]
    Name(#[from] InvalidDeckname),
}

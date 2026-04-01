//! Create deck profile operation.
//!
//! Allows users to create a new empty deck with metadata (name, commander).
//! Decks start empty and cards are added via separate operations.
//!
//! # Validation
//!
//! - **Name**: 1-64 characters, no profanity
//! - **Commander**: Optional card ID for commander format decks
//!
//! # Uniqueness
//!
//! Users cannot have multiple decks with the same name (user_id + name combination is unique).

use crate::domain::deck::models::deck::{
    deck_name::{DeckName, InvalidDeckname},
    format::{Format, InvalidFormat},
};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing a [`CreateDeckProfile`] request.
#[derive(Debug, Error)]
pub enum InvalidCreateDeckProfile {
    /// Deck name doesn't meet requirements (length, profanity).
    #[error(transparent)]
    DeckName(#[from] InvalidDeckname),
    /// Format string is not a recognized format.
    #[error(transparent)]
    Format(#[from] InvalidFormat),
}

/// Errors that can occur during deck profile creation execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CreateDeckProfileError {
    /// User already has a deck with this name.
    #[error("deck with name and user id combination already exists")]
    Duplicate,
    /// User has reached the maximum number of decks.
    #[error("deck limit reached")]
    LimitReached,
    /// Database returned invalid data after creation.
    #[error("deck created but database returned invalid object {0}")]
    DeckFromDb(anyhow::Error),
    /// Database operation failed.
    #[error(transparent)]
    Database(anyhow::Error),
}

/// Request to create a new deck profile.
///
/// Creates an empty deck with metadata. Cards are added separately via
/// deck card operations.
///
/// # Example
///
/// ```rust,ignore
/// let create = CreateDeckProfile::new(
///     "My EDH Deck",
///     Some(commander_card_id),
///     user_id
/// )?;
/// ```
#[derive(Debug, Clone)]
pub struct CreateDeckProfile {
    /// Validated deck name (1-64 chars, no profanity).
    pub name: DeckName,
    /// Optional commander card ID for Commander format.
    pub commander_id: Option<Uuid>,
    /// Optional deck format.
    pub format: Option<Format>,
    /// Owner of this deck.
    pub user_id: Uuid,
    /// Whether the requesting user's email is verified (from JWT claim).
    /// Used to select the appropriate deck count limit.
    pub email_verified: bool,
}

impl CreateDeckProfile {
    /// Creates a new deck profile creation request with validation.
    ///
    /// # Parameters
    ///
    /// - `name`: Deck name (will be validated)
    /// - `commander_id`: Optional commander card ID
    /// - `user_id`: Owner's user ID
    /// - `email_verified`: Whether the user's email is verified
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCreateDeckProfile`] if name doesn't meet requirements.
    pub fn new(
        name: impl Into<String>,
        commander_id: Option<Uuid>,
        format: Option<&str>,
        user_id: Uuid,
        email_verified: bool,
    ) -> Result<Self, InvalidCreateDeckProfile> {
        let name = DeckName::new(name)?;
        let format = format.map(Format::try_from).transpose()?;
        Ok(Self {
            name,
            commander_id,
            format,
            user_id,
            email_verified,
        })
    }
}

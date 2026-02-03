//! Create deck profile operation.
//!
//! Allows users to create a new empty deck with metadata (name, commander, copy limit).
//! Decks start empty and cards are added via separate operations.
//!
//! # Validation
//!
//! - **Name**: 1-64 characters, no profanity
//! - **Copy Limit**: 1 (singleton) or 4 (standard) - defaults to 4 if not specified
//! - **Commander**: Optional card ID for commander format decks
//!
//! # Uniqueness
//!
//! Users cannot have multiple decks with the same name (user_id + name combination is unique).

use crate::domain::deck::models::deck::{
    copy_max::{CopyMax, InvalidCopyMax},
    deck_name::{DeckName, InvalidDeckname},
};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing a [`CreateDeckProfile`] request.
#[derive(Debug, Error)]
pub enum InvalidCreateDeckProfile {
    /// Deck name doesn't meet requirements (length, profanity).
    #[error(transparent)]
    DeckName(#[from] InvalidDeckname),
    /// Copy limit is invalid (must be 1 or 4).
    #[error(transparent)]
    CopyMax(#[from] InvalidCopyMax),
}

/// Errors that can occur during deck profile creation execution.
#[cfg(feature = "zerver")]
#[derive(Debug, Error)]
pub enum CreateDeckProfileError {
    /// User already has a deck with this name.
    #[error("deck with name and user id combination already exists")]
    Duplicate,
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
///     Some(1), // Singleton (Commander)
///     user_id
/// )?;
/// ```
#[derive(Debug, Clone)]
pub struct CreateDeckProfile {
    /// Validated deck name (1-64 chars, no profanity).
    pub name: DeckName,
    /// Optional commander card ID for Commander format.
    pub commander_id: Option<Uuid>,
    /// Optional copy limit (1 = singleton, 4 = standard). Defaults to 4.
    pub copy_max: Option<CopyMax>,
    /// Owner of this deck.
    pub user_id: Uuid,
}

impl CreateDeckProfile {
    /// Creates a new deck profile creation request with validation.
    ///
    /// # Parameters
    ///
    /// - `name`: Deck name (will be validated)
    /// - `commander_id`: Optional commander card ID
    /// - `copy_max`: Optional copy limit (1 or 4)
    /// - `user_id`: Owner's user ID
    ///
    /// # Errors
    ///
    /// Returns [`InvalidCreateDeckProfile`] if:
    /// - Name doesn't meet requirements
    /// - Copy limit is not 1 or 4
    pub fn new(
        name: &str,
        commander_id: Option<Uuid>,
        copy_max: Option<i32>,
        user_id: Uuid,
    ) -> Result<Self, InvalidCreateDeckProfile> {
        let name = DeckName::new(name)?;
        let copy_max: Option<CopyMax> = copy_max.map(CopyMax::new).transpose()?;
        Ok(Self {
            name,
            commander_id,
            copy_max,
            user_id,
        })
    }
}

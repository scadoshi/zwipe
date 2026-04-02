//! Create deck profile operation.

use crate::domain::deck::{
    DeckName, InvalidDeckname,
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

/// Request to create a new deck profile.
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
    /// Whether the requesting user's email is verified.
    pub email_verified: bool,
}

impl CreateDeckProfile {
    /// Creates a new deck profile creation request with validation.
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

//! Update deck profile operation.

use crate::domain::deck::{
    DeckName, InvalidDeckname,
    format::{Format, InvalidFormat},
};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur while constructing an [`UpdateDeckProfile`] request.
#[derive(Debug, Error)]
pub enum InvalidUpdateDeckProfile {
    /// Deck name doesn't meet requirements.
    #[error(transparent)]
    DeckName(InvalidDeckname),
    /// Format string is not a recognized format.
    #[error(transparent)]
    Format(InvalidFormat),
    /// No fields specified for update.
    #[error("must update at least one field")]
    NoUpdates,
}

impl From<InvalidDeckname> for InvalidUpdateDeckProfile {
    fn from(value: InvalidDeckname) -> Self {
        Self::DeckName(value)
    }
}

impl From<InvalidFormat> for InvalidUpdateDeckProfile {
    fn from(value: InvalidFormat) -> Self {
        Self::Format(value)
    }
}

/// Request to update deck profile metadata.
#[derive(Debug, Clone)]
pub struct UpdateDeckProfile {
    /// ID of deck to update.
    pub deck_id: Uuid,
    /// Optional new name.
    pub name: Option<DeckName>,
    /// Optional commander update.
    pub commander_id: Option<Option<Uuid>>,
    /// Optional format update.
    pub format: Option<Option<Format>>,
    /// Requesting user (for authorization).
    pub user_id: Uuid,
}

impl UpdateDeckProfile {
    /// Creates a new deck profile update request with validation.
    pub fn new(
        deck_id: Uuid,
        name: Option<&str>,
        commander_id: Option<Option<Uuid>>,
        format: Option<Option<&str>>,
        user_id: Uuid,
    ) -> Result<Self, InvalidUpdateDeckProfile> {
        if name.is_none() && commander_id.is_none() && format.is_none() {
            return Err(InvalidUpdateDeckProfile::NoUpdates);
        }
        let name = name.map(DeckName::new).transpose()?;
        let format = format
            .map(|update| update.map(Format::try_from).transpose())
            .transpose()?;

        Ok(Self {
            deck_id,
            name,
            commander_id,
            format,
            user_id,
        })
    }
}

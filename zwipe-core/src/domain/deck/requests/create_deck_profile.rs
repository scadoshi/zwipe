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
    /// Optional partner commander card ID.
    pub partner_commander_id: Option<Uuid>,
    /// Optional background enchantment card ID.
    pub background_id: Option<Uuid>,
    /// Optional signature spell card ID.
    pub signature_spell_id: Option<Uuid>,
    /// Optional deck format.
    pub format: Option<Format>,
    /// Owner of this deck.
    pub user_id: Uuid,
    /// Whether the requesting user's email is verified.
    pub email_verified: bool,
}

impl CreateDeckProfile {
    /// Creates a builder with the required fields.
    pub fn builder(
        name: impl Into<String>,
        user_id: Uuid,
        email_verified: bool,
    ) -> CreateDeckProfileBuilder {
        CreateDeckProfileBuilder {
            name: name.into(),
            user_id,
            email_verified,
            commander_id: None,
            partner_commander_id: None,
            background_id: None,
            signature_spell_id: None,
            format: None,
        }
    }
}

/// Builder for [`CreateDeckProfile`].
pub struct CreateDeckProfileBuilder {
    name: String,
    user_id: Uuid,
    email_verified: bool,
    commander_id: Option<Uuid>,
    partner_commander_id: Option<Uuid>,
    background_id: Option<Uuid>,
    signature_spell_id: Option<Uuid>,
    format: Option<String>,
}

impl CreateDeckProfileBuilder {
    /// Sets the commander card ID.
    pub fn commander_id(mut self, id: Option<Uuid>) -> Self {
        self.commander_id = id;
        self
    }

    /// Sets the partner commander card ID.
    pub fn partner_commander_id(mut self, id: Option<Uuid>) -> Self {
        self.partner_commander_id = id;
        self
    }

    /// Sets the background enchantment card ID.
    pub fn background_id(mut self, id: Option<Uuid>) -> Self {
        self.background_id = id;
        self
    }

    /// Sets the signature spell card ID.
    pub fn signature_spell_id(mut self, id: Option<Uuid>) -> Self {
        self.signature_spell_id = id;
        self
    }

    /// Sets the deck format from a string.
    pub fn format(mut self, format: Option<&str>) -> Self {
        self.format = format.map(|s| s.to_string());
        self
    }

    /// Validates and builds the request.
    pub fn build(self) -> Result<CreateDeckProfile, InvalidCreateDeckProfile> {
        let name = DeckName::new(self.name)?;
        let format = self
            .format
            .as_deref()
            .map(Format::try_from)
            .transpose()?;
        Ok(CreateDeckProfile {
            name,
            commander_id: self.commander_id,
            partner_commander_id: self.partner_commander_id,
            background_id: self.background_id,
            signature_spell_id: self.signature_spell_id,
            format,
            user_id: self.user_id,
            email_verified: self.email_verified,
        })
    }
}

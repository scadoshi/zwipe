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
    /// Optional partner commander update.
    pub partner_commander_id: Option<Option<Uuid>>,
    /// Optional background enchantment update.
    pub background_id: Option<Option<Uuid>>,
    /// Optional signature spell update.
    pub signature_spell_id: Option<Option<Uuid>>,
    /// Optional format update.
    pub format: Option<Option<Format>>,
    /// Requesting user (for authorization).
    pub user_id: Uuid,
}

impl UpdateDeckProfile {
    /// Creates a builder with the required fields.
    pub fn builder(deck_id: Uuid, user_id: Uuid) -> UpdateDeckProfileBuilder {
        UpdateDeckProfileBuilder {
            deck_id,
            user_id,
            name: None,
            commander_id: None,
            partner_commander_id: None,
            background_id: None,
            signature_spell_id: None,
            format: None,
        }
    }
}

/// Builder for [`UpdateDeckProfile`].
pub struct UpdateDeckProfileBuilder {
    deck_id: Uuid,
    user_id: Uuid,
    name: Option<String>,
    commander_id: Option<Option<Uuid>>,
    partner_commander_id: Option<Option<Uuid>>,
    background_id: Option<Option<Uuid>>,
    signature_spell_id: Option<Option<Uuid>>,
    format: Option<Option<String>>,
}

impl UpdateDeckProfileBuilder {
    /// Sets the new deck name.
    pub fn name(mut self, name: Option<&str>) -> Self {
        self.name = name.map(|s| s.to_string());
        self
    }

    /// Sets the commander update.
    pub fn commander_id(mut self, id: Option<Option<Uuid>>) -> Self {
        self.commander_id = id;
        self
    }

    /// Sets the partner commander update.
    pub fn partner_commander_id(mut self, id: Option<Option<Uuid>>) -> Self {
        self.partner_commander_id = id;
        self
    }

    /// Sets the background enchantment update.
    pub fn background_id(mut self, id: Option<Option<Uuid>>) -> Self {
        self.background_id = id;
        self
    }

    /// Sets the signature spell update.
    pub fn signature_spell_id(mut self, id: Option<Option<Uuid>>) -> Self {
        self.signature_spell_id = id;
        self
    }

    /// Sets the format update.
    pub fn format(mut self, format: Option<Option<&str>>) -> Self {
        self.format = format.map(|opt| opt.map(|s| s.to_string()));
        self
    }

    /// Validates and builds the request.
    pub fn build(self) -> Result<UpdateDeckProfile, InvalidUpdateDeckProfile> {
        if self.name.is_none()
            && self.commander_id.is_none()
            && self.partner_commander_id.is_none()
            && self.background_id.is_none()
            && self.signature_spell_id.is_none()
            && self.format.is_none()
        {
            return Err(InvalidUpdateDeckProfile::NoUpdates);
        }
        let name = self.name.as_deref().map(DeckName::new).transpose()?;
        let format = self
            .format
            .map(|update| update.as_deref().map(Format::try_from).transpose())
            .transpose()?;

        Ok(UpdateDeckProfile {
            deck_id: self.deck_id,
            name,
            commander_id: self.commander_id,
            partner_commander_id: self.partner_commander_id,
            background_id: self.background_id,
            signature_spell_id: self.signature_spell_id,
            format,
            user_id: self.user_id,
        })
    }
}

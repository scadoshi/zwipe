//! Create deck profile operation.

use crate::domain::deck::{
    DeckName, DeckTag, InvalidDeckname, InvalidDeckTag, MAX_DECK_TAGS,
    deck_tag::parse_tags,
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
    /// A tag string is not a recognized deck tag.
    #[error(transparent)]
    DeckTag(#[from] InvalidDeckTag),
    /// More than [`MAX_DECK_TAGS`] tags were supplied.
    #[error("a deck may have at most {MAX_DECK_TAGS} tags")]
    TooManyTags,
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
    /// Deck archetype/strategy tags (validated, deduped, at most [`MAX_DECK_TAGS`]).
    pub tags: Vec<DeckTag>,
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
            tags: Vec::new(),
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
    tags: Vec<String>,
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

    /// Sets the deck tags from raw strings (validated, deduped, and capped on build).
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
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
        let tags = parse_tags(&self.tags)?;
        if tags.len() > MAX_DECK_TAGS {
            return Err(InvalidCreateDeckProfile::TooManyTags);
        }
        Ok(CreateDeckProfile {
            name,
            commander_id: self.commander_id,
            partner_commander_id: self.partner_commander_id,
            background_id: self.background_id,
            signature_spell_id: self.signature_spell_id,
            format,
            tags,
            user_id: self.user_id,
            email_verified: self.email_verified,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn builder(tags: Vec<&str>) -> CreateDeckProfileBuilder {
        CreateDeckProfile::builder("My Deck", Uuid::new_v4(), true)
            .tags(tags.into_iter().map(str::to_string).collect())
    }

    #[test]
    fn parses_and_dedupes_tags() {
        let req = builder(vec!["aggro", "tokens", "aggro"]).build().unwrap();
        assert_eq!(req.tags, vec![DeckTag::Aggro, DeckTag::Tokens]);
    }

    #[test]
    fn rejects_unknown_tag() {
        assert!(matches!(
            builder(vec!["aggro", "not_a_tag"]).build(),
            Err(InvalidCreateDeckProfile::DeckTag(_))
        ));
    }

    #[test]
    fn rejects_too_many_tags() {
        let res = builder(vec![
            "aggro", "control", "tokens", "burn", "mill", "stax",
        ])
        .build();
        assert!(matches!(res, Err(InvalidCreateDeckProfile::TooManyTags)));
    }
}

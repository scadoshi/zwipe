//! Update deck profile operation.

use crate::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use crate::domain::deck::{
    DeckName, DeckOtherTag, DeckTag, InvalidDeckname, InvalidDeckOtherTag, InvalidDeckTag,
    InvalidPowerLevel, MAX_DECK_OTHER_TAGS, MAX_DECK_TAGS, PowerLevel,
    deck_other_tag::parse_other_tags,
    deck_tag::parse_tags,
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
    /// A tag string is not a recognized deck tag.
    #[error(transparent)]
    DeckTag(InvalidDeckTag),
    /// More than [`MAX_DECK_TAGS`] tags were supplied.
    #[error("a deck may have at most {MAX_DECK_TAGS} tags")]
    TooManyTags,
    /// Power level string is not a recognized power level.
    #[error(transparent)]
    PowerLevel(InvalidPowerLevel),
    /// An other-tag string is not a recognized other-tag.
    #[error(transparent)]
    DeckOtherTag(InvalidDeckOtherTag),
    /// More than [`MAX_DECK_OTHER_TAGS`] other-tags were supplied.
    #[error("a deck may have at most {MAX_DECK_OTHER_TAGS} other-tags")]
    TooManyOtherTags,
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

impl From<InvalidDeckTag> for InvalidUpdateDeckProfile {
    fn from(value: InvalidDeckTag) -> Self {
        Self::DeckTag(value)
    }
}

impl From<InvalidPowerLevel> for InvalidUpdateDeckProfile {
    fn from(value: InvalidPowerLevel) -> Self {
        Self::PowerLevel(value)
    }
}

impl From<InvalidDeckOtherTag> for InvalidUpdateDeckProfile {
    fn from(value: InvalidDeckOtherTag) -> Self {
        Self::DeckOtherTag(value)
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
    /// Optional tags update. `Some` replaces the deck's full tag set (an empty
    /// vec clears all tags); `None` leaves tags untouched.
    pub tags: Option<Vec<DeckTag>>,
    /// Optional power level update. `Some(None)` clears it; `None` leaves it
    /// untouched.
    pub power_level: Option<Option<PowerLevel>>,
    /// Optional other-tags update. `Some` replaces the full set (empty clears
    /// all); `None` leaves them untouched.
    pub other_tags: Option<Vec<DeckOtherTag>>,
    /// Optional land target update. `Some(None)` clears the override (back to
    /// the format heuristic); `None` leaves it untouched.
    pub land_target: Option<Option<i32>>,
    /// Optional price target update. `Some(None)` clears the budget; `None`
    /// leaves it untouched.
    pub price_target: Option<Option<f64>>,
    /// Optional price target currency update.
    pub price_target_currency: Option<Option<PriceCurrency>>,
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
            tags: None,
            power_level: None,
            other_tags: None,
            land_target: None,
            price_target: None,
            price_target_currency: None,
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
    tags: Option<Option<Vec<String>>>,
    power_level: Option<Option<String>>,
    other_tags: Option<Option<Vec<String>>>,
    land_target: Option<Option<i32>>,
    price_target: Option<Option<f64>>,
    price_target_currency: Option<Option<PriceCurrency>>,
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

    /// Sets the tags update. Outer `Some` means "update tags"; the inner value
    /// is the new full set (`None`/empty clears all tags). `None` leaves tags
    /// untouched.
    pub fn tags(mut self, tags: Option<Option<Vec<String>>>) -> Self {
        self.tags = tags;
        self
    }

    /// Sets the power level update. Outer `Some` means "update"; inner `None`
    /// clears it. `None` leaves it untouched.
    pub fn power_level(mut self, power_level: Option<Option<&str>>) -> Self {
        self.power_level = power_level.map(|opt| opt.map(|s| s.to_string()));
        self
    }

    /// Sets the other-tags update. Outer `Some` means "update"; the inner value
    /// is the new full set (`None`/empty clears all). `None` leaves them untouched.
    pub fn other_tags(mut self, other_tags: Option<Option<Vec<String>>>) -> Self {
        self.other_tags = other_tags;
        self
    }

    /// Sets the land target update. Outer `Some` means "update"; inner `None`
    /// clears the override. `None` leaves it untouched.
    pub fn land_target(mut self, land_target: Option<Option<i32>>) -> Self {
        self.land_target = land_target;
        self
    }

    /// Sets the price target update. Outer `Some` means "update"; inner `None`
    /// clears the budget. `None` leaves it untouched.
    pub fn price_target(mut self, price_target: Option<Option<f64>>) -> Self {
        self.price_target = price_target;
        self
    }

    /// Sets the price target currency update.
    pub fn price_target_currency(
        mut self,
        price_target_currency: Option<Option<PriceCurrency>>,
    ) -> Self {
        self.price_target_currency = price_target_currency;
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
            && self.tags.is_none()
            && self.power_level.is_none()
            && self.other_tags.is_none()
            && self.land_target.is_none()
            && self.price_target.is_none()
            && self.price_target_currency.is_none()
        {
            return Err(InvalidUpdateDeckProfile::NoUpdates);
        }
        let name = self.name.as_deref().map(DeckName::new).transpose()?;
        let format = self
            .format
            .map(|update| update.as_deref().map(Format::try_from).transpose())
            .transpose()?;
        let tags = match self.tags {
            None => None,
            Some(raw) => {
                let parsed = parse_tags(&raw.unwrap_or_default())?;
                if parsed.len() > MAX_DECK_TAGS {
                    return Err(InvalidUpdateDeckProfile::TooManyTags);
                }
                Some(parsed)
            }
        };
        let power_level = self
            .power_level
            .map(|update| update.as_deref().map(PowerLevel::try_from).transpose())
            .transpose()?;
        let other_tags = match self.other_tags {
            None => None,
            Some(raw) => {
                let parsed = parse_other_tags(&raw.unwrap_or_default())?;
                if parsed.len() > MAX_DECK_OTHER_TAGS {
                    return Err(InvalidUpdateDeckProfile::TooManyOtherTags);
                }
                Some(parsed)
            }
        };

        Ok(UpdateDeckProfile {
            deck_id: self.deck_id,
            name,
            commander_id: self.commander_id,
            partner_commander_id: self.partner_commander_id,
            background_id: self.background_id,
            signature_spell_id: self.signature_spell_id,
            format,
            tags,
            power_level,
            other_tags,
            land_target: self.land_target,
            price_target: self.price_target,
            price_target_currency: self.price_target_currency,
            user_id: self.user_id,
        })
    }
}

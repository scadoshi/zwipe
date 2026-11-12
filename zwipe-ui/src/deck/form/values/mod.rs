//! What the deck form holds and what it sends, as the app's Create and Edit screens do it: the values, Create's request, Edit's sparse update against the deck as saved, the knock-on clears a format or commander change makes, and which command-zone slots show.

use std::collections::HashSet;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::commander_eligibility;
use zwipe_core::domain::deck::{DeckOtherTag, DeckProfile, Format, PowerLevel};
use zwipe_core::http::contracts::deck::{HttpCreateDeckProfile, HttpUpdateDeckProfile};
use zwipe_core::http::helpers::Patch;

use super::{follow_land_target, parse_price, price_text};
use crate::deck::command_zone::{self, Slot};

/// Every field the Create and Edit forms hold.
///
/// Edit keeps a second copy, the values as last loaded, and sends only what differs from it ([`FormValues::update_request`]). The command zone holds whole cards, which the form needs to decide which slots show, and Edit compares them whole, as the app does.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FormValues {
    /// The deck name as typed. Checked with `DeckName::new` before a save.
    pub name: String,
    /// The format, if picked.
    pub format: Option<Format>,
    /// The commander, or the oathbreaker.
    pub commander: Option<Card>,
    /// The commander's partner.
    pub partner: Option<Card>,
    /// The commander's background.
    pub background: Option<Card>,
    /// The oathbreaker's signature spell.
    pub signature_spell: Option<Card>,
    /// Deck tag slugs, in pick order.
    pub tags: Vec<String>,
    /// Oracle tag slugs, in pick order.
    pub oracle_tags: Vec<String>,
    /// Other tags, in pick order.
    pub other_tags: Vec<DeckOtherTag>,
    /// The power level, if picked.
    pub power_level: Option<PowerLevel>,
    /// The land target. `None` is Not set, which falls back to the format's usual count.
    pub land_target: Option<i32>,
    /// The price box's text. Empty, zero, negative or unparsable is no budget ([`parse_price`]).
    pub price_text: String,
    /// The price target's currency.
    pub currency: PriceCurrency,
}

impl FormValues {
    /// The oracle ids of the commander, partner, background and signature
    /// spell picked so far, which every slot's Swipe select leaves out.
    #[must_use]
    pub fn command_zone_oracle_ids(&self) -> HashSet<Uuid> {
        command_zone::zone_oracle_ids([
            self.commander.as_ref(),
            self.partner.as_ref(),
            self.background.as_ref(),
            self.signature_spell.as_ref(),
        ])
    }

    /// A saved deck's values before its command-zone cards load: every field but the command zone, which each card fills as it arrives ([`FormValues::slot_mut`]). A missing currency reads as USD.
    #[must_use]
    pub fn saved(profile: &DeckProfile) -> Self {
        Self {
            name: profile.name.to_string(),
            format: profile.format,
            commander: None,
            partner: None,
            background: None,
            signature_spell: None,
            tags: profile.tags.clone(),
            oracle_tags: profile.oracle_tags.clone(),
            other_tags: profile.other_tags.clone(),
            power_level: profile.power_level,
            land_target: profile.land_target,
            price_text: profile.price_target.map_or_default(price_text),
            currency: profile.price_target_currency.unwrap_or(PriceCurrency::Usd),
        }
    }

    /// The card in `slot`.
    #[must_use]
    pub fn slot(&self, slot: Slot) -> Option<&Card> {
        match slot {
            Slot::Commander => self.commander.as_ref(),
            Slot::Partner => self.partner.as_ref(),
            Slot::Background => self.background.as_ref(),
            Slot::SignatureSpell => self.signature_spell.as_ref(),
        }
    }

    /// The card in `slot`, to fill it with no knock-on clear, as a load does. A commander the user picks goes through [`FormValues::set_commander`].
    pub fn slot_mut(&mut self, slot: Slot) -> &mut Option<Card> {
        match slot {
            Slot::Commander => &mut self.commander,
            Slot::Partner => &mut self.partner,
            Slot::Background => &mut self.background,
            Slot::SignatureSpell => &mut self.signature_spell,
        }
    }

    /// Whether `slot`'s field shows, from the format and the commander ([`shows`]).
    #[must_use]
    pub fn shows(&self, slot: Slot) -> bool {
        shows(slot, self.format, self.commander.as_ref())
    }

    /// Picks `format` from the format picker: the commander goes, and the signature spell unless the new format has one. Clearing a commander that was set also clears its partner and background ([`FormValues::set_commander`]), and every Filter goes back on. A land target still at the old format's usual count moves to the new one's ([`follow_land_target`]).
    pub fn pick_format(&mut self, format: Format) -> Cascade {
        self.land_target = follow_land_target(self.land_target, self.format, Some(format));
        self.format = Some(format);
        let mut cleared = vec![Slot::Commander];
        if !format.has_signature_spell() {
            cleared.push(Slot::SignatureSpell);
        }
        self.clear_slots(cleared)
    }

    /// Clears the format: the commander and the signature spell go, then the partner and background as for [`FormValues::pick_format`], and every Filter goes back on.
    pub fn clear_format(&mut self) -> Cascade {
        self.format = None;
        self.clear_slots(vec![Slot::Commander, Slot::SignatureSpell])
    }

    /// Empties `cleared` for a format change, the commander through [`FormValues::set_commander`].
    fn clear_slots(&mut self, cleared: Vec<Slot>) -> Cascade {
        let reset = self.set_commander(None).reset;
        for &slot in &cleared {
            *self.slot_mut(slot) = None;
        }
        Cascade {
            cleared,
            reset,
            filters_on: true,
        }
    }

    /// Sets the commander. A change from one commander to another, or to none, clears the partner and the background, which depend on the commander's abilities ([`clears_mates`]). Filling an empty slot never does, so a load that brings the partner in before the commander keeps it.
    pub fn set_commander(&mut self, card: Option<Card>) -> Cascade {
        let previous = self.commander.as_ref().map(|c| c.scryfall_data.id);
        let current = card.as_ref().map(|c| c.scryfall_data.id);
        self.commander = card;
        if !clears_mates(previous, current) {
            return Cascade::default();
        }
        self.partner = None;
        self.background = None;
        Cascade {
            cleared: Vec::new(),
            reset: vec![Slot::Partner, Slot::Background],
            filters_on: false,
        }
    }

    /// Create's request: empty tag lists go as absent, a budget of zero or less (or one that does not parse) goes as absent, and the currency goes only with a budget. The name is checked before this.
    #[must_use]
    pub fn create_request(&self) -> HttpCreateDeckProfile {
        let price = parse_price(&self.price_text);
        HttpCreateDeckProfile::builder(&self.name)
            .commander_id(id(self.commander.as_ref()))
            .partner_commander_id(id(self.partner.as_ref()))
            .background_id(id(self.background.as_ref()))
            .signature_spell_id(id(self.signature_spell.as_ref()))
            .format(self.format.map(legality_key))
            .tags(non_empty(self.tags.clone()))
            .power_level(self.power_level.map(|p| p.to_string()))
            .other_tags(non_empty(other_tag_strings(&self.other_tags)))
            .oracle_tags(non_empty(self.oracle_tags.clone()))
            .land_target(self.land_target)
            .price_target(price)
            .price_target_currency(price.map(|_| self.currency))
            .build()
    }

    /// Edit's request: each field that differs from `saved`, set to its new value or cleared, with the rest left off the wire. `None` when nothing differs, which hides Save.
    ///
    /// Tags compare in order and the command zone compares whole cards. The budget compares as parsed, so `50` and `50.0` are the same and emptying the box sends a clear. A changed currency always sends its value.
    #[must_use]
    pub fn update_request(&self, saved: &Self) -> Option<HttpUpdateDeckProfile> {
        let name = (self.name != saved.name).then_some(self.name.as_str());
        let commander = patch(&saved.commander, &self.commander, || {
            id(self.commander.as_ref())
        });
        let partner = patch(&saved.partner, &self.partner, || id(self.partner.as_ref()));
        let background = patch(&saved.background, &self.background, || {
            id(self.background.as_ref())
        });
        let signature_spell = patch(&saved.signature_spell, &self.signature_spell, || {
            id(self.signature_spell.as_ref())
        });
        let format = patch(&saved.format, &self.format, || {
            self.format.map(legality_key)
        });
        let tags = patch(&saved.tags, &self.tags, || Some(self.tags.clone()));
        let land_target = patch(&saved.land_target, &self.land_target, || self.land_target);
        let price_now = parse_price(&self.price_text);
        let price = patch(&parse_price(&saved.price_text), &price_now, || price_now);
        let currency = patch(&saved.currency, &self.currency, || Some(self.currency));
        let power_level = patch(&saved.power_level, &self.power_level, || {
            self.power_level.map(|p| p.to_string())
        });
        let other_tags = patch(&saved.other_tags, &self.other_tags, || {
            Some(other_tag_strings(&self.other_tags))
        });
        let oracle_tags = patch(&saved.oracle_tags, &self.oracle_tags, || {
            Some(self.oracle_tags.clone())
        });
        let changed = name.is_some()
            || [
                commander.is_changed(),
                partner.is_changed(),
                background.is_changed(),
                signature_spell.is_changed(),
                format.is_changed(),
                tags.is_changed(),
                land_target.is_changed(),
                price.is_changed(),
                currency.is_changed(),
                power_level.is_changed(),
                other_tags.is_changed(),
                oracle_tags.is_changed(),
            ]
            .contains(&true);
        changed.then(|| {
            HttpUpdateDeckProfile::builder()
                .name(name)
                .commander_id(commander)
                .partner_commander_id(partner)
                .background_id(background)
                .signature_spell_id(signature_spell)
                .format(format)
                .tags(tags)
                .land_target(land_target)
                .price_target(price)
                .price_target_currency(currency)
                .power_level(power_level)
                .other_tags(other_tags)
                .oracle_tags(oracle_tags)
                .build()
        })
    }
}

/// What a knock-on clear emptied beside the values, so each client also resets what it shows for those slots.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cascade {
    /// Slots whose card went. A client empties the name the slot's input shows; a typed search stays, as in the app.
    pub cleared: Vec<Slot>,
    /// Slots emptied whole: the card, the name shown, the typed search and its results.
    pub reset: Vec<Slot>,
    /// Whether every slot's Filter goes back on. True for any format change. The app turns them on whenever the format changes, from any source, which also covers a load and a Cancel that puts the old format back.
    pub filters_on: bool,
}

/// Whether `slot`'s field shows on the form. The commander shows in a format with one; the partner and the background show under it when the commander can take one; the signature spell shows in a format with one.
#[must_use]
pub fn shows(slot: Slot, format: Option<Format>, commander: Option<&Card>) -> bool {
    let commander_shows = format.is_some_and(|f| f.has_commander());
    match slot {
        Slot::Commander => commander_shows,
        Slot::Partner => {
            commander_shows
                && commander.is_some_and(|c| commander_eligibility::partner_kind(c).is_some())
        }
        Slot::Background => {
            commander_shows && commander.is_some_and(commander_eligibility::has_choose_a_background)
        }
        Slot::SignatureSpell => format.is_some_and(|f| f.has_signature_spell()),
    }
}

/// Whether a commander change from `previous` to `current` (printing ids) clears the partner and the background: a change from one commander to another, or to none. Filling an empty slot never clears, since the edit screen loads its cards in any order, and setting the same commander again keeps them.
#[must_use]
pub fn clears_mates(previous: Option<Uuid>, current: Option<Uuid>) -> bool {
    previous.is_some() && previous != current
}

/// One field's patch: unchanged when `current` equals `saved`, else set to `new()`.
fn patch<T: PartialEq, U>(saved: &T, current: &T, new: impl FnOnce() -> Option<U>) -> Patch<U> {
    if saved == current {
        Patch::Unchanged
    } else {
        Patch::Set(new())
    }
}

/// A card's printing id.
fn id(card: Option<&Card>) -> Option<Uuid> {
    card.map(|c| c.scryfall_data.id)
}

/// The format as the server stores it.
fn legality_key(format: Format) -> String {
    format.to_legality_key().to_owned()
}

/// Other tags as the server takes them.
fn other_tag_strings(tags: &[DeckOtherTag]) -> Vec<String> {
    tags.iter().map(ToString::to_string).collect()
}

/// `Some(v)` for a non-empty list, `None` for an empty one.
fn non_empty<T>(v: Vec<T>) -> Option<Vec<T>> {
    (!v.is_empty()).then_some(v)
}

#[cfg(test)]
mod tests;

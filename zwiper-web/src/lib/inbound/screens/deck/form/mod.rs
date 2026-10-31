//! The deck form Create and Edit share: its state, the fields, the four
//! command-zone searches, the format, deck tag and oracle tag pickers, and
//! Swipe select for each command-zone slot.
//!
//! On a wide window the form splits into two columns, `Profile` on the left
//! and `Budget` above `Tags` on the right, and the pickers open as a side
//! panel over the right column so the form's cascades show as they happen.
//! Below the breakpoint it is the app's form with full-screen pickers.

pub mod command_zone;
pub mod fields;
pub mod overlays;
pub mod pickers;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{DeckName, DeckOtherTag, DeckProfile, Format, PowerLevel};
use zwipe_ui::deck::form::Slot;

use crate::domain::deck::FormValues;

/// One command-zone slot: the picked card, the text its input shows, the
/// search typed, the slot's `Filter` toggle and its Swipe select.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlotState {
    /// The picked card.
    pub card: Signal<Option<Card>>,
    /// What the input shows: the picked card's name, or what is typed.
    pub text: Signal<String>,
    /// The search typed; the results float while it is long enough.
    pub query: Signal<String>,
    /// Whether the slot's rule narrows the search. On by default.
    pub filter_on: Signal<bool>,
    /// Whether the slot's Swipe select is open.
    pub swipe_open: Signal<bool>,
}

impl SlotState {
    fn new() -> Self {
        Self {
            card: Signal::new(None),
            text: Signal::new(String::new()),
            query: Signal::new(String::new()),
            filter_on: Signal::new(true),
            swipe_open: Signal::new(false),
        }
    }

    /// Picks `card`, showing its name.
    pub fn pick(mut self, card: Card) {
        self.text.set(card.scryfall_data.name.clone());
        self.card.set(Some(card));
    }

    /// Clears the card and its text, leaving the search as it is.
    pub fn clear_card(mut self) {
        self.card.set(None);
        self.text.set(String::new());
    }

    /// Clears the card, its text and the search, which hides the results.
    pub fn clear_all(mut self) {
        self.clear_card();
        self.query.set(String::new());
    }

    /// The picked card's printing id.
    fn id(self) -> Option<Uuid> {
        self.card.read().as_ref().map(|c| c.scryfall_data.id)
    }
}

/// The whole form, `Copy`, so fields, pickers and overlays share it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeckForm {
    /// The deck name as typed.
    pub name: Signal<String>,
    /// The name's error line, set once the name has been typed in.
    pub name_error: Signal<Option<String>>,
    /// The format.
    pub format: Signal<Option<Format>>,
    /// The commander, or the oathbreaker.
    pub commander: SlotState,
    /// The partner.
    pub partner: SlotState,
    /// The background.
    pub background: SlotState,
    /// The signature spell.
    pub signature_spell: SlotState,
    /// The power level.
    pub power_level: Signal<Option<PowerLevel>>,
    /// The land target; `None` is Not set.
    pub land_target: Signal<Option<i32>>,
    /// The price box's text.
    pub price_text: Signal<String>,
    /// The price target's currency.
    pub currency: Signal<PriceCurrency>,
    /// Deck tag slugs.
    pub tags: Signal<Vec<String>>,
    /// The oracle tags the deck tags added last, which a new deck tag pick
    /// swaps out while keeping hand picks.
    pub applied_seed: Signal<Vec<String>>,
    /// Oracle tag slugs.
    pub oracle_tags: Signal<Vec<String>>,
    /// Other tags.
    pub other_tags: Signal<Vec<DeckOtherTag>>,
    /// Whether the format picker is open.
    pub format_open: Signal<bool>,
    /// Whether the deck tag picker is open.
    pub tags_open: Signal<bool>,
    /// Whether the oracle tag picker is open.
    pub oracle_tags_open: Signal<bool>,
}

impl DeckForm {
    /// The state of `slot`.
    pub fn slot(self, slot: Slot) -> SlotState {
        match slot {
            Slot::Commander => self.commander,
            Slot::Partner => self.partner,
            Slot::Background => self.background,
            Slot::SignatureSpell => self.signature_spell,
        }
    }

    /// The form's values, read so a memo over them follows every edit.
    pub fn values(self) -> FormValues {
        FormValues {
            name: self.name.read().clone(),
            format: *self.format.read(),
            commander: self.commander.id(),
            partner: self.partner.id(),
            background: self.background.id(),
            signature_spell: self.signature_spell.id(),
            tags: self.tags.read().clone(),
            land_target: *self.land_target.read(),
            price_text: self.price_text.read().clone(),
            currency: *self.currency.read(),
            power_level: *self.power_level.read(),
            other_tags: self.other_tags.read().clone(),
            oracle_tags: self.oracle_tags.read().clone(),
        }
    }

    /// Fills every field but the command zone from a saved deck. The deck
    /// tags' compiled seed counts as applied, so the next deck tag pick swaps
    /// out exactly those oracle tags.
    pub fn fill(mut self, profile: &DeckProfile) {
        let saved = FormValues::saved(profile);
        self.name.set(saved.name);
        self.format.set(saved.format);
        self.tags.set(saved.tags);
        self.land_target.set(saved.land_target);
        self.price_text.set(saved.price_text);
        self.currency.set(saved.currency);
        self.power_level.set(saved.power_level);
        self.other_tags.set(saved.other_tags);
        self.oracle_tags.set(saved.oracle_tags);
        self.applied_seed
            .set(zwipe_core::domain::deck::seed_oracle_tags(&profile.tags));
    }

    /// Sets the format from the picker: the commander goes, and the
    /// signature spell unless the new format has one.
    pub fn select_format(mut self, format: Format) {
        self.format.set(Some(format));
        self.commander.clear_card();
        if !format.has_signature_spell() {
            self.signature_spell.clear_card();
        }
    }

    /// Clears the format, the commander and the signature spell.
    pub fn clear_format(mut self) {
        self.format.set(None);
        self.commander.clear_card();
        self.signature_spell.clear_card();
    }
}

/// An empty form with its rules running: the name checks itself once typed
/// in, a format change turns every `Filter` back on, and a commander change
/// (not the first load) clears the partner and the background.
pub fn use_deck_form() -> DeckForm {
    let form = use_hook(|| DeckForm {
        name: Signal::new(String::new()),
        name_error: Signal::new(None),
        format: Signal::new(None),
        commander: SlotState::new(),
        partner: SlotState::new(),
        background: SlotState::new(),
        signature_spell: SlotState::new(),
        power_level: Signal::new(None),
        land_target: Signal::new(None),
        price_text: Signal::new(String::new()),
        currency: Signal::new(PriceCurrency::Usd),
        tags: Signal::new(Vec::new()),
        applied_seed: Signal::new(Vec::new()),
        oracle_tags: Signal::new(Vec::new()),
        other_tags: Signal::new(Vec::new()),
        format_open: Signal::new(false),
        tags_open: Signal::new(false),
        oracle_tags_open: Signal::new(false),
    });

    let mut touched = use_signal(|| false);
    use_effect(move || {
        let value = (form.name)();
        if !value.is_empty() && !*touched.peek() {
            touched.set(true);
        }
        if *touched.peek() {
            let mut error = form.name_error;
            error.set(DeckName::new(value).err().map(|e| e.to_string()));
        }
    });

    use_effect(move || {
        let _format = (form.format)();
        for slot in Slot::ALL {
            let mut on = form.slot(slot).filter_on;
            on.set(true);
        }
    });

    // The edit screen loads its cards in any order, so only a change from
    // one commander to another, or to none, clears the commander's mates.
    let mut previous: Signal<Option<Uuid>> = use_signal(|| None);
    use_effect(move || {
        let current = form
            .commander
            .card
            .read()
            .as_ref()
            .map(|c| c.scryfall_data.id);
        let before = *previous.peek();
        previous.set(current);
        if before.is_none() || before == current {
            return;
        }
        form.partner.clear_all();
        form.background.clear_all();
    });

    form
}

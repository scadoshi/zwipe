//! Create deck and Edit deck: the app's shared deck form, its command-zone searches and cascades, and the format, deck tag, oracle tag and swipe pickers over it.

pub(crate) mod draw;
pub(crate) mod keys;
pub(crate) mod pickers;
pub(crate) mod swipe_pick;
#[cfg(test)]
mod tests;

use std::time::Duration;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::commander_eligibility::{self, PartnerKind};
use zwipe_core::domain::deck::{
    Deck, DeckName, DeckOtherTag, DeckProfile, DeckTagView, Format, PowerLevel, seed_oracle_tags,
    seed_oracle_tags_from_catalog,
};
use zwipe_core::http::contracts::deck::{HttpCreateDeckProfile, HttpUpdateDeckProfile};
use zwipe_core::http::helpers::Patch;

use self::pickers::{FormatPicker, OraclePicker, TagPicker};
use self::swipe_pick::SwipePicker;
use super::{Screen, card_sheets};
use crate::domain::deck::form::{self, Slot};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::info::InfoPopup;
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::outbound::account::Account;

/// How long typing must pause before a command-zone search goes out, as in the app.
pub(crate) const SEARCH_DEBOUNCE: Duration = Duration::from_millis(800);

/// The named-partner search returns at most this many cards.
const PARTNER_SEARCH_LIMIT: u32 = 5;

/// Whether the form makes a new deck or changes one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Create,
    Edit { deck_id: Uuid },
}

impl Mode {
    /// The screen's title.
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Create => "Create deck",
            Self::Edit { .. } => "Edit deck",
        }
    }

    /// The key the server stores this form's hint under.
    pub(crate) fn hint_key(self) -> &'static str {
        match self {
            Self::Create => "create_deck",
            Self::Edit { .. } => "edit_deck",
        }
    }
}

/// What a command-zone field's results layer shows.
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) enum Dropdown {
    #[default]
    Hidden,
    Searching,
    /// The hits, or "No results" when empty.
    Results(Vec<Card>),
}

/// One command-zone field: the chosen card, what is typed, its Filter toggle, and its search.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CardField {
    pub(crate) card: Option<Card>,
    /// The chosen card's name, or the query being typed.
    pub(crate) input: TextInput,
    pub(crate) filter_on: bool,
    /// Counts searches so a reply to an older one is dropped.
    pub(crate) generation: u64,
    pub(crate) dropdown: Dropdown,
    /// The highlighted hit.
    pub(crate) highlighted: usize,
    /// The filter the swipe picker applied last, kept while the picker is closed.
    pub(crate) swipe_filter: CardQueryBuilder,
}

impl Default for CardField {
    fn default() -> Self {
        Self {
            card: None,
            input: TextInput::default().with_placeholder("Not set"),
            filter_on: true,
            generation: 0,
            dropdown: Dropdown::Hidden,
            highlighted: 0,
            swipe_filter: CardQueryBuilder::new(),
        }
    }
}

impl CardField {
    /// Sets the chosen card and shows its name, closing the results.
    fn choose(&mut self, card: Option<Card>) {
        let name = card
            .as_ref()
            .map(|c| c.scryfall_data.name.clone())
            .unwrap_or_default();
        self.input.set_value(&name);
        self.card = card;
        self.dropdown = Dropdown::Hidden;
        self.generation = self.generation.saturating_add(1);
    }

    pub(crate) fn card_id(&self) -> Option<Uuid> {
        self.card.as_ref().map(|c| c.scryfall_data.id)
    }

    /// The results shown, when there are some.
    pub(crate) fn hits(&self) -> &[Card] {
        match &self.dropdown {
            Dropdown::Results(cards) => cards,
            _ => &[],
        }
    }
}

/// A focusable field, top to bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Field {
    Name,
    Format,
    Card(Slot),
    Power,
    Land,
    Currency,
    Price,
    DeckTags,
    OracleTags,
    OtherTags,
}

impl Field {
    /// The field's "?" explainer, as the app's info buttons word it.
    pub(crate) fn info(self) -> InfoPopup {
        match self {
            Self::Name => InfoPopup::new(
                "Deck name",
                &["Shown in your deck list. Change it any time"],
            ),
            Self::Format => InfoPopup::new(
                "Format",
                &[
                    "Sets deck size, card pool and the command zone",
                    "Changing it clears the commander and signature spell",
                ],
            ),
            Self::Card(Slot::Commander) => InfoPopup::new(
                "Commander",
                &[
                    "Search only shows cards that can lead this format. Filter searches any card",
                    "Swipe picks one by swiping",
                ],
            ),
            Self::Card(Slot::Partner) => {
                InfoPopup::new("Partner", &["Shows when the commander allows a partner"])
            }
            Self::Card(Slot::Background) => InfoPopup::new(
                "Background",
                &["Shows when the commander can take a background"],
            ),
            Self::Card(Slot::SignatureSpell) => InfoPopup::new(
                "Signature spell",
                &["Oathbreaker only: the instant or sorcery that starts in the command zone"],
            ),
            Self::Power => InfoPopup::new(
                "Power level",
                &["The Commander bracket the deck aims for, 1 to 5"],
            ),
            Self::Land => InfoPopup::new(
                "Land target",
                &[
                    "How many lands the deck should run. Fewer warns on the deck",
                    "Not set uses the format's usual count",
                ],
            ),
            Self::Currency | Self::Price => InfoPopup::new(
                "Price target",
                &["A budget for the mainboard. Going over warns on the deck"],
            ),
            Self::DeckTags => InfoPopup::new(
                "Deck tags",
                &[
                    "The archetypes the deck is built around, like Aristocrats or Voltron",
                    "Picking one pre-selects the oracle tags that define it",
                    "Up to 5",
                ],
            ),
            Self::OracleTags => InfoPopup::new(
                "Oracle tags",
                &[
                    "The specific things the deck does, like spot removal, ramp or reanimation",
                    "They shape which cards we suggest",
                    "Your deck tags pre-pick a starter set from about 4,500. Fine to leave as is",
                ],
            ),
            Self::OtherTags => InfoPopup::new(
                "Other tags",
                &["Labels that do not change suggestions, like Budget or Jank"],
            ),
        }
    }
}

/// A full-screen picker or popup over the form.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Overlay {
    Format(Box<FormatPicker>),
    DeckTags(TagPicker),
    OracleTags(OraclePicker),
    Swipe(Box<SwipePicker>),
    Info(InfoPopup),
}

/// The values an edit started from, to send only what changed.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Original {
    pub(crate) name: String,
    pub(crate) format: Option<Format>,
    /// Each slot's card once it has loaded.
    pub(crate) cards: [Option<Uuid>; 4],
    pub(crate) power: Option<PowerLevel>,
    pub(crate) land_target: Option<i32>,
    pub(crate) price: String,
    pub(crate) currency: PriceCurrency,
    pub(crate) tags: Vec<String>,
    pub(crate) oracle_tags: Vec<String>,
    pub(crate) other_tags: Vec<DeckOtherTag>,
}

/// The Create and Edit form's state: every field, which one has the focus, the overlay open over it, and whether a load or save is in flight.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DeckForm {
    pub(crate) mode: Mode,
    /// Edit only: the deck is still loading.
    pub(crate) loading: bool,
    /// Edit only: why the deck failed to load.
    pub(crate) load_error: Option<String>,
    pub(crate) saving: bool,
    pub(crate) name: TextInput,
    /// Whether the name has held text, after which every change is checked.
    pub(crate) name_touched: bool,
    pub(crate) format: Option<Format>,
    /// The four command-zone fields, in [`Slot::ALL`] order.
    pub(crate) cards: [CardField; 4],
    pub(crate) power: Option<PowerLevel>,
    pub(crate) power_focus: usize,
    pub(crate) land_target: Option<i32>,
    pub(crate) currency: PriceCurrency,
    pub(crate) price: TextInput,
    pub(crate) tags: Vec<String>,
    pub(crate) tag_focus: usize,
    pub(crate) oracle_tags: Vec<String>,
    pub(crate) oracle_focus: usize,
    pub(crate) other_tags: Vec<DeckOtherTag>,
    pub(crate) other_focus: usize,
    /// The oracle tags the deck tags seeded last, swapped out when the deck tags change.
    pub(crate) applied_seed: Vec<String>,
    pub(crate) deck_tag_catalog: Vec<DeckTagView>,
    /// The oracle tag catalog, `None` until it loads.
    pub(crate) oracle_catalog: Option<Vec<OracleTag>>,
    pub(crate) focus: Field,
    pub(crate) original: Option<Original>,
    pub(crate) overlay: Option<Overlay>,
}

/// The index of `slot` in the form's card fields.
pub(crate) fn slot_index(slot: Slot) -> usize {
    match slot {
        Slot::Commander => 0,
        Slot::Partner => 1,
        Slot::Background => 2,
        Slot::SignatureSpell => 3,
    }
}

impl DeckForm {
    /// Comes back into view: takes the tag the oracle tag dictionary's Use left, into the swipe picker's filter sheet or the oracle tag picker, whichever opened it.
    pub(crate) fn on_return(&mut self, cx: &mut Context<'_>) {
        if let Some(Overlay::Swipe(picker)) = &mut self.overlay
            && card_sheets::filter::returned_from_dictionary(picker.sheet.as_mut(), cx)
        {
            return;
        }
        let Some(slug) = cx.shared.caches.handoff.oracle_tag.take() else {
            return;
        };
        if matches!(self.overlay, Some(Overlay::OracleTags(_))) {
            pickers::adopt_dictionary_tag(self, slug, cx);
        }
    }

    pub(crate) fn new(mode: Mode) -> Self {
        Self {
            mode,
            loading: matches!(mode, Mode::Edit { .. }),
            load_error: None,
            saving: false,
            name: TextInput::default().with_placeholder("Not set"),
            name_touched: false,
            format: None,
            cards: Default::default(),
            power: None,
            power_focus: 0,
            land_target: None,
            currency: PriceCurrency::Usd,
            price: TextInput::default().with_placeholder("Not set"),
            tags: Vec::new(),
            tag_focus: 0,
            oracle_tags: Vec::new(),
            oracle_focus: 0,
            other_tags: Vec::new(),
            other_focus: 0,
            applied_seed: Vec::new(),
            deck_tag_catalog: Vec::new(),
            oracle_catalog: None,
            focus: Field::Name,
            original: None,
            overlay: None,
        }
    }

    pub(crate) fn card(&self, slot: Slot) -> &CardField {
        let [commander, partner, background, spell] = &self.cards;
        match slot {
            Slot::Commander => commander,
            Slot::Partner => partner,
            Slot::Background => background,
            Slot::SignatureSpell => spell,
        }
    }

    pub(crate) fn card_mut(&mut self, slot: Slot) -> &mut CardField {
        let [commander, partner, background, spell] = &mut self.cards;
        match slot {
            Slot::Commander => commander,
            Slot::Partner => partner,
            Slot::Background => background,
            Slot::SignatureSpell => spell,
        }
    }

    pub(crate) fn commander(&self) -> Option<&Card> {
        self.card(Slot::Commander).card.as_ref()
    }

    /// Whether a slot's field shows, under the app's rules.
    pub(crate) fn shows(&self, slot: Slot) -> bool {
        let commander_shown = self.format.is_some_and(|f| f.has_commander());
        match slot {
            Slot::Commander => commander_shown,
            Slot::Partner => {
                commander_shown
                    && self
                        .commander()
                        .is_some_and(|c| commander_eligibility::partner_kind(c).is_some())
            }
            Slot::Background => {
                commander_shown
                    && self
                        .commander()
                        .is_some_and(commander_eligibility::has_choose_a_background)
            }
            Slot::SignatureSpell => self.format.is_some_and(|f| f.has_signature_spell()),
        }
    }

    /// The fields that show, top to bottom.
    pub(crate) fn fields(&self) -> Vec<Field> {
        let mut fields = vec![Field::Name, Field::Format];
        fields.extend(
            Slot::ALL
                .into_iter()
                .filter(|s| self.shows(*s))
                .map(Field::Card),
        );
        fields.extend([
            Field::Power,
            Field::Land,
            Field::Currency,
            Field::Price,
            Field::DeckTags,
            Field::OracleTags,
            Field::OtherTags,
        ]);
        fields
    }

    /// Moves the focus to the next or the previous field, stopping at the ends.
    pub(crate) fn step_focus(&mut self, toward: Toward) {
        let fields = self.fields();
        let here = fields.iter().position(|f| *f == self.focus).unwrap_or(0);
        let next = toward.clamped(here, fields.len());
        if let Some(field) = fields.get(next) {
            self.focus = *field;
        }
    }

    /// Keeps the focus on a field that shows.
    fn clamp_focus(&mut self) {
        if !self.fields().contains(&self.focus) {
            self.focus = Field::Format;
        }
    }

    /// Checks the name once it has held text, as the app checks it on every change.
    pub(crate) fn check_name(&mut self) {
        if !self.name.value().is_empty() {
            self.name_touched = true;
        }
        if self.name_touched {
            let error = DeckName::new(self.name.value())
                .err()
                .map(|e| e.to_string());
            self.name.set_error(error);
        }
    }

    /// Sets the commander. Changing from one card to another, or to none, clears the partner and background.
    pub(crate) fn set_commander(&mut self, card: Option<Card>) {
        let before = self.card(Slot::Commander).card_id();
        let after = card.as_ref().map(|c| c.scryfall_data.id);
        self.card_mut(Slot::Commander).choose(card);
        if before.is_some() && before != after {
            for slot in [Slot::Partner, Slot::Background] {
                let field = self.card_mut(slot);
                *field = CardField {
                    filter_on: field.filter_on,
                    ..CardField::default()
                };
            }
        }
        self.clamp_focus();
    }

    /// Sets a slot's card, through the commander cascade for the commander.
    pub(crate) fn set_card(&mut self, slot: Slot, card: Option<Card>) {
        if slot == Slot::Commander {
            self.set_commander(card);
        } else {
            self.card_mut(slot).choose(card);
        }
    }

    /// Sets or clears the format: every Filter goes back on, the commander clears, and the signature spell clears unless the new format has one.
    pub(crate) fn set_format(&mut self, format: Option<Format>) {
        self.format = format;
        for field in &mut self.cards {
            field.filter_on = true;
        }
        self.set_commander(None);
        if !format.is_some_and(|f| f.has_signature_spell()) {
            self.card_mut(Slot::SignatureSpell).choose(None);
        }
        self.clamp_focus();
    }

    /// Picks a card for `slot` from a search or a swipe, then looks for a named partner when it is the commander.
    pub(crate) fn pick_card(&mut self, slot: Slot, card: Card, cx: &mut Context<'_>) {
        let named = (slot == Slot::Commander)
            .then(|| commander_eligibility::partner_kind(&card))
            .flatten();
        let commander_id = card.scryfall_data.id;
        self.set_card(slot, Some(card));
        if let Some(PartnerKind::Named(mate)) = named {
            cx.request(Request::FindPartner { commander_id, mate }.into());
        }
    }

    /// Restarts a slot's search for what is typed: hidden below the slot's minimum, else "Searching..." until the pause passes.
    pub(crate) fn search(&mut self, slot: Slot, cx: &mut Context<'_>) {
        let field = self.card_mut(slot);
        field.generation = field.generation.saturating_add(1);
        field.highlighted = 0;
        if field.input.len() < slot.min_chars() {
            field.dropdown = Dropdown::Hidden;
            return;
        }
        field.dropdown = Dropdown::Searching;
        cx.request(
            Request::Debounce {
                slot,
                generation: field.generation,
            }
            .into(),
        );
    }

    /// The query for a slot's search once the pause has passed.
    fn search_query(&self, slot: Slot) -> Option<CardQuery> {
        let field = self.card(slot);
        form::search_query(
            slot,
            field.input.value(),
            field.filter_on,
            self.format,
            self.commander(),
        )
        .ok()
    }

    /// The deck tags after a picker closes: swaps the oracle tags the old ones seeded for those the new ones seed, and says so when any were added.
    pub(crate) fn reseed(&mut self, cx: &mut Context<'_>) {
        let new_seed = seed_oracle_tags_from_catalog(&self.tags, &self.deck_tag_catalog);
        let (oracle_tags, added) =
            form::reseed_oracle_tags(&self.oracle_tags, &self.applied_seed, &new_seed);
        self.oracle_tags = oracle_tags;
        self.applied_seed = new_seed;
        if added {
            cx.toast(
                ToastKind::Info,
                "Oracle tags derived from your deck tags".to_owned(),
            );
        }
    }

    /// Fills the form from a loaded deck and remembers each value as the original.
    pub(crate) fn fill(&mut self, profile: &DeckProfile) {
        self.loading = false;
        self.name.set_value(&profile.name);
        self.format = profile.format;
        self.power = profile.power_level;
        self.land_target = profile.land_target;
        self.currency = profile.price_target_currency.unwrap_or_default();
        self.price.set_value(
            &profile
                .price_target
                .map(form::price_text)
                .unwrap_or_default(),
        );
        self.tags.clone_from(&profile.tags);
        self.oracle_tags.clone_from(&profile.oracle_tags);
        self.other_tags.clone_from(&profile.other_tags);
        self.applied_seed = seed_oracle_tags(&profile.tags);
        self.original = Some(Original {
            name: self.name.value().to_owned(),
            format: self.format,
            cards: [None; 4],
            power: self.power,
            land_target: self.land_target,
            price: self.price.value().to_owned(),
            currency: self.currency,
            tags: self.tags.clone(),
            oracle_tags: self.oracle_tags.clone(),
            other_tags: self.other_tags.clone(),
        });
    }

    /// The edit as the server takes it: only the fields that differ from the original.
    pub(crate) fn changes(&self) -> Option<Changes> {
        let original = self.original.as_ref()?;
        let card_patch = |slot: Slot| {
            let now = self.card(slot).card_id();
            if now == original.cards.get(slot_index(slot)).copied().flatten() {
                Patch::Unchanged
            } else {
                Patch::Set(now)
            }
        };
        let patch = |changed: bool, value| {
            if changed {
                Patch::Set(value)
            } else {
                Patch::Unchanged
            }
        };
        let price = form::parse_price(self.price.value());
        Some(Changes {
            name: (self.name.value() != original.name).then(|| self.name.value().to_owned()),
            commander: card_patch(Slot::Commander),
            partner: card_patch(Slot::Partner),
            background: card_patch(Slot::Background),
            signature_spell: card_patch(Slot::SignatureSpell),
            format: patch(
                self.format != original.format,
                self.format.map(|f| f.to_legality_key().to_owned()),
            ),
            tags: patch_vec(self.tags != original.tags, &self.tags),
            land_target: if self.land_target == original.land_target {
                Patch::Unchanged
            } else {
                Patch::Set(self.land_target)
            },
            price_target: if price == form::parse_price(&original.price) {
                Patch::Unchanged
            } else {
                Patch::Set(price)
            },
            currency: if self.currency == original.currency {
                Patch::Unchanged
            } else {
                Patch::Set(Some(self.currency))
            },
            power: patch(
                self.power != original.power,
                self.power.map(|p| p.to_string()),
            ),
            other_tags: patch_vec(
                self.other_tags != original.other_tags,
                &self
                    .other_tags
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            ),
            oracle_tags: patch_vec(self.oracle_tags != original.oracle_tags, &self.oracle_tags),
        })
    }

    /// Whether an edit has anything to save, which shows "Save changes".
    pub(crate) fn has_changes(&self) -> bool {
        self.changes().is_some_and(|c| c.any())
    }

    /// The new deck as the server takes it.
    pub(crate) fn fields_for_create(&self) -> NewDeck {
        let price = form::parse_price(self.price.value());
        NewDeck {
            name: self.name.value().to_owned(),
            commander: self.card(Slot::Commander).card_id(),
            partner: self.card(Slot::Partner).card_id(),
            background: self.card(Slot::Background).card_id(),
            signature_spell: self.card(Slot::SignatureSpell).card_id(),
            format: self.format.map(|f| f.to_legality_key().to_owned()),
            tags: (!self.tags.is_empty()).then(|| self.tags.clone()),
            power: self.power.map(|p| p.to_string()),
            other_tags: (!self.other_tags.is_empty())
                .then(|| self.other_tags.iter().map(ToString::to_string).collect()),
            oracle_tags: (!self.oracle_tags.is_empty()).then(|| self.oracle_tags.clone()),
            land_target: self.land_target,
            price_target: price,
            currency: price.map(|_| self.currency),
        }
    }

    /// Saves: checks the name, then creates the deck or sends the changes.
    pub(crate) fn save(&mut self, cx: &mut Context<'_>) {
        if self.saving || self.loading || self.load_error.is_some() {
            return;
        }
        if let Err(e) = DeckName::new(self.name.value()) {
            self.name.set_error(Some(e.to_string()));
            self.focus = Field::Name;
            return;
        }
        match self.mode {
            Mode::Create => {
                self.saving = true;
                cx.request(Request::Create(Box::new(self.fields_for_create())).into());
            }
            Mode::Edit { deck_id } => {
                let Some(changes) = self.changes().filter(Changes::any) else {
                    return;
                };
                self.saving = true;
                cx.request(
                    Request::Update {
                        deck_id,
                        changes: Box::new(changes),
                    }
                    .into(),
                );
            }
        }
    }
}

fn patch_vec(changed: bool, value: &[String]) -> Patch<Vec<String>> {
    if changed {
        Patch::Set(Some(value.to_vec()))
    } else {
        Patch::Unchanged
    }
}

/// A new deck's fields, as `HttpCreateDeckProfile` takes them.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NewDeck {
    pub(crate) name: String,
    pub(crate) commander: Option<Uuid>,
    pub(crate) partner: Option<Uuid>,
    pub(crate) background: Option<Uuid>,
    pub(crate) signature_spell: Option<Uuid>,
    pub(crate) format: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
    pub(crate) power: Option<String>,
    pub(crate) other_tags: Option<Vec<String>>,
    pub(crate) oracle_tags: Option<Vec<String>>,
    pub(crate) land_target: Option<i32>,
    pub(crate) price_target: Option<f64>,
    pub(crate) currency: Option<PriceCurrency>,
}

impl NewDeck {
    fn body(&self) -> HttpCreateDeckProfile {
        HttpCreateDeckProfile::builder(&self.name)
            .commander_id(self.commander)
            .partner_commander_id(self.partner)
            .background_id(self.background)
            .signature_spell_id(self.signature_spell)
            .format(self.format.clone())
            .tags(self.tags.clone())
            .power_level(self.power.clone())
            .other_tags(self.other_tags.clone())
            .oracle_tags(self.oracle_tags.clone())
            .land_target(self.land_target)
            .price_target(self.price_target)
            .price_target_currency(self.currency)
            .build()
    }
}

/// An edit's changed fields, as `HttpUpdateDeckProfile` takes them.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Changes {
    pub(crate) name: Option<String>,
    pub(crate) commander: Patch<Uuid>,
    pub(crate) partner: Patch<Uuid>,
    pub(crate) background: Patch<Uuid>,
    pub(crate) signature_spell: Patch<Uuid>,
    pub(crate) format: Patch<String>,
    pub(crate) tags: Patch<Vec<String>>,
    pub(crate) land_target: Patch<i32>,
    pub(crate) price_target: Patch<f64>,
    pub(crate) currency: Patch<PriceCurrency>,
    pub(crate) power: Patch<String>,
    pub(crate) other_tags: Patch<Vec<String>>,
    pub(crate) oracle_tags: Patch<Vec<String>>,
}

impl Changes {
    /// Whether anything changed.
    pub(crate) fn any(&self) -> bool {
        self.name.is_some()
            || self.commander != Patch::Unchanged
            || self.partner != Patch::Unchanged
            || self.background != Patch::Unchanged
            || self.signature_spell != Patch::Unchanged
            || self.format != Patch::Unchanged
            || self.tags != Patch::Unchanged
            || self.land_target != Patch::Unchanged
            || self.price_target != Patch::Unchanged
            || self.currency != Patch::Unchanged
            || self.power != Patch::Unchanged
            || self.other_tags != Patch::Unchanged
            || self.oracle_tags != Patch::Unchanged
    }

    fn body(&self) -> HttpUpdateDeckProfile {
        HttpUpdateDeckProfile::builder()
            .name(self.name.as_deref())
            .commander_id(self.commander.clone())
            .partner_commander_id(self.partner.clone())
            .background_id(self.background.clone())
            .signature_spell_id(self.signature_spell.clone())
            .format(self.format.clone())
            .tags(self.tags.clone())
            .land_target(self.land_target.clone())
            .price_target(self.price_target.clone())
            .price_target_currency(self.currency.clone())
            .power_level(self.power.clone())
            .other_tags(self.other_tags.clone())
            .oracle_tags(self.oracle_tags.clone())
            .build()
    }
}

/// Shows the form's hint again.
pub(crate) fn reopen_hint(cx: &mut Context<'_>, mode: Mode) {
    hint::reopen(cx, mode.hint_key());
}

/// The form's work.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    DeckTagCatalog,
    OracleCatalog,
    /// Waits out the typing pause before a slot's search.
    Debounce {
        slot: Slot,
        generation: u64,
    },
    Search {
        slot: Slot,
        generation: u64,
        query: Box<CardQuery>,
    },
    /// Looks for the commander's named partner.
    FindPartner {
        commander_id: Uuid,
        mate: String,
    },
    LoadDeck(Uuid),
    FetchCard {
        slot: Slot,
        id: Uuid,
    },
    Create(Box<NewDeck>),
    Update {
        deck_id: Uuid,
        changes: Box<Changes>,
    },
    /// A page of the swipe picker's pile.
    SwipePage {
        generation: u64,
        query: Box<CardQuery>,
    },
    MaybeboardAdd(Uuid),
    MaybeboardRemove(Uuid),
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::DeckTagCatalog => "deck_tag_catalog",
            Self::OracleCatalog => "oracle_catalog",
            Self::Debounce { .. } => "debounce",
            Self::Search { .. } => "search",
            Self::FindPartner { .. } => "find_partner",
            Self::LoadDeck(..) => "load_deck",
            Self::FetchCard { .. } => "fetch_card",
            Self::Create(..) => "create",
            Self::Update { .. } => "update",
            Self::SwipePage { .. } => "swipe_page",
            Self::MaybeboardAdd(..) => "maybeboard_add",
            Self::MaybeboardRemove(..) => "maybeboard_remove",
        }
    }
}

/// The form's results.
#[derive(Debug)]
pub(crate) enum Update {
    DeckTagCatalog(Vec<DeckTagView>),
    OracleCatalog(Vec<OracleTag>),
    Debounced {
        slot: Slot,
        generation: u64,
    },
    Results {
        slot: Slot,
        generation: u64,
        cards: Vec<Card>,
    },
    SearchFailed {
        slot: Slot,
        generation: u64,
    },
    PartnerHits {
        commander_id: Uuid,
        mate: String,
        cards: Vec<Card>,
    },
    DeckLoaded(Box<Deck>),
    LoadFailed(String),
    CardLoaded {
        slot: Slot,
        card: Box<Card>,
    },
    Created(Box<DeckProfile>),
    Updated,
    SaveFailed(String),
    SwipePage {
        generation: u64,
        cards: Vec<Card>,
    },
    MaybeboardAdded,
    MaybeboardRemoved,
    /// A toast for a failure with nothing else to show.
    Error(String),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::DeckForm(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::DeckForm(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut DeckForm> {
    match screen {
        Screen::DeckForm(form) => Some(form),
        _ => None,
    }
}

/// Opens Create with an empty form.
pub(crate) fn open_create(cx: &mut Context<'_>) {
    open_with(cx, DeckForm::new(Mode::Create));
}

/// Opens Create seeded with a commander, as the Commander maybeboard's "Create deck" does: the format is Commander and the card is the commander.
pub(crate) fn open_create_with_commander(cx: &mut Context<'_>, commander: Card) {
    let mut form = DeckForm::new(Mode::Create);
    form.format = Some(Format::Commander);
    form.card_mut(Slot::Commander).choose(Some(commander));
    open_with(cx, form);
}

/// Opens Edit for `deck_id` and loads the deck.
pub(crate) fn open_edit(cx: &mut Context<'_>, deck_id: Uuid) {
    cx.request(Request::LoadDeck(deck_id).into());
    open_with(cx, DeckForm::new(Mode::Edit { deck_id }));
}

fn open_with(cx: &mut Context<'_>, form: DeckForm) {
    let mode = form.mode;
    cx.push(Screen::DeckForm(Box::new(form)));
    cx.request(Request::DeckTagCatalog.into());
    cx.request(Request::OracleCatalog.into());
    hint::once(cx, mode.hint_key());
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let error = |e: &zwipe_client::ClientError| screens::failure(e, |m| Update::Error(m).into());
    match request {
        Request::DeckTagCatalog => {
            match async { client.fetch_deck_tags(&account.fresh().await?).await }.await {
                Ok(tags) => Some(Update::DeckTagCatalog(tags).into()),
                Err(e) => Some(error(&e)),
            }
        }
        Request::OracleCatalog => match client.fetch_oracle_tags().await {
            Ok(tags) => Some(Update::OracleCatalog(tags).into()),
            Err(e) => Some(error(&e)),
        },
        Request::Debounce { slot, generation } => {
            tokio::time::sleep(SEARCH_DEBOUNCE).await;
            Some(Update::Debounced { slot, generation }.into())
        }
        Request::Search {
            slot,
            generation,
            query,
        } => Some(
            match async { client.search_cards(&query, &account.fresh().await?).await }.await {
                Ok(cards) => Update::Results {
                    slot,
                    generation,
                    cards,
                }
                .into(),
                // A failed search only closes the results, as in the app.
                Err(_) => Update::SearchFailed { slot, generation }.into(),
            },
        ),
        Request::FindPartner { commander_id, mate } => {
            find_partner(account, commander_id, mate).await
        }
        Request::LoadDeck(deck_id) => Some(
            match async { client.fetch_deck(deck_id, &account.fresh().await?).await }.await {
                Ok(deck) => Update::DeckLoaded(Box::new(deck)).into(),
                Err(e) => screens::failure(&e, |m| Update::LoadFailed(m).into()),
            },
        ),
        Request::FetchCard { slot, id } => Some(match client.fetch_card(id).await {
            Ok(card) => Update::CardLoaded {
                slot,
                card: Box::new(card),
            }
            .into(),
            Err(e) => error(&e),
        }),
        Request::Create(deck) => Some(create(account, &deck).await),
        Request::Update { deck_id, changes } => Some(save(account, deck_id, &changes).await),
        Request::SwipePage { generation, query } => Some(
            match async {
                client
                    .search_commanders(&query, &account.fresh().await?)
                    .await
            }
            .await
            {
                Ok(cards) => Update::SwipePage { generation, cards }.into(),
                Err(e) => error(&e),
            },
        ),
        Request::MaybeboardAdd(oracle_id) => Some(maybeboard_add(account, oracle_id).await),
        Request::MaybeboardRemove(oracle_id) => Some(maybeboard_remove(account, oracle_id).await),
    }
}

/// Looks up the commander's named partner; a failed lookup leaves the partner empty, as in the app.
async fn find_partner(
    account: &Account,
    commander_id: Uuid,
    mate: String,
) -> Option<screens::Update> {
    let client = &account.client;
    let mut builder = CardQueryBuilder::with_name_contains(&mate);
    builder.set_limit(PARTNER_SEARCH_LIMIT);
    let query = builder.build().ok()?;
    let cards = async { client.search_cards(&query, &account.fresh().await?).await }
        .await
        .ok()?;
    Some(
        Update::PartnerHits {
            commander_id,
            mate,
            cards,
        }
        .into(),
    )
}

/// Creates the new deck's profile.
async fn create(account: &Account, deck: &NewDeck) -> screens::Update {
    match async {
        account
            .client
            .create_deck_profile(&deck.body(), &account.fresh().await?)
            .await
    }
    .await
    {
        Ok(profile) => Update::Created(Box::new(profile)).into(),
        Err(e) => screens::failure(&e, |m| Update::SaveFailed(m).into()),
    }
}

/// Saves the edited deck's changed fields.
async fn save(account: &Account, deck_id: Uuid, changes: &Changes) -> screens::Update {
    match async {
        account
            .client
            .update_deck_profile(deck_id, &changes.body(), &account.fresh().await?)
            .await
    }
    .await
    {
        Ok(_) => Update::Updated.into(),
        Err(e) => screens::failure(&e, |m| Update::SaveFailed(m).into()),
    }
}

/// Adds a commander to the user's commander maybeboard.
async fn maybeboard_add(account: &Account, oracle_id: Uuid) -> screens::Update {
    match async {
        account
            .client
            .add_commander_maybeboard_card(oracle_id, &account.fresh().await?)
            .await
    }
    .await
    {
        Ok(()) => Update::MaybeboardAdded.into(),
        Err(e) => screens::failure(&e, |m| Update::Error(m).into()),
    }
}

/// Takes a commander off the user's commander maybeboard.
async fn maybeboard_remove(account: &Account, oracle_id: Uuid) -> screens::Update {
    match async {
        account
            .client
            .remove_commander_maybeboard_card(oracle_id, &account.fresh().await?)
            .await
    }
    .await
    {
        Ok(()) => Update::MaybeboardRemoved.into(),
        Err(e) => screens::failure(&e, |m| Update::Error(m).into()),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let on_top = matches!(nav.top(), Screen::DeckForm(_));
    let Some(form) = nav.find_mut(pick) else {
        if let Update::Error(message) | Update::SaveFailed(message) = update {
            cx.toast(ToastKind::Error, message);
        }
        return;
    };
    match update {
        Update::DeckTagCatalog(tags) => form.deck_tag_catalog = tags,
        Update::OracleCatalog(tags) => form.oracle_catalog = Some(tags),
        Update::Debounced { slot, generation } => debounced(form, slot, generation, cx),
        Update::Results {
            slot,
            generation,
            cards,
        } => {
            let field = form.card_mut(slot);
            if field.generation == generation {
                field.dropdown = Dropdown::Results(cards);
                field.highlighted = 0;
            }
        }
        Update::SearchFailed { slot, generation } => {
            let field = form.card_mut(slot);
            if field.generation == generation {
                field.dropdown = Dropdown::Hidden;
            }
        }
        Update::PartnerHits {
            commander_id,
            mate,
            cards,
        } => partner_hits(form, commander_id, &mate, &cards, cx),
        Update::DeckLoaded(deck) => deck_loaded(form, &deck, cx),
        Update::LoadFailed(message) => {
            form.loading = false;
            form.load_error = Some(message.clone());
            cx.toast(ToastKind::Error, message);
        }
        Update::CardLoaded { slot, card } => card_loaded(form, slot, *card),
        Update::Created(profile) => {
            form.saving = false;
            if on_top {
                screens::deck::replace_with(cx, *profile);
            }
        }
        Update::Updated => {
            form.saving = false;
            if on_top {
                cx.back();
            }
        }
        Update::SaveFailed(message) => {
            form.saving = false;
            cx.toast(ToastKind::Error, message);
        }
        Update::SwipePage { generation, cards } => {
            if let Some(Overlay::Swipe(picker)) = &mut form.overlay {
                picker.take_page(generation, cards, cx);
            }
        }
        Update::MaybeboardAdded => cx.toast(
            ToastKind::Success,
            "Added to your commander maybeboard".to_owned(),
        ),
        Update::MaybeboardRemoved => cx.toast_for(
            ToastKind::Info,
            "Removed from your commander maybeboard".to_owned(),
            toast::QUICK,
        ),
        Update::Error(message) => cx.toast(ToastKind::Error, message),
    }
}

/// Runs the slot's search once typing settles, unless a newer keystroke superseded it.
fn debounced(form: &mut DeckForm, slot: Slot, generation: u64, cx: &mut Context<'_>) {
    if form.card(slot).generation != generation {
        return;
    }
    match form.search_query(slot) {
        Some(query) => cx.request(
            Request::Search {
                slot,
                generation,
                query: Box::new(query),
            }
            .into(),
        ),
        None => form.card_mut(slot).dropdown = Dropdown::Hidden,
    }
}

/// Picks the named partner when the same commander is still chosen and the partner is empty.
fn partner_hits(
    form: &mut DeckForm,
    commander_id: Uuid,
    mate: &str,
    cards: &[Card],
    cx: &mut Context<'_>,
) {
    let same_commander = form.card(Slot::Commander).card_id() == Some(commander_id);
    if same_commander
        && form.card(Slot::Partner).card.is_none()
        && let Some(partner) = form::named_partner(cards, mate).cloned()
    {
        cx.toast(
            ToastKind::Info,
            format!("Partner found and selected: {}", partner.scryfall_data.name),
        );
        form.set_card(Slot::Partner, Some(partner));
    }
}

/// Fills the form from the loaded deck and fetches each chosen card.
fn deck_loaded(form: &mut DeckForm, deck: &Deck, cx: &mut Context<'_>) {
    form.fill(&deck.deck_profile);
    let profile = &deck.deck_profile;
    for (slot, id) in [
        (Slot::Commander, profile.commander_id),
        (Slot::Partner, profile.partner_commander_id),
        (Slot::Background, profile.background_id),
        (Slot::SignatureSpell, profile.signature_spell_id),
    ] {
        if let Some(id) = id {
            cx.request(Request::FetchCard { slot, id }.into());
        }
    }
}

/// Sets a fetched card into its slot and marks it as the slot's original.
fn card_loaded(form: &mut DeckForm, slot: Slot, card: Card) {
    let id = card.scryfall_data.id;
    form.card_mut(slot).choose(Some(card));
    if let Some(original) = &mut form.original
        && let Some(card) = original.cards.get_mut(slot_index(slot))
    {
        *card = Some(id);
    }
}

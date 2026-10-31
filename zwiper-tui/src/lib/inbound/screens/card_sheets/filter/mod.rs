//! The card filter sheet the Add, deck cards and Remove screens open: fifteen accordion sections edit a draft of the host's filter, Apply hands it back, and closing any other way drops it.
//!
//! The host keeps its applied filter. Opening the sheet copies it as the draft and a snapshot; Apply returns the draft for the host to commit, and Back restores nothing because the host's filter never changed.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::scryfall_data::rarity::Rarity;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::Format;
use zwipe_core::domain::user::hints::HINT_FILTER;

use super::Request;
use crate::domain::filter::fields::{
    End, Exclude, MatchGroup, RangeGroup, SideGroup, StrList, TextField, card_types,
    identity_colors, parse_number, toggle_identity, toggled,
};
use crate::domain::filter::picker::{PickerField, Sources};
use crate::domain::filter::sections::{self, ChipSet, Modes, NumberField, Row, Section, StepRow};
use crate::domain::filter::{self, AddContext, Adopted, ApplyAction, FilterState};
use crate::domain::stores::catalog::Catalog;
use crate::inbound::app::Context;
use crate::inbound::screens::oracle_tags;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::{self, ToastKind};

/// The produced-mana chips: the five colors and colorless.
const PRODUCED: [&str; 6] = ["W", "U", "B", "R", "G", "C"];

/// What Reset does on this host.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum ResetRule {
    /// Back to the blank filter.
    #[default]
    Plain,
    /// Add's search: back to the deck-context default, keeping synergy.
    AddSearch(AddContext),
    /// Add's maybeboard: back to blank, keeping synergy.
    AddMaybeboard,
}

/// How a host opens the sheet.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct SheetConfig {
    /// Whether the Format section shows. Every host shows it.
    pub(crate) show_format: bool,
    /// Whether Apply refuses an empty draft over an empty filter, as Add's search does.
    pub(crate) validate: bool,
    pub(crate) reset: ResetRule,
    /// Where the word pickers draw from.
    pub(crate) sources: Sources,
}

/// Hands the dictionary's tag to `sheet` when a host comes back into view. Returns whether the dictionary was open over that sheet.
pub(crate) fn returned_from_dictionary(
    sheet: Option<&mut FilterSheet>,
    cx: &mut Context<'_>,
) -> bool {
    sheet.is_some_and(|sheet| sheet.take_dictionary_tag(cx))
}

/// What a key did to the sheet.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SheetOutcome {
    /// The sheet stays open.
    Open,
    /// The sheet closed without changing the filter.
    Closed,
    /// Apply committed this filter; the sheet closed.
    Applied(Box<CardQueryBuilder>),
}

/// One place the focus can rest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Target {
    Header(Section),
    Row(Row),
}

/// A picker's typed query and the result it would pick on Enter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct PickerInput {
    pub(crate) input: TextInput,
    pub(crate) highlight: usize,
}

/// The open filter sheet.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FilterSheet {
    pub(crate) draft: CardQueryBuilder,
    snapshot: CardQueryBuilder,
    pub(crate) config: SheetConfig,
    pub(crate) modes: Modes,
    /// The one open section.
    pub(crate) open: Option<Section>,
    /// Index into [`FilterSheet::targets`].
    pub(crate) focus: usize,
    pub(crate) texts: HashMap<TextField, TextInput>,
    pub(crate) numbers: HashMap<NumberField, TextInput>,
    pub(crate) pickers: HashMap<PickerField, PickerInput>,
    /// The focused chip in each chip row.
    pub(crate) chip_focus: HashMap<ChipSet, usize>,
    /// Which end of a range stepper the focus is on: false for min, true for max.
    pub(crate) step_max: bool,
    /// The tag list the oracle tag dictionary was opened for, while it is open over the host.
    pub(crate) dictionary: Option<PickerField>,
}

impl FilterSheet {
    /// Opens the sheet over `filter`: shows the filter hint the first time, warns when a list failed to load, and fetches the lists this host needs.
    pub(crate) fn open(
        cx: &mut Context<'_>,
        filter: &CardQueryBuilder,
        config: SheetConfig,
    ) -> Self {
        hint::once(cx, HINT_FILTER);
        let catalog = &mut cx.shared.cards.catalog;
        if catalog.any_failed() {
            cx.toast(
                ToastKind::Warning,
                "Filter lists didn't load, trying again".to_owned(),
            );
        }
        let wanted = cx.shared.cards.catalog.claim(config.sources.needs());
        for kind in wanted {
            cx.request(Request::Catalog(kind).into());
        }
        Self::new(filter, config)
    }

    /// The sheet over `filter`, with every section closed.
    pub(crate) fn new(filter: &CardQueryBuilder, config: SheetConfig) -> Self {
        let mut sheet = Self {
            draft: filter.clone(),
            snapshot: filter.clone(),
            config,
            modes: Modes::default(),
            open: None,
            focus: 0,
            texts: HashMap::new(),
            numbers: HashMap::new(),
            pickers: HashMap::new(),
            chip_focus: HashMap::new(),
            step_max: false,
            dictionary: None,
        };
        sheet.sync_inputs();
        sheet
    }

    /// Whether the draft still matches what was applied, which grays out Apply.
    pub(crate) fn unchanged(&self) -> bool {
        self.draft == self.snapshot
    }

    /// The sections this host shows.
    pub(crate) fn sections(&self) -> Vec<Section> {
        Section::shown(self.config.show_format)
    }

    /// The open section's rows.
    pub(crate) fn rows(&self) -> Vec<Row> {
        self.open
            .map(|section| sections::rows(section, &self.draft, &self.modes))
            .unwrap_or_default()
    }

    /// Every place the focus can rest, top to bottom.
    pub(crate) fn targets(&self) -> Vec<Target> {
        let mut targets = Vec::new();
        for section in self.sections() {
            targets.push(Target::Header(section));
            if self.open == Some(section) {
                targets.extend(
                    self.rows()
                        .into_iter()
                        .filter(|row| row.focusable())
                        .map(Target::Row),
                );
            }
        }
        targets
    }

    pub(crate) fn focused(&self) -> Option<Target> {
        self.targets().get(self.focus).copied()
    }

    /// Whether the focus is in a text box, where letters type instead of acting.
    pub(crate) fn typing(&self) -> bool {
        matches!(
            self.focused(),
            Some(Target::Row(Row::Text(_) | Row::Number(_) | Row::Picker(_)))
        )
    }

    /// Rebuilds the text boxes from the draft, after something other than typing changed it.
    pub(crate) fn sync_inputs(&mut self) {
        for field in [
            TextField::NameContains,
            TextField::NameNotContains,
            TextField::OracleContains,
            TextField::OracleNotContains,
            TextField::FlavorContains,
            TextField::FlavorNotContains,
        ] {
            let input = TextInput::default()
                .with_placeholder(field.placeholder())
                .with_value(field.get(&self.draft));
            self.texts.insert(field, input);
        }
        for field in [
            NumberField::ManaValue,
            NumberField::ManaMin,
            NumberField::ManaMax,
            NumberField::PriceMin,
            NumberField::PriceMax,
        ] {
            let input = TextInput::default()
                .with_placeholder(field.placeholder())
                .with_value(&field.text(&self.draft));
            self.numbers.insert(field, input);
        }
    }

    /// Moves the focus by one, parsing a number box it leaves.
    pub(crate) fn move_focus(&mut self, cx: &mut Context<'_>, toward: Toward) {
        self.blur(cx);
        self.focus = toward.clamped(self.focus, self.targets().len());
        self.step_max = false;
    }

    /// Opens the focused section, closing any other, or closes it.
    pub(crate) fn toggle_section(&mut self, section: Section) {
        self.open = if self.open == Some(section) {
            None
        } else {
            Some(section)
        };
        self.focus = self
            .targets()
            .iter()
            .position(|t| *t == Target::Header(section))
            .unwrap_or_default();
    }

    /// Leaves a number box: parses what was typed into the draft, or toasts why it could not.
    pub(crate) fn blur(&mut self, cx: &mut Context<'_>) {
        let Some(Target::Row(Row::Number(field))) = self.focused() else {
            return;
        };
        let text = |sheet: &Self, field: NumberField| -> String {
            sheet
                .numbers
                .get(&field)
                .map(|i| i.value().to_owned())
                .unwrap_or_default()
        };
        match field {
            NumberField::ManaValue => match parse_number(&text(self, field)) {
                Ok(None) => {
                    self.draft.unset_cmc_equals();
                }
                Ok(Some(n)) => {
                    self.draft.set_cmc_equals(n);
                    if let Some(input) = self.numbers.get_mut(&field) {
                        input.set_value(&n.to_string());
                    }
                }
                Err(()) => cx.toast_for(
                    ToastKind::Error,
                    "Invalid mana value".to_owned(),
                    toast::QUICK,
                ),
            },
            NumberField::ManaMin | NumberField::ManaMax => {
                let (min, max) = (
                    text(self, NumberField::ManaMin),
                    text(self, NumberField::ManaMax),
                );
                match (parse_number(&min), parse_number(&max)) {
                    (Ok(None), Ok(None)) => {
                        self.draft.unset_cmc_range();
                    }
                    (Ok(Some(min)), Ok(Some(max))) => {
                        self.draft.set_cmc_range((min, max));
                    }
                    (Ok(_), Ok(_)) => {}
                    _ => cx.toast_for(
                        ToastKind::Error,
                        "Invalid mana value range".to_owned(),
                        toast::QUICK,
                    ),
                }
            }
            NumberField::PriceMin | NumberField::PriceMax => {
                match parse_number(&text(self, field)) {
                    Ok(value) => {
                        let min = field == NumberField::PriceMin;
                        match (min, value) {
                            (true, None) => self.draft.unset_price_min(),
                            (true, Some(n)) => self.draft.set_price_min(n),
                            (false, None) => self.draft.unset_price_max(),
                            (false, Some(n)) => self.draft.set_price_max(n),
                        };
                    }
                    Err(()) => {
                        cx.toast_for(ToastKind::Error, "Invalid price".to_owned(), toast::QUICK);
                    }
                }
            }
        }
    }

    /// Opens the oracle tag dictionary over the host for the tag list `field` names.
    pub(crate) fn open_dictionary(&mut self, field: PickerField, cx: &mut Context<'_>) {
        self.dictionary = Some(field);
        oracle_tags::open(cx);
    }

    /// Takes the tag the dictionary's Use left into the draft, as the app's `adopt_otag` does, when the dictionary was opened from this sheet. Returns whether it was, so the host skips its own refresh: the dictionary closing is not a return from another screen.
    pub(crate) fn take_dictionary_tag(&mut self, cx: &mut Context<'_>) -> bool {
        let Some(field) = self.dictionary.take() else {
            return false;
        };
        let Some(slug) = cx.shared.caches.handoff.oracle_tag.take() else {
            return true;
        };
        let exclude = matches!(field, PickerField::Exclude(_));
        let (kind, message) = match filter::adopt_oracle_tag(&mut self.draft, &slug, exclude) {
            Adopted::Already => (ToastKind::Info, "Already in filter"),
            Adopted::Added => (ToastKind::Success, "Added to filter"),
        };
        cx.toast_for(kind, message.to_owned(), toast::QUICK);
        self.sync_inputs();
        true
    }

    /// Back: closes the sheet, saying so when edits are dropped.
    pub(crate) fn back(&self, cx: &mut Context<'_>) -> SheetOutcome {
        if !self.unchanged() {
            cx.toast_for(ToastKind::Info, "Filter unchanged".to_owned(), toast::QUICK);
        }
        SheetOutcome::Closed
    }

    /// Apply: refuses a contradiction or an empty search, closes on no change, else commits.
    pub(crate) fn apply(&self, cx: &mut Context<'_>) -> SheetOutcome {
        if let Some(message) = filter::contradiction(&self.draft) {
            cx.toast(ToastKind::Warning, message);
            return SheetOutcome::Open;
        }
        let action = filter::apply_action(
            self.config.validate,
            FilterState::of(&self.snapshot),
            FilterState::of(&self.draft),
            self.unchanged(),
        );
        match action {
            ApplyAction::RefuseEmpty => {
                cx.toast_for(
                    ToastKind::Warning,
                    "Filter is already empty".to_owned(),
                    toast::QUICK,
                );
                SheetOutcome::Open
            }
            ApplyAction::NoChange => SheetOutcome::Closed,
            ApplyAction::Commit { cleared } => {
                let message = if cleared {
                    "Filter cleared"
                } else {
                    "Filter applied"
                };
                cx.toast_for(ToastKind::Success, message.to_owned(), toast::QUICK);
                SheetOutcome::Applied(Box::new(self.draft.clone()))
            }
        }
    }

    /// Reset: stages the host's default in the draft. Apply commits it; Back drops it.
    pub(crate) fn reset(&mut self, cx: &mut Context<'_>) {
        let result = match &self.config.reset {
            ResetRule::Plain => {
                self.draft.clear();
                Ok(())
            }
            ResetRule::AddSearch(context) => context.reset_search(&mut self.draft),
            ResetRule::AddMaybeboard => filter::reset_maybeboard(&mut self.draft),
        };
        match result {
            Ok(()) => cx.toast_for(ToastKind::Info, "Filter reset".to_owned(), toast::QUICK),
            Err(filter::AlreadyDefault) => {
                cx.toast_for(
                    ToastKind::Warning,
                    "Filter already at default".to_owned(),
                    toast::QUICK,
                );
            }
        }
        self.sync_inputs();
        self.clamp_focus();
    }

    /// Keeps the focus on a target that still exists after rows change.
    pub(crate) fn clamp_focus(&mut self) {
        self.focus = self.focus.min(self.targets().len().saturating_sub(1));
    }

    /// The chips of a row as labels and whether each is on.
    pub(crate) fn chips(&self, set: ChipSet, catalog: &Catalog) -> Vec<(String, bool)> {
        let b = &self.draft;
        let named = |values: Vec<String>, picked: &[String]| -> Vec<(String, bool)> {
            values
                .into_iter()
                .map(|v| {
                    let on = picked.contains(&v);
                    (v, on)
                })
                .collect()
        };
        match set {
            ChipSet::BasicIncludes | ChipSet::BasicExcludes => {
                let picked = if set == ChipSet::BasicIncludes {
                    MatchGroup::BasicTypes.values(b)
                } else {
                    Exclude::BasicTypes.values(b)
                };
                named(
                    card_types().iter().map(ToString::to_string).collect(),
                    &picked,
                )
            }
            ChipSet::Colors => {
                let picked = identity_colors(b);
                Color::all()
                    .into_iter()
                    .map(|c| (c.to_short_name(), picked.contains(&c)))
                    .collect()
            }
            ChipSet::ProducedIncludes => named(
                PRODUCED.map(str::to_owned).to_vec(),
                &MatchGroup::Produced.values(b),
            ),
            ChipSet::ProducedExcludes => named(
                PRODUCED.map(str::to_owned).to_vec(),
                &StrList::ProducedExclude.get(b),
            ),
            ChipSet::Rarity => named(
                Rarity::all().iter().map(Rarity::to_long_name).collect(),
                &SideGroup::Rarity.values(b),
            ),
            ChipSet::RolesInclude | ChipSet::RolesExclude => {
                let picked = if set == ChipSet::RolesInclude {
                    MatchGroup::Roles.values(b)
                } else {
                    StrList::RolesExclude.get(b)
                };
                catalog
                    .card_roles
                    .ready()
                    .map(|roles| {
                        roles
                            .iter()
                            .map(|r| (r.display_name.clone(), picked.contains(&r.slug)))
                            .collect()
                    })
                    .unwrap_or_default()
            }
            ChipSet::CommanderIn => Format::commander_formats()
                .iter()
                .map(|f| {
                    (
                        f.display_name().to_owned(),
                        b.is_commander_in_format() == Some(f),
                    )
                })
                .collect(),
            ChipSet::LegalIn => {
                let picked = StrList::Legal.get(b);
                Format::all()
                    .iter()
                    .map(|f| {
                        let on = picked.iter().any(|k| k == f.to_legality_key());
                        (f.display_name().to_owned(), on)
                    })
                    .collect()
            }
            ChipSet::Currency => {
                let current = sections::currency(b);
                PriceCurrency::all()
                    .iter()
                    .map(|c| (c.label().to_owned(), *c == current))
                    .collect()
            }
            ChipSet::SortBy => CardSortKey::all()
                .into_iter()
                .map(|k| (k.to_string(), b.sort() == Some(k)))
                .collect(),
            ChipSet::SortOrder => vec![(b.direction().to_string(), true)],
            ChipSet::Config(toggle) => {
                let value = toggle.get(b);
                vec![
                    ("Yes".to_owned(), value == Some(true)),
                    ("No".to_owned(), value == Some(false)),
                    ("Any".to_owned(), value.is_none()),
                ]
            }
        }
    }

    /// Taps the chip at `index` in a row.
    pub(crate) fn tap_chip(&mut self, set: ChipSet, index: usize, catalog: &Catalog) {
        let modes = self.modes.clone();
        let b = &mut self.draft;
        match set {
            ChipSet::BasicIncludes | ChipSet::BasicExcludes => {
                tap_basic_type(b, &modes, set, index);
            }
            ChipSet::Colors => {
                if let Some(color) = Color::all().get(index) {
                    let mode = modes.range_of(b, RangeGroup::ColorIdentity);
                    toggle_identity(b, mode, *color);
                }
            }
            ChipSet::ProducedIncludes => {
                if let Some(symbol) = PRODUCED.get(index) {
                    let mode = modes.match_of(b, MatchGroup::Produced);
                    let values = toggled(MatchGroup::Produced.values(b), symbol);
                    MatchGroup::Produced.write(b, mode, values);
                }
            }
            ChipSet::ProducedExcludes => {
                if let Some(symbol) = PRODUCED.get(index) {
                    let values = toggled(StrList::ProducedExclude.get(b), symbol);
                    StrList::ProducedExclude.set(b, values);
                }
            }
            ChipSet::Rarity => {
                if let Some(rarity) = Rarity::all().get(index) {
                    let side = modes.side_of(b, SideGroup::Rarity);
                    let values = toggled(SideGroup::Rarity.values(b), &rarity.to_long_name());
                    SideGroup::Rarity.write(b, side, values);
                }
            }
            ChipSet::RolesInclude | ChipSet::RolesExclude => {
                tap_role(b, &modes, catalog, set, index);
            }
            ChipSet::CommanderIn => {
                if let Some(format) = Format::commander_formats().get(index) {
                    if b.is_commander_in_format() == Some(format) {
                        b.unset_is_commander_in_format();
                    } else {
                        b.set_is_commander_in_format(*format);
                    }
                }
            }
            ChipSet::LegalIn => {
                if let Some(format) = Format::all().get(index) {
                    let values = toggled(StrList::Legal.get(b), format.to_legality_key());
                    StrList::Legal.set(b, values);
                }
            }
            ChipSet::Currency => {
                if let Some(currency) = PriceCurrency::all().get(index) {
                    b.set_price_currency(*currency);
                }
            }
            ChipSet::SortBy => {
                if let Some(key) = CardSortKey::all().get(index) {
                    if b.sort() == Some(*key) {
                        b.unset_sort();
                    } else {
                        b.set_sort(*key);
                    }
                }
            }
            ChipSet::SortOrder => {
                b.set_direction(b.direction().flipped());
            }
            ChipSet::Config(toggle) => {
                let chip = match index {
                    0 => Some(true),
                    1 => Some(false),
                    _ => None,
                };
                toggle.tap(b, chip);
            }
        }
    }

    /// Steps a stepper row by `delta` on the focused end.
    pub(crate) fn step(&mut self, row: StepRow, delta: i32) {
        match row {
            StepRow::Exact(stat) => stat.step_exact(&mut self.draft, delta),
            StepRow::Range(stat) => {
                let end = if self.step_max { End::Max } else { End::Min };
                stat.step_range(&mut self.draft, end, delta);
            }
        }
    }
}

/// Taps the basic type chip at `index` in the include or exclude row `set`.
fn tap_basic_type(b: &mut CardQueryBuilder, modes: &Modes, set: ChipSet, index: usize) {
    let labels: Vec<String> = card_types().iter().map(ToString::to_string).collect();
    let Some(name) = labels.get(index).cloned() else {
        return;
    };
    if set == ChipSet::BasicIncludes {
        let mode = modes.match_of(b, MatchGroup::BasicTypes);
        let values = toggled(MatchGroup::BasicTypes.values(b), &name);
        MatchGroup::BasicTypes.write(b, mode, values);
    } else {
        let values = toggled(Exclude::BasicTypes.values(b), &name);
        Exclude::BasicTypes.write(b, values);
    }
}

/// Taps the card role chip at `index` in the include or exclude row `set`.
fn tap_role(
    b: &mut CardQueryBuilder,
    modes: &Modes,
    catalog: &Catalog,
    set: ChipSet,
    index: usize,
) {
    let Some(slug) = catalog
        .card_roles
        .ready()
        .and_then(|roles| roles.get(index))
        .map(|r| r.slug.clone())
    else {
        return;
    };
    if set == ChipSet::RolesInclude {
        let mode = modes.match_of(b, MatchGroup::Roles);
        let values = toggled(MatchGroup::Roles.values(b), &slug);
        MatchGroup::Roles.write(b, mode, values);
    } else {
        let values = toggled(StrList::RolesExclude.get(b), &slug);
        StrList::RolesExclude.set(b, values);
    }
}

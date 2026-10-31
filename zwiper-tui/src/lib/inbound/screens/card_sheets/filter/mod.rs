//! The card filter sheet the Add, deck cards and Remove screens open: fifteen accordion sections edit a draft of the host's filter, Apply hands it back, and closing any other way drops it.
//!
//! The host keeps its applied filter. The draft, its snapshot and every rule for Apply, Back and Reset are zwipe-ui's `FilterDraft`, which returns its toasts for the sheet to show; this module keeps the focus, the text boxes and the chips. Apply does nothing while the draft matches the snapshot, as in the app.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use chrono::Utc;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::scryfall_data::rarity::Rarity;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::Format;
use zwipe_ui::filter::draft::{ApplyOutcome, FilterDraft, SheetConfig};
use zwipe_ui::filter::fields::{
    End, Exclude, MatchGroup, RangeGroup, SideGroup, StrList, TextField, card_types,
    identity_colors, toggle_identity, toggled,
};
use zwipe_ui::filter::picker::PickerField;
use zwipe_ui::filter::sections::{self, ChipSet, Modes, NumberField, Row, Section, StepRow};
use zwipe_ui::stores::catalog::Catalog;

use super::Request;
use crate::inbound::app::Context;
use crate::inbound::screens::oracle_tags;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::text_input::TextInput;

/// The produced-mana chips: the five colors and colorless.
const PRODUCED: [&str; 6] = ["W", "U", "B", "R", "G", "C"];

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
    /// The draft, its snapshot, the host's config and the mode chips, whose rules zwipe-ui keeps.
    pub(crate) filter: FilterDraft,
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
        let (filter, opened) =
            FilterDraft::open(filter, config, &mut cx.shared.cards.catalog, Utc::now());
        hint::once(cx, opened.hint.key());
        if let Some(toast) = opened.toast {
            cx.show(toast);
        }
        for kind in opened.fetch {
            cx.request(Request::Catalog(kind).into());
        }
        Self::over(filter)
    }

    /// The sheet over `filter`, with every section closed.
    #[cfg(test)]
    pub(crate) fn new(filter: &CardQueryBuilder, config: SheetConfig) -> Self {
        Self::over(FilterDraft::new(filter, config))
    }

    /// The sheet over a draft, with every section closed.
    fn over(filter: FilterDraft) -> Self {
        let mut sheet = Self {
            filter,
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
        self.filter.unchanged()
    }

    /// The sections this host shows.
    pub(crate) fn sections(&self) -> Vec<Section> {
        Section::shown(self.filter.config.show_format)
    }

    /// The open section's rows.
    pub(crate) fn rows(&self) -> Vec<Row> {
        self.open
            .map(|section| sections::rows(section, &self.filter.draft, &self.filter.modes))
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
                .with_value(field.get(&self.filter.draft));
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
                .with_value(&field.text(&self.filter.draft));
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
        let other = match field {
            NumberField::ManaMin => text(self, NumberField::ManaMax),
            NumberField::ManaMax => text(self, NumberField::ManaMin),
            NumberField::ManaValue | NumberField::PriceMin | NumberField::PriceMax => String::new(),
        };
        if let Err(toast) = self.filter.set_number(field, &text(self, field), &other) {
            cx.show(toast);
            return;
        }
        if field == NumberField::ManaValue
            && let Some(n) = self.filter.draft.cmc_equals()
            && let Some(input) = self.numbers.get_mut(&field)
        {
            input.set_value(&n.to_string());
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
        let toast = self.filter.adopt_oracle_tag(field, &slug);
        cx.show(toast);
        self.sync_inputs();
        true
    }

    /// Back: closes the sheet, saying so when edits are dropped.
    pub(crate) fn back(&self, cx: &mut Context<'_>) -> SheetOutcome {
        if let Some(toast) = self.filter.back() {
            cx.show(toast);
        }
        SheetOutcome::Closed
    }

    /// Apply: does nothing while the draft is unchanged, refuses a contradiction or an empty search, else commits.
    pub(crate) fn apply(&self, cx: &mut Context<'_>) -> SheetOutcome {
        match self.filter.apply() {
            ApplyOutcome::Disabled => SheetOutcome::Open,
            ApplyOutcome::Refused(toast) => {
                cx.show(toast);
                SheetOutcome::Open
            }
            ApplyOutcome::Applied { filter, toast } => {
                cx.show(toast);
                SheetOutcome::Applied(filter)
            }
        }
    }

    /// Reset: stages the host's default in the draft. Apply commits it; Back drops it.
    pub(crate) fn reset(&mut self, cx: &mut Context<'_>) {
        let toast = self.filter.reset();
        cx.show(toast);
        self.sync_inputs();
        self.clamp_focus();
    }

    /// Keeps the focus on a target that still exists after rows change.
    pub(crate) fn clamp_focus(&mut self) {
        self.focus = self.focus.min(self.targets().len().saturating_sub(1));
    }

    /// The chips of a row as labels and whether each is on.
    pub(crate) fn chips(&self, set: ChipSet, catalog: &Catalog) -> Vec<(String, bool)> {
        let b = &self.filter.draft;
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
        let modes = self.filter.modes.clone();
        let b = &mut self.filter.draft;
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
            StepRow::Exact(stat) => stat.step_exact(&mut self.filter.draft, delta),
            StepRow::Range(stat) => {
                let end = if self.step_max { End::Max } else { End::Min };
                stat.step_range(&mut self.filter.draft, end, delta);
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

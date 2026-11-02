//! The card filter sheet that Add, the deck cards view, Remove and the Commander maybeboard open over their filter.
//!
//! The model holds the sheet's draft against the filter it opened over ([`FilterDraft`]), decides Apply, Back and Reset, keeps the tag list the oracle tag dictionary feeds, and raises the filter hint and the lists warning. The host keeps its applied filter and takes a new one only through [`Effect::Apply`]. Whether the sheet shows, which sections are open, the steppers, chips and text boxes as drawn stay in the client; a client edits the draft with `zwipe_ui::filter`'s field and chip helpers and hands it back with [`Action::Edit`], or uses the actions here that carry the rule with them.
//!
//! The sheet sends no requests of its own. The word lists its pickers search live in the client's catalog store, which every screen shares and which dedupes its own fetches, so the model asks for them with [`Effect::EnsureLists`] and takes no tickets.

#[cfg(test)]
mod tests;

use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::user::hints::OneTimeHint;

use crate::filter::draft::{ApplyOutcome, FilterDraft, SheetConfig};
use crate::filter::picker::{PickerField, Sources};
use crate::filter::sections::{self, ModeChip, Modes, NumberField, Row, Section};
use crate::stores::catalog::Kind;
use crate::toast::{Toast, ToastKind};

/// The sheet's header.
pub const TITLE: &str = "Filter";

/// The button that closes the sheet and drops the draft.
pub const BACK: &str = "Back";

/// The button that hands the draft to the host.
pub const APPLY: &str = "Apply";

/// The button that stages the host's default filter.
pub const RESET: &str = "Reset";

/// The warning raised when a list the pickers search failed to load.
pub const LISTS_FAILED: &str = "Filter lists didn't load, trying again";

/// Where an action sends the user. The client maps each to its own route or overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The oracle tag dictionary, opened over the sheet. Its Use answers with [`Action::UseTag`].
    OracleTagDictionary,
}

/// Whether Reset says what it did. Add, the deck cards view and Remove toast; the Commander maybeboard and Swipe select stage the blank filter quietly.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ResetNotice {
    /// "Filter reset", or "Filter already at default" when there is nothing to reset.
    #[default]
    Toast,
    /// Stages the blank filter with no toast, whatever the host's Reset rule.
    Quiet,
}

/// What a user does on the sheet.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Opens the sheet over the host's applied `filter`. Does nothing while the sheet is open.
    Open {
        /// The host's applied filter, which the draft starts from.
        filter: Box<CardQueryBuilder>,
        /// How the host opens the sheet: Format, the empty-search check, its Reset rule and its picker sources.
        config: Box<SheetConfig>,
        /// Whether Reset toasts.
        notice: ResetNotice,
    },
    /// A list the pickers search failed to load. Warns once until the sheet next closes.
    ListsFailed,
    /// Replaces the open draft with the filter the client's controls left. Does nothing while the sheet is closed.
    Edit(Box<CardQueryBuilder>),
    /// Presses a section header's clear button.
    ClearSection(Section),
    /// Presses a mode chip, moving any values to the other mode.
    FlipMode(ModeChip),
    /// Picks a search result in a picker.
    Pick {
        /// The picker.
        field: PickerField,
        /// The value picked.
        value: String,
    },
    /// Removes a picked value's pill.
    Unpick {
        /// The picker.
        field: PickerField,
        /// The value removed.
        value: String,
    },
    /// Leaves a number box: writes what was typed, or toasts why it is not a number.
    SetNumber {
        /// The box.
        field: NumberField,
        /// What the box holds.
        text: String,
        /// The other end's text when `field` is a mana value range end, else empty.
        other: String,
    },
    /// Presses a Dictionary chip: opens the oracle tag dictionary to feed this tag list.
    OpenDictionary(PickerField),
    /// Presses Use on a tag in the dictionary: adds the slug to the list it was opened for.
    UseTag(String),
    /// Presses Apply.
    Apply,
    /// Presses Reset: stages the host's default filter for Apply to commit.
    Reset,
    /// Closes the sheet any way but Apply: Back, the backdrop, the OS back gesture.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Fetches these lists into the client's catalog store unless they are there or on the way.
    EnsureLists(&'static [Kind]),
    /// Gives the host this filter to apply and search with.
    Apply(Box<CardQueryBuilder>),
    /// The filter the sheet opened over, for a client that lets the sheet edit the host's filter in place to put back. A client that edits its own copy has nothing to do.
    Restore(Box<CardQueryBuilder>),
    /// Closes the sheet.
    Close,
    /// Goes to this screen or overlay.
    Navigate(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// Whether the visit has offered the filter hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// Whether the lists warning is up for this opening.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum ListsWarning {
    #[default]
    Unsaid,
    Said,
}

/// The card filter sheet for one host visit. Starts closed with no draft; send [`Action::Open`] when the sheet opens.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    /// The draft of the open sheet, or the one it closed on.
    sheet: Option<FilterDraft>,
    open: bool,
    notice: ResetNotice,
    /// The tag list the dictionary feeds while it is up.
    dictionary: Option<PickerField>,
    hint: HintOffer,
    lists: ListsWarning,
}

impl State {
    /// The sheet as it first draws: closed, with no draft.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// The filter hint is offered the first time the sheet opens in a visit. Every action but [`Action::Open`] and [`Action::ListsFailed`] does nothing while the sheet is closed.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Open {
                filter,
                config,
                notice,
            } => self.open_over(&filter, *config, notice),
            Action::ListsFailed => {
                if self.lists == ListsWarning::Said {
                    return Vec::new();
                }
                self.lists = ListsWarning::Said;
                vec![Effect::Toast(Toast::normal(
                    ToastKind::Warning,
                    LISTS_FAILED,
                ))]
            }
            Action::Apply => self.apply(),
            Action::Back => self.back(),
            Action::Reset => self.reset(),
            Action::UseTag(slug) => self.use_tag(&slug),
            Action::SetNumber { field, text, other } => {
                let Some(sheet) = self.open_sheet() else {
                    return Vec::new();
                };
                match sheet.set_number(field, &text, &other) {
                    Ok(()) => Vec::new(),
                    Err(toast) => vec![Effect::Toast(toast)],
                }
            }
            Action::OpenDictionary(field) => {
                if !self.open {
                    return Vec::new();
                }
                self.dictionary = Some(field);
                vec![Effect::Navigate(Destination::OracleTagDictionary)]
            }
            Action::Edit(filter) => {
                if let Some(sheet) = self.open_sheet() {
                    sheet.draft = *filter;
                }
                Vec::new()
            }
            Action::ClearSection(section) => {
                if let Some(sheet) = self.open_sheet() {
                    section.clear(&mut sheet.draft);
                }
                Vec::new()
            }
            Action::FlipMode(chip) => {
                if let Some(sheet) = self.open_sheet() {
                    sheet.modes.flip(&mut sheet.draft, chip);
                }
                Vec::new()
            }
            Action::Pick { field, value } => {
                if let Some(sheet) = self.open_sheet() {
                    field.pick(&mut sheet.draft, &sheet.modes, &value);
                }
                Vec::new()
            }
            Action::Unpick { field, value } => {
                if let Some(sheet) = self.open_sheet() {
                    field.unpick(&mut sheet.draft, &sheet.modes, &value);
                }
                Vec::new()
            }
        }
    }

    /// Whether the sheet is open.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// The draft: the open sheet's, or the one it closed on, so the sheet sliding away still draws it. `None` before the first opening.
    #[must_use]
    pub fn draft(&self) -> Option<&CardQueryBuilder> {
        self.sheet.as_ref().map(|sheet| &sheet.draft)
    }

    /// The filter the open sheet started from. `None` while the sheet is closed.
    #[must_use]
    pub fn applied(&self) -> Option<&CardQueryBuilder> {
        self.sheet
            .as_ref()
            .filter(|_| self.open)
            .map(FilterDraft::snapshot)
    }

    /// Whether Apply is off: the sheet is open and the draft matches the filter it started from.
    #[must_use]
    pub fn unchanged(&self) -> bool {
        self.open && self.sheet.as_ref().is_some_and(FilterDraft::unchanged)
    }

    /// The sections this host shows, in order: Format only where the host asked for it.
    #[must_use]
    pub fn sections(&self) -> Vec<Section> {
        let show_format = self
            .sheet
            .as_ref()
            .is_some_and(|sheet| sheet.config.show_format);
        Section::shown(show_format)
    }

    /// The shown sections with something set, which draw their clear button.
    #[must_use]
    pub fn active_sections(&self) -> Vec<Section> {
        let Some(draft) = self.draft() else {
            return Vec::new();
        };
        self.sections()
            .into_iter()
            .filter(|section| section.is_active(draft))
            .collect()
    }

    /// The rows `section` draws as the draft and its mode chips stand.
    #[must_use]
    pub fn rows(&self, section: Section) -> Vec<Row> {
        match &self.sheet {
            Some(sheet) => sections::rows(section, &sheet.draft, &sheet.modes),
            None => sections::rows(section, &CardQueryBuilder::default(), &Modes::default()),
        }
    }

    /// The mode chips chosen while their fields are empty. `None` before the first opening.
    #[must_use]
    pub fn modes(&self) -> Option<&Modes> {
        self.sheet.as_ref().map(|sheet| &sheet.modes)
    }

    /// Where the word pickers draw from for this host. `None` before the first opening.
    #[must_use]
    pub fn sources(&self) -> Option<&Sources> {
        self.sheet.as_ref().map(|sheet| &sheet.config.sources)
    }

    /// The tag list the oracle tag dictionary feeds, from the last Dictionary chip pressed while the sheet is open.
    #[must_use]
    pub fn dictionary(&self) -> Option<PickerField> {
        self.dictionary
    }

    /// The draft while the sheet is open.
    fn open_sheet(&mut self) -> Option<&mut FilterDraft> {
        if !self.open {
            return None;
        }
        self.sheet.as_mut()
    }

    fn open_over(
        &mut self,
        filter: &CardQueryBuilder,
        config: SheetConfig,
        notice: ResetNotice,
    ) -> Vec<Effect> {
        if self.open {
            return Vec::new();
        }
        let needs = config.sources.needs();
        self.sheet = Some(FilterDraft::new(filter, config));
        self.open = true;
        self.notice = notice;
        self.dictionary = None;
        let mut effects = Vec::new();
        if self.hint == HintOffer::Waiting {
            self.hint = HintOffer::Offered;
            effects.push(Effect::OfferHint(OneTimeHint::Filter));
        }
        effects.push(Effect::EnsureLists(needs));
        effects
    }

    /// Marks the sheet closed, ready for the next opening's warning.
    fn close(&mut self) {
        self.open = false;
        self.dictionary = None;
        self.lists = ListsWarning::Unsaid;
    }

    fn apply(&mut self) -> Vec<Effect> {
        let Some(sheet) = self.open_sheet() else {
            return Vec::new();
        };
        match sheet.apply() {
            ApplyOutcome::Disabled => Vec::new(),
            ApplyOutcome::Refused(toast) => vec![Effect::Toast(toast)],
            ApplyOutcome::Applied { filter, toast } => {
                self.close();
                vec![Effect::Apply(filter), Effect::Toast(toast), Effect::Close]
            }
        }
    }

    fn back(&mut self) -> Vec<Effect> {
        let Some(sheet) = self.open_sheet() else {
            return Vec::new();
        };
        let mut effects = Vec::new();
        if let Some(toast) = sheet.back() {
            effects.push(Effect::Restore(Box::new(sheet.snapshot().clone())));
            effects.push(Effect::Toast(toast));
        }
        self.close();
        effects.push(Effect::Close);
        effects
    }

    fn reset(&mut self) -> Vec<Effect> {
        let notice = self.notice;
        let Some(sheet) = self.open_sheet() else {
            return Vec::new();
        };
        if notice == ResetNotice::Quiet {
            sheet.draft.clear();
            return Vec::new();
        }
        vec![Effect::Toast(sheet.reset())]
    }

    fn use_tag(&mut self, slug: &str) -> Vec<Effect> {
        let Some(field) = self.dictionary else {
            return Vec::new();
        };
        let Some(sheet) = self.open_sheet() else {
            return Vec::new();
        };
        vec![Effect::Toast(sheet.adopt_oracle_tag(field, slug))]
    }
}

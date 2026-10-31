//! The card filter sheet's draft, as the Add, deck cards and Remove screens
//! use it: the sheet edits a copy of the host's filter, Apply hands it back,
//! and closing any other way drops it.
//!
//! Opening copies the host's applied filter as the draft and as a snapshot.
//! Every action returns what the client should show instead of showing it:
//! a toast, and whether the sheet stays open or hands back a filter.

use chrono::{DateTime, Utc};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::user::hints::OneTimeHint;

use super::fields::parse_number;
use super::picker::{PickerField, Sources};
use super::sections::{Modes, NumberField};
use super::{AddContext, Adopted, AlreadyDefault, ApplyAction, FilterState};
use crate::stores::catalog::{Catalog, Kind};
use crate::toast::{Toast, ToastKind};

/// What Reset does on a host.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ResetRule {
    /// Back to the blank filter: the deck cards view and Remove.
    #[default]
    Plain,
    /// Add's search: back to the deck-context default, keeping synergy.
    AddSearch(AddContext),
    /// Add's maybeboard: back to blank, keeping synergy.
    AddMaybeboard,
}

/// How a host opens the sheet.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SheetConfig {
    /// Whether the Format section shows.
    pub show_format: bool,
    /// Whether Apply refuses an empty draft over an empty filter, as Add's
    /// search does, since an empty filter would serve every card.
    pub validate: bool,
    /// What Reset does.
    pub reset: ResetRule,
    /// Where the word pickers draw from.
    pub sources: Sources,
}

/// What opening the sheet asks of the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    /// The one-time hint to show the first time, [`OneTimeHint::Filter`].
    pub hint: OneTimeHint,
    /// A warning when a public list failed to load last time.
    pub toast: Option<Toast>,
    /// The lists to fetch, already marked in flight.
    pub fetch: Vec<Kind>,
}

/// What Apply did.
#[derive(Debug, Clone, PartialEq)]
pub enum ApplyOutcome {
    /// Nothing changed since the sheet opened, so Apply is disabled and does
    /// nothing, as in the app.
    Disabled,
    /// Apply was refused; the sheet stays open with this toast.
    Refused(Toast),
    /// The draft is committed: the host takes the filter and shows the toast,
    /// and the sheet closes.
    Applied {
        /// The filter to apply.
        filter: Box<CardQueryBuilder>,
        /// "Filter applied" or "Filter cleared".
        toast: Toast,
    },
}

/// The open filter sheet's draft.
#[derive(Debug, Clone, PartialEq)]
pub struct FilterDraft {
    /// The filter the sheet edits.
    pub draft: CardQueryBuilder,
    /// The filter that was applied when the sheet opened.
    snapshot: CardQueryBuilder,
    /// How the host opened the sheet.
    pub config: SheetConfig,
    /// The mode chips chosen while their fields are empty.
    pub modes: Modes,
}

impl FilterDraft {
    /// The draft over `filter`.
    #[must_use]
    pub fn new(filter: &CardQueryBuilder, config: SheetConfig) -> Self {
        Self {
            draft: filter.clone(),
            snapshot: filter.clone(),
            config,
            modes: Modes::default(),
        }
    }

    /// Opens the sheet over `filter` at `now`: the draft, plus the filter
    /// hint, a warning when a list failed to load, and the lists this host
    /// needs fetched.
    pub fn open(
        filter: &CardQueryBuilder,
        config: SheetConfig,
        catalog: &mut Catalog,
        now: DateTime<Utc>,
    ) -> (Self, Opened) {
        let toast = catalog
            .any_public_failed()
            .then(|| Toast::normal(ToastKind::Warning, "Filter lists didn't load, trying again"));
        let fetch = catalog.claim(config.sources.needs(), now);
        let opened = Opened {
            hint: OneTimeHint::Filter,
            toast,
            fetch,
        };
        (Self::new(filter, config), opened)
    }

    /// Whether the draft still matches what was applied, which disables
    /// Apply.
    #[must_use]
    pub fn unchanged(&self) -> bool {
        self.draft == self.snapshot
    }

    /// Apply: disabled while nothing changed, refused for a contradiction or
    /// an empty search, otherwise the draft to commit.
    #[must_use]
    pub fn apply(&self) -> ApplyOutcome {
        if self.unchanged() {
            return ApplyOutcome::Disabled;
        }
        if let Some(message) = super::contradiction(&self.draft) {
            return ApplyOutcome::Refused(Toast::normal(ToastKind::Warning, message));
        }
        let action = super::apply_action(
            self.config.validate,
            FilterState::of(&self.snapshot),
            FilterState::of(&self.draft),
            false,
        );
        match action {
            ApplyAction::RefuseEmpty => {
                ApplyOutcome::Refused(Toast::quick(ToastKind::Warning, "Filter is already empty"))
            }
            ApplyAction::NoChange | ApplyAction::Commit { cleared: false } => {
                self.applied("Filter applied")
            }
            ApplyAction::Commit { cleared: true } => self.applied("Filter cleared"),
        }
    }

    /// The draft committed with a success toast.
    fn applied(&self, message: &str) -> ApplyOutcome {
        ApplyOutcome::Applied {
            filter: Box::new(self.draft.clone()),
            toast: Toast::quick(ToastKind::Success, message),
        }
    }

    /// Back, the backdrop or the OS back gesture: the sheet closes and the
    /// host's filter stays as it was, with a toast when edits were dropped.
    #[must_use]
    pub fn back(&self) -> Option<Toast> {
        (!self.unchanged()).then(|| Toast::quick(ToastKind::Info, "Filter unchanged"))
    }

    /// Reset: stages the host's default in the draft for Apply to commit.
    pub fn reset(&mut self) -> Toast {
        let result = match &self.config.reset {
            ResetRule::Plain => {
                self.draft.clear();
                Ok(())
            }
            ResetRule::AddSearch(context) => context.reset_search(&mut self.draft),
            ResetRule::AddMaybeboard => super::reset_maybeboard(&mut self.draft),
        };
        match result {
            Ok(()) => Toast::quick(ToastKind::Info, "Filter reset"),
            Err(AlreadyDefault) => Toast::quick(ToastKind::Warning, "Filter already at default"),
        }
    }

    /// Takes a tag the oracle tag dictionary's Use picked for `field`'s
    /// list into the draft.
    pub fn adopt_oracle_tag(&mut self, field: PickerField, slug: &str) -> Toast {
        let exclude = matches!(field, PickerField::Exclude(_));
        match super::adopt_oracle_tag(&mut self.draft, slug, exclude) {
            Adopted::Already => Toast::quick(ToastKind::Info, "Already in filter"),
            Adopted::Added => Toast::quick(ToastKind::Success, "Added to filter"),
        }
    }

    /// Writes what was typed in a number box into the draft as the box
    /// loses focus. `min` and `max` are both range boxes' text when `field`
    /// is one of them; a range needs both ends or neither.
    ///
    /// # Errors
    ///
    /// Returns the error toast for text that is not a number.
    pub fn set_number(&mut self, field: NumberField, text: &str, other: &str) -> Result<(), Toast> {
        let invalid = |message: &str| Toast::quick(ToastKind::Error, message);
        match field {
            NumberField::ManaValue => match parse_number(text) {
                Ok(None) => {
                    self.draft.unset_cmc_equals();
                }
                Ok(Some(n)) => {
                    self.draft.set_cmc_equals(n);
                }
                Err(_) => return Err(invalid("Invalid mana value")),
            },
            NumberField::ManaMin | NumberField::ManaMax => {
                let (min, max) = if field == NumberField::ManaMin {
                    (text, other)
                } else {
                    (other, text)
                };
                match (parse_number(min), parse_number(max)) {
                    (Ok(None), Ok(None)) => {
                        self.draft.unset_cmc_range();
                    }
                    (Ok(Some(min)), Ok(Some(max))) => {
                        self.draft.set_cmc_range((min, max));
                    }
                    (Ok(_), Ok(_)) => {}
                    _ => return Err(invalid("Invalid mana value range")),
                }
            }
            NumberField::PriceMin | NumberField::PriceMax => {
                let Ok(value) = parse_number(text) else {
                    return Err(invalid("Invalid price"));
                };
                match (field == NumberField::PriceMin, value) {
                    (true, None) => self.draft.unset_price_min(),
                    (true, Some(n)) => self.draft.set_price_min(n),
                    (false, None) => self.draft.unset_price_max(),
                    (false, Some(n)) => self.draft.set_price_max(n),
                };
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn search(validate: bool) -> SheetConfig {
        SheetConfig {
            validate,
            ..SheetConfig::default()
        }
    }

    #[test]
    fn apply_is_disabled_until_the_draft_changes_as_in_the_app() {
        let draft = FilterDraft::new(&CardQueryBuilder::default(), search(true));
        assert_eq!(
            draft.apply(),
            ApplyOutcome::Disabled,
            "no 'Filter is already empty' toast for an untouched empty search"
        );
        assert_eq!(draft.back(), None);
    }

    #[test]
    fn apply_commits_a_change_and_names_a_clear() {
        let mut draft = FilterDraft::new(&CardQueryBuilder::default(), search(false));
        draft.draft.set_name_contains("bolt".to_owned());
        let ApplyOutcome::Applied { filter, toast } = draft.apply() else {
            panic!("applies");
        };
        assert_eq!(filter.name_contains(), Some("bolt"));
        assert_eq!(toast.text, "Filter applied");

        let mut draft = FilterDraft::new(&filter, search(false));
        assert_eq!(draft.reset().text, "Filter reset");
        let ApplyOutcome::Applied { toast, .. } = draft.apply() else {
            panic!("applies");
        };
        assert_eq!(toast.text, "Filter cleared");
    }

    #[test]
    fn back_says_when_edits_drop() {
        let mut draft = FilterDraft::new(&CardQueryBuilder::default(), search(false));
        draft.draft.set_name_contains("x".to_owned());
        assert_eq!(
            draft.back().map(|t| t.text),
            Some("Filter unchanged".to_owned())
        );
    }

    #[test]
    fn number_boxes_parse_or_say_why_not() {
        let mut draft = FilterDraft::new(&CardQueryBuilder::default(), search(false));
        assert!(draft.set_number(NumberField::ManaValue, "3", "").is_ok());
        assert_eq!(draft.draft.cmc_equals(), Some(3.0));
        assert_eq!(
            draft
                .set_number(NumberField::PriceMin, "abc", "")
                .map_err(|t| t.text),
            Err("Invalid price".to_owned())
        );
        assert!(draft.set_number(NumberField::ManaMin, "2", "").is_ok());
        assert_eq!(draft.draft.cmc_range(), None, "a range needs both ends");
        assert!(draft.set_number(NumberField::ManaMax, "5", "2").is_ok());
        assert_eq!(draft.draft.cmc_range(), Some((2.0, 5.0)));
    }

    #[test]
    fn opening_warns_after_a_failed_list_and_claims_what_it_needs() {
        let mut catalog = Catalog::default();
        let now = Utc::now();
        let (_, opened) = FilterDraft::open(
            &CardQueryBuilder::default(),
            SheetConfig::default(),
            &mut catalog,
            now,
        );
        assert_eq!(opened.hint, OneTimeHint::Filter);
        assert_eq!(opened.toast, None);
        assert_eq!(opened.fetch.len(), Kind::FILTER.len());
        catalog.keywords.settle(None, now);
        let (_, opened) = FilterDraft::open(
            &CardQueryBuilder::default(),
            SheetConfig::default(),
            &mut catalog,
            now,
        );
        assert_eq!(
            opened.toast.map(|t| t.text),
            Some("Filter lists didn't load, trying again".to_owned())
        );
        assert_eq!(opened.fetch, [Kind::Keywords]);
    }
}

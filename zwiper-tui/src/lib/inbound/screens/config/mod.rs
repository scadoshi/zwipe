//! The Config screen: the theme with its Dark/Light button and the Themes sheet, the pip style, Universes Beyond and its Exceptions sheet.
//!
//! The toggles are optimistic, each with its own request count: the new value shows at once, an answer to a tap that a newer tap has overtaken is ignored, and a failure puts back the value from before that tap. Theme and pips show from the start; the Universes Beyond controls wait for the preferences to load, and stay off for good if they cannot.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use zwipe_core::domain::card::scryfall_data::universe::UbFranchiseView;
use zwipe_core::domain::user::hints::HINT_CONFIG;
use zwipe_core::domain::user::preferences::{PipStyle, UserPreferences};
use zwipe_core::http::contracts::user::HttpUpdatePreferences;

use super::{Screen, failure};
use crate::domain::theme::{self, Palette, PickerRow};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::widgets::hint::{self, HintTopic};
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::outbound::account::Account;

/// A change to one preference, as Config sends it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreferenceChange {
    /// A theme from the Themes sheet, keeping the mode.
    Theme {
        theme: String,
        dark_mode: bool,
    },
    /// The same theme in the other mode.
    DarkMode {
        theme: String,
        dark_mode: bool,
    },
    Pips(PipStyle),
    HideUniversesBeyond(bool),
    Exceptions(Vec<String>),
}

impl PreferenceChange {
    /// Applies the change to `preferences`, the way the screen shows it.
    pub(crate) fn apply(&self, preferences: &mut UserPreferences) {
        match self {
            Self::Theme { theme, dark_mode } | Self::DarkMode { theme, dark_mode } => {
                preferences.theme.clone_from(theme);
                preferences.dark_mode = *dark_mode;
            }
            Self::Pips(style) => preferences.pip_style = *style,
            Self::HideUniversesBeyond(hide) => preferences.exclude_universes_beyond = *hide,
            Self::Exceptions(slugs) => preferences.universes_beyond_exceptions.clone_from(slugs),
        }
    }

    /// The same preference as `preferences` hold it now, to put back if this change fails.
    pub(crate) fn undo(&self, preferences: &UserPreferences) -> Self {
        match self {
            Self::Theme { .. } => Self::Theme {
                theme: preferences.theme.clone(),
                dark_mode: preferences.dark_mode,
            },
            Self::DarkMode { .. } => Self::DarkMode {
                theme: preferences.theme.clone(),
                dark_mode: preferences.dark_mode,
            },
            Self::Pips(_) => Self::Pips(preferences.pip_style),
            Self::HideUniversesBeyond(_) => {
                Self::HideUniversesBeyond(preferences.exclude_universes_beyond)
            }
            Self::Exceptions(_) => {
                Self::Exceptions(preferences.universes_beyond_exceptions.clone())
            }
        }
    }

    /// The app's toast when the server takes the change, read from its answer.
    pub(crate) fn saved_message(&self, saved: &UserPreferences) -> &'static str {
        match self {
            Self::Theme { .. } => "Theme saved",
            Self::DarkMode { .. } if saved.dark_mode => "Dark mode on",
            Self::DarkMode { .. } => "Dark mode off",
            Self::Pips(_) => match saved.pip_style {
                PipStyle::Theme => "Pips in theme colors",
                PipStyle::Natural => "Pips in natural colors",
            },
            Self::HideUniversesBeyond(_) if saved.exclude_universes_beyond => {
                "Universes Beyond hidden"
            }
            Self::HideUniversesBeyond(_) => "Universes Beyond shown",
            Self::Exceptions(_) => "Exceptions saved",
        }
    }

    /// Which optimistic toggle this is, for its own request count; `None` for the two sheets' saves.
    fn toggle(&self) -> Option<usize> {
        match self {
            Self::DarkMode { .. } => Some(0),
            Self::Pips(_) => Some(1),
            Self::HideUniversesBeyond(_) => Some(2),
            Self::Theme { .. } | Self::Exceptions(_) => None,
        }
    }
}

impl From<PreferenceChange> for HttpUpdatePreferences {
    fn from(change: PreferenceChange) -> Self {
        let mut request = Self {
            theme: None,
            dark_mode: None,
            exclude_universes_beyond: None,
            universes_beyond_exceptions: None,
            pip_style: None,
        };
        match change {
            PreferenceChange::Theme { theme, dark_mode }
            | PreferenceChange::DarkMode { theme, dark_mode } => {
                request.theme = Some(theme);
                request.dark_mode = Some(dark_mode);
            }
            PreferenceChange::Pips(style) => request.pip_style = Some(style),
            PreferenceChange::HideUniversesBeyond(hide) => {
                request.exclude_universes_beyond = Some(hide);
            }
            PreferenceChange::Exceptions(slugs) => {
                request.universes_beyond_exceptions = Some(slugs);
            }
        }
        request
    }
}

/// A button on Config's rows, in focus order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stop {
    /// The Dark or Light button in the Theme row.
    Mode,
    /// The Theme row's Change, which opens the Themes sheet.
    Theme,
    Pips,
    UniversesBeyond,
    Exceptions,
}

impl Stop {
    pub(crate) const ALL: [Self; 5] = [
        Self::Mode,
        Self::Theme,
        Self::Pips,
        Self::UniversesBeyond,
        Self::Exceptions,
    ];
}

/// The Themes sheet: the theme it opened on, and the row under the selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThemeSheet {
    /// The theme the sheet opened on, which Back puts back.
    pub(crate) original: String,
    /// The selected row of [`theme::picker_rows`], always a theme.
    pub(crate) row: usize,
}

impl ThemeSheet {
    /// A sheet on `current`, or on the first theme when `current` is not listed.
    pub(crate) fn opening(current: &str) -> Self {
        let slug = theme::current_slug(current);
        let rows = theme::picker_rows();
        let row = rows
            .iter()
            .position(|r| matches!(r, PickerRow::Theme(s) if *s == slug))
            .or_else(|| rows.iter().position(|r| matches!(r, PickerRow::Theme(_))))
            .unwrap_or(0);
        Self {
            original: slug.to_owned(),
            row,
        }
    }

    /// The theme under the selection.
    pub(crate) fn theme(&self) -> Option<&'static str> {
        match theme::picker_rows().get(self.row) {
            Some(PickerRow::Theme(slug)) => Some(slug),
            _ => None,
        }
    }

    /// Whether a different theme than the one it opened on is picked.
    pub(crate) fn picked(&self) -> bool {
        self.theme().is_some_and(|t| t != self.original)
    }

    /// Moves to the next theme down or up, skipping section labels.
    pub(crate) fn step(&mut self, down: bool) {
        let rows = theme::picker_rows();
        let is_theme = |i: &usize| matches!(rows.get(*i), Some(PickerRow::Theme(_)));
        let found = if down {
            (self.row.saturating_add(1)..rows.len()).find(is_theme)
        } else {
            (0..self.row).rev().find(is_theme)
        };
        if let Some(row) = found {
            self.row = row;
        }
    }
}

/// The Exceptions sheet: the draft of picked franchises and the focused chip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExceptionsSheet {
    /// The picked slugs, in the order they were picked.
    pub(crate) draft: Vec<String>,
    pub(crate) chip: usize,
}

/// Whether two lists of slugs pick the same set, ignoring order.
pub(crate) fn same_set(a: &[String], b: &[String]) -> bool {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

/// What is open over Config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Sheet {
    Themes(ThemeSheet),
    Exceptions(ExceptionsSheet),
}

/// The Config screen's state.
#[derive(Debug)]
pub(crate) struct ConfigView {
    /// The preferences on screen.
    pub(crate) shown: UserPreferences,
    /// Whether the preferences loaded; the Universes Beyond controls wait on it.
    pub(crate) loaded: bool,
    pub(crate) focus: usize,
    /// Each toggle's request count: Dark/Light, pips, Universes Beyond.
    pub(crate) seqs: [u64; 3],
    pub(crate) sheet: Option<Sheet>,
    /// The franchises the exceptions can name, as the server lists them; empty until they load.
    pub(crate) franchises: Vec<UbFranchiseView>,
    /// Whether the Exceptions sheet has opened on this screen, for its one-time hint.
    pub(crate) exceptions_opened: bool,
}

impl ConfigView {
    /// Config on `shown`, the preferences as last applied.
    pub(crate) fn new(shown: UserPreferences) -> Self {
        Self {
            shown,
            loaded: false,
            focus: 0,
            seqs: [0; 3],
            sheet: None,
            franchises: Vec::new(),
            exceptions_opened: false,
        }
    }

    pub(crate) fn focused(&self) -> Stop {
        Stop::ALL.get(self.focus).copied().unwrap_or(Stop::Mode)
    }

    pub(crate) fn next(&mut self) {
        self.focus = self
            .focus
            .saturating_add(1)
            .min(Stop::ALL.len().saturating_sub(1));
    }

    pub(crate) fn previous(&mut self) {
        self.focus = self.focus.saturating_sub(1);
    }

    /// Whether `stop` can be pressed: the Universes Beyond button once loaded, and Exceptions once loaded while Universes Beyond is hidden.
    pub(crate) fn enabled(&self, stop: Stop) -> bool {
        match stop {
            Stop::UniversesBeyond => self.loaded,
            Stop::Exceptions => self.loaded && self.shown.exclude_universes_beyond,
            _ => true,
        }
    }

    /// Shows a toggle at once and numbers it; returns its count.
    pub(crate) fn begin(&mut self, change: &PreferenceChange) -> u64 {
        change.apply(&mut self.shown);
        let Some(slot) = change.toggle().and_then(|i| self.seqs.get_mut(i)) else {
            return 0;
        };
        *slot = slot.saturating_add(1);
        *slot
    }

    /// Whether `seq` is still the latest tap of `change`'s toggle.
    pub(crate) fn latest(&self, change: &PreferenceChange, seq: u64) -> bool {
        change
            .toggle()
            .and_then(|i| self.seqs.get(i))
            .is_none_or(|latest| *latest == seq)
    }
}

/// Config's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Fetches the preferences. Also sent at sign-in, to draw in the account's theme.
    LoadPreferences,
    /// Sends a change; `undo` puts back what it replaced if it fails.
    Save {
        seq: u64,
        change: PreferenceChange,
        undo: PreferenceChange,
    },
    /// Fetches the franchises the exceptions can name.
    LoadFranchises,
}

/// Config's results.
#[derive(Debug)]
pub(crate) enum Update {
    Preferences(Box<UserPreferences>),
    PreferencesUnavailable,
    Saved {
        seq: u64,
        change: PreferenceChange,
        preferences: Box<UserPreferences>,
    },
    SaveFailed {
        seq: u64,
        undo: PreferenceChange,
        message: String,
    },
    Franchises(Vec<UbFranchiseView>),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Config(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Config(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut ConfigView> {
    match screen {
        Screen::Config(view) => Some(view),
        _ => None,
    }
}

/// Opens Config on the preferences as last applied, fetches them and the franchises, and shows the `config` hint the first time.
pub(crate) fn open(cx: &mut Context<'_>) {
    let shown = cx.shared.preferences.clone().unwrap_or_default();
    cx.push(Screen::Config(Box::new(ConfigView::new(shown))));
    cx.request(Request::LoadPreferences.into());
    cx.request(Request::LoadFranchises.into());
    hint::once(cx, HINT_CONFIG);
}

/// The `?` topic of a stop's row.
pub(crate) fn topic(stop: Stop) -> Option<HintTopic> {
    match stop {
        Stop::Mode | Stop::Theme => Some(HintTopic::Theme),
        Stop::Pips => Some(HintTopic::Pips),
        Stop::UniversesBeyond => Some(HintTopic::UniversesBeyond),
        Stop::Exceptions => None,
    }
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    match request {
        Request::LoadPreferences => {
            match async { client.fetch_preferences(&account.fresh().await?).await }.await {
                Ok(preferences) => {
                    account.set_preferences(preferences.clone()).await;
                    Some(Update::Preferences(Box::new(preferences)).into())
                }
                Err(e) => Some(failure(&e, |_| Update::PreferencesUnavailable.into())),
            }
        }
        Request::Save { seq, change, undo } => {
            let result = async {
                let session = account.fresh().await?;
                client
                    .update_preferences(HttpUpdatePreferences::from(change.clone()), &session)
                    .await
            }
            .await;
            Some(match result {
                Ok(preferences) => {
                    account.set_preferences(preferences.clone()).await;
                    Update::Saved {
                        seq,
                        change,
                        preferences: Box::new(preferences),
                    }
                    .into()
                }
                Err(e) => failure(&e, |message| {
                    Update::SaveFailed { seq, undo, message }.into()
                }),
            })
        }
        Request::LoadFranchises => client
            .fetch_ub_franchises()
            .await
            .ok()
            .map(|franchises| Update::Franchises(franchises).into()),
    }
}

/// Draws in `preferences`' theme and pips, as the app's live theme.
fn show(cx: &mut Context<'_>, preferences: &UserPreferences) {
    cx.shared.palette = Palette::for_preferences(preferences);
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let view = nav.find_mut(pick);
    match update {
        Update::Preferences(preferences) => match view {
            // An open Config takes the Universes Beyond settings and the pips; the theme stays the live one.
            Some(view) => {
                view.shown.exclude_universes_beyond = preferences.exclude_universes_beyond;
                view.shown
                    .universes_beyond_exceptions
                    .clone_from(&preferences.universes_beyond_exceptions);
                view.shown.pip_style = preferences.pip_style;
                view.loaded = true;
            }
            None => cx.use_preferences(&preferences),
        },
        Update::PreferencesUnavailable => {}
        Update::Saved {
            seq,
            change,
            preferences,
        } => {
            let Some(view) = view else {
                cx.use_preferences(&preferences);
                return;
            };
            if !view.latest(&change, seq) {
                return;
            }
            let kept_theme = (view.shown.theme.clone(), view.shown.dark_mode);
            view.shown.clone_from(&preferences);
            if matches!(
                change,
                PreferenceChange::Pips(_)
                    | PreferenceChange::HideUniversesBeyond(_)
                    | PreferenceChange::Exceptions(_)
            ) {
                // A toggle elsewhere leaves a theme preview or a newer mode tap on screen.
                (view.shown.theme, view.shown.dark_mode) = kept_theme;
            }
            let shown = view.shown.clone();
            cx.use_preferences(&shown);
            let duration = match change {
                PreferenceChange::Theme { .. } | PreferenceChange::Exceptions(_) => toast::QUICK,
                _ => toast::NORMAL,
            };
            cx.toast_for(
                ToastKind::Success,
                change.saved_message(&preferences).to_owned(),
                duration,
            );
        }
        Update::SaveFailed { seq, undo, message } => {
            if let Some(view) = view
                && view.latest(&undo, seq)
                && undo.toggle().is_some()
            {
                undo.apply(&mut view.shown);
                let shown = view.shown.clone();
                show(cx, &shown);
            }
            cx.toast(ToastKind::Error, message);
        }
        Update::Franchises(franchises) => {
            if let Some(view) = view {
                view.franchises = franchises;
            }
        }
    }
}

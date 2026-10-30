//! The config screen: the preferences as saved and as shown, the selected row, and the theme and exceptions lists.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use zwipe_core::domain::card::scryfall_data::universe::UbFranchiseView;
use zwipe_core::domain::user::preferences::UserPreferences;
use zwipe_core::http::contracts::user::HttpUpdatePreferences;

use super::{Screen, failure};
use crate::domain::theme::{self, Palette, PickerRow};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::outbound::account::Account;

/// A change to one preference, as the config screen sends it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreferenceChange {
    /// A new theme from the picker, keeping the mode.
    Theme {
        theme: String,
        dark_mode: bool,
    },
    /// The same theme in the other mode.
    DarkMode {
        theme: String,
        dark_mode: bool,
    },
    Pips(zwipe_core::domain::user::preferences::PipStyle),
    HideUniversesBeyond(bool),
    Exceptions(Vec<String>),
}

impl PreferenceChange {
    /// Applies the change to `preferences`, the way the screen shows it before the server answers.
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

    /// The app's message when the server takes the change.
    pub(crate) fn saved_message(&self) -> &'static str {
        match self {
            Self::Theme { .. } => "Theme saved",
            Self::DarkMode {
                dark_mode: true, ..
            } => "Dark mode on",
            Self::DarkMode {
                dark_mode: false, ..
            } => "Dark mode off",
            Self::Pips(zwipe_core::domain::user::preferences::PipStyle::Theme) => {
                "Pips in theme colors"
            }
            Self::Pips(zwipe_core::domain::user::preferences::PipStyle::Natural) => {
                "Pips in natural colors"
            }
            Self::HideUniversesBeyond(true) => "Universes Beyond hidden",
            Self::HideUniversesBeyond(false) => "Universes Beyond shown",
            Self::Exceptions(_) => "Exceptions saved",
        }
    }
}

/// A row of the config screen, in the app's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConfigRow {
    Theme,
    Mode,
    Pips,
    UniversesBeyond,
    Exceptions,
}

impl ConfigRow {
    pub(crate) const ALL: [Self; 5] = [
        Self::Theme,
        Self::Mode,
        Self::Pips,
        Self::UniversesBeyond,
        Self::Exceptions,
    ];
}

/// The config screen.
#[derive(Debug, Default)]
pub(crate) struct ConfigView {
    /// The preferences as the server last confirmed them; `None` until they load.
    pub(crate) saved: Option<UserPreferences>,
    /// The preferences on screen, ahead of the server while a change is out.
    pub(crate) shown: Option<UserPreferences>,
    pub(crate) row: usize,
    /// Numbers each change, so only the answer to the latest one counts.
    pub(crate) seq: u64,
    pub(crate) picker: Option<ThemePicker>,
    pub(crate) exceptions: Option<ExceptionsEditor>,
}

impl ConfigView {
    pub(crate) fn selected(&self) -> ConfigRow {
        ConfigRow::ALL
            .get(self.row)
            .copied()
            .unwrap_or(ConfigRow::Theme)
    }

    pub(crate) fn next_row(&mut self) {
        self.row = self
            .row
            .saturating_add(1)
            .min(ConfigRow::ALL.len().saturating_sub(1));
    }

    pub(crate) fn previous_row(&mut self) {
        self.row = self.row.saturating_sub(1);
    }

    /// Whether the exceptions can be changed: only once the preferences have loaded, and only while Universes Beyond is hidden.
    pub(crate) fn exceptions_open(&self) -> bool {
        self.shown
            .as_ref()
            .is_some_and(|p| p.exclude_universes_beyond)
    }

    /// Shows `change` at once and numbers it. `None` before the preferences load.
    pub(crate) fn begin(&mut self, change: &PreferenceChange) -> Option<u64> {
        let shown = self.shown.as_mut()?;
        change.apply(shown);
        self.seq = self.seq.saturating_add(1);
        Some(self.seq)
    }

    /// Takes the server's preferences after change `seq`; an answer to an older change is ignored.
    pub(crate) fn confirm(&mut self, seq: u64, preferences: UserPreferences) -> bool {
        if seq != self.seq {
            return false;
        }
        self.saved = Some(preferences.clone());
        self.shown = Some(preferences);
        true
    }

    /// Puts back the last confirmed preferences after change `seq` failed; an older failure is ignored.
    pub(crate) fn revert(&mut self, seq: u64) -> bool {
        if seq != self.seq {
            return false;
        }
        self.shown.clone_from(&self.saved);
        true
    }
}

/// The theme list, previewing the theme under the selection.
#[derive(Debug)]
pub(crate) struct ThemePicker {
    /// The selected row of [`theme::picker_rows`], always a theme, never a header.
    pub(crate) row: usize,
    /// The theme the picker opened on, which Esc puts back.
    pub(crate) original: String,
}

impl ThemePicker {
    /// A picker on `current`, or on the first theme when `current` is not listed.
    pub(crate) fn opening(current: &str) -> Self {
        let slug = theme::current_slug(current);
        let rows = theme::picker_rows();
        let row = rows
            .iter()
            .position(|r| matches!(r, PickerRow::Theme(s) if *s == slug))
            .or_else(|| rows.iter().position(|r| matches!(r, PickerRow::Theme(_))))
            .unwrap_or(0);
        Self {
            row,
            original: current.to_owned(),
        }
    }

    /// The theme under the selection.
    pub(crate) fn theme(&self) -> Option<&'static str> {
        match theme::picker_rows().get(self.row) {
            Some(PickerRow::Theme(slug)) => Some(slug),
            _ => None,
        }
    }

    /// Moves to the next theme down, skipping headers.
    pub(crate) fn down(&mut self) {
        let rows = theme::picker_rows();
        if let Some(row) = (self.row.saturating_add(1)..rows.len())
            .find(|i| matches!(rows.get(*i), Some(PickerRow::Theme(_))))
        {
            self.row = row;
        }
    }

    /// Moves to the next theme up, skipping headers.
    pub(crate) fn up(&mut self) {
        let rows = theme::picker_rows();
        if let Some(row) = (0..self.row)
            .rev()
            .find(|i| matches!(rows.get(*i), Some(PickerRow::Theme(_))))
        {
            self.row = row;
        }
    }
}

/// The franchise exceptions being edited.
#[derive(Debug)]
pub(crate) struct ExceptionsEditor {
    /// Every franchise, as the server lists them; `None` until they load.
    pub(crate) franchises: Option<Vec<UbFranchiseView>>,
    /// The picked slugs, in the order they were picked.
    pub(crate) draft: Vec<String>,
    pub(crate) saved: Vec<String>,
    pub(crate) row: usize,
}

impl ExceptionsEditor {
    pub(crate) fn opening(saved: &[String]) -> Self {
        Self {
            franchises: None,
            draft: saved.to_vec(),
            saved: saved.to_vec(),
            row: 0,
        }
    }

    /// Whether the draft picks a different set than what is saved, ignoring order.
    pub(crate) fn changed(&self) -> bool {
        let mut draft = self.draft.clone();
        let mut saved = self.saved.clone();
        draft.sort_unstable();
        saved.sort_unstable();
        draft != saved
    }

    /// Picks or unpicks the franchise under the selection.
    pub(crate) fn toggle(&mut self) {
        let Some(slug) = self
            .franchises
            .as_ref()
            .and_then(|f| f.get(self.row))
            .map(|f| f.slug.clone())
        else {
            return;
        };
        if let Some(i) = self.draft.iter().position(|s| *s == slug) {
            self.draft.remove(i);
        } else {
            self.draft.push(slug);
        }
    }

    pub(crate) fn down(&mut self) {
        let count = self.franchises.as_ref().map_or(0, Vec::len);
        self.row = self.row.saturating_add(1).min(count.saturating_sub(1));
    }

    pub(crate) fn up(&mut self) {
        self.row = self.row.saturating_sub(1);
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

/// The config screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Fetches the preferences. Also sent at sign-in, to draw in the account's theme.
    LoadPreferences,
    SavePreferences {
        seq: u64,
        change: PreferenceChange,
    },
    /// Fetches the franchises the exceptions can name.
    LoadFranchises,
}

/// The config screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    Preferences(Box<UserPreferences>),
    PreferencesUnavailable,
    Saved {
        seq: u64,
        preferences: Box<UserPreferences>,
        message: &'static str,
    },
    SaveFailed {
        seq: u64,
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

/// Opens the config screen over the current one and fetches the preferences it shows.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.shared.status.clear();
    cx.push(Screen::Config(Box::default()));
    cx.request(Request::LoadPreferences.into());
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
                // A launch that cannot fetch the preferences keeps the session's copy.
                Err(e) => Some(failure(&e, |_| Update::PreferencesUnavailable.into())),
            }
        }
        Request::SavePreferences { seq, change } => {
            let message = change.saved_message();
            let result = async {
                let session = account.fresh().await?;
                client
                    .update_preferences(HttpUpdatePreferences::from(change), &session)
                    .await
            }
            .await;
            Some(match result {
                Ok(preferences) => {
                    account.set_preferences(preferences.clone()).await;
                    Update::Saved {
                        seq,
                        preferences: Box::new(preferences),
                        message,
                    }
                    .into()
                }
                Err(e) => failure(&e, |message| Update::SaveFailed { seq, message }.into()),
            })
        }
        Request::LoadFranchises => client
            .fetch_ub_franchises()
            .await
            .ok()
            .map(|franchises| Update::Franchises(franchises).into()),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let view = nav.find_mut(pick);
    match update {
        Update::Preferences(preferences) => {
            // Fills the config screen when it is waiting, and draws in the preferences either way.
            if let Some(view) = view {
                if view.saved.is_some() {
                    return;
                }
                view.saved = Some((*preferences).clone());
                view.shown = Some((*preferences).clone());
            }
            cx.use_preferences(&preferences);
        }
        Update::PreferencesUnavailable => {}
        Update::Saved {
            seq,
            preferences,
            message,
        } => {
            if let Some(view) = view
                && view.confirm(seq, (*preferences).clone())
            {
                message.clone_into(&mut cx.shared.status);
                cx.use_preferences(&preferences);
            }
        }
        Update::SaveFailed { seq, message } => {
            if let Some(view) = view
                && view.revert(seq)
            {
                if let Some(shown) = &view.shown {
                    cx.shared.palette = Palette::for_preferences(shown);
                }
                cx.shared.status = message;
            }
        }
        Update::Franchises(franchises) => {
            if let Some(editor) = view.and_then(|view| view.exceptions.as_mut()) {
                editor.franchises = Some(franchises);
            }
        }
    }
}

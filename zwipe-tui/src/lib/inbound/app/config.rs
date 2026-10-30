//! The config screen's state: the preferences as saved and as shown, the selected row, and the open picker.

use zwipe_core::domain::card::scryfall_data::universe::UbFranchiseView;
use zwipe_core::domain::user::preferences::UserPreferences;

use crate::domain::theme::{self, PickerRow};

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

#[cfg(test)]
mod tests {
    use zwipe_core::domain::user::preferences::PipStyle;

    use super::*;

    fn loaded() -> ConfigView {
        ConfigView {
            saved: Some(UserPreferences::default()),
            shown: Some(UserPreferences::default()),
            ..ConfigView::default()
        }
    }

    #[test]
    fn a_change_shows_at_once_and_a_failure_puts_it_back() {
        let mut view = loaded();
        let seq = view
            .begin(&PreferenceChange::Pips(PipStyle::Natural))
            .unwrap();
        assert_eq!(view.shown.as_ref().unwrap().pip_style, PipStyle::Natural);
        assert!(view.revert(seq));
        assert_eq!(view.shown.as_ref().unwrap().pip_style, PipStyle::Theme);
    }

    #[test]
    fn only_the_latest_change_is_answered() {
        let mut view = loaded();
        let first = view
            .begin(&PreferenceChange::HideUniversesBeyond(true))
            .unwrap();
        let second = view
            .begin(&PreferenceChange::HideUniversesBeyond(false))
            .unwrap();
        assert!(
            !view.revert(first),
            "a late failure of an older change is ignored"
        );
        assert!(!view.shown.as_ref().unwrap().exclude_universes_beyond);
        let answer = UserPreferences::default();
        assert!(view.confirm(second, answer));
    }

    #[test]
    fn nothing_changes_before_the_preferences_load() {
        assert_eq!(
            ConfigView::default().begin(&PreferenceChange::Pips(PipStyle::Natural)),
            None
        );
        assert!(!ConfigView::default().exceptions_open());
    }

    #[test]
    fn the_picker_opens_on_the_current_theme_and_skips_headers() {
        let mut picker = ThemePicker::opening("gruvbox");
        assert_eq!(picker.theme(), Some("gruvbox"));
        let renamed = ThemePicker::opening("ristretto");
        assert_eq!(renamed.theme(), Some("monokai-pro-ristretto"));
        for _ in 0..200 {
            picker.down();
        }
        assert!(picker.theme().is_some());
        for _ in 0..200 {
            picker.up();
        }
        assert_eq!(
            picker.theme(),
            Some("darcula"),
            "the first theme, past the first header"
        );
    }

    #[test]
    fn exceptions_toggle_in_pick_order_and_compare_as_a_set() {
        let franchise = |slug: &str| UbFranchiseView {
            slug: slug.to_owned(),
            name: slug.to_uppercase(),
        };
        let mut editor = ExceptionsEditor::opening(&["b".to_owned()]);
        editor.franchises = Some(vec![franchise("a"), franchise("b")]);
        editor.toggle();
        assert_eq!(editor.draft, ["b", "a"]);
        assert!(editor.changed());
        editor.down();
        editor.toggle();
        editor.toggle();
        assert_eq!(editor.draft, ["a", "b"]);
        editor.up();
        editor.toggle();
        assert!(!editor.changed(), "back to just b");
    }
}

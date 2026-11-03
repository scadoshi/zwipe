//! Config's rules: the changes a client sends, the sparse update request
//! each one makes, the optimistic toggles that ignore stale replies and roll
//! back on failure, the Universes Beyond exception chips, and every toast
//! Config raises, in the app's words, kinds and bands.
//!
//! Three Config buttons are optimistic toggles: Dark/Light, pips and
//! Universes Beyond. Each shows its new value at once and sends it. Every
//! tap takes a [`Ticket`] from that toggle's own [`Generation`]; when the
//! reply lands, [`settle`] says what the toggle shows, or that a newer tap
//! has overtaken it. The Themes and Exceptions sheets save without the
//! guard and change nothing until the server answers.

use zwipe_core::domain::card::scryfall_data::universe;
use zwipe_core::domain::user::preferences::{PipColor, UserPreferences};
use zwipe_core::http::contracts::user::HttpUpdatePreferences;

use crate::generation::{Generation, Ticket};
use crate::toast::{Toast, ToastKind};

/// A change to one preference, as Config sends it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PreferenceChange {
    /// A theme kept in the Themes sheet, in the mode it was picked in.
    Theme {
        /// The theme's slug.
        theme: String,
        /// Whether it is the dark variant.
        dark_mode: bool,
    },
    /// The Theme row's Dark/Light button: the same theme in the other mode.
    DarkMode {
        /// The current theme's slug, sent along with the mode.
        theme: String,
        /// Whether the new mode is dark.
        dark_mode: bool,
    },
    /// The Pips button.
    Pips(PipColor),
    /// The Universes Beyond button: `true` hides them.
    HideUniversesBeyond(bool),
    /// The Exceptions sheet's Save: the whole list of franchise slugs.
    Exceptions(Vec<String>),
}

impl PreferenceChange {
    /// Writes the change into `preferences`, the way the screen shows it
    /// before the server answers.
    pub fn apply(&self, preferences: &mut UserPreferences) {
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

    /// The same preference as `preferences` hold it now, so applying it puts
    /// back what this change replaces. Take it before [`apply`](Self::apply).
    #[must_use]
    pub fn undo(&self, preferences: &UserPreferences) -> Self {
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

    /// Which optimistic toggle sent this change; `None` for the two sheets'
    /// saves, which are not optimistic.
    #[must_use]
    pub fn toggle(&self) -> Option<Toggle> {
        match self {
            Self::DarkMode { .. } => Some(Toggle::DarkMode),
            Self::Pips(_) => Some(Toggle::Pips),
            Self::HideUniversesBeyond(_) => Some(Toggle::UniversesBeyond),
            Self::Theme { .. } | Self::Exceptions(_) => None,
        }
    }

    /// The words the app says once the server takes the change, read from
    /// `saved`, the preferences it answered with.
    #[must_use]
    pub fn saved_message(&self, saved: &UserPreferences) -> &'static str {
        match self {
            Self::Theme { .. } => "Theme saved",
            Self::DarkMode { .. } if saved.dark_mode => "Dark mode on",
            Self::DarkMode { .. } => "Dark mode off",
            Self::Pips(_) => match saved.pip_style {
                PipColor::Theme => "Pips in theme colors",
                PipColor::Classic => "Pips in classic colors",
            },
            Self::HideUniversesBeyond(_) if saved.exclude_universes_beyond => {
                "Universes Beyond hidden"
            }
            Self::HideUniversesBeyond(_) => "Universes Beyond shown",
            Self::Exceptions(_) => "Exceptions saved",
        }
    }

    /// The toast once the server takes the change: success, with
    /// [`saved_message`](Self::saved_message)'s words. A sheet's save is up
    /// for [`QUICK`](crate::toast::QUICK); a toggle's is up for
    /// [`NORMAL`](crate::toast::NORMAL), since it changes something Config
    /// cannot show.
    #[must_use]
    pub fn saved_toast(&self, saved: &UserPreferences) -> Toast {
        let message = self.saved_message(saved);
        match self.toggle() {
            Some(_) => Toast::normal(ToastKind::Success, message),
            None => Toast::quick(ToastKind::Success, message),
        }
    }
}

impl From<PreferenceChange> for HttpUpdatePreferences {
    /// The sparse update that sends only what the change sets. A theme and
    /// a mode always go together.
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

/// One of Config's optimistic toggles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Toggle {
    /// The Theme row's Dark/Light button.
    DarkMode,
    /// The Pips button.
    Pips,
    /// The Universes Beyond Show/Hide button.
    UniversesBeyond,
}

/// Each toggle's own [`Generation`], for a client that keeps them together.
/// A tap on one toggle never makes another toggle's reply stale.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Toggles {
    dark_mode: Generation,
    pips: Generation,
    universes_beyond: Generation,
}

impl Toggles {
    /// The generation `toggle`'s taps count in.
    #[must_use]
    pub fn generation(self, toggle: Toggle) -> Generation {
        match toggle {
            Toggle::DarkMode => self.dark_mode,
            Toggle::Pips => self.pips,
            Toggle::UniversesBeyond => self.universes_beyond,
        }
    }

    /// Takes a ticket for a new tap on `toggle`, which is now its latest.
    pub fn next_ticket(&mut self, toggle: Toggle) -> Ticket {
        match toggle {
            Toggle::DarkMode => self.dark_mode.next_ticket(),
            Toggle::Pips => self.pips.next_ticket(),
            Toggle::UniversesBeyond => self.universes_beyond.next_ticket(),
        }
    }

    /// Whether `ticket` is still `toggle`'s latest tap.
    #[must_use]
    pub fn is_current(self, toggle: Toggle, ticket: Ticket) -> bool {
        self.generation(toggle).is_current(ticket)
    }
}

/// What a toggle shows once the request holding `ticket` answers: the
/// server's value in `reply`, or `prev` (the value from before that tap)
/// when the request failed. `None` leaves the toggle alone, since a newer
/// tap's request owns it now.
#[must_use]
pub fn settle<T>(generation: Generation, ticket: Ticket, reply: Option<T>, prev: T) -> Option<T> {
    generation.is_current(ticket).then(|| reply.unwrap_or(prev))
}

/// The saved exceptions as the Exceptions row's chips show them: franchise
/// names, alphabetical, with slugs this build does not know dropped.
#[must_use]
pub fn exception_names<S: AsRef<str>>(slugs: &[S]) -> Vec<&'static str> {
    let mut names: Vec<&'static str> = slugs
        .iter()
        .filter_map(|slug| universe::franchise_by_slug(slug.as_ref()))
        .map(|franchise| franchise.name)
        .collect();
    names.sort_unstable();
    names
}

/// The toast when the Themes sheet closes without a new pick: info, up for
/// [`QUICK`](crate::toast::QUICK).
#[must_use]
pub fn theme_unchanged() -> Toast {
    Toast::quick(ToastKind::Info, "Theme unchanged")
}

/// The toast when the Exceptions sheet closes on a draft that differs from
/// the saved list: info, up for [`QUICK`](crate::toast::QUICK). A draft that
/// matches says nothing.
#[must_use]
pub fn exceptions_unchanged() -> Toast {
    Toast::quick(ToastKind::Info, "Exceptions unchanged")
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::toast::{NORMAL, QUICK};

    fn preferences() -> UserPreferences {
        UserPreferences {
            theme: "gruvbox".to_owned(),
            dark_mode: true,
            exclude_universes_beyond: false,
            universes_beyond_exceptions: vec!["a".to_owned()],
            pip_style: PipColor::Theme,
        }
    }

    fn slugs(items: &[&str]) -> Vec<String> {
        items.iter().map(ToString::to_string).collect()
    }

    fn every_change() -> Vec<PreferenceChange> {
        vec![
            PreferenceChange::Theme {
                theme: "dracula".to_owned(),
                dark_mode: false,
            },
            PreferenceChange::DarkMode {
                theme: "gruvbox".to_owned(),
                dark_mode: false,
            },
            PreferenceChange::Pips(PipColor::Classic),
            PreferenceChange::HideUniversesBeyond(true),
            PreferenceChange::Exceptions(slugs(&["b", "c"])),
        ]
    }

    #[test]
    fn a_reply_overtaken_by_a_newer_tap_changes_nothing() {
        let mut generation = Generation::default();
        let first = generation.next_ticket();
        let second = generation.next_ticket();
        assert_eq!(
            settle(generation, second, Some(false), true),
            Some(false),
            "the newest reply lands"
        );
        assert_eq!(
            settle(generation, first, Some(true), false),
            None,
            "a late older reply is dropped"
        );
        assert_eq!(
            settle(generation, first, None, false),
            None,
            "and so is a late older failure"
        );
    }

    #[test]
    fn the_latest_failure_rolls_back_to_the_value_before_its_tap() {
        let mut generation = Generation::default();
        let _first = generation.next_ticket();
        let second = generation.next_ticket();
        assert_eq!(settle(generation, second, None, true), Some(true));
    }

    #[test]
    fn toggles_count_their_taps_apart() {
        let mut toggles = Toggles::default();
        let dark = toggles.next_ticket(Toggle::DarkMode);
        let pips = toggles.next_ticket(Toggle::Pips);
        toggles.next_ticket(Toggle::UniversesBeyond);
        assert!(toggles.is_current(Toggle::DarkMode, dark));
        assert!(toggles.is_current(Toggle::Pips, pips));
        let newer = toggles.next_ticket(Toggle::DarkMode);
        assert!(!toggles.is_current(Toggle::DarkMode, dark));
        assert!(toggles.is_current(Toggle::DarkMode, newer));
        assert!(toggles.is_current(Toggle::Pips, pips), "pips went stale");
    }

    #[test]
    fn undo_puts_back_what_the_change_replaced() {
        for change in every_change() {
            let before = preferences();
            let undo = change.undo(&before);
            let mut shown = before.clone();
            change.apply(&mut shown);
            assert_ne!(shown, before, "{change:?} changed nothing");
            undo.apply(&mut shown);
            assert_eq!(shown, before, "{change:?}");
        }
    }

    #[test]
    fn only_the_three_buttons_are_toggles() {
        let toggles: Vec<Option<Toggle>> = every_change()
            .iter()
            .map(PreferenceChange::toggle)
            .collect();
        assert_eq!(
            toggles,
            [
                None,
                Some(Toggle::DarkMode),
                Some(Toggle::Pips),
                Some(Toggle::UniversesBeyond),
                None,
            ]
        );
    }

    #[test]
    fn each_saved_toast_has_the_apps_words_kind_and_band() {
        let mut on = preferences();
        on.dark_mode = true;
        on.pip_style = PipColor::Theme;
        on.exclude_universes_beyond = true;
        let mut off = preferences();
        off.dark_mode = false;
        off.pip_style = PipColor::Classic;
        off.exclude_universes_beyond = false;

        let theme = PreferenceChange::Theme {
            theme: "dracula".to_owned(),
            dark_mode: true,
        };
        let dark = PreferenceChange::DarkMode {
            theme: "gruvbox".to_owned(),
            dark_mode: true,
        };
        let pips = PreferenceChange::Pips(PipColor::Theme);
        let universes = PreferenceChange::HideUniversesBeyond(true);
        let exceptions = PreferenceChange::Exceptions(Vec::new());
        let cases: [(&PreferenceChange, &UserPreferences, &str, Duration); 9] = [
            (&theme, &on, "Theme saved", QUICK),
            (&dark, &on, "Dark mode on", NORMAL),
            (&dark, &off, "Dark mode off", NORMAL),
            (&pips, &on, "Pips in theme colors", NORMAL),
            (&pips, &off, "Pips in classic colors", NORMAL),
            (&universes, &on, "Universes Beyond hidden", NORMAL),
            (&universes, &off, "Universes Beyond shown", NORMAL),
            (&exceptions, &on, "Exceptions saved", QUICK),
            (&exceptions, &off, "Exceptions saved", QUICK),
        ];
        for (change, saved, words, band) in cases {
            assert_eq!(
                change.saved_toast(saved),
                Toast::new(ToastKind::Success, words, band),
                "{change:?}"
            );
        }
    }

    #[test]
    fn the_saved_words_follow_the_reply_not_the_request() {
        let mut saved = preferences();
        saved.dark_mode = false;
        let asked_for_dark = PreferenceChange::DarkMode {
            theme: "gruvbox".to_owned(),
            dark_mode: true,
        };
        assert_eq!(asked_for_dark.saved_message(&saved), "Dark mode off");
    }

    #[test]
    fn the_unchanged_toasts_have_the_apps_words_kind_and_band() {
        assert_eq!(
            theme_unchanged(),
            Toast::new(ToastKind::Info, "Theme unchanged", QUICK)
        );
        assert_eq!(
            exceptions_unchanged(),
            Toast::new(ToastKind::Info, "Exceptions unchanged", QUICK)
        );
    }

    #[test]
    fn each_change_sends_only_its_own_fields() {
        let request = HttpUpdatePreferences::from(PreferenceChange::DarkMode {
            theme: "gruvbox".to_owned(),
            dark_mode: false,
        });
        assert_eq!(request.theme.as_deref(), Some("gruvbox"));
        assert_eq!(request.dark_mode, Some(false));
        assert_eq!(request.exclude_universes_beyond, None);
        assert_eq!(request.universes_beyond_exceptions, None);
        assert_eq!(request.pip_style, None);

        let request = HttpUpdatePreferences::from(PreferenceChange::Theme {
            theme: "dracula".to_owned(),
            dark_mode: true,
        });
        assert_eq!(request.theme.as_deref(), Some("dracula"));
        assert_eq!(request.dark_mode, Some(true));
        assert_eq!(request.pip_style, None);

        let request = HttpUpdatePreferences::from(PreferenceChange::Pips(PipColor::Classic));
        assert_eq!(request.pip_style, Some(PipColor::Classic));
        assert_eq!((request.theme, request.dark_mode), (None, None));
        assert_eq!(request.exclude_universes_beyond, None);
        assert_eq!(request.universes_beyond_exceptions, None);

        let request = HttpUpdatePreferences::from(PreferenceChange::HideUniversesBeyond(true));
        assert_eq!(request.exclude_universes_beyond, Some(true));
        assert_eq!((request.theme, request.dark_mode), (None, None));
        assert_eq!(request.universes_beyond_exceptions, None);
        assert_eq!(request.pip_style, None);

        let request = HttpUpdatePreferences::from(PreferenceChange::Exceptions(slugs(&["b"])));
        assert_eq!(request.universes_beyond_exceptions, Some(slugs(&["b"])));
        assert_eq!((request.theme, request.dark_mode), (None, None));
        assert_eq!(request.exclude_universes_beyond, None);
        assert_eq!(request.pip_style, None);
    }

    #[test]
    fn unknown_slugs_drop_out_of_the_chips() {
        assert_eq!(
            exception_names(&slugs(&["not-a-franchise"])),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn known_slugs_show_by_name_in_order() {
        let mut known: Vec<String> = universe::selectable_franchises()
            .take(3)
            .map(|f| f.slug.to_owned())
            .collect();
        known.reverse();
        known.push("not-a-franchise".to_owned());
        let names = exception_names(&known);
        assert_eq!(names.len(), 3, "{names:?}");
        assert!(names.is_sorted(), "{names:?}");
    }
}

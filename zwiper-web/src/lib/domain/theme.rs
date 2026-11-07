//! The theme a launch starts in, and which copy wins when the server's
//! settings arrive.
//!
//! This browser keeps two things in its theme store: the theme it last
//! showed, written on every change, and the server's theme as it last saw
//! it. A launch starts in the theme it last showed. When the server's
//! settings land, the one that changed more recently wins: the server's
//! copy, when it moved since this browser last saw it (another device chose
//! it); else this browser's, which then goes up to the server.

use components::ThemeConfig;
use zwipe_core::domain::user::models::settings::UserSettings;
pub use zwipe_ui::theme_sync::{ThemeSync, reconcile};

/// The theme this browser last showed first, else a live session's
/// settings, else the default. A retired slug in the settings moves to its
/// replacement and an unknown one falls back to the default.
pub fn initial_theme(settings: Option<&UserSettings>, stored: Option<ThemeConfig>) -> ThemeConfig {
    stored
        .or_else(|| settings.map(components::theme_from_settings))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme(name: &str, is_dark: bool) -> ThemeConfig {
        ThemeConfig {
            name: name.to_string(),
            is_dark,
        }
    }

    fn settings(name: &str, dark_mode: bool) -> UserSettings {
        UserSettings {
            theme: name.to_string(),
            dark_mode,
            ..UserSettings::default()
        }
    }

    #[test]
    fn a_launch_starts_in_the_theme_it_last_showed_over_the_sessions_copy() {
        // The session's copy is from the last login or refresh; the store
        // holds the toggle made since.
        let shown = initial_theme(Some(&settings("nord", true)), Some(theme("nord", false)));
        assert_eq!(shown, theme("nord", false));
    }

    #[test]
    fn with_nothing_stored_the_session_decides() {
        let shown = initial_theme(Some(&settings("nord", true)), None);
        assert_eq!(shown, theme("nord", true));
    }

    #[test]
    fn nothing_known_starts_in_the_default() {
        assert_eq!(initial_theme(None, None), ThemeConfig::default());
    }
}

//! The theme a launch starts in.

use components::ThemeConfig;
use zwipe_core::domain::user::models::preferences::UserPreferences;

/// A live session's preferences first, else the theme this browser used
/// last, else the default. A retired slug in the preferences moves to its
/// replacement and an unknown one falls back to the default.
pub fn initial_theme(
    preferences: Option<&UserPreferences>,
    stored: Option<ThemeConfig>,
) -> ThemeConfig {
    preferences
        .map(components::theme_from_preferences)
        .or(stored)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stored() -> ThemeConfig {
        ThemeConfig {
            name: "flexoki".to_string(),
            is_dark: false,
        }
    }

    #[test]
    fn the_session_wins_over_the_stored_theme() {
        let preferences = UserPreferences {
            theme: "nord".to_string(),
            dark_mode: true,
            ..UserPreferences::default()
        };
        let theme = initial_theme(Some(&preferences), Some(stored()));
        assert_eq!(theme.name, "nord");
        assert!(theme.is_dark);
    }

    #[test]
    fn logged_out_starts_in_the_stored_theme() {
        assert_eq!(initial_theme(None, Some(stored())), stored());
    }

    #[test]
    fn nothing_known_starts_in_the_default() {
        assert_eq!(initial_theme(None, None), ThemeConfig::default());
    }
}

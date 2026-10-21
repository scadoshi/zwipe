//! The theme a user saved, as the theme the UI kit displays.
//!
//! The hosts hold one theme type, orangui's [`ThemeConfig`], since every theme
//! hook and picker takes it. zwipe-core keeps its own copy of the theme list
//! because zerver validates a saved theme against it and core cannot depend on
//! a UI crate; the test below keeps the two lists equal. A `From` impl cannot
//! join two foreign types (the orphan rule), so the conversion is a function.

use orangui::ThemeConfig;
use zwipe_core::domain::user::models::preferences::UserPreferences;

/// The theme a user's saved preferences select. A retired slug moves to its
/// replacement, and one this build does not know (a theme added after it)
/// falls back to the default.
#[must_use]
pub fn theme_from_preferences(prefs: &UserPreferences) -> ThemeConfig {
    ThemeConfig {
        name: prefs.theme.clone(),
        is_dark: prefs.dark_mode,
    }
    .resolve()
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::user::preferences as core;

    #[test]
    fn core_theme_list_matches_orangui() {
        assert_eq!(core::ALLOWED_THEMES, orangui::ALLOWED_THEMES);
        assert_eq!(core::COLORBLIND_THEMES, orangui::COLORBLIND_THEMES);
        assert_eq!(core::RENAMED_THEMES, orangui::RETIRED_THEMES);
    }

    #[test]
    fn saved_preferences_always_name_a_theme_with_a_palette() {
        let prefs = |theme: &str| core::UserPreferences {
            theme: theme.to_string(),
            ..core::UserPreferences::default()
        };
        let name = |theme: &str| super::theme_from_preferences(&prefs(theme)).name;
        assert_eq!(name("flexoki"), "flexoki");
        assert_eq!(name("visual-studio"), "vscode");
        assert_eq!(name("a-theme-from-a-newer-build"), "gruvbox");
    }

    #[test]
    fn core_theme_names_match_orangui() {
        for slug in orangui::ALLOWED_THEMES {
            assert_eq!(
                core::display_theme_name(slug),
                orangui::display_theme_name(slug),
                "{slug}"
            );
        }
    }
}

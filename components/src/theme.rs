//! The theme a user saved, as the theme the UI kit displays.
//!
//! The hosts hold one theme type, orangui's [`ThemeConfig`], since every theme
//! hook and picker takes it. zwipe-core keeps its own copy of the theme list
//! because zerver validates a saved theme against it and core cannot depend on
//! a UI crate; the test below keeps the two lists equal. A `From` impl cannot
//! join two foreign types (the orphan rule), so the conversion is a function.

use orangui::ThemeConfig;
use zwipe_core::domain::user::models::preferences::UserPreferences;

/// The theme a user's saved preferences select.
#[must_use]
pub fn theme_from_preferences(prefs: &UserPreferences) -> ThemeConfig {
    ThemeConfig {
        name: prefs.theme.clone(),
        is_dark: prefs.dark_mode,
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::user::preferences as core;

    #[test]
    fn core_theme_list_matches_orangui() {
        assert_eq!(core::ALLOWED_THEMES, orangui::ALLOWED_THEMES);
        assert_eq!(core::COLORBLIND_THEMES, orangui::COLORBLIND_THEMES);
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

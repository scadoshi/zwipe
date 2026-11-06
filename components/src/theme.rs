//! The theme a user saved, as the theme the UI kit displays.
//!
//! The hosts hold one theme type, orangui's [`ThemeConfig`], since every theme
//! hook and picker takes it. zwipe-core keeps its own copy of the theme list
//! because zerver validates a saved theme against it and core cannot depend on
//! a UI crate; the test below keeps the two lists equal. A `From` impl cannot
//! join two foreign types (the orphan rule), so the conversion is a function.

use orangui::ThemeConfig;
use zwipe_core::domain::user::models::settings::{PipStyle, UserSettings};

/// The theme a user's saved settings select. A retired slug moves to its
/// replacement, and one this build does not know (a theme added after it)
/// falls back to the default.
#[must_use]
pub fn theme_from_settings(settings: &UserSettings) -> ThemeConfig {
    ThemeConfig {
        name: settings.theme.clone(),
        is_dark: settings.dark_mode,
    }
    .resolve()
    .unwrap_or_default()
}

/// The class a pip color adds beside the theme class: `pips-classic` puts
/// back the classic fills in `magic.css`, and the theme's own colors need no
/// class.
#[must_use]
pub const fn pip_style_class(color: PipStyle) -> &'static str {
    match color {
        PipStyle::Theme => "",
        PipStyle::Classic => "pips-classic",
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::user::settings as core;

    #[test]
    fn the_classic_pip_class_is_styled() {
        let class = super::pip_style_class(core::PipStyle::Classic);
        assert!(
            crate::MAGIC_CSS.contains(&format!(".{class} {{")),
            "magic.css has no .{class} rule"
        );
        assert_eq!(super::pip_style_class(core::PipStyle::Theme), "");
    }

    #[test]
    fn core_theme_list_matches_orangui() {
        assert_eq!(core::ALLOWED_THEMES, orangui::ALLOWED_THEMES);
        assert_eq!(core::COLORBLIND_THEMES, orangui::COLORBLIND_THEMES);
        assert_eq!(core::RENAMED_THEMES, orangui::RETIRED_THEMES);
    }

    #[test]
    fn saved_settings_always_name_a_theme_with_a_palette() {
        let settings = |theme: &str| core::UserSettings {
            theme: theme.to_string(),
            ..core::UserSettings::default()
        };
        let name = |theme: &str| super::theme_from_settings(&settings(theme)).name;
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

//! User display preferences (theme, dark mode).
//!
//! Preferences are stored per-user and embedded in JWT claims for instant
//! application on login. Users without a preferences row get defaults.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Allowed theme identifiers. Validated on update.
pub const ALLOWED_THEMES: &[&str] = &[
    "achromatopsia",
    "ayu",
    "catppuccin",
    "deuteranopia",
    "docs-rs",
    "dracula",
    "ethereal",
    "everforest",
    "github",
    "gruvbox",
    "hackerman",
    "kanagawa",
    "matte-black",
    "miasma",
    "monokai",
    "night-owl",
    "nord",
    "one-dark",
    "osaka-jade",
    "powershell",
    "protanopia",
    "ristretto",
    "rose-pine",
    "rustbox",
    "solarized",
    "synthwave-84",
    "tokyo-night",
    "tritanopia",
    "vantablack",
    "vscode",
    "zenburn",
];

/// The themes with palettes adjusted for color-vision deficiency. Every theme
/// picker groups these in their own section at the bottom, so the list lives
/// here once.
pub const COLORBLIND_THEMES: &[&str] =
    &["protanopia", "deuteranopia", "tritanopia", "achromatopsia"];

/// Human-readable label for a theme slug: title-cased words, with the
/// accents and brand casings that title-casing cannot produce special-cased.
/// One copy here, beside the list it labels, so zwiper and the site cannot
/// drift on how a theme is spelled.
#[must_use]
pub fn display_theme_name(slug: &str) -> String {
    match slug {
        "rose-pine" => return "Rosé Pine".to_string(),
        "vscode" => return "VS Code".to_string(),
        "github" => return "GitHub".to_string(),
        "synthwave-84" => return "Synthwave '84".to_string(),
        "powershell" => return "PowerShell".to_string(),
        "docs-rs" => return "docs.rs".to_string(),
        _ => {}
    }
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// User display preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Theme identifier (e.g. "gruvbox", "dracula").
    pub theme: String,
    /// Whether dark mode is active.
    pub dark_mode: bool,
    /// Hide Universes Beyond cards (no in-universe printing) from card
    /// serving and commander select. serde(default) keeps JWT claims and
    /// payloads minted before this field existed deserializing.
    #[serde(default)]
    pub exclude_universes_beyond: bool,
    /// Franchise slugs served anyway while the exclusion is on
    /// (`card::scryfall_data::universe::FRANCHISES`). Kept when the exclusion
    /// is toggled off so re-enabling restores the whitelist.
    #[serde(default)]
    pub universes_beyond_exceptions: Vec<String>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "gruvbox".to_string(),
            dark_mode: true,
            exclude_universes_beyond: false,
            universes_beyond_exceptions: Vec::new(),
        }
    }
}

/// Validated request to update a user's preferences.
///
/// Uses `Option<T>` for partial update semantics: `None` means unchanged.
#[derive(Debug)]
pub struct UpdatePreferences {
    /// User to update.
    pub user_id: Uuid,
    /// New theme identifier, or `None` to leave unchanged.
    pub theme: Option<String>,
    /// New dark mode setting, or `None` to leave unchanged.
    pub dark_mode: Option<bool>,
    /// New Universes Beyond exclusion, or `None` to leave unchanged.
    pub exclude_universes_beyond: Option<bool>,
    /// New exception whitelist (replaces the whole list), or `None` to leave
    /// unchanged. Slugs are validated against `universe::FRANCHISES`.
    pub universes_beyond_exceptions: Option<Vec<String>>,
}

impl UpdatePreferences {
    /// Validates and constructs the request.
    pub fn new(
        user_id: Uuid,
        theme: Option<&str>,
        dark_mode: Option<bool>,
        exclude_universes_beyond: Option<bool>,
        universes_beyond_exceptions: Option<Vec<String>>,
    ) -> Result<Self, InvalidUpdatePreferences> {
        if let Some(theme) = theme
            && !ALLOWED_THEMES.contains(&theme)
        {
            return Err(InvalidUpdatePreferences::InvalidTheme);
        }
        if let Some(slugs) = &universes_beyond_exceptions
            && slugs.iter().any(|s| {
                crate::domain::card::scryfall_data::universe::franchise_by_slug(s).is_none()
            })
        {
            return Err(InvalidUpdatePreferences::InvalidFranchise);
        }
        Ok(Self {
            user_id,
            theme: theme.map(|t| t.to_string()),
            dark_mode,
            exclude_universes_beyond,
            universes_beyond_exceptions,
        })
    }
}

/// Validation error for preference updates.
#[derive(Debug, Error)]
pub enum InvalidUpdatePreferences {
    /// Theme is not in the allowed list.
    #[error("invalid theme")]
    InvalidTheme,
    /// An exception slug is not a known Universes Beyond franchise.
    #[error("invalid franchise")]
    InvalidFranchise,
}

#[cfg(test)]
mod tests {
    use super::*;

    // == default ==

    #[test]
    fn theme_names_title_case_and_keep_brand_spellings() {
        assert_eq!(display_theme_name("tokyo-night"), "Tokyo Night");
        assert_eq!(display_theme_name("gruvbox"), "Gruvbox");
        assert_eq!(display_theme_name("rose-pine"), "Rosé Pine");
        assert_eq!(display_theme_name("vscode"), "VS Code");
        assert_eq!(display_theme_name("docs-rs"), "docs.rs");
    }

    #[test]
    fn every_colorblind_theme_is_allowed() {
        for slug in COLORBLIND_THEMES {
            assert!(ALLOWED_THEMES.contains(slug), "{slug}");
        }
    }

    #[test]
    fn every_allowed_theme_has_its_label() {
        let labels: Vec<String> = ALLOWED_THEMES
            .iter()
            .map(|t| display_theme_name(t))
            .collect();
        assert_eq!(
            labels,
            [
                "Achromatopsia",
                "Ayu",
                "Catppuccin",
                "Deuteranopia",
                "docs.rs",
                "Dracula",
                "Ethereal",
                "Everforest",
                "GitHub",
                "Gruvbox",
                "Hackerman",
                "Kanagawa",
                "Matte Black",
                "Miasma",
                "Monokai",
                "Night Owl",
                "Nord",
                "One Dark",
                "Osaka Jade",
                "PowerShell",
                "Protanopia",
                "Ristretto",
                "Rosé Pine",
                "Rustbox",
                "Solarized",
                "Synthwave '84",
                "Tokyo Night",
                "Tritanopia",
                "Vantablack",
                "VS Code",
                "Zenburn",
            ]
        );
    }

    #[test]
    fn every_allowed_theme_has_a_non_empty_label() {
        for slug in ALLOWED_THEMES {
            assert!(!display_theme_name(slug).is_empty(), "{slug}");
        }
    }

    #[test]
    fn default_is_gruvbox_dark() {
        let prefs = UserPreferences::default();
        assert_eq!(prefs.theme, "gruvbox");
        assert!(prefs.dark_mode);
    }

    // == valid theme names ==

    #[test]
    fn accepts_all_allowed_themes() {
        let id = Uuid::new_v4();
        for theme in ALLOWED_THEMES {
            let result = UpdatePreferences::new(id, Some(theme), Some(true), None, None);
            assert!(result.is_ok(), "should accept theme: {theme}");
        }
    }

    #[test]
    fn rejects_unknown_theme() {
        let result =
            UpdatePreferences::new(Uuid::new_v4(), Some("not-a-theme"), Some(true), None, None);
        assert!(matches!(
            result,
            Err(InvalidUpdatePreferences::InvalidTheme)
        ));
    }

    #[test]
    fn rejects_empty_theme() {
        let result = UpdatePreferences::new(Uuid::new_v4(), Some(""), Some(true), None, None);
        assert!(matches!(
            result,
            Err(InvalidUpdatePreferences::InvalidTheme)
        ));
    }

    // == partial update (None) ==

    #[test]
    fn none_theme_passes_through() {
        let result = UpdatePreferences::new(Uuid::new_v4(), None, Some(false), None, None).unwrap();
        assert!(result.theme.is_none());
        assert_eq!(result.dark_mode, Some(false));
    }

    #[test]
    fn none_dark_mode_passes_through() {
        let result =
            UpdatePreferences::new(Uuid::new_v4(), Some("dracula"), None, None, None).unwrap();
        assert_eq!(result.theme.as_deref(), Some("dracula"));
        assert!(result.dark_mode.is_none());
    }

    #[test]
    fn both_none_is_valid() {
        let result = UpdatePreferences::new(Uuid::new_v4(), None, None, None, None).unwrap();
        assert!(result.theme.is_none());
        assert!(result.dark_mode.is_none());
    }

    // == universes beyond ==

    #[test]
    fn accepts_known_franchise_slugs() {
        let result = UpdatePreferences::new(
            Uuid::new_v4(),
            None,
            None,
            Some(true),
            Some(vec![
                "middle-earth".to_string(),
                "final-fantasy".to_string(),
            ]),
        )
        .unwrap();
        assert_eq!(result.exclude_universes_beyond, Some(true));
        assert_eq!(
            result
                .universes_beyond_exceptions
                .as_deref()
                .map(<[String]>::len),
            Some(2)
        );
    }

    #[test]
    fn rejects_unknown_franchise_slug() {
        let result = UpdatePreferences::new(
            Uuid::new_v4(),
            None,
            None,
            Some(true),
            Some(vec!["not-a-franchise".to_string()]),
        );
        assert!(matches!(
            result,
            Err(InvalidUpdatePreferences::InvalidFranchise)
        ));
    }

    #[test]
    fn old_payload_without_ub_fields_deserializes_with_defaults() {
        // JWT claims and wire payloads minted before the fields existed.
        let prefs: UserPreferences =
            serde_json::from_str(r#"{"theme":"gruvbox","dark_mode":true}"#).unwrap();
        assert!(!prefs.exclude_universes_beyond);
        assert!(prefs.universes_beyond_exceptions.is_empty());
    }
}

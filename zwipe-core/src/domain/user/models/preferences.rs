//! User display preferences (theme, dark mode, pip style).
//!
//! Preferences are stored per-user and embedded in JWT claims for instant
//! application on login. Users without a preferences row get defaults.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Allowed theme identifiers. Validated on update.
pub const ALLOWED_THEMES: &[&str] = &[
    "achromatopsia",
    "ayu",
    "bluloco",
    "catppuccin",
    "darcula",
    "deuteranopia",
    "docs-rs",
    "dracula",
    "eclipse",
    "ethereal",
    "everforest",
    "flexoki",
    "github",
    "github-high-contrast",
    "gruvbox",
    "hackerman",
    "kanagawa",
    "kanagawa-dragon",
    "material",
    "matte-black",
    "melange",
    "miasma",
    "modus",
    "monokai",
    "monokai-pro",
    "monokai-pro-machine",
    "monokai-pro-octagon",
    "monokai-pro-ristretto",
    "monokai-pro-spectrum",
    "neovim",
    "night-owl",
    "nightfox",
    "nord",
    "one-dark",
    "osaka-jade",
    "oxocarbon",
    "poimandres",
    "powershell",
    "protanopia",
    "rose-pine",
    "rustbox",
    "solarized",
    "synthwave-84",
    "terafox",
    "tokyo-night",
    "tritanopia",
    "vantablack",
    "vscode",
    "xcode",
    "zenburn",
];

/// Slugs missing from [`ALLOWED_THEMES`], each mapped to the theme its users
/// move to (a renamed theme's new slug, a removed theme's closest look), so an
/// outdated client keeps working until it updates. The server's migrations
/// move stored rows the same way.
pub const RENAMED_THEMES: &[(&str, &str)] = &[
    ("ristretto", "monokai-pro-ristretto"),
    ("visual-studio", "vscode"),
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
        "github-high-contrast" => return "GitHub High Contrast".to_string(),
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

/// How mana pips are colored, app-wide.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PipStyle {
    /// Pips in the current theme's own five mana colors.
    #[default]
    Theme,
    /// Pips in Magic's classic fills, the same on every theme.
    Natural,
}

impl PipStyle {
    /// The stored and wire name: `theme` or `natural`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Theme => "theme",
            Self::Natural => "natural",
        }
    }

    /// The other style, for a two-state toggle.
    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::Theme => Self::Natural,
            Self::Natural => Self::Theme,
        }
    }
}

impl fmt::Display for PipStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pip style name that is neither `theme` nor `natural`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown pip style")]
pub struct ParsePipStyleError;

impl FromStr for PipStyle {
    type Err = ParsePipStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "theme" => Ok(Self::Theme),
            "natural" => Ok(Self::Natural),
            _ => Err(ParsePipStyleError),
        }
    }
}

/// User display preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Theme identifier (e.g. "gruvbox", "dracula").
    pub theme: String,
    /// Whether dark mode is active.
    pub dark_mode: bool,
    /// Hide Universes Beyond cards (no in-universe printing) from card
    /// serving and commander select. Claims and payloads without the field
    /// read as `false`.
    #[serde(default)]
    pub exclude_universes_beyond: bool,
    /// Franchise slugs served anyway while the exclusion is on
    /// (`card::scryfall_data::universe::FRANCHISES`). Kept when the exclusion
    /// is toggled off so re-enabling restores the whitelist.
    #[serde(default)]
    pub universes_beyond_exceptions: Vec<String>,
    /// How mana pips are colored. Claims and payloads without the field read
    /// as [`PipStyle::Theme`].
    #[serde(default)]
    pub pip_style: PipStyle,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "gruvbox".to_string(),
            dark_mode: true,
            exclude_universes_beyond: false,
            universes_beyond_exceptions: Vec::new(),
            pip_style: PipStyle::Theme,
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
    /// New pip style, or `None` to leave unchanged.
    pub pip_style: Option<PipStyle>,
}

impl UpdatePreferences {
    /// Validates and constructs the request. A slug in [`RENAMED_THEMES`]
    /// becomes the theme it maps to.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidUpdatePreferences::InvalidTheme`] when `theme` is not
    /// in [`ALLOWED_THEMES`], and
    /// [`InvalidUpdatePreferences::InvalidFranchise`] when an exception slug
    /// names no known Universes Beyond franchise.
    pub fn new(
        user_id: Uuid,
        theme: Option<&str>,
        dark_mode: Option<bool>,
        exclude_universes_beyond: Option<bool>,
        universes_beyond_exceptions: Option<Vec<String>>,
        pip_style: Option<PipStyle>,
    ) -> Result<Self, InvalidUpdatePreferences> {
        let theme = theme.map(|t| {
            RENAMED_THEMES
                .iter()
                .find(|(from, _)| *from == t)
                .map_or(t, |(_, to)| to)
        });
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
            theme: theme.map(ToString::to_string),
            dark_mode,
            exclude_universes_beyond,
            universes_beyond_exceptions,
            pip_style,
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
                "Bluloco",
                "Catppuccin",
                "Darcula",
                "Deuteranopia",
                "docs.rs",
                "Dracula",
                "Eclipse",
                "Ethereal",
                "Everforest",
                "Flexoki",
                "GitHub",
                "GitHub High Contrast",
                "Gruvbox",
                "Hackerman",
                "Kanagawa",
                "Kanagawa Dragon",
                "Material",
                "Matte Black",
                "Melange",
                "Miasma",
                "Modus",
                "Monokai",
                "Monokai Pro",
                "Monokai Pro Machine",
                "Monokai Pro Octagon",
                "Monokai Pro Ristretto",
                "Monokai Pro Spectrum",
                "Neovim",
                "Night Owl",
                "Nightfox",
                "Nord",
                "One Dark",
                "Osaka Jade",
                "Oxocarbon",
                "Poimandres",
                "PowerShell",
                "Protanopia",
                "Rosé Pine",
                "Rustbox",
                "Solarized",
                "Synthwave '84",
                "Terafox",
                "Tokyo Night",
                "Tritanopia",
                "Vantablack",
                "VS Code",
                "Xcode",
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
            let result = UpdatePreferences::new(id, Some(theme), Some(true), None, None, None);
            assert!(result.is_ok(), "should accept theme: {theme}");
        }
    }

    #[test]
    fn an_old_slug_updates_to_its_current_theme() {
        let result =
            UpdatePreferences::new(Uuid::new_v4(), Some("ristretto"), None, None, None, None);
        assert_eq!(
            result.unwrap().theme.as_deref(),
            Some("monokai-pro-ristretto")
        );
    }

    #[test]
    fn a_removed_theme_updates_to_its_closest_one() {
        let result = UpdatePreferences::new(
            Uuid::new_v4(),
            Some("visual-studio"),
            None,
            None,
            None,
            None,
        );
        assert_eq!(result.unwrap().theme.as_deref(), Some("vscode"));
    }

    #[test]
    fn rejects_unknown_theme() {
        let result = UpdatePreferences::new(
            Uuid::new_v4(),
            Some("not-a-theme"),
            Some(true),
            None,
            None,
            None,
        );
        assert!(matches!(
            result,
            Err(InvalidUpdatePreferences::InvalidTheme)
        ));
    }

    #[test]
    fn rejects_empty_theme() {
        let result = UpdatePreferences::new(Uuid::new_v4(), Some(""), Some(true), None, None, None);
        assert!(matches!(
            result,
            Err(InvalidUpdatePreferences::InvalidTheme)
        ));
    }

    // == partial update (None) ==

    #[test]
    fn none_theme_passes_through() {
        let result =
            UpdatePreferences::new(Uuid::new_v4(), None, Some(false), None, None, None).unwrap();
        assert!(result.theme.is_none());
        assert_eq!(result.dark_mode, Some(false));
    }

    #[test]
    fn none_dark_mode_passes_through() {
        let result =
            UpdatePreferences::new(Uuid::new_v4(), Some("dracula"), None, None, None, None)
                .unwrap();
        assert_eq!(result.theme.as_deref(), Some("dracula"));
        assert!(result.dark_mode.is_none());
    }

    #[test]
    fn both_none_is_valid() {
        let result = UpdatePreferences::new(Uuid::new_v4(), None, None, None, None, None).unwrap();
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
            None,
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
            None,
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
        assert_eq!(prefs.universes_beyond_exceptions, Vec::<String>::new());
        assert_eq!(prefs.pip_style, PipStyle::Theme);
    }

    // == pip style ==

    #[test]
    fn pip_style_round_trips_through_its_name() {
        for style in [PipStyle::Theme, PipStyle::Natural] {
            assert_eq!(style.as_str().parse::<PipStyle>(), Ok(style));
            let json = serde_json::to_string(&style).unwrap();
            assert_eq!(json, format!("\"{style}\""));
            assert_eq!(serde_json::from_str::<PipStyle>(&json).unwrap(), style);
        }
        assert_eq!("classic".parse::<PipStyle>(), Err(ParsePipStyleError));
    }

    #[test]
    fn pip_style_toggles_between_the_two() {
        assert_eq!(PipStyle::Theme.toggled(), PipStyle::Natural);
        assert_eq!(PipStyle::Natural.toggled(), PipStyle::Theme);
    }

    #[test]
    fn pip_style_passes_through_the_update() {
        let result = UpdatePreferences::new(
            Uuid::new_v4(),
            None,
            None,
            None,
            None,
            Some(PipStyle::Natural),
        )
        .unwrap();
        assert_eq!(result.pip_style, Some(PipStyle::Natural));
    }
}

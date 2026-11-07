//! User display settings (theme, dark mode, pip style).
//!
//! Settings are stored per-user and embedded in JWT claims for instant
//! application on login. Users without a settings row get defaults.

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
    /// Pips in the current theme's own five mana colors. Stored as
    /// `themed`; the wire writes `theme`, the name 1.15.0 to 1.18.0 parse,
    /// and reads both.
    #[default]
    #[serde(rename = "theme", alias = "themed")]
    Themed,
    /// Pips in Magic's classic fills, the same on every theme. Stored as
    /// `classic`; the wire still writes `natural`, the name 1.15.0 to 1.17.0 parse,
    /// and reads both.
    #[serde(rename = "natural", alias = "classic")]
    Classic,
}

impl PipStyle {
    /// The stored name: `themed` or `classic`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Themed => "themed",
            Self::Classic => "classic",
        }
    }

    /// The other color, for a two-state toggle.
    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::Themed => Self::Classic,
            Self::Classic => Self::Themed,
        }
    }
}

impl fmt::Display for PipStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pip color name that is none of `themed`, `classic` or the earlier
/// `theme` and `natural`.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown pip color")]
pub struct ParsePipStyleError;

impl FromStr for PipStyle {
    type Err = ParsePipStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "themed" | "theme" => Ok(Self::Themed),
            "classic" | "natural" => Ok(Self::Classic),
            _ => Err(ParsePipStyleError),
        }
    }
}

/// What the swipe screens say after a swipe.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SwipeConfirmations {
    /// A quiet chip at the top left that holds the last action.
    #[default]
    Chip,
    /// A toast for each action.
    Toasts,
    /// Nothing.
    Off,
}

impl SwipeConfirmations {
    /// Every value, in the order the setting offers them.
    pub const ALL: [Self; 3] = [Self::Chip, Self::Toasts, Self::Off];

    /// The next value, for a button that cycles through them.
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Chip => Self::Toasts,
            Self::Toasts => Self::Off,
            Self::Off => Self::Chip,
        }
    }

    /// The stored and wire name: `chip`, `toasts` or `off`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chip => "chip",
            Self::Toasts => "toasts",
            Self::Off => "off",
        }
    }
}

impl fmt::Display for SwipeConfirmations {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A swipe confirmations name that is none of `chip`, `toasts` or `off`,
/// carrying the name refused.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown swipe confirmations {0:?}")]
pub struct ParseSwipeConfirmationsError(pub String);

impl FromStr for SwipeConfirmations {
    type Err = ParseSwipeConfirmationsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|value| value.as_str() == s)
            .ok_or_else(|| ParseSwipeConfirmationsError(s.to_owned()))
    }
}

/// User display settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSettings {
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
    /// as [`PipStyle::Themed`].
    #[serde(default)]
    pub pip_style: PipStyle,
    /// What the swipe screens say after a swipe. Claims and payloads without
    /// the field, every client before 1.18.1 and every row before the
    /// column, read as [`SwipeConfirmations::Chip`].
    #[serde(default)]
    pub swipe_confirmations: SwipeConfirmations,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme: "gruvbox".to_string(),
            dark_mode: true,
            exclude_universes_beyond: false,
            universes_beyond_exceptions: Vec::new(),
            pip_style: PipStyle::Themed,
            swipe_confirmations: SwipeConfirmations::Chip,
        }
    }
}

/// Validated request to update a user's settings.
///
/// Uses `Option<T>` for partial update semantics: `None` means unchanged.
#[derive(Debug)]
pub struct UpdateSettings {
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
    /// New pip colors, or `None` to leave unchanged.
    pub pip_style: Option<PipStyle>,
    /// New swipe confirmations, or `None` to leave unchanged.
    pub swipe_confirmations: Option<SwipeConfirmations>,
}

impl UpdateSettings {
    /// Starts an update for `user_id` that changes nothing. Set what
    /// changes, then [`build`](UpdateSettingsBuilder::build) it.
    #[must_use]
    pub fn builder(user_id: Uuid) -> UpdateSettingsBuilder {
        UpdateSettingsBuilder {
            user_id,
            theme: None,
            dark_mode: None,
            exclude_universes_beyond: None,
            universes_beyond_exceptions: None,
            pip_style: None,
            swipe_confirmations: None,
        }
    }
}

/// Builds an [`UpdateSettings`]: each setter takes the new value, or `None`
/// to leave that setting unchanged, and [`build`](Self::build) validates the
/// whole request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateSettingsBuilder {
    user_id: Uuid,
    theme: Option<String>,
    dark_mode: Option<bool>,
    exclude_universes_beyond: Option<bool>,
    universes_beyond_exceptions: Option<Vec<String>>,
    pip_style: Option<PipStyle>,
    swipe_confirmations: Option<SwipeConfirmations>,
}

impl UpdateSettingsBuilder {
    /// The theme's slug.
    #[must_use]
    pub fn theme(mut self, theme: Option<&str>) -> Self {
        self.theme = theme.map(ToString::to_string);
        self
    }

    /// Whether dark mode is on.
    #[must_use]
    pub fn dark_mode(mut self, dark_mode: Option<bool>) -> Self {
        self.dark_mode = dark_mode;
        self
    }

    /// Whether Universes Beyond cards are hidden.
    #[must_use]
    pub fn exclude_universes_beyond(mut self, exclude: Option<bool>) -> Self {
        self.exclude_universes_beyond = exclude;
        self
    }

    /// The franchise slugs served while Universes Beyond is hidden.
    #[must_use]
    pub fn universes_beyond_exceptions(mut self, slugs: Option<Vec<String>>) -> Self {
        self.universes_beyond_exceptions = slugs;
        self
    }

    /// How mana pips are colored.
    #[must_use]
    pub fn pip_style(mut self, pip_style: Option<PipStyle>) -> Self {
        self.pip_style = pip_style;
        self
    }

    /// What the swipe screens say after a swipe.
    #[must_use]
    pub fn swipe_confirmations(mut self, swipe_confirmations: Option<SwipeConfirmations>) -> Self {
        self.swipe_confirmations = swipe_confirmations;
        self
    }

    /// Validates the request. A slug in [`RENAMED_THEMES`] becomes the theme
    /// it maps to.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidUpdateSettings::InvalidTheme`] when the theme is not
    /// in [`ALLOWED_THEMES`], and
    /// [`InvalidUpdateSettings::InvalidFranchise`] when an exception slug
    /// names no known Universes Beyond franchise.
    pub fn build(self) -> Result<UpdateSettings, InvalidUpdateSettings> {
        let theme = self.theme.as_deref().map(|t| {
            RENAMED_THEMES
                .iter()
                .find(|(from, _)| *from == t)
                .map_or(t, |(_, to)| to)
        });
        if let Some(theme) = theme
            && !ALLOWED_THEMES.contains(&theme)
        {
            return Err(InvalidUpdateSettings::InvalidTheme);
        }
        if let Some(slugs) = &self.universes_beyond_exceptions
            && slugs.iter().any(|s| {
                crate::domain::card::scryfall_data::universe::franchise_by_slug(s).is_none()
            })
        {
            return Err(InvalidUpdateSettings::InvalidFranchise);
        }
        Ok(UpdateSettings {
            user_id: self.user_id,
            theme: theme.map(ToString::to_string),
            dark_mode: self.dark_mode,
            exclude_universes_beyond: self.exclude_universes_beyond,
            universes_beyond_exceptions: self.universes_beyond_exceptions,
            pip_style: self.pip_style,
            swipe_confirmations: self.swipe_confirmations,
        })
    }
}

/// Validation error for setting updates.
#[derive(Debug, Error)]
pub enum InvalidUpdateSettings {
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
        let settings = UserSettings::default();
        assert_eq!(settings.theme, "gruvbox");
        assert!(settings.dark_mode);
    }

    // == valid theme names ==

    #[test]
    fn accepts_all_allowed_themes() {
        let id = Uuid::new_v4();
        for theme in ALLOWED_THEMES {
            let result = UpdateSettings::builder(id)
                .theme(Some(theme))
                .dark_mode(Some(true))
                .build();
            assert!(result.is_ok(), "should accept theme: {theme}");
        }
    }

    #[test]
    fn an_old_slug_updates_to_its_current_theme() {
        let result = UpdateSettings::builder(Uuid::new_v4())
            .theme(Some("ristretto"))
            .build();
        assert_eq!(
            result.unwrap().theme.as_deref(),
            Some("monokai-pro-ristretto")
        );
    }

    #[test]
    fn a_removed_theme_updates_to_its_closest_one() {
        let result = UpdateSettings::builder(Uuid::new_v4())
            .theme(Some("visual-studio"))
            .build();
        assert_eq!(result.unwrap().theme.as_deref(), Some("vscode"));
    }

    #[test]
    fn rejects_unknown_theme() {
        let result = UpdateSettings::builder(Uuid::new_v4())
            .theme(Some("not-a-theme"))
            .dark_mode(Some(true))
            .build();
        assert!(matches!(result, Err(InvalidUpdateSettings::InvalidTheme)));
    }

    #[test]
    fn rejects_empty_theme() {
        let result = UpdateSettings::builder(Uuid::new_v4())
            .theme(Some(""))
            .dark_mode(Some(true))
            .build();
        assert!(matches!(result, Err(InvalidUpdateSettings::InvalidTheme)));
    }

    // == partial update (None) ==

    #[test]
    fn none_theme_passes_through() {
        let result = UpdateSettings::builder(Uuid::new_v4())
            .dark_mode(Some(false))
            .build()
            .unwrap();
        assert!(result.theme.is_none());
        assert_eq!(result.dark_mode, Some(false));
    }

    #[test]
    fn none_dark_mode_passes_through() {
        let result = UpdateSettings::builder(Uuid::new_v4())
            .theme(Some("dracula"))
            .build()
            .unwrap();
        assert_eq!(result.theme.as_deref(), Some("dracula"));
        assert!(result.dark_mode.is_none());
    }

    #[test]
    fn both_none_is_valid() {
        let result = UpdateSettings::builder(Uuid::new_v4()).build().unwrap();
        assert!(result.theme.is_none());
        assert!(result.dark_mode.is_none());
    }

    // == universes beyond ==

    #[test]
    fn accepts_known_franchise_slugs() {
        let result = UpdateSettings::builder(Uuid::new_v4())
            .exclude_universes_beyond(Some(true))
            .universes_beyond_exceptions(Some(vec![
                "middle-earth".to_string(),
                "final-fantasy".to_string(),
            ]))
            .build()
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
        let result = UpdateSettings::builder(Uuid::new_v4())
            .exclude_universes_beyond(Some(true))
            .universes_beyond_exceptions(Some(vec!["not-a-franchise".to_string()]))
            .build();
        assert!(matches!(
            result,
            Err(InvalidUpdateSettings::InvalidFranchise)
        ));
    }

    #[test]
    fn old_payload_without_ub_fields_deserializes_with_defaults() {
        // JWT claims and wire payloads minted before the fields existed.
        let settings: UserSettings =
            serde_json::from_str(r#"{"theme":"gruvbox","dark_mode":true}"#).unwrap();
        assert!(!settings.exclude_universes_beyond);
        assert_eq!(settings.universes_beyond_exceptions, Vec::<String>::new());
        assert_eq!(settings.pip_style, PipStyle::Themed);
    }

    // == pip color ==

    #[test]
    fn pip_style_round_trips_through_its_name() {
        for color in [PipStyle::Themed, PipStyle::Classic] {
            assert_eq!(color.as_str().parse::<PipStyle>(), Ok(color));
            let json = serde_json::to_string(&color).unwrap();
            assert_eq!(serde_json::from_str::<PipStyle>(&json).unwrap(), color);
        }
        assert_eq!("bogus".parse::<PipStyle>(), Err(ParsePipStyleError));
    }

    #[test]
    fn classic_stores_as_classic_and_reads_either_name() {
        assert_eq!(PipStyle::Classic.as_str(), "classic");
        for name in ["classic", "natural"] {
            assert_eq!(name.parse::<PipStyle>(), Ok(PipStyle::Classic));
            assert_eq!(
                serde_json::from_str::<PipStyle>(&format!("\"{name}\"")).unwrap(),
                PipStyle::Classic
            );
        }
    }

    #[test]
    fn themed_stores_as_themed_and_reads_either_name() {
        assert_eq!(PipStyle::Themed.as_str(), "themed");
        for name in ["themed", "theme"] {
            assert_eq!(name.parse::<PipStyle>(), Ok(PipStyle::Themed));
            assert_eq!(
                serde_json::from_str::<PipStyle>(&format!("\"{name}\"")).unwrap(),
                PipStyle::Themed
            );
        }
    }

    /// The pip enum as 1.15.0 to 1.17.0 carry it: `theme` and `natural`
    /// only. 1.17.1 to 1.18.0 read those two and `classic`.
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum ShippedPipStyle {
        Theme,
        Natural,
    }

    /// The settings as 1.15.0 to 1.18.0 read them.
    #[derive(Deserialize)]
    struct ShippedSettings {
        pip_style: ShippedPipStyle,
    }

    #[test]
    fn every_settings_value_parses_as_shipped_clients_read_it() {
        for (style, shipped) in [
            (PipStyle::Themed, ShippedPipStyle::Theme),
            (PipStyle::Classic, ShippedPipStyle::Natural),
        ] {
            let settings = UserSettings {
                pip_style: style,
                ..UserSettings::default()
            };
            let json = serde_json::to_string(&settings).unwrap();
            let read: ShippedSettings = serde_json::from_str(&json).unwrap();
            assert_eq!(read.pip_style, shipped, "{json}");
        }
    }

    #[test]
    fn the_wire_writes_the_name_shipped_clients_parse() {
        assert_eq!(
            serde_json::to_string(&PipStyle::Classic).unwrap(),
            "\"natural\""
        );
        assert_eq!(
            serde_json::to_string(&PipStyle::Themed).unwrap(),
            "\"theme\""
        );
    }

    // == swipe confirmations ==

    #[test]
    fn a_payload_without_swipe_confirmations_reads_as_chip() {
        let settings: UserSettings =
            serde_json::from_str(r#"{"theme":"gruvbox","dark_mode":true,"pip_style":"theme"}"#)
                .unwrap();
        assert_eq!(settings.swipe_confirmations, SwipeConfirmations::Chip);
    }

    #[test]
    fn swipe_confirmations_round_trip_through_their_names() {
        for value in SwipeConfirmations::ALL {
            assert_eq!(value.as_str().parse::<SwipeConfirmations>(), Ok(value));
            let json = serde_json::to_string(&value).unwrap();
            assert_eq!(json, format!("\"{}\"", value.as_str()));
            assert_eq!(
                serde_json::from_str::<SwipeConfirmations>(&json).unwrap(),
                value
            );
        }
        assert_eq!(
            "bogus".parse::<SwipeConfirmations>(),
            Err(ParseSwipeConfirmationsError("bogus".to_owned()))
        );
    }

    #[test]
    fn pip_style_toggles_between_the_two() {
        assert_eq!(PipStyle::Themed.toggled(), PipStyle::Classic);
        assert_eq!(PipStyle::Classic.toggled(), PipStyle::Themed);
    }

    #[test]
    fn pip_style_passes_through_the_update() {
        let result = UpdateSettings::builder(Uuid::new_v4())
            .pip_style(Some(PipStyle::Classic))
            .build()
            .unwrap();
        assert_eq!(result.pip_style, Some(PipStyle::Classic));
    }
}

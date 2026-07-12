//! Theme configuration for UI display.
//!
//! Maps a theme name + dark mode flag to the CSS class that should be applied
//! to the root element. Shared across zwiper and zite.

use serde::{Deserialize, Serialize};

use super::preferences::UserPreferences;

/// Display theme configuration used by the UI. Serializable so clients can cache
/// the last-used theme locally (see zwiper `theme_store`, zite `theme_store`) and
/// theme pre-auth screens before a session loads.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme identifier (e.g. "gruvbox", "dracula").
    pub name: String,
    /// Whether dark mode is active.
    pub is_dark: bool,
}

impl ThemeConfig {
    /// Returns the CSS class to apply to the screen root: `theme-{name}-{dark|light}`.
    pub fn css_class(&self) -> String {
        let mode = if self.is_dark { "dark" } else { "light" };
        format!("theme-{}-{}", self.name, mode)
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "gruvbox".to_string(),
            is_dark: true,
        }
    }
}

impl From<&UserPreferences> for ThemeConfig {
    fn from(prefs: &UserPreferences) -> Self {
        Self {
            name: prefs.theme.clone(),
            is_dark: prefs.dark_mode,
        }
    }
}

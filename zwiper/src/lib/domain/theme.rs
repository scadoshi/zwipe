//! Theme configuration for the application UI.

use zwipe::domain::user::models::preferences::{DARK_ONLY_THEMES, UserPreferences};

/// Display theme configuration used by the UI.
#[derive(Clone, PartialEq)]
pub struct ThemeConfig {
    /// Theme identifier (e.g. "gruvbox", "zwipe").
    pub name: String,
    /// Whether dark mode is active.
    pub is_dark: bool,
}

impl ThemeConfig {
    /// Returns the CSS class to apply to the screen root.
    ///
    /// Dark-only themes (zwipe) get `theme-{name}`.
    /// Others get `theme-{name}-{dark|light}`.
    /// The default zwipe theme returns an empty string since `:root` already
    /// defines the zwipe colors.
    pub fn css_class(&self) -> String {
        if self.name == "zwipe" {
            // Default theme — no override class needed, :root handles it
            String::new()
        } else if DARK_ONLY_THEMES.contains(&self.name.as_str()) {
            format!("theme-{}", self.name)
        } else {
            let mode = if self.is_dark { "dark" } else { "light" };
            format!("theme-{}-{}", self.name, mode)
        }
    }

    /// Whether this theme supports a light mode toggle.
    pub fn has_light_mode(&self) -> bool {
        !DARK_ONLY_THEMES.contains(&self.name.as_str())
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "zwipe".to_string(),
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

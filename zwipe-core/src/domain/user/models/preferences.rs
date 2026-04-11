//! User display preferences (theme, dark mode).
//!
//! Preferences are stored per-user and embedded in JWT claims for instant
//! application on login. Users without a preferences row get defaults.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Allowed theme identifiers. Validated on update.
pub const ALLOWED_THEMES: &[&str] = &[
    "catppuccin",
    "deuteranopia",
    "dracula",
    "everforest",
    "gruvbox",
    "monokai",
    "nord",
    "one-dark",
    "protanopia",
    "rose-pine",
    "rustbox",
    "solarized",
    "tokyo-night",
    "tritanopia",
    "zwipe",
];

/// User display preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserPreferences {
    /// Theme identifier (e.g. "gruvbox", "zwipe").
    pub theme: String,
    /// Whether dark mode is active.
    pub dark_mode: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "zwipe".to_string(),
            dark_mode: true,
        }
    }
}

/// Validated request to update a user's preferences.
///
/// Uses `Option<T>` for partial update semantics — `None` means unchanged.
#[derive(Debug)]
pub struct UpdatePreferences {
    /// User to update.
    pub user_id: Uuid,
    /// New theme identifier, or `None` to leave unchanged.
    pub theme: Option<String>,
    /// New dark mode setting, or `None` to leave unchanged.
    pub dark_mode: Option<bool>,
}

impl UpdatePreferences {
    /// Validates and constructs the request.
    pub fn new(
        user_id: Uuid,
        theme: Option<&str>,
        dark_mode: Option<bool>,
    ) -> Result<Self, InvalidUpdatePreferences> {
        if let Some(theme) = theme
            && !ALLOWED_THEMES.contains(&theme)
        {
            return Err(InvalidUpdatePreferences::InvalidTheme);
        }
        Ok(Self {
            user_id,
            theme: theme.map(|t| t.to_string()),
            dark_mode,
        })
    }
}

/// Validation error for preference updates.
#[derive(Debug, Error)]
pub enum InvalidUpdatePreferences {
    /// Theme is not in the allowed list.
    #[error("invalid theme")]
    InvalidTheme,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========
    //  default
    // =========

    #[test]
    fn default_is_zwipe_dark() {
        let prefs = UserPreferences::default();
        assert_eq!(prefs.theme, "zwipe");
        assert!(prefs.dark_mode);
    }

    // ====================
    //  valid theme names
    // ====================

    #[test]
    fn accepts_all_allowed_themes() {
        let id = Uuid::new_v4();
        for theme in ALLOWED_THEMES {
            let result = UpdatePreferences::new(id, Some(theme), Some(true));
            assert!(result.is_ok(), "should accept theme: {theme}");
        }
    }

    #[test]
    fn rejects_unknown_theme() {
        let result = UpdatePreferences::new(Uuid::new_v4(), Some("not-a-theme"), Some(true));
        assert!(matches!(
            result,
            Err(InvalidUpdatePreferences::InvalidTheme)
        ));
    }

    #[test]
    fn rejects_empty_theme() {
        let result = UpdatePreferences::new(Uuid::new_v4(), Some(""), Some(true));
        assert!(matches!(
            result,
            Err(InvalidUpdatePreferences::InvalidTheme)
        ));
    }

    // =======================
    //  partial update (None)
    // =======================

    #[test]
    fn none_theme_passes_through() {
        let result = UpdatePreferences::new(Uuid::new_v4(), None, Some(false)).unwrap();
        assert!(result.theme.is_none());
        assert_eq!(result.dark_mode, Some(false));
    }

    #[test]
    fn none_dark_mode_passes_through() {
        let result = UpdatePreferences::new(Uuid::new_v4(), Some("dracula"), None).unwrap();
        assert_eq!(result.theme.as_deref(), Some("dracula"));
        assert!(result.dark_mode.is_none());
    }

    #[test]
    fn both_none_is_valid() {
        let result = UpdatePreferences::new(Uuid::new_v4(), None, None).unwrap();
        assert!(result.theme.is_none());
        assert!(result.dark_mode.is_none());
    }
}

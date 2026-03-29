//! User display preferences (theme, dark mode).
//!
//! Preferences are stored per-user and embedded in JWT claims for instant
//! application on login. Users without a preferences row get defaults.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Allowed theme identifiers. Validated on update.
pub const ALLOWED_THEMES: &[&str] = &[
    "rustbox",
    "gruvbox",
    "dracula",
    "everforest",
    "catppuccin",
    "tokyo-night",
    "nord",
    "zwipe",
    "vantablack",
];

/// Themes that do not support light mode.
pub const DARK_ONLY_THEMES: &[&str] = &["zwipe", "vantablack"];

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
    ///
    /// Forces `dark_mode = true` for dark-only themes (zwipe, vantablack).
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
        // Force dark mode for dark-only themes
        let dark_mode = match theme {
            Some(t) if DARK_ONLY_THEMES.contains(&t) => Some(true),
            _ => dark_mode,
        };
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

/// Error from the update preferences operation.
#[derive(Debug, Error)]
pub enum UpdatePreferencesError {
    /// Validation failed.
    #[error(transparent)]
    Invalid(#[from] InvalidUpdatePreferences),
    /// Database error.
    #[error("database error")]
    Database(#[from] anyhow::Error),
}

/// Error from the get preferences operation.
#[derive(Debug, Error)]
pub enum GetPreferencesError {
    /// Database error.
    #[error("database error")]
    Database(#[from] anyhow::Error),
}

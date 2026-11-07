//! User account HTTP request contracts.

use serde::{Deserialize, Serialize};

use crate::domain::user::settings::{PipStyle, SwipeConfirmations};

/// HTTP request body for updating settings.
///
/// Uses `Option<T>` for partial update semantics: absent fields are unchanged.
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpUpdateSettings {
    /// Theme identifier, or absent to leave unchanged.
    pub theme: Option<String>,
    /// Dark mode setting, or absent to leave unchanged.
    pub dark_mode: Option<bool>,
    /// Universes Beyond exclusion, or absent to leave unchanged. serde
    /// defaults keep payloads from clients that predate the field.
    #[serde(default)]
    pub exclude_universes_beyond: Option<bool>,
    /// Exception franchise slugs (whole-list replace), or absent to leave
    /// unchanged.
    #[serde(default)]
    pub universes_beyond_exceptions: Option<Vec<String>>,
    /// Pip style, or absent to leave unchanged. serde defaults keep payloads
    /// from clients that predate the field.
    #[serde(default)]
    pub pip_style: Option<PipStyle>,
    /// Swipe confirmations, or absent to leave unchanged. serde defaults keep
    /// payloads from clients that predate the field.
    #[serde(default)]
    pub swipe_confirmations: Option<SwipeConfirmations>,
}

/// HTTP request body for marking a one-time UI hint as shown.
///
/// Responds with the updated user so the client can sync its session in
/// place.
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpMarkHintShown {
    /// Hint key (lowercase snake case, e.g. `"add_swipes"`).
    pub hint: String,
}

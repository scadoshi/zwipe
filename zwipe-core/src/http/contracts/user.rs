//! User account HTTP request contracts.

use serde::{Deserialize, Serialize};

/// HTTP request body for updating preferences.
///
/// Uses `Option<T>` for partial update semantics — absent fields are unchanged.
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpUpdatePreferences {
    /// Theme identifier, or absent to leave unchanged.
    pub theme: Option<String>,
    /// Dark mode setting, or absent to leave unchanged.
    pub dark_mode: Option<bool>,
}

//! User account HTTP request contracts.

use serde::{Deserialize, Serialize};

use crate::domain::user::preferences::PipColor;

/// HTTP request body for updating preferences.
///
/// Uses `Option<T>` for partial update semantics: absent fields are unchanged.
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpUpdatePreferences {
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
    /// Pip color, `theme` or `classic`, or absent to leave unchanged. serde
    /// defaults keep payloads from clients that predate the field.
    #[serde(default)]
    pub pip_style: Option<PipColor>,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn pip_style(body: &str) -> Result<Option<PipColor>, serde_json::Error> {
        serde_json::from_str::<HttpUpdatePreferences>(body).map(|request| request.pip_style)
    }

    #[test]
    fn the_request_takes_theme_or_classic() {
        assert_eq!(
            pip_style(r#"{"pip_style":"theme"}"#).unwrap(),
            Some(PipColor::Theme)
        );
        assert_eq!(
            pip_style(r#"{"pip_style":"classic"}"#).unwrap(),
            Some(PipColor::Classic)
        );
    }

    #[test]
    fn the_request_refuses_the_earlier_natural() {
        assert!(pip_style(r#"{"pip_style":"natural"}"#).is_err());
    }

    #[test]
    fn an_absent_or_null_pip_color_leaves_it_unchanged() {
        assert_eq!(pip_style("{}").unwrap(), None);
        assert_eq!(pip_style(r#"{"pip_style":null}"#).unwrap(), None);
    }
}

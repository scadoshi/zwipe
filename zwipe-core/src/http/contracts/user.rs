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
    /// from clients that predate the field. The earlier `theme` and `natural`
    /// are taken too: every build through 1.18.16 sends them.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn pip_style(body: &str) -> Result<Option<PipStyle>, serde_json::Error> {
        serde_json::from_str::<HttpUpdateSettings>(body).map(|request| request.pip_style)
    }

    #[test]
    fn the_request_takes_classic() {
        assert_eq!(
            pip_style(r#"{"pip_style":"classic"}"#).unwrap(),
            Some(PipStyle::Classic)
        );
    }

    #[test]
    fn the_request_takes_natural_as_shipped_clients_send_it() {
        assert_eq!(
            pip_style(r#"{"pip_style":"natural"}"#).unwrap(),
            Some(PipStyle::Classic)
        );
    }

    #[test]
    fn the_request_takes_themed() {
        assert_eq!(
            pip_style(r#"{"pip_style":"themed"}"#).unwrap(),
            Some(PipStyle::Themed)
        );
    }

    #[test]
    fn the_request_takes_theme_as_shipped_clients_send_it() {
        assert_eq!(
            pip_style(r#"{"pip_style":"theme"}"#).unwrap(),
            Some(PipStyle::Themed)
        );
    }

    #[test]
    fn the_request_refuses_an_unknown_pip_style() {
        assert!(pip_style(r#"{"pip_style":"bogus"}"#).is_err());
    }

    #[test]
    fn an_absent_or_null_pip_style_leaves_it_unchanged() {
        assert_eq!(pip_style("{}").unwrap(), None);
        assert_eq!(pip_style(r#"{"pip_style":null}"#).unwrap(), None);
    }
}

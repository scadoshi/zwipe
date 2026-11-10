//! User account HTTP request contracts.

use serde::{Deserialize, Deserializer, Serialize, de};

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
    /// from clients that predate the field. Only the current names are taken
    /// ([`PipStyle`]'s `FromStr`), so the earlier `natural` is refused.
    #[serde(default, deserialize_with = "current_pip_style")]
    pub pip_style: Option<PipStyle>,
    /// Swipe confirmations, or absent to leave unchanged. serde defaults keep
    /// payloads from clients that predate the field.
    #[serde(default)]
    pub swipe_confirmations: Option<SwipeConfirmations>,
}

/// A request's pip style under its current name only. The earlier names a
/// client's stored copy may hold read through serde, but a request naming one
/// is refused.
fn current_pip_style<'de, D>(deserializer: D) -> Result<Option<PipStyle>, D::Error>
where
    D: Deserializer<'de>,
{
    let name: Option<String> = Option::deserialize(deserializer)?;
    name.map(|name| name.parse().map_err(de::Error::custom))
        .transpose()
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
    fn the_request_refuses_the_earlier_natural() {
        assert!(pip_style(r#"{"pip_style":"natural"}"#).is_err());
    }

    #[test]
    fn an_absent_or_null_pip_style_leaves_it_unchanged() {
        assert_eq!(pip_style("{}").unwrap(), None);
        assert_eq!(pip_style(r#"{"pip_style":null}"#).unwrap(), None);
    }
}

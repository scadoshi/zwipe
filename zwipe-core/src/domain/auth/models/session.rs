//! Session entity shared between frontend and backend.
//!
//! A session represents a successful authentication with dual tokens:
//! - **Access Token**: Short-lived JWT (24h) for authenticating API requests
//! - **Refresh Token**: Long-lived token (14d) for obtaining new access tokens

use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::auth::models::access_token::AccessToken;
use crate::domain::auth::models::refresh_token::RefreshToken;
use crate::domain::user::models::User;
use crate::domain::user::models::settings::UserSettings;

/// Maximum number of concurrent sessions allowed per user.
///
/// When this limit is exceeded, the oldest session is automatically revoked.
pub const MAXIMUM_SESSION_COUNT: u8 = 10;

/// A successful authentication response containing user data and tokens.
///
/// Serializes with the settings under `settings`. Deserializes from that key
/// or from `preferences`, the name before 1.18.0, so a session an app before
/// 1.18.0 stored on the device still loads after the update; a build only
/// rewrites its stored copy on its next refresh. Reading both keys at once
/// also keeps a reply from a server before the rename's last phase loading.
/// The fallback can go once no session an app before 1.18.0 stored can still
/// be live: 30 days, a login's longest life, after the floor reached 1.18.0.
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
#[serde(try_from = "SessionWire")]
pub struct Session {
    /// The authenticated user's public profile information.
    pub user: User,

    /// JWT access token for authenticating API requests (24h expiry).
    pub access_token: AccessToken,

    /// Long-lived refresh token for obtaining new access tokens (14d expiry).
    pub refresh_token: RefreshToken,

    /// User's display settings (theme, dark mode).
    pub settings: UserSettings,
}

/// The session as it reads off the wire or out of storage: `settings`, or
/// `preferences` from before 1.18.0, or both. A serde alias would not do: it
/// refuses a document that carries both keys.
#[derive(Deserialize)]
struct SessionWire {
    user: User,
    access_token: AccessToken,
    refresh_token: RefreshToken,
    #[serde(default)]
    settings: Option<UserSettings>,
    #[serde(default)]
    preferences: Option<UserSettings>,
}

/// Neither `settings` nor `preferences` was present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("missing field `settings`")]
pub struct MissingSettings;

impl TryFrom<SessionWire> for Session {
    type Error = MissingSettings;

    fn try_from(wire: SessionWire) -> Result<Self, Self::Error> {
        let settings = wire.settings.or(wire.preferences).ok_or(MissingSettings)?;
        Ok(Self {
            user: wire.user,
            access_token: wire.access_token,
            refresh_token: wire.refresh_token,
            settings,
        })
    }
}

impl Session {
    /// Checks if the refresh token has expired.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.refresh_token.expires_at < Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::Duration;
    use uuid::Uuid;

    use super::*;
    use crate::domain::auth::models::access_token::Jwt;
    use crate::domain::user::models::email::Email;
    use crate::domain::user::models::username::Username;

    fn make_session(refresh_expires_at: chrono::DateTime<Utc>) -> Session {
        let user = User::new(
            Uuid::new_v4(),
            Username::new("alice").unwrap(),
            Email::from_str("alice@example.com").unwrap(),
        );
        let access_token = AccessToken {
            value: Jwt::from_str("header.payload.signature").unwrap(),
            expires_at: Utc::now().checked_add_signed(Duration::hours(24)).unwrap(),
        };
        let refresh_token = RefreshToken {
            value: "x".repeat(64),
            expires_at: refresh_expires_at,
        };
        Session {
            user,
            access_token,
            refresh_token,
            settings: UserSettings::default(),
        }
    }

    #[test]
    fn test_session_is_expired_when_refresh_token_has_past_expiry() {
        let past = Utc::now() - Duration::seconds(1);
        let session = make_session(past);
        assert!(session.is_expired());
    }

    #[test]
    fn test_session_is_not_expired_when_refresh_token_is_in_future() {
        let future = Utc::now() + Duration::days(14);
        let session = make_session(future);
        assert!(!session.is_expired());
    }

    #[test]
    fn serializes_settings_under_the_new_key_only() {
        let session = make_session(Utc::now() + Duration::days(14));
        let wire = serde_json::to_value(&session).unwrap();
        assert!(wire.get("settings").is_some(), "settings key: {wire}");
        assert!(wire.get("preferences").is_none(), "no legacy key: {wire}");
        let back: Session = serde_json::from_value(wire).unwrap();
        assert_eq!(back, session);
    }

    #[test]
    fn loads_a_session_stored_under_the_legacy_key() {
        let session = make_session(Utc::now() + Duration::days(14));
        let mut wire = serde_json::to_value(&session).unwrap();
        let settings = wire.as_object_mut().unwrap().remove("settings").unwrap();
        wire["preferences"] = settings;
        let back: Session = serde_json::from_value(wire).unwrap();
        assert_eq!(back, session);
    }

    #[test]
    fn loads_a_reply_carrying_both_keys() {
        let session = make_session(Utc::now() + Duration::days(14));
        let mut wire = serde_json::to_value(&session).unwrap();
        wire["preferences"] = wire["settings"].clone();
        let back: Session = serde_json::from_value(wire).unwrap();
        assert_eq!(back, session);
    }

    #[test]
    fn refuses_a_session_with_neither_key() {
        let session = make_session(Utc::now() + Duration::days(14));
        let mut wire = serde_json::to_value(&session).unwrap();
        wire.as_object_mut().unwrap().remove("settings");
        let err = serde_json::from_value::<Session>(wire).unwrap_err();
        assert!(err.to_string().contains("settings"), "{err}");
    }
}

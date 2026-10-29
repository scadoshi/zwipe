//! User entity and its value objects.

pub mod email;
pub mod hints;
pub mod preferences;
pub mod theme;
pub mod username;

use std::collections::HashMap;

use chrono::{DateTime, Utc};
pub use email::{Email, InvalidEmail};
pub use hints::{InvalidHintKey, MarkHintShown};
pub use preferences::{
    ALLOWED_THEMES, InvalidUpdatePreferences, ParsePipStyleError, PipStyle, UpdatePreferences,
    UserPreferences,
};
use serde::{Deserialize, Serialize};
pub use username::{InvalidUsername, Username};
use uuid::Uuid;

/// A public user profile entity.
///
/// This represents the publicly-visible information about a user. It deliberately
/// excludes sensitive data like password hashes, which are only handled in the
/// authentication layer.
///
/// # Security
///
/// This type is safe to return in API responses and does not contain:
/// - Password or password hash
/// - Session tokens
/// - Private settings
/// - Account creation date (could reveal user enumeration)
///
/// # Usage
///
/// This entity is returned from:
/// - User profile endpoints
/// - Authentication responses (as part of Session)
/// - Deck ownership information
///
/// # Example
///
/// ```rust,ignore
/// use zwipe_core::domain::user::User;
///
/// let user = User::new(id, username, email);
/// ```
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub struct User {
    /// Unique identifier for the user.
    pub id: Uuid,

    /// The user's validated username (3-20 chars, no profanity).
    pub username: Username,

    /// The user's validated email address.
    pub email: Email,

    /// When the user's email was verified. `None` means not yet verified.
    #[serde(default)]
    pub email_verified_at: Option<DateTime<Utc>>,

    /// One-time UI hints already shown to this user, keyed by hint id
    /// (see [`hints`]). Defaults to empty for responses from older servers.
    #[serde(default)]
    pub hints_shown: HashMap<String, bool>,

    /// An address the user asked to switch to, held until a link sent to it
    /// is clicked. Absent from the JSON when there is none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_email: Option<Email>,

    /// When the link for `pending_email` stops working. Absent from the JSON
    /// when there is no pending change, and from servers older than the field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_email_expires_at: Option<DateTime<Utc>>,
}

impl User {
    /// Creates a new user entity from validated components.
    ///
    /// # Arguments
    ///
    /// * `id`: unique user identifier
    /// * `username`: validated username
    /// * `email`: validated email address
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let user = User::new(id, username, email);
    /// ```
    pub fn new(id: Uuid, username: Username, email: Email) -> Self {
        Self {
            id,
            username,
            email,
            email_verified_at: None,
            hints_shown: HashMap::new(),
            pending_email: None,
            pending_email_expires_at: None,
        }
    }

    /// Whether a one-time UI hint has already been shown to this user.
    pub fn has_seen_hint(&self, key: &str) -> bool {
        self.hints_shown.get(key).copied().unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn user() -> User {
        User::new(
            Uuid::nil(),
            Username::new("tester").unwrap(),
            Email::new("tester@test.local").unwrap(),
        )
    }

    #[test]
    fn pending_expiry_round_trips_when_present() {
        let mut u = user();
        u.pending_email = Some(Email::new("new@test.local").unwrap());
        u.pending_email_expires_at = Some(Utc.with_ymd_and_hms(2026, 10, 24, 12, 0, 0).unwrap());
        let json = serde_json::to_value(&u).unwrap();
        assert_eq!(
            json.get("pending_email_expires_at"),
            Some(&serde_json::json!("2026-10-24T12:00:00Z"))
        );
        let back: User = serde_json::from_value(json).unwrap();
        assert_eq!(back, u);
    }

    #[test]
    fn pending_expiry_is_omitted_and_defaults_when_absent() {
        let u = user();
        let json = serde_json::to_value(&u).unwrap();
        assert!(json.get("pending_email_expires_at").is_none(), "{json}");

        // An older server sends the pending address without the expiry.
        let old = serde_json::json!({
            "id": Uuid::nil(),
            "username": "tester",
            "email": "tester@test.local",
            "pending_email": "new@test.local",
        });
        let back: User = serde_json::from_value(old).unwrap();
        assert!(back.pending_email.is_some());
        assert_eq!(back.pending_email_expires_at, None);
    }
}

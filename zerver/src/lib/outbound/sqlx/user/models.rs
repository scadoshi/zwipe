//! Database-to-domain user model conversions.

use chrono::{DateTime, Utc};
use sqlx_macros::FromRow;
use uuid::Uuid;
use zwipe_core::domain::Email;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::username::Username;

/// raw database user record
/// (unvalidated data from `PostgreSQL`)
#[expect(missing_docs, reason = "each field mirrors the column it is read from")]
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub hints_shown: serde_json::Value,
}

/// Converts a database user to the domain user. The username and email read
/// back as stored, without the rules for new input.
impl From<DatabaseUser> for User {
    fn from(value: DatabaseUser) -> Self {
        let username = Username::from_stored(value.username);
        let email = Email::from_stored(value.email);
        Self {
            id: value.id,
            username,
            email,
            email_verified_at: value.email_verified_at,
            // lenient: a malformed map degrades to "no hints shown"
            hints_shown: serde_json::from_value(value.hints_shown).unwrap_or_default(),
            pending_email: None,
            pending_email_expires_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_username_with_a_banned_word_still_loads() {
        let db = DatabaseUser {
            id: Uuid::nil(),
            username: "fuck".to_string(),
            email: "tester@test.local".to_string(),
            email_verified_at: None,
            hints_shown: serde_json::json!({}),
        };
        let user = User::from(db);
        assert_eq!(&*user.username, "fuck");
    }

    #[test]
    fn stored_email_that_fails_todays_rules_still_loads() {
        let db = DatabaseUser {
            id: Uuid::nil(),
            username: "tester".to_string(),
            email: "tester@localhost".to_string(),
            email_verified_at: None,
            hints_shown: serde_json::json!({}),
        };
        let user = User::from(db);
        assert_eq!(&*user.email, "tester@localhost");
    }
}

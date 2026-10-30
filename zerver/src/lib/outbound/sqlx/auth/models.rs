//! Database-to-domain auth model conversions.

use chrono::{DateTime, Utc};
use sqlx_macros::FromRow;
use uuid::Uuid;
use zwipe_core::domain::user::username::Username;

use crate::domain::auth::models::UserWithPasswordHash;
use crate::domain::auth::models::password::HashedPassword;
use crate::outbound::sqlx::auth::error::IntoUserWithPasswordHashError;

/// raw database user with password hash record
/// (unvalidated data from `PostgreSQL`)
#[expect(missing_docs, reason = "each field mirrors the column it is read from")]
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUserWithPasswordHash {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub hints_shown: serde_json::Value,
}

/// converts database user with password hash
/// to validated domain user with password hash
impl TryFrom<DatabaseUserWithPasswordHash> for UserWithPasswordHash {
    type Error = IntoUserWithPasswordHashError;

    fn try_from(value: DatabaseUserWithPasswordHash) -> Result<Self, Self::Error> {
        let username = Username::from_stored(value.username);
        let email = zwipe_core::domain::Email::from_stored(value.email);
        let password_hash = HashedPassword::new(&value.password_hash)?;
        Ok(Self {
            id: value.id,
            username,
            email,
            password_hash,
            email_verified_at: value.email_verified_at,
            // lenient: a malformed map degrades to "no hints shown"
            hints_shown: serde_json::from_value(value.hints_shown).unwrap_or_default(),
        })
    }
}

/// raw database refresh token record
#[expect(missing_docs, reason = "each field mirrors the column it is read from")]
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseRefreshToken {
    pub id: i32,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    /// Shared by every token rotated from the same login.
    pub family_id: Uuid,
    /// When the family's login happened; the absolute lifespan counts from here.
    pub login_at: DateTime<Utc>,
    /// Set once the token has been rotated; a later use of it is a replay.
    pub replaced_at: Option<DateTime<Utc>>,
    /// Client platform string (`ios`/`android`/`desktop`/`web`), NULL if unknown.
    pub platform: Option<String>,
    /// Client app version string (e.g. `"1.6.1"`), NULL if unknown.
    pub client_version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth::models::password::Password;

    #[test]
    fn stored_username_with_a_banned_word_still_loads() {
        let hash = HashedPassword::generate(Password::new("SecurePass123!").unwrap()).unwrap();
        let db = DatabaseUserWithPasswordHash {
            id: Uuid::nil(),
            username: "fuck".to_string(),
            email: "tester@test.local".to_string(),
            password_hash: hash.to_string(),
            email_verified_at: None,
            hints_shown: serde_json::json!({}),
        };
        let user = UserWithPasswordHash::try_from(db).unwrap();
        assert_eq!(&*user.username, "fuck");
    }

    #[test]
    fn stored_email_that_fails_todays_rules_still_loads() {
        let hash = HashedPassword::generate(Password::new("SecurePass123!").unwrap()).unwrap();
        let db = DatabaseUserWithPasswordHash {
            id: Uuid::nil(),
            username: "tester".to_string(),
            email: "tester@localhost".to_string(),
            password_hash: hash.to_string(),
            email_verified_at: None,
            hints_shown: serde_json::json!({}),
        };
        let user = UserWithPasswordHash::try_from(db).unwrap();
        assert_eq!(&*user.email, "tester@localhost");
    }
}

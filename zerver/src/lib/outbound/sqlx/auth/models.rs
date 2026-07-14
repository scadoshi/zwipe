use crate::{
    domain::auth::models::{UserWithPasswordHash, password::HashedPassword},
    outbound::sqlx::auth::error::IntoUserWithPasswordHashError,
};
use chrono::{DateTime, Utc};
use sqlx_macros::FromRow;
use uuid::Uuid;
use zwipe_core::domain::user::username::Username;

/// raw database user with password hash record
/// (unvalidated data from `PostgreSQL`)
#[allow(missing_docs)]
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUserWithPasswordHash {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub lockout_until: Option<DateTime<Utc>>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub hints_shown: serde_json::Value,
}

/// converts database user with password hash
/// to validated domain user with password hash
impl TryFrom<DatabaseUserWithPasswordHash> for UserWithPasswordHash {
    type Error = IntoUserWithPasswordHashError;

    fn try_from(value: DatabaseUserWithPasswordHash) -> Result<Self, Self::Error> {
        let username = Username::new(value.username)?;
        let email = zwipe_core::domain::Email::new(&value.email)?;
        let password_hash = HashedPassword::new(&value.password_hash)?;
        Ok(Self {
            id: value.id,
            username,
            email,
            password_hash,
            lockout_until: value.lockout_until,
            email_verified_at: value.email_verified_at,
            // lenient: a malformed map degrades to "no hints shown"
            hints_shown: serde_json::from_value(value.hints_shown).unwrap_or_default(),
        })
    }
}

/// raw database refresh token record
#[allow(missing_docs)]
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseRefreshToken {
    pub id: i32,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    /// Client platform string (`ios`/`android`/`desktop`/`web`), NULL if unknown.
    pub platform: Option<String>,
    /// Client app version string (e.g. `"1.6.1"`), NULL if unknown.
    pub client_version: Option<String>,
}

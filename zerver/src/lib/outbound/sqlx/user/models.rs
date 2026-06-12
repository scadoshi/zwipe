use chrono::{DateTime, Utc};
use sqlx_macros::FromRow;
use uuid::Uuid;

use crate::outbound::sqlx::user::error::IntoUserError;
use zwipe_core::domain::{user::{username::Username, User}, Email};

/// raw database user record
/// (unvalidated data from `PostgreSQL`)
#[allow(missing_docs)]
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub hints_shown: serde_json::Value,
}

/// converts database user to validated domain user
impl TryFrom<DatabaseUser> for User {
    type Error = IntoUserError;

    fn try_from(value: DatabaseUser) -> Result<Self, Self::Error> {
        let username = Username::new(value.username)?;
        let email = Email::new(&value.email)?;
        Ok(Self {
            id: value.id,
            username,
            email,
            email_verified_at: value.email_verified_at,
            // lenient: a malformed map degrades to "no hints shown"
            hints_shown: serde_json::from_value(value.hints_shown).unwrap_or_default(),
        })
    }
}

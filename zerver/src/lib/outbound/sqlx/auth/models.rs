use crate::{
    domain::auth::models::{UserWithPasswordHash, password::HashedPassword},
    outbound::sqlx::auth::error::IntoUserWithPasswordHashError,
};
use zwipe_core::domain::user::username::Username;
use chrono::NaiveDateTime;
use sqlx_macros::FromRow;
use uuid::Uuid;

/// raw database user with password hash record
/// (unvalidated data from `PostgreSQL`)
#[allow(missing_docs)]
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUserWithPasswordHash {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub lockout_until: Option<NaiveDateTime>,
    pub email_verified_at: Option<NaiveDateTime>,
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
        })
    }
}

/// raw database refresh token record
#[allow(missing_docs)]
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseRefreshToken {
    pub id: i32,
    pub user_id: Uuid,
    pub expires_at: NaiveDateTime,
    pub revoked: bool,
}

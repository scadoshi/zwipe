use email_address::EmailAddress;
use sqlx::query_as;
use sqlx_macros::FromRow;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::user::models::{GetUser, GetUserError, User, Username, UsernameError};
use crate::domain::user::ports::UserRepository;
use crate::outbound::sqlx::postgres::Postgres;

// ========
//  errors
// ========

#[derive(Debug, Error)]
pub enum ToUserError {
    #[error(transparent)]
    Id(uuid::Error),
    #[error(transparent)]
    Username(UsernameError),
    #[error(transparent)]
    Email(email_address::Error),
}

impl From<uuid::Error> for ToUserError {
    fn from(value: uuid::Error) -> Self {
        Self::Id(value)
    }
}

impl From<UsernameError> for ToUserError {
    fn from(value: UsernameError) -> Self {
        Self::Username(value)
    }
}

impl From<email_address::Error> for ToUserError {
    fn from(value: email_address::Error) -> Self {
        Self::Email(value)
    }
}

impl From<sqlx::Error> for GetUserError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<ToUserError> for GetUserError {
    fn from(value: ToUserError) -> Self {
        Self::UserFromDb(value.into())
    }
}

// ===========
//   db types
// ===========

/// raw database user record
/// (unvalidated data from `PostgreSQL`)
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUser {
    pub id: String,
    pub username: String,
    pub email: String,
}

/// converts database user to validated domain user
impl TryFrom<DatabaseUser> for User {
    type Error = ToUserError;

    fn try_from(value: DatabaseUser) -> Result<Self, Self::Error> {
        let id = Uuid::try_parse(&value.id)?;
        let username = Username::new(&value.username)?;
        let email =
            EmailAddress::parse_with_options(&value.email, email_address::Options::default())?;
        Ok(Self {
            id,
            username,
            email,
        })
    }
}

impl UserRepository for Postgres {
    // =====
    //  get
    // =====
    async fn get_user(&self, request: &GetUser) -> Result<User, GetUserError> {
        let database_user = query_as!(
            DatabaseUser,
            "SELECT id, username, email FROM users WHERE id = $1",
            request.id()
        )
        .fetch_one(&self.pool)
        .await?;

        let user: User = database_user.try_into()?;

        Ok(user)
    }
}

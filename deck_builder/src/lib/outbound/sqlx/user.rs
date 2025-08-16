use anyhow::{anyhow, Context};
use email_address::EmailAddress;
use sqlx::query_as;

use crate::domain::user::models::{User, UserCreationError, UserCreationRequest, UserId, UserName};
use crate::domain::user::ports::UserRepository;
use crate::outbound::sqlx::postgres::Postgres;

type TempError = Box<dyn std::error::Error>;

#[derive(Debug, Clone)]
pub struct DatabaseUser {
    id: i32,
    username: String,
    email: String,
}

impl TryFrom<DatabaseUser> for User {
    type Error = anyhow::Error;
    fn try_from(value: DatabaseUser) -> Result<Self, Self::Error> {
        let id = UserId::new(value.id).context("Failed to validate user ID")?;
        let username = UserName::new(&value.username).context("Failed to validate username")?;
        let email =
            EmailAddress::parse_with_options(&value.email, email_address::Options::default())
                .context("Failed to validate email")?;
        Ok(Self {
            id,
            username,
            email,
        })
    }
}

const UNIQUE_CONSTRAINT_VIOLATION_CODE: &str = "23505";

impl UserRepository for Postgres {
    async fn create_user(&self, req: &UserCreationRequest) -> Result<User, UserCreationError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| UserCreationError::DatabaseError(anyhow!("{}", e)))?;

        let database_user: DatabaseUser = query_as!(
            DatabaseUser,
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email",
            req.username.to_string(),
            req.email.to_string(),
            req.password_hash.to_string(),
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e)
                if e.code() == Some(UNIQUE_CONSTRAINT_VIOLATION_CODE.into()) =>
            {
                UserCreationError::Duplicate
            }
            e => UserCreationError::DatabaseError(anyhow!("{}", e)),
        })?;
        let user = User::try_from(database_user)
            .map_err(|e| UserCreationError::ReturnedUserInvalid(anyhow!("{e}")))?;
        Ok(user)
    }
}

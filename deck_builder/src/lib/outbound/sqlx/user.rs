use crate::domain::models::user::{User, UserCreationError, UserCreationRequest, UserId, UserName};
use crate::domain::ports::repositories::user::UserRepository;
use crate::outbound::sqlx::postgres::Postgres;
use chrono::{NaiveDate, NaiveDateTime};
use sqlx::{query_as, FromRow};
use tracing_subscriber::registry::Data;

#[derive(Debug, Clone, FromRow)]
pub struct SqlxUserInsert {
    username: String,
    email: String,
}

impl From<User> for SqlxUserInsert {
    fn from(value: User) -> Self {
        Self {
            username: value.username.to_string(),
            email: value.username.to_string(),
        }
    }
}

const UNIQUE_CONSTRAINT_VIOLATION_CODE: &str = "23505";

impl UserRepository for Postgres {
    async fn create_user(&self, req: &UserCreationRequest) -> Result<User, UserCreationError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| UserCreationError::DatabaseError(Box::new(e)))?;

        let user: User = query_as!(
            User,
            "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING *",
            req.username.to_string(),
            req.email.to_string()
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e)
                if e.code() == Some(UNIQUE_CONSTRAINT_VIOLATION_CODE.into()) =>
            {
                UserCreationError::Duplicate
            }
            e => UserCreationError::DatabaseError(Box::new(e)),
        })?;

        //
        //
        //
        //
        todo!()
    }
}

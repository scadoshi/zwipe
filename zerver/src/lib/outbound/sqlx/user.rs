use email_address::EmailAddress;
use sqlx::{query, query_as, QueryBuilder};
use sqlx_macros::FromRow;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::user::models::{
    CreateUser, CreateUserError, DeleteUser, DeleteUserError, GetUser, GetUserError, UpdateUser,
    UpdateUserError, User, UserName, UserNameError,
};
use crate::domain::user::ports::UserRepository;
use crate::outbound::sqlx::postgres::{IsConstraintViolation, Postgres};

// ========
//  errors
// ========

#[derive(Debug, Error)]
pub enum ToUserError {
    #[error(transparent)]
    Id(uuid::Error),
    #[error(transparent)]
    Username(UserNameError),
    #[error(transparent)]
    Email(email_address::Error),
}

impl From<uuid::Error> for ToUserError {
    fn from(value: uuid::Error) -> Self {
        Self::Id(value)
    }
}

impl From<UserNameError> for ToUserError {
    fn from(value: UserNameError) -> Self {
        Self::Username(value)
    }
}

impl From<email_address::Error> for ToUserError {
    fn from(value: email_address::Error) -> Self {
        Self::Email(value)
    }
}

impl From<sqlx::Error> for CreateUserError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            e if e.is_unique_constraint_violation() => Self::Duplicate,
            e => Self::Database(e.into()),
        }
    }
}

impl From<ToUserError> for CreateUserError {
    fn from(value: ToUserError) -> Self {
        Self::UserFromDb(value.into())
    }
}

impl From<sqlx::Error> for UpdateUserError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            e if e.is_unique_constraint_violation() => Self::Duplicate,
            e => Self::Database(e.into()),
        }
    }
}

impl From<ToUserError> for UpdateUserError {
    fn from(value: ToUserError) -> Self {
        Self::UserFromDb(value.into())
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

impl From<sqlx::Error> for DeleteUserError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
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
        let username = UserName::new(&value.username)?;
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
    // ========
    //  create
    // ========
    async fn create_user(&self, request: &CreateUser) -> Result<User, CreateUserError> {
        let mut tx = self.pool.begin().await?;

        let database_user = query_as!(
            DatabaseUser,
            "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id, username, email",
            request.username.to_string(),
            request.email.to_string(),
        )
        .fetch_one(&mut *tx)
        .await?;

        let user: User = database_user.try_into()?;

        tx.commit().await?;

        Ok(user)
    }
    // =====
    //  get
    // =====
    async fn get_user(&self, request: &GetUser) -> Result<User, GetUserError> {
        let database_user = query_as!(
            DatabaseUser,
            "SELECT id, username, email FROM users WHERE (id::text = $1 OR username = $1 OR email = $1)",
            request.as_str()
        )
        .fetch_one(&self.pool)
        .await?;

        let user: User = database_user.try_into()?;

        Ok(user)
    }
    // ========
    //  update
    // ========
    async fn update_user(&self, request: &UpdateUser) -> Result<User, UpdateUserError> {
        let mut tx = self.pool.begin().await?;

        let mut qb = QueryBuilder::new("UPDATE users SET ");
        let mut sep = qb.separated(", ");
        if let Some(username) = &request.username {
            sep.push("username = ")
                .push_bind_unseparated(username.to_string());
        }
        if let Some(email) = &request.email {
            sep.push("email = ")
                .push_bind_unseparated(email.to_string());
        }
        let now = chrono::Utc::now().naive_utc();
        sep.push("updated_at = ").push_bind_unseparated(now);

        qb.push(" WHERE id = ")
            .push_bind(request.id)
            .push(" RETURNING id, username, email");

        let database_user: DatabaseUser = qb.build_query_as().fetch_one(&mut *tx).await?;

        let user: User = database_user.try_into()?;

        tx.commit().await?;

        Ok(user)
    }
    // ========
    //  delete
    // ========
    async fn delete_user(&self, request: &DeleteUser) -> Result<(), DeleteUserError> {
        let mut tx = self.pool.begin().await?;

        let result = query!("DELETE FROM users WHERE id = $1", request.id())
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DeleteUserError::NotFound);
        }

        tx.commit().await?;

        Ok(())
    }
}

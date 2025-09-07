use std::str::FromStr;

use email_address::EmailAddress;
use sqlx::{query, query_as};
use sqlx_macros::FromRow;
use uuid::Uuid;
use thiserror::Error;

use crate::domain::auth::models::password::HashedPassword;
use crate::domain::auth::models::{
    AuthenticateUser, AuthenticateUserError, ChangeEmail, ChangeEmailError, ChangePassword, ChangePasswordError, ChangeUsername, ChangeUsernameError, DeleteUser, DeleteUserError, RegisterUser, RegisterUserError, UserWithPasswordHash
};
use crate::domain::auth::ports::AuthRepository;
use crate::domain::user::models::{User, Username, UsernameError};
use crate::outbound::sqlx::postgres::{IsConstraintViolation, Postgres};
use crate::outbound::sqlx::user::{DatabaseUser, ToUserError};

// ========
//  errors
// ========

#[derive(Debug, Error)]
pub enum ToUserWithPasswordHashError {
    #[error(transparent)]
    InvalidId(uuid::Error),
    #[error(transparent)]
    InvalidUsername(UsernameError),
    #[error(transparent)]
    InvalidEmail(email_address::Error),
    #[error(transparent)]
    InvalidPasswordHash(argon2::password_hash::Error),
}

impl From<argon2::password_hash::Error> for ToUserWithPasswordHashError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::InvalidPasswordHash(value)
    }
}

impl From<uuid::Error> for ToUserWithPasswordHashError {
    fn from(value: uuid::Error) -> Self {
        Self::InvalidId(value)
    }
}

impl From<UsernameError> for ToUserWithPasswordHashError {
    fn from(value: UsernameError) -> Self {
        Self::InvalidUsername(value)
    }
}

impl From<email_address::Error> for ToUserWithPasswordHashError {
    fn from(value: email_address::Error) -> Self {
        Self::InvalidEmail(value)
    }
}

impl From<ToUserError> for RegisterUserError {
    fn from(value: ToUserError) -> Self {
        Self::UserFromDb(value.into())
    }
}

impl From<ToUserWithPasswordHashError> for AuthenticateUserError {
    fn from(value: ToUserWithPasswordHashError) -> Self {
        Self::UserFromDb(value.into())
    }
}

impl From<ToUserError> for AuthenticateUserError {
    fn from(value: ToUserError) -> Self {
        Self::UserFromDb(value.into())
    }
}

impl From<sqlx::Error> for RegisterUserError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            e if e.is_unique_constraint_violation() => Self::Duplicate,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for AuthenticateUserError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::UserNotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for ChangeUsernameError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<ToUserError> for ChangeUsernameError {
    fn from(value: ToUserError) -> Self {
        Self::UserFromDb(value.into())
    }
}

impl From<sqlx::Error> for ChangeEmailError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<ToUserError> for ChangeEmailError {
    fn from(value: ToUserError) -> Self {
        Self::UserFromDb(value.into())
    }
}

impl From<sqlx::Error> for DeleteUserError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

// ==========
//  db types
// ==========

/// raw database user with password hash record 
/// (unvalidated data from `PostgreSQL`)
#[derive(Debug, Clone, FromRow)]
pub struct DatabaseUserWithPasswordHash {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

/// converts database user with password hash
/// to validated domain user with password hash
impl TryFrom<DatabaseUserWithPasswordHash> for UserWithPasswordHash {
    type Error = ToUserWithPasswordHashError;

    fn try_from(value: DatabaseUserWithPasswordHash) -> Result<Self, Self::Error> {
        let id = Uuid::try_parse(&value.id)?;
        let username = Username::new(&value.username)?;
        let email = EmailAddress::from_str(&value.email)?;

        let password_hash = HashedPassword::new(&value.password_hash)?;

        Ok(Self {
            id,
            username,
            email,
            password_hash,
        })
    }
}

impl AuthRepository for Postgres {
    // ========
    //  create
    // ========
    async fn create_user_with_password_hash(
        &self,
        request: &RegisterUser,
    ) -> Result<User, RegisterUserError> {
        let mut tx = self
            .pool
            .begin()
            .await?;

        let database_user = query_as!(
            DatabaseUser, 
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email", 
            request.username.to_string(), 
            request.email.to_string(), 
            request.password_hash.to_string()
        ).fetch_one(&mut *tx).await?;

        let user: User = database_user
            .try_into()?;

        tx.commit().await?;

        Ok(user)
    }
    // =====
    //  get
    // =====
    async fn get_user_with_password_hash(
        &self,
        request: &AuthenticateUser,
    ) -> Result<UserWithPasswordHash, AuthenticateUserError> {

        let database_user: DatabaseUserWithPasswordHash = query_as!(
            DatabaseUserWithPasswordHash,
            "SELECT id, username, email, password_hash FROM users WHERE (id::text = $1 OR username = $1 OR email = $1)",
            request.identifier
        )
        .fetch_one(&self.pool)
        .await?;

        let user: UserWithPasswordHash = database_user.try_into()?;

        Ok(user)
    }
    // ========
    //  update                           
    // ========
    async fn change_password(
        &self,
        request: &ChangePassword,
    ) -> Result<(), ChangePasswordError> {
        let mut tx = self
            .pool
            .begin()
            .await?;

        query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2", 
            request.password_hash.to_string(), 
            request.user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ChangePasswordError::UserNotFound,
            e => ChangePasswordError::Database(e.into()),
        })?;

        tx.commit()
            .await
            .map_err(|e| ChangePasswordError::Database(e.into()))?;

        Ok(())
    }

    async fn change_username(
            &self,
            request: &ChangeUsername,
        ) -> Result<User, ChangeUsernameError> {
        let mut tx = self.pool.begin().await?;

        let database_user = query_as!(
            DatabaseUser, 
            "UPDATE users SET username = $1 WHERE id = $2 RETURNING id, username, email", 
            request.username.to_string(), 
            request.user_id
        ).fetch_one(&mut *tx).await?;

        let user: User = database_user.try_into()?;

        tx.commit().await?;

        Ok(user)
    }

    async fn change_email(
            &self,
            request: &ChangeEmail,
        ) -> Result<User, ChangeEmailError> {
        let mut tx = self.pool.begin().await?;

        let database_user = query_as!(
            DatabaseUser, 
            "UPDATE users SET email = $1 WHERE id = $2 RETURNING id, username, email", 
            request.email.to_string(), 
            request.user_id
        ).fetch_one(&mut *tx).await?;

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



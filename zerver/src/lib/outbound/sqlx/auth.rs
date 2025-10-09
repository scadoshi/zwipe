use std::str::FromStr;
use chrono::{NaiveDateTime, Utc};
use email_address::EmailAddress;
use sqlx::{query, query_as, PgTransaction};
use sqlx_macros::FromRow;
use uuid::Uuid;
use thiserror::Error;
use crate::domain::auth::models::password::HashedPassword;
use crate::domain::auth::models::refresh_token::{RefreshToken, Sha256Hash};
use crate::domain::auth::models::session::{CreateSessionError, DeleteExpiredSessionsError, EnforceSessionMaximumError, RefreshSession, RefreshSessionError, RevokeSessionsError, MAXIMUM_SESSION_COUNT};
use crate::domain::auth::models::{
    AuthenticateUser, AuthenticateUserError, ChangeEmail, ChangeEmailError, ChangePassword, ChangePasswordError, ChangeUsername, ChangeUsernameError, DeleteUser, DeleteUserError, RegisterUser, RegisterUserError, UserWithPasswordHash
};
use crate::domain::auth::ports::AuthRepository;
use crate::domain::user::models::{InvalidUsername, User, Username};
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
    InvalidUsername(InvalidUsername),
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

impl From<InvalidUsername> for ToUserWithPasswordHashError {
    fn from(value: InvalidUsername) -> Self {
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

impl From<sqlx::Error> for RevokeSessionsError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<sqlx::Error> for CreateSessionError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<sqlx::Error> for EnforceSessionMaximumError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }   
}

impl From<sqlx::Error> for DeleteExpiredSessionsError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }   
}

impl From<sqlx::Error> for RefreshSessionError {
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

#[derive(Debug, Clone, FromRow)]
pub struct DatabaseRefreshToken {
    id: i32, 
    user_id: Uuid, 
    expires_at: NaiveDateTime, 
    revoked: bool,
}

// ==========
//   helpers
// ==========

/// takes existing transaction to create a new refresh token
/// commit will be handled outside of this
async fn create_refresh_token_with_tx(user_id: &Uuid, tx:  &mut PgTransaction<'_>) -> Result<RefreshToken, CreateSessionError> {
    let refresh_token = RefreshToken::generate();
    let _result = query_as!(
        DatabaseRefreshToken,
        "INSERT INTO refresh_tokens (user_id, value_hash, expires_at) VALUES ($1, $2, $3) RETURNING id, user_id, expires_at, revoked",
        user_id, 
        refresh_token.sha256_hash(), 
        refresh_token.expires_at
    ).fetch_one(&mut **tx).await?;
    enforce_refresh_token_max_with_tx(&user_id, tx).await?;
    Ok(refresh_token)
}

/// takes existing transaction
/// removes oldest refresh tokens until
/// total is not greater than max
async fn enforce_refresh_token_max_with_tx(
    user_id: &Uuid, 
    tx:  &mut PgTransaction<'_>
) -> Result<(), EnforceSessionMaximumError> {
query!(r#"DELETE FROM refresh_tokens WHERE id IN (
            SELECT id FROM (
                SELECT 
                    id, 
                    ROW_NUMBER() OVER(PARTITION BY user_id ORDER BY created_at DESC) token_number
                FROM refresh_tokens
                WHERE user_id = $1
            ) users_refresh_tokens
            WHERE token_number > $2
    )"#,
    user_id,
    MAXIMUM_SESSION_COUNT as i64
)
    .execute(&mut **tx)
    .await?;

Ok(())
}

// ======
//  main
// ======
impl AuthRepository for Postgres {
    // ========
    //  create
    // ========
    async fn create_user_and_refresh_token(
        &self,
        request: &RegisterUser,
    ) -> Result<(User, RefreshToken), RegisterUserError> {
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

        let refresh_token = create_refresh_token_with_tx(&user.id, &mut tx).await?;

        tx.commit().await?;

        Ok((user, refresh_token))
    }

    async fn create_refresh_token(
            &self,
            user_id: &Uuid,
        ) -> Result<RefreshToken, CreateSessionError> {

        let mut tx = self.pool.begin().await?;
        let refresh_token = create_refresh_token_with_tx(user_id, &mut tx).await?;
        tx.commit().await?;

    Ok(refresh_token)
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

    async fn use_refresh_token(
            &self,
            request: &RefreshSession,
        ) -> Result<RefreshToken, RefreshSessionError> {
        let mut tx = self.pool.begin().await?;

        let existing = query_as!(
            DatabaseRefreshToken, 
            "SELECT id, user_id, expires_at, revoked FROM refresh_tokens WHERE value_hash = $1",
            request.refresh_token.sha256_hash()
        ).fetch_one(&mut *tx).await.map_err(|e| {
            match e {
                sqlx::Error::RowNotFound => RefreshSessionError::NotFound(request.user_id),
                e => RefreshSessionError::Database(e.into()),
            }
        })?;

        if existing.user_id != request.user_id {
            return Err(RefreshSessionError::Forbidden(request.user_id));
        }

        if existing.expires_at < Utc::now().naive_local() {
            return Err(RefreshSessionError::Expired(request.user_id));
        }

        if existing.revoked {
            return Err(RefreshSessionError::Revoked(request.user_id));
        }

        query!("DELETE FROM refresh_tokens WHERE id = $1", existing.id).execute(&mut *tx).await?;

        let new = create_refresh_token_with_tx(&request.user_id, &mut tx).await?;

        tx.commit().await?;

        Ok(new)
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

    async fn delete_users_refresh_tokens(
            &self,
            user_id: &Uuid,
        ) -> Result<(), RevokeSessionsError> {
        let mut tx = self.pool.begin().await?;

        query!("DELETE FROM refresh_tokens WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn delete_expired_refresh_tokens(
            &self,
        ) -> Result<(), DeleteExpiredSessionsError> {
        let mut tx = self.pool.begin().await?;

        query!("DELETE FROM refresh_tokens WHERE expires_at < NOW()")
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}


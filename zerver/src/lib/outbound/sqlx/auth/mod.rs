pub mod error;
pub mod models;
pub mod helpers;

use chrono::{ Utc};
use sqlx::{query, query_as};
use uuid::Uuid;
use crate::domain::auth::models::refresh_token::{RefreshToken, Sha256Hash};
use crate::domain::auth::models::session::{create_session::CreateSessionError, delete_expired_sessions::DeleteExpiredSessionsError, refresh_session::{RefreshSession, RefreshSessionError}, revoke_sessions::RevokeSessionsError};
use crate::domain::auth::models::{
    authenticate_user::{AuthenticateUser, AuthenticateUserError}, change_email::{ChangeEmail, ChangeEmailError}, change_password::{ChangePassword, ChangePasswordError}, change_username::{ChangeUsername, ChangeUsernameError}, delete_user::{DeleteUser, DeleteUserError}, register_user::{RegisterUser, RegisterUserError}, UserWithPasswordHash
};
use crate::domain::auth::ports::AuthRepository;
use crate::domain::user::models::{User};
use crate::outbound::sqlx::auth::helpers::TxHelper;
use crate::outbound::sqlx::auth::models::{DatabaseRefreshToken, DatabaseUserWithPasswordHash};
use crate::outbound::sqlx::postgres::{ Postgres};
use crate::outbound::sqlx::user::{models::DatabaseUser};

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

        let user: User = database_user.try_into()?;
        let refresh_token = tx.create_refresh_token(user.id).await?;
        tx.commit().await?;

        Ok((user, refresh_token))
    }

    async fn create_refresh_token(
            &self,
            user_id: Uuid,
        ) -> Result<RefreshToken, CreateSessionError> {

        let mut tx = self.pool.begin().await?;
        let refresh_token = tx.create_refresh_token(user_id).await?;
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
            request.new_password_hash.to_string(), 
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
            request.new_username.to_string(), 
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

        let new = tx.create_refresh_token(request.user_id).await?;

        tx.commit().await?;

        Ok(new)
    }
    // ========
    //  delete
    // ========
    async fn delete_user(&self, request: &DeleteUser) -> Result<(), DeleteUserError> {
        let mut tx = self.pool.begin().await?;

        let result = query!("DELETE FROM users WHERE id = $1", request.user_id)
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
            user_id: Uuid,
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


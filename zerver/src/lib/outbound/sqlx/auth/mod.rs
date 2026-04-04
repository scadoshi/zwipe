//! Authentication and account management repository implementation.

/// SQLx error-to-domain error mappings and intermediate conversion errors.
pub mod error;
/// Transaction helpers for refresh token creation and session limits.
pub mod helpers;
/// Database-to-domain auth model conversions.
pub mod models;

use crate::domain::auth::models::password::HashedPassword;
use zwipe_core::domain::auth::models::refresh_token::{RefreshToken, Sha256Hash};
use crate::domain::auth::models::UserWithPasswordHash;
use crate::domain::auth::requests::{
    authenticate_user::{AuthenticateUser, AuthenticateUserError},
    change_email::{ChangeEmail, ChangeEmailError},
    change_password::{ChangePassword, ChangePasswordError},
    change_username::{ChangeUsername, ChangeUsernameError},
    create_session::CreateSessionError,
    delete_expired_sessions::DeleteExpiredSessionsError,
    delete_user::{DeleteUser, DeleteUserError},
    refresh_session::{RefreshSession, RefreshSessionError},
    register_user::{RegisterUser, RegisterUserError},
    reset_password::ResetPasswordError,
    revoke_sessions::RevokeSessionsError,
    verify_email::VerifyEmailError,
};
use crate::domain::auth::ports::AuthRepository;
use zwipe_core::domain::user::User;
use crate::outbound::sqlx::auth::helpers::TxHelper;
use crate::outbound::sqlx::auth::models::{DatabaseRefreshToken, DatabaseUserWithPasswordHash};
use crate::outbound::sqlx::postgres::Postgres;
use crate::outbound::sqlx::user::models::DatabaseUser;
use chrono::{NaiveDateTime, Utc};
use sqlx::{query, query_as, query_scalar};
use uuid::Uuid;

impl AuthRepository for Postgres {
    // ========
    //  create
    // ========
    async fn create_user_and_refresh_token(
        &self,
        request: &RegisterUser,
    ) -> Result<(User, RefreshToken), RegisterUserError> {
        let mut tx = self.pool.begin().await?;

        let database_user = query_as!(
            DatabaseUser,
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email, email_verified_at",
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
    /// Looks up a user by ID, username, **or** email using a single `OR` query.
    async fn get_user_with_password_hash(
        &self,
        request: &AuthenticateUser,
    ) -> Result<UserWithPasswordHash, AuthenticateUserError> {
        let database_user: DatabaseUserWithPasswordHash = query_as!(
            DatabaseUserWithPasswordHash,
            "SELECT id, username, email, password_hash, lockout_until, email_verified_at FROM users WHERE (id::text = $1 OR username = $1 OR email = $1)",
            request.identifier
        )
        .fetch_one(&self.pool)
        .await?;

        let user: UserWithPasswordHash = database_user.try_into()?;

        Ok(user)
    }
    // ========
    //  lockout
    // ========

    /// Atomically increments failed login counter with a sliding 30-minute window.
    /// Sets `lockout_until = NOW() + 30 min` after 5 failures within the window.
    async fn increment_failed_attempts(&self, user_id: Uuid) -> Result<(), AuthenticateUserError> {
        query!(
            r#"
            UPDATE users
            SET
                failed_login_attempts = CASE
                    WHEN COALESCE(last_failed_at, '1970-01-01'::TIMESTAMP) < NOW() - INTERVAL '30 minutes'
                    THEN 1
                    ELSE failed_login_attempts + 1
                END,
                last_failed_at = NOW(),
                lockout_until = CASE
                    WHEN (
                        CASE
                            WHEN COALESCE(last_failed_at, '1970-01-01'::TIMESTAMP) < NOW() - INTERVAL '30 minutes'
                            THEN 1
                            ELSE failed_login_attempts + 1
                        END
                    ) >= 5
                    THEN NOW() + INTERVAL '30 minutes'
                    ELSE lockout_until
                END
            WHERE id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthenticateUserError::Database(e.into()))?;

        Ok(())
    }

    /// Clears failed login counter and lockout on successful authentication.
    async fn reset_failed_attempts(&self, user_id: Uuid) -> Result<(), AuthenticateUserError> {
        query!(
            "UPDATE users SET failed_login_attempts = 0, last_failed_at = NULL, lockout_until = NULL WHERE id = $1",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthenticateUserError::Database(e.into()))?;

        Ok(())
    }

    // ========
    //  update
    // ========
    async fn change_password_and_revoke_sessions(&self, request: &ChangePassword) -> Result<(), ChangePasswordError> {
        let mut tx = self.pool.begin().await?;

        let result = query!(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            request.new_password_hash.to_string(),
            request.user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| ChangePasswordError::Database(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(ChangePasswordError::UserNotFound);
        }

        query!(
            "DELETE FROM refresh_tokens WHERE user_id = $1",
            request.user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| ChangePasswordError::Database(e.into()))?;

        tx.commit()
            .await
            .map_err(|e| ChangePasswordError::Database(e.into()))?;

        Ok(())
    }

    async fn change_username(&self, request: &ChangeUsername) -> Result<User, ChangeUsernameError> {
        let mut tx = self.pool.begin().await?;

        let database_user = query_as!(
            DatabaseUser,
            "UPDATE users SET username = $1 WHERE id = $2 RETURNING id, username, email, email_verified_at",
            request.new_username.to_string(),
            request.user_id
        ).fetch_one(&mut *tx).await?;

        let user: User = database_user.try_into()?;

        tx.commit().await?;

        Ok(user)
    }

    async fn change_email(&self, request: &ChangeEmail) -> Result<User, ChangeEmailError> {
        let mut tx = self.pool.begin().await?;

        let database_user = query_as!(
            DatabaseUser,
            "UPDATE users SET email = $1, email_verified_at = NULL WHERE id = $2 RETURNING id, username, email, email_verified_at",
            request.email.to_string(),
            request.user_id
        ).fetch_one(&mut *tx).await?;

        let user: User = database_user.try_into()?;

        tx.commit().await?;

        Ok(user)
    }

    /// Rotates a refresh token: validates the existing token (ownership, expiry,
    /// revocation), deletes it, and issues a new one — all within a single transaction.
    async fn use_refresh_token(
        &self,
        request: &RefreshSession,
    ) -> Result<RefreshToken, RefreshSessionError> {
        let mut tx = self.pool.begin().await?;

        let existing = query_as!(
            DatabaseRefreshToken,
            "SELECT id, user_id, expires_at, revoked FROM refresh_tokens WHERE value_hash = $1",
            request.refresh_token.sha256_hash()
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RefreshSessionError::NotFound(request.user_id),
            e => RefreshSessionError::Database(e.into()),
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

        query!("DELETE FROM refresh_tokens WHERE id = $1", existing.id)
            .execute(&mut *tx)
            .await?;

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

    async fn delete_users_refresh_tokens(&self, user_id: Uuid) -> Result<(), RevokeSessionsError> {
        let mut tx = self.pool.begin().await?;

        query!("DELETE FROM refresh_tokens WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn delete_expired_refresh_tokens(&self) -> Result<(), DeleteExpiredSessionsError> {
        let mut tx = self.pool.begin().await?;

        let result = query!("DELETE FROM refresh_tokens WHERE expires_at < NOW()")
            .execute(&mut *tx)
            .await?;
        tracing::info!(
            event = "token_cleanup",
            rows_deleted = result.rows_affected()
        );

        tx.commit().await?;

        Ok(())
    }

    // ========================
    //  email verification
    // ========================

    async fn store_email_verification_token(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: NaiveDateTime,
    ) -> Result<(), RegisterUserError> {
        query!(
            "INSERT INTO email_verification_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
            user_id,
            token_hash,
            expires_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RegisterUserError::Database(e.into()))?;
        Ok(())
    }

    async fn use_email_verification_token(
        &self,
        token_hash: &str,
    ) -> Result<Uuid, VerifyEmailError> {
        let user_id: Option<Uuid> = query_scalar!(
            "DELETE FROM email_verification_tokens WHERE token_hash = $1 AND expires_at > NOW() RETURNING user_id",
            token_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| VerifyEmailError::Database(e.into()))?;

        user_id.ok_or(VerifyEmailError::InvalidToken)
    }

    async fn mark_email_verified(&self, user_id: Uuid) -> Result<(), VerifyEmailError> {
        query!(
            "UPDATE users SET email_verified_at = NOW() WHERE id = $1",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| VerifyEmailError::Database(e.into()))?;
        Ok(())
    }

    async fn delete_email_verification_tokens(&self, user_id: Uuid) -> Result<(), anyhow::Error> {
        query!(
            "DELETE FROM email_verification_tokens WHERE user_id = $1",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(())
    }

    // ========================
    //  password reset
    // ========================

    async fn get_user_id_by_email(&self, email: &str) -> Result<Option<Uuid>, anyhow::Error> {
        let user_id: Option<Uuid> = query_scalar!("SELECT id FROM users WHERE email = $1", email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(user_id)
    }

    async fn is_password_reset_on_cooldown(&self, user_id: Uuid) -> Result<bool, anyhow::Error> {
        let on_cooldown: bool = query_scalar!(
            r#"SELECT EXISTS(
                SELECT 1 FROM password_reset_tokens
                WHERE user_id = $1 AND created_at > NOW() - INTERVAL '5 minutes'
            ) as "exists!""#,
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(on_cooldown)
    }

    async fn delete_password_reset_tokens(&self, user_id: Uuid) -> Result<(), anyhow::Error> {
        query!(
            "DELETE FROM password_reset_tokens WHERE user_id = $1",
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(())
    }

    async fn store_password_reset_token(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: NaiveDateTime,
    ) -> Result<(), anyhow::Error> {
        query!(
            "INSERT INTO password_reset_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
            user_id,
            token_hash,
            expires_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(())
    }

    async fn use_password_reset_token(&self, token_hash: &str) -> Result<Uuid, ResetPasswordError> {
        let user_id: Option<Uuid> = query_scalar!(
            "DELETE FROM password_reset_tokens WHERE token_hash = $1 AND expires_at > NOW() RETURNING user_id",
            token_hash
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ResetPasswordError::Database(e.into()))?;

        user_id.ok_or(ResetPasswordError::InvalidToken)
    }

    async fn reset_password_and_revoke_sessions(
        &self,
        user_id: Uuid,
        new_hash: HashedPassword,
    ) -> Result<(), ResetPasswordError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ResetPasswordError::Database(e.into()))?;

        query!(
            "UPDATE users SET password_hash = $1, failed_login_attempts = 0, last_failed_at = NULL, lockout_until = NULL WHERE id = $2",
            new_hash.to_string(),
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| ResetPasswordError::Database(e.into()))?;

        query!("DELETE FROM refresh_tokens WHERE user_id = $1", user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ResetPasswordError::Database(e.into()))?;

        tx.commit()
            .await
            .map_err(|e| ResetPasswordError::Database(e.into()))?;

        Ok(())
    }
}

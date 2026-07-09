//! Transaction helper traits for authentication operations.
//!
//! Provides extension methods on PostgreSQL transactions to manage refresh tokens
//! within existing transaction contexts. Commits are handled by the caller.

use std::future::Future;

use crate::{
    domain::auth::requests::{
        create_session::CreateSessionError, enforce_session_maximum::EnforceSessionMaximumError,
    },
    outbound::sqlx::auth::models::DatabaseRefreshToken,
};
use sqlx::{PgTransaction, query, query_as};
use uuid::Uuid;
use zwipe_core::domain::auth::models::{
    platform::ClientPlatform,
    refresh_token::{RefreshToken, Sha256Hash},
    session::MAXIMUM_SESSION_COUNT,
};

/// Extension trait for refresh token operations within a PostgreSQL transaction.
///
/// These helpers allow composing multiple auth operations in a single transaction
/// without committing prematurely. The caller is responsible for committing.
pub trait TxHelper {
    /// Creates a new refresh token for the specified user.
    ///
    /// Generates a cryptographically secure token, stores its hash in the database,
    /// and enforces the maximum session count by removing oldest tokens if needed.
    fn create_refresh_token(
        &mut self,
        user_id: Uuid,
        platform: Option<ClientPlatform>,
    ) -> impl Future<Output = Result<RefreshToken, CreateSessionError>> + Send;

    /// Enforces the maximum number of concurrent sessions per user.
    ///
    /// Deletes the oldest refresh tokens when the count exceeds [`MAXIMUM_SESSION_COUNT`],
    /// keeping only the most recently created tokens.
    fn enforce_refresh_token_max(
        &mut self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), EnforceSessionMaximumError>> + Send;
}

impl<'a> TxHelper for PgTransaction<'a> {
    async fn create_refresh_token(
        &mut self,
        user_id: Uuid,
        platform: Option<ClientPlatform>,
    ) -> Result<RefreshToken, CreateSessionError> {
        let refresh_token = RefreshToken::generate();
        let platform = platform.map(|p| p.to_string());
        let _result = query_as!(
            DatabaseRefreshToken,
            "INSERT INTO refresh_tokens (user_id, value_hash, expires_at, platform) VALUES ($1, $2, $3, $4) RETURNING id, user_id, expires_at, revoked, platform",
            user_id,
            refresh_token.sha256_hash(),
            refresh_token.expires_at,
            platform
        )
        .fetch_one(&mut **self)
        .await?;
        self.enforce_refresh_token_max(user_id).await?;
        Ok(refresh_token)
    }

    async fn enforce_refresh_token_max(
        &mut self,
        user_id: Uuid,
    ) -> Result<(), EnforceSessionMaximumError> {
        query!(
            r#"DELETE FROM refresh_tokens WHERE id IN (
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
        .execute(&mut **self)
        .await?;

        Ok(())
    }
}

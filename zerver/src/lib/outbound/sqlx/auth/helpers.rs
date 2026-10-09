//! Transaction helper traits for authentication operations.
//!
//! Provides extension methods on PostgreSQL transactions to manage refresh tokens
//! within existing transaction contexts. Commits are handled by the caller.

use std::future::Future;

use crate::domain::auth::requests::{
    create_session::CreateSessionError, enforce_session_maximum::EnforceSessionMaximumError,
};
use chrono::{DateTime, Utc};
use sqlx::{PgTransaction, query};
use uuid::Uuid;
use zwipe_core::domain::auth::models::{
    platform::ClientPlatform,
    refresh_token::{RefreshToken, Sha256Hash},
    session::MAXIMUM_SESSION_COUNT,
};

/// The chain of refresh tokens that one login produces.
///
/// Every rotation carries the same `id` and `login_at`, so a replayed token
/// can take down every sibling and the absolute lifespan counts from the login.
#[derive(Debug, Clone, Copy)]
pub struct RefreshFamily {
    /// Shared by every token in the chain.
    pub id: Uuid,
    /// When the login that started the chain happened.
    pub login_at: DateTime<Utc>,
}

impl RefreshFamily {
    /// Starts a new family at a login or registration.
    pub fn start() -> Self {
        Self {
            id: Uuid::new_v4(),
            login_at: Utc::now(),
        }
    }
}

/// Extension trait for refresh token operations within a PostgreSQL transaction.
///
/// These helpers allow composing multiple auth operations in a single transaction
/// without committing prematurely. The caller is responsible for committing.
pub trait TxHelper {
    /// Creates a new refresh token for the specified user in `family`.
    ///
    /// Generates a cryptographically secure token, stores its hash in the database,
    /// and prunes the user's tokens (expired + beyond the session cap).
    fn create_refresh_token(
        &mut self,
        user_id: Uuid,
        family: RefreshFamily,
        platform: Option<ClientPlatform>,
        client_version: Option<String>,
    ) -> impl Future<Output = Result<RefreshToken, CreateSessionError>> + Send;

    /// Prunes the user's refresh tokens: deletes expired ones and, of the live
    /// rest (not yet rotated), keeps only the [`MAXIMUM_SESSION_COUNT`] most
    /// recently created. Replaced rows do not count against the cap.
    ///
    /// Runs at every insert, so the table stays bounded without a global sweeper.
    fn prune_users_refresh_tokens(
        &mut self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), EnforceSessionMaximumError>> + Send;
}

impl<'a> TxHelper for PgTransaction<'a> {
    async fn create_refresh_token(
        &mut self,
        user_id: Uuid,
        family: RefreshFamily,
        platform: Option<ClientPlatform>,
        client_version: Option<String>,
    ) -> Result<RefreshToken, CreateSessionError> {
        let refresh_token = RefreshToken::generate();
        let platform = platform.map(|p| p.to_string());
        query!(
            "INSERT INTO refresh_tokens (user_id, value_hash, expires_at, family_id, login_at, platform, client_version) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            user_id,
            refresh_token.sha256_hash(),
            refresh_token.expires_at,
            family.id,
            family.login_at,
            platform,
            client_version
        )
        .execute(&mut **self)
        .await?;
        self.prune_users_refresh_tokens(user_id).await?;
        Ok(refresh_token)
    }

    async fn prune_users_refresh_tokens(
        &mut self,
        user_id: Uuid,
    ) -> Result<(), EnforceSessionMaximumError> {
        query!(
            r#"DELETE FROM refresh_tokens WHERE id IN (
                        SELECT id FROM (
                            SELECT
                                id,
                                ROW_NUMBER() OVER(ORDER BY created_at DESC) token_number
                            FROM refresh_tokens
                            WHERE user_id = $1 AND replaced_at IS NULL
                        ) live_refresh_tokens
                        WHERE token_number > $2
                ) OR (user_id = $1 AND expires_at < NOW())"#,
            user_id,
            MAXIMUM_SESSION_COUNT as i64
        )
        .execute(&mut **self)
        .await?;

        Ok(())
    }
}

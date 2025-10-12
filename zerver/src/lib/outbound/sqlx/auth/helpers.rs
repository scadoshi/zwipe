use std::future::Future;

use sqlx::{query_as, PgTransaction, query};
use uuid::Uuid;
use crate::{domain::auth::models::{refresh_token::{RefreshToken, Sha256Hash}, session::{CreateSessionError, EnforceSessionMaximumError, MAXIMUM_SESSION_COUNT}}, outbound::sqlx::auth::models::DatabaseRefreshToken};

// **commits are handled outside of these helpers**

pub trait TxHelper {
    fn create_refresh_token(&mut self, user_id: &Uuid) -> 
        impl Future<Output = Result<RefreshToken, CreateSessionError>> + Send;

    fn enforce_refresh_token_max(&mut self, user_id: &Uuid) -> 
        impl Future<Output = Result<(), EnforceSessionMaximumError>> + Send;
}

/// creates new refresh token for user
impl<'a> TxHelper for PgTransaction<'a> {
    async fn create_refresh_token(&mut self, user_id: &Uuid) -> 
            Result<RefreshToken, CreateSessionError> {
                let refresh_token = RefreshToken::generate();
                let _result = query_as!(
                    DatabaseRefreshToken,
                    "INSERT INTO refresh_tokens (user_id, value_hash, expires_at) VALUES ($1, $2, $3) RETURNING id, user_id, expires_at, revoked",
                    user_id, 
                    refresh_token.sha256_hash(), 
                    refresh_token.expires_at
                ).fetch_one(&mut **self).await?;
                self.enforce_refresh_token_max(&user_id).await?;
                Ok(refresh_token)
            }

    /// removes oldest refresh tokens until total is not greater than max
    async fn enforce_refresh_token_max(&mut self, user_id: &Uuid) -> 
            Result<(), EnforceSessionMaximumError> {
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
                .execute(&mut **self)
                .await?;

            Ok(())
    }
}
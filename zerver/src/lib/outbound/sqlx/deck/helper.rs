//! Query-based deck ownership verification.

use std::future::Future;

use sqlx::{query_scalar, PgPool};
use uuid::Uuid;

/// Checks deck ownership by querying the `decks` table.
///
/// Implemented on `Uuid` so that a user ID can directly verify ownership:
/// `user_id.owns_deck(deck_id, &pool)`. Every mutating deck operation calls
/// this before proceeding.
pub trait OwnsDeck {
    /// Returns `true` if `self` (user ID) owns the given deck.
    fn owns_deck(
        &self,
        deck_id: Uuid,
        pool: &PgPool,
    ) -> impl Future<Output = Result<bool, sqlx::Error>> + Send;
}

impl OwnsDeck for Uuid {
    async fn owns_deck(&self, deck_id: Uuid, pool: &PgPool) -> Result<bool, sqlx::Error> {
        let deck_user_id = query_scalar!("SELECT user_id FROM decks WHERE id = $1", deck_id)
            .fetch_one(pool)
            .await?;
        Ok(deck_user_id == *self)
    }
}

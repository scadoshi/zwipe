use std::future::Future;

use sqlx::{query_scalar, PgPool};
use uuid::Uuid;

pub trait OwnsDeck {
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

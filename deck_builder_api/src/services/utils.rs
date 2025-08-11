use sqlx::{query, PgPool};

pub async fn delete_all_cards(pg_pool: &PgPool) -> Result<(), sqlx::Error> {
    query!("DELETE FROM scryfall_cards;")
        .execute(pg_pool)
        .await?;
    Ok(())
}

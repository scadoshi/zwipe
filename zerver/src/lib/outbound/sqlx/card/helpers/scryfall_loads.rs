//! The Scryfall bulk file each sync loaded last, in `scryfall_loads`.

use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::PgConnection;

use crate::domain::card::models::scryfall_pull::ScryfallLoad;

/// When Scryfall made the `kind` file loaded last, or `None` before any load.
///
/// # Errors
///
/// Returns an error if the read fails.
pub async fn fetch_load(
    conn: &mut PgConnection,
    kind: &str,
) -> anyhow::Result<Option<DateTime<Utc>>> {
    sqlx::query_scalar("SELECT updated_at FROM scryfall_loads WHERE kind = $1")
        .bind(kind)
        .fetch_optional(&mut *conn)
        .await
        .with_context(|| format!("failed to read the last {kind} load"))
}

/// Records `load`, inside the caller's transaction, unless a newer file of its kind is already
/// recorded: the stored date only ever rises.
///
/// # Errors
///
/// Returns an error if the write fails.
pub async fn record_load(conn: &mut PgConnection, load: &ScryfallLoad) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO scryfall_loads (kind, updated_at, uri, items, loaded_at)
         VALUES ($1, $2, $3, $4, now())
         ON CONFLICT (kind) DO UPDATE SET
             updated_at = EXCLUDED.updated_at,
             uri = EXCLUDED.uri,
             items = EXCLUDED.items,
             loaded_at = EXCLUDED.loaded_at
         WHERE EXCLUDED.updated_at >= scryfall_loads.updated_at",
    )
    .bind(load.kind)
    .bind(load.updated_at)
    .bind(&load.uri)
    .bind(i64::try_from(load.items).unwrap_or(i64::MAX))
    .execute(&mut *conn)
    .await
    .with_context(|| format!("failed to record the {} load", load.kind))?;
    Ok(())
}

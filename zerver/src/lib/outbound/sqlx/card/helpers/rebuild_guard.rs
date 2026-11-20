//! The checks a rebuild of a `card_profiles` column from the oracle tag tables passes, inside its
//! transaction: its input isn't empty while profiles carry the column, and it leaves at least
//! [`scryfall_pull::REBUILD_FLOOR_PCT`] percent of the profiles that carried it. A refusal returns before the
//! transaction commits, so it rolls back.

use anyhow::Context;
use sqlx::PgConnection;

use crate::domain::card::models::scryfall_pull;

/// A `card_profiles` column rebuilt from the oracle tag tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    /// `oracle_tags`, the projection.
    OracleTags,
    /// `oracle_tags_by_role`, the grouping.
    TagGroups,
    /// `card_roles`, the derived roles.
    CardRoles,
}

impl Column {
    /// The column, as a refusal names it.
    fn what(self) -> &'static str {
        match self {
            Self::OracleTags => "the oracle tag projection",
            Self::TagGroups => "the oracle tag grouping",
            Self::CardRoles => "the card roles",
        }
    }

    /// Counts the profiles carrying the column.
    fn carrying_sql(self) -> &'static str {
        match self {
            Self::OracleTags => {
                "SELECT count(*) FROM card_profiles
                 WHERE COALESCE(oracle_tags, '[]'::jsonb) <> '[]'::jsonb"
            }
            Self::TagGroups => {
                "SELECT count(*) FROM card_profiles
                 WHERE COALESCE(oracle_tags_by_role, '{}'::jsonb) <> '{}'::jsonb"
            }
            Self::CardRoles => {
                "SELECT count(*) FROM card_profiles
                 WHERE COALESCE(card_roles, '[]'::jsonb) <> '[]'::jsonb"
            }
        }
    }
}

/// Profiles carrying `column` now.
async fn carrying(conn: &mut PgConnection, column: Column) -> anyhow::Result<u64> {
    let n: i64 = sqlx::query_scalar(column.carrying_sql())
        .fetch_one(&mut *conn)
        .await
        .with_context(|| format!("failed to count the profiles for {}", column.what()))?;
    Ok(u64::try_from(n).unwrap_or_default())
}

/// Checks `column`'s input before its rebuild and returns the profiles carrying it, for
/// [`after`].
///
/// # Errors
///
/// [`scryfall_pull::RefusedPull::NoInput`] when the card tags are empty while profiles carry
/// the column, or an error if a count fails.
pub async fn before(conn: &mut PgConnection, column: Column) -> anyhow::Result<u64> {
    let input: i64 = sqlx::query_scalar("SELECT count(*) FROM card_oracle_tags")
        .fetch_one(&mut *conn)
        .await
        .context("failed to count the card tags")?;
    let carrying = carrying(&mut *conn, column).await?;
    scryfall_pull::check_rebuild_input(
        column.what(),
        u64::try_from(input).unwrap_or_default(),
        carrying,
    )?;
    Ok(carrying)
}

/// Checks `column`'s rebuild left at least [`scryfall_pull::REBUILD_FLOOR_PCT`] percent of the `before`
/// profiles carrying it.
///
/// # Errors
///
/// [`scryfall_pull::RefusedPull::RebuildShrunk`] under the floor, or an error if the count
/// fails.
pub async fn after(conn: &mut PgConnection, column: Column, before: u64) -> anyhow::Result<()> {
    let after = carrying(&mut *conn, column).await?;
    scryfall_pull::check_rebuild(column.what(), after, before)?;
    Ok(())
}

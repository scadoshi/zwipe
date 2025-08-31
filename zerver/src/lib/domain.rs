pub mod auth;
pub mod card;
pub mod deck;
pub mod health;
pub mod logo;
pub mod user;

use anyhow::anyhow;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("database error: {0}")]
pub struct DatabaseError(anyhow::Error);

impl From<sqlx::Error> for DatabaseError {
    fn from(value: sqlx::Error) -> Self {
        Self(anyhow!("{value}"))
    }
}

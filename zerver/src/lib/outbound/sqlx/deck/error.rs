//! Error mappings between `SQLx` errors and deck domain errors.

use thiserror::Error;
use zwipe_core::domain::deck::board::InvalidBoard;
use zwipe_core::domain::deck::format::InvalidFormat;
use zwipe_core::domain::deck::quantity::InvalidQuantity;

use crate::domain::deck::models::deck::create_deck_profile::CreateDeckProfileError;
use crate::domain::deck::models::deck::delete_deck::DeleteDeckError;
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
use crate::domain::deck::models::deck::update_deck_profile::UpdateDeckProfileError;
use crate::domain::deck::models::deck_card::create_deck_card::CreateDeckCardError;
use crate::domain::deck::models::deck_card::delete_deck_card::DeleteDeckCardError;
use crate::domain::deck::models::deck_card::get_deck_card::GetDeckCardError;
use crate::domain::deck::models::deck_card::update_deck_card::UpdateDeckCardError;
use crate::outbound::sqlx::postgres::IsConstraintViolation;

/// Errors from converting a database deck profile row into domain types.
#[derive(Debug, Error)]
#[expect(missing_docs, reason = "each variant's message names the failure")]
pub enum IntoDeckProfileError {
    #[error(transparent)]
    Format(#[from] InvalidFormat),
}

impl From<IntoDeckProfileError> for CreateDeckProfileError {
    fn from(value: IntoDeckProfileError) -> Self {
        Self::DeckFromDb(value.into())
    }
}

impl From<IntoDeckProfileError> for UpdateDeckProfileError {
    fn from(value: IntoDeckProfileError) -> Self {
        Self::DeckFromDb(value.into())
    }
}

impl From<IntoDeckProfileError> for GetDeckProfileError {
    fn from(value: IntoDeckProfileError) -> Self {
        GetDeckProfileError::DeckProfileFromDb(value.into())
    }
}

impl From<sqlx::Error> for CreateDeckProfileError {
    fn from(value: sqlx::Error) -> Self {
        let matched = match value {
            e if e.is_unique_constraint_violation() => Self::Duplicate,
            e => Self::Database(e.into()),
        };
        tracing::error!("{:#?}", matched);
        matched
    }
}

impl From<sqlx::Error> for GetDeckProfileError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for UpdateDeckProfileError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            e if e.is_unique_constraint_violation() => Self::Duplicate,
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for DeleteDeckError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

/// Errors from converting a database deck card row into domain types.
#[derive(Debug, Error)]
#[expect(missing_docs, reason = "each variant's message names the failure")]
pub enum IntoDeckCardError {
    #[error(transparent)]
    InvalidId(uuid::Error),
    #[error(transparent)]
    InvalidDeckId(uuid::Error),
    #[error(transparent)]
    InvalidCardId(uuid::Error),
    #[error(transparent)]
    InvalidOracleId(uuid::Error),
    #[error(transparent)]
    InvalidQuantity(#[from] InvalidQuantity),
    #[error(transparent)]
    InvalidBoard(#[from] InvalidBoard),
}

impl From<IntoDeckCardError> for CreateDeckCardError {
    fn from(value: IntoDeckCardError) -> Self {
        Self::DeckCardFromDb(value.into())
    }
}

impl From<IntoDeckCardError> for GetDeckCardError {
    fn from(value: IntoDeckCardError) -> Self {
        Self::DeckCardFromDb(value.into())
    }
}

impl From<IntoDeckCardError> for UpdateDeckCardError {
    fn from(value: IntoDeckCardError) -> Self {
        Self::DeckCardFromDb(value.into())
    }
}

impl From<sqlx::Error> for CreateDeckCardError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            e if e.is_unique_constraint_violation() => Self::Duplicate,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for GetDeckCardError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for UpdateDeckCardError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            e if e.is_check_constraint_violation() => Self::QuantityUnderflow,
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for DeleteDeckCardError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

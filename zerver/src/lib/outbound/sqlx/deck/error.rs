use crate::{
    domain::deck::models::{
        deck::{
            create_deck_profile::CreateDeckProfileError, deck_name::InvalidDeckname,
            delete_deck::DeleteDeckError, get_deck::GetDeckProfileError,
            get_deck_profiles::GetDeckProfilesError, update_deck_profile::UpdateDeckProfileError,
        },
        deck_card::{
            create_deck_card::CreateDeckCardError, delete_deck_card::DeleteDeckCardError,
            get_deck_card::GetDeckCardError, quantity::InvalidQuantity,
            update_deck_card::UpdateDeckCardError,
        },
    },
    outbound::sqlx::postgres::IsConstraintViolation,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntoDeckProfileError {
    #[error(transparent)]
    DeckName(InvalidDeckname),
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

#[derive(Debug, Error)]
pub enum IntoDeckCardError {
    #[error(transparent)]
    InvalidId(uuid::Error),
    #[error(transparent)]
    InvalidDeckId(uuid::Error),
    #[error(transparent)]
    InvalidCardId(uuid::Error),
    #[error(transparent)]
    InvalidQuantity(InvalidQuantity),
}

impl From<InvalidQuantity> for IntoDeckCardError {
    fn from(value: InvalidQuantity) -> Self {
        Self::InvalidQuantity(value)
    }
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
            e if e.is_check_constraint_violation() => Self::InvalidResultingQuantity,
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

impl From<sqlx::Error> for GetDeckProfilesError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

impl From<IntoDeckProfileError> for GetDeckProfilesError {
    fn from(value: IntoDeckProfileError) -> Self {
        Self::DeckProfileFromDb(value.into())
    }
}

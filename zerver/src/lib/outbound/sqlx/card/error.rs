use crate::{
    domain::card::models::{
        card_profile::get_card_profile::GetCardProfileError,
        create_card::CreateCardError,
        scryfall_data::get_scryfall_data::{GetScryfallDataError, SearchScryfallDataError},
    },
    outbound::sqlx::postgres::IsConstraintViolation,
};

impl From<sqlx::Error> for CreateCardError {
    fn from(value: sqlx::Error) -> Self {
        if value.is_unique_constraint_violation() {
            return CreateCardError::UniqueConstraintViolation(value.into());
        }
        CreateCardError::Database(value.into())
    }
}

impl From<sqlx::Error> for GetScryfallDataError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => GetScryfallDataError::NotFound,
            e => GetScryfallDataError::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for GetCardProfileError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => Self::NotFound,
            e => Self::Database(e.into()),
        }
    }
}

impl From<sqlx::Error> for SearchScryfallDataError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

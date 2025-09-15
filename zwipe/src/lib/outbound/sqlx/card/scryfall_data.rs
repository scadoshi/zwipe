use crate::domain::card::models::scryfall_data::SearchScryfallDataError;

pub mod all_parts;
pub mod card_faces;
pub mod colors;
pub mod image_uris;
pub mod legalities;
pub mod prices;

// ======
//  error
// ======

impl From<sqlx::Error> for SearchScryfallDataError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.into())
    }
}

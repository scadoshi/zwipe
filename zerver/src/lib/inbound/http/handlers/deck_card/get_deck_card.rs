//! Error conversions for deck card lookups.

use crate::domain::deck::models::deck_card::get_deck_card::GetDeckCardError;
use crate::inbound::http::{ApiError, To500};

impl From<GetDeckCardError> for ApiError {
    fn from(value: GetDeckCardError) -> Self {
        match value {
            GetDeckCardError::NotFound => Self::NotFound("deck card not found".to_string()),
            GetDeckCardError::Database(e) | GetDeckCardError::DeckCardFromDb(e) => e.to_500(),
            // Someone else's deck and no deck at all read the same, as on
            // the deck routes.
            GetDeckCardError::Forbidden => Self::NotFound("deck not found".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Someone else's deck reads the same as no deck, as on the other deck
    /// routes.
    #[test]
    fn foreign_deck_answers_not_found() {
        let error = ApiError::from(GetDeckCardError::Forbidden);
        assert!(
            matches!(&error, ApiError::NotFound(message) if message == "deck not found"),
            "got {error:?}"
        );
    }
}

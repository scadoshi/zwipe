#[cfg(feature = "zerver")]
use crate::inbound::http::Log500;
#[cfg(feature = "zerver")]
use crate::{
    domain::deck::models::deck_card::get_deck_card::GetDeckCardError, inbound::http::ApiError,
};

#[cfg(feature = "zerver")]
impl From<GetDeckCardError> for ApiError {
    fn from(value: GetDeckCardError) -> Self {
        match value {
            GetDeckCardError::NotFound => Self::NotFound("deck card not found".to_string()),
            GetDeckCardError::Database(e) => e.log_500(),
            GetDeckCardError::DeckCardFromDb(e) => e.log_500(),
            GetDeckCardError::Forbidden => Self::Forbidden(GetDeckCardError::Forbidden.to_string()),
        }
    }
}

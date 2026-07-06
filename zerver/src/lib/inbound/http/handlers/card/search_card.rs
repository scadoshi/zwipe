#[cfg(feature = "zerver")]
use crate::{
    domain::card::models::search_card::error::SearchCardsError,
    inbound::http::{ApiError, AppState, Log500, middleware::AuthenticatedUser},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use zwipe_core::domain::card::{Card, search_card::card_filter::CardQuery};

#[cfg(feature = "zerver")]
impl From<SearchCardsError> for ApiError {
    fn from(value: SearchCardsError) -> Self {
        value.log_500()
    }
}

/// Searches cards using a `CardQuery` deserialized from the JSON body.
#[cfg(feature = "zerver")]
pub async fn search_cards(
    _: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<CardQuery>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    state
        .card_service
        .search_cards(&body)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}

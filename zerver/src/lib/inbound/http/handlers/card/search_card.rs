use crate::{
    domain::card::models::search_card::error::SearchCardsError,
    inbound::http::{ApiError, AppState, To500, middleware::AuthenticatedUser},
};
use axum::{Json, extract::State, http::StatusCode};
use zwipe_core::domain::card::{Card, search_card::card_filter::CardQuery};

impl From<SearchCardsError> for ApiError {
    fn from(value: SearchCardsError) -> Self {
        value.to_500()
    }
}

/// Searches cards using a `CardQuery` deserialized from the JSON body.
pub async fn search_cards(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<CardQuery>,
) -> Result<(StatusCode, Json<Vec<Card>>), ApiError> {
    state
        .card_service
        .search_cards(&body, user.id)
        .await
        .map_err(ApiError::from)
        .map(|cards| (StatusCode::OK, Json(cards)))
}

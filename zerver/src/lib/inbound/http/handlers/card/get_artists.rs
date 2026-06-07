#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::{ports::CardService, requests::get_artists::GetArtistsError},
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State};
#[cfg(feature = "zerver")]
use reqwest::StatusCode;

#[cfg(feature = "zerver")]
impl From<GetArtistsError> for ApiError {
    fn from(value: GetArtistsError) -> Self {
        match value {
            GetArtistsError::Database(e) => e.log_500(),
        }
    }
}

/// Returns distinct artist names.
#[cfg(feature = "zerver")]
pub async fn get_artists<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
) -> Result<(StatusCode, Json<Vec<String>>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    state
        .card_service
        .get_artists()
        .await
        .map_err(ApiError::from)
        .map(|artists| (StatusCode::OK, Json(artists)))
}

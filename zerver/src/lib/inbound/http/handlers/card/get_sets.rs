#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::{models::get_sets::GetSetsError, ports::CardService},
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{extract::State, Json};
#[cfg(feature = "zerver")]
use reqwest::StatusCode;

#[cfg(feature = "zerver")]
impl From<GetSetsError> for ApiError {
    fn from(value: GetSetsError) -> Self {
        match value {
            GetSetsError::Database(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn get_sets<AS, US, HS, CS, DS>(
    _: AuthenticatedUser,
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
        .get_sets()
        .await
        .map_err(ApiError::from)
        .map(|sets| (StatusCode::OK, Json(sets)))
}

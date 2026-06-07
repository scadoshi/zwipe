#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::{ports::CardService, requests::get_card_types::GetCardTypesError},
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{extract::State, Json};
#[cfg(feature = "zerver")]
use reqwest::StatusCode;

#[cfg(feature = "zerver")]
impl From<GetCardTypesError> for ApiError {
    fn from(value: GetCardTypesError) -> Self {
        match value {
            GetCardTypesError::Database(e) => e.log_500(),
        }
    }
}

/// Returns distinct card type names.
#[cfg(feature = "zerver")]
pub async fn get_card_types<AS, US, HS, CS, DS>(
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
        .get_card_types()
        .await
        .map_err(ApiError::from)
        .map(|all_types| (StatusCode::OK, Json(all_types)))
}

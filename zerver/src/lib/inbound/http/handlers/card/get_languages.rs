#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::{models::get_languages::GetLanguagesError, ports::CardService},
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
impl From<GetLanguagesError> for ApiError {
    fn from(value: GetLanguagesError) -> Self {
        match value {
            GetLanguagesError::Database(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn get_languages<AS, US, HS, CS, DS>(
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
        .get_languages()
        .await
        .map_err(ApiError::from)
        .map(|languages| (StatusCode::OK, Json(languages)))
}

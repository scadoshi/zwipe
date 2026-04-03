#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::{ports::CardService, requests::get_oracle_words::GetOracleWordsError},
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
impl From<GetOracleWordsError> for ApiError {
    fn from(value: GetOracleWordsError) -> Self {
        match value {
            GetOracleWordsError::Database(e) => e.log_500(),
        }
    }
}

/// Returns distinct normalized words extracted from oracle text.
#[cfg(feature = "zerver")]
pub async fn get_oracle_words<AS, US, HS, CS, DS>(
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
        .get_oracle_words()
        .await
        .map_err(ApiError::from)
        .map(|all_words| (StatusCode::OK, Json(all_words)))
}

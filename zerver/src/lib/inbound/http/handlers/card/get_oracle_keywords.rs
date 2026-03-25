#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::{models::get_oracle_keywords::GetOracleKeywordsError, ports::CardService},
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
impl From<GetOracleKeywordsError> for ApiError {
    fn from(value: GetOracleKeywordsError) -> Self {
        match value {
            GetOracleKeywordsError::Database(e) => e.log_500(),
        }
    }
}

/// Returns distinct oracle keyword ability names.
#[cfg(feature = "zerver")]
pub async fn get_oracle_keywords<AS, US, HS, CS, DS>(
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
        .get_oracle_keywords()
        .await
        .map_err(ApiError::from)
        .map(|all_keywords| (StatusCode::OK, Json(all_keywords)))
}

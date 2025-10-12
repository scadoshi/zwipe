#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::{
            models::{
                card_profile::get_card_profile::GetCardProfileError,
                get_card::{GetCard, GetCardError},
                scryfall_data::GetScryfallDataError,
                Card,
            },
            ports::CardService,
        },
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

#[cfg(feature = "zerver")]
impl From<GetCardProfileError> for ApiError {
    fn from(value: GetCardProfileError) -> Self {
        match value {
            GetCardProfileError::NotFound => Self::NotFound("card profile not found".to_string()),
            GetCardProfileError::CardProfileFromDb(e) => e.log_500(),
            GetCardProfileError::Database(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<GetScryfallDataError> for ApiError {
    fn from(value: GetScryfallDataError) -> Self {
        match value {
            GetScryfallDataError::NotFound => Self::NotFound("scryfall data not found".to_string()),
            GetScryfallDataError::Database(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<GetCardError> for ApiError {
    fn from(value: GetCardError) -> Self {
        match value {
            GetCardError::GetCardProfileError(e) => ApiError::from(e),
            GetCardError::GetScryfallDataError(e) => ApiError::from(e),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn get_card<AS, US, HS, CS, DS>(
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(card_profile_id): Path<String>,
    _: AuthenticatedUser,
) -> Result<(StatusCode, Json<Card>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = GetCard::new(&card_profile_id)?;

    state
        .card_service
        .get_card(&request)
        .await
        .map_err(ApiError::from)
        .map(|card| (StatusCode::OK, Json(card)))
}

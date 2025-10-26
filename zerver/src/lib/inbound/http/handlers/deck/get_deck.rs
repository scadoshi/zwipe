#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck::{get_deck::GetDeckError, Deck},
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState},
};
#[cfg(feature = "zerver")]
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
impl From<GetDeckError> for ApiError {
    fn from(value: GetDeckError) -> Self {
        match value {
            GetDeckError::GetCardError(e) => ApiError::from(e),
            GetDeckError::GetDeckCardError(e) => ApiError::from(e),
            GetDeckError::GetCardProfileError(e) => ApiError::from(e),
            GetDeckError::GetDeckProfileError(e) => ApiError::from(e),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn get_deck<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(deck_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Deck>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfile;

    let request = GetDeckProfile::new(user.id, deck_id);

    state
        .deck_service
        .get_deck(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck| (StatusCode::OK, Json(deck)))
}

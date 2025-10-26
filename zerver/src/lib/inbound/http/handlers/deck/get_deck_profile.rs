#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck::{
                deck_profile::DeckProfile,
                get_deck_profile::{GetDeckProfile, GetDeckProfileError},
            },
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
impl From<GetDeckProfileError> for ApiError {
    fn from(value: GetDeckProfileError) -> Self {
        use crate::inbound::http::Log500;

        match value {
            GetDeckProfileError::Database(e) => e.log_500(),
            GetDeckProfileError::DeckProfileFromDb(e) => e.log_500(),
            GetDeckProfileError::Forbidden => {
                Self::Forbidden("deck does not belong to requesting user".to_string())
            }
            GetDeckProfileError::NotFound => Self::NotFound("deck not found".to_string()),
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn get_deck_profile<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Path(deck_id): Path<Uuid>,
) -> Result<(StatusCode, Json<DeckProfile>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = GetDeckProfile::new(user.id, deck_id);

    state
        .deck_service
        .get_deck_profile(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck_profile| (StatusCode::OK, Json(deck_profile)))
}

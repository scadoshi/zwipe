#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};

#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::{
            models::deck::{
                deck_profile::DeckProfile,
                get_deck_profiles::{
                    GetDeckProfiles, GetDeckProfilesError, InvalidGetDeckProfiles,
                },
            },
            ports::DeckService,
        },
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState},
};

#[cfg(feature = "zerver")]
impl From<GetDeckProfilesError> for ApiError {
    fn from(value: GetDeckProfilesError) -> Self {
        use crate::inbound::http::Log500;

        match value {
            GetDeckProfilesError::Database(e) => e.log_500(),
            GetDeckProfilesError::DeckProfileFromDb(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<InvalidGetDeckProfiles> for ApiError {
    fn from(value: InvalidGetDeckProfiles) -> Self {
        match value {
            InvalidGetDeckProfiles::UserId(e) => {
                Self::UnprocessableEntity(format!("invalid user id: {e}"))
            }
        }
    }
}

#[cfg(feature = "zerver")]
pub async fn get_deck_profiles<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
) -> Result<(StatusCode, Json<Vec<DeckProfile>>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = GetDeckProfiles::new(user.id);
    state
        .deck_service
        .get_deck_profiles(&request)
        .await
        .map_err(ApiError::from)
        .map(|deck_profiles| (StatusCode::OK, Json(deck_profiles)))
}

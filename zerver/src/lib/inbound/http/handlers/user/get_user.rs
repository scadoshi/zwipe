#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::ports::AuthService,
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::{
            models::{
                get_user::{GetUser, GetUserError},
                User,
            },
            ports::UserService,
        },
    },
    inbound::http::{middleware::AuthenticatedUser, ApiError, AppState, Log500},
};
#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};

#[cfg(feature = "zerver")]
impl From<GetUserError> for ApiError {
    fn from(value: GetUserError) -> Self {
        match value {
            GetUserError::NotFound => Self::NotFound("user not found".to_string()),
            GetUserError::Database(e) => e.log_500(),
            GetUserError::UserFromDb(e) => e.log_500(),
        }
    }
}

#[cfg(feature = "zerver")]
impl From<AuthenticatedUser> for GetUser {
    fn from(value: AuthenticatedUser) -> Self {
        GetUser::from(value.id)
    }
}

#[cfg(feature = "zerver")]
pub async fn get_user<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
) -> Result<(StatusCode, Json<User>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = GetUser::from(user);

    state
        .user_service
        .get_user(&request)
        .await
        .map_err(ApiError::from)
        .map(|user| (StatusCode::OK, Json(user.into())))
}

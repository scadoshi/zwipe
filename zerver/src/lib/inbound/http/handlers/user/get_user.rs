#[cfg(feature = "zerver")]
use crate::{
    domain::user::models::get_user::GetUserError,
    inbound::http::{ApiError, AppState, Log500, middleware::AuthenticatedUser},
};
#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use zwipe_core::domain::user::{User, requests::get_user::GetUser};

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

/// Returns the authenticated user's own profile (identity from JWT, no path params).
#[cfg(feature = "zerver")]
pub async fn get_user(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<User>), ApiError> {
    let request = GetUser::from(user);

    state
        .user_service
        .get_user(&request)
        .await
        .map_err(ApiError::from)
        .map(|user| (StatusCode::OK, Json(user)))
}

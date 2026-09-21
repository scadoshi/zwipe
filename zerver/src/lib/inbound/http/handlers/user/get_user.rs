use crate::{
    domain::user::models::get_user::GetUserError,
    inbound::http::{ApiError, AppState, To500, middleware::AuthenticatedUser},
};
use axum::{Json, extract::State, http::StatusCode};
use zwipe_core::domain::user::{User, requests::get_user::GetUser};

impl From<GetUserError> for ApiError {
    fn from(value: GetUserError) -> Self {
        match value {
            GetUserError::NotFound => Self::NotFound("user not found".to_string()),
            GetUserError::Database(e) => e.to_500(),
            GetUserError::UserFromDb(e) => e.to_500(),
        }
    }
}

impl From<AuthenticatedUser> for GetUser {
    fn from(value: AuthenticatedUser) -> Self {
        GetUser::from(value.id)
    }
}

/// Returns the authenticated user's own profile (identity from JWT, no path params).
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

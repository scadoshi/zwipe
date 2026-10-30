//! Returns the authenticated user's profile.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::requests::get_user::GetUser;

use crate::domain::user::models::get_user::GetUserError;
use crate::inbound::http::middleware::AuthenticatedUser;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<GetUserError> for ApiError {
    fn from(value: GetUserError) -> Self {
        match value {
            GetUserError::NotFound => Self::NotFound("user not found".to_string()),
            GetUserError::Database(e) | GetUserError::UserFromDb(e) => e.to_500(),
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

#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use zwipe_core::http::contracts::auth::HttpDeleteUser;

use crate::{domain::auth::requests::delete_user::InvalidDeleteUser, inbound::http::ApiError};
#[cfg(feature = "zerver")]
use crate::{
    domain::auth::requests::delete_user::{DeleteUser, DeleteUserError},
    inbound::http::{AppState, Log500, middleware::AuthenticatedUser},
};

#[cfg(feature = "zerver")]
impl From<DeleteUserError> for ApiError {
    fn from(value: DeleteUserError) -> Self {
        match value {
            DeleteUserError::NotFound => Self::NotFound("user not found".to_string()),
            DeleteUserError::Database(e) => e.log_500(),
            DeleteUserError::AuthenticateUserError(e) => ApiError::from(e),
        }
    }
}

impl From<InvalidDeleteUser> for ApiError {
    fn from(value: InvalidDeleteUser) -> Self {
        match value {
            InvalidDeleteUser::Userid(e) => {
                Self::UnprocessableEntity(format!("invalid user id {e}"))
            }
            InvalidDeleteUser::Password => {
                Self::UnprocessableEntity("invalid password".to_string())
            }
        }
    }
}

/// Deletes the user's account and all associated data after password verification.
#[cfg(feature = "zerver")]
pub async fn delete_user(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Json(body): Json<HttpDeleteUser>,
) -> Result<StatusCode, ApiError> {
    let request = DeleteUser::new(user.id, &body.password)?;

    state
        .auth_service
        .delete_user(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::NO_CONTENT)
}

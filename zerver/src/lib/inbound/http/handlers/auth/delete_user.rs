#[cfg(feature = "zerver")]
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{domain::auth::models::delete_user::InvalidDeleteUser, inbound::http::ApiError};
#[cfg(feature = "zerver")]
use crate::{
    domain::{
        auth::{
            models::delete_user::{DeleteUser, DeleteUserError},
            ports::AuthService,
        },
        card::ports::CardService,
        deck::ports::DeckService,
        health::ports::HealthService,
        user::ports::UserService,
    },
    inbound::http::{middleware::AuthenticatedUser, AppState, Log500},
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpDeleteUser {
    pub password: String,
}

#[cfg(feature = "zerver")]
pub async fn delete_user<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
    Json(body): Json<HttpDeleteUser>,
) -> Result<StatusCode, ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let request = DeleteUser::new(user.id, &body.password)?;

    state
        .auth_service
        .delete_user(&request)
        .await
        .map_err(ApiError::from)
        .map(|_| StatusCode::NO_CONTENT)
}

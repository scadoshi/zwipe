//! Pending email change confirmation handler.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use zwipe_core::http::contracts::auth::HttpConfirmEmailChange;

use crate::domain::auth::requests::confirm_email_change::{
    ConfirmEmailChange, ConfirmEmailChangeError,
};
use crate::domain::metrics::models::kinds::AuditAction;
use crate::inbound::http::{ApiError, AppState, To500};

impl From<ConfirmEmailChangeError> for ApiError {
    fn from(value: ConfirmEmailChangeError) -> Self {
        match value {
            ConfirmEmailChangeError::InvalidToken => {
                Self::UnprocessableEntity("token not found or expired".to_string())
            }
            ConfirmEmailChangeError::AddressTaken => {
                Self::Conflict("that address is already in use by another account".to_string())
            }
            ConfirmEmailChangeError::UserFromDb(e) | ConfirmEmailChangeError::Database(e) => {
                e.to_500()
            }
        }
    }
}

/// Switches a user to their pending email using the token from the link sent
/// to it.
pub async fn confirm_email_change(
    State(state): State<AppState>,
    Json(body): Json<HttpConfirmEmailChange>,
) -> Result<StatusCode, ApiError> {
    let request = ConfirmEmailChange { token: body.token };
    let user = state
        .auth_service
        .confirm_email_change(&request)
        .await
        .map_err(ApiError::from)?;

    let metrics = std::sync::Arc::clone(&state.metrics_service);
    tokio::spawn(async move {
        if let Err(e) = metrics
            .record_audit(user.id, AuditAction::EmailChanged)
            .await
        {
            tracing::warn!(error = ?e, "metrics: audit email change failed");
        }
    });

    Ok(StatusCode::OK)
}

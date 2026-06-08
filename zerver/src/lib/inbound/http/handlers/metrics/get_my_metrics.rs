use axum::{Json, extract::State, http::StatusCode};

use crate::{
    domain::{
        auth::ports::AuthService, card::ports::CardService, deck::ports::DeckService,
        health::ports::HealthService, user::ports::UserService,
    },
    inbound::http::{ApiError, AppState, middleware::AuthenticatedUser},
};
use zwipe_core::http::contracts::metrics::HttpLifetimeCounters;

/// Returns the caller's lifetime metric totals.
pub async fn get_my_metrics<AS, US, HS, CS, DS>(
    user: AuthenticatedUser,
    State(state): State<AppState<AS, US, HS, CS, DS>>,
) -> Result<(StatusCode, Json<HttpLifetimeCounters>), ApiError>
where
    AS: AuthService,
    US: UserService,
    HS: HealthService,
    CS: CardService,
    DS: DeckService,
{
    let counters = state
        .metrics_service
        .lifetime_counters(user.id)
        .await
        .map_err(ApiError::from)?;

    Ok((StatusCode::OK, Json(HttpLifetimeCounters::from(counters))))
}

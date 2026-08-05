//! The featured flavor card — one shared pick per UTC hour.
//!
//! Everyone sees the same card at the same moment: the pick is deterministic
//! per hour (see the repository's `featured_flavor_id`), and the `TtlSlot` on
//! AppState serves it from memory with a deadline pinned to the top of the
//! next UTC hour — one DB pass per hour flip, single-flight under load, and a
//! restart mid-hour re-derives the identical card. Plan:
//! `context/plans/featured_flavor.md`.

#[cfg(feature = "zerver")]
use axum::{Json, extract::State, http::StatusCode};
#[cfg(feature = "zerver")]
use chrono::Timelike;
#[cfg(feature = "zerver")]
use std::time::{Duration, Instant};
#[cfg(feature = "zerver")]
use zwipe_core::domain::card::Card;

#[cfg(feature = "zerver")]
use crate::{
    domain::card::requests::get_card::GetCardError,
    inbound::http::{ApiError, AppState},
};

/// Serves the hour's featured flavor card (unauthed; the app home screen and
/// the zwipe.net home page both read it).
#[cfg(feature = "zerver")]
pub async fn get_featured_flavor(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Card>), ApiError> {
    let service = std::sync::Arc::clone(&state.card_service);
    let card = state
        .featured_flavor
        .get_or_refresh(|| async move {
            let now = chrono::Utc::now();
            let hour_key = now.format("%Y-%m-%d-%H").to_string();
            let card = service.featured_flavor(&hour_key).await?;
            Ok::<_, GetCardError>((card, next_utc_hour_deadline(&now)))
        })
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::OK, Json(card)))
}

/// Monotonic deadline for the top of the next UTC hour, so every slot flip
/// lands on the wall-clock boundary rather than drifting per refresh time.
#[cfg(feature = "zerver")]
fn next_utc_hour_deadline(now: &chrono::DateTime<chrono::Utc>) -> Instant {
    let secs_into_hour = u64::from(now.minute()) * 60 + u64::from(now.second());
    Instant::now() + Duration::from_secs(3600 - secs_into_hour.min(3599))
}

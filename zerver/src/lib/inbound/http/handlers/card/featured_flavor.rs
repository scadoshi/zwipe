//! The featured flavor card — one shared pick per UTC hour.
//!
//! Everyone sees the same card at the same moment: the pick is deterministic
//! per hour (see the repository's `featured_flavor_id`), and the `TtlSlot` on
//! AppState serves it from memory with a deadline pinned to the top of the
//! next UTC hour — one DB pass per hour flip, single-flight under load, and a
//! restart mid-hour re-derives the identical card. Plan:
//! `context/plans/featured_flavor.md`.

#[cfg(feature = "zerver")]
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
};
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
///
/// `Cache-Control` declares freshness to the top of the UTC hour so edge
/// caches (Cloudflare fronts prod) expire exactly when the pick flips.
/// Without it, a CF cache rule edge-cached the response for ~20 hours and the
/// whole world saw one card all day (2026-08-06).
#[cfg(feature = "zerver")]
pub async fn get_featured_flavor(
    State(state): State<AppState>,
) -> Result<(StatusCode, [(header::HeaderName, String); 1], Json<Card>), ApiError> {
    let service = std::sync::Arc::clone(&state.card_service);
    let card = state
        .featured_flavor
        .get_or_refresh(|| async move {
            let now = chrono::Utc::now();
            let hour_key = now.format("%Y-%m-%d-%H").to_string();
            let card = service.featured_flavor(&hour_key).await?;
            Ok::<_, GetCardError>((
                card,
                Instant::now() + Duration::from_secs(secs_until_next_utc_hour(&now)),
            ))
        })
        .await
        .map_err(ApiError::from)?;
    let max_age = secs_until_next_utc_hour(&chrono::Utc::now());
    Ok((
        StatusCode::OK,
        [(header::CACHE_CONTROL, format!("public, max-age={max_age}"))],
        Json(card),
    ))
}

/// Seconds until the top of the next UTC hour — the slot deadline and the
/// response `max-age` both derive from it, so the in-memory flip and the edge
/// cache expiry land on the same wall-clock boundary.
#[cfg(feature = "zerver")]
fn secs_until_next_utc_hour(now: &chrono::DateTime<chrono::Utc>) -> u64 {
    let secs_into_hour = u64::from(now.minute()) * 60 + u64::from(now.second());
    3600 - secs_into_hour.min(3599)
}

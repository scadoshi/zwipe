//! Post-mutation deck-completion check.
//!
//! Called by deck and deck_card handlers after any mutation succeeds. Loads
//! the full deck, runs validation, and if the deck has just reached a valid
//! state for the first time, stamps `first_completed_at`, emits a
//! `DeckCompleted` event, and increments the lifetime completed counter.
//!
//! Designed to be fire-and-forget via `tokio::spawn` — a metrics failure
//! must not break the user-visible mutation that triggered it.

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{
    deck::ports::ErasedDeckService,
    metrics::{models::kinds::EventKind, ports::ErasedMetricsService},
};
use zwipe_core::domain::deck::requests::get_deck_profile::GetDeckProfile;

/// Runs the deck-completion check after a mutation. Logs but does not return
/// errors — callers spawn this off the request path.
pub async fn check_deck_completion(
    deck_service: Arc<dyn ErasedDeckService>,
    metrics: Arc<dyn ErasedMetricsService>,
    user_id: Uuid,
    deck_id: Uuid,
) {
    let request = GetDeckProfile::new(user_id, deck_id);
    let deck = match deck_service.get_deck(&request).await {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(error = ?e, %deck_id, "metrics: get_deck failed during completion check");
            return;
        }
    };

    if !deck.warnings.is_empty() {
        return;
    }

    match metrics.mark_deck_first_completed(deck_id).await {
        Ok(true) => {}
        Ok(false) => return,
        Err(e) => {
            tracing::warn!(error = ?e, %deck_id, "metrics: mark_deck_first_completed failed");
            return;
        }
    }

    if let Err(e) = metrics
        .record_event(user_id, EventKind::DeckCompleted, Some(deck_id))
        .await
    {
        tracing::warn!(error = ?e, %deck_id, "metrics: record deck_completed event failed");
    }

    if let Err(e) = metrics.increment_decks_completed(user_id).await {
        tracing::warn!(error = ?e, %deck_id, "metrics: increment decks_completed failed");
    }
}

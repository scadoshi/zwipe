//! Background session upkeep loop and app context providers.
//!
//! Periodically keeps the access token fresh via the single-flight
//! [`EnsureFresh`] helper, and initializes the app-wide Dioxus context
//! (session, client, card search state, theme, telemetry buffer).

use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::components::telemetry::{
    flush_loop::spawn_usage_flusher, usage_buffer::UsageBuffer,
};
use crate::outbound::{client::ZwipeClient, session::Persist};
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::interval;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::{search_card::card_filter::builder::CardFilterBuilder, Card};
use zwipe_core::domain::user::models::theme::ThemeConfig;

/// Spawns a background task that periodically refreshes the user session.
///
/// Also initializes context providers for session, client, card filter, and cards.
pub fn spawn_upkeeper() {
    tracing::debug!("upkeeper spawned");
    let session = use_signal(Session::infallible_load);
    use_context_provider(|| session);

    let client = use_signal(ZwipeClient::new);
    use_context_provider(|| client);

    // card search state - used by deck card screens
    let filter_builder = use_signal(CardFilterBuilder::default);
    use_context_provider(|| filter_builder);

    let cards = use_signal(Vec::<Card>::new);
    use_context_provider(|| cards);

    let last_search_filter: Signal<Option<CardFilterBuilder>> = use_signal(|| None);
    use_context_provider(|| last_search_filter);

    // Theme — initialize from session preferences if logged in
    let theme = use_signal(|| {
        session
            .peek()
            .as_ref()
            .map(|s| ThemeConfig::from(&s.preferences))
            .unwrap_or_default()
    });
    use_context_provider(|| theme);

    // Usage telemetry buffer (swipe / search counters, flushed every 30s).
    let usage_buffer = use_signal(UsageBuffer::new);
    use_context_provider(|| usage_buffer);
    spawn_usage_flusher(usage_buffer.peek().clone(), client, session);

    spawn(async move {
        // first tick fires immediately — this is the cold-start refresh
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            // single-flight: free of races with any in-flight user action
            let _ = session.ensure_fresh(client).await;
        }
    });
}

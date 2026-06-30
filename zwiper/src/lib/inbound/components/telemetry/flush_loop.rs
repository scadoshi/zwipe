//! Background flusher that posts batched usage to the backend.

use std::time::Duration;

use dioxus::prelude::{ReadableExt, Signal, document, spawn};
use tokio::time::interval;

use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::outbound::client::ZwipeClient;
use crate::outbound::client::metrics::record_usage::ClientRecordUsage;
use zwipe_core::domain::auth::models::session::Session;

/// Interval between automatic flushes when the app is foregrounded.
pub const FLUSH_INTERVAL_SECS: u64 = 30;

/// Spawns a periodic flush task. Each tick snapshots counters and posts them
/// if any are non-zero. The task drops the batch on HTTP failure — vanity
/// data isn't worth retry plumbing.
pub fn spawn_usage_flusher(
    buffer: UsageBuffer,
    client: Signal<ZwipeClient>,
    session: Signal<Option<Session>>,
) {
    spawn(async move {
        let mut tick = interval(Duration::from_secs(FLUSH_INTERVAL_SECS));
        loop {
            tick.tick().await;
            flush_once(&buffer, &client, &session).await;
        }
    });
}

/// Spawns a task that flushes whenever the app is **backgrounded** — the JS
/// `visibilitychange → hidden` / `pagehide` events. Backgrounding precedes a
/// swipe-to-kill, so this captures the last unflushed window (especially the
/// suggestion signal) that the 30s timer would otherwise lose. A true instant
/// foreground kill is unrecoverable in any framework; this covers the rest.
pub fn spawn_visibility_flusher(
    buffer: UsageBuffer,
    client: Signal<ZwipeClient>,
    session: Signal<Option<Session>>,
) {
    spawn(async move {
        let mut eval = document::eval(
            "document.addEventListener('visibilitychange', () => { \
                 if (document.visibilityState === 'hidden') dioxus.send(true); \
             }); \
             window.addEventListener('pagehide', () => dioxus.send(true));",
        );
        // Each hide event sends a message; flush on every one until the eval
        // channel closes (app teardown).
        while eval.recv::<bool>().await.is_ok() {
            flush_once(&buffer, &client, &session).await;
        }
    });
}

/// One-shot flush — useful on screen exit / route changes.
pub async fn flush_once(
    buffer: &UsageBuffer,
    client: &dioxus::prelude::Signal<ZwipeClient>,
    session: &dioxus::prelude::Signal<Option<Session>>,
) {
    let Some(batch) = buffer.snapshot_and_zero() else {
        return;
    };
    let Ok(current_session) = session.ensure_fresh(*client).await else {
        // Not logged in / token couldn't refresh — drop the batch.
        // Metrics are user-scoped vanity data; not worth retry plumbing.
        return;
    };
    let client = client.peek().clone();
    if let Err(e) = client.record_usage(&batch, &current_session).await {
        tracing::debug!(error = ?e, "usage flush failed; dropping batch");
    }
}

//! Background flusher that posts batched usage to the backend.

use std::time::Duration;

use dioxus::prelude::{ReadableExt, spawn};
use tokio::time::interval;

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
    client: dioxus::prelude::Signal<ZwipeClient>,
    session: dioxus::prelude::Signal<Option<Session>>,
) {
    spawn(async move {
        let mut tick = interval(Duration::from_secs(FLUSH_INTERVAL_SECS));
        loop {
            tick.tick().await;
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
    let Some(current_session) = session.peek().clone() else {
        // Not logged in — drop the batch. Metrics are user-scoped.
        return;
    };
    let client = client.peek().clone();
    if let Err(e) = client.record_usage(&batch, &current_session).await {
        tracing::debug!(error = ?e, "usage flush failed; dropping batch");
    }
}

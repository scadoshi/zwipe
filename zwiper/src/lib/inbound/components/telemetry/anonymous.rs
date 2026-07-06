//! Pre-auth funnel events (app opened, register viewed/submitted).
//!
//! Each app launch gets one random session UUID, held in memory only — it
//! identifies a funnel attempt, not a person or an install. Posts are
//! fire-and-forget: a failed send is logged and dropped, never surfaced.

use crate::outbound::client::{
    ZwipeClient, metrics::record_anonymous_event::ClientRecordAnonymousEvent,
};
use dioxus::prelude::*;
use std::sync::LazyLock;
use uuid::Uuid;
use zwipe_core::http::contracts::metrics::{AnonymousEventKind, HttpAnonymousEvent};

/// Random session id for this launch. Process-global and never persisted:
/// a relaunch is a fresh funnel attempt.
static SESSION_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

/// Posts one pre-auth funnel event in the background.
///
/// Must be called from component scope (uses the Dioxus runtime to spawn).
pub fn record_anonymous_event(client: Signal<ZwipeClient>, kind: AnonymousEventKind) {
    let event = HttpAnonymousEvent {
        session_id: *SESSION_ID,
        kind,
    };
    let http = client.peek().clone();
    spawn(async move {
        if let Err(e) = http.record_anonymous_event(&event).await {
            tracing::debug!("anonymous event {} failed: {e}", event.kind.as_str());
        }
    });
}

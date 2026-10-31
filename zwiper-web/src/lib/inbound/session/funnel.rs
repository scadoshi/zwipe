//! The anonymous funnel events the app sends before anyone logs in: opened
//! logged out, Register viewed, Register submitted.
//!
//! Each page load gets one random id, held in memory only, so the server can
//! count distinct attempts without knowing a person or a browser. A post is
//! fire and forget: a failure is logged at debug and dropped.

use std::sync::LazyLock;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::http::contracts::metrics::{AnonymousEventKind, HttpAnonymousEvent};

/// This page load's funnel id. A reload is a new attempt.
static FUNNEL_ID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

/// Posts one funnel event in the background. Call it from a component.
pub fn record(client: Signal<ZwipeClient>, kind: AnonymousEventKind) {
    let event = HttpAnonymousEvent {
        session_id: *FUNNEL_ID,
        kind,
    };
    let http = client.peek().clone();
    spawn(async move {
        if let Err(e) = http.record_anonymous_event(&event).await {
            tracing::debug!("anonymous event {} failed: {e}", event.kind.as_str());
        }
    });
}

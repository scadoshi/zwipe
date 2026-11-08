//! Usage telemetry, the same events the app sends: swipes, searches,
//! suggestion and Swipe select signals, removals, pile ends and handled
//! errors in one batch, plus the crash report a panic leaves for the next
//! load.
//!
//! The buffer is zwipe-ui's [`SharedUsageBuffer`], tagged `web` with this
//! build's version. It goes out every 30 seconds, when the tab goes to the
//! background, and before a search or a Clear skips, and is emptied unsent
//! when the session ends. A failed send drops the batch.

use std::time::Duration;

use chrono::Utc;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::metrics::HttpCrashReport;
use zwipe_ui::telemetry::vocabulary::Screen;
use zwipe_ui::telemetry::{Failure, SharedUsageBuffer};

use crate::config::APP_VERSION;
use crate::inbound::session::ensure_fresh::ensure_fresh;
use crate::outbound::storage::{BrowserStorage, Storage};
use crate::outbound::timer;

/// Time between flushes while the page is open.
const FLUSH_INTERVAL: Duration = Duration::from_secs(30);

/// The `localStorage` key a panic leaves its crash report under.
const CRASH_KEY: &str = "zwipe.crash";

/// Sends `true` each time the tab goes to the background.
const HIDDEN_JS: &str = "document.addEventListener('visibilitychange', () => { if (document.visibilityState === 'hidden') dioxus.send(true); });";

/// The screen a screen's handled errors are reported under. Each screen
/// provides it once with [`use_telemetry_screen`]; [`use_authed`](crate::inbound::chrome::authed::use_authed)
/// and the shared components under it read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelemetryScreen(pub Screen);

/// Names the screen this component and everything under it reports
/// errors under.
pub fn use_telemetry_screen(screen: Screen) {
    use_context_provider(|| TelemetryScreen(screen));
}

/// The reporting screen's stored name, or `""` outside any screen.
pub fn screen_name() -> &'static str {
    try_consume_context::<TelemetryScreen>().map_or("", |s| s.0.as_str())
}

/// The shared usage buffer, in a `Copy` handle closures and spawned tasks
/// capture freely.
pub fn use_usage() -> Signal<SharedUsageBuffer> {
    let buffer: SharedUsageBuffer = use_context();
    use_signal(move || buffer)
}

/// The report a failed request makes.
pub fn failure(error: &ClientError) -> Failure {
    match error {
        ClientError::Network(_) => Failure::Network(error.to_string()),
        ClientError::Decode(detail) => Failure::Decode(detail.to_string()),
        api => Failure::Status(api.status().unwrap_or(500), api.to_string()),
    }
}

/// Sends everything buffered, when there is anything and a session to send
/// it under. Without one, or on a failed send, the batch is dropped.
pub async fn flush_once(
    usage: Signal<SharedUsageBuffer>,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
) {
    let batch = usage.peek().snapshot_and_zero();
    let Some(batch) = batch else {
        return;
    };
    let Ok(current) = ensure_fresh(session, client).await else {
        return;
    };
    let http = client.peek().clone();
    if let Err(e) = http.record_usage(&batch, &current).await {
        tracing::debug!(error = ?e, "usage flush failed; dropping batch");
    }
}

/// Provides the buffer, starts the flushes, empties it when the session
/// ends, and sends a crash report the last load left. Call once, in the
/// root, after the session context.
pub fn use_telemetry(session: Signal<Option<Session>>, client: Signal<ZwipeClient>) {
    let buffer = use_context_provider(|| SharedUsageBuffer::new(ClientPlatform::Web, APP_VERSION));
    let usage = use_signal(move || buffer);

    use_effect(move || {
        if session.read().is_none() {
            usage.peek().clear();
        }
    });

    use_future(move || async move {
        loop {
            timer::sleep(FLUSH_INTERVAL).await;
            flush_once(usage, session, client).await;
        }
    });

    use_future(move || async move {
        let mut hidden = document::eval(HIDDEN_JS);
        while hidden.recv::<bool>().await.is_ok() {
            flush_once(usage, session, client).await;
        }
    });

    use_hook(move || {
        let Some(report) = take_crash() else {
            return;
        };
        let http = client.peek().clone();
        spawn(async move {
            match http.record_crash(&report).await {
                Ok(()) => {
                    if let Err(e) = BrowserStorage.remove(CRASH_KEY) {
                        tracing::warn!("crash report sent but not cleared: {e}");
                    }
                    tracing::info!("crash report {} delivered", report.crash_id);
                }
                Err(e) => tracing::warn!("crash report send failed (will retry): {e}"),
            }
        });
    });
}

/// The crash report the last load left, if any.
fn take_crash() -> Option<HttpCrashReport> {
    let stored = BrowserStorage.read(CRASH_KEY)?;
    serde_json::from_str(&stored)
        .inspect_err(|e| tracing::warn!("stored crash report unreadable: {e}"))
        .ok()
}

/// Installs the panic hook, chained before the previous one: the first
/// panic of the page load writes its crash report to `localStorage` for the
/// next load to send. Call once, before launch.
pub fn install_panic_hook() {
    use std::sync::atomic::{AtomicBool, Ordering};
    static REPORTED: AtomicBool = AtomicBool::new(false);
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if !REPORTED.swap(true, Ordering::Relaxed) {
            let report = HttpCrashReport {
                crash_id: Uuid::new_v4(),
                client_version: APP_VERSION.to_owned(),
                platform: ClientPlatform::Web,
                message: format!("{info}"),
                occurred_at: Utc::now(),
            }
            .clamped();
            if let Ok(json) = serde_json::to_string(&report) {
                let _ = BrowserStorage.write(CRASH_KEY, &json);
            }
        }
        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_failure_keeps_its_kind() {
        assert!(matches!(
            failure(&ClientError::NotFound("gone".to_owned())),
            Failure::Status(404, m) if m == "gone"
        ));
        let decode = serde_json::from_str::<u32>("\"x\"").unwrap_err();
        assert!(matches!(
            failure(&ClientError::from(decode)),
            Failure::Decode(_)
        ));
    }
}

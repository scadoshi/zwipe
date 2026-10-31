//! Crash capture, after the app's: a panic hook writes one report to `crash.json` in the config folder, and the next launch posts it and deletes the file only once the server took it.
//!
//! One file, last crash wins across launches, so a crash loop overwrites rather than piles up. Within a process the first panic wins: a second panic during the unwind would replace the real cause. The `crash_id` is stamped at panic time, so however many launches retry the send, the server stores one row.

use std::fs;
use std::panic::{self, PanicHookInfo};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::platform::ClientPlatform;
use zwipe_core::http::contracts::metrics::HttpCrashReport;

use super::files;

/// The crash file's name in the config folder.
pub(crate) const CRASH_FILE_NAME: &str = "crash.json";

/// The file a crash report waits in until the server takes it.
#[derive(Debug, Clone)]
pub(crate) struct CrashStore {
    path: PathBuf,
}

impl CrashStore {
    /// `crash.json` in the app's config folder.
    ///
    /// # Errors
    ///
    /// Fails when the system has no config directory.
    pub(crate) fn in_config_dir() -> anyhow::Result<Self> {
        Ok(Self::at(files::app_dir()?.join(CRASH_FILE_NAME)))
    }

    /// A store at `path`.
    pub(crate) fn at(path: PathBuf) -> Self {
        Self { path }
    }

    /// Writes `report`, replacing any report already waiting. Best effort: a panic hook has nowhere to report its own failure.
    fn save(&self, report: &HttpCrashReport) {
        if let Ok(json) = serde_json::to_string(report) {
            files::write_private(&self.path, &json).ok();
        }
    }

    /// The report a previous run left, if there is one and it reads.
    pub(crate) fn pending(&self) -> Option<HttpCrashReport> {
        let bytes = fs::read(&self.path).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// Posts the waiting report, unauthenticated so it works before sign-in, and deletes the file only when the server took it. A failed send keeps the file for the next launch.
    pub(crate) async fn send_pending(&self, client: &ZwipeClient) {
        let Some(report) = self.pending() else {
            return;
        };
        if client.record_crash(&report).await.is_ok() {
            files::remove(&self.path).ok();
        }
    }
}

/// What the panic hook does beyond the hook it wraps: saves the first panic's report and ignores the rest.
#[derive(Debug)]
struct CrashHook {
    store: CrashStore,
    fired: AtomicBool,
}

impl CrashHook {
    fn new(store: CrashStore) -> Self {
        Self {
            store,
            fired: AtomicBool::new(false),
        }
    }

    /// Saves a report of `message` on the first call only.
    fn capture(&self, message: String) {
        if self.fired.swap(true, Ordering::Relaxed) {
            return;
        }
        let report = HttpCrashReport {
            crash_id: Uuid::new_v4(),
            client_version: env!("CARGO_PKG_VERSION").to_owned(),
            platform: ClientPlatform::Terminal,
            message,
            occurred_at: Utc::now(),
        }
        .clamped();
        self.store.save(&report);
    }
}

/// Installs a panic hook that saves the crash report, then runs the hook it replaced. Install it before the terminal starts: ratatui then wraps this hook with its own, so the terminal is restored first.
pub(crate) fn install_panic_hook(store: CrashStore) {
    let hook = CrashHook::new(store);
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info: &PanicHookInfo<'_>| {
        // `{info}` carries the panic message and where it happened.
        hook.capture(format!("{info}"));
        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::server::serve_once;

    fn temp_store(name: &str) -> CrashStore {
        CrashStore::at(std::env::temp_dir().join(format!(
            "zwiper-tui-crash-{name}-{}.json",
            std::process::id()
        )))
    }

    #[test]
    fn a_report_round_trips_and_stays_until_removed() {
        let store = temp_store("round-trip");
        files::remove(&store.path).unwrap();
        assert!(store.pending().is_none(), "clean slate");

        let hook = CrashHook::new(store.clone());
        hook.capture("panicked at src/lib/foo.rs:42:9:\nboom".to_owned());
        let first = store.pending().expect("a report waits");
        let again = store.pending().expect("still there until sent");
        assert_eq!(first.crash_id, again.crash_id, "retries keep one crash_id");
        assert_eq!(first.platform, ClientPlatform::Terminal);
        assert_eq!(first.client_version, env!("CARGO_PKG_VERSION"));
        assert!(first.message.contains("foo.rs:42:9"), "{}", first.message);

        files::remove(&store.path).unwrap();
        assert!(store.pending().is_none(), "removed exactly once");
        files::remove(&store.path).unwrap();
    }

    #[test]
    fn the_hook_writes_the_first_panic_only() {
        let store = temp_store("once");
        let hook = CrashHook::new(store.clone());
        hook.capture("first".to_owned());
        hook.capture("second".to_owned());
        let report = store.pending().expect("a report waits");
        files::remove(&store.path).unwrap();
        assert_eq!(report.message, "first");
    }

    #[tokio::test]
    async fn an_accepted_send_deletes_the_file_once() {
        let store = temp_store("sent");
        CrashHook::new(store.clone()).capture("boom".to_owned());
        let (url, request) = serve_once("204 No Content", "").await;
        let client = ZwipeClient::new(url);
        store.send_pending(&client).await;
        let request = request.await.unwrap();
        assert!(request.contains(r#""platform":"terminal""#), "{request}");
        assert!(
            store.pending().is_none(),
            "deleted after the server took it"
        );
        // A second launch has nothing to send and makes no request.
        store.send_pending(&client).await;
    }

    #[tokio::test]
    async fn a_failed_send_keeps_the_file() {
        let store = temp_store("failed-send");
        CrashHook::new(store.clone()).capture("boom".to_owned());
        // Nothing listens on port 9 here, so the post fails.
        let client = ZwipeClient::new("http://127.0.0.1:9".parse().unwrap());
        store.send_pending(&client).await;
        let kept = store.pending();
        files::remove(&store.path).unwrap();
        assert!(kept.is_some(), "kept for the next launch");
    }
}

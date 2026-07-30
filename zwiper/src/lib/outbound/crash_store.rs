//! Crash capture: a panic hook writes the report to disk; the next launch
//! posts it and clears the file only on a 2xx.
//!
//! Native-only (iOS/Android/desktop) — the web preview gets no-ops: a disk
//! write isn't a thing there and browser crashes are a different animal. One
//! file, last-crash-wins: a crash loop overwrites rather than accumulates.
//! The `crash_id` is stamped at panic time, so however many launches retry
//! the send, the server stores exactly one row per crash.

use zwipe_core::http::contracts::metrics::HttpCrashReport;

/// Installs the panic hook (chaining the previous one). The hook does exactly
/// one thing beyond the default: serialize an [`HttpCrashReport`] to the
/// crash file. No network, no allocation-heavy work.
#[cfg(not(target_arch = "wasm32"))]
pub fn install_panic_hook() {
    use chrono::Utc;
    use uuid::Uuid;
    use zwipe_core::domain::auth::models::platform::ClientPlatform;

    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let report = HttpCrashReport {
            crash_id: Uuid::new_v4(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
            platform: ClientPlatform::CURRENT,
            // `{info}` carries the panic payload + source location.
            message: format!("{info}"),
            occurred_at: Utc::now(),
        }
        .clamped();
        write_report(&report);
        previous(info);
    }));
}

/// Serializes the report to the crash file (best effort — a panic hook has
/// nowhere to report its own failures).
#[cfg(not(target_arch = "wasm32"))]
fn write_report(report: &HttpCrashReport) {
    if let (Some(path), Ok(bytes)) = (platform::crash_file(), serde_json::to_vec(report)) {
        let _ = std::fs::write(path, bytes);
    }
}

/// Reads the pending crash report, if a previous run left one.
#[cfg(not(target_arch = "wasm32"))]
pub fn take_pending() -> Option<HttpCrashReport> {
    let path = platform::crash_file()?;
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Deletes the crash file. Call ONLY after the server acknowledged the report
/// (2xx) — an unsent report must survive for the next launch's retry.
#[cfg(not(target_arch = "wasm32"))]
pub fn clear() {
    if let Some(path) = platform::crash_file() {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::{HttpCrashReport, clear, take_pending, write_report};
    use uuid::Uuid;
    use zwipe_core::domain::auth::models::platform::ClientPlatform;

    /// One test owns the whole lifecycle — the store is a single shared file
    /// (desktop: temp dir), so split tests would race under the parallel
    /// harness.
    #[test]
    fn crash_file_round_trip_and_exactly_once_semantics() {
        clear();
        assert!(take_pending().is_none(), "clean slate");

        let crash_id = Uuid::new_v4();
        let report = HttpCrashReport {
            crash_id,
            client_version: "1.8.0".to_string(),
            platform: ClientPlatform::Desktop,
            message: "panicked at 'boom', src/lib/foo.rs:42:9".to_string(),
            occurred_at: chrono::Utc::now(),
        };
        write_report(&report);

        // A failed send leaves the file: reading it again yields the SAME
        // crash_id — the server-side dedupe key that makes retries safe.
        let first = take_pending().expect("pending crash found");
        let second = take_pending().expect("still pending until cleared");
        assert_eq!(first.crash_id, crash_id);
        assert_eq!(second.crash_id, crash_id);
        assert_eq!(first.message, report.message);

        // Last-crash-wins: a newer crash overwrites, never accumulates.
        let newer = HttpCrashReport {
            crash_id: Uuid::new_v4(),
            ..report
        };
        write_report(&newer);
        assert_eq!(take_pending().unwrap().crash_id, newer.crash_id);

        // 2xx path: clear deletes; nothing survives for the next launch.
        clear();
        assert!(take_pending().is_none(), "cleared after acknowledged send");
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    use std::path::PathBuf;

    const CRASH_FILE: &str = "zwiper-crash.json";

    /// Android: the app's private files dir (same home as session/theme).
    #[cfg(target_os = "android")]
    pub fn crash_file() -> Option<PathBuf> {
        Some(
            crate::outbound::android_fs::files_dir()
                .ok()?
                .join(CRASH_FILE),
        )
    }

    /// iOS: the sandbox's Documents dir — always writable, survives restarts.
    #[cfg(target_os = "ios")]
    pub fn crash_file() -> Option<PathBuf> {
        Some(
            PathBuf::from(std::env::var_os("HOME")?)
                .join("Documents")
                .join(CRASH_FILE),
        )
    }

    /// Desktop (dev builds): the temp dir is plenty — no user-facing value in
    /// persisting dev crashes across reboots.
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub fn crash_file() -> Option<PathBuf> {
        Some(std::env::temp_dir().join(CRASH_FILE))
    }
}

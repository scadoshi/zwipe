//! Local cache of the public counters (decks created, searches run, cards
//! swiped) that the home and auth screens show.
//!
//! Keeps the last answer zerver gave so a launch with no signal shows the
//! numbers it had rather than rolling digits that never land. Same backends
//! as [`theme_store`]: the keyring on Apple and desktop, a JSON file in
//! Android private storage. Never deleted on logout; the counters are public.
//!
//! [`theme_store`]: crate::outbound::theme_store

use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

/// Persisting the last public counters across application restarts.
pub trait PersistCounts {
    /// Saves the counters, logging errors instead of returning them.
    fn infallible_save(&self);
    /// Loads the cached counters, returning `None` if not found.
    fn load() -> anyhow::Result<Option<HttpPublicMetrics>>;
    /// Loads the cached counters, logging errors and returning `None` on failure.
    fn infallible_load() -> Option<HttpPublicMetrics>;
}

impl PersistCounts for HttpPublicMetrics {
    fn infallible_save(&self) {
        if let Err(e) = platform::save(self) {
            tracing::error!("failed to save public counters: {e}");
        }
    }

    fn load() -> anyhow::Result<Option<HttpPublicMetrics>> {
        platform::load()
    }

    fn infallible_load() -> Option<HttpPublicMetrics> {
        match Self::load() {
            Ok(counts) => counts,
            Err(e) => {
                tracing::error!("failed to load public counters: {e}");
                None
            }
        }
    }
}

// Apple / desktop: OS store via keyring (same service as the session).
#[cfg(not(target_os = "android"))]
mod platform {
    use super::HttpPublicMetrics;
    use crate::outbound::keyring_entry::{self, Entry};

    fn service() -> String {
        env!("CARGO_PKG_NAME").to_string() + "-service"
    }

    fn username() -> String {
        env!("CARGO_PKG_NAME").to_string() + "-counts"
    }

    fn entry() -> Result<Entry, keyring::Error> {
        keyring_entry::entry(&service(), &username())
    }

    pub fn save(counts: &HttpPublicMetrics) -> anyhow::Result<()> {
        entry()?.set_secret(&serde_json::to_vec(counts)?)?;
        Ok(())
    }

    pub fn load() -> anyhow::Result<Option<HttpPublicMetrics>> {
        match entry()?.get_secret() {
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        }
    }
}

// Android: a JSON file in the app's private internal storage.
#[cfg(target_os = "android")]
mod platform {
    use super::HttpPublicMetrics;
    use crate::outbound::android_fs::files_dir;
    use std::path::PathBuf;

    fn counts_path() -> anyhow::Result<PathBuf> {
        Ok(files_dir()?.join("counts.json"))
    }

    pub fn save(counts: &HttpPublicMetrics) -> anyhow::Result<()> {
        std::fs::write(counts_path()?, serde_json::to_vec(counts)?)?;
        Ok(())
    }

    pub fn load() -> anyhow::Result<Option<HttpPublicMetrics>> {
        match std::fs::read(counts_path()?) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        }
    }
}

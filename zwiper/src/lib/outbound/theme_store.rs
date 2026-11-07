//! Local theme persistence.
//!
//! Caches the last-used [`ThemeConfig`](zwipe_core::domain::user::models::theme::ThemeConfig) so pre-auth screens (login, register,
//! verify, reset) render in it before a session loads, instead of the default.
//! The theme can drift (changed on another device), so this is a best-effort
//! "last known" that a live session's settings override.
//!
//! The theme isn't a secret, but this reuses the same backends as [`session`]
//! (keyring on Apple/desktop, a JSON file in Android private storage) to avoid
//! new platform code. Unlike the session, it is never deleted on logout; the
//! whole point is that the login screen keeps the last theme.
//!
//! Beside it sits the server's theme as this device last saw it, which
//! [`zwipe_ui::theme_sync::reconcile`] weighs against a fetched copy so the
//! latest choice wins.
//!
//! [`session`]: crate::outbound::session

use components::ThemeConfig;

/// Persisting the last-used theme across application restarts.
pub trait PersistTheme {
    /// Saves the theme, logging errors instead of returning them.
    fn infallible_save(&self);
    /// Loads the cached theme, returning `None` if not found or if it names no
    /// theme this build has. A retired theme loads as its replacement.
    ///
    /// # Errors
    ///
    /// Fails when the store cannot be read or the copy does not parse.
    fn load() -> anyhow::Result<Option<ThemeConfig>>;
    /// Loads the cached theme, logging errors and returning `None` on failure.
    fn infallible_load() -> Option<ThemeConfig>;
}

/// Which of the two stored themes.
#[derive(Clone, Copy)]
enum Slot {
    /// The theme the app last showed.
    Shown,
    /// The server's theme as this device last saw it.
    Server,
}

impl PersistTheme for ThemeConfig {
    fn infallible_save(&self) {
        if let Err(e) = platform::save(Slot::Shown, self) {
            tracing::error!("failed to save theme: {e}");
        }
    }

    fn load() -> anyhow::Result<Option<ThemeConfig>> {
        Ok(platform::load(Slot::Shown)?.and_then(ThemeConfig::resolve))
    }

    fn infallible_load() -> Option<ThemeConfig> {
        match Self::load() {
            Ok(theme) => theme,
            Err(e) => {
                tracing::error!("failed to load theme: {e}");
                None
            }
        }
    }
}

/// Records `theme` as the server's copy, as this device saw it last, and
/// logs a failure.
pub fn save_server_seen(theme: &ThemeConfig) {
    if let Err(e) = platform::save(Slot::Server, theme) {
        tracing::error!("failed to save the server's theme: {e}");
    }
}

/// The server's theme as this device last saw it, resolved as
/// [`PersistTheme::load`] resolves the theme. `None` when none was recorded
/// or it cannot be read.
pub fn load_server_seen() -> Option<ThemeConfig> {
    match platform::load(Slot::Server) {
        Ok(theme) => theme.and_then(ThemeConfig::resolve),
        Err(e) => {
            tracing::error!("failed to load the server's theme: {e}");
            None
        }
    }
}

// Apple / desktop: OS store via keyring (same service as the session).
#[cfg(not(target_os = "android"))]
mod platform {
    use super::{Slot, ThemeConfig};
    use crate::outbound::keyring_entry::{self, Entry};

    const SERVICE: &str = concat!(env!("CARGO_PKG_NAME"), "-service");

    fn entry(slot: Slot) -> Result<Entry, keyring::Error> {
        let username = match slot {
            Slot::Shown => concat!(env!("CARGO_PKG_NAME"), "-theme"),
            Slot::Server => concat!(env!("CARGO_PKG_NAME"), "-theme-server"),
        };
        keyring_entry::entry(SERVICE, username)
    }

    pub(crate) fn save(slot: Slot, theme: &ThemeConfig) -> anyhow::Result<()> {
        entry(slot)?.set_secret(&serde_json::to_vec(theme)?)?;
        Ok(())
    }

    pub(crate) fn load(slot: Slot) -> anyhow::Result<Option<ThemeConfig>> {
        match entry(slot)?.get_secret() {
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        }
    }
}

// Android: a JSON file in the app's private internal storage.
#[cfg(target_os = "android")]
mod platform {
    use std::path::PathBuf;

    use super::{Slot, ThemeConfig};
    use crate::outbound::android_fs;

    fn theme_path(slot: Slot) -> anyhow::Result<PathBuf> {
        let file = match slot {
            Slot::Shown => "theme.json",
            Slot::Server => "theme_server.json",
        };
        Ok(android_fs::files_dir()?.join(file))
    }

    pub(crate) fn save(slot: Slot, theme: &ThemeConfig) -> anyhow::Result<()> {
        std::fs::write(theme_path(slot)?, serde_json::to_vec(theme)?)?;
        Ok(())
    }

    pub(crate) fn load(slot: Slot) -> anyhow::Result<Option<ThemeConfig>> {
        match std::fs::read(theme_path(slot)?) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        }
    }
}

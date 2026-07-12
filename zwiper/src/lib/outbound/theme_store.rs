//! Local theme persistence.
//!
//! Caches the last-used [`ThemeConfig`] so pre-auth screens (login, register,
//! verify, reset) render in it before a session loads, instead of the default.
//! The theme can drift (changed on another device), so this is a best-effort
//! "last known" that a live session's preferences override.
//!
//! The theme isn't a secret, but this reuses the same backends as [`session`]
//! (keyring on Apple/desktop, a JSON file in Android private storage) to avoid
//! new platform code. Unlike the session, it is never deleted on logout — the
//! whole point is that the login screen keeps the last theme.
//!
//! [`session`]: crate::outbound::session

use zwipe_core::domain::user::models::theme::ThemeConfig;

/// Persisting the last-used theme across application restarts.
pub trait PersistTheme {
    /// Saves the theme, logging errors instead of returning them.
    fn infallible_save(&self);
    /// Loads the cached theme, returning `None` if not found.
    fn load() -> anyhow::Result<Option<ThemeConfig>>;
    /// Loads the cached theme, logging errors and returning `None` on failure.
    fn infallible_load() -> Option<ThemeConfig>;
}

impl PersistTheme for ThemeConfig {
    fn infallible_save(&self) {
        if let Err(e) = platform::save(self) {
            tracing::error!("failed to save theme: {e}");
        }
    }

    fn load() -> anyhow::Result<Option<ThemeConfig>> {
        platform::load()
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

// ============================================================================
// Apple / desktop: OS store via keyring (same service as the session).
// ============================================================================
#[cfg(not(target_os = "android"))]
mod platform {
    use super::ThemeConfig;
    use keyring::default;

    fn service() -> String {
        env!("CARGO_PKG_NAME").to_string() + "-service"
    }

    fn username() -> String {
        env!("CARGO_PKG_NAME").to_string() + "-theme"
    }

    pub fn save(theme: &ThemeConfig) -> anyhow::Result<()> {
        let credential =
            default::default_credential_builder().build(None, &service(), &username())?;
        credential.set_secret(&serde_json::to_vec(theme)?)?;
        Ok(())
    }

    pub fn load() -> anyhow::Result<Option<ThemeConfig>> {
        let credential =
            default::default_credential_builder().build(None, &service(), &username())?;
        match credential.get_secret() {
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        }
    }
}

// ============================================================================
// Android: a JSON file in the app's private internal storage.
// ============================================================================
#[cfg(target_os = "android")]
mod platform {
    use super::ThemeConfig;
    use crate::outbound::android_fs::files_dir;
    use std::path::PathBuf;

    fn theme_path() -> anyhow::Result<PathBuf> {
        Ok(files_dir()?.join("theme.json"))
    }

    pub fn save(theme: &ThemeConfig) -> anyhow::Result<()> {
        std::fs::write(theme_path()?, serde_json::to_vec(theme)?)?;
        Ok(())
    }

    pub fn load() -> anyhow::Result<Option<ThemeConfig>> {
        match std::fs::read(theme_path()?) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        }
    }
}

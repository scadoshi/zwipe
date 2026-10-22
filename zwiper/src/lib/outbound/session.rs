//! Session persistence.
//!
//! On Apple/desktop platforms the session lives in the OS secure store via the
//! `keyring` crate (Keychain on iOS/macOS, Credential Manager on Windows,
//! Secret Service on Linux).
//!
//! `keyring` has **no Android backend**: there it silently falls back to an
//! in-memory mock store that's lost on app restart. So on Android we persist
//! the session to a file in the app's **private internal storage**
//! (`/data/data/<pkg>/files/session.json`), sealed under an Android Keystore
//! AES key (`outbound/android_keystore.rs`). The path is resolved through JNI
//! from the Android `Context` that `ndk-context` exposes.

use zwipe_core::domain::auth::models::session::Session;

/// Trait for persisting sessions across application restarts.
pub trait Persist {
    /// Saves the session to persistent storage.
    fn save(&self) -> anyhow::Result<()>;
    /// Saves the session, logging errors instead of returning them.
    fn infallible_save(&self);
    /// Loads a session from storage, returning `None` if not found or expired.
    fn load() -> anyhow::Result<Option<Session>>;
    /// Loads a session, logging errors and returning `None` on failure.
    fn infallible_load() -> Option<Session>;
    /// Deletes the session from storage.
    fn delete(&self) -> anyhow::Result<()>;
    /// Deletes the session, logging errors instead of returning them.
    fn infallible_delete(&self);
}

impl Persist for Session {
    fn save(&self) -> anyhow::Result<()> {
        platform::save(self)
    }

    fn infallible_save(&self) {
        if let Err(e) = self.save() {
            tracing::error!("failed to save session: {e}");
        }
    }

    fn load() -> anyhow::Result<Option<Self>> {
        platform::load()
    }

    fn infallible_load() -> Option<Session> {
        match Session::load() {
            Ok(session) => session,
            Err(e) => {
                tracing::error!("failed to load session: {e}");
                None
            }
        }
    }

    fn delete(&self) -> anyhow::Result<()> {
        platform::delete()
    }

    fn infallible_delete(&self) {
        if let Err(e) = self.delete() {
            tracing::error!("failed to delete session: {e}");
        }
    }
}

// Apple / desktop: OS secure store via keyring.
#[cfg(not(target_os = "android"))]
mod platform {
    use super::Session;
    use crate::outbound::keyring_entry::{self, Entry};

    fn service() -> String {
        env!("CARGO_PKG_NAME").to_string() + "-service"
    }

    fn username() -> String {
        env!("CARGO_PKG_NAME").to_string() + "-user"
    }

    /// (service, username) must keep mapping to the Keychain item the
    /// keyring-3 backend wrote, or existing sessions drop on upgrade.
    fn entry() -> Result<Entry, keyring::Error> {
        keyring_entry::entry(&service(), &username())
    }

    pub fn save(session: &Session) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec(session)?;
        entry()?.set_secret(&bytes)?;
        Ok(())
    }

    pub fn load() -> anyhow::Result<Option<Session>> {
        let entry = entry()?;
        match entry.get_secret() {
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => {
                let session: Session = serde_json::from_slice(&bytes)?;
                if session.is_expired() {
                    entry.delete_credential()?;
                    return Ok(None);
                }
                Ok(Some(session))
            }
        }
    }

    pub fn delete() -> anyhow::Result<()> {
        entry()?.delete_credential()?;
        Ok(())
    }
}

// Android: the session JSON sealed under a Keystore key, in a file in the
// app's private internal storage.
#[cfg(target_os = "android")]
mod platform {
    use std::path::PathBuf;

    use super::Session;
    use crate::outbound::android_fs::files_dir;
    use crate::outbound::android_keystore;

    fn session_path() -> anyhow::Result<PathBuf> {
        Ok(files_dir()?.join("session.json"))
    }

    pub fn save(session: &Session) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec(session)?;
        let blob = android_keystore::seal(&bytes)?;
        std::fs::write(session_path()?, blob)?;
        Ok(())
    }

    /// A file that starts with `{` is plaintext JSON from a build before the
    /// Keystore; it is read as is and rewritten sealed. A sealed file that does
    /// not open (the key is gone after a reinstall, or the file is corrupt)
    /// counts as logged out and is removed.
    pub fn load() -> anyhow::Result<Option<Session>> {
        let path = session_path()?;
        let bytes = match std::fs::read(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
            Ok(bytes) => bytes,
        };
        let plaintext = bytes.first() == Some(&b'{');
        let json = if plaintext {
            tracing::info!("session: sealing a plaintext session file");
            bytes
        } else {
            match android_keystore::open(&bytes) {
                Ok(json) => json,
                Err(e) => {
                    tracing::warn!(
                        "session: sealed file does not open, treating as logged out: {e}"
                    );
                    let _ = std::fs::remove_file(&path);
                    return Ok(None);
                }
            }
        };
        let session: Session = match serde_json::from_slice(&json) {
            Ok(session) => session,
            Err(e) => {
                tracing::warn!("session: file does not parse, treating as logged out: {e}");
                let _ = std::fs::remove_file(&path);
                return Ok(None);
            }
        };
        if session.is_expired() {
            let _ = std::fs::remove_file(&path);
            return Ok(None);
        }
        if plaintext {
            save(&session)?;
        }
        Ok(Some(session))
    }

    pub fn delete() -> anyhow::Result<()> {
        match std::fs::remove_file(session_path()?) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => Ok(other?),
        }
    }
}

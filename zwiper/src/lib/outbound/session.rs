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
    ///
    /// # Errors
    ///
    /// Fails when the session cannot be serialized or written.
    fn save(&self) -> anyhow::Result<()>;
    /// Saves the session, logging errors instead of returning them.
    fn infallible_save(&self);
    /// Loads a session from storage, returning `None` if not found or expired.
    ///
    /// # Errors
    ///
    /// Fails when the store cannot be read or the session does not parse.
    fn load() -> anyhow::Result<Option<Session>>;
    /// Loads a session, logging errors and returning `None` on failure.
    fn infallible_load() -> Option<Session>;
    /// Deletes the session from storage.
    ///
    /// # Errors
    ///
    /// Fails when the store cannot remove the session.
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

    const SERVICE: &str = concat!(env!("CARGO_PKG_NAME"), "-service");
    const USERNAME: &str = concat!(env!("CARGO_PKG_NAME"), "-user");

    /// `SERVICE` and `USERNAME` name the Keychain item existing installs hold;
    /// changing either logs every user out.
    fn entry() -> Result<Entry, keyring::Error> {
        keyring_entry::entry(SERVICE, USERNAME)
    }

    pub(crate) fn save(session: &Session) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec(session)?;
        entry()?.set_secret(&bytes)?;
        Ok(())
    }

    pub(crate) fn load() -> anyhow::Result<Option<Session>> {
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

    pub(crate) fn delete() -> anyhow::Result<()> {
        entry()?.delete_credential()?;
        Ok(())
    }
}

/// How Android's session file reads: sealed, legacy plaintext, or neither.
#[cfg(any(target_os = "android", test))]
mod stored {
    /// What a session file holds once read.
    #[derive(Debug, PartialEq, Eq)]
    pub(crate) enum Stored {
        /// A sealed file that opened, with its JSON.
        Sealed(Vec<u8>),
        /// Plaintext JSON from before sealing, to be rewritten sealed.
        Plaintext(Vec<u8>),
        /// Neither: the key is gone after a reinstall, or the file is
        /// corrupt. Why it did not open.
        Unreadable(String),
    }

    /// Reads a session file's `bytes`, opening a sealed one with `open`.
    ///
    /// A sealed file starts with a random nonce, so its first byte is `{` one
    /// time in 256; the bytes are tried as sealed first, and only a file
    /// that does not open and starts with `{` counts as plaintext.
    pub(crate) fn read(
        bytes: Vec<u8>,
        open: impl FnOnce(&[u8]) -> anyhow::Result<Vec<u8>>,
    ) -> Stored {
        match open(&bytes) {
            Ok(json) => Stored::Sealed(json),
            Err(_) if bytes.first() == Some(&b'{') => Stored::Plaintext(bytes),
            Err(e) => Stored::Unreadable(e.to_string()),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// The nonce a fake seal writes, starting with `{` as one real
        /// nonce in 256 does.
        const BRACE_NONCE: &[u8; 12] = b"{nonce-bytes";

        /// Opens what [`seal`] wrote, and nothing else, as the Keystore
        /// would.
        fn open(blob: &[u8]) -> anyhow::Result<Vec<u8>> {
            blob.strip_prefix(BRACE_NONCE.as_slice())
                .map(<[u8]>::to_vec)
                .ok_or_else(|| anyhow::anyhow!("tag mismatch"))
        }

        fn seal(json: &[u8]) -> Vec<u8> {
            [BRACE_NONCE.as_slice(), json].concat()
        }

        const JSON: &[u8] = br#"{"user":"someone"}"#;

        #[test]
        fn a_sealed_file_whose_nonce_starts_with_a_brace_opens() {
            assert_eq!(read(seal(JSON), open), Stored::Sealed(JSON.to_vec()));
        }

        #[test]
        fn plaintext_json_reads_as_plaintext() {
            assert_eq!(read(JSON.to_vec(), open), Stored::Plaintext(JSON.to_vec()));
        }

        #[test]
        fn a_file_that_neither_opens_nor_is_json_is_unreadable() {
            assert_eq!(
                read(b"\x01garbage".to_vec(), open),
                Stored::Unreadable("tag mismatch".to_owned())
            );
        }
    }
}

// Android: the session JSON sealed under a Keystore key, in a file in the
// app's private internal storage.
#[cfg(target_os = "android")]
mod platform {
    use std::path::PathBuf;

    use super::Session;
    use super::stored::{self, Stored};
    use crate::outbound::{android_fs, android_keystore};

    fn session_path() -> anyhow::Result<PathBuf> {
        Ok(android_fs::files_dir()?.join("session.json"))
    }

    pub(crate) fn save(session: &Session) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec(session)?;
        let blob = android_keystore::seal(&bytes)?;
        std::fs::write(session_path()?, blob)?;
        Ok(())
    }

    /// A sealed file opens; plaintext JSON from before sealing is read as is
    /// and rewritten sealed (see [`stored::read`]). A file that is neither
    /// (the key is gone after a reinstall, or the file is corrupt) counts as
    /// logged out and is removed.
    pub(crate) fn load() -> anyhow::Result<Option<Session>> {
        let path = session_path()?;
        let bytes = match std::fs::read(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
            Ok(bytes) => bytes,
        };
        let (json, plaintext) = match stored::read(bytes, android_keystore::open) {
            Stored::Sealed(json) => (json, false),
            Stored::Plaintext(json) => {
                tracing::info!("session: sealing a plaintext session file");
                (json, true)
            }
            Stored::Unreadable(e) => {
                tracing::warn!("session: sealed file does not open, treating as logged out: {e}");
                let _ = std::fs::remove_file(&path);
                return Ok(None);
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

    pub(crate) fn delete() -> anyhow::Result<()> {
        match std::fs::remove_file(session_path()?) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => Ok(other?),
        }
    }
}

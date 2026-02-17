//! Session persistence using the system keyring.
//!
//! Stores user session data securely in the OS keychain/credential manager,
//! allowing sessions to persist across application restarts.

use keyring::default;
use zwipe::domain::auth::models::session::Session;

fn credential_username() -> String {
    env!("CARGO_PKG_NAME").to_string() + "-user"
}

fn credential_service() -> String {
    env!("CARGO_PKG_NAME").to_string() + "-service"
}

/// Trait for persisting sessions to secure OS keychain storage.
///
/// Provides methods for saving, loading, and deleting user sessions from
/// the system's credential manager (Keychain on macOS, Credential Manager
/// on Windows, Secret Service on Linux).
pub trait Persist {
    /// Saves the session to secure storage.
    fn save(&self) -> anyhow::Result<()>;
    /// Saves the session, logging errors instead of returning them.
    fn infallible_save(&self);
    /// Loads a session from secure storage, returning `None` if not found or expired.
    fn load() -> anyhow::Result<Option<Session>>;
    /// Loads a session, logging errors and returning `None` on failure.
    fn infallible_load() -> Option<Session>;
    /// Deletes the session from secure storage.
    fn delete(&self) -> anyhow::Result<()>;
    /// Deletes the session, logging errors instead of returning them.
    fn infallible_delete(&self);
}

impl Persist for Session {
    fn save(&self) -> anyhow::Result<()> {
        let credential = default::default_credential_builder().build(
            None,
            &credential_service(),
            &credential_username(),
        )?;
        let bytes = serde_json::to_vec(self)?;
        credential.set_secret(&bytes)?;
        Ok(())
    }

    fn infallible_save(&self) {
        match self.save() {
            Ok(()) => (),
            Err(e) => tracing::error!("failed to save session: {e}"),
        }
    }

    fn load() -> anyhow::Result<Option<Self>> {
        let credential = default::default_credential_builder().build(
            None,
            &credential_service(),
            &credential_username(),
        )?;
        let result = credential.get_secret();
        match result {
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => {
                let session: Session = serde_json::from_slice(&bytes)?;
                if session.is_expired() {
                    credential.delete_credential()?;
                    return Ok(None);
                }
                Ok(Some(session))
            }
        }
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
        let credential = default::default_credential_builder().build(
            None,
            &credential_service(),
            &credential_username(),
        )?;
        credential.delete_credential()?;
        Ok(())
    }

    fn infallible_delete(&self) {
        match self.delete() {
            Ok(()) => (),
            Err(e) => tracing::error!("failed to delete session: {e}"),
        }
    }
}

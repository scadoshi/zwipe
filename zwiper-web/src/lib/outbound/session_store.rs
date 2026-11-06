//! The session, kept in browser storage between visits.
//!
//! The session holds the refresh token, so anything that can run script on
//! this origin can read it. The app keeps it in the OS keychain; a browser
//! has no equivalent that survives a reload.

use zwipe_core::domain::auth::models::session::Session;

use crate::outbound::storage::{BrowserStorage, Storage, StorageError};

/// The storage key the session lives under.
pub const SESSION_KEY: &str = "zwiper-web.session";

/// Why the session could not be saved or loaded.
#[derive(Debug, thiserror::Error)]
pub enum SessionStoreError {
    /// The store refused the read or write.
    #[error(transparent)]
    Storage(#[from] StorageError),
    /// The session would not serialize.
    #[error("session would not serialize: {0}")]
    Encode(#[source] serde_json::Error),
    /// The stored copy does not parse as a session.
    #[error("stored session does not parse: {0}")]
    Decode(#[source] serde_json::Error),
}

/// Saves, loads and deletes the one session in `S`.
#[derive(Debug, Clone, Default)]
pub struct SessionStore<S> {
    storage: S,
}

impl SessionStore<BrowserStorage> {
    /// The store over the browser's `localStorage`.
    pub fn browser() -> Self {
        Self::new(BrowserStorage)
    }
}

impl<S: Storage> SessionStore<S> {
    /// A store over `storage`.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Writes `session` over any stored one.
    ///
    /// # Errors
    ///
    /// Fails when the session will not serialize or the store refuses it.
    pub fn save(&self, session: &Session) -> Result<(), SessionStoreError> {
        let json = serde_json::to_string(session).map_err(SessionStoreError::Encode)?;
        self.storage.write(SESSION_KEY, &json)?;
        Ok(())
    }

    /// The stored session, or `None` when there is none. A session whose
    /// refresh token has expired is deleted and reads as `None`.
    ///
    /// # Errors
    ///
    /// Fails when the stored copy does not parse, or when an expired one
    /// cannot be removed.
    pub fn load(&self) -> Result<Option<Session>, SessionStoreError> {
        let Some(json) = self.storage.read(SESSION_KEY) else {
            return Ok(None);
        };
        let session: Session = serde_json::from_str(&json).map_err(SessionStoreError::Decode)?;
        if session.is_expired() {
            self.storage.remove(SESSION_KEY)?;
            return Ok(None);
        }
        Ok(Some(session))
    }

    /// Removes the stored session. Deleting when there is none succeeds.
    ///
    /// # Errors
    ///
    /// Fails when the store refuses the removal.
    pub fn delete(&self) -> Result<(), SessionStoreError> {
        self.storage.remove(SESSION_KEY)?;
        Ok(())
    }

    /// [`save`](Self::save), logging a failure instead of returning it.
    pub fn infallible_save(&self, session: &Session) {
        if let Err(e) = self.save(session) {
            tracing::error!("failed to save session: {e}");
        }
    }

    /// [`load`](Self::load), logging a failure and reading it as `None`.
    pub fn infallible_load(&self) -> Option<Session> {
        self.load().unwrap_or_else(|e| {
            tracing::error!("failed to load session: {e}");
            None
        })
    }

    /// [`delete`](Self::delete), logging a failure instead of returning it.
    pub fn infallible_delete(&self) {
        if let Err(e) = self.delete() {
            tracing::error!("failed to delete session: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::*;
    use crate::outbound::storage::MemoryStorage;
    use crate::test_fixtures;

    fn store() -> SessionStore<MemoryStorage> {
        SessionStore::new(MemoryStorage::default())
    }

    #[test]
    fn a_saved_session_loads_back_equal() {
        let store = store();
        let session = test_fixtures::session();
        store.save(&session).unwrap();
        assert_eq!(store.load().unwrap(), Some(session));
    }

    #[test]
    fn an_empty_store_loads_none() {
        assert_eq!(store().load().unwrap(), None);
    }

    #[test]
    fn a_save_replaces_the_last_session() {
        let store = store();
        store.save(&test_fixtures::session()).unwrap();
        let rotated = test_fixtures::session();
        store.save(&rotated).unwrap();
        assert_eq!(store.load().unwrap(), Some(rotated));
    }

    #[test]
    fn delete_leaves_nothing_and_can_repeat() {
        let store = store();
        store.save(&test_fixtures::session()).unwrap();
        store.delete().unwrap();
        store.delete().unwrap();
        assert_eq!(store.load().unwrap(), None);
    }

    #[test]
    fn an_expired_session_loads_none_and_is_removed() {
        let store = store();
        let mut session = test_fixtures::session();
        session.refresh_token.expires_at = Utc::now() - Duration::seconds(1);
        store.save(&session).unwrap();
        assert_eq!(store.load().unwrap(), None);
        assert_eq!(store.storage.read(SESSION_KEY), None);
    }

    #[test]
    fn a_corrupt_copy_is_an_error_and_reads_as_none_quietly() {
        let store = store();
        store.storage.write(SESSION_KEY, "{not json").unwrap();
        assert!(matches!(store.load(), Err(SessionStoreError::Decode(_))));
        assert_eq!(store.infallible_load(), None);
    }

    #[test]
    fn the_browser_store_off_wasm_holds_nothing() {
        let store = SessionStore::browser();
        assert!(store.save(&test_fixtures::session()).is_err());
        assert_eq!(store.load().unwrap(), None);
    }
}

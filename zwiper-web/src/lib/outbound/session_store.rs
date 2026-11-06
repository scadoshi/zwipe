//! The session, kept in browser storage between visits.
//!
//! Anything that can run script on this origin can read `localStorage`, so
//! no token is ever written here. The refresh token lives in an `HttpOnly`
//! cookie the server sets and the page cannot read; the access token lives
//! in memory and is refreshed through that cookie after a reload.

use std::str::FromStr;

use chrono::DateTime;
use zwipe_core::domain::auth::models::access_token::Jwt;
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
    /// The placeholder access token would not build.
    #[error("placeholder access token would not build")]
    Placeholder,
}

/// A JWT-shaped stand-in for the access token in storage. It is no token:
/// no server accepts it, and its expiry makes the session refresh first.
const PLACEHOLDER_JWT: &str = "stored.without.token";

/// `session` as it may be written to storage: its access token replaced by
/// an expired placeholder and its refresh token value blanked.
fn without_tokens(session: &Session) -> Result<Session, SessionStoreError> {
    let mut stored = session.clone();
    stored.access_token.value =
        Jwt::from_str(PLACEHOLDER_JWT).map_err(|_| SessionStoreError::Placeholder)?;
    stored.access_token.expires_at = DateTime::UNIX_EPOCH;
    stored.refresh_token.value = String::new();
    Ok(stored)
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

    /// Writes `session` over any stored one, without its tokens: page scripts
    /// can read `localStorage`, so it keeps only the user, the settings and
    /// the refresh token's expiry. The access token is a placeholder marked
    /// expired, so a loaded session refreshes through the `HttpOnly` cookie
    /// before its first call, and the refresh token is blank.
    ///
    /// # Errors
    ///
    /// Fails when the session will not serialize or the store refuses it.
    pub fn save(&self, session: &Session) -> Result<(), SessionStoreError> {
        let json =
            serde_json::to_string(&without_tokens(session)?).map_err(SessionStoreError::Encode)?;
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
    fn a_saved_session_loads_back_without_its_tokens() {
        let store = store();
        let session = test_fixtures::session();
        store.save(&session).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded.user, session.user);
        assert_eq!(loaded.settings, session.settings);
        assert_eq!(
            loaded.refresh_token.expires_at,
            session.refresh_token.expires_at
        );
        assert_eq!(loaded.refresh_token.value, "");
        assert_eq!(loaded.access_token.value.as_str(), PLACEHOLDER_JWT);
        assert_eq!(loaded.access_token.expires_at, DateTime::UNIX_EPOCH);
    }

    #[test]
    fn no_token_reaches_storage() {
        let store = store();
        let session = test_fixtures::session();
        store.save(&session).unwrap();
        let raw = store.storage.read(SESSION_KEY).unwrap();
        assert!(
            !raw.contains(session.access_token.value.as_str()),
            "access token stored"
        );
        assert!(
            !raw.contains(&session.refresh_token.value),
            "refresh token stored"
        );
    }

    #[test]
    fn a_loaded_session_refreshes_before_its_first_call() {
        let store = store();
        store.save(&test_fixtures::session()).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(
            zwipe_ui::session::freshness(&loaded, Utc::now()),
            zwipe_ui::session::Freshness::Stale
        );
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
        assert_eq!(
            store.load().unwrap(),
            Some(without_tokens(&rotated).unwrap())
        );
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

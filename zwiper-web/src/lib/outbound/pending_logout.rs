//! A logout the server has not heard yet.
//!
//! The refresh token lives in an `HttpOnly` cookie the page cannot delete;
//! only the server's answer to a logout expires it. When logout cannot reach
//! the server, this mark stays in storage so the next start, or the browser
//! coming back online, tells it then. A new login replaces the cookie and
//! clears the mark.

use crate::outbound::storage::{BrowserStorage, Storage};

/// The storage key the mark lives under.
pub const PENDING_LOGOUT_KEY: &str = "zwiper-web.logout-pending";

/// Whether a logout still has to reach the server.
#[derive(Debug, Clone, Default)]
pub struct PendingLogout<S> {
    storage: S,
}

impl PendingLogout<BrowserStorage> {
    /// The mark in the browser's `localStorage`.
    pub fn browser() -> Self {
        Self::new(BrowserStorage)
    }
}

impl<S: Storage> PendingLogout<S> {
    /// The mark in `storage`.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Whether a logout is waiting on the server.
    pub fn is_marked(&self) -> bool {
        self.storage.read(PENDING_LOGOUT_KEY).is_some()
    }

    /// Records that a logout is waiting on the server. A store that refuses
    /// the write is logged: the logout has already happened here.
    pub fn mark(&self) {
        if let Err(e) = self.storage.write(PENDING_LOGOUT_KEY, "1") {
            tracing::warn!("could not record the pending logout: {e}");
        }
    }

    /// Drops the mark once the server has heard, or a new login replaced the
    /// cookie.
    pub fn clear(&self) {
        if let Err(e) = self.storage.remove(PENDING_LOGOUT_KEY) {
            tracing::warn!("could not clear the pending logout: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outbound::storage::MemoryStorage;

    #[test]
    fn the_mark_stays_until_cleared() {
        let pending = PendingLogout::new(MemoryStorage::default());
        assert!(!pending.is_marked());
        pending.mark();
        assert!(pending.is_marked());
        pending.clear();
        assert!(!pending.is_marked());
    }
}

//! Whether the card's details panel beside a swipe card is shown, kept in
//! browser storage so the eye's last choice holds across visits.

use crate::outbound::storage::{BrowserStorage, Storage};

/// The storage key the choice lives under.
pub const DETAILS_KEY: &str = "zwiper-web.details-panel";

/// The stored value while the panel is shown.
const SHOWN: &str = "shown";

/// The stored value while the panel is hidden.
const HIDDEN: &str = "hidden";

/// Saves and loads whether the details panel is shown, in `S`.
#[derive(Debug, Clone, Default)]
pub struct DetailsStore<S> {
    storage: S,
}

impl DetailsStore<BrowserStorage> {
    /// The store over the browser's `localStorage`.
    pub fn browser() -> Self {
        Self::new(BrowserStorage)
    }
}

impl<S: Storage> DetailsStore<S> {
    /// A store over `storage`.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Whether the panel was last left shown, or `None` when the eye was
    /// never pressed or the value is unreadable.
    pub fn load(&self) -> Option<bool> {
        match self.storage.read(DETAILS_KEY)?.as_str() {
            SHOWN => Some(true),
            HIDDEN => Some(false),
            _ => None,
        }
    }

    /// Stores whether the panel is `shown`, logging a failure; a lost write
    /// only means the next visit opens on the default.
    pub fn save(&self, shown: bool) {
        let value = if shown { SHOWN } else { HIDDEN };
        if let Err(e) = self.storage.write(DETAILS_KEY, value) {
            tracing::debug!("details panel choice not saved: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outbound::storage::MemoryStorage;

    #[test]
    fn a_saved_choice_loads_back() {
        let store = DetailsStore::new(MemoryStorage::default());
        store.save(false);
        assert_eq!(store.load(), Some(false));
        store.save(true);
        assert_eq!(store.load(), Some(true));
    }

    #[test]
    fn nothing_or_garbage_loads_no_choice() {
        assert_eq!(DetailsStore::new(MemoryStorage::default()).load(), None);
        let storage = MemoryStorage::default();
        storage.write(DETAILS_KEY, "nope").unwrap();
        assert_eq!(DetailsStore::new(storage).load(), None);
    }
}

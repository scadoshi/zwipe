//! Whose counters Home shows, kept in browser storage so Home opens on the
//! last choice. It stays after logout, as the app keeps it.

use zwipe_ui::home::Scope;

use crate::outbound::storage::{BrowserStorage, Storage};

/// The storage key the scope lives under.
pub const SCOPE_KEY: &str = "zwiper-web.count-scope";

/// Saves and loads the count scope in `S`, as the app's bytes `all` or
/// `mine`.
#[derive(Debug, Clone, Default)]
pub struct ScopeStore<S> {
    storage: S,
}

impl ScopeStore<BrowserStorage> {
    /// The store over the browser's `localStorage`.
    pub fn browser() -> Self {
        Self::new(BrowserStorage)
    }
}

impl<S: Storage> ScopeStore<S> {
    /// A store over `storage`.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// The stored scope; everybody when there is none or it is anything but
    /// `mine`.
    pub fn load(&self) -> Scope {
        self.storage
            .read(SCOPE_KEY)
            .map_or_else(Scope::default, |stored| {
                Scope::from_bytes(stored.as_bytes())
            })
    }

    /// Stores `scope`, logging a failure; a lost write only means the next
    /// visit opens on everybody.
    pub fn save(&self, scope: Scope) {
        let value = String::from_utf8_lossy(scope.as_bytes());
        if let Err(e) = self.storage.write(SCOPE_KEY, &value) {
            tracing::debug!("count scope not saved: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outbound::storage::MemoryStorage;

    #[test]
    fn a_saved_scope_loads_back() {
        let store = ScopeStore::new(MemoryStorage::default());
        store.save(Scope::OnlyMe);
        assert_eq!(store.load(), Scope::OnlyMe);
        store.save(Scope::Everybody);
        assert_eq!(store.load(), Scope::Everybody);
    }

    #[test]
    fn nothing_or_garbage_loads_everybody() {
        assert_eq!(
            ScopeStore::new(MemoryStorage::default()).load(),
            Scope::Everybody
        );
        let storage = MemoryStorage::default();
        storage.write(SCOPE_KEY, "nope").unwrap();
        assert_eq!(ScopeStore::new(storage).load(), Scope::Everybody);
    }
}

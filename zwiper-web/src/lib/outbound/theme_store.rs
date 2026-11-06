//! The last theme, kept in browser storage so the login screen opens in it.
//!
//! Unlike the session, it stays after logout.

use components::ThemeConfig;

use crate::outbound::storage::{BrowserStorage, Storage};

/// The storage key the theme lives under.
pub const THEME_KEY: &str = "zwiper-web.theme";

/// Saves and loads the last theme in `S`.
#[derive(Debug, Clone, Default)]
pub struct ThemeStore<S> {
    storage: S,
}

impl ThemeStore<BrowserStorage> {
    /// The store over the browser's `localStorage`.
    pub fn browser() -> Self {
        Self::new(BrowserStorage)
    }
}

impl<S: Storage> ThemeStore<S> {
    /// A store over `storage`.
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// The stored theme. `None` when there is none, when it does not parse,
    /// or when it names no theme this build has; a retired theme loads as
    /// its replacement.
    pub fn load(&self) -> Option<ThemeConfig> {
        let json = self.storage.read(THEME_KEY)?;
        serde_json::from_str::<ThemeConfig>(&json)
            .ok()
            .and_then(ThemeConfig::resolve)
    }

    /// Stores `theme`, logging a failure. The theme is a convenience, so a
    /// lost write costs only the next visit's first look.
    pub fn save(&self, theme: &ThemeConfig) {
        let result = serde_json::to_string(theme)
            .map_err(|e| e.to_string())
            .and_then(|json| {
                self.storage
                    .write(THEME_KEY, &json)
                    .map_err(|e| e.to_string())
            });
        if let Err(e) = result {
            tracing::debug!("theme not saved: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outbound::storage::MemoryStorage;

    #[test]
    fn a_saved_theme_loads_back() {
        let store = ThemeStore::new(MemoryStorage::default());
        let theme = ThemeConfig {
            name: "nord".to_string(),
            is_dark: false,
        };
        store.save(&theme);
        assert_eq!(store.load(), Some(theme));
    }

    #[test]
    fn an_unknown_theme_loads_none() {
        let store = ThemeStore::new(MemoryStorage::default());
        store.save(&ThemeConfig {
            name: "a-theme-from-a-newer-build".to_string(),
            is_dark: true,
        });
        assert_eq!(store.load(), None);
    }

    #[test]
    fn garbage_loads_none() {
        let storage = MemoryStorage::default();
        storage.write(THEME_KEY, "nope").unwrap();
        assert_eq!(ThemeStore::new(storage).load(), None);
    }
}

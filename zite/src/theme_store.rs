//! Local theme persistence for the website.
//!
//! Remembers the visitor's picked theme across reloads via the browser's
//! `localStorage`, so the site opens in their last-used theme instead of the
//! hardcoded default. Client-only: on the server build (SSR) these are no-ops,
//! so the signal falls back to [`ThemeConfig::default`] and the client corrects
//! it on hydration (localStorage exists only in the browser).

use zwipe_components::ThemeConfig;

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::ThemeConfig;

    /// `localStorage` key holding the JSON-serialized [`ThemeConfig`].
    const KEY: &str = "zwipe.theme";

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    pub fn load() -> Option<ThemeConfig> {
        let raw = storage()?.get_item(KEY).ok().flatten()?;
        serde_json::from_str(&raw).ok()
    }

    pub fn save(cfg: &ThemeConfig) {
        if let Some(storage) = storage()
            && let Ok(json) = serde_json::to_string(cfg)
        {
            let _ = storage.set_item(KEY, &json);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::ThemeConfig;

    pub fn load() -> Option<ThemeConfig> {
        None
    }

    pub fn save(_cfg: &ThemeConfig) {}
}

pub use imp::{load, save};

//! A site's theme, remembered across visits in the browser's `localStorage`.
//!
//! Storage exists only in the browser, so off wasm (the server build, the
//! prerender) loading finds nothing and saving does nothing: the page renders
//! [`ThemeConfig::default`] and the browser adopts the stored theme after
//! hydration.

use dioxus::prelude::*;
use zwipe_core::domain::user::models::theme::ThemeConfig;

#[cfg(target_arch = "wasm32")]
mod imp {
    use super::ThemeConfig;

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    pub fn load(key: &str) -> Option<ThemeConfig> {
        let raw = storage()?.get_item(key).ok().flatten()?;
        serde_json::from_str(&raw).ok()
    }

    pub fn save(key: &str, cfg: &ThemeConfig) {
        if let Some(storage) = storage()
            && let Ok(json) = serde_json::to_string(cfg)
        {
            let _ = storage.set_item(key, &json);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::ThemeConfig;

    pub fn load(_key: &str) -> Option<ThemeConfig> {
        None
    }

    pub fn save(_key: &str, _cfg: &ThemeConfig) {}
}

/// The site's theme, stored as JSON under `key` (the sites use
/// `"zwipe.theme"`, so one pick follows a visitor across them).
///
/// Starts at the default so the browser's first render matches the
/// prerendered page; seeding it from storage would desync hydration, which
/// keeps the server's DOM and never reconciles it. The stored theme is
/// adopted just after mount instead, and every change after that is saved.
/// Changes before the stored theme is read are not saved, so the default
/// never overwrites it.
///
/// Putting the theme class on the page (the body, a wrapper) stays the
/// host's, as does providing the signal as context.
pub fn use_persisted_theme(key: &'static str) -> Signal<ThemeConfig> {
    let mut theme = use_signal(ThemeConfig::default);
    let mut loaded = use_signal(|| false);

    use_effect(move || {
        if let Some(stored) = imp::load(key) {
            theme.set(stored);
        }
        loaded.set(true);
    });

    use_effect(move || {
        let cfg = theme.read().clone();
        if loaded() {
            imp::save(key, &cfg);
        }
    });

    theme
}

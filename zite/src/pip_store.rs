//! The visitor's pip style, remembered across visits in the browser's
//! `localStorage` beside the theme (`zwipe.theme`), the same way orangui's
//! `use_persisted_theme` keeps that.
//!
//! Storage exists only in the browser, so off wasm (the server build, the
//! prerender) loading finds nothing and saving does nothing: the page renders
//! [`PipStyle::Theme`] and the browser adopts the stored style after
//! hydration. `index.html` reads the same key before first paint, so the
//! classic pips show without a flash.

use dioxus::prelude::*;
use zwipe_core::domain::user::preferences::PipStyle;

/// The `localStorage` key, beside the theme's `zwipe.theme`.
pub(crate) const PIP_STYLE_KEY: &str = "zwipe.pips";

#[cfg(target_arch = "wasm32")]
mod imp {
    use zwipe_core::domain::user::preferences::PipStyle;

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    pub(super) fn load(key: &str) -> Option<PipStyle> {
        let raw = storage()?.get_item(key).ok().flatten()?;
        serde_json::from_str(&raw).ok()
    }

    pub(super) fn save(key: &str, style: PipStyle) {
        if let Some(storage) = storage()
            && let Ok(json) = serde_json::to_string(&style)
        {
            let _ = storage.set_item(key, &json);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use zwipe_core::domain::user::preferences::PipStyle;

    pub(super) fn load(_key: &str) -> Option<PipStyle> {
        None
    }

    pub(super) fn save(_key: &str, _style: PipStyle) {}
}

/// The site's pip style, stored as JSON under [`PIP_STYLE_KEY`].
///
/// Starts at the default so the browser's first render matches the
/// prerendered page, then adopts the stored style just after mount; every
/// change after that is saved. Changes before the stored style is read are
/// not saved, so the default never overwrites it.
pub(crate) fn use_persisted_pip_style() -> Signal<PipStyle> {
    let mut style = use_signal(PipStyle::default);
    let mut loaded = use_signal(|| false);

    use_effect(move || {
        if let Some(stored) = imp::load(PIP_STYLE_KEY) {
            style.set(stored);
        }
        loaded.set(true);
    });

    use_effect(move || {
        let current = style();
        if loaded() {
            imp::save(PIP_STYLE_KEY, current);
        }
    });

    style
}

//! A card's badges ease in and out, slide over and pop on each change, in the stacks and the
//! details drawer (`assets/badge-ease.js`).

use dioxus::prelude::*;

/// The badges' motion, installed once a page: see the script's own header.
pub const BADGE_EASE_JS: &str = include_str!("../assets/badge-ease.js");

/// Installs [`BADGE_EASE_JS`] when the first component that shows badges mounts.
pub(crate) fn use_badge_ease() {
    use_hook(|| {
        document::eval(BADGE_EASE_JS);
    });
}

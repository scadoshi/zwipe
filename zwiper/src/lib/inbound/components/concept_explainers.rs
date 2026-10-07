//! Canonical concept explainers: Deck tags, Oracle tags.
//!
//! One source of truth per concept, reused by the on-demand `InfoButton`
//! explainers and the one-time hint dialogs that touch the same ground. Each
//! returns only the body (a `HintBullets`), so callers supply their own
//! `HintDialog` shell and title.

use crate::inbound::components::hint_dialog::HintColored;
use dioxus::prelude::*;
use zwipe_components::{HintBullet, HintBullets};
use zwipe_core::domain::deck::MAX_DECK_TAGS;

/// Deck tags: the archetype(s) you pick that seed oracle tags.
#[component]
pub fn DeckTagsExplainer() -> Element {
    rsx! {
        HintBullets {
            HintBullet { "The " HintColored { color: "--accent-secondary", "archetypes" } " the deck is built around, like Aristocrats or Voltron" }
            HintBullet { "Picking one pre-selects the " HintColored { color: "--accent-tertiary", "oracle tags" } " that define it" }
            HintBullet { "Up to " HintColored { color: "--accent-primary", "{MAX_DECK_TAGS}" } }
        }
    }
}

/// Oracle tags: the granular, directly editable tags that sharpen suggestions.
#[component]
pub fn OracleTagsExplainer() -> Element {
    rsx! {
        HintBullets {
            HintBullet { "The " HintColored { color: "--accent-tertiary", "specific" } " things the deck does, like spot removal, ramp or reanimation" }
            HintBullet { "They " HintColored { color: "--accent-secondary", "shape which cards we suggest" } }
            HintBullet { "Your " HintColored { color: "--accent-primary", "deck tags" } " pre-pick a starter set from about 4,500. Fine to leave as is" }
        }
    }
}

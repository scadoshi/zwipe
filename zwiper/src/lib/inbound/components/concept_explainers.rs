//! Canonical concept explainers: Deck tags, Oracle tags.
//!
//! One source of truth per concept, reused by the on-demand `InfoButton`
//! explainers and the one-time hint dialogs that touch the same ground. Each
//! returns only the body (a `HintBullets`), so callers supply their own
//! `HintDialog` shell and title.

use crate::inbound::components::hint_dialog::{HintBullet, HintBullets, HintColored};
use dioxus::prelude::*;
use zwipe_core::domain::deck::MAX_DECK_TAGS;

/// Deck tags: the archetype(s) you pick that seed oracle tags.
#[component]
pub fn DeckTagsExplainer() -> Element {
    rsx! {
        HintBullets {
            HintBullet {
                "Pick the "
                HintColored { color: "--accent-secondary", "archetype" }
                "(s) your deck is built around, like Aristocrats or Voltron."
            }
            HintBullet {
                "Picking one auto-selects the "
                HintColored { color: "--accent-tertiary", "oracle tags" }
                " that define it, which sharpens the cards we suggest."
            }
            HintBullet {
                "Add up to "
                HintColored { color: "--accent-tertiary", "{MAX_DECK_TAGS} tags" }
                "."
            }
        }
    }
}

/// Oracle tags: the granular, directly editable tags that sharpen suggestions.
#[component]
pub fn OracleTagsExplainer() -> Element {
    rsx! {
        HintBullets {
            HintBullet {
                "The "
                HintColored { color: "--accent-tertiary", "specific" }
                " things your deck does, like spot removal, ramp, or reanimation."
            }
            HintBullet {
                "Selecting them "
                HintColored { color: "--accent-secondary", "sharpens which cards we suggest" }
                "."
            }
            HintBullet {
                "Your "
                HintColored { color: "--accent-secondary", "deck tags" }
                " pre-pick a starter set from the ~4,500 available. Leave them if you're not sure."
            }
        }
    }
}

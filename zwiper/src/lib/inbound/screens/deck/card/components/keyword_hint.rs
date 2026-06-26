//! "Keywords" dialog for the active swipe card.
//!
//! While swiping, a card with keyword abilities gets a "Keywords" button in the
//! util bar (alongside Back / Filter / Refresh) that opens this dialog, which
//! lists every keyword on the card with a short definition (via
//! [`keyword_reminder`]). Cards without keywords show nothing.
//!
//! The dialog is rendered *outside* the util bar on purpose: the util bar has a
//! `transform` (an iOS fix) which would otherwise become the containing block
//! for the dialog's fixed positioning and anchor it to the footer. So the
//! button lives in the util bar but this controlled dialog renders alongside
//! the screen's other dialogs, with [`card_has_keywords`] gating the button.

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::keyword::keyword_reminder;

use crate::inbound::components::hint_dialog::{HintBullet, HintBullets, HintDialog, HintKey};

/// True if the card has any keyword abilities worth a "Keywords" button.
pub(crate) fn card_has_keywords(card: &Card) -> bool {
    card.scryfall_data
        .keywords
        .as_ref()
        .is_some_and(|k| !k.is_empty())
}

/// Dialog listing each keyword on the card with its reminder. Controlled via
/// `open`; pair it with a util-bar button that sets `open` to true.
#[component]
pub(crate) fn KeywordHintDialog(open: Signal<bool>, card: Card) -> Element {
    let keywords = card.scryfall_data.keywords.as_deref().unwrap_or_default();
    if keywords.is_empty() {
        return rsx! {};
    }

    let items: Vec<(String, &'static str)> = keywords
        .iter()
        .map(|k| (k.clone(), keyword_reminder(k)))
        .collect();

    rsx! {
        HintDialog {
            open,
            title: "Keywords",
            HintBullets {
                for (kw , def) in items.iter() {
                    HintBullet {
                        key: "{kw}",
                        HintKey { "{kw}" }
                        " {def}"
                    }
                }
            }
        }
    }
}

//! App-root hint host: a sender/receiver for on-demand "?" help.
//!
//! An [`InfoButton`](super::info_button::InfoButton) anywhere in the tree posts a
//! [`HintTopic`] to a context signal; this single [`HintHost`], mounted at the app
//! root (beside the router), renders the dialog. Because the dialog only ever
//! renders here, outside every screen's `content-enter` / scroll container, its
//! `position: fixed` overlay can't be trapped by an ancestor's containing block.
//! A single `Option` also means only one hint shows at a time.

use dioxus::prelude::*;

use crate::inbound::components::{
    alert_dialog::{
        AlertDialogAction, AlertDialogActions, AlertDialogContent, AlertDialogDescription,
        AlertDialogRoot, AlertDialogTitle,
    },
    concept_explainers::{CardRolesExplainer, DeckTagsExplainer, OracleTagsExplainer},
};

/// A concept a "?" button can explain. Cheap `Copy` message on the hint channel.
#[derive(Clone, Copy, PartialEq)]
pub enum HintTopic {
    /// The archetype tags you pick on a deck.
    DeckTags,
    /// The granular functional tags that sharpen suggestions.
    OracleTags,
    /// The read-side role chips shown on a card.
    CardRoles,
}

impl HintTopic {
    /// Dialog title for this topic (mirrors the in-app labels).
    pub fn title(&self) -> &'static str {
        match self {
            Self::DeckTags => "Deck tags",
            Self::OracleTags => "Oracle tags",
            Self::CardRoles => "Card roles",
        }
    }
}

/// The receiver: reads the hint channel and renders the current topic's dialog.
/// Mount once at the app root, beside the router. `spawn_upkeeper` provides the
/// `Signal<Option<HintTopic>>` this reads.
#[component]
pub fn HintHost() -> Element {
    let mut hint: Signal<Option<HintTopic>> = use_context();
    let topic = hint();
    rsx! {
        AlertDialogRoot {
            open: topic.is_some(),
            on_open_change: move |open: bool| {
                if !open {
                    hint.set(None);
                }
            },
            AlertDialogContent {
                if let Some(topic) = topic {
                    AlertDialogTitle { "{topic.title()}" }
                    hr { class: "dialog-rule" }
                    AlertDialogDescription {
                        match topic {
                            HintTopic::DeckTags => rsx! { DeckTagsExplainer {} },
                            HintTopic::OracleTags => rsx! { OracleTagsExplainer {} },
                            HintTopic::CardRoles => rsx! { CardRolesExplainer {} },
                        }
                    }
                    hr { class: "dialog-rule" }
                    AlertDialogActions {
                        AlertDialogAction { on_click: move |_| hint.set(None), "Got it" }
                    }
                }
            }
        }
    }
}

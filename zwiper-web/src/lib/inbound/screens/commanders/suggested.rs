//! The Commanders screen's Suggested list: the suggested commanders with their
//! Swipe and Clear skips, its Show row over the list at every width. The
//! screen's one filter narrows it.

use components::{Chip, DangerZoneAction};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::inbound::cards::card_row::ShowRowArt;
use crate::inbound::chrome::ghost::{GhostShowRow, GhostText};
use crate::inbound::screens::playstyle::{
    ShowIdentityChips, SuggestedFold, SuggestedOverlays, Suggestions,
};

/// The list's header.
const LIST_TITLE: &str = "Suggested";

/// The Suggested list over the screen's `list`, which the screen's filter narrows; its rows open
/// into `expanded_card`. Its Show row's Art chip shows or hides this list's art only.
#[component]
pub(super) fn SuggestedPanel(list: Suggestions, expanded_card: Signal<Option<Uuid>>) -> Element {
    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));
    let Suggestions {
        deck_count,
        is_loading,
        load_failed,
        active_identity,
        identities,
        ..
    } = list;
    // The console's Show row.
    let show_chips = rsx! {
        div { class: "web-list-console",
            ShowIdentityChips { active_identity, identities: identities(),
                Chip {
                    selected: show_row_art(),
                    onclick: move |_| show_row_art.toggle(),
                    "Art"
                }
            }
        }
    };
    rsx! {
        div {
            class: "web-commanders-list web-commanders-suggested",
            "data-cursor-list": "",
            div { class: "flex-col web-suggested",
                if is_loading() {
                    GhostShowRow {}
                    div { class: "card-group",
                        div { class: "card-group-header", GhostText { chars: 9 } }
                        for i in 0..3 {
                            div { key: "{i}", class: "skeleton-card-row web-ghost-card-row",
                                div { class: "skeleton-bar skeleton-card-thumb" }
                                div { class: "skeleton-bar skeleton-card-bar-row" }
                            }
                        }
                    }
                } else if load_failed() {
                    div { class: "message-empty",
                        span { class: "chip-note", "Couldn't load your suggestions" }
                    }
                } else if deck_count() == 0 {
                    div { class: "message-empty",
                        span { class: "chip-note", "Build a deck to see suggestions" }
                    }
                } else {
                    // The console sits over the list at every width, as Staples' and the
                    // Maybeboard's do.
                    div { class: "deck-controls", {show_chips} }
                    SuggestedFold { list, title: LIST_TITLE, expanded_card }
                    DangerZoneAction {
                        label: "Clear skips",
                        button: "Clear",
                        onclick: move |()| list.ask_clear(),
                    }
                }
            }
        }
        SuggestedOverlays { list }
    }
}

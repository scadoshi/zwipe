//! The Commanders screen's Suggested list: the suggested commanders with their
//! Swipe and Clear skips, its Show row over the list at every width. The
//! screen's one filter narrows it.

use components::{Chip, DangerZoneAction, LoadFailed};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::screens::suggested_commanders as suggested;

use crate::inbound::cards::card_row::ShowRowArt;
use crate::inbound::chrome::ghost::{GhostButton, GhostChip, GhostChipRow, GhostHint, GhostText};
use crate::inbound::screens::playstyle::{SuggestedFold, Suggestions};

/// The list's header.
const LIST_TITLE: &str = suggested::LIST_TITLE;

/// The Suggested list over the screen's `list`, which the screen's filter narrows; its rows open
/// into `expanded_card`. Its Show row's Art chip shows or hides this list's art only. Its
/// overlays render at the screen's root.
#[component]
pub(super) fn SuggestedPanel(list: Suggestions, expanded_card: Signal<Option<Uuid>>) -> Element {
    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));
    // The console's Show row, Art alone: colors come from the screen's filter.
    let show_chips = rsx! {
        div { class: "web-list-console",
            div { class: "chip-row", "data-settle": "",
                span { class: "stat-chip stat-chip-p1 chip-row-label", "Show" }
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
                if list.is_loading() {
                    GhostChipRow { label: 4, chips: vec![3] }
                    div { class: "card-group",
                        div { class: "card-group-header card-group-header-with-hint",
                            GhostText { chars: 9 }
                            span { class: "card-list-fold-hint", GhostHint {} }
                            div { class: "card-group-header-actions", GhostButton { chars: 5 } }
                        }
                        for i in 0..3 {
                            div { key: "{i}", class: "skeleton-card-row web-ghost-card-row",
                                div { class: "skeleton-bar skeleton-card-thumb" }
                                div { class: "skeleton-bar skeleton-card-bar-row" }
                            }
                        }
                    }
                    div { class: "card-list-more", GhostChip { chars: 9, button: true } }
                } else if list.load_failed() {
                    LoadFailed { message: suggested::LOAD_FAILED, on_retry: move |()| list.retry() }
                } else {
                    // The console sits over the list at every width, as Staples' and the
                    // Maybeboard's do.
                    div { class: "deck-controls", {show_chips} }
                    SuggestedFold { list, title: LIST_TITLE, expanded_card }
                    DangerZoneAction {
                        label: suggested::CLEAR_SKIPS,
                        button: "Clear",
                        onclick: move |()| list.ask_clear(),
                    }
                }
            }
        }
    }
}

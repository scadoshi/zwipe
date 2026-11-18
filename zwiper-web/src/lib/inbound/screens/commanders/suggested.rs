//! The Commanders screen's Suggested list: Playstyle's suggested commanders,
//! drawn with Playstyle's own fold, Swipe and Clear skips, the Show row over
//! the fold on a wide window and inside it below the breakpoint, with the
//! list's own filter beside it.

use components::{Chip, DangerZoneAction};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;

use super::list_filter::ListFilterButton;
use crate::inbound::cards::card_row::ShowRowArt;
use crate::inbound::chrome::ghost::{GhostShowRow, GhostText};
use crate::inbound::chrome::media::use_wide;
use crate::inbound::screens::playstyle::{
    ShowIdentityChips, SuggestedFold, SuggestedOverlays, Suggestions,
};

/// The list's header.
const LIST_TITLE: &str = "Suggested";

/// The Suggested list over the screen's `list`, narrowed by its own `filter`, which its console's
/// funnel opens through `filter_open`; its rows open into `expanded_card`. Its Show row's Art chip
/// shows or hides this list's art only.
#[component]
pub(super) fn SuggestedPanel(
    list: Suggestions,
    filter: Signal<CardQueryBuilder>,
    filter_open: Signal<bool>,
    expanded_card: Signal<Option<Uuid>>,
) -> Element {
    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));
    let wide = use_wide();
    let Suggestions {
        deck_count,
        is_loading,
        load_failed,
        active_identity,
        identities,
        ..
    } = list;
    // The console's Show row, then the list's own filter.
    let show_chips = rsx! {
        div { class: "web-list-console",
            ShowIdentityChips { active_identity, identities: identities(),
                Chip {
                    selected: show_row_art(),
                    onclick: move |_| show_row_art.toggle(),
                    "Art"
                }
            }
            ListFilterButton { filter, filter_open }
        }
    };
    rsx! {
        div { class: "web-commanders-list web-commanders-suggested", "data-cursor-list": "",
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
                    if wide() {
                        div { class: "deck-controls", {show_chips.clone()} }
                    }
                    SuggestedFold { list, title: LIST_TITLE, wide: wide(), show_chips, expanded_card }
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

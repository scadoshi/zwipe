//! Playstyle (the app's suggested commanders): what the user builds, read
//! from every deck they own, over their top suggested commanders and their
//! staples.
//!
//! On a wide window the profile's Summary is the hero at the top of zite's
//! page and every profile section is a fold, each open on entry. The Show row
//! sits above the suggestions and Staples' rows above Staples. Below the
//! breakpoint it is the app's single column with the profile as one
//! accordion, Color map open, and the chip rows inside their panels. Swipe
//! sits in the list's header and Clear skips is the Danger zone panel after
//! Staples.

mod chips;
mod profile;
mod rows;
mod skeleton;
mod suggestions;

use std::collections::HashSet;

use components::deck_cursor::use_card_list_cursor;
use components::{Chip, DangerZoneAction, HoverStackHost, use_hover_stack};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::suggestions::HttpStaple;
use zwipe_ui::playstyle::StapleDeck;
use zwipe_ui::screens::playstyle as model;
use zwipe_ui::shortcuts::ListLayout;
use zwipe_ui::telemetry::vocabulary::Screen;

pub use self::chips::{IdentityPips, ShowIdentityChips};
use self::profile::{BuilderProfileSections, ProfileSummary};
use self::rows::StaplesPanel;
pub use self::rows::SuggestionReasons;
use self::skeleton::PlaystyleSkeleton;
pub(crate) use self::suggestions::{
    SuggestedFold, SuggestedOverlays, Suggestions, use_suggestions,
};
use crate::inbound::cards::card_row::{CardOverlaysHost, ShowRowArt, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::cards::row_keys::use_card_row_keys;
use crate::inbound::cards::stack_view::use_row_faces;
use crate::inbound::chrome::media::use_wide;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;

/// The suggestions list's title.
const LIST_TITLE: &str = "Suggested commanders";

/// The Playstyle screen.
#[component]
pub fn Playstyle() -> Element {
    telemetry::use_telemetry_screen(Screen::SuggestedCommanders);
    let hint_open = use_one_time_hint(OneTimeHint::SuggestedCommanders);
    let mut show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));
    let examples = use_otag_reveals();
    let overlays = use_card_overlays();
    let hover_stack = use_hover_stack();

    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    let list = use_suggestions(None);
    let Suggestions {
        mut entries,
        deck_count,
        profile,
        is_loading,
        load_failed,
        active_identity,
        identities,
        deck_profiles,
        ..
    } = list;
    // The hero's Summary only shows on a wide window, where it is a fold.
    let summary_section: Signal<Option<String>> = use_signal(|| None);
    // The titles of the profile folds shut on a wide window; all start open.
    let folded: Signal<HashSet<String>> = use_signal(HashSet::new);
    let wide = use_wide();
    use_row_faces();
    // Every card the screen lists, which the card keys act on.
    let listed_cards = use_memo(move || {
        let staples = profile
            .read()
            .as_ref()
            .map_or_default(|p| p.staples.iter().map(|s| s.card.clone()).collect::<Vec<_>>());
        entries
            .read()
            .iter()
            .map(|s| s.card.clone())
            .chain(staples)
            .collect::<Vec<Card>>()
    });
    use_card_row_keys(listed_cards, expanded_card, ListLayout::SideBySide);
    // The arrow keys and h, j, k and l move a cursor through the rows: h and l cross between
    // the two lists, each marked as a cursor list.
    use_card_list_cursor();

    let cold_start = deck_count() == 0;
    // The user's decks, which name a staple's deck chips; none until they load.
    let staple_decks: Vec<StapleDeck> = deck_profiles
        .read()
        .as_ref()
        .map_or_default(|decks| decks.iter().map(StapleDeck::from).collect());
    // A suggested commander's printing carries into Create deck; a staple only
    // browses its printings.
    let picks = overlays
        .printing
        .read()
        .as_ref()
        .is_some_and(|card| model::picks_printing(&entries.read(), card));
    let folds = wide().then_some(folded);
    let show_chips = rsx! {
        ShowIdentityChips { active_identity, identities: identities(),
            Chip {
                selected: show_row_art(),
                onclick: move |_| show_row_art.toggle(),
                "Art"
            }
        }
    };
    let staples: Vec<HttpStaple> = profile
        .read()
        .as_ref()
        .map_or_default(|p| p.staples.clone());

    rsx! {
        ScreenFrame {
            title: "Playstyle",
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div {
                class: "screen-content",
                onmousemove: move |e| hover_stack.track(e.client_coordinates().x),
                if is_loading() {
                    PlaystyleSkeleton { wide: wide() }
                } else if load_failed() {
                    div { class: "message-empty",
                        span { class: "chip-note", "Couldn't load your suggestions" }
                    }
                } else if cold_start {
                    div { class: "message-empty",
                        span { class: "chip-note", "Build a deck to see your playstyle" }
                    }
                } else {
                    // The lists settle into place when a Show, Type or Color chip filters them.
                    div { class: "web-playstyle web-playstyle-grid web-page content-enter", "data-settle-scope": "",
                        if let Some(profile) = profile() {
                            div { class: "web-hero web-playstyle-summary-hero web-playstyle-cell",
                                ProfileSummary { profile, open_section: summary_section, folds }
                            }
                        }
                        div { class: "web-columns",
                            if let Some(profile) = profile() {
                                div { class: "web-playstyle-profile",
                                    BuilderProfileSections { profile, folds }
                                }
                            }
                            div { class: "web-playstyle-list",
                                div { class: "web-playstyle-cell web-playstyle-suggested", "data-cursor-list": "",
                                    if wide() {
                                        div { class: "deck-controls", {show_chips.clone()} }
                                    }
                                    SuggestedFold {
                                        list,
                                        title: LIST_TITLE,
                                        wide: wide(),
                                        show_chips,
                                        expanded_card,
                                    }
                                }
                                if !staples.is_empty() {
                                    StaplesPanel { staples, deck_count: deck_count(), decks: staple_decks, expanded_card, wide: wide() }
                                }
                                div { class: "web-playstyle-cell web-playstyle-danger",
                                    DangerZoneAction {
                                        label: "Clear skips",
                                        button: "Clear",
                                        onclick: move |()| list.ask_clear(),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::SuggestedCommanders, open: hint_open }
        SuggestedOverlays { list }
        HoverStackHost { stack: hover_stack }
        CardOverlaysHost {
            overlays,
            on_printing_save: picks.then_some(EventHandler::new(move |picked: Card| {
                let target = picked.scryfall_data.oracle_id;
                if target.is_some() {
                    let mut expanded = expanded_card;
                    expanded.set(Some(picked.scryfall_data.id));
                    if let Some(slot) = entries
                        .write()
                        .iter_mut()
                        .find(|s| s.card.scryfall_data.oracle_id == target)
                    {
                        slot.card = picked;
                    }
                }
            })),
        }
        OtagExamplesHost { examples }
    }
}

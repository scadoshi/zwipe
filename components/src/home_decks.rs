//! Home's Decks panel: the deck count and the Create and All decks buttons in
//! its header, the latest decks as tiles under it, each a tap into the deck.
//! With no deck the one button is Create a deck over one muted line.

use dioxus::prelude::*;
use orangui::{Button, ButtonVariant};
use uuid::Uuid;
use zwipe_ui::screens::home::{self, DeckPanel, DeckTile, Decks, DecksAction, LATEST_DECKS};

use crate::LoadFailed;
use crate::icons::{DockIcon, PLUS};

/// The panel between the hero and the Colors panel, in the Colors panel's
/// style: Decks and the count chip centered in its header with a + (Create a
/// deck) and the All decks chip in the right corner, then the [`LATEST_DECKS`] most recently updated decks
/// as three tiles across, each opening its deck. With no deck the count chip
/// hides, the corner chip is Create a deck and a muted line sits under the
/// header, the Colors panel's shape before it has counts; while the fetch is
/// out three tiles ghost; a failed fetch with nothing cached shows a muted
/// line under the All decks chip. Home's model decides which.
///
/// `onopen` is the All decks chip, `ondeck` a tile's tap with the deck's id
/// and `oncreate` the + or the Create a deck chip; the host routes each. With
/// `onretry` the failed line offers Retry.
#[component]
pub fn HomeDecks(
    decks: Decks,
    onopen: EventHandler,
    ondeck: EventHandler<Uuid>,
    oncreate: EventHandler,
    #[props(default)] onretry: Option<EventHandler>,
) -> Element {
    let action = decks.action();
    let onaction = move |()| match action {
        DecksAction::AllDecks => onopen.call(()),
        DecksAction::CreateDeck => oncreate.call(()),
    };
    rsx! {
        div { class: "container-sm flex-col home-flavor home-decks content-enter-delayed",
            match decks {
                Decks::Loaded(panel) => rsx! {
                    PanelHeader { count: Some(panel.count), action, onaction, oncreate }
                    DeckTiles { panel, ondeck }
                },
                Decks::Empty => rsx! {
                    PanelHeader { count: None, action, onaction, oncreate }
                    div { class: "message-empty home-decks-line", {home::NO_DECKS} }
                },
                Decks::Failed => rsx! {
                    PanelHeader { count: None, action, onaction, oncreate }
                    if let Some(onretry) = onretry {
                        LoadFailed { message: home::DECKS_FAILED, on_retry: onretry, class: "home-decks-line" }
                    } else {
                        div { class: "message-empty home-decks-line", {home::DECKS_FAILED} }
                    }
                },
                Decks::Loading => rsx! {
                    PanelHeader { count: None, action, onaction, oncreate }
                    div { class: "home-deck-tiles",
                        for i in 0..LATEST_DECKS {
                            div { key: "{i}", class: "home-deck-tile",
                                div { class: "skeleton-bar home-deck-tile-art" }
                                div { class: "home-deck-tile-chips",
                                    div { class: "skeleton-bar skeleton-chip home-deck-skeleton-chip" }
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

/// The panel's header: the title and the count chip (when there is a
/// count) centered as one unit, the action chip in the right corner. Beside
/// All decks sits a + that creates a deck; with no deck the one chip is
/// Create a deck.
#[component]
fn PanelHeader(
    count: Option<u32>,
    action: DecksAction,
    onaction: EventHandler,
    oncreate: EventHandler,
) -> Element {
    let label = match action {
        DecksAction::AllDecks => home::ALL_DECKS,
        DecksAction::CreateDeck => home::CREATE_DECK,
    };
    let offers_create = action == DecksAction::AllDecks;
    rsx! {
        div {
            class: if offers_create { "settings-list-header home-panel-header-centered home-decks-header-create" } else { "settings-list-header home-panel-header-centered" },
            div { class: "home-panel-header-title",
                span { class: "settings-list-title", {home::DECKS} }
                if let Some(count) = count {
                    span { class: "stat-chip", "{count}" }
                }
            }
            div { class: "home-panel-header-action",
                if offers_create {
                    Button {
                        variant: ButtonVariant::Action,
                        class: "btn-icon",
                        onclick: move |_| oncreate.call(()),
                        DockIcon { path: PLUS }
                        span { class: "sr-only", {home::CREATE_DECK} }
                    }
                }
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| onaction.call(()),
                    {label}
                }
            }
        }
    }
}

/// Three tiles across: the commander's art crop in a rounded square (a
/// filled square with the deck's pips without one), then the name chip and the
/// format chip under it. The name keeps to one line and ellipsizes; the
/// format sits beside it when both fit and under it otherwise.
#[component]
fn DeckTiles(panel: DeckPanel, ondeck: EventHandler<Uuid>) -> Element {
    rsx! {
        div { class: "home-deck-tiles",
            for deck in panel.latest {
                button {
                    key: "{deck.id}",
                    class: "home-deck-tile",
                    onclick: move |_| ondeck.call(deck.id),
                    div { class: "home-deck-tile-art",
                        if let Some(url) = deck.art_url.as_deref() {
                            img { src: "{url}", loading: "lazy", draggable: false, alt: "" }
                        } else {
                            {pips(&deck)}
                        }
                    }
                    div { class: "home-deck-tile-chips",
                        span { class: "stat-chip home-deck-tile-name", "{deck.name}" }
                        if let Some(format) = deck.format {
                            span { class: "stat-chip stat-chip-muted", "{format.display_name()}" }
                        }
                    }
                }
            }
        }
    }
}

/// The deck's color identity as mana pips, or the colorless pip for a deck
/// with no colored card.
fn pips(deck: &DeckTile) -> Element {
    if deck.color_identity.is_empty() {
        return rsx! {
            i { class: "ms ms-c ms-cost" }
        };
    }
    rsx! {
        for color in deck.color_identity.iter().copied() {
            i {
                key: "{color.to_short_name()}",
                class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
            }
        }
    }
}

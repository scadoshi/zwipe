//! Home's Decks panel: the deck count and an All decks chip in its header,
//! the latest decks as tiles under it, each a tap into the deck. With no
//! deck the chip is Create a deck over one muted line.

use components::{Button, ButtonVariant};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::screens::home::{self, DeckPanel, DeckTile, Decks, DecksAction, LATEST_DECKS};

/// The last deck list fetched, kept above the router (as
/// `Signal<Option<CachedDecks>>`) so a return to Home lists the decks at
/// once while the refetch is out.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedDecks {
    /// The user the decks belong to; another user's copy is never shown.
    pub user_id: Uuid,
    /// What zerver answered, most recently updated first.
    pub decks: Vec<DeckTile>,
}

/// The panel between the hero and the Colors panel, in the Colors panel's
/// style: Decks and the count chip centered in its header with the All decks
/// chip in the right corner, then the [`LATEST_DECKS`] most recently updated decks
/// as three tiles across, each opening its deck. With no deck the count chip
/// hides, the corner chip is Create a deck and a muted line sits under the
/// header, the Colors panel's shape before it has counts; while the fetch is
/// out three tiles ghost; a failed fetch with nothing cached shows a muted
/// line under the All decks chip. Home's model decides which.
#[component]
pub fn HomeDecks(
    decks: Decks,
    onopen: EventHandler,
    ondeck: EventHandler<Uuid>,
    oncreate: EventHandler,
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
                    PanelHeader { count: Some(panel.count), action, onaction }
                    DeckTiles { panel, ondeck }
                },
                Decks::Empty => rsx! {
                    PanelHeader { count: None, action, onaction }
                    div { class: "message-empty home-decks-line", {home::NO_DECKS} }
                },
                Decks::Failed => rsx! {
                    PanelHeader { count: None, action, onaction }
                    div { class: "message-empty home-decks-line", {home::DECKS_FAILED} }
                },
                Decks::Loading => rsx! {
                    PanelHeader { count: None, action, onaction }
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
/// count) centered as one unit, the action chip (All decks, or Create a
/// deck with no deck) alone in the right corner.
#[component]
fn PanelHeader(count: Option<u32>, action: DecksAction, onaction: EventHandler) -> Element {
    let label = match action {
        DecksAction::AllDecks => home::ALL_DECKS,
        DecksAction::CreateDeck => home::CREATE_DECK,
    };
    rsx! {
        div { class: "settings-list-header home-panel-header-centered",
            div { class: "home-panel-header-title",
                span { class: "settings-list-title", {home::DECKS} }
                if let Some(count) = count {
                    span { class: "stat-chip", "{count}" }
                }
            }
            div { class: "home-panel-header-action",
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

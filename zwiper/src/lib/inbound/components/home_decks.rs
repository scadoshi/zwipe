//! Home's Decks panel: the deck count and an All decks chip in its header,
//! the latest decks as tiles under it, each a tap into the deck.

use components::{Button, ButtonVariant};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_ui::screens::home::{self, DeckPanel, DeckTile, Decks, LATEST_DECKS};

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
/// style: Decks at the left of its header, the count chip and the All decks
/// chip at the right, then the [`LATEST_DECKS`] most recently updated decks
/// as three tiles across, each opening its deck. With no deck the one row
/// opens Create deck; while the fetch is out three tiles ghost; a failed
/// fetch with nothing cached shows a muted line over the All decks chip.
/// Home's model decides which.
#[component]
pub fn HomeDecks(
    decks: Decks,
    onopen: EventHandler,
    ondeck: EventHandler<Uuid>,
    oncreate: EventHandler,
) -> Element {
    rsx! {
        div { class: "container-sm flex-col home-flavor home-decks content-enter-delayed",
            match decks {
                Decks::Loaded(panel) => rsx! {
                    PanelHeader { count: Some(panel.count), onopen }
                    if panel.latest.is_empty() {
                        div {
                            class: "settings-row home-deck-row",
                            role: "button",
                            onclick: move |_| oncreate.call(()),
                            span { class: "home-deck-name", {home::CREATE_DECK} }
                        }
                    } else {
                        DeckTiles { panel, ondeck }
                    }
                },
                Decks::Failed => rsx! {
                    PanelHeader { count: None, onopen }
                    div { class: "message-empty home-decks-failed", {home::DECKS_FAILED} }
                },
                Decks::Loading => rsx! {
                    PanelHeader { count: None, onopen }
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

/// The panel's header: the title on the left, the count chip (when there
/// is a count) and the All decks chip on the right.
#[component]
fn PanelHeader(count: Option<u32>, onopen: EventHandler) -> Element {
    rsx! {
        div { class: "settings-list-header",
            span { class: "settings-list-title", {home::DECKS} }
            div { class: "card-group-header-actions",
                if let Some(count) = count {
                    span { class: "stat-chip", "{count}" }
                }
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| onopen.call(()),
                    {home::ALL_DECKS}
                }
            }
        }
    }
}

/// Three tiles across: the commander's art crop in a squircle (a filled
/// squircle with the deck's pips without one), then the name chip and the
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

/// The deck's color identity as mana pips in WUBRG order, or the colorless
/// pip for a deck with no colored card.
fn pips(deck: &DeckTile) -> Element {
    let colors: Vec<Color> = Color::all()
        .into_iter()
        .filter(|color| {
            deck.color_identity
                .iter()
                .any(|code| Color::try_from(code.as_str()).ok().as_ref() == Some(color))
        })
        .collect();
    if colors.is_empty() {
        return rsx! {
            i { class: "ms ms-c ms-cost" }
        };
    }
    rsx! {
        for color in colors {
            i {
                key: "{color.to_short_name()}",
                class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
            }
        }
    }
}

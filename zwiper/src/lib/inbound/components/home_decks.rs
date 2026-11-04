//! Home's Decks panel: the deck count and an All decks chip in its header,
//! the latest decks under it, each a tap into the deck.

use components::{Button, ButtonVariant};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_ui::screens::home::{self, DeckPanel, DeckRow, Decks, LATEST_DECKS};

/// The last deck list fetched, kept above the router (as
/// `Signal<Option<CachedDecks>>`) so a return to Home lists the decks at
/// once while the refetch is out.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedDecks {
    /// The user the decks belong to; another user's copy is never shown.
    pub user_id: Uuid,
    /// What zerver answered, most recently updated first.
    pub decks: Vec<DeckRow>,
}

/// How the panel's body draws the latest decks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(dead_code, reason = "both layouts stay until the owner picks one")]
enum Layout {
    /// One row per deck: the name on the left, the format chip on the right.
    Rows,
    /// A row of tiles: the commander's art, or the deck's color pips, over
    /// the name.
    Tiles,
}

/// The layout Home draws. Switch to [`Layout::Tiles`] for the tiles.
const LAYOUT: Layout = Layout::Rows;

/// The panel between the hero and the Color map, in the Color map's panel
/// style: Decks at the left of its header, the count chip and the All decks
/// chip at the right, then the [`LATEST_DECKS`] most recently updated decks,
/// each opening its deck. With no deck the one row opens Create deck; while
/// the fetch is out three rows ghost; a failed fetch with nothing cached
/// shows a muted line over the All decks chip. Home's model decides which.
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
                        match LAYOUT {
                            Layout::Rows => rsx! { DeckRows { panel, ondeck } },
                            Layout::Tiles => rsx! { DeckTiles { panel, ondeck } },
                        }
                    }
                },
                Decks::Failed => rsx! {
                    PanelHeader { count: None, onopen }
                    div { class: "message-empty home-decks-failed", {home::DECKS_FAILED} }
                },
                Decks::Loading => rsx! {
                    PanelHeader { count: None, onopen }
                    for i in 0..LATEST_DECKS {
                        div { key: "{i}", class: "settings-row home-deck-row",
                            div { class: "skeleton-bar home-deck-skeleton-name" }
                            div { class: "skeleton-bar skeleton-chip" }
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

/// Layout A: one settings row per deck, the name on the left and the format
/// chip on the right.
#[component]
fn DeckRows(panel: DeckPanel, ondeck: EventHandler<Uuid>) -> Element {
    rsx! {
        for deck in panel.latest {
            div {
                key: "{deck.id}",
                class: "settings-row home-deck-row",
                role: "button",
                onclick: move |_| ondeck.call(deck.id),
                span { class: "home-deck-name", "{deck.name}" }
                if let Some(format) = deck.format {
                    span { class: "stat-chip stat-chip-p3", "{format.display_name()}" }
                }
            }
        }
    }
}

/// Layout B: a row of tiles, each the commander's art crop (or the deck's
/// color pips without one) over the name on one line.
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
                    div { class: "home-deck-tile-name", "{deck.name}" }
                }
            }
        }
    }
}

/// The deck's color identity as mana pips in WUBRG order, or the colorless
/// pip for a deck with no colored card.
fn pips(deck: &DeckRow) -> Element {
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

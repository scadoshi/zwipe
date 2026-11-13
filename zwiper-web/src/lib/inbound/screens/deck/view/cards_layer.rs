//! The deck's Cards on a phone, as the app draws them: a full-screen layer over the deck's
//! Profile with a back button, the deck's head with its featured cards, the page's controls
//! (Quick add for the owner, View, Group by, Boards and Show), then the List's groups with their price and cart or the stacks, in tap mode on
//! a touch screen and under hover with a mouse. The owner's deck and a shared deck draw the same
//! layer; only the owner's passes edit actions and the edit buttons in the dock.
//!
//! The layer sits on the overlay stack, so Esc and the OS back close it as they close a sheet,
//! and a sheet it opens (the filter, a card's drawer) sits over it. The route holds it open
//! (`?view=cards`, [`use_cards_route`]).

use std::collections::{HashMap, HashSet};

use components::{
    BackArrow, CardGroup, DeckEditActions, DeckRow, DeckRowHost, DeckSection, DeckStacks, GroupBuy,
    PageHeader, StackDetails, TailPlace, group_tail, use_overlay_back_action, use_pointer,
};
use dioxus::history::history;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::Board;
use zwipe_ui::deck::view::DeckView;
use zwipe_ui::deck_cards::keys::RowInfo;
use zwipe_ui::deck_cards::stacks::CardsView;
use zwipe_ui::screens::deck_cards::TITLE;

use crate::inbound::router::Route;

/// The query a deck's route carries while its Cards are open.
const CARDS_QUERY: &str = "view=cards";

/// Opens (`true`) and closes (`false`) a deck's Cards through its route, `route` giving the
/// deck's route with a view. Opening pushes the route with `?view=cards`, so the browser's Back
/// closes the layer. Closing goes back one entry when the layer opened from this screen, and
/// otherwise (a link or a refresh that opened on the Cards) puts the Profile in its place, so
/// closing never leaves the deck. A close that comes from the browser's own back, which has
/// already left the Cards, only records the Profile as the screen's place.
pub(crate) fn use_cards_route(
    route: impl Fn(Option<DeckView>) -> Route + 'static,
) -> Callback<bool> {
    let navigator = use_navigator();
    let mut opened_here = use_signal(|| false);
    use_callback(move |open: bool| {
        let on_cards = history().current_route().contains(CARDS_QUERY);
        if open {
            opened_here.set(true);
            navigator.push(route(Some(DeckView::Cards)));
        } else if !on_cards {
            opened_here.set(false);
            navigator.replace(route(None));
        } else if *opened_here.peek() && navigator.can_go_back() {
            opened_here.set(false);
            navigator.go_back();
        } else {
            opened_here.set(false);
            navigator.replace(route(None));
        }
    })
}

/// The Cards layer; see the module's doc. `controls` are the page's own, `dock` the floating
/// icon bar's buttons, and `on_close` closes the layer from its back button, Esc or the OS back.
#[component]
pub(crate) fn DeckCardsLayer(
    // The deck's sections, each with its rows, in the page's order.
    sections: Vec<(DeckSection, Vec<RowInfo>)>,
    // List or Stacks.
    view: Signal<CardsView>,
    // The rows' price currency.
    currency: PriceCurrency,
    // The folded groups, the open row, its face and every card's face.
    collapsed: Signal<HashSet<String>>,
    selected: Signal<Option<Uuid>>,
    face: Signal<usize>,
    faces: Signal<HashMap<Uuid, usize>>,
    // Whether the rows show their art.
    show_art: Signal<bool>,
    // Each listed card's board.
    boards: HashMap<Uuid, Board>,
    // The stacks' reveal and drawer.
    details: StackDetails,
    // The store dialog a group's cart opens.
    group_buy: GroupBuy,
    // Whether the stacks deal their cards in.
    dealing: bool,
    // Whether the deck has no main board cards to show.
    empty: bool,
    // The rows' edits; `None` for a shared deck.
    actions: Option<DeckEditActions>,
    // The rows' buttons.
    host: DeckRowHost,
    // The page's head (the name, its facts and the featured cards) and its controls.
    head: Element,
    controls: Element,
    // The floating icon bar's buttons.
    dock: Element,
    on_close: Callback<()>,
) -> Element {
    let open = use_signal(|| true);
    let layer = use_overlay_back_action(open.into(), on_close);
    let pointer = use_pointer();
    let touch = !pointer();
    let stacks_on = view() == CardsView::Stacks;
    let mut folds = collapsed;
    rsx! {
        div {
            class: "screen web-cards-layer web-scroll-screen",
            role: "dialog",
            "aria-modal": "true",
            "aria-label": TITLE,
            style: layer().map(|z| format!("--overlay-z: {z};")),
            PageHeader {
                title: TITLE,
                leading: rsx! {
                    BackArrow { class: "page-header-support", onclick: move |()| on_close.call(()) }
                },
            }
            div {
                class: "screen-content web-deck-cards-view",
                "data-settle-scope": "",
                // Any tap while touch stacks show can reveal a card or close one.
                "data-settle": (stacks_on && touch).then_some(""),
                // A tap off every card closes the reveal; a card's own tap stops at the card.
                onclick: move |_| {
                    if touch {
                        details.dismiss();
                    }
                },
                div { class: "web-deck-cards-controls",
                    {head}
                    {controls}
                }
                div { class: "web-deck-cards-list",
                    if stacks_on {
                        DeckStacks {
                            // A change of input lays the stacks out again for it.
                            key: "{touch}",
                            sections,
                            currency,
                            collapsed,
                            details,
                            faces,
                            group_buy,
                            dealing,
                            touch,
                        }
                    } else {
                        for (section, rows) in sections {
                            CardGroup {
                                key: "{section.key}",
                                title: section.title.clone(),
                                pips: section.pips.clone(),
                                expanded: !collapsed.read().contains(&section.key),
                                ontoggle: {
                                    let key = section.key.clone();
                                    move |()| {
                                        let mut set = folds.write();
                                        if !set.remove(&key) {
                                            set.insert(key.clone());
                                        }
                                    }
                                },
                                trailing: group_tail(TailPlace::List, section.title, rows.clone(), currency, group_buy),
                                for row in rows {
                                    {
                                        let card_id = row.card.scryfall_data.id;
                                        let board = boards.get(&card_id).copied().unwrap_or(Board::Deck);
                                        rsx! {
                                            DeckRow {
                                                key: "{card_id}",
                                                row,
                                                board,
                                                currency,
                                                selected,
                                                face,
                                                faces,
                                                show_art: Some(show_art),
                                                actions,
                                                host,
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if empty {
                        p { class: "deck-empty",
                            span { class: "chip-note", "No cards" }
                        }
                    }
                }
            }
            div { class: "action-dock action-dock-right", {dock} }
        }
    }
}

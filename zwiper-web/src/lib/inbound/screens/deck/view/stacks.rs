//! The deck page's Stacks view: each section as a [`CardStack`] under the
//! decks list's own group chip ([`DeckGroupSection`]: the group's pips,
//! name and count, folding it, its total price in a money chip nested at
//! its end), the cards floating on the page's grid under it, the stacks
//! flowing into as many columns as the page fits. Hover
//! reveals a card in place; a click opens its details in the card details
//! drawer with the actions its row has, and a click on another card swaps
//! it in. The reveal and open state is zwipe-ui's [`Reveal`](zwipe_ui::deck_cards::stacks::Reveal).

use std::collections::{HashMap, HashSet};

use components::{CardBoardMoves, CardEditButtons, CardStack, CartButton, DeckSection, StackCard};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{Board, deck_metrics};
use zwipe_ui::deck_cards::stacks::Gesture;

use super::group_buy::{GroupBuy, group_cards};
use crate::inbound::cards::card_info::CardDetailsSheet;
use crate::inbound::cards::card_row::CardOverlays;
use crate::inbound::cards::stack_view::{DRAWER_CLASSES, StackDetails};
use crate::inbound::screens::deck::cards::editor::{Editor, Meta, RowInfo, RowKind, board_of};
use crate::inbound::screens::deck::list::DeckGroupSection;

/// The total of `rows` in `currency`, each card's price times its copies;
/// `None` when no card has a price in it.
fn total_price(rows: &[RowInfo], currency: PriceCurrency) -> Option<String> {
    let mut priced = false;
    let mut sum = 0.0_f64;
    for row in rows {
        if let Some(each) = deck_metrics::card_price(&row.card.scryfall_data, currency) {
            priced = true;
            // A row holds at least one copy (`Quantity` is positive; a zone or token row is 1).
            sum += each * f64::from(row.qty.max(1));
        }
    }
    priced.then(|| currency.format_amount(sum))
}

/// The stacks for `sections`, folding by `collapsed` as the bands do, each card on its face in
/// `faces` (the front where it has none), its header's cart opening `group_buy` on its cards.
pub(crate) fn stacks_view(
    sections: Vec<(DeckSection, Vec<RowInfo>)>,
    currency: PriceCurrency,
    collapsed: Signal<HashSet<String>>,
    details: StackDetails,
    faces: Signal<HashMap<Uuid, usize>>,
    group_buy: GroupBuy,
) -> Element {
    let mut reveal = details.reveal;
    let shown = reveal();
    let shown_faces = faces();
    rsx! {
        div { class: "deck-stacks",
            for (section, rows) in sections {
                {
                    let price = total_price(&rows, currency);
                    let buyable = !group_cards(&rows).is_empty();
                    let group = section.title.clone();
                    let for_buy = rows.clone();
                    let ids: Vec<Uuid> = rows.iter().map(|r| r.card.scryfall_data.id).collect();
                    let cards: Vec<StackCard> = rows
                        .into_iter()
                        .map(|row| {
                            let face = shown_faces.get(&row.card.scryfall_data.id).copied().unwrap_or(0);
                            StackCard { card: row.card, qty: row.qty, face }
                        })
                        .collect();
                    rsx! {
                        div { key: "{section.key}", class: "deck-stack",
                            DeckGroupSection {
                                group_key: section.key.clone(),
                                header: section.title,
                                pips: section.pips,
                                count: None,
                                collapsed,
                                trailing: (price.is_some() || buyable).then(|| rsx! {
                                    span { class: "deck-stack-tail",
                                        if let Some(p) = price {
                                            span { class: "stat-chip stat-chip-money", "{p}" }
                                        }
                                        if buyable {
                                            CartButton { onclick: move |_| group_buy.offer(&group, &for_buy) }
                                        }
                                    }
                                }),
                                CardStack {
                                    cards,
                                    revealed: shown.revealed_in(&ids),
                                    open: shown.open(),
                                    on_hover: move |id| reveal.write().apply(Gesture::Hover(id)),
                                    on_leave: move |id| reveal.write().apply(Gesture::Leave(id)),
                                    on_select: move |id| details.click(id),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The open card's details drawer, from `rows` (every listed row), with
/// the actions its kind allows: Image (the screen's image preview in
/// `overlays`, as a list row's Image opens it) and Printings in the panel's
/// foot, then the
/// quantity stepper and the star, then the board moves. It shows the
/// [`Reveal::drawn`](zwipe_ui::deck_cards::stacks::Reveal::drawn) card and is mounted, closed, on the first row before
/// any click; nothing when the list is empty.
pub(crate) fn stack_details(
    ed: &Editor,
    rows: &[RowInfo],
    row_meta: &HashMap<Uuid, Meta>,
    details: StackDetails,
    overlays: CardOverlays,
) -> Element {
    let StackDetails {
        reveal,
        details_open,
    } = details;
    // The sheet stays mounted on a card so it slides in and out on its
    // `show` class: the drawn card, else the first row before any click.
    let drawn = reveal().drawn();
    let Some(row) = rows
        .iter()
        .find(|r| Some(r.card.scryfall_data.id) == drawn)
        .or_else(|| rows.first())
        .cloned()
    else {
        return rsx! {};
    };
    let Editor {
        mut printing,
        mut printing_open,
        change_quantity,
        move_to_board,
        toggle_mvp,
        mut faces,
        ..
    } = *ed;
    let RowInfo { card, qty, kind } = row;
    let card_id = card.scryfall_data.id;
    let shown_face = faces.read().get(&card_id).copied().unwrap_or(0);
    let print_card = card.clone();
    let image_of = card.scryfall_data.clone();
    let CardOverlays { mut preview, .. } = overlays;
    let on_image =
        EventHandler::new(move |face: usize| preview.set(Some((image_of.clone(), face))));
    let slot = match kind {
        RowKind::Zone(slot) => Some(slot),
        _ => None,
    };
    let on_printings = (kind != RowKind::Token).then(|| {
        EventHandler::new(move |()| {
            printing.set(Some((print_card.clone(), slot)));
            printing_open.set(true);
        })
    });
    let board = match kind {
        RowKind::Board(b) => Some(b),
        RowKind::Main { .. } => Some(board_of(row_meta, card_id)),
        RowKind::Zone(_) | RowKind::Token => None,
    };
    let mvp = match kind {
        RowKind::Main { mvp } => Some(mvp),
        _ => None,
    };
    let editable = matches!(kind, RowKind::Main { .. } | RowKind::Board(_));
    let footer = editable.then(|| {
        rsx! {
            // Plain action rows, as the panel's own foot lays out Printings, so
            // every row starts at the same left edge.
            div { class: "card-action-row",
                CardEditButtons {
                    qty,
                    on_qty_change: move |d| change_quantity.call((card_id, d, /* record */ true)),
                    mvp,
                    on_toggle_mvp: move |()| {
                        if let Some(is_mvp) = mvp {
                            toggle_mvp.call((card_id, is_mvp));
                        }
                    },
                }
            }
            CardBoardMoves {
                current_board: board,
                on_move_to: move |to: Board| move_to_board.call((card_id, to, /* record */ true)),
            }
        }
    });
    rsx! {
        CardDetailsSheet {
            open: details_open,
            card,
            on_image,
            on_printings,
            footer,
            class: DRAWER_CLASSES,
            face: Some(shown_face),
            on_face_change: move |face: usize| {
                faces.write().insert(card_id, face);
            },
        }
    }
}

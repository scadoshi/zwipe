//! The deck page's Stacks view: each section as a [`CardStack`] under the
//! decks list's own group chip ([`DeckGroupSection`]: the group's pips,
//! name and count, folding it, its total price in a money chip nested at
//! its end), the cards floating on the page's grid under it, the stacks
//! flowing into as many columns as the page fits. Hover
//! reveals a card in place; a click opens its details in the card details
//! drawer with the actions its row has, and a click on another card swaps
//! it in. The reveal and open state is zwipe-ui's [`Reveal`].

use std::collections::{HashMap, HashSet};

use components::{CardBoardMoves, CardEditButtons, CardStack, DeckSection, StackCard};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{Board, deck_metrics};
use zwipe_ui::deck_cards::stacks::{Gesture, Reveal};

use crate::inbound::cards::card_info::CardDetailsSheet;
use crate::inbound::cards::card_row::CardOverlays;
use crate::inbound::screens::deck::cards::editor::{Editor, Meta, RowInfo, RowKind};
use crate::inbound::screens::deck::list::DeckGroupSection;

/// The total of `rows` in `currency`, each card's price times its copies;
/// `None` when no card has a price in it.
fn total_price(rows: &[RowInfo], currency: PriceCurrency) -> Option<String> {
    let mut priced = false;
    let mut sum = 0.0_f64;
    for row in rows {
        if let Some(each) = deck_metrics::card_price(&row.card.scryfall_data, currency) {
            priced = true;
            sum += each * f64::from(row.qty.max(1));
        }
    }
    priced.then(|| currency.format_amount(sum))
}

/// The stacks' reveal and whether the open card's drawer shows, which a click on a card and the
/// deck cards screen's `d` both move. The page provides it as context, ahead of the deck editor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct StackDetails {
    /// Which card shows in full and which card's details are open.
    pub(crate) reveal: Signal<Reveal>,
    /// Whether the open card's details drawer shows.
    pub(crate) details_open: Signal<bool>,
}

impl StackDetails {
    /// A click on card `id`: its details open, in place of any open one, or close where they are
    /// open.
    pub(crate) fn click(self, id: Uuid) {
        let Self {
            mut reveal,
            mut details_open,
        } = self;
        reveal.write().apply(Gesture::Click(id));
        details_open.set(reveal.peek().open().is_some());
    }
}

/// The stacks for `sections`, folding by `collapsed` as the bands do, each card on its face in
/// `faces` (the front where it has none).
pub(crate) fn stacks_view(
    sections: &[(DeckSection, Vec<RowInfo>)],
    currency: PriceCurrency,
    collapsed: Signal<HashSet<String>>,
    details: StackDetails,
    faces: Signal<HashMap<Uuid, usize>>,
) -> Element {
    let mut reveal = details.reveal;
    let shown = reveal();
    let shown_faces = faces();
    rsx! {
        div { class: "deck-stacks",
            for (section, rows) in sections.iter().cloned() {
                {
                    let price = total_price(&rows, currency);
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
                                trailing: price.map(|p| rsx! { span { class: "stat-chip stat-chip-money", "{p}" } }),
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
/// [`Reveal::drawn`] card and is mounted, closed, on the first row before
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
        RowKind::Main { .. } => Some(row_meta.get(&card_id).map_or(Board::Deck, |m| m.board)),
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
                    on_qty_change: move |d| change_quantity.call((card_id, d, true)),
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
                on_move_to: move |to: Board| move_to_board.call((card_id, to, true)),
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

/// The details drawer's classes: the card details drawer's own, then
/// [`STACK_DETAILS_CLASS`](crate::inbound::chrome::shortcuts::STACK_DETAILS_CLASS), so
/// the deck cards screen's hover keys reach the screen through it.
const DRAWER_CLASSES: &str = "card-details-drawer deck-stack-details";

/// Drops focus from the stack card the drawer opened from, so its focus
/// outline goes with the drawer.
const BLUR_STACK_CARD_JS: &str = "if (document.activeElement?.classList.contains('card-stack-card')) document.activeElement.blur();";

/// Keeps the drawer and the reveal state agreeing: a drawer closed by its
/// ×, Escape, the backdrop or the back gesture closes the open card and
/// clears its outline, and an open card that left the list (its last copy
/// removed) closes the drawer.
pub(crate) fn use_stack_details_sync(
    mut reveal: Signal<Reveal>,
    mut details_open: Signal<bool>,
    listed: Memo<Vec<Uuid>>,
) {
    use_effect(move || {
        let open = details_open();
        let card = reveal.peek().open();
        if !open && card.is_some() {
            reveal.write().apply(Gesture::Close);
            document::eval(BLUR_STACK_CARD_JS);
        }
        if open && card.is_some_and(|id| !listed().contains(&id)) {
            details_open.set(false);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbound::chrome::shortcuts::STACK_DETAILS_CLASS;

    #[test]
    fn the_drawer_names_the_class_the_keys_pass_through() {
        let classes: Vec<&str> = DRAWER_CLASSES.split_whitespace().collect();
        assert!(classes.contains(&"card-details-drawer"));
        assert!(classes.contains(&STACK_DETAILS_CLASS));
    }
}

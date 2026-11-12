//! The deck page's cards, shared by the owner's deck page and a shared deck's: List (each group
//! a [`DeckBand`] of [`DeckRow`]s) or Stacks (each group a [`CardStack`] under its
//! [`DeckGroupSection`] chip, a click opening the card in the [`DeckStackDrawer`]).
//!
//! The edit controls (the quantity stepper, Star and the board moves) show only with
//! [`DeckEditActions`]; a shared deck passes none and shows only what reads. Printings and the
//! image preview are the host's, through callbacks, so this module never calls the API.

use std::collections::{HashMap, HashSet};

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::Board;
use zwipe_ui::buy::{group_cards, group_price};
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck_cards::columns;
use zwipe_ui::deck_cards::keys::{RowInfo, RowKind};
use zwipe_ui::deck_cards::stacks::{CardsView, Gesture, Reveal};

use crate::{
    CardBoardMoves, CardDetailsSheet, CardEditButtons, CardRow, CardStack, CartButton, DeckBand,
    DeckGroupSection, DeckSection, GroupBuy, StackCard,
};

/// The deck's edits, each taking the card's id: a quantity change by a step, a move to a board,
/// and a star toggle from the card's current star. The `bool` in the first two records the edit
/// for undo.
#[derive(Clone, Copy, PartialEq)]
#[expect(
    missing_debug_implementations,
    reason = "callbacks, which have no Debug"
)]
pub struct DeckEditActions {
    /// Changes the card's quantity by the step.
    pub change_quantity: Callback<(Uuid, i32, bool)>,
    /// Moves the card to the board.
    pub move_to_board: Callback<(Uuid, Board, bool)>,
    /// Stars the card, or unstars it when the `bool` says it is starred.
    pub toggle_mvp: Callback<(Uuid, bool)>,
}

/// Opens a card's printings: the card, and the command zone slot it sits in, if any.
pub type OnPrinting = Callback<(Card, Option<Slot>)>;

/// What the host does for the cards' buttons, each left out where the host has none: the image
/// preview (the card's data and shown face), the printings a card may save (`on_printing`, not a
/// token's), the printings only viewed (`on_view_printing`, for a row without the first), and an
/// Oracle tag's description and examples.
#[derive(Clone, Copy, PartialEq, Default)]
#[expect(
    missing_debug_implementations,
    reason = "callbacks, which have no Debug"
)]
pub struct DeckRowHost {
    /// Opens the image preview on the card's data and shown face.
    pub on_image: Option<Callback<(ScryfallData, usize)>>,
    /// Opens the printings of a card that may save one.
    pub on_printing: Option<OnPrinting>,
    /// Opens the printings of a list row without `on_printing`, to view.
    pub on_view_printing: Option<Callback<Card>>,
    /// An Oracle tag's description.
    pub describe_tag: Option<Callback<String, Option<String>>>,
    /// Opens an Oracle tag's example cards.
    pub on_examples: Option<Callback<String>>,
}

/// The stacks' reveal and whether the open card's drawer shows, which a click on a card and the
/// `d` key both move. A screen with stacks provides it as context.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StackDetails {
    /// Which card shows in full and which card's details are open.
    pub reveal: Signal<Reveal>,
    /// Whether the open card's details drawer shows.
    pub details_open: Signal<bool>,
}

impl StackDetails {
    /// A click on card `id`: its details open, in place of any open one, or close where they are
    /// open.
    pub fn click(self, id: Uuid) {
        let Self {
            mut reveal,
            mut details_open,
        } = self;
        reveal.write().apply(Gesture::Click(id));
        details_open.set(reveal.peek().open().is_some());
    }
}

/// The details drawer's classes: the card details drawer's own, then `deck-stack-details`, the
/// class a card screen's keys pass through.
pub const DRAWER_CLASSES: &str = "card-details-drawer deck-stack-details";

/// Drops focus from the stack card the drawer opened from, so its focus
/// outline goes with the drawer.
const BLUR_STACK_CARD_JS: &str = "if (document.activeElement?.classList.contains('card-stack-card')) document.activeElement.blur();";

/// Keeps the drawer and the reveal state agreeing: a drawer closed by its
/// ×, Escape, the backdrop or the back gesture closes the open card and
/// clears its outline, and an open card that left the list closes the
/// drawer.
pub fn use_stack_details_sync(
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

/// The deck's cards in `view`: List bands of rows, in two columns ([`columns::split_at`]), or
/// Stacks, each group folding by `collapsed` and its cart opening `group_buy` on its cards. `boards` holds each listed card's board (the
/// main board for one it lacks), `selected` the open row, `face` the open row's face and `faces`
/// the face each card shows, and `show_art` whether rows show their art, read by each row so a
/// toggle renders the rows alone. While `dealing` every stack deals its cards in.
#[component]
pub fn DeckCardsView(
    sections: Vec<(DeckSection, Vec<RowInfo>)>,
    view: CardsView,
    currency: PriceCurrency,
    collapsed: Signal<HashSet<String>>,
    details: StackDetails,
    selected: Signal<Option<Uuid>>,
    face: Signal<usize>,
    faces: Signal<HashMap<Uuid, usize>>,
    boards: HashMap<Uuid, Board>,
    group_buy: GroupBuy,
    #[props(default)] dealing: bool,
    #[props(default)] show_art: Option<Signal<bool>>,
    actions: Option<DeckEditActions>,
    host: DeckRowHost,
) -> Element {
    if view == CardsView::Stacks {
        return rsx! {
            DeckStacks { sections, currency, collapsed, details, faces, group_buy, dealing }
        };
    }
    let row_counts: Vec<usize> = sections.iter().map(|(_, rows)| rows.len()).collect();
    let mut left = sections;
    let right = left.split_off(columns::split_at(&row_counts));
    let band = |(section, rows): (DeckSection, Vec<RowInfo>)| {
        rsx! {
            DeckBand {
                key: "{section.key}",
                accessory: (!group_cards(&rows).is_empty()).then(|| {
                    let header = section.title.clone();
                    let for_buy = rows.clone();
                    rsx! {
                        CartButton { onclick: move |_| group_buy.offer(&header, &for_buy) }
                    }
                }),
                section,
                folded: collapsed,
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
                                show_art,
                                actions,
                                host,
                            }
                        }
                    }
                }
            }
        }
    };
    // Two plain columns, laid out as one flow until the page is wide enough to pair them.
    rsx! {
        div { class: "deck-list-columns",
            div { class: "deck-list-column",
                for group in left {
                    {band(group)}
                }
            }
            div { class: "deck-list-column",
                for group in right {
                    {band(group)}
                }
            }
        }
    }
}

/// The editing actions a row passes to its [`CardRow`], by its kind.
#[derive(Default)]
struct RowActions {
    on_qty: Option<EventHandler<i32>>,
    on_move: Option<EventHandler<Board>>,
    board: Option<Board>,
    mvp: Option<bool>,
    on_star: Option<EventHandler<()>>,
}

/// The edits a row of `kind` on `board` offers with `actions`: a main or other board row's
/// quantity and move, and a main row's star; nothing for a zone or token row, or without actions.
fn row_actions(
    actions: Option<DeckEditActions>,
    card_id: Uuid,
    kind: RowKind,
    board: Board,
) -> RowActions {
    let Some(DeckEditActions {
        change_quantity,
        move_to_board,
        toggle_mvp,
    }) = actions
    else {
        return RowActions::default();
    };
    let on_qty = Some(EventHandler::new(move |d| {
        change_quantity.call((card_id, d, /* record */ true));
    }));
    let on_move = Some(EventHandler::new(move |b| {
        move_to_board.call((card_id, b, /* record */ true));
    }));
    match kind {
        RowKind::Main { mvp } => RowActions {
            on_qty,
            on_move,
            board: Some(board),
            mvp: Some(mvp),
            on_star: Some(EventHandler::new(move |()| toggle_mvp.call((card_id, mvp)))),
        },
        RowKind::Board(b) => RowActions {
            on_qty,
            on_move,
            board: Some(b),
            ..RowActions::default()
        },
        RowKind::Zone(_) | RowKind::Token => RowActions::default(),
    }
}

/// One listed row as a selectable [`CardRow`], with the edits its kind allows when the host
/// passes `actions`, and the host's buttons from `host`. It reads whether
/// it is the open row and the face it shows through memos of its own, so a change to one row
/// renders that row alone. A click on it shows its front.
#[component]
pub fn DeckRow(
    row: RowInfo,
    board: Board,
    currency: PriceCurrency,
    selected: Signal<Option<Uuid>>,
    mut face: Signal<usize>,
    mut faces: Signal<HashMap<Uuid, usize>>,
    #[props(default)] show_art: Option<Signal<bool>>,
    actions: Option<DeckEditActions>,
    #[props(default)] host: DeckRowHost,
) -> Element {
    let DeckRowHost {
        on_image,
        on_printing,
        on_view_printing,
        describe_tag,
        on_examples,
    } = host;
    let card_id = row.card.scryfall_data.id;
    let shown_face = use_memo(use_reactive!(|card_id| faces
        .read()
        .get(&card_id)
        .copied()
        .unwrap_or(0)))();
    let is_selected = use_memo(use_reactive!(|card_id| selected() == Some(card_id)))();
    let RowInfo { card, qty, kind } = row;
    let RowActions {
        on_qty,
        on_move,
        board,
        mvp,
        on_star,
    } = row_actions(actions, card_id, kind, board);
    let slot = match kind {
        RowKind::Zone(slot) => Some(slot),
        _ => None,
    };
    let on_print = on_printing
        .filter(|_| kind != RowKind::Token)
        .map(|open| EventHandler::new(move |c: Card| open.call((c, slot))))
        .or_else(|| on_view_printing.map(|open| EventHandler::new(move |c: Card| open.call(c))));
    let on_image = on_image.map(|open| {
        let image_of = card.scryfall_data.clone();
        EventHandler::new(move |()| open.call((image_of.clone(), shown_face)))
    });
    rsx! {
        div {
            class: if is_selected { "web-deck-row selected" } else { "web-deck-row" },
            "data-row": "{card_id}",
            onclick: move |_| face.set(0),
            div { class: "web-card-row", "data-row": "{card_id}",
                CardRow {
                    card,
                    qty,
                    expanded_card: selected,
                    show_art: show_art.map(|on| on()),
                    on_image,
                    on_face_change: move |next: usize| {
                        faces.write().insert(card_id, next);
                    },
                    face: Some(shown_face),
                    on_qty_change: on_qty,
                    on_move_to: on_move,
                    current_board: board,
                    on_printing: on_print,
                    mvp,
                    on_toggle_mvp: on_star,
                    price_currency: currency,
                    show_classification: true,
                    describe_tag,
                    on_examples,
                }
            }
        }
    }
}

/// The stacks for `sections`, folding by `collapsed` as the bands do, each card on its face in
/// `faces` (the front where it has none), its header showing the group's total price and its cart
/// opening `group_buy` on its cards. While `dealing` every stack deals its cards in, top to
/// bottom. Hover reveals a card in place; a click opens it in the drawer.
#[component]
pub fn DeckStacks(
    sections: Vec<(DeckSection, Vec<RowInfo>)>,
    currency: PriceCurrency,
    collapsed: Signal<HashSet<String>>,
    details: StackDetails,
    faces: Signal<HashMap<Uuid, usize>>,
    group_buy: GroupBuy,
    #[props(default)] dealing: bool,
) -> Element {
    let mut reveal = details.reveal;
    let shown = reveal();
    let shown_faces = faces();
    rsx! {
        div { class: "deck-stacks",
            for (section, rows) in sections {
                {
                    let price = group_price(&rows, currency);
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
                        div { key: "{section.key}", class: "deck-stack", "data-fold": "{section.key}",
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
                                    dealing,
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

/// The open stack card's details drawer, from `rows` (every listed row): Image and Printings
/// (`host`'s, the saving printings not on a token, else the viewing ones) in the panel's foot,
/// then, with `actions`, the quantity stepper and the star, then the board moves. `boards`
/// holds each card's board, the main board for one it lacks. It shows the
/// [`Reveal::drawn`] card and is mounted, closed, on the first row before any click, so it slides
/// in and out; nothing when the list is empty.
#[component]
pub fn DeckStackDrawer(
    rows: Vec<RowInfo>,
    boards: HashMap<Uuid, Board>,
    details: StackDetails,
    mut faces: Signal<HashMap<Uuid, usize>>,
    actions: Option<DeckEditActions>,
    #[props(default)] host: DeckRowHost,
) -> Element {
    let DeckRowHost {
        on_image,
        on_printing,
        on_view_printing,
        describe_tag,
        ..
    } = host;
    let StackDetails {
        reveal,
        details_open,
    } = details;
    let drawn = reveal().drawn();
    let Some(row) = rows
        .iter()
        .find(|r| Some(r.card.scryfall_data.id) == drawn)
        .or_else(|| rows.first())
        .cloned()
    else {
        return rsx! {};
    };
    let RowInfo { card, qty, kind } = row;
    let card_id = card.scryfall_data.id;
    let shown_face = faces.read().get(&card_id).copied().unwrap_or(0);
    let on_image = on_image.map(|open| {
        let image_of = card.scryfall_data.clone();
        EventHandler::new(move |face: usize| open.call((image_of.clone(), face)))
    });
    let slot = match kind {
        RowKind::Zone(slot) => Some(slot),
        _ => None,
    };
    let print_card = card.clone();
    let on_printings = on_printing
        .filter(|_| kind != RowKind::Token)
        .map(|open| {
            let print_card = print_card.clone();
            EventHandler::new(move |()| open.call((print_card.clone(), slot)))
        })
        .or_else(|| {
            on_view_printing.map(|open| EventHandler::new(move |()| open.call(print_card.clone())))
        });
    let board = boards.get(&card_id).copied().unwrap_or(Board::Deck);
    let RowActions {
        on_qty,
        on_move,
        board,
        mvp,
        on_star,
    } = row_actions(actions, card_id, kind, board);
    let footer = on_move.map(|on_move_to| {
        rsx! {
            // Plain action rows, as the panel's own foot lays out Printings, so
            // every row starts at the same left edge.
            div { class: "card-action-row",
                CardEditButtons { qty, on_qty_change: on_qty, mvp, on_toggle_mvp: on_star }
            }
            CardBoardMoves { current_board: board, on_move_to }
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
            describe_tag,
        }
    }
}

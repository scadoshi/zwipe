//! The deck page's cards, shared by the owner's deck page and a shared deck's: List (each group
//! a [`DeckBand`] of [`DeckRow`]s) or Stacks (each group a [`CardStack`] under its
//! [`DeckGroupSection`] chip, a click opening the card in the [`DeckStackDrawer`]).
//!
//! The edit controls (the quantity stepper, Star and the board moves) show only with
//! [`DeckEditActions`]; a shared deck passes none and shows only what reads. Printings and the
//! image preview are the host's, through callbacks, so this module never calls the API.

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{Board, Format};
use zwipe_ui::buy::{group_cards, group_price};
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck_cards::keys::{RowInfo, RowKind};
use zwipe_ui::deck_cards::listing::{Listed, RowMeta, main_row_of};
use zwipe_ui::deck_cards::stacks::{CardsView, Gesture, Reveal, VIEW_LABEL};
use zwipe_ui::deck_cards::{Group, columns};

use crate::{
    CardBoardMoves, CardDetailsSheet, CardEditButtons, CardRow, CardStack, CartButton, Chip,
    DeckBand, DeckGroupSection, DeckSection, GroupBuy, StackCard, use_overlay_back_action,
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

    /// A tap on card `id` (phones): the first reveals it, a second on the revealed card opens its
    /// details, and one on the open card closes them.
    pub fn tap(self, id: Uuid) {
        let Self {
            mut reveal,
            mut details_open,
        } = self;
        reveal.write().apply(Gesture::Tap(id));
        details_open.set(reveal.peek().open().is_some());
    }

    /// A tap off every card (phones): the reveal goes back to the open card, or to none.
    pub fn dismiss(self) {
        let Self { mut reveal, .. } = self;
        if reveal.peek().revealed() != reveal.peek().open() {
            reveal.write().apply(Gesture::Dismiss);
        }
    }
}

/// The deck's sections from `listed`, each with its rows, in the deck page's order: tokens, the
/// maybeboard and sideboard, the commander and partner under the format's header, the background
/// and the signature spell each under its own, then the main board's groups and its lands. A main
/// board header counts copies by `meta`; the others count cards.
#[must_use]
pub fn deck_sections<S: BuildHasher>(
    listed: Listed,
    meta: &HashMap<Uuid, RowMeta, S>,
    format: Option<Format>,
) -> Vec<(DeckSection, Vec<RowInfo>)> {
    let Listed {
        tokens,
        maybe,
        side,
        zone,
        groups,
        lands,
        ..
    } = listed;
    let mut sections: Vec<(DeckSection, Vec<RowInfo>)> = Vec::new();
    if !tokens.is_empty() {
        let rows = tokens
            .iter()
            .map(|card| RowInfo {
                card: card.clone(),
                qty: 1,
                kind: RowKind::Token,
            })
            .collect();
        sections.push((
            DeckSection {
                key: "Tokens".to_owned(),
                title: format!("Tokens ({})", tokens.len()),
                pips: None,
                cards: tokens,
            },
            rows,
        ));
    }
    for (board, title, on_board) in [
        (Board::Maybeboard, "Maybeboard", maybe),
        (Board::Sideboard, "Sideboard", side),
    ] {
        if on_board.is_empty() {
            continue;
        }
        let rows = on_board
            .iter()
            .map(|entry| RowInfo {
                card: entry.card.clone(),
                qty: *entry.deck_card.quantity,
                kind: RowKind::Board(board),
            })
            .collect();
        sections.push((
            DeckSection {
                key: title.to_owned(),
                title: format!("{title} ({})", on_board.len()),
                pips: None,
                cards: on_board.into_iter().map(|e| e.card).collect(),
            },
            rows,
        ));
    }
    sections.extend(zone_sections(zone, format));
    sections.extend(main_sections(groups, lands, meta));
    sections
}

/// The command zone's sections: the commander and partner under the format's header, then the
/// background and the signature spell each under its own.
fn zone_sections(
    zone: Vec<(Slot, Card)>,
    format: Option<Format>,
) -> Vec<(DeckSection, Vec<RowInfo>)> {
    let has_partner = zone.iter().any(|(slot, _)| *slot == Slot::Partner);
    let mut sections: Vec<(DeckSection, Vec<RowInfo>)> = Vec::new();
    let commanders: Vec<(Slot, Card)> = zone
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Commander | Slot::Partner))
        .cloned()
        .collect();
    if !commanders.is_empty() {
        let title = Slot::Commander.group_header(format, has_partner).to_owned();
        sections.push((
            DeckSection {
                key: title.clone(),
                title,
                pips: None,
                cards: commanders.iter().map(|(_, c)| c.clone()).collect(),
            },
            commanders
                .into_iter()
                .map(|(slot, card)| RowInfo {
                    card,
                    qty: 1,
                    kind: RowKind::Zone(slot),
                })
                .collect(),
        ));
    }
    for (slot, card) in zone
        .into_iter()
        .filter(|(slot, _)| matches!(slot, Slot::Background | Slot::SignatureSpell))
    {
        let title = slot.group_header(format, has_partner).to_owned();
        sections.push((
            DeckSection {
                key: title.clone(),
                title,
                pips: None,
                cards: vec![card.clone()],
            },
            vec![RowInfo {
                card,
                qty: 1,
                kind: RowKind::Zone(slot),
            }],
        ));
    }
    sections
}

/// The main board's sections: its nonland groups, then its lands, each header counting copies by
/// `meta`.
fn main_sections<S: BuildHasher>(
    groups: Vec<Group>,
    lands: Vec<Card>,
    meta: &HashMap<Uuid, RowMeta, S>,
) -> Vec<(DeckSection, Vec<RowInfo>)> {
    let qty_count = |cards: &[Card]| -> i32 {
        cards
            .iter()
            .map(|c| meta.get(&c.scryfall_data.id).map_or(1, |m| m.qty))
            .fold(0, i32::saturating_add)
    };
    let mut sections: Vec<(DeckSection, Vec<RowInfo>)> = Vec::new();
    for group in groups {
        let n = qty_count(&group.cards);
        let title = if group.label.is_empty() {
            format!("({n})")
        } else {
            format!("{} ({n})", group.label)
        };
        let rows = group.cards.iter().map(|c| main_row_of(meta, c)).collect();
        sections.push((
            DeckSection {
                key: group.key,
                title,
                pips: group.pips,
                cards: group.cards,
            },
            rows,
        ));
    }
    if !lands.is_empty() {
        let n = qty_count(&lands);
        let rows = lands.iter().map(|c| main_row_of(meta, c)).collect();
        sections.push((
            DeckSection {
                key: "Lands".to_owned(),
                title: format!("Lands ({n})"),
                pips: None,
                cards: lands,
            },
            rows,
        ));
    }
    sections
}

/// The View chip row: List and Stacks, the one in `view` selected. A tap sets `view` and calls
/// `on_change` with it, where the host keeps the choice.
#[component]
pub fn CardsViewSwitch(
    mut view: Signal<CardsView>,
    #[props(default)] on_change: Option<EventHandler<CardsView>>,
) -> Element {
    rsx! {
        div { class: "chip-row deck-view-switch",
            span { class: "stat-chip stat-chip-p1 chip-row-label", "{VIEW_LABEL}" }
            for choice in [CardsView::List, CardsView::Stacks] {
                Chip {
                    key: "{choice.key()}",
                    selected: view() == choice,
                    onclick: move |_| {
                        view.set(choice);
                        if let Some(handler) = on_change {
                            handler.call(choice);
                        }
                    },
                    "{choice.label()}"
                }
            }
        }
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
///
/// With `touch`, for phones, the stacks are in [`CardStack`]'s tap mode: a first tap reveals a
/// card, a second opens it in the drawer, and the OS back clears a revealed card before it leaves
/// the screen. The stacks lie in two columns of their own heights ([`columns::masonry`]), each
/// under the one above it in its column, so a fold or a reveal moves only its own column. A
/// header press calls `on_fold` with the group's key where the host passes it, else folds it in
/// `collapsed`.
#[component]
pub fn DeckStacks(
    sections: Vec<(DeckSection, Vec<RowInfo>)>,
    currency: PriceCurrency,
    collapsed: Signal<HashSet<String>>,
    details: StackDetails,
    faces: Signal<HashMap<Uuid, usize>>,
    group_buy: GroupBuy,
    #[props(default)] dealing: bool,
    #[props(default)] touch: bool,
    #[props(default)] on_fold: Option<EventHandler<String>>,
) -> Element {
    let mut reveal = details.reveal;
    // The open card's sheet takes the back itself; a card only revealed takes it here.
    let revealing = use_memo(move || {
        let shown = reveal();
        touch && shown.revealed().is_some() && shown.open().is_none()
    });
    // Back clears the reveal with no click behind it, so it starts the settle itself and the
    // rows the reveal pushed down slide back up.
    let dismiss = use_callback(move |()| {
        crate::settle::watch();
        details.dismiss();
    });
    use_overlay_back_action(revealing.into(), dismiss);
    let width = use_stacks_width(touch);
    // A fold reweighs the stacks and can move some to another column, so it starts the settle,
    // which slides them there.
    let mut folds = collapsed;
    let on_fold = Some(EventHandler::new(move |key: String| {
        crate::settle::watch();
        if let Some(on_fold) = on_fold {
            on_fold.call(key);
            return;
        }
        let mut set = folds.write();
        if !set.remove(&key) {
            set.insert(key);
        }
    }));
    let on_hover =
        (!touch).then(|| EventHandler::new(move |id| reveal.write().apply(Gesture::Hover(id))));
    let on_leave =
        (!touch).then(|| EventHandler::new(move |id| reveal.write().apply(Gesture::Leave(id))));
    let shown = reveal();
    let shown_faces = faces();
    let folded = collapsed();
    // Each stack with its height as the columns weigh it.
    let stacks: Vec<(u32, Element)> = sections
        .into_iter()
        .map(|(section, rows)| {
            let height = columns::stack_height(rows.len(), folded.contains(&section.key));
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
            let stack = rsx! {
                div { key: "{section.key}", class: "deck-stack", "data-fold": "{section.key}",
                    DeckGroupSection {
                        group_key: section.key.clone(),
                        header: section.title,
                        pips: section.pips,
                        count: None,
                        collapsed,
                        on_fold,
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
                            on_hover,
                            on_leave,
                            on_select: move |id| if touch { details.tap(id) } else { details.click(id) },
                        }
                    }
                }
            };
            (height, stack)
        })
        .collect();
    // Two columns on a phone; on a wide page as many as its width fits, once it is measured.
    let column_total = if touch {
        Some(2)
    } else {
        width().map(columns::column_count)
    };
    let Some(column_total) = column_total else {
        return rsx! {
            div { class: "deck-stacks",
                for (_, stack) in stacks {
                    {stack}
                }
            }
        };
    };
    // Columns of their own heights, each stack under the one above it in its column.
    let heights: Vec<u32> = stacks.iter().map(|(height, _)| *height).collect();
    let mut laid: Vec<Vec<Element>> = vec![Vec::new(); column_total.max(1)];
    for (column, (_, stack)) in columns::place(&heights, column_total)
        .into_iter()
        .zip(stacks)
    {
        if let Some(held) = laid.get_mut(column) {
            held.push(stack);
        }
    }
    rsx! {
        div { class: "deck-stacks deck-stacks-columns",
            for (at , held) in laid.into_iter().enumerate() {
                div { key: "{at}", class: "deck-stacks-column",
                    for stack in held {
                        {stack}
                    }
                }
            }
        }
    }
}

/// The width the stacks last measured, in hundredths of a rem, so stacks mounted again lay out
/// their columns from the first render.
static LAST_WIDTH: GlobalSignal<Option<u32>> = Signal::global(|| None);

/// The width the stacks have on the page, in hundredths of a rem, kept current as it changes:
/// the content box of the element the stacks sit in, which stays put while the stacks swap
/// their grid for columns. `None` until it is first measured, and off the browser.
fn use_stacks_width(touch: bool) -> Signal<Option<u32>> {
    let mut width = use_signal(|| *LAST_WIDTH.peek());
    use_effect(move || {
        if touch {
            return;
        }
        spawn(async move {
            let mut sizes = document::eval(STACKS_WIDTH_JS);
            while let Ok(now) = sizes.recv::<u32>().await {
                if *width.peek() != Some(now) {
                    width.set(Some(now));
                    *LAST_WIDTH.write() = Some(now);
                }
            }
        });
    });
    use_drop(move || {
        if !touch {
            document::eval(STACKS_WIDTH_DROP_JS);
        }
    });
    width
}

/// Sends the width of the element the stacks sit in, in hundredths of a rem, then again on each
/// change.
const STACKS_WIDTH_JS: &str = r"
const stacks = document.querySelector('.deck-stacks');
const frame = stacks && stacks.parentElement;
if (frame) {
  const send = (px) => {
    const rem = parseFloat(getComputedStyle(document.documentElement).fontSize) || 16;
    dioxus.send(Math.floor(px / rem * 100));
  };
  const sizes = new ResizeObserver((entries) => { for (const e of entries) send(e.contentRect.width); });
  sizes.observe(frame);
  window.__zwStacksWidth = sizes;
}
";

/// Stops measuring the stacks' width.
const STACKS_WIDTH_DROP_JS: &str = r"
if (window.__zwStacksWidth) { window.__zwStacksWidth.disconnect(); delete window.__zwStacksWidth; }
";

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

#[cfg(test)]
mod tests {
    use dioxus::dioxus_core::NoOpMutations;

    use super::*;
    use crate::{DeckSection, use_group_buy};

    /// A real card, as the API serves it.
    const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

    /// A group under `key` of `n` main board copies of the card, each its own card.
    fn group(key: &str, n: u128, base: u128) -> (DeckSection, Vec<RowInfo>) {
        let rows: Vec<RowInfo> = (0..n)
            .map(|i| {
                let mut card: Card = serde_json::from_str(CARD_JSON).unwrap();
                card.scryfall_data.id = Uuid::from_u128(base.saturating_add(i));
                RowInfo {
                    card,
                    qty: 1,
                    kind: RowKind::Main { mvp: false },
                }
            })
            .collect();
        let section = DeckSection {
            key: key.to_owned(),
            title: key.to_owned(),
            pips: None,
            cards: rows.iter().map(|r| r.card.clone()).collect(),
        };
        (section, rows)
    }

    /// Pointer stacks on a page 40rem across, a width measured before they mount.
    fn app() -> Element {
        use_hook(|| *LAST_WIDTH.write() = Some(4000));
        let details = StackDetails {
            reveal: use_signal(Reveal::default),
            details_open: use_signal(|| false),
        };
        rsx! {
            DeckStacks {
                sections: vec![group("Creatures", 30, 100), group("Lands", 1, 200), group("Artifacts", 1, 300), group("Enchantments", 1, 400), group("Instants", 1, 500)],
                currency: PriceCurrency::default(),
                collapsed: use_signal(HashSet::new),
                details,
                faces: use_signal(HashMap::new),
                group_buy: use_group_buy(),
            }
        }
    }

    #[test]
    fn pointer_stacks_fill_as_many_columns_as_the_width_holds_each_under_the_shortest() {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        for _ in 0..4 {
            dom.process_events();
            dom.render_immediate(&mut NoOpMutations);
        }
        let html = dioxus_ssr::render(&dom);
        let laid: Vec<Vec<&str>> = html
            .split(r#"class="deck-stacks-column""#)
            .skip(1)
            .map(|column| {
                column
                    .split(r#"data-fold=""#)
                    .skip(1)
                    .filter_map(|rest| rest.split_once('"').map(|(key, _)| key))
                    .collect()
            })
            .collect();
        assert_eq!(
            columns::column_count(4000),
            3,
            "40rem holds three 12.5rem columns"
        );
        assert_eq!(
            laid,
            [
                vec!["Creatures"],
                vec!["Lands", "Enchantments"],
                vec!["Artifacts", "Instants"]
            ],
            "{html}"
        );
    }

    #[test]
    fn the_stacks_measure_the_content_box_they_sit_in_in_hundredths_of_a_rem() {
        for part in [
            "querySelector('.deck-stacks')",
            "parentElement",
            "new ResizeObserver",
            "contentRect.width",
            "/ rem * 100",
        ] {
            assert!(
                STACKS_WIDTH_JS.contains(part),
                "{part} in {STACKS_WIDTH_JS}"
            );
        }
        assert!(STACKS_WIDTH_DROP_JS.contains("disconnect()"));
    }
}

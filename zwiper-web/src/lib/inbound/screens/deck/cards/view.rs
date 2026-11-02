//! The deck cards view: every card in the deck, to edit.
//!
//! At three columns the controls (the deck's header, the featured cards,
//! Quick add, Group by, Boards and Show) sit on the left, the card groups in
//! the middle, and the selected row's image, details and actions on the
//! right. Between the breakpoints the controls sit above the groups. A click
//! selects a row into the pane instead of opening it inline; ↑ and ↓ move
//! the selection, `+` and `-` change its quantity, F flips it, `/` focuses
//! Quick add and Ctrl+Z is `Undo`. A row dragged onto a board's header or
//! chip moves there. Below the breakpoint it is the app's list, rows
//! opening inline.

use std::collections::{HashMap, HashSet};

use chrono::Utc;
use components::{ActionBar, Button, ButtonVariant, CardDetails, Chip, TOAST_QUICK};
use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::deck_metrics::deck_price;
use zwipe_core::domain::deck::{Board, DeckEntry, DeckProfile, Quantity, deck_tag_label};
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpPatchDeckCard};
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck_cards::{self, Boards, Bursts, Write, Zone};
use zwipe_ui::deck_signals::{Crossing, budget_crossing, land_count, land_crossing};
use zwipe_ui::filter;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::filter::picker::Sources;
use zwipe_ui::stores::filters::FilterScope;
use zwipe_ui::stores::undo::{UndoAction, create_board_field};
use zwipe_ui::timing::QUANTITY_FLUSH;

use super::featured::FeaturedCards;
use super::quick_add::{QUICK_ADD_ID, QuickAdd};
use super::signal_toast;
use crate::inbound::card_filter::CardFilterPanel;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_row::{
    CardOverlays, CardOverlaysHost, CardRow, ShowRowArt, use_card_overlays,
};
use crate::inbound::cards::otag::{OtagExamplesHost, OtagExamplesOpen, use_otag_reveals};
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::cards::swipe_stack::{Swipe, use_swipe_key_handler};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, open_and_record_hint};
use crate::inbound::chrome::page_keys::{focus_by_id, use_page_keys, use_undo_key};
use crate::inbound::router::Route;
use crate::inbound::screens::deck::skeletons::ChipRowSkeleton;
use crate::inbound::screens::deck::view::cards::zone as zone_of;
use crate::inbound::shell::ScreenFrame;
use crate::outbound::timer;

/// What a row is, which decides its actions.
#[derive(Debug, Clone, Copy, PartialEq)]
enum RowKind {
    /// A main board card: quantity, moves, printing and the MVP star.
    Main {
        /// Whether it is starred.
        mvp: bool,
    },
    /// A maybeboard or sideboard card: quantity, moves and printing.
    Board(Board),
    /// A command zone card: printing only.
    Zone(Slot),
    /// A token: no actions.
    Token,
}

/// One listed row, for the detail pane and the keys.
#[derive(Debug, Clone, PartialEq)]
struct RowInfo {
    card: Card,
    qty: i32,
    kind: RowKind,
}

/// A row's quantity, board and star, the first entry for a printing winning.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Meta {
    qty: i32,
    board: Board,
    mvp: bool,
}

/// The editing actions a row passes to its [`CardRow`], by its kind.
#[derive(Default)]
struct RowActions {
    on_qty: Option<EventHandler<i32>>,
    on_move: Option<EventHandler<Board>>,
    board: Option<Board>,
    mvp: Option<bool>,
    on_star: Option<EventHandler<()>>,
    on_print: Option<EventHandler<Card>>,
    drag: Option<EventHandler<()>>,
}

/// What the view lists, in order, after the pass and the Boards and Show
/// rows; `rows` flattens it for the pane and the arrow keys.
#[derive(Debug, Clone, PartialEq, Default)]
struct Listed {
    tokens: Vec<Card>,
    maybe: Vec<DeckEntry>,
    side: Vec<DeckEntry>,
    zone: Vec<(Slot, Card)>,
    groups: Vec<deck_cards::Group>,
    lands: Vec<Card>,
    rows: Vec<RowInfo>,
}

/// A main board row, its quantity and star from the deck's entries.
fn main_row_of(meta: &HashMap<Uuid, Meta>, card: &Card) -> RowInfo {
    let m = meta.get(&card.scryfall_data.id).copied();
    RowInfo {
        card: card.clone(),
        qty: m.map_or(1, |m| m.qty),
        kind: RowKind::Main {
            mvp: m.is_some_and(|m| m.mvp),
        },
    }
}

/// The sections the view lists, in the app's order: tokens (sorted, never
/// filtered), the maybeboard and sideboard, the command zone, then the main
/// board's groups and its lands.
fn list_rows(
    shown: &deck_cards::Display,
    meta: &HashMap<Uuid, Meta>,
    tokens: Vec<Card>,
    filter: &CardQueryBuilder,
    boards: Boards,
    show_zone: bool,
) -> Listed {
    let sort = filter.sort().unwrap_or(CardSortKey::Name);
    let tokens: Vec<Card> = Cards::from(tokens).sorted(sort, filter.direction()).into();
    let maybe = if boards.maybe {
        shown.maybe.clone()
    } else {
        Vec::new()
    };
    let side = if boards.side {
        shown.side.clone()
    } else {
        Vec::new()
    };
    let zone = if show_zone {
        shown.zone.clone()
    } else {
        Vec::new()
    };
    let groups = if boards.main {
        shown.groups.clone()
    } else {
        Vec::new()
    };
    let lands = if boards.main {
        shown.lands.clone()
    } else {
        Vec::new()
    };
    let mut rows: Vec<RowInfo> = tokens
        .iter()
        .map(|card| RowInfo {
            card: card.clone(),
            qty: 1,
            kind: RowKind::Token,
        })
        .collect();
    for (entry, board) in maybe
        .iter()
        .map(|e| (e, Board::Maybeboard))
        .chain(side.iter().map(|e| (e, Board::Sideboard)))
    {
        rows.push(RowInfo {
            card: entry.card.clone(),
            qty: *entry.deck_card.quantity,
            kind: RowKind::Board(board),
        });
    }
    let commanders = zone
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Commander | Slot::Partner));
    let others = zone
        .iter()
        .filter(|(slot, _)| !matches!(slot, Slot::Commander | Slot::Partner));
    for (slot, card) in commanders.chain(others) {
        rows.push(RowInfo {
            card: card.clone(),
            qty: 1,
            kind: RowKind::Zone(*slot),
        });
    }
    for card in groups
        .iter()
        .flat_map(|g| g.cards.iter())
        .chain(lands.iter())
    {
        rows.push(main_row_of(meta, card));
    }
    Listed {
        tokens,
        maybe,
        side,
        zone,
        groups,
        lands,
        rows,
    }
}

/// A collapsible card group, its header a drop target for `drop_to` when
/// rows can be dragged onto its board.
#[component]
fn CardGroup(
    title: String,
    section_key: String,
    #[props(default)] pips: Option<Vec<Color>>,
    collapsed: Signal<HashSet<String>>,
    #[props(default)] drop_to: Option<Board>,
    on_drop: EventHandler<Board>,
    children: Element,
) -> Element {
    let open = !collapsed().contains(&section_key);
    let mut over = use_signal(|| false);
    let header_class = match (open, over()) {
        (true, true) => "card-group-header group-collapsible expanded web-drop-over",
        (true, false) => "card-group-header group-collapsible expanded",
        (false, true) => "card-group-header group-collapsible web-drop-over",
        (false, false) => "card-group-header group-collapsible",
    };
    rsx! {
        div { class: if open { "card-group row-enter" } else { "card-group row-enter collapsed" },
            div {
                class: header_class,
                onclick: move |_| {
                    let mut set = collapsed.write();
                    if !set.remove(&section_key) {
                        set.insert(section_key.clone());
                    }
                },
                ondragover: move |e| {
                    if drop_to.is_some() {
                        e.prevent_default();
                        over.set(true);
                    }
                },
                ondragleave: move |_| over.set(false),
                ondrop: move |e| {
                    over.set(false);
                    if let Some(board) = drop_to {
                        e.prevent_default();
                        on_drop.call(board);
                    }
                },
                span { class: "card-row-arrow", "\u{25b8}" }
                if let Some(pips) = pips.as_ref().filter(|p| !p.is_empty()) {
                    span { class: "identity-pips",
                        for color in pips.iter() {
                            i {
                                key: "{color.to_short_name()}",
                                class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
                            }
                        }
                    }
                }
                "{title}"
            }
            div { class: if open { "collapsible open" } else { "collapsible" },
                div { class: "collapsible-inner", {children} }
            }
        }
    }
}

/// The deck cards view for `deck_id`.
#[component]
pub fn ViewDeckCard(deck_id: Uuid) -> Element {
    let navigator = use_navigator();
    let toast = use_toast();
    let authed = use_authed();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let CardStores {
        mut filters,
        mut undo,
        ..
    } = use_context();

    let restored = use_hook(|| filters.peek().restore(FilterScope::Cards, deck_id));
    let restored_active = restored.as_ref().is_some_and(|f| !f.is_empty());
    let mut filter = use_signal(|| restored.clone().unwrap_or_default());
    let mut entries: Signal<Vec<DeckEntry>> = use_signal(Vec::new);
    let mut zone: Signal<Zone> = use_signal(Zone::default);
    let mut zone_cards: Signal<Vec<Card>> = use_signal(Vec::new);
    let mut profile: Signal<Option<DeckProfile>> = use_signal(|| None);
    let mut loaded = use_signal(|| false);
    let mut tokens: Signal<Option<Vec<Card>>> = use_signal(|| None);
    let mut group_by = use_signal(|| GroupByOption::CardType);
    let mut boards = use_signal(Boards::default);
    let mut show_tokens = use_signal(|| false);
    let mut show_zone = use_signal(|| true);
    let mut show_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_art));
    let collapsed: Signal<HashSet<String>> = use_signal(HashSet::new);
    let mut selected: Signal<Option<Uuid>> = use_signal(|| None);
    let mut face = use_signal(|| 0usize);
    let mut bursts = use_signal(Bursts::default);
    let mut removed_rows: Signal<HashMap<Uuid, DeckEntry>> = use_signal(HashMap::new);
    let mut printing: Signal<Option<(Card, Option<Slot>)>> = use_signal(|| None);
    let mut printing_open = use_signal(|| false);
    let mut filter_open = use_signal(|| false);
    let mut dragging: Signal<Option<Uuid>> = use_signal(|| None);
    let hint_open = use_signal(|| false);
    let mut hint_fired = use_signal(|| false);
    let examples = use_otag_reveals();
    let overlays = use_card_overlays();
    let CardOverlays { mut preview, .. } = overlays;
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);

    let currency = move || -> PriceCurrency {
        profile
            .peek()
            .as_ref()
            .and_then(|p| p.price_target_currency)
            .unwrap_or_default()
    };
    let land_target = move || profile.peek().as_ref().and_then(|p| p.land_target);
    let budget = move || profile.peek().as_ref().and_then(|p| p.price_target);

    // Leaving parks the filter and sends every quantity burst still waiting.
    use_drop(move || {
        if let Ok(applied) = filter.try_peek() {
            filters
                .write()
                .park(FilterScope::Cards, deck_id, applied.clone());
        }
        let Ok(mut open) = bursts.try_write() else {
            return;
        };
        for (card_id, burst) in open.drain() {
            spawn_forever(async move {
                match burst.write() {
                    Write::Nothing => {}
                    Write::Delete => {
                        authed
                            .run("change_quantity", |c, s| async move {
                                c.delete_deck_card(deck_id, card_id, &s).await
                            })
                            .await;
                    }
                    Write::Set(quantity) => {
                        let request = HttpPatchDeckCard::new(Some(quantity), None);
                        authed
                            .run("change_quantity", |c, s| async move {
                                c.update_deck_card(deck_id, card_id, &request, &s).await
                            })
                            .await;
                    }
                }
            });
        }
    });

    use_hook(move || {
        spawn(async move {
            let Some(deck) = authed
                .run("load_deck", |c, s| async move {
                    c.fetch_deck(deck_id, &s).await
                })
                .await
            else {
                return;
            };
            let held = zone_of(&deck);
            let mut list = deck.entries.clone();
            held.strip(&mut list);
            zone.set(held);
            zone_cards.set(deck.command_zone_cards.clone());
            profile.set(Some(deck.deck_profile));
            entries.set(list);
            loaded.set(true);
            if restored_active {
                toast.warning(
                    "Filter is active".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
        spawn(async move {
            let found = authed
                .run_quiet("load_tokens", |c, s| async move {
                    c.fetch_deck_tokens(deck_id, &s).await
                })
                .await;
            tokens.set(Some(found.unwrap_or_default()));
        });
    });

    // The hint waits for a deck with cards, so an empty deck keeps it.
    use_effect(move || {
        if !entries.read().is_empty() && !*hint_fired.peek() {
            hint_fired.set(true);
            open_and_record_hint(OneTimeHint::DeckCards, session, client, hint_open);
        }
    });

    let display = use_memo(move || {
        deck_cards::pass(&entries.read(), &zone.read(), &filter.read(), group_by())
    });
    let meta = use_memo(move || {
        let mut map: HashMap<Uuid, Meta> = HashMap::new();
        for e in entries.read().iter() {
            map.entry(e.deck_card.scryfall_data_id)
                .or_insert_with(|| Meta {
                    qty: *e.deck_card.quantity,
                    board: e.deck_card.board,
                    mvp: e.deck_card.mvp_at.is_some(),
                });
        }
        map
    });

    // Raises the land and budget toasts for a change from `before`.
    let crossings = move |before: &[DeckEntry]| {
        let after = entries.peek().clone();
        let cur = currency();
        let lands = land_crossing(
            land_target(),
            land_count(before),
            land_count(&after),
            Crossing::Both,
        );
        let zone_now = zone_cards.peek().clone();
        let price = budget_crossing(
            budget(),
            cur,
            deck_price(before, &zone_now, cur),
            deck_price(&after, &zone_now, cur),
        );
        signal_toast(toast, lands);
        signal_toast(toast, price);
    };

    let change_quantity = use_callback(move |(card_id, delta, record): (Uuid, i32, bool)| {
        let current = meta.peek().get(&card_id).map_or(1, |m| m.qty);
        let Some(target) = current.checked_add(delta) else {
            return;
        };
        let before = entries.peek().clone();
        if target < 1 {
            let Some(entry) = before
                .iter()
                .find(|e| e.deck_card.scryfall_data_id == card_id)
                .cloned()
            else {
                return;
            };
            removed_rows.write().insert(card_id, entry.clone());
            if record {
                let baseline = bursts.peek().baseline(card_id).unwrap_or(current);
                undo.write().push(
                    deck_id,
                    UndoAction::Removed {
                        entry: Box::new(entry),
                        baseline,
                    },
                );
            }
            entries
                .write()
                .retain(|e| e.deck_card.scryfall_data_id != card_id);
            crossings(&before);
            if record {
                toast.info(
                    "Card removed".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        } else {
            if let Ok(quantity) = Quantity::new(target)
                && let Some(entry) = entries
                    .write()
                    .iter_mut()
                    .find(|e| e.deck_card.scryfall_data_id == card_id)
            {
                entry.deck_card.quantity = quantity;
            }
            crossings(&before);
        }
        let generation = bursts.write().tap(card_id, current, delta, record);
        spawn(async move {
            timer::sleep(QUANTITY_FLUSH).await;
            let Some(burst) = bursts.write().due(card_id, generation) else {
                return;
            };
            let removed = removed_rows.write().remove(&card_id);
            match burst.write() {
                Write::Nothing => {}
                Write::Delete => {
                    let reply = authed
                        .try_run("change_quantity", |c, s| async move {
                            c.delete_deck_card(deck_id, card_id, &s).await
                        })
                        .await;
                    if reply.is_err() {
                        undo.write().take_newest(deck_id, |a| {
                            matches!(a, UndoAction::Removed { entry, .. } if entry.deck_card.scryfall_data_id == card_id)
                        });
                        let back = entries
                            .peek()
                            .iter()
                            .any(|e| e.deck_card.scryfall_data_id == card_id);
                        if let (false, Some(mut restored)) = (back, removed) {
                            if let Ok(quantity) = Quantity::new(burst.baseline) {
                                restored.deck_card.quantity = quantity;
                            }
                            entries.write().push(restored);
                        }
                    }
                }
                Write::Set(quantity) => {
                    let request = HttpPatchDeckCard::new(Some(quantity), None);
                    let reply = authed
                        .try_run("change_quantity", |c, s| async move {
                            c.update_deck_card(deck_id, card_id, &request, &s).await
                        })
                        .await;
                    let name = entries
                        .peek()
                        .iter()
                        .find(|e| e.deck_card.scryfall_data_id == card_id)
                        .map(|e| e.card.scryfall_data.name.clone())
                        .unwrap_or_default();
                    match reply {
                        Ok(_) if burst.record => undo.write().push(
                            deck_id,
                            UndoAction::QuantityChanged {
                                card_id,
                                card_name: name,
                                net: burst.delta,
                                baseline: burst.baseline,
                            },
                        ),
                        Ok(_) => {}
                        Err(_) => {
                            if let Ok(quantity) = Quantity::new(burst.baseline)
                                && let Some(entry) = entries
                                    .write()
                                    .iter_mut()
                                    .find(|e| e.deck_card.scryfall_data_id == card_id)
                            {
                                entry.deck_card.quantity = quantity;
                            }
                        }
                    }
                }
            }
        });
    });

    let move_to_board = use_callback(move |(card_id, target, record): (Uuid, Board, bool)| {
        let Some(entry) = entries
            .peek()
            .iter()
            .find(|e| e.deck_card.scryfall_data_id == card_id)
            .cloned()
        else {
            return;
        };
        let from = entry.deck_card.board;
        if from == target {
            return;
        }
        if record {
            undo.write().push(
                deck_id,
                UndoAction::MovedBoard {
                    card_id,
                    card_name: entry.card.scryfall_data.name,
                    from,
                },
            );
        }
        let mut set_board = move |board: Board| {
            if let Some(e) = entries
                .write()
                .iter_mut()
                .find(|e| e.deck_card.scryfall_data_id == card_id)
            {
                e.deck_card.board = board;
            }
        };
        set_board(target);
        spawn(async move {
            let request = HttpPatchDeckCard::new(None, Some(target.display_name().to_owned()));
            if authed
                .try_run("move_board", |c, s| async move {
                    c.update_deck_card(deck_id, card_id, &request, &s).await
                })
                .await
                .is_err()
            {
                set_board(from);
            }
        });
        if record {
            toast.info(
                format!("Moved to {}", target.display_name()),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
    });

    let toggle_mvp = use_callback(move |(card_id, starred): (Uuid, bool)| {
        let old = entries
            .peek()
            .iter()
            .find(|e| e.deck_card.scryfall_data_id == card_id)
            .and_then(|e| e.deck_card.mvp_at);
        let mut set_mvp = move |at| {
            if let Some(e) = entries
                .write()
                .iter_mut()
                .find(|e| e.deck_card.scryfall_data_id == card_id)
            {
                e.deck_card.mvp_at = at;
            }
        };
        set_mvp((!starred).then(Utc::now));
        spawn(async move {
            let request = HttpPatchDeckCard::with_mvp(!starred);
            match authed
                .try_run("toggle_mvp", |c, s| async move {
                    c.update_deck_card(deck_id, card_id, &request, &s).await
                })
                .await
            {
                Ok(saved) => set_mvp(saved.mvp_at),
                Err(_) => set_mvp(old),
            }
        });
    });

    let save_printing = use_callback(move |picked: Card| {
        let Some((old_card, slot)) = printing.peek().clone() else {
            return;
        };
        let old_id = old_card.scryfall_data.id;
        let new_id = picked.scryfall_data.id;
        spawn(async move {
            if let Some(slot) = slot {
                let request = slot.update_request(Some(new_id));
                if authed
                    .run("change_printing", |c, s| async move {
                        c.update_deck_profile(deck_id, &request, &s).await
                    })
                    .await
                    .is_some()
                {
                    *zone.write().slot_mut(slot) = Some(picked.clone());
                    undo.write().push(
                        deck_id,
                        UndoAction::CommandZonePrintingChanged {
                            slot,
                            old_card: Box::new(old_card),
                            new_id,
                        },
                    );
                    printing.set(Some((picked, Some(slot))));
                    if *selected.peek() == Some(old_id) {
                        selected.set(Some(new_id));
                    }
                }
                return;
            }
            let request = HttpPatchDeckCard::with_printing(&new_id.to_string());
            if authed
                .run("change_printing", |c, s| async move {
                    c.update_deck_card(deck_id, old_id, &request, &s).await
                })
                .await
                .is_some()
            {
                if let Some(e) = entries
                    .write()
                    .iter_mut()
                    .find(|e| e.deck_card.scryfall_data_id == old_id)
                {
                    e.card = picked.clone();
                    e.deck_card.scryfall_data_id = new_id;
                }
                undo.write().push(
                    deck_id,
                    UndoAction::PrintingChanged {
                        old_card: Box::new(old_card),
                        new_id,
                    },
                );
                printing.set(Some((picked, None)));
                if *selected.peek() == Some(old_id) {
                    selected.set(Some(new_id));
                }
            }
        });
    });

    let apply_undo = use_callback(move |()| {
        let popped = undo.write().pop(deck_id);
        let Some(action) = popped else {
            return;
        };
        let quick = ToastOptions::default().duration(TOAST_QUICK);
        if deck_cards::is_stale(&action, &entries.peek(), &zone.peek()) {
            toast.info("Already changed".to_string(), quick);
            return;
        }
        let message = deck_cards::undo_message(&action);
        match action {
            UndoAction::Added { card_id, .. } => {
                entries
                    .write()
                    .retain(|e| e.deck_card.scryfall_data_id != card_id);
                spawn(async move {
                    authed
                        .run("undo", |c, s| async move {
                            c.delete_deck_card(deck_id, card_id, &s).await
                        })
                        .await;
                });
            }
            UndoAction::Removed { entry, baseline } => {
                let mut entry = *entry;
                if let Ok(quantity) = Quantity::new(baseline) {
                    entry.deck_card.quantity = quantity;
                }
                let card_id = entry.deck_card.scryfall_data_id;
                let request = HttpCreateDeckCard::new(
                    &entry.card.scryfall_data,
                    baseline,
                    create_board_field(entry.deck_card.board),
                );
                entries.write().push(entry);
                spawn(async move {
                    let reply = authed
                        .try_run("undo", |c, s| async move {
                            c.create_deck_card(deck_id, &request, &s).await
                        })
                        .await;
                    let mut list = entries.write();
                    match reply {
                        Ok(saved) => {
                            if let Some(e) = list
                                .iter_mut()
                                .find(|e| e.deck_card.scryfall_data_id == card_id)
                            {
                                e.deck_card = saved;
                            }
                        }
                        Err(_) => list.retain(|e| e.deck_card.scryfall_data_id != card_id),
                    }
                });
            }
            UndoAction::QuantityChanged { card_id, net, .. } => {
                change_quantity.call((card_id, net.saturating_neg(), false));
            }
            UndoAction::MovedBoard { card_id, from, .. } => {
                move_to_board.call((card_id, from, false));
            }
            UndoAction::PrintingChanged { old_card, new_id } => {
                let old_card = *old_card;
                let old_id = old_card.scryfall_data.id;
                if let Some(e) = entries
                    .write()
                    .iter_mut()
                    .find(|e| e.deck_card.scryfall_data_id == new_id)
                {
                    e.card = old_card;
                    e.deck_card.scryfall_data_id = old_id;
                }
                if *selected.peek() == Some(new_id) {
                    selected.set(Some(old_id));
                }
                let request = HttpPatchDeckCard::with_printing(&old_id.to_string());
                spawn(async move {
                    authed
                        .run("undo", |c, s| async move {
                            c.update_deck_card(deck_id, new_id, &request, &s).await
                        })
                        .await;
                });
            }
            UndoAction::CommandZonePrintingChanged { slot, old_card, .. } => {
                let old_id = old_card.scryfall_data.id;
                *zone.write().slot_mut(slot) = Some(*old_card);
                let request = slot.update_request(Some(old_id));
                spawn(async move {
                    authed
                        .run("undo", |c, s| async move {
                            c.update_deck_profile(deck_id, &request, &s).await
                        })
                        .await;
                });
            }
        }
        toast.info(message, quick);
    });

    let reset_filter = move |_| {
        filter.set(CardQueryBuilder::default());
        selected.set(None);
        toast.info(
            "Filter reset".to_string(),
            ToastOptions::default().duration(TOAST_QUICK),
        );
    };

    // == the rows as they list, top to bottom ==
    let listing = use_memo(move || {
        let applied = filter.read().clone();
        let token_cards = if show_tokens() {
            tokens().unwrap_or_default()
        } else {
            Vec::new()
        };
        list_rows(
            &display.read(),
            &meta.read(),
            token_cards,
            &applied,
            boards(),
            show_zone(),
        )
    });
    let Listed {
        tokens: token_rows,
        maybe: maybe_rows,
        side: side_rows,
        zone: zone_rows,
        groups: main_groups,
        lands: main_lands,
        rows,
    } = listing();
    let row_meta = meta();
    let applied = filter.read().clone();
    let shown_boards = boards();
    let format = profile.read().as_ref().and_then(|p| p.format);
    let oathbreaker = format.is_some_and(|f| f.has_signature_spell());
    let main_row = |card: &Card| -> RowInfo { main_row_of(&row_meta, card) };
    let pane_row =
        selected().and_then(|id| rows.iter().find(|r| r.card.scryfall_data.id == id).cloned());
    let qty_count = |cards: &[Card]| -> i32 {
        cards
            .iter()
            .map(|c| row_meta.get(&c.scryfall_data.id).map_or(1, |m| m.qty))
            .fold(0, i32::saturating_add)
    };
    let has_maybe = entries
        .read()
        .iter()
        .any(|e| e.deck_card.board.is_maybeboard());
    let has_side = entries
        .read()
        .iter()
        .any(|e| e.deck_card.board.is_sideboard());
    let featured = profile.read().as_ref().map_or_else(Vec::new, |_| {
        deck_cards::featured(&zone.read(), &entries.read(), oathbreaker)
    });
    let sources = Sources::deck(
        &entries
            .read()
            .iter()
            .map(|e| e.card.clone())
            .collect::<Vec<_>>(),
    );
    let cur = profile
        .read()
        .as_ref()
        .and_then(|p| p.price_target_currency)
        .unwrap_or_default();
    let undo_ready = !undo.read().is_empty(deck_id);
    let filter_on = !applied.is_empty();
    let dot = filter::shows_dot(&applied);
    let has_zone = !zone_rows.is_empty();
    let has_partner = zone_rows.iter().any(|(slot, _)| *slot == Slot::Partner);

    use_swipe_key_handler(
        vec![Swipe::Up, Swipe::Down],
        Callback::new(move |swipe: Swipe| {
            let list: Vec<Uuid> = listing
                .peek()
                .rows
                .iter()
                .map(|r| r.card.scryfall_data.id)
                .collect();
            if list.is_empty() {
                return;
            }
            let at = selected
                .peek()
                .and_then(|id| list.iter().position(|v| *v == id));
            let next = match (swipe, at) {
                (Swipe::Up, None) => list.len().saturating_sub(1),
                (Swipe::Down, Some(i)) => i.saturating_add(1).min(list.len().saturating_sub(1)),
                (_, Some(i)) => i.saturating_sub(1),
                (_, None) => 0,
            };
            if let Some(id) = list.get(next) {
                selected.set(Some(*id));
                face.set(0);
                document::eval(&format!(
                    "const el = document.querySelector('[data-row=\"{id}\"]'); if (el) el.scrollIntoView({{ block: 'nearest' }});"
                ));
            }
        }),
    );
    use_page_keys(Callback::new(move |key: String| {
        let row = selected.peek().and_then(|id| {
            listing
                .peek()
                .rows
                .iter()
                .find(|r| r.card.scryfall_data.id == id)
                .cloned()
        });
        match key.as_str() {
            "/" => focus_by_id(QUICK_ADD_ID),
            "+" | "=" | "-" => {
                let Some(row) = row else {
                    return;
                };
                if matches!(row.kind, RowKind::Main { .. } | RowKind::Board(_)) {
                    let delta = if key == "-" { -1 } else { 1 };
                    change_quantity.call((row.card.scryfall_data.id, delta, true));
                }
            }
            "f" | "F" => {
                if let Some(row) = row {
                    let count = components::card_face_count(&row.card);
                    if count > 1 {
                        let next = face
                            .peek()
                            .saturating_add(1)
                            .checked_rem(count)
                            .unwrap_or(0);
                        face.set(next);
                    }
                }
            }
            _ => {}
        }
    }));
    use_undo_key(Callback::new(move |()| apply_undo.call(())));

    let mut on_drop = move |board: Board| {
        if let Some(card_id) = dragging.take() {
            move_to_board.call((card_id, board, true));
        }
    };
    let meta_for_rows = row_meta.clone();
    let row_view = move |row: RowInfo| -> Element {
        let card_id = row.card.scryfall_data.id;
        let RowInfo { card, qty, kind } = row;
        let is_selected = selected() == Some(card_id);
        let RowActions {
            on_qty,
            on_move,
            board,
            mvp,
            on_star,
            on_print,
            drag,
        } = match kind {
            RowKind::Main { mvp } => RowActions {
                on_qty: Some(EventHandler::new(move |d| {
                    change_quantity.call((card_id, d, true));
                })),
                on_move: Some(EventHandler::new(move |b| {
                    move_to_board.call((card_id, b, true));
                })),
                board: Some(meta_for_rows.get(&card_id).map_or(Board::Deck, |m| m.board)),
                mvp: Some(mvp),
                on_star: Some(EventHandler::new(move |()| toggle_mvp.call((card_id, mvp)))),
                on_print: Some(EventHandler::new(move |c: Card| {
                    printing.set(Some((c, None)));
                    printing_open.set(true);
                })),
                drag: Some(EventHandler::new(move |()| dragging.set(Some(card_id)))),
            },
            RowKind::Board(b) => RowActions {
                on_qty: Some(EventHandler::new(move |d| {
                    change_quantity.call((card_id, d, true));
                })),
                on_move: Some(EventHandler::new(move |t| {
                    move_to_board.call((card_id, t, true));
                })),
                board: Some(b),
                on_print: Some(EventHandler::new(move |c: Card| {
                    printing.set(Some((c, None)));
                    printing_open.set(true);
                })),
                drag: Some(EventHandler::new(move |()| dragging.set(Some(card_id)))),
                ..RowActions::default()
            },
            RowKind::Zone(slot) => RowActions {
                on_print: Some(EventHandler::new(move |c: Card| {
                    printing.set(Some((c, Some(slot))));
                    printing_open.set(true);
                })),
                ..RowActions::default()
            },
            RowKind::Token => RowActions::default(),
        };
        rsx! {
            div {
                key: "{card_id}",
                class: if is_selected { "web-deck-row selected" } else { "web-deck-row" },
                "data-row": "{card_id}",
                onclick: move |_| face.set(0),
                CardRow {
                    card,
                    qty,
                    expanded_card: selected,
                    on_qty_change: on_qty,
                    on_move_to: on_move,
                    current_board: board,
                    mvp,
                    on_toggle_mvp: on_star,
                    price_currency: cur,
                    on_printing: on_print,
                    on_drag_start: drag,
                }
            }
        }
    };

    rsx! {
        ScreenFrame {
            title: "Deck cards",
            description: Some("Every card in the deck, with quantities, boards and printings".to_string()),
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div { class: "screen-content web-deck-cards-view",
                div { class: "web-deck-cards-controls",
                    if let Some(p) = profile() {
                        div { class: "deck-cards-header row-enter",
                            span { class: "deck-cards-header-name", "{p.name}" }
                            if let Some(fmt) = p.format {
                                span { class: "stat-chip stat-chip-p3", "{fmt.display_name()}" }
                            }
                            if let Some(pl) = p.power_level {
                                span { class: "stat-chip stat-chip-p5", "{pl.display_name()}" }
                            }
                            for tag in p.tags.iter() {
                                span { key: "{tag}", class: "stat-chip stat-chip-muted", "{deck_tag_label(tag)}" }
                            }
                            for tag in p.other_tags.iter() {
                                span { key: "{tag}", class: "stat-chip stat-chip-p5", "{tag.display_name()}" }
                            }
                        }
                    } else if !loaded() {
                        div { class: "deck-cards-header",
                            div { class: "skeleton-bar skeleton-deck-list-title" }
                            div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                            div { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                            div { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                        }
                    }
                    FeaturedCards {
                        cards: featured,
                        loaded: loaded(),
                        on_tap: move |card: Card| preview.set(Some((card.scryfall_data, 0))),
                    }
                    if loaded() {
                        QuickAdd { deck_id, entries, on_added: move |()| {} }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "Group by" }
                            for option in GroupByOption::all() {
                                Chip {
                                    key: "{option}",
                                    selected: group_by() == option,
                                    onclick: move |_| {
                                        selected.set(None);
                                        group_by.set(option);
                                    },
                                    "{option}"
                                }
                            }
                        }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "Boards" }
                            for (label, board, on, enabled) in [
                                ("Main", Board::Deck, shown_boards.main, true),
                                ("Maybe", Board::Maybeboard, shown_boards.maybe, has_maybe || shown_boards.maybe),
                                ("Side", Board::Sideboard, shown_boards.side, has_side || shown_boards.side),
                            ] {
                                span {
                                    key: "{label}",
                                    class: "web-filter-chip",
                                    ondragover: move |e| e.prevent_default(),
                                    ondrop: move |e| {
                                        e.prevent_default();
                                        on_drop(board);
                                    },
                                    Chip {
                                        selected: on,
                                        disabled: !enabled,
                                        onclick: move |_| {
                                            boards.write().toggle(board);
                                            selected.set(None);
                                        },
                                        "{label}"
                                    }
                                }
                            }
                        }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "Show" }
                            Chip { selected: show_tokens(), onclick: move |_| show_tokens.toggle(), "Tokens" }
                            Chip { selected: show_zone(), onclick: move |_| show_zone.toggle(), "Command zone" }
                            Chip { selected: show_art(), onclick: move |_| show_art.toggle(), "Art" }
                        }
                    } else {
                        div { class: "chip-row",
                            div { class: "skeleton-bar skeleton-chip skeleton-chip-lg stat-chip-p1 skeleton-tinted" }
                            div { class: "skeleton-input-compact" }
                        }
                        ChipRowSkeleton { label: "md", chips: vec!["sm", "lg", "md", "lg"] }
                        ChipRowSkeleton { label: "md", chips: vec!["sm", "md", "sm"] }
                        ChipRowSkeleton { label: "sm", chips: vec!["md", "lg", "sm"] }
                    }
                }
                div { class: "web-deck-cards-list",
                    if !loaded() {
                        DeckCardListGhost {}
                    }
                    if !token_rows.is_empty() {
                        CardGroup {
                            title: format!("Tokens ({})", token_rows.len()),
                            section_key: "Tokens",
                            collapsed,
                            on_drop,
                            for card in token_rows {
                                {row_view(RowInfo { card, qty: 1, kind: RowKind::Token })}
                            }
                        }
                    }
                    if !maybe_rows.is_empty() {
                        CardGroup {
                            title: format!("Maybeboard ({})", maybe_rows.len()),
                            section_key: "Maybeboard",
                            collapsed,
                            drop_to: Some(Board::Maybeboard),
                            on_drop,
                            for entry in maybe_rows {
                                {row_view(RowInfo { qty: *entry.deck_card.quantity, card: entry.card, kind: RowKind::Board(Board::Maybeboard) })}
                            }
                        }
                    }
                    if !side_rows.is_empty() {
                        CardGroup {
                            title: format!("Sideboard ({})", side_rows.len()),
                            section_key: "Sideboard",
                            collapsed,
                            drop_to: Some(Board::Sideboard),
                            on_drop,
                            for entry in side_rows {
                                {row_view(RowInfo { qty: *entry.deck_card.quantity, card: entry.card, kind: RowKind::Board(Board::Sideboard) })}
                            }
                        }
                    }
                    if has_zone {
                        for (slot, card) in zone_rows.clone() {
                            if slot != Slot::Partner || !zone_rows.iter().any(|(s, _)| *s == Slot::Commander) {
                                div { key: "{slot:?}", class: "card-group row-enter",
                                    div { class: "card-group-header", "{slot.group_header(format, has_partner)}" }
                                    {row_view(RowInfo { card, qty: 1, kind: RowKind::Zone(slot) })}
                                    if slot == Slot::Commander {
                                        for (partner_slot, partner) in zone_rows.iter().filter(|(s, _)| *s == Slot::Partner).cloned() {
                                            {row_view(RowInfo { card: partner, qty: 1, kind: RowKind::Zone(partner_slot) })}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    for group in main_groups.clone() {
                        CardGroup {
                            key: "{group.key}",
                            title: if group.label.is_empty() { format!("({})", qty_count(&group.cards)) } else { format!("{} ({})", group.label, qty_count(&group.cards)) },
                            section_key: group.key.clone(),
                            pips: group.pips.clone(),
                            collapsed,
                            drop_to: Some(Board::Deck),
                            on_drop,
                            for card in group.cards.iter() {
                                {row_view(main_row(card))}
                            }
                        }
                    }
                    if !main_lands.is_empty() {
                        CardGroup {
                            title: format!("Lands ({})", qty_count(&main_lands)),
                            section_key: "Lands",
                            collapsed,
                            drop_to: Some(Board::Deck),
                            on_drop,
                            for card in main_lands.iter() {
                                {row_view(main_row(card))}
                            }
                        }
                    }
                    if shown_boards.main && main_groups.is_empty() && main_lands.is_empty() && loaded() {
                        p { class: "text-muted", "No cards" }
                    }
                }
                div { class: "web-deck-cards-pane",
                    if let Some(row) = pane_row {
                        DetailPane {
                            key: "{row.card.scryfall_data.id}",
                            row,
                            face,
                            board: row_meta.get(&selected().unwrap_or_default()).map(|m| m.board),
                            on_examples,
                            on_qty: move |(id, d): (Uuid, i32)| change_quantity.call((id, d, true)),
                            on_move: move |(id, b): (Uuid, Board)| move_to_board.call((id, b, true)),
                            on_star: move |(id, s): (Uuid, bool)| toggle_mvp.call((id, s)),
                            on_printing: move |(c, slot): (Card, Option<Slot>)| {
                                printing.set(Some((c, slot)));
                                printing_open.set(true);
                            },
                            on_image: move |(c, f): (Card, usize)| preview.set(Some((c.scryfall_data, f))),
                        }
                    } else {
                        p { class: "text-muted web-deck-cards-pane-empty", "Select a card" }
                    }
                }
            }
            ActionBar {
                Button { variant: ButtonVariant::Action, onclick: move |_| navigator.go_back(), "Back" }
                Button { variant: ButtonVariant::Action, onclick: move |_| { navigator.push(Route::AddDeckCard { deck_id }); }, "Add" }
                Button { variant: ButtonVariant::Action, onclick: move |_| { navigator.push(Route::RemoveDeckCard { deck_id }); }, "Remove" }
                Button { variant: ButtonVariant::Action, onclick: move |_| filter_open.set(true),
                    "Filter"
                    if dot {
                        span { class: "filter-dot" }
                    }
                }
                if undo_ready {
                    Button { variant: ButtonVariant::Action, onclick: move |_| apply_undo.call(()), "Undo" }
                }
                if filter_on {
                    Button { variant: ButtonVariant::Action, class: "btn-action-clear", onclick: reset_filter, "Reset filter" }
                }
            }
        }
        CardFilterPanel {
            open: filter_open,
            filter,
            config: SheetConfig { show_format: true, sources, ..SheetConfig::default() },
            on_apply: move |next: CardQueryBuilder| {
                filter.set(next);
                selected.set(None);
            },
        }
        CardOverlaysHost { overlays, on_printing_save: move |_| {} }
        OtagExamplesHost { examples }
        OneTimeHintDialog { hint: OneTimeHint::DeckCards, open: hint_open }
        if let Some((card, _)) = printing() {
            PrintingSheet {
                key: "{card.scryfall_data.id}",
                card,
                open: printing_open,
                on_save: move |picked: Card| save_printing.call(picked),
            }
        }
    }
}

/// The groups' ghost while the deck loads.
#[component]
fn DeckCardListGhost() -> Element {
    rsx! {
        div { class: "skeleton-card-list",
            div { class: "skeleton-card-group",
                div { class: "skeleton-card-group-header",
                    div { class: "skeleton-bar skeleton-card-bar-header" }
                }
                div { class: "skeleton-card-row",
                    div { class: "skeleton-bar skeleton-card-thumb" }
                    div { class: "skeleton-bar skeleton-card-bar-row" }
                }
            }
            div { class: "skeleton-card-group",
                div { class: "skeleton-card-group-header",
                    div { class: "skeleton-bar skeleton-card-bar-header skeleton-card-bar-header-wide" }
                }
                for i in 0..6 {
                    div { key: "{i}", class: "skeleton-card-row",
                        div { class: "skeleton-bar skeleton-card-thumb" }
                        div { class: "skeleton-bar skeleton-card-bar-row" }
                    }
                }
            }
        }
    }
}

/// The selected row on a wide window: its image, its details and the same
/// actions the row opens inline below the breakpoint.
#[component]
fn DetailPane(
    row: RowInfo,
    face: Signal<usize>,
    board: Option<Board>,
    on_examples: Option<Callback<String>>,
    on_qty: EventHandler<(Uuid, i32)>,
    on_move: EventHandler<(Uuid, Board)>,
    on_star: EventHandler<(Uuid, bool)>,
    on_printing: EventHandler<(Card, Option<Slot>)>,
    on_image: EventHandler<(Card, usize)>,
) -> Element {
    let describe_tag = crate::inbound::cards::otag::use_describe_tag();
    let RowInfo { card, qty, kind } = row;
    let id = card.scryfall_data.id;
    let shown_face = face();
    let card_for_image = card.clone();
    let card_for_print = card.clone();
    let current = match kind {
        RowKind::Board(b) => b,
        _ => board.unwrap_or(Board::Deck),
    };
    let editable = matches!(kind, RowKind::Main { .. } | RowKind::Board(_));
    let slot = match kind {
        RowKind::Zone(slot) => Some(slot),
        _ => None,
    };
    let moves: Vec<(&'static str, Board)> = match current {
        Board::Deck => vec![
            ("To maybeboard", Board::Maybeboard),
            ("To sideboard", Board::Sideboard),
        ],
        Board::Maybeboard => vec![
            ("To mainboard", Board::Deck),
            ("To sideboard", Board::Sideboard),
        ],
        Board::Sideboard => vec![
            ("To mainboard", Board::Deck),
            ("To maybeboard", Board::Maybeboard),
        ],
    };
    let has_image = card
        .scryfall_data
        .primary_image_url(ImageSize::Normal)
        .is_some();
    rsx! {
        div { class: "sheet-panel web-deck-pane",
            if has_image {
                components::FlippableCardImage {
                    key: "{id}-{shown_face}",
                    sd: card.scryfall_data.clone(),
                    size: ImageSize::Normal,
                    class: "web-deck-pane-image".to_string(),
                    initial_face: shown_face,
                    on_face_change: move |f: usize| face.set(f),
                }
            }
            CardDetails {
                card,
                show_name: true,
                show_cost: true,
                show_classification: true,
                face: Some(face),
                describe_tag: Some(describe_tag),
                on_examples,
                on_image: has_image.then(|| EventHandler::new(move |()| on_image.call((card_for_image.clone(), face())))),
                has_actions: !matches!(kind, RowKind::Token),
                actions: rsx! {
                    div { class: "card-action-row",
                        if editable {
                            button { class: "card-action-btn", onclick: move |_| on_qty.call((id, -1)), "-" }
                            span { class: "card-action-count", "{qty}" }
                            button { class: "card-action-btn", onclick: move |_| on_qty.call((id, 1)), "+" }
                        }
                        if !matches!(kind, RowKind::Token) {
                            button {
                                class: "card-action-btn",
                                onclick: move |_| on_printing.call((card_for_print.clone(), slot)),
                                "Printing"
                            }
                        }
                        if let RowKind::Main { mvp } = kind {
                            button { class: "card-action-btn", onclick: move |_| on_star.call((id, mvp)),
                                if mvp { "Unstar" } else { "Star" }
                            }
                        }
                    }
                    if editable {
                        div { class: "card-action-row",
                            for (label, target) in moves {
                                button {
                                    key: "{label}",
                                    class: "card-action-btn",
                                    style: "white-space:nowrap;",
                                    onclick: move |_| on_move.call((id, target)),
                                    "{label}"
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

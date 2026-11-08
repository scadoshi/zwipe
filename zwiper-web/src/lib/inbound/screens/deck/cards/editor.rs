//! The deck's editing engine, behind the Cards screen below the breakpoint
//! and the deck page on a wide window: the deck's entries, zone and
//! profile, the filter, the group and board rows, the selection the keys
//! move, the quantity bursts, moves, stars, printings and undo, each saved
//! as it happens. [`use_deck_editor`] holds it all, and [`row_view`] draws
//! one row with the actions its kind allows.

use std::collections::{HashMap, HashSet};

use chrono::Utc;
use components::TOAST_QUICK;
use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::deck_metrics::deck_price;
use zwipe_core::domain::deck::{Board, Deck, DeckEntry, DeckProfile, Quantity};
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpPatchDeckCard};
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck_cards::{self, Boards, Bursts, Write, Zone};
use zwipe_ui::deck_signals::{Crossing, budget_crossing, land_count, land_crossing};
use zwipe_ui::screens;
use zwipe_ui::stores::filters::FilterScope;
use zwipe_ui::stores::undo::{UndoAction, create_board_field};
use zwipe_ui::timing::QUANTITY_FLUSH;

use super::quick_add::QUICK_ADD_ID;
use super::signal_toast;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_row::{CardRow, ShowRowArt};
use crate::inbound::cards::swipe_stack::{Swipe, use_swipe_key_handler};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, open_and_record_hint};
use crate::inbound::chrome::page_keys::{focus_by_id, use_page_keys, use_undo_key};
use crate::inbound::screens::deck::view::cards::zone as zone_of;
use crate::outbound::timer;

/// What a row is, which decides its actions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RowKind {
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
pub(crate) struct RowInfo {
    pub card: Card,
    pub qty: i32,
    pub kind: RowKind,
}

/// A row's quantity, board and star, the first entry for a printing winning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Meta {
    pub qty: i32,
    pub board: Board,
    pub mvp: bool,
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
pub(crate) struct Listed {
    pub tokens: Vec<Card>,
    pub maybe: Vec<DeckEntry>,
    pub side: Vec<DeckEntry>,
    pub zone: Vec<(Slot, Card)>,
    pub groups: Vec<deck_cards::Group>,
    pub lands: Vec<Card>,
    pub rows: Vec<RowInfo>,
}

/// A main board row, its quantity and star from the deck's entries.
pub(crate) fn main_row_of(meta: &HashMap<Uuid, Meta>, card: &Card) -> RowInfo {
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
pub(crate) fn list_rows(
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

/// The engine's state and actions. `Copy`: every field is a signal, a memo
/// or a callback.
#[derive(Clone, Copy)]
pub(crate) struct Editor {
    /// The deck as fetched, with its warnings and command zone cards.
    pub deck: Signal<Option<Deck>>,
    /// The entries, the command zone stripped out.
    pub entries: Signal<Vec<DeckEntry>>,
    pub zone: Signal<Zone>,
    pub profile: Signal<Option<DeckProfile>>,
    pub loaded: Signal<bool>,
    pub tokens: Signal<Option<Vec<Card>>>,
    /// The filter sheet's filter.
    pub filter: Signal<CardQueryBuilder>,
    pub group_by: Signal<GroupByOption>,
    pub boards: Signal<Boards>,
    pub show_tokens: Signal<bool>,
    pub show_zone: Signal<bool>,
    pub show_art: Signal<bool>,
    pub collapsed: Signal<HashSet<String>>,
    pub selected: Signal<Option<Uuid>>,
    pub face: Signal<usize>,
    pub printing: Signal<Option<(Card, Option<Slot>)>>,
    pub printing_open: Signal<bool>,
    pub filter_open: Signal<bool>,
    pub dragging: Signal<Option<Uuid>>,
    pub hint_open: Signal<bool>,
    pub listing: Memo<Listed>,
    pub meta: Memo<HashMap<Uuid, Meta>>,
    pub change_quantity: Callback<(Uuid, i32, bool)>,
    pub move_to_board: Callback<(Uuid, Board, bool)>,
    pub toggle_mvp: Callback<(Uuid, bool)>,
    pub save_printing: Callback<Card>,
    pub apply_undo: Callback<()>,
    pub reset_filter: Callback<()>,
    pub reload: Callback<()>,
    pub on_drop: Callback<Board>,
}

impl Editor {
    /// The price currency the deck's target is in, the default without one.
    pub(crate) fn currency(&self) -> PriceCurrency {
        self.profile
            .read()
            .as_ref()
            .and_then(|p| p.price_target_currency)
            .unwrap_or_default()
    }
}

/// The engine for `deck_id`: fetches the deck and its tokens, parks the
/// filter on leave, and binds the keys (↑ ↓ move the selection, `+` and `-`
/// change its quantity, F flips it, `/` focuses Quick add, Ctrl+Z undoes).
#[expect(
    clippy::too_many_lines,
    reason = "the engine's handlers share its signals and read as one piece"
)]
pub(crate) fn use_deck_editor(deck_id: Uuid) -> Editor {
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
    let mut deck: Signal<Option<Deck>> = use_signal(|| None);
    let mut loaded = use_signal(|| false);
    let mut tokens: Signal<Option<Vec<Card>>> = use_signal(|| None);
    let group_by = use_signal(|| GroupByOption::CardType);
    let boards = use_signal(Boards::default);
    let show_tokens = use_signal(|| false);
    let show_zone = use_signal(|| true);
    let show_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_art));
    let collapsed: Signal<HashSet<String>> = use_signal(HashSet::new);
    let mut selected: Signal<Option<Uuid>> = use_signal(|| None);
    let mut face = use_signal(|| 0usize);
    let mut bursts = use_signal(Bursts::default);
    let mut removed_rows: Signal<HashMap<Uuid, DeckEntry>> = use_signal(HashMap::new);
    let mut printing: Signal<Option<(Card, Option<Slot>)>> = use_signal(|| None);
    let printing_open = use_signal(|| false);
    let filter_open = use_signal(|| false);
    let mut dragging: Signal<Option<Uuid>> = use_signal(|| None);
    let hint_open = use_signal(|| false);
    let mut hint_fired = use_signal(|| false);

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

    // Fetches the deck, first on mount and again after a fix made elsewhere.
    let reload = use_callback(move |()| {
        spawn(async move {
            let Some(fetched) = authed
                .run("load_deck", |c, s| async move {
                    c.fetch_deck(deck_id, &s).await
                })
                .await
            else {
                return;
            };
            let held = zone_of(&fetched);
            let mut list = fetched.entries.clone();
            held.strip(&mut list);
            zone.set(held);
            zone_cards.set(fetched.command_zone_cards.clone());
            profile.set(Some(fetched.deck_profile.clone()));
            entries.set(list);
            deck.set(Some(fetched));
            let first = !*loaded.peek();
            loaded.set(true);
            if first && restored_active {
                toast.warning(
                    "Filter is active".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });
    use_hook(move || {
        reload.call(());
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
                screens::deck_cards::moved_to(target),
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

    let reset_filter = use_callback(move |()| {
        filter.set(CardQueryBuilder::default());
        selected.set(None);
        toast.info(
            "Filter reset".to_string(),
            ToastOptions::default().duration(TOAST_QUICK),
        );
    });

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

    let on_drop = use_callback(move |board: Board| {
        if let Some(card_id) = dragging.take() {
            move_to_board.call((card_id, board, true));
        }
    });

    Editor {
        deck,
        entries,
        zone,
        profile,
        loaded,
        tokens,
        filter,
        group_by,
        boards,
        show_tokens,
        show_zone,
        show_art,
        collapsed,
        selected,
        face,
        printing,
        printing_open,
        filter_open,
        dragging,
        hint_open,
        listing,
        meta,
        change_quantity,
        move_to_board,
        toggle_mvp,
        save_printing,
        apply_undo,
        reset_filter,
        reload,
        on_drop,
    }
}

/// One listed row as a selectable [`CardRow`] with the editing actions its
/// kind allows.
pub(crate) fn row_view(
    ed: &Editor,
    cur: PriceCurrency,
    row_meta: &HashMap<Uuid, Meta>,
    row: RowInfo,
) -> Element {
    let Editor {
        selected,
        mut face,
        mut printing,
        mut printing_open,
        mut dragging,
        change_quantity,
        move_to_board,
        toggle_mvp,
        ..
    } = *ed;
    let meta_for_rows = row_meta.clone();
    {
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
    }
}

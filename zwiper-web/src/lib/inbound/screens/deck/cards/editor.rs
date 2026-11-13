//! The deck's editing engine, behind the Cards screen below the breakpoint
//! and the deck page on a wide window: the deck's entries, zone and
//! profile, the filter, the group and board rows, the selection the keys
//! move, the quantity bursts, moves, stars, printings and undo, each saved
//! as it happens. [`use_deck_editor`] holds it all, and [`row_view`] draws
//! one row with the actions its kind allows.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use components::keys::{focus_by_id, use_undo_key};
use components::swipe_stack::{Swipe, use_swipe_key_handler};
use components::{DeckEditActions, DeckHoverKeys, DeckRowHost, TOAST_QUICK, use_deck_hover_keys};
use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
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
use crate::inbound::cards::card_row::{CardOverlays, ShowRowArt};
use crate::inbound::cards::otag::{OtagDescribe, OtagExamplesOpen};
use crate::inbound::cards::stack_view::StackDetails;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, open_and_record_hint};
use crate::inbound::chrome::shortcuts::{self, CardKey, ShortcutsScreen};
use crate::inbound::chrome::shortcuts_sheet::use_offer_shortcuts;
use crate::inbound::telemetry;
use crate::outbound::timer;

/// What a key does on the deck cards screen besides the hover keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScreenKey {
    /// Focuses Quick add's search.
    QuickAdd,
    /// Opens the filter.
    Filter,
}

/// The screen action `key` stands for: `/` Quick add, `f` the filter ([`shortcuts::card_key`]).
/// Flipping a card is `v`, a hover key.
fn screen_key(key: &str) -> Option<ScreenKey> {
    if shortcuts::card_key(key) == Some(CardKey::Filter) {
        return Some(ScreenKey::Filter);
    }
    match key {
        "/" => Some(ScreenKey::QuickAdd),
        _ => None,
    }
}

use zwipe_ui::deck_cards::keys::RowKind;
pub(crate) use zwipe_ui::deck_cards::listing::{Listed, RowMeta as Meta, board_of, list_rows};

/// The engine's state and actions. `Copy`: every field is a signal, a memo
/// or a callback, which compare by identity.
#[derive(Clone, Copy, PartialEq)]
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
    /// The face each card shows, the front where it has none: its row's art and preview, its
    /// stack image and its details follow it, and Flip and `v` turn it.
    pub faces: Signal<HashMap<Uuid, usize>>,
    pub printing: Signal<Option<(Card, Option<Slot>)>>,
    pub printing_open: Signal<bool>,
    pub filter_open: Signal<bool>,
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

/// What a move takes: the card, its target board, whether it records undo, the star to put
/// back once it lands, and an undo's toast and entry (raised on landing, put back on failure).
type MoveCard = (
    Uuid,
    Board,
    bool,
    Option<DateTime<Utc>>,
    Option<(String, UndoAction)>,
);

/// Who sets a star: a toggle, which records undo, or an undo, which toasts `message` once the
/// server takes it.
#[derive(Debug, Clone, PartialEq)]
enum StarBy {
    /// The Star button or Shift+S.
    Toggle,
    /// Undo, with its toast and its entry, which goes back on the log if the write fails.
    Undo(String, Box<UndoAction>),
    /// Undo's move back putting the star back once the move has landed: no record and no toast
    /// of its own; a refusal toasts the server's word and leaves the card unstarred.
    Quiet,
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
    // The screen's image preview, which `i` opens for a card that shows no Image button.
    let overlays = try_use_context::<CardOverlays>();
    let authed = use_authed();
    let usage = telemetry::use_usage();
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
    let faces: Signal<HashMap<Uuid, usize>> = use_signal(HashMap::new);
    // The Stacks view's reveal, which `d` moves as a click does; absent where the screen has no
    // stacks.
    let stack_details = try_use_context::<StackDetails>();
    let mut bursts = use_signal(Bursts::default);
    let mut removed_rows: Signal<HashMap<Uuid, DeckEntry>> = use_signal(HashMap::new);
    let mut printing: Signal<Option<(Card, Option<Slot>)>> = use_signal(|| None);
    let mut printing_open = use_signal(|| false);
    let filter_open = use_signal(|| false);
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
            let fetch = |c: ZwipeClient, s: Session| async move { c.fetch_deck(deck_id, &s).await };
            // Quiet: the deck view's own profile fetch raises the one toast.
            let fetched = authed.run_quiet("load_deck", fetch).await;
            let Some(fetched) = fetched else {
                return;
            };
            let held = Zone::of_deck(&fetched);
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
    let meta = use_memo(move || deck_cards::listing::meta_of(&entries.read()));

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

    // A quantity undo's toast and entry, by card, until its debounced write lands: the toast
    // shows then, and a failed write puts the entry back on the log so the undo can be tried
    // again.
    let mut undo_retry: Signal<HashMap<Uuid, (String, UndoAction)>> = use_signal(HashMap::new);
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
            usage
                .peek()
                .record_removal(deck_id, entry.card.scryfall_data.oracle_id);
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
            let retry = undo_retry.write().remove(&card_id);
            let mut put_back = {
                let retry = retry.clone();
                move || {
                    if let Some((_, entry)) = retry.clone() {
                        undo.write().push(deck_id, entry);
                    }
                }
            };
            let landed = move || {
                if let Some((message, _)) = retry.clone() {
                    toast.info(message, ToastOptions::default().duration(TOAST_QUICK));
                }
            };
            match burst.write() {
                Write::Nothing => landed(),
                Write::Delete => {
                    let reply = authed
                        .try_run("change_quantity", |c, s| async move {
                            c.delete_deck_card(deck_id, card_id, &s).await
                        })
                        .await;
                    if reply.is_ok() {
                        landed();
                    } else {
                        put_back();
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
                        .map_or_default(|e| e.card.scryfall_data.name.clone());
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
                        Ok(_) => landed(),
                        Err(_) => {
                            put_back();
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

    // Sets a row's star time at once and patches the star to match. A toggle records undo once
    // the server takes it; an undo raises its toast then, so a refusal shows alone.
    let set_star = use_callback(
        move |(card_id, at, by): (Uuid, Option<DateTime<Utc>>, StarBy)| {
            let Some(entry) = entries
                .peek()
                .iter()
                .find(|e| e.deck_card.scryfall_data_id == card_id)
                .cloned()
            else {
                return;
            };
            let old = entry.deck_card.mvp_at;
            let card_name = entry.card.scryfall_data.name;
            let mut set_mvp = move |at| {
                if let Some(e) = entries
                    .write()
                    .iter_mut()
                    .find(|e| e.deck_card.scryfall_data_id == card_id)
                {
                    e.deck_card.mvp_at = at;
                }
            };
            set_mvp(at);
            spawn(async move {
                let request = HttpPatchDeckCard::with_mvp(at.is_some());
                let name = match by {
                    StarBy::Toggle => "toggle_mvp",
                    StarBy::Undo(..) | StarBy::Quiet => "undo",
                };
                let reply = authed
                    .try_run(name, |c, s| async move {
                        c.update_deck_card(deck_id, card_id, &request, &s).await
                    })
                    .await;
                if let Ok(saved) = reply {
                    set_mvp(saved.mvp_at);
                    match by {
                        StarBy::Toggle => undo.write().push(
                            deck_id,
                            UndoAction::MvpToggled {
                                card_id,
                                card_name,
                                before: old,
                            },
                        ),
                        StarBy::Undo(message, _) => {
                            toast.info(message, ToastOptions::default().duration(TOAST_QUICK));
                        }
                        StarBy::Quiet => {}
                    }
                } else {
                    set_mvp(old);
                    // A failed undo goes back on the log, to try again.
                    if let StarBy::Undo(_, entry) = by {
                        undo.write().push(deck_id, *entry);
                    }
                }
            });
        },
    );

    // A move to `target`. Off the main board a card loses its star, as the server drops it, and
    // the undo entry keeps the star time; `restar` stars it again in the same write, for undo's
    // move back, in a write of its own once the move lands, so a refused star can't take the move
    // with it; `landed` is undo's toast, raised only then.
    let move_card = use_callback(move |(card_id, target, record, restar, landed): MoveCard| {
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
        let moved = screens::deck_cards::moved_to(
            *entry.deck_card.quantity,
            &entry.card.scryfall_data.name,
            target,
        );
        let was_starred = entry.deck_card.mvp_at;
        let unstars = target != Board::Deck;
        if record {
            undo.write().push(
                deck_id,
                UndoAction::MovedBoard {
                    card_id,
                    card_name: entry.card.scryfall_data.name,
                    from,
                    starred: was_starred.filter(|_| unstars),
                },
            );
        }
        let mut set_board = move |board: Board, mvp_at: Option<DateTime<Utc>>| {
            if let Some(e) = entries
                .write()
                .iter_mut()
                .find(|e| e.deck_card.scryfall_data_id == card_id)
            {
                e.deck_card.board = board;
                e.deck_card.mvp_at = mvp_at;
            }
        };
        set_board(target, if unstars { None } else { was_starred });
        spawn(async move {
            let request = HttpPatchDeckCard::new(None, Some(target.display_name().to_owned()));
            let reply = authed
                .try_run("move_board", |c, s| async move {
                    c.update_deck_card(deck_id, card_id, &request, &s).await
                })
                .await;
            // The reply carries the star as the server left it.
            if let Ok(saved) = reply {
                set_board(target, saved.mvp_at);
                if let Some((message, _)) = landed {
                    toast.info(message, ToastOptions::default().duration(TOAST_QUICK));
                }
                if restar.is_some() {
                    set_star.call((card_id, restar, StarBy::Quiet));
                }
            } else {
                set_board(from, was_starred);
                // A failed undo goes back on the log, to try again.
                if let Some((_, entry)) = landed {
                    undo.write().push(deck_id, entry);
                }
            }
        });
        if record {
            toast.info(moved, ToastOptions::default().duration(TOAST_QUICK));
        }
    });
    let move_to_board = use_callback(move |(card_id, target, record): (Uuid, Board, bool)| {
        move_card.call((card_id, target, record, None, None));
    });

    let toggle_mvp = use_callback(move |(card_id, starred): (Uuid, bool)| {
        // A star adds or takes a featured card, which moves the cards under the strip.
        components::settle::watch();
        set_star.call((card_id, (!starred).then(Utc::now), StarBy::Toggle));
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
                    components::deck_cursor::follow_card(old_id, new_id);
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
                components::deck_cursor::follow_card(old_id, new_id);
                // The open details follow the card to its new printing.
                if let Some(details) = stack_details {
                    details.rekey(old_id, new_id);
                }
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
        // An undo can add or take a featured card and move the cards under it.
        components::settle::watch();
        let popped = undo.write().pop(deck_id);
        let Some(action) = popped else {
            return;
        };
        let quick = ToastOptions::default().duration(TOAST_QUICK);
        if deck_cards::is_stale(&action, &entries.peek(), &zone.peek()) {
            toast.info("Already changed".to_string(), quick);
            return;
        }
        if matches!(
            action,
            UndoAction::MvpToggled {
                before: Some(_),
                ..
            }
        ) && deck_cards::mvp_count(&entries.peek()) >= deck_cards::MAX_MVPS
        {
            toast.info(screens::deck_cards::MVP_CAP_REACHED.to_string(), quick);
            return;
        }
        // The toast, raised once the undo's write lands.
        let message = deck_cards::undo_message(&action);
        // The entry, to put back on the log if its write fails, so the undo can be tried again.
        let retry = action.clone();
        let mut put_back = {
            let retry = retry.clone();
            move || undo.write().push(deck_id, retry.clone())
        };
        match action {
            UndoAction::Added { card_id, .. } => {
                let row = entries
                    .peek()
                    .iter()
                    .find(|e| e.deck_card.scryfall_data_id == card_id)
                    .cloned();
                entries
                    .write()
                    .retain(|e| e.deck_card.scryfall_data_id != card_id);
                spawn(async move {
                    let deleted = authed
                        .run("undo", |c, s| async move {
                            c.delete_undone_deck_card(deck_id, card_id, &s).await
                        })
                        .await;
                    if deleted.is_some() {
                        toast.info(message, quick);
                    } else {
                        if let Some(row) = row {
                            entries.write().push(row);
                        }
                        put_back();
                    }
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
                )
                .with_mvp_at(entry.deck_card.mvp_at);
                entries.write().push(entry);
                spawn(async move {
                    let reply = authed
                        .try_run("undo", |c, s| async move {
                            c.create_deck_card(deck_id, &request, &s).await
                        })
                        .await;
                    if let Ok(saved) = reply {
                        if let Some(e) = entries
                            .write()
                            .iter_mut()
                            .find(|e| e.deck_card.scryfall_data_id == card_id)
                        {
                            e.deck_card = saved;
                        }
                        toast.info(message, quick);
                    } else {
                        entries
                            .write()
                            .retain(|e| e.deck_card.scryfall_data_id != card_id);
                        put_back();
                    }
                });
            }
            UndoAction::QuantityChanged { card_id, net, .. } => {
                undo_retry.write().insert(card_id, (message, retry));
                change_quantity.call((card_id, net.saturating_neg(), /* record */ false));
            }
            UndoAction::MovedBoard {
                card_id,
                from,
                starred,
                ..
            } => {
                // Back on the main board the star follows once the move lands, unless the deck
                // already has its three MVPs; the toast waits for the move too.
                let wants_star = starred.filter(|_| from == Board::Deck);
                let capped = wants_star.is_some()
                    && deck_cards::mvp_count(&entries.peek()) >= deck_cards::MAX_MVPS;
                if capped {
                    toast.info(
                        screens::deck_cards::MVP_CAP_REACHED.to_string(),
                        ToastOptions::default().duration(TOAST_QUICK),
                    );
                }
                move_card.call((
                    card_id,
                    from,
                    /* record */ false,
                    wants_star.filter(|_| !capped),
                    Some((message, retry)),
                ));
            }
            UndoAction::MvpToggled {
                card_id, before, ..
            } => {
                set_star.call((card_id, before, StarBy::Undo(message, Box::new(retry))));
            }
            UndoAction::PrintingChanged { old_card, new_id } => {
                let old_card = *old_card;
                let old_id = old_card.scryfall_data.id;
                components::deck_cursor::follow_card(new_id, old_id);
                if let Some(details) = stack_details {
                    details.rekey(new_id, old_id);
                }
                let mut shown = None;
                if let Some(e) = entries
                    .write()
                    .iter_mut()
                    .find(|e| e.deck_card.scryfall_data_id == new_id)
                {
                    shown = Some(std::mem::replace(&mut e.card, old_card));
                    e.deck_card.scryfall_data_id = old_id;
                }
                if *selected.peek() == Some(new_id) {
                    selected.set(Some(old_id));
                }
                let request = HttpPatchDeckCard::with_printing(&old_id.to_string());
                spawn(async move {
                    let saved = authed
                        .run("undo", |c, s| async move {
                            c.update_deck_card(deck_id, new_id, &request, &s).await
                        })
                        .await;
                    if saved.is_some() {
                        toast.info(message, quick);
                    } else {
                        // The printing the server still holds comes back on the row.
                        if let Some(card) = shown
                            && let Some(e) = entries
                                .write()
                                .iter_mut()
                                .find(|e| e.deck_card.scryfall_data_id == old_id)
                        {
                            e.card = card;
                            e.deck_card.scryfall_data_id = new_id;
                        }
                        put_back();
                    }
                });
            }
            UndoAction::CommandZonePrintingChanged {
                slot,
                old_card,
                new_id,
            } => {
                let old_id = old_card.scryfall_data.id;
                components::deck_cursor::follow_card(new_id, old_id);
                let shown = zone.write().slot_mut(slot).replace(*old_card);
                let request = slot.update_request(Some(old_id));
                spawn(async move {
                    let saved = authed
                        .run("undo", |c, s| async move {
                            c.update_deck_profile(deck_id, &request, &s).await
                        })
                        .await;
                    if saved.is_some() {
                        toast.info(message, quick);
                    } else {
                        *zone.write().slot_mut(slot) = shown;
                        put_back();
                    }
                });
            }
        }
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

    // On the computer layout the keyboard cursor takes the arrows (`components::deck_cursor`);
    // narrower, they step the open row.
    let computer = components::use_computer();
    use_swipe_key_handler(
        vec![Swipe::Up, Swipe::Down],
        Callback::new(move |swipe: Swipe| {
            if *computer.peek() {
                return;
            }
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
    // Shift+H opens the shortcuts sheet with this screen's keys.
    use_offer_shortcuts(ShortcutsScreen::DeckCards);
    // `p`, `i`, `v`, `d`, `m`, `s`, Shift+S, `-` and `+` act on the card under the cursor, else
    // the open one, as its own buttons and a click on it do; `u` undoes, as the dock's Undo does.
    use_deck_hover_keys(DeckHoverKeys {
        row_of: Callback::new(move |id: Uuid| {
            let row = listing
                .peek()
                .rows
                .iter()
                .find(|r| r.card.scryfall_data.id == id)
                .cloned()?;
            let board = match row.kind {
                RowKind::Board(board) => board,
                RowKind::Main { .. } | RowKind::Zone(_) | RowKind::Token => {
                    board_of(&meta.peek(), id)
                }
            };
            Some((row, board))
        }),
        faces,
        details: stack_details,
        actions: Some(DeckEditActions {
            change_quantity,
            move_to_board,
            toggle_mvp,
        }),
        on_printing: Callback::new(move |picked: (Card, Option<Slot>)| {
            printing.set(Some(picked));
            printing_open.set(true);
        }),
        on_image: Callback::new(move |shown: (ScryfallData, usize)| {
            if let Some(CardOverlays { mut preview, .. }) = overlays {
                preview.set(Some(shown));
            }
        }),
        other_keys: Some(Callback::new(move |key: String| {
            if shortcuts::undo_letter(&key) {
                apply_undo.call(());
                return;
            }
            match screen_key(&key) {
                Some(ScreenKey::QuickAdd) => focus_by_id(QUICK_ADD_ID),
                Some(ScreenKey::Filter) => {
                    let mut filter_open = filter_open;
                    filter_open.set(true);
                }
                None => {}
            }
        })),
    });
    use_undo_key(Callback::new(move |()| apply_undo.call(())));

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
        faces,
        printing,
        printing_open,
        filter_open,
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
    }
}

/// The deck's edits as the shared deck cards view takes them.
impl Editor {
    /// The quantity, board and star edits.
    pub(crate) fn edit_actions(&self) -> DeckEditActions {
        DeckEditActions {
            change_quantity: self.change_quantity,
            move_to_board: self.move_to_board,
            toggle_mvp: self.toggle_mvp,
        }
    }
}

/// The deck cards' host buttons: the screen's image preview and view-only printings
/// ([`CardOverlays`]), `ed`'s own printings sheet, and the Oracle tags' descriptions and examples
/// where the screen provides them. Call after the screen provides those contexts.
pub(crate) fn use_deck_row_host(ed: &Editor) -> DeckRowHost {
    let overlays = try_use_context::<CardOverlays>();
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let Editor {
        mut printing,
        mut printing_open,
        ..
    } = *ed;
    let on_printing = use_callback(move |(card, slot): (Card, Option<Slot>)| {
        printing.set(Some((card, slot)));
        printing_open.set(true);
    });
    let on_image = use_callback(move |(sd, face): (ScryfallData, usize)| {
        if let Some(CardOverlays { mut preview, .. }) = overlays {
            preview.set(Some((sd, face)));
        }
    });
    let on_view_printing = use_callback(move |card: Card| {
        if let Some(CardOverlays { printing, .. }) = overlays {
            let mut printing = printing;
            printing.set(Some(card));
        }
    });
    DeckRowHost {
        on_image: overlays.is_some().then_some(on_image),
        on_printing: Some(on_printing),
        on_view_printing: overlays.is_some().then_some(on_view_printing),
        describe_tag,
        on_examples,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slash_focuses_quick_add_and_f_opens_the_filter_as_everywhere() {
        assert_eq!(screen_key("/"), Some(ScreenKey::QuickAdd));
        for byte in b' '..=b'~' {
            let key = char::from(byte).to_string();
            assert_eq!(
                screen_key(&key) == Some(ScreenKey::Filter),
                shortcuts::card_key(&key) == Some(CardKey::Filter),
                "{key}"
            );
        }
    }
}

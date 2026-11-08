//! Add cards: swipe search results or the maybeboard into the deck.
//!
//! Right adds a copy to the main deck, left skips the card for a week, up
//! sends it to the maybeboard and down undoes, by drag, the arrow keys or
//! the four buttons under the card. On a wide window the card's details sit
//! beside it, and the filter opens as the drawer from the funnel;
//! below that it is a side panel behind `Filter`. Ctrl+Z also undoes, F
//! flips the card, P opens its printings and R is `Refresh`.
//!
//! The search stack and the filters live in the root stores, so leaving and
//! coming back to the same deck resumes where swiping stopped.

use std::collections::HashSet;

use components::icons::{BoltIcon, DockIcon, REFRESH};
use components::{
    Chip, DockFilter, LoadFailed, SheetFrame, SheetWide, TOAST_NORMAL, TOAST_QUICK,
    reset_image_ease, use_back_online,
};
use dioxus::core::spawn_forever;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::{Board, DeckEntry};
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpPatchDeckCard};
use zwipe_core::http::contracts::metrics::PileEndKind;
use zwipe_ui::add::{AddDeck, normalized, related_ids, synergy_note};
use zwipe_ui::deck_signals::{Crossing, Level, budget_crossing, land_crossing, price_of};
use zwipe_ui::filter::AddContext;
use zwipe_ui::filter::draft::{ResetRule, SheetConfig};
use zwipe_ui::generation::Generation;
use zwipe_ui::hint::Role;
use zwipe_ui::screens::add_cards;
use zwipe_ui::stack::{
    AddAction, CARDS_WARNING_THRESHOLD, CardStack, MAX_CARDS_IN_STACK, MaybeboardAction,
    NO_CARDS_LEFT, NO_CARDS_MATCH, STACK_WARNING_EVERY, StackAction,
};
use zwipe_ui::stores::filters::FilterScope;
use zwipe_ui::stores::in_flight::AddOutcome;
use zwipe_ui::stores::parked::ParkedStack;
use zwipe_ui::stores::undo;
use zwipe_ui::stores::undo::UndoAction;
use zwipe_ui::telemetry::vocabulary::{DeckScreen, Screen};
use zwipe_ui::timing::LOAD_MORE_AT;

use super::{OneLine, SwipeDetails, flip_top_card};
use crate::inbound::card_filter::CardFilterPanel;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_info::{CardDetailsSheet, CardSkeleton};
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::cards::swipe_confirm::{SwipeScreenChips, Tone, use_swipe_confirm};
use crate::inbound::cards::swipe_stack::{
    Swipe, SwipeDock, SwipeStack, use_swipe_control, use_swipe_keys_or_undo,
};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::details_eye::{DockEye, Surface, use_details_eye};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::chrome::page_keys::{use_page_keys, use_undo_key};
use crate::inbound::root_signal::use_signal_for_root_tasks;
use crate::inbound::session::ensure_fresh::ensure_fresh;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;

/// A downward pull this long, in pixels, on the empty end of the pile
/// undoes the last swipe, as in the app.
const BACK_PULL_PX: f64 = 60.0;

/// What Add swipes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddSource {
    /// Search results from the server.
    Search,
    /// The deck's own maybeboard.
    Maybeboard,
}

impl AddSource {
    /// The other source, for the From chip.
    fn toggled(self) -> Self {
        match self {
            Self::Search => Self::Maybeboard,
            Self::Maybeboard => Self::Search,
        }
    }

    /// The source's name on the From chip, the app's.
    fn label(self) -> &'static str {
        match self {
            Self::Search => add_cards::Source::Search.label(),
            Self::Maybeboard => add_cards::Source::Maybeboard.label(),
        }
    }

    /// The filter store scope each source remembers its filter under.
    fn scope(self) -> FilterScope {
        match self {
            Self::Search => FilterScope::AddSearch,
            Self::Maybeboard => FilterScope::AddMaybeboard,
        }
    }
}

/// The swipe directions every Add stack takes.
const ALL_SWIPES: [Swipe; 4] = [Swipe::Left, Swipe::Right, Swipe::Up, Swipe::Down];

/// The Add cards screen for `deck_id`.
#[component]
pub fn AddDeckCard(deck_id: Uuid) -> Element {
    telemetry::use_telemetry_screen(Screen::Deck(DeckScreen::CardAdd));
    let toast = use_toast();
    let confirm = use_swipe_confirm();
    let authed = use_authed();
    let usage = telemetry::use_usage();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let CardStores {
        mut filters,
        mut stacks,
        mut undo,
        mut in_flight,
        add_stack: mut stack,
        mut last_search,
    } = use_context();

    let restored = use_hook(|| filters.peek().restore(FilterScope::AddSearch, deck_id));
    let had_restored = restored.is_some();
    let mut filter = use_signal(|| restored.clone().unwrap_or_default());
    let mut deck: Signal<Option<AddDeck>> = use_signal_for_root_tasks(|| None);
    let mut source = use_signal(|| AddSource::Search);
    let mut counter = use_signal(|| 0u32);
    let mut mb_stack: Signal<CardStack<MaybeboardAction>> = use_signal(CardStack::default);
    let mut warming = use_signal(|| false);
    let mut warm_before = use_signal(|| false);
    let mut related: Signal<HashSet<Uuid>> = use_signal(HashSet::new);
    let mut related_announced = use_signal(|| false);
    let mut exhausted = use_signal(|| false);
    let mut loading_cards = use_signal(|| true);
    // The loads that failed, which Retry and the connection coming back
    // send again: the deck's context, and the search from the top.
    let mut deck_failed = use_signal(|| false);
    let mut search_failed = use_signal(|| false);
    let mut loading_more = use_signal(|| false);
    let mut searches = use_signal(Generation::default);
    let mut first_run = use_signal(|| true);
    let mut show_details = use_signal(|| false);
    let eye = use_details_eye(show_details, Surface::Window);
    let mut printing_open = use_signal(|| false);
    let mut filter_open = use_signal(|| false);
    let mut pull_from: Signal<Option<(f64, f64)>> = use_signal(|| None);
    let mut hint_open = use_one_time_hint(OneTimeHint::AddDeckCards);
    let control = use_swipe_control();

    let context = move || -> AddContext {
        deck.peek()
            .as_ref()
            .map(AddDeck::context)
            .unwrap_or_default()
    };
    let holds = move |card: &Card| deck.peek().as_ref().is_some_and(|d| d.holds(card));
    let mut bump = move || {
        let next = counter.peek().wrapping_add(1);
        counter.set(next);
    };

    // Leaving parks the filter under the live source and the search stack
    // under the filter that produced it.
    use_drop(move || {
        let (Ok(at), Ok(applied)) = (source.try_peek(), filter.try_peek()) else {
            return;
        };
        filters.write().park(at.scope(), deck_id, applied.clone());
        let parked_filter = last_search.peek().clone();
        let was_exhausted = exhausted.try_peek().is_ok_and(|e| *e);
        if let Some(parked_filter) = parked_filter
            && stack.peek().has_served()
        {
            let (cards, index, history) = stack.peek().park_state();
            stacks.write().park(
                deck_id,
                ParkedStack {
                    cards,
                    index,
                    history,
                    filter: parked_filter,
                    exhausted: was_exhausted,
                },
            );
        }
    });

    // The deck's context: what it holds, its land and price totals, and the
    // legality, colors, synergy and land rules the search starts from.
    // Until it lands the pile ghosts; a failure toasts once and leaves
    // Couldn't load cards with a Retry.
    let fetch_context = use_callback(move |()| {
        deck_failed.set(false);
        if deck.peek().is_none() {
            loading_cards.set(true);
        }
        spawn(async move {
            let Some(full) = authed
                .run("load_deck_context", |c, s| async move {
                    c.fetch_deck(deck_id, &s).await
                })
                .await
            else {
                deck_failed.set(true);
                if deck.peek().is_none() {
                    loading_cards.set(false);
                }
                return;
            };
            let add = AddDeck::from_deck(&full);
            add.seed(&mut filter.write(), had_restored);
            deck.set(Some(add));
            if !stack.peek().has_served() {
                bump();
            }
        });
    });
    use_hook(move || fetch_context.call(()));
    // Sends again whatever failed, from Retry or the connection coming back.
    let retry = use_callback(move |()| {
        if *deck_failed.peek() {
            fetch_context.call(());
        }
        if *search_failed.peek() {
            bump();
        }
    });
    use_back_online(retry);

    // The search: on mount, resuming a parked stack when its filter still
    // stands, and on every bump while the source is Search.
    use_effect(move || {
        let _bumped = counter();
        if *source.peek() != AddSource::Search {
            return;
        }
        let builder = normalized(&filter.peek());
        let first = *first_run.peek();
        if first {
            first_run.set(false);
            let parked = stacks.write().take(deck_id);
            if let Some(parked) = parked
                && normalized(&parked.filter) == builder
                && !parked.cards.is_empty()
            {
                exhausted.set(parked.exhausted);
                last_search.set(Some(parked.filter));
                stack
                    .write()
                    .restore(parked.cards, parked.index, parked.history);
                return;
            }
        }
        let ticket = searches.write().next_ticket();
        // Until the deck lands the search can't run, and the stack ghosts.
        loading_cards.set(deck.peek().is_none() && !*deck_failed.peek());
        search_failed.set(false);
        stack.write().clear_ahead();
        reset_image_ease();
        last_search.set(None);
        exhausted.set(false);
        if deck.peek().is_some() && !context().serves(&filter.peek()) {
            toast.warning(
                add_cards::TRY_A_FILTER.to_string(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
            return;
        }
        let Ok(query) = builder.build() else {
            // Nothing to search on: once the deck is in, say so rather than
            // leave a blank ghost.
            if deck.peek().is_some() {
                toast.warning(
                    add_cards::TRY_A_FILTER.to_string(),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
            }
            return;
        };
        loading_cards.set(true);
        let snapshot = filter.peek().clone();
        // Ahead of the deck, the deck's own load toasts a failure for both.
        let quiet = deck.peek().is_none();
        usage.peek().record_search();
        spawn(async move {
            // Buffered usage feeds synergy ordering, so it goes out before
            // the search.
            telemetry::flush_once(usage, session, client).await;
            let call = |c: ZwipeClient, s: Session| async move {
                c.search_deck_cards(deck_id, &query, &s).await
            };
            let reply = if quiet {
                authed.try_run_quiet("search_cards", call).await
            } else {
                authed.try_run("search_cards", call).await
            };
            if !searches.peek().is_current(ticket) {
                return;
            }
            loading_cards.set(false);
            let Ok(page) = reply else {
                search_failed.set(true);
                return;
            };
            warming.set(!page.synergy_applied);
            related.set(related_ids(&page));
            related_announced.set(false);
            let served: Vec<Card> = page.cards.into_iter().filter(|c| !holds(c)).collect();
            // An empty reply needs no toast: the ghost card says No cards
            // match.
            if served.is_empty() {
                usage
                    .peek()
                    .record_pile_end(deck_id, PileEndKind::Empty, snapshot.synergy());
            }
            stack.write().replace_ahead(served);
            last_search.set(Some(snapshot));
        });
    });

    // The maybeboard source, filtered and sorted in memory on every bump.
    use_effect(move || {
        let _bumped = counter();
        if *source.peek() != AddSource::Maybeboard {
            return;
        }
        let cards: Vec<Card> = deck
            .peek()
            .as_ref()
            .map(|d| d.maybeboard.iter().map(|e| e.card.clone()).collect())
            .unwrap_or_default();
        let applied = filter.peek().clone();
        let mut shown = Cards::from(cards);
        if !applied.is_empty()
            && let Ok(criteria) = applied.build_criteria()
        {
            shown = shown.matching(&criteria);
        }
        mb_stack
            .write()
            .replace(shown.sorted(applied.sort(), applied.direction()).into());
    });

    // Synergy turning cold says so once per change.
    use_effect(move || {
        let now = warming();
        let before = *warm_before.peek();
        warm_before.set(now);
        if now && !before {
            toast.info(
                "Synergy warming up; showing all cards for now".to_string(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
        }
    });

    // The first related card to reach the top says the synergy picks ran out.
    use_effect(move || {
        let top = stack
            .read()
            .current()
            .map(|c| c.card_profile.scryfall_data_id);
        let Some(top) = top else {
            return;
        };
        if *source.peek() == AddSource::Search
            && !*related_announced.peek()
            && related.peek().contains(&top)
        {
            related_announced.set(true);
            usage
                .peek()
                .record_pile_end(deck_id, PileEndKind::SynergyExhausted, true);
            toast.info(
                "That's every synergy pick. Showing cards like your deck".to_string(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
        }
    });

    let load_more = use_callback(move |()| {
        if stack.peek().len() >= MAX_CARDS_IN_STACK {
            toast.warning(
                add_cards::STACK_LIMIT_REACHED.to_owned(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
            return;
        }
        if *loading_more.peek() {
            return;
        }
        let Ok(query) = normalized(&filter.peek()).build() else {
            return;
        };
        loading_more.set(true);
        let ticket = searches.peek().current();
        let synergy = filter.peek().synergy();
        usage.peek().record_search();
        spawn(async move {
            let reply = authed
                .try_run("load_more", |c, s| async move {
                    c.search_deck_cards(deck_id, &query, &s).await
                })
                .await;
            if !searches.peek().is_current(ticket) {
                loading_more.set(false);
                return;
            }
            if let Ok(page) = reply {
                warming.set(!page.synergy_applied);
                related.write().extend(related_ids(&page));
                let seen: HashSet<Uuid> = stack
                    .peek()
                    .cards()
                    .iter()
                    .map(|c| c.card_profile.scryfall_data_id)
                    .collect();
                let fresh: Vec<Card> = page
                    .cards
                    .into_iter()
                    .filter(|c| !seen.contains(&c.card_profile.scryfall_data_id) && !holds(c))
                    .collect();
                if fresh.is_empty() {
                    exhausted.set(true);
                    usage
                        .peek()
                        .record_pile_end(deck_id, PileEndKind::Empty, synergy);
                    // The cursor waits past the last card: the stack has run out.
                    if stack.peek().index() >= stack.peek().len() {
                        toast.info(
                            NO_CARDS_LEFT.to_owned(),
                            ToastOptions::default().duration(TOAST_QUICK),
                        );
                    }
                } else {
                    stack.write().append(fresh);
                }
            }
            loading_more.set(false);
        });
    });

    // Moves past the swiped card and returns what that says: the stack
    // nearing its cap, or that it ran out.
    let advance = use_callback(move |()| {
        let mut said = Vec::new();
        let total = stack.peek().len();
        if !stack.write().advance() {
            return said;
        }
        let index = stack.peek().index();
        if (CARDS_WARNING_THRESHOLD..MAX_CARDS_IN_STACK).contains(&total)
            && total.checked_rem(STACK_WARNING_EVERY) == Some(0)
        {
            said.push((Level::Info, add_cards::STACK_LIMIT_NEAR.to_owned()));
        }
        if index >= total && *exhausted.peek() {
            said.push((Level::Info, NO_CARDS_LEFT.to_owned()));
        } else if total > 0 && index.saturating_add(1) >= total.saturating_sub(LOAD_MORE_AT) {
            load_more.call(());
        }
        said
    });

    // Sends one copy to the main deck or the maybeboard. The answer can
    // land after the screen is gone, so it runs in the root.
    let add_card = use_callback(move |(card, board): (Card, Board)| {
        let card_id = card.scryfall_data.id;
        let maybe = board.is_maybeboard();
        let board_name = maybe.then(|| Board::Maybeboard.display_name().to_owned());
        let request = HttpCreateDeckCard::new(&card.scryfall_data, 1, board_name);
        in_flight.write().begin(card_id);
        let op = if maybe { "add_maybeboard" } else { "add_card" };
        spawn_forever(async move {
            let reply = authed
                .run(op, |c, s| async move {
                    c.create_deck_card(deck_id, &request, &s).await
                })
                .await;
            let outcome = in_flight.write().finish(card_id);
            let Some(deck_card) = reply else {
                return;
            };
            if outcome == AddOutcome::Undo {
                authed
                    .run("undo_add", |c, s| async move {
                        c.delete_deck_card(deck_id, card_id, &s).await
                    })
                    .await;
                return;
            }
            let card_name = card.scryfall_data.name.clone();
            if let Ok(mut held) = deck.try_write()
                && let Some(d) = held.as_mut()
            {
                if let Some(oracle_id) = card.scryfall_data.oracle_id {
                    d.in_deck.insert(oracle_id);
                }
                if maybe {
                    d.maybeboard.push(DeckEntry { card, deck_card });
                }
            }
            undo.write()
                .push(deck_id, UndoAction::Added { card_id, card_name });
        });
    });

    // The running land count and price after an add or its undo, returning
    // the upward crossings' signals.
    let mut count_add = move |card: &Card, sign: i32| {
        let mut signals = Vec::new();
        if let Some(d) = deck.write().as_mut() {
            if card.scryfall_data.is_land() {
                let before = d.land_count;
                d.land_count = before.saturating_add(sign).max(0);
                if sign > 0 {
                    signals.push(land_crossing(
                        d.land_target,
                        before,
                        d.land_count,
                        Crossing::Up,
                    ));
                }
            }
            if d.budget.is_some_and(|b| b > 0.0) {
                let before = d.total_price;
                let price = price_of(card, d.currency) * f64::from(sign);
                d.total_price = (before + price).max(0.0);
                if sign > 0 {
                    signals.push(budget_crossing(d.budget, d.currency, before, d.total_price));
                }
            }
        }
        signals
    };

    let undo_search = use_callback(move |()| {
        let popped = stack.write().pop_action();
        let Some(action) = popped else {
            toast.info("Nothing to undo".to_string(), ToastOptions::default());
            return;
        };
        let stepped = stack.write().step_back(&action);
        if !stepped {
            toast.warning("No previous card".to_string(), ToastOptions::default());
            stack.write().record(action);
            return;
        }
        let current = stack.peek().current().cloned();
        let Some(card) = current else {
            stack.write().unwind_undo(action);
            return;
        };
        let card_id = card.scryfall_data.id;
        if action == AddAction::Skip {
            if let Some(oracle_id) = card.scryfall_data.oracle_id {
                spawn(async move {
                    authed
                        .run("undo_skip", |c, s| async move {
                            c.unskip_deck_card(deck_id, oracle_id, &s).await
                        })
                        .await;
                });
            }
            confirm.info("Undid skip");
            return;
        }
        let maybe = action == AddAction::Maybe;
        let message = if maybe {
            "Undid maybeboard"
        } else {
            "Undid add"
        };
        if in_flight.write().undo(card_id) {
            if !maybe {
                count_add(&card, -1);
            }
            confirm.success(message);
            return;
        }
        let taken = undo.write().take_newest(
            deck_id,
            |a| matches!(a, UndoAction::Added { card_id: id, .. } if *id == card_id),
        );
        if taken.is_none() && !holds(&card) {
            confirm.success(message);
            return;
        }
        spawn(async move {
            let fresh = match ensure_fresh(session, client).await {
                Ok(fresh) => fresh,
                Err(e) => {
                    toast.error(e.to_user_message(), ToastOptions::default());
                    stack.write().unwind_undo(action);
                    if let Some(entry) = taken {
                        undo.write().push(deck_id, entry);
                    }
                    return;
                }
            };
            let http = client.peek().clone();
            match http.delete_deck_card(deck_id, card_id, &fresh).await {
                Ok(()) => {
                    if let Some(d) = deck.write().as_mut() {
                        if let Some(oracle_id) = card.scryfall_data.oracle_id {
                            d.in_deck.remove(&oracle_id);
                        }
                        if maybe {
                            d.maybeboard
                                .retain(|e| e.deck_card.scryfall_data_id != card_id);
                        }
                    }
                    if !maybe {
                        count_add(&card, -1);
                    }
                    confirm.success(message);
                }
                Err(e) => {
                    tracing::warn!("undo failed: {e}");
                    toast.error(undo::UNDO_FAILED.to_owned(), ToastOptions::default());
                    if let Some(entry) = taken {
                        undo.write().push(deck_id, entry);
                    }
                }
            }
        });
    });

    let undo_maybeboard = use_callback(move |()| {
        let popped = mb_stack.write().pop_action();
        let Some(action) = popped else {
            toast.info("Nothing to undo".to_string(), ToastOptions::default());
            return;
        };
        let quick = ToastOptions::default().duration(TOAST_QUICK);
        mb_stack.write().prime_entering(action.exited());
        let MaybeboardAction::Promote { card } = action else {
            if !mb_stack.write().retreat_wrapping() {
                mb_stack.write().cancel_entering();
                return;
            }
            confirm.info("Undid skip");
            return;
        };
        let card = *card;
        let card_id = card.scryfall_data.id;
        let taken = undo.write().take_newest(deck_id, |a| {
            matches!(a, UndoAction::MovedBoard { card_id: id, from: Board::Maybeboard, .. } if *id == card_id)
        });
        let already_back = deck.peek().as_ref().is_some_and(|d| {
            d.maybeboard
                .iter()
                .any(|e| e.deck_card.scryfall_data_id == card_id)
        });
        if taken.is_none() && already_back {
            mb_stack.write().cancel_entering();
            toast.info("Already changed".to_string(), quick);
            return;
        }
        mb_stack.write().insert_current(card.clone());
        spawn(async move {
            let fresh = match ensure_fresh(session, client).await {
                Ok(fresh) => fresh,
                Err(e) => {
                    toast.error(e.to_user_message(), ToastOptions::default());
                    mb_stack.write().cancel_entering();
                    if let Some(entry) = taken {
                        undo.write().push(deck_id, entry);
                    }
                    return;
                }
            };
            let http = client.peek().clone();
            let back =
                HttpPatchDeckCard::new(None, Some(Board::Maybeboard.display_name().to_owned()));
            match http.update_deck_card(deck_id, card_id, &back, &fresh).await {
                Ok(deck_card) => {
                    if let Some(d) = deck.write().as_mut() {
                        if let Some(oracle_id) = card.scryfall_data.oracle_id {
                            d.in_deck.remove(&oracle_id);
                        }
                        d.maybeboard.push(DeckEntry { card, deck_card });
                    }
                    confirm.success("Undid move to mainboard");
                }
                Err(e) => {
                    tracing::warn!("undo failed: {e}");
                    toast.error(undo::UNDO_FAILED.to_owned(), ToastOptions::default());
                    if let Some(entry) = taken {
                        undo.write().push(deck_id, entry);
                    }
                }
            }
        });
    });

    let undo_now = use_callback(move |()| match *source.peek() {
        AddSource::Search => undo_search.call(()),
        AddSource::Maybeboard => undo_maybeboard.call(()),
    });

    let mut on_search_swipe = move |(swipe, card): (Swipe, Card)| match swipe {
        Swipe::Right => {
            stack.write().record(AddAction::Add);
            add_card.call((card.clone(), Board::Deck));
            let mut line = OneLine::default();
            line.confirm(Tone::Success, "Added to deck");
            for signal in count_add(&card, 1) {
                line.signal(signal);
            }
            for said in advance.call(()) {
                line.signal(Some(said));
            }
            line.raise(confirm, toast);
        }
        Swipe::Left => {
            if let Some(oracle_id) = card.scryfall_data.oracle_id {
                spawn_forever(async move {
                    authed
                        .run_quiet("skip_card", |c, s| async move {
                            c.skip_deck_card(deck_id, oracle_id, &s).await
                        })
                        .await;
                });
            }
            stack.write().record(AddAction::Skip);
            let mut line = OneLine::default();
            line.confirm(Tone::Info, "Skipped");
            for said in advance.call(()) {
                line.signal(Some(said));
            }
            line.raise(confirm, toast);
        }
        Swipe::Up => {
            stack.write().record(AddAction::Maybe);
            add_card.call((card, Board::Maybeboard));
            let mut line = OneLine::default();
            line.confirm(Tone::Info, "Added to maybeboard");
            for said in advance.call(()) {
                line.signal(Some(said));
            }
            line.raise(confirm, toast);
        }
        Swipe::Down => undo_search.call(()),
    };

    let mut on_maybeboard_swipe = move |(swipe, card): (Swipe, Card)| match swipe {
        Swipe::Right => {
            let card_id = card.scryfall_data.id;
            let card_name = card.scryfall_data.name.clone();
            let oracle_id = card.scryfall_data.oracle_id;
            mb_stack.write().record(MaybeboardAction::Promote {
                card: Box::new(card),
            });
            if let Some(d) = deck.write().as_mut() {
                d.maybeboard
                    .retain(|e| e.deck_card.scryfall_data_id != card_id);
                if let Some(oracle_id) = oracle_id {
                    d.in_deck.insert(oracle_id);
                }
            }
            mb_stack.write().remove_current();
            let mut line = OneLine::default();
            line.confirm_as(
                Tone::Success,
                add_cards::MOVED_TO_MAINBOARD,
                add_cards::MOVED_TO_MAINBOARD_CHIP,
            );
            if mb_stack.peek().is_empty() {
                line.ran_out();
            }
            line.raise(confirm, toast);
            spawn_forever(async move {
                let to_deck =
                    HttpPatchDeckCard::new(None, Some(Board::Deck.display_name().to_owned()));
                if authed
                    .run("promote_maybeboard", |c, s| async move {
                        c.update_deck_card(deck_id, card_id, &to_deck, &s).await
                    })
                    .await
                    .is_some()
                {
                    undo.write().push(
                        deck_id,
                        UndoAction::MovedBoard {
                            card_id,
                            card_name,
                            from: Board::Maybeboard,
                        },
                    );
                }
            });
        }
        Swipe::Left => {
            mb_stack.write().record(MaybeboardAction::Skip);
            confirm.info("Skipped");
            mb_stack.write().advance_wrapping();
        }
        Swipe::Up => {
            confirm.info("Already on maybeboard");
        }
        Swipe::Down => undo_maybeboard.call(()),
    };

    let mut switch_source = move |to: AddSource| {
        let from = *source.peek();
        if from == to {
            return;
        }
        filters
            .write()
            .park(from.scope(), deck_id, filter.peek().clone());
        let remembered = filters.peek().restore(to.scope(), deck_id);
        let next = match (to, remembered) {
            (_, Some(remembered)) => remembered,
            (AddSource::Maybeboard, None) => CardQueryBuilder::default(),
            (AddSource::Search, None) => context().fresh_search(),
        };
        filter.set(next);
        source.set(to);
        mb_stack.write().rewind();
        bump();
    };

    let refresh = use_callback(move |()| {
        let quick = ToastOptions::default().duration(TOAST_QUICK);
        if *source.peek() == AddSource::Maybeboard {
            spawn(async move {
                if let Some(full) = authed
                    .run("search_cards", |c, s| async move {
                        c.fetch_deck(deck_id, &s).await
                    })
                    .await
                {
                    if let Some(d) = deck.write().as_mut() {
                        d.maybeboard = full
                            .entries
                            .into_iter()
                            .filter(|e| e.deck_card.board.is_maybeboard())
                            .collect();
                    }
                    mb_stack.write().rewind();
                    let next = counter.peek().wrapping_add(1);
                    counter.set(next);
                }
            });
            toast.info("Maybeboard refreshed".to_string(), quick);
            return;
        }
        if deck.peek().is_none() {
            fetch_context.call(());
            return;
        }
        let Ok(query) = normalized(&filter.peek()).build() else {
            toast.warning("Filter is empty".to_string(), quick);
            return;
        };
        let ticket = searches.write().next_ticket();
        stack.write().reset();
        reset_image_ease();
        last_search.set(None);
        exhausted.set(false);
        loading_cards.set(true);
        search_failed.set(false);
        let snapshot = filter.peek().clone();
        usage.peek().record_search();
        spawn(async move {
            telemetry::flush_once(usage, session, client).await;
            let reply = authed
                .try_run("search_cards", |c, s| async move {
                    c.search_deck_cards(deck_id, &query, &s).await
                })
                .await;
            if !searches.peek().is_current(ticket) {
                return;
            }
            loading_cards.set(false);
            let Ok(page) = reply else {
                search_failed.set(true);
                return;
            };
            warming.set(!page.synergy_applied);
            related.set(related_ids(&page));
            related_announced.set(false);
            let served: Vec<Card> = page.cards.into_iter().filter(|c| !holds(c)).collect();
            stack.write().replace(served);
            last_search.set(Some(snapshot));
        });
        toast.info("Stack refreshed".to_string(), quick);
    });

    let at = source();
    let current: Option<Card> = match at {
        AddSource::Search => stack.read().current().cloned(),
        AddSource::Maybeboard => mb_stack.read().current_wrapping().cloned(),
    };
    let has_current = current.is_some();
    use_swipe_keys_or_undo(
        control,
        ALL_SWIPES.to_vec(),
        Callback::new(move |()| match *source.peek() {
            AddSource::Search => stack.peek().current().is_some(),
            AddSource::Maybeboard => mb_stack.peek().current_wrapping().is_some(),
        }),
        undo_now,
    );
    use_undo_key(Callback::new(move |()| undo_now.call(())));
    use_page_keys(Callback::new(move |key: String| match key.as_str() {
        "f" | "F" => flip_top_card(),
        "p" | "P" => {
            if *source.peek() == AddSource::Search && stack.peek().current().is_some() {
                printing_open.set(true);
            }
        }
        "r" | "R" => refresh.call(()),
        _ => {}
    }));

    let ctx = deck
        .read()
        .as_ref()
        .map(AddDeck::context)
        .unwrap_or_default();
    let has_commander = ctx.has_commander;
    let dot = ctx.shows_dot(&filter.read());
    let synergy_on = filter.read().synergy();
    let panel_config = SheetConfig {
        show_format: true,
        validate: at == AddSource::Search,
        reset: match at {
            AddSource::Search => ResetRule::AddSearch(ctx.clone()),
            AddSource::Maybeboard => ResetRule::AddMaybeboard,
        },
        ..SheetConfig::default()
    };
    let panel_key = format!("{at:?}-{}-{}", deck.read().is_some(), ctx.lands_at_target);
    let on_apply = move |applied: CardQueryBuilder| {
        filter.set(applied);
        bump();
    };
    let (window, entering, end_of_pile) = match at {
        AddSource::Search => {
            let s = stack.read();
            (
                s.window().to_vec(),
                s.entering(),
                s.has_served() && s.current().is_none(),
            )
        }
        AddSource::Maybeboard => {
            let s = mb_stack.read();
            (s.window().to_vec(), s.entering(), false)
        }
    };
    // The ghost says why when the filter leaves nothing to deal: a search
    // that came back empty, or maybeboard cards none of which match.
    let ghost_note = match at {
        AddSource::Search => {
            (!loading_cards() && last_search.read().is_some() && !stack.read().has_served())
                .then_some(NO_CARDS_MATCH)
        }
        AddSource::Maybeboard => {
            let has_maybe = deck
                .read()
                .as_ref()
                .is_some_and(|d| !d.maybeboard.is_empty());
            (has_maybe && !filter.read().is_empty() && mb_stack.read().is_empty())
                .then_some(NO_CARDS_MATCH)
        }
    };
    // In place of the ghost, with a Retry, when the cards could not load.
    let load_failed = at == AddSource::Search
        && !stack.read().has_served()
        && (search_failed() || (deck_failed() && deck.read().is_none()));
    // Under it, after an empty search with Synergy on, whether Synergy may
    // be the reason.
    let ghost_detail = if at == AddSource::Search && ghost_note.is_some() {
        let synergy = last_search
            .read()
            .as_ref()
            .is_some_and(CardQueryBuilder::synergy);
        synergy_note(synergy, warming())
    } else {
        None
    };
    let buttons = vec![
        (Swipe::Left, "Skip"),
        (Swipe::Up, "Maybeboard"),
        (Swipe::Down, "Undo"),
        (Swipe::Right, "Add"),
    ];

    rsx! {
        ScreenFrame {
            title: add_cards::TITLE,
            on_hint: move |()| hint_open.set(true),
            div { class: "screen-content card-swipe content-enter web-cards-swipe {eye.body_class()}",
                SwipeScreenChips { title: add_cards::TITLE, tone: Role::Success, confirm, at_end: current.is_none() }
                div { class: "web-cards-swipe-main",
                    div { class: "form-container",
                        if current.is_some() {
                            SwipeStack {
                                cards: window,
                                allowed: ALL_SWIPES.to_vec(),
                                control,
                                entering,
                                on_entered: move |()| match *source.peek() {
                                    AddSource::Search => stack.write().cancel_entering(),
                                    AddSource::Maybeboard => mb_stack.write().cancel_entering(),
                                },
                                on_swipe: move |pair: (Swipe, Card)| {
                                    let (swipe, card) = &pair;
                                    usage.peek().record_swipe((*swipe).into());
                                    let at = *source.peek();
                                    if at == AddSource::Search && *swipe != Swipe::Down {
                                        usage
                                            .peek()
                                            .record_signal(deck_id, card.scryfall_data.oracle_id, (*swipe).into());
                                    }
                                    match at {
                                        AddSource::Search => on_search_swipe(pair),
                                        AddSource::Maybeboard => on_maybeboard_swipe(pair),
                                    }
                                },
                            }
                        } else if end_of_pile {
                            if loading_more() {
                                CardSkeleton {}
                            } else {
                                div {
                                    class: "web-pile-end",
                                    onpointerdown: move |e: Event<PointerData>| {
                                        let p = e.client_coordinates();
                                        pull_from.set(Some((p.x, p.y)));
                                    },
                                    onpointerup: move |e: Event<PointerData>| {
                                        let Some((sx, sy)) = pull_from.take() else {
                                            return;
                                        };
                                        let p = e.client_coordinates();
                                        let (dx, dy) = (p.x - sx, p.y - sy);
                                        if dy >= BACK_PULL_PX && dy.abs() > dx.abs() {
                                            undo_search.call(());
                                        }
                                    },
                                    CardSkeleton {}
                                }
                            }
                        } else if at == AddSource::Search && loading_cards() {
                            CardSkeleton {}
                        } else if load_failed {
                            LoadFailed { message: add_cards::LOAD_FAILED, on_retry: retry }
                        } else {
                            CardSkeleton { note: ghost_note, detail: ghost_detail }
                        }
                    }
                }
                SwipeDetails {
                    card: current.clone(),
                    on_printings: (at == AddSource::Search).then(|| EventHandler::new(move |()| printing_open.set(true))),
                    loading: (end_of_pile && loading_more()) || (at == AddSource::Search && loading_cards()),
                }
            }
            // The bottom row: From and the chip naming the current source on
            // the left, a tap switching to the other; the action dock (synergy,
            // filter, refresh, card details) on the right. Back is the header's
            // arrow.
            div { class: "swipe-bottom-row",
                div { class: "swipe-bottom-chips",
                    span { class: "stat-chip stat-chip-p1 chip-row-label", "{add_cards::FROM}" }
                    Chip {
                        selected: true,
                        onclick: move |_| switch_source(at.toggled()),
                        "{at.label()}"
                    }
                }
                SwipeDock {
                    control,
                    buttons,
                    card_on_top: has_current,
                    on_undo: move |()| undo_now.call(()),
                    div { class: "action-dock",
                        // Synergy on or off: constrains the stack to the commander's
                        // synergy pool, then sorts within it. Lit while on.
                        if at == AddSource::Search && has_commander {
                            button {
                                class: if synergy_on { "action-dock-btn action-dock-on" } else { "action-dock-btn" },
                                "aria-label": "Synergy",
                                "aria-pressed": "{synergy_on}",
                                onclick: move |_| {
                                    let on = !filter.peek().synergy();
                                    filter.write().set_synergy(on);
                                    bump();
                                    let said = if on { add_cards::SYNERGY_ON } else { add_cards::SYNERGY_OFF };
                                    toast.info(
                                        said.to_string(),
                                        ToastOptions::default().duration(TOAST_QUICK),
                                    );
                                },
                                BoltIcon {}
                            }
                        }
                        DockFilter {
                            label: "Filter",
                            dot,
                            onclick: move |()| filter_open.set(true),
                        }
                        button {
                            class: "action-dock-btn",
                            "aria-label": "Refresh",
                            onclick: move |_| refresh.call(()),
                            DockIcon { path: REFRESH }
                        }
                        span { class: "action-dock-divider" }
                        DockEye { eye, disabled: !has_current }
                    }
                }
            }
        }
        if let Some(card) = current {
            SheetFrame { wide: SheetWide::Modal,
                if at == AddSource::Search {
                    CardDetailsSheet {
                        open: show_details,
                        card: card.clone(),
                        on_printings: move |()| printing_open.set(true),
                    }
                } else {
                    CardDetailsSheet { open: show_details, card: card.clone() }
                }
            }
            if at == AddSource::Search {
                PrintingSheet {
                    key: "{card.scryfall_data.id}",
                    card,
                    open: printing_open,
                    saved_message: "Printing swapped",
                    on_save: move |picked: Card| {
                        stack.write().replace_current(picked);
                        show_details.set(false);
                    },
                }
            }
        }
        // Keyed, so a source switch remounts it with a fresh draft; the key
        // needs a block of its own.
        Fragment {
            CardFilterPanel {
                key: "{panel_key}",
                open: filter_open,
                filter,
                config: panel_config,
                on_apply,
                }
        }
        OneTimeHintDialog { hint: OneTimeHint::AddDeckCards, open: hint_open }
    }
}

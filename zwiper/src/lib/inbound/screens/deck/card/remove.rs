use super::components::{
    card_info::{CardInfoDisplay, CardRulesDialog, CardSkeleton, RulesButton},
    filter_store::{FilterScope, FilterStore},
    printing_sheet::PrintingSheet,
};
use crate::{
    inbound::{
        components::{
            auth::{bouncer::Bouncer, ensure_session::EnsureFresh},
            chip::Chip,
            hint_dialog::{
                HintBullet, HintBullets, HintColored, HintDialog, HintLine, use_one_time_hint,
            },
            interactions::swipe::{SwipeStack, config::SwipeConfig, direction::Direction},
            screen_header::ScreenHeader,
            telemetry::usage_buffer::UsageBuffer,
        },
        screens::deck::card::{
            components::{
                action_history::{RemoveAction, StackAction},
                card_stack::use_card_stack,
            },
            filter::{card_filter_sheet::CardFilterSheet, deck_cards::DeckCards},
        },
    },
    outbound::client::{
        ZwipeClient,
        card::get_card::ClientGetCard,
        deck::get_deck::ClientGetDeck,
        deck_card::{
            create_deck_card::ClientCreateDeckCard, delete_deck_card::ClientDeleteDeckCard,
            update_deck_card::ClientUpdateDeckCard,
        },
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::{
            Card,
            search_card::{
                card_filter::{
                    builder::CardQueryBuilder, card_sort_key::CardSortKey,
                    price_currency::PriceCurrency,
                },
                cards::Cards,
            },
        },
        deck::{
            Board, DeckEntry,
            deck_metrics::{budget_tier, mainboard_total_price},
        },
        user::models::hints::HINT_REMOVE_DECK_CARDS,
    },
    http::contracts::deck_card::{HttpCreateDeckCard, HttpUpdateDeckCard},
};

/// Board filter for the remove screen.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum BoardFilter {
    /// Show only active deck cards (default).
    #[default]
    Deck,
    /// Show only maybeboard cards.
    Maybeboard,
    /// Show only sideboard cards.
    Sideboard,
    /// Show all cards regardless of board.
    All,
}

#[component]
pub fn Remove(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    // This screen's own filter: seeded from the per-(screen, deck) store,
    // provided as context so the filter modules bind to it, parked on leave.
    let filter_store: FilterStore = use_context();
    let filter_builder: Signal<CardQueryBuilder> = use_signal(|| {
        filter_store
            .restore(FilterScope::Remove, deck_id)
            .unwrap_or_default()
    });
    use_context_provider(|| filter_builder);
    use_drop(move || {
        let mut store = filter_store;
        store.park(FilterScope::Remove, deck_id, filter_builder.peek().clone());
    });

    // Filter overlay state
    let mut filters_overlay_open = use_signal(|| false);

    // Swipe vocabulary hint: auto-opens on this user's first visit, the
    // grayed "?" in the util bar reopens it on demand.
    let swipe_hint_open = use_one_time_hint(HINT_REMOVE_DECK_CARDS);
    let show_rules = use_signal(|| false);
    let mut printing_open = use_signal(|| false);

    // Incrementing this re-runs the filter effect
    let mut filter_reset_counter: Signal<u32> = use_signal(|| 0);
    use_context_provider(|| filter_reset_counter);

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let usage_buffer: Signal<UsageBuffer> = use_context();
    let toast = use_toast();

    // Source of truth — all entries in the deck
    let mut deck_entries: Signal<Vec<DeckEntry>> = use_signal(Vec::new);

    // Provide Card list context for filter sheet
    let mut deck_cards_for_filter: Signal<Vec<Card>> = use_signal(Vec::new);
    use_context_provider(|| DeckCards(deck_cards_for_filter));

    // What the swipe UI iterates over (may be a filtered subset of
    // `deck_entries`) — a cycling stack over the displayed cards.
    let mut stack = use_card_stack::<RemoveAction>();
    // Guards filter effect from running before the deck has loaded
    let mut deck_loaded: Signal<bool> = use_signal(|| false);
    // Board filter state
    let mut board_filter: Signal<BoardFilter> = use_signal(BoardFilter::default);

    // Effective land target (deck override, else format heuristic) for the
    // below-target warning when lands leave the mainboard.
    let mut land_target: Signal<Option<i32>> = use_signal(|| None);
    // Primary commander oracle id, for keying the removal suggestion signal.
    let mut commander_oracle_id: Signal<Option<Uuid>> = use_signal(|| None);
    // Deck price budget + currency for the 50/75/100% crossing toasts. A budget
    // crossing only fires upward, so removals are silent; moving a card back to
    // the mainboard can raise the total enough to cross.
    let mut price_budget: Signal<Option<f64>> = use_signal(|| None);
    let mut price_budget_currency: Signal<PriceCurrency> = use_signal(|| PriceCurrency::Usd);

    // Swipe config — the stack owns its own SwipeState internally.
    // Thresholds tuned for responsive rapid swiping: a short drag (60px) OR
    // a quick flick (1.5 px/ms over the 10px minimum) commits.
    let swipe_config = SwipeConfig::new(
        vec![
            Direction::Left,
            Direction::Right,
            Direction::Up,
            Direction::Down,
        ],
        60.0,
        1.5,
    );

    let current_card = move || stack.current_wrapping();

    // Mainboard land count from the source of truth (quantity-aware, MDFC land
    // faces included via is_land). Recomputed each call so it never drifts.
    let main_land_count = move || -> i32 {
        deck_entries
            .peek()
            .iter()
            .filter(|e| e.deck_card.board.is_active() && e.card.scryfall_data.is_land())
            .map(|e| *e.deck_card.quantity)
            .sum()
    };

    // Warn once when a land action drops the mainboard below its target.
    let warn_if_below_target = move |before: i32, after: i32| {
        if let Some(target) = land_target()
            && before >= target
            && after < target
        {
            toast.warning(
                format!("Below land target ({target})"),
                ToastOptions::default().duration(Duration::from_millis(2500)),
            );
        }
    };

    // Mainboard total in the budget currency from the source of truth.
    let total_price =
        move || -> f64 { mainboard_total_price(&deck_entries.peek(), price_budget_currency()) };

    // Toast when a change raises the deck into a higher budget band (50/75/100%),
    // reporting the exact percentage. Compares the band before/after, so it fires
    // once per upward crossing and re-fires if the total dips and crosses again.
    let warn_budget_crossing = move |before: f64, after: f64| {
        let Some(budget) = price_budget() else {
            return;
        };
        if budget <= 0.0 {
            return;
        }
        if budget_tier(budget, after) > budget_tier(budget, before) {
            let pct = after / budget * 100.0;
            let amount = price_budget_currency().format_amount(budget);
            let msg = format!("Deck at {pct:.2}% of your {amount} budget");
            let opts = ToastOptions::default().duration(Duration::from_millis(2500));
            if after >= budget {
                toast.warning(msg, opts);
            } else {
                toast.info(msg, opts);
            }
        }
    };

    // Effect 1 — mount load (reads `session` reactively)
    use_effect(move || {
        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(_) => {
                    return;
                }
            };

            match client().get_deck(deck_id, &session).await {
                Ok(deck) => {
                    let all_cards: Vec<Card> =
                        deck.entries.iter().map(|e| e.card.clone()).collect();
                    deck_cards_for_filter.set(all_cards);
                    // Explicit target only — no land toasts unless the user set one.
                    land_target.set(deck.deck_profile.land_target);
                    let budget_currency = deck
                        .deck_profile
                        .price_target_currency
                        .unwrap_or(PriceCurrency::Usd);
                    price_budget.set(deck.deck_profile.price_target);
                    price_budget_currency.set(budget_currency);
                    // Resolve the primary commander's oracle id for the removal signal.
                    if let Some(commander_id) = deck.deck_profile.commander_id
                        && let Ok(card) = client().get_card(commander_id).await
                    {
                        commander_oracle_id.set(card.scryfall_data.oracle_id);
                    }
                    deck_entries.set(deck.entries);
                    deck_loaded.set(true);
                    let current = *filter_reset_counter.peek();
                    filter_reset_counter.set(current + 1);
                }
                Err(e) => {
                    tracing::warn!("deck load failed: {e}");
                    toast.error(e.to_user_message(), ToastOptions::default());
                }
            }
        });
    });

    // Effect 2 — filter (reads `filter_reset_counter`, `board_filter` reactively; peeks entries)
    use_effect(move || {
        let _ = filter_reset_counter();
        let _ = board_filter();

        if !*deck_loaded.peek() {
            return;
        }

        let entries = deck_entries.peek().clone();
        let bf = *board_filter.peek();
        let builder = filter_builder.peek().clone();

        // Step 1: filter by board
        let board_filtered_cards: Vec<Card> = entries
            .iter()
            .filter(|e| match bf {
                BoardFilter::Deck => e.deck_card.board.is_active(),
                BoardFilter::Maybeboard => e.deck_card.board.is_maybeboard(),
                BoardFilter::Sideboard => e.deck_card.board.is_sideboard(),
                BoardFilter::All => true,
            })
            .map(|e| e.card.clone())
            .collect();

        // Step 2: apply card attribute criteria, then the builder's sort. The
        // in-memory path takes bare criteria — no pagination to zero out.
        let filtered = if builder.is_empty() {
            board_filtered_cards
        } else {
            match builder.build_criteria() {
                Ok(criteria) => Cards::from(board_filtered_cards).matching(&criteria).into(),
                Err(_) => board_filtered_cards,
            }
        };
        let filtered: Vec<Card> = Cards::from(filtered)
            .sorted(builder.sort(), builder.ascending())
            .into();

        stack.replace(filtered);
    });

    let delete_card_from_deck = move || {
        let Some(card) = current_card() else {
            return;
        };

        let scryfall_data_id = card.scryfall_data.id;

        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(e) => {
                    toast.error(e.to_user_message(), ToastOptions::default());
                    return;
                }
            };

            if let Err(e) = client()
                .delete_deck_card(deck_id, scryfall_data_id, &session)
                .await
            {
                tracing::warn!("delete deck card failed: {e}");
                toast.error(e.to_user_message(), ToastOptions::default());
            }
        });
    };

    let move_card_to_board = move |to: Board| {
        let Some(card) = current_card() else {
            return;
        };

        let scryfall_data_id = card.scryfall_data.id;
        let request = HttpUpdateDeckCard::new(None, Some(to.display_name().to_string()));

        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(e) => {
                    toast.error(e.to_user_message(), ToastOptions::default());
                    return;
                }
            };

            if let Err(e) = client()
                .update_deck_card(deck_id, scryfall_data_id, &request, &session)
                .await
            {
                tracing::warn!("move card to board failed: {e}");
                toast.error(e.to_user_message(), ToastOptions::default());
            }
        });
    };

    // Called at animation end to physically remove the card from both vecs.
    let mut remove_current_card = move || {
        let card_id = stack.current().map(|c| c.card_profile.scryfall_data_id);
        if let Some(id) = card_id {
            deck_entries
                .write()
                .retain(|e| e.card.card_profile.scryfall_data_id != id);
            stack.remove_current();
        }
        // The cursor is unchanged — the next card slides into position.
    };

    // Board-move counterpart of remove_current_card: the entry survives in
    // the source of truth with its new board (so the board chips stay
    // truthful without a refetch); only the displayed stack drops the card.
    let mut move_current_card_locally = move |to: Board| {
        let card_id = stack.current().map(|c| c.card_profile.scryfall_data_id);
        if let Some(id) = card_id {
            if let Some(entry) = deck_entries
                .write()
                .iter_mut()
                .find(|e| e.card.card_profile.scryfall_data_id == id)
            {
                entry.deck_card.board = to;
            }
            stack.remove_current();
        }
        // The cursor is unchanged — the next card slides into position.
    };

    let mut undo_last_action = move || {
        let Some(action) = stack.pop_action() else {
            toast.info("Nothing to undo".to_string(), ToastOptions::default());
            return;
        };

        // Play the enter animation from the direction the card originally
        // exited.
        stack.prime_entering(action.exited());

        match action {
            RemoveAction::Keep => {
                if !stack.retreat_wrapping() {
                    stack.cancel_entering();
                    return;
                }
                toast.info(
                    "Undid skip".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            }
            RemoveAction::Remove { card } => {
                // Re-insert into displayed cards so the card reappears
                let card = *card;
                stack.insert_current(card.clone());

                // Restore on the backend
                let request = HttpCreateDeckCard::new(&card.scryfall_data, 1, None);
                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            toast.error(e.to_user_message(), ToastOptions::default());
                            stack.cancel_entering();
                            return;
                        }
                    };

                    match client().create_deck_card(deck_id, &request, &session).await {
                        Ok(deck_card) => {
                            // Re-add entry to source of truth
                            deck_entries.write().push(DeckEntry { card, deck_card });
                            toast.success(
                                "Undid remove".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo remove (create deck card) failed: {e}");
                            toast.error(format!("Failed to undo: {}", e), ToastOptions::default());
                        }
                    }
                });
            }
            RemoveAction::MoveBoard { card, from, .. } => {
                // Re-insert into displayed cards so the card reappears
                let card = *card;
                stack.insert_current(card.clone());

                // Move back to the board it came from, server then local
                let scryfall_data_id = card.scryfall_data.id;
                let request = HttpUpdateDeckCard::new(None, Some(from.display_name().to_string()));

                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            toast.error(e.to_user_message(), ToastOptions::default());
                            stack.cancel_entering();
                            return;
                        }
                    };

                    match client()
                        .update_deck_card(deck_id, scryfall_data_id, &request, &session)
                        .await
                    {
                        Ok(_) => {
                            if let Some(entry) = deck_entries
                                .write()
                                .iter_mut()
                                .find(|e| e.card.scryfall_data.id == scryfall_data_id)
                            {
                                entry.deck_card.board = from;
                            }
                            toast.success(
                                "Undid move".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo board move (update deck card) failed: {e}");
                            toast.error(format!("Failed to undo: {}", e), ToastOptions::default());
                        }
                    }
                });
            }
        }
    };

    rsx! {
        Bouncer {
            div { class: "screen",
                ScreenHeader { title: "Remove Deck Cards", hint: swipe_hint_open }

                div { class: "screen-content card-swipe content-enter",

                // Board filter chips
                div { style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    div { class: "chip-row",
                        span { class: "chip-row-label", "Boards:" }
                        for (label, variant) in [("Main", BoardFilter::Deck), ("Maybe", BoardFilter::Maybeboard), ("Side", BoardFilter::Sideboard), ("All", BoardFilter::All)] {
                            Chip {
                                selected: board_filter() == variant,
                                onclick: move |_| {
                                    board_filter.set(variant);
                                    let current = *filter_reset_counter.peek();
                                    filter_reset_counter.set(current + 1);
                                },
                                "{label}"
                            }
                        }
                    }
                }

                div { class: "form-container",
                    if !stack.is_empty() {
                        SwipeStack {
                            cards: stack.window(),
                            config: swipe_config,
                            entering: stack.entering(),
                            on_swipe_left: move |_card: Card| {
                                usage_buffer().record_swipe(Direction::Left);
                                stack.record(RemoveAction::Keep);
                                toast.info(
                                    "Skipped".to_string(),
                                    ToastOptions::default().duration(Duration::from_millis(1500)),
                                );
                                // Skip: advance circularly within the stack
                                stack.advance_wrapping();
                            },
                            on_swipe_right: move |card: Card| {
                                usage_buffer().record_swipe(Direction::Right);
                                // Removal signal (delayed negative) keyed by commander + card.
                                usage_buffer().record_removal(commander_oracle_id(), card.scryfall_data.oracle_id);
                                stack.record(RemoveAction::Remove { card: Box::new(card) });
                                delete_card_from_deck();
                                toast.success(
                                    "Removed from deck".to_string(),
                                    ToastOptions::default().duration(Duration::from_millis(1500)),
                                );
                                let before = main_land_count();
                                let before_price = total_price();
                                remove_current_card();
                                warn_if_below_target(before, main_land_count());
                                warn_budget_crossing(before_price, total_price());
                            },
                            on_swipe_up: move |card: Card| {
                                usage_buffer().record_swipe(Direction::Up);
                                // Stage/promote toggle: maybeboard cards graduate
                                // to main, everything else stages to maybeboard.
                                let from = deck_entries
                                    .peek()
                                    .iter()
                                    .find(|e| e.card.scryfall_data.id == card.scryfall_data.id)
                                    .map(|e| e.deck_card.board)
                                    .unwrap_or_default();
                                let to = if from.is_maybeboard() { Board::Deck } else { Board::Maybeboard };
                                stack.record(RemoveAction::MoveBoard { card: Box::new(card), from, to });
                                move_card_to_board(to);
                                let message = if to.is_maybeboard() { "Moved to maybeboard" } else { "Moved to main" };
                                toast.info(message.to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                let before = main_land_count();
                                let before_price = total_price();
                                move_current_card_locally(to);
                                warn_if_below_target(before, main_land_count());
                                warn_budget_crossing(before_price, total_price());
                            },
                            on_swipe_down: move |_card: Card| {
                                usage_buffer().record_swipe(Direction::Down);
                                undo_last_action();
                            },
                        }

                        if let Some(card) = current_card() {
                            CardInfoDisplay { card: card.clone() }
                            CardRulesDialog {
                                open: show_rules,
                                card,
                                on_printings: move |_| printing_open.set(true),
                            }
                        }
                    } else {
                        CardSkeleton {}
                    }

                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| filters_overlay_open.set(true),
                    "Filter"
                    if !filter_builder.read().is_empty() || filter_builder.read().sort().is_some() {
                        span { class: "filter-dot" }
                    }
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        stack.rewind();
                        if filter_builder.peek().sort() == Some(CardSortKey::Random) {
                            let current = *filter_reset_counter.peek();
                            filter_reset_counter.set(current + 1);
                        }
                        toast.info(
                            "Stack refreshed".to_string(),
                            ToastOptions::default().duration(Duration::from_millis(1500)),
                        );
                    },
                    "Refresh"
                }
                if current_card().is_some() {
                    RulesButton { open: show_rules }
                }
            }

            // Root-mounted so the bottom sheet covers the footer. View-only:
            // browsing printings never changes the deck (remove keys on the
            // exact scryfall_data_id).
            if let Some(card) = current_card() {
                PrintingSheet {
                    card,
                    open: printing_open,
                    on_save: move |_: Card| {},
                    read_only: true,
                }
            }

            HintDialog {
                open: swipe_hint_open,
                title: "Swipe to trim",
                HintBullets {
                    HintBullet {
                        "Swipe "
                        HintColored { color: "--color-success", "right" }
                        " to remove a card from your deck."
                    }
                    HintBullet {
                        "Swipe "
                        HintColored { color: "--color-error", "left" }
                        " to keep it."
                    }
                    HintBullet {
                        "Swipe "
                        HintColored { color: "--color-warning", "up" }
                        " to move a card to your maybeboard, or a maybeboard card into main."
                    }
                    HintBullet {
                        "Swipe "
                        HintColored { color: "--accent-tertiary", "down" }
                        " to undo your last swipe."
                    }
                }
                HintLine { "The board chips at the top choose which board you are trimming." }
            }

            CardFilterSheet {
                open: filters_overlay_open,
                show_format_filter: true,
                show_active_indicators: true,
            }
            }
        }
    }
}

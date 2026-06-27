use super::components::card_info::{CardInfoDisplay, CardSkeleton};
use super::components::keyword_hint::{KeywordHintDialog, card_has_keywords};
use crate::{
    inbound::{
        components::{
            auth::{bouncer::Bouncer, ensure_session::EnsureFresh},
            hint_dialog::{HintBullet, HintBullets, HintColored, HintDialog, HintLine, use_one_time_hint},
            interactions::swipe::{SwipeStack, config::SwipeConfig, direction::Direction},
            telemetry::usage_buffer::UsageBuffer,
        },
        screens::deck::card::{
            components::action_history::SwipeAction,
            filter::{card_filter_sheet::CardFilterSheet, deck_cards::DeckCards},
        },
    },
    outbound::client::{
        ZwipeClient,
        deck::get_deck::ClientGetDeck,
        deck_card::{
            create_deck_card::ClientCreateDeckCard, delete_deck_card::ClientDeleteDeckCard,
            update_deck_card::ClientUpdateDeckCard,
        },
    },
};
use dioxus::prelude::*;
use crate::inbound::components::screen_header::ScreenHeader;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpUpdateDeckCard};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::hints::HINT_REMOVE_DECK_CARDS;
use zwipe_core::domain::card::{
    Card,
    search_card::{
        card_filter::{builder::CardFilterBuilder, order_by_option::OrderByOption},
        filter_cards::{FilterCards, SortCards},
    },
};
use zwipe_core::domain::deck::{Board, DeckEntry};

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

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    // When Some, the SwipeStack plays a keyframe entering from this direction
    // on the next top card, and clears it on animationend. Set by undo.
    let mut entering_direction: Signal<Option<Direction>> = use_signal(|| None);

    // Local undo stack
    let mut action_history: Signal<Vec<SwipeAction>> = use_signal(Vec::new);

    // Filter overlay state
    let mut filters_overlay_open = use_signal(|| false);

    // Swipe vocabulary hint: auto-opens on this user's first visit, the
    // grayed "?" in the util bar reopens it on demand.
    let swipe_hint_open = use_one_time_hint(HINT_REMOVE_DECK_CARDS);
    let mut keyword_hint_open = use_signal(|| false);

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

    // What the swipe UI iterates over (may be a filtered subset)
    let mut displayed_cards: Signal<Vec<Card>> = use_signal(Vec::new);
    // Guards filter effect from running before the deck has loaded
    let mut deck_loaded: Signal<bool> = use_signal(|| false);
    // Board filter state
    let mut board_filter: Signal<BoardFilter> = use_signal(BoardFilter::default);

    let mut current_index = use_signal(|| 0_usize);

    // Swipe config — the stack owns its own SwipeState internally.
    // Thresholds tuned for responsive rapid swiping: a short drag (60px) OR
    // a quick flick (1.5 px/ms over the 10px minimum) commits.
    let swipe_config = SwipeConfig::new(
        vec![Direction::Left, Direction::Right, Direction::Up, Direction::Down],
        60.0,
        1.5,
    );

    let current_card = move || {
        let cards = displayed_cards();
        if cards.is_empty() {
            return None;
        }
        let idx = current_index() % cards.len();
        cards.get(idx).cloned()
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

        // Step 2: apply card attribute filter
        let mut filtered = if builder.is_empty() {
            board_filtered_cards
        } else {
            let mut b = builder.clone();
            b.set_limit(10_000);
            b.set_offset(0);
            match b.build() {
                Ok(filter) => board_filtered_cards.filter_by(&filter),
                Err(_) => board_filtered_cards,
            }
        };

        if builder.is_empty() {
            filtered.sort_by_filter(&builder);
        }

        displayed_cards.set(filtered);
        current_index.set(0);
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
        let idx = current_index();
        let card_id = displayed_cards()
            .get(idx)
            .map(|c| c.card_profile.scryfall_data_id);
        if let Some(id) = card_id {
            deck_entries
                .write()
                .retain(|e| e.card.card_profile.scryfall_data_id != id);
            if idx < displayed_cards.read().len() {
                displayed_cards.write().remove(idx);
            }
        }
        // current_index is unchanged — the next card slides into position
    };

    // Board-move counterpart of remove_current_card: the entry survives in
    // the source of truth with its new board (so the board chips stay
    // truthful without a refetch); only the displayed stack drops the card.
    let mut move_current_card_locally = move |to: Board| {
        let idx = current_index();
        let card_id = displayed_cards()
            .get(idx)
            .map(|c| c.card_profile.scryfall_data_id);
        if let Some(id) = card_id {
            if let Some(entry) = deck_entries
                .write()
                .iter_mut()
                .find(|e| e.card.card_profile.scryfall_data_id == id)
            {
                entry.deck_card.board = to;
            }
            if idx < displayed_cards.read().len() {
                displayed_cards.write().remove(idx);
            }
        }
        // current_index is unchanged — the next card slides into position
    };

    let mut undo_last_action = move || {
        let Some(action) = action_history.write().pop() else {
            toast.info("Nothing to undo".to_string(), ToastOptions::default());
            return;
        };

        // Ask the stack to play the enter animation from the direction the
        // card originally exited.
        entering_direction.set(Some(action.exited().clone()));

        match action {
            SwipeAction::Skip { .. } => {
                let len = displayed_cards().len();
                if len == 0 {
                    entering_direction.set(None);
                    return;
                }
                let idx = current_index();
                let prev = if idx == 0 { len - 1 } else { idx - 1 };
                current_index.set(prev);
                toast.info(
                    "Undid skip".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            }
            SwipeAction::Do { card, .. } => {
                // Re-insert into displayed cards so the card reappears
                let card = *card;
                let idx = current_index();
                displayed_cards.write().insert(idx, card.clone());

                // Restore on the backend
                let request = HttpCreateDeckCard::new(&card.scryfall_data, 1, None);
                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            toast.error(e.to_user_message(), ToastOptions::default());
                            entering_direction.set(None);
                            return;
                        }
                    };

                    match client().create_deck_card(deck_id, &request, &session).await {
                        Ok(deck_card) => {
                            // Re-add entry to source of truth
                            deck_entries.write().push(DeckEntry {
                                card,
                                deck_card,
                            });
                            toast.success(
                                "Undid remove".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo remove (create deck card) failed: {e}");
                            toast.error(
                                format!("Failed to undo: {}", e),
                                ToastOptions::default(),
                            );
                        }
                    }
                });
            }
            SwipeAction::Maybeboard { card, .. } => {
                // Re-insert into displayed cards so the card reappears
                let card = *card;
                let idx = current_index();
                displayed_cards.write().insert(idx, card.clone());

                // Move back from maybeboard to active on the backend
                let scryfall_data_id = card.scryfall_data.id;
                let request = HttpUpdateDeckCard::new(None, Some("deck".to_string()));

                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            toast.error(e.to_user_message(), ToastOptions::default());
                            entering_direction.set(None);
                            return;
                        }
                    };

                    match client()
                        .update_deck_card(deck_id, scryfall_data_id, &request, &session)
                        .await
                    {
                        Ok(_) => {
                            // Flip the flag back in the source of truth
                            if let Some(entry) = deck_entries
                                .write()
                                .iter_mut()
                                .find(|e| e.card.scryfall_data.id == scryfall_data_id)
                            {
                                entry.deck_card.board = Board::Deck;
                            }
                            toast.success(
                                "Undid maybeboard".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo maybeboard (update deck card) failed: {e}");
                            toast.error(
                                format!("Failed to undo: {}", e),
                                ToastOptions::default(),
                            );
                        }
                    }
                });
            }
            SwipeAction::MoveBoard { card, from, .. } => {
                // Re-insert into displayed cards so the card reappears
                let card = *card;
                let idx = current_index();
                displayed_cards.write().insert(idx, card.clone());

                // Move back to the board it came from, server then local
                let scryfall_data_id = card.scryfall_data.id;
                let request = HttpUpdateDeckCard::new(None, Some(from.display_name().to_string()));

                spawn(async move {
                    let session = match session.ensure_fresh(client).await {
                        Ok(session) => session,
                        Err(e) => {
                            toast.error(e.to_user_message(), ToastOptions::default());
                            entering_direction.set(None);
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
                            toast.error(
                                format!("Failed to undo: {}", e),
                                ToastOptions::default(),
                            );
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
                            button {
                                class: if board_filter() == variant { "chip selected" } else { "chip" },
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
                    if !displayed_cards().is_empty() {
                        SwipeStack {
                            cards: {
                                let all = displayed_cards();
                                all.into_iter().skip(current_index()).take(crate::inbound::components::interactions::swipe::STACK_DEPTH).collect::<Vec<_>>()
                            },
                            config: swipe_config,
                            entering: entering_direction,
                            on_swipe_left: move |card: Card| {
                                usage_buffer().record_swipe(Direction::Left);
                                action_history.write().push(SwipeAction::Skip { card: Box::new(card), exited: Direction::Left });
                                toast.info(
                                    "Skipped".to_string(),
                                    ToastOptions::default().duration(Duration::from_millis(1500)),
                                );
                                // Skip: advance circularly within displayed_cards
                                let len = displayed_cards().len();
                                if len > 0 {
                                    current_index.set((current_index() + 1) % len);
                                }
                            },
                            on_swipe_right: move |card: Card| {
                                usage_buffer().record_swipe(Direction::Right);
                                action_history.write().push(SwipeAction::Do { card: Box::new(card), exited: Direction::Right });
                                delete_card_from_deck();
                                toast.success(
                                    "Removed from deck".to_string(),
                                    ToastOptions::default().duration(Duration::from_millis(1500)),
                                );
                                remove_current_card();
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
                                action_history.write().push(SwipeAction::MoveBoard { card: Box::new(card), exited: Direction::Up, from, to });
                                move_card_to_board(to);
                                let message = if to.is_maybeboard() { "Moved to maybeboard" } else { "Moved to main" };
                                toast.info(message.to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                move_current_card_locally(to);
                            },
                            on_swipe_down: move |_card: Card| {
                                usage_buffer().record_swipe(Direction::Down);
                                undo_last_action();
                            },
                        }

                        if let Some(card) = current_card() {
                            CardInfoDisplay { card }
                        }
                    } else {
                        CardSkeleton {}
                    }

                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| filters_overlay_open.set(true),
                    "Filter"
                    if !filter_builder.read().is_empty() {
                        span { class: "filter-dot" }
                    }
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        current_index.set(0);
                        action_history.write().clear();
                        if filter_builder.peek().order_by() == Some(OrderByOption::Random) {
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
                if !filter_builder.read().is_empty() {
                    button {
                        class: "util-btn util-btn-clear",
                        onclick: move |_| {
                            filter_builder.write().clear();
                            board_filter.set(BoardFilter::Deck);
                            let current = *filter_reset_counter.peek();
                            filter_reset_counter.set(current + 1);
                            toast.info(
                                "Filter cleared".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        },
                        "Clear"
                    }
                }
                if current_card().as_ref().is_some_and(card_has_keywords) {
                    button {
                        class: "util-btn",
                        onclick: move |_| keyword_hint_open.set(true),
                        "Keywords"
                    }
                }
            }

            if let Some(card) = current_card() {
                KeywordHintDialog { open: keyword_hint_open, card }
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

use super::components::card_info::{CardInfoDisplay, CardSkeleton};
use crate::{
    inbound::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{
                Swipeable, config::SwipeConfig, direction::Direction, state::SwipeState,
            },
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
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe_core::http::contracts::deck_card::{HttpCreateDeckCard, HttpUpdateDeckCard};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::{
    Card,
    scryfall_data::image_uris::ImageUris,
    search_card::{
        card_filter::{builder::CardFilterBuilder, order_by_option::OrderByOption},
        filter_cards::{FilterCards, SortCards},
    },
};
use zwipe_core::domain::deck::DeckEntry;

/// Tri-state filter for maybeboard status on the remove screen.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum MaybeboardFilter {
    /// Show only active deck cards (default).
    #[default]
    Deck,
    /// Show only maybeboard cards.
    Maybeboard,
    /// Show all cards regardless of maybeboard status.
    All,
}

#[component]
pub fn Remove(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let mut is_animating = use_signal(|| false);
    let mut animation_direction = use_signal(|| Direction::Left);

    // Local undo stack
    let mut action_history: Signal<Vec<SwipeAction>> = use_signal(Vec::new);

    // Filter overlay state
    let mut filters_overlay_open = use_signal(|| false);

    // Incrementing this re-runs the filter effect
    let mut filter_reset_counter: Signal<u32> = use_signal(|| 0);
    use_context_provider(|| filter_reset_counter);

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
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
    // Maybeboard filter state
    let mut maybeboard_filter: Signal<MaybeboardFilter> = use_signal(MaybeboardFilter::default);

    let mut current_index = use_signal(|| 0_usize);

    let swipe_state = use_signal(SwipeState::new);
    let swipe_config = SwipeConfig::new(
        vec![Direction::Left, Direction::Right, Direction::Up, Direction::Down],
        150.0,
        5.0,
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
        session.upkeep(client);
        let Some(session) = session() else {
            return;
        };

        spawn(async move {
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
                    toast.error(e.to_string(), ToastOptions::default());
                }
            }
        });
    });

    // Effect 2 — filter (reads `filter_reset_counter`, `maybeboard_filter` reactively; peeks entries)
    use_effect(move || {
        let _ = filter_reset_counter();
        let _ = maybeboard_filter();

        if !*deck_loaded.peek() {
            return;
        }

        let entries = deck_entries.peek().clone();
        let mb_filter = *maybeboard_filter.peek();
        let builder = filter_builder.peek().clone();

        // Step 1: filter by maybeboard status
        let mb_filtered_cards: Vec<Card> = entries
            .iter()
            .filter(|e| match mb_filter {
                MaybeboardFilter::Deck => !e.deck_card.maybeboard,
                MaybeboardFilter::Maybeboard => e.deck_card.maybeboard,
                MaybeboardFilter::All => true,
            })
            .map(|e| e.card.clone())
            .collect();

        // Step 2: apply card attribute filter
        let mut filtered = if builder.is_empty() {
            mb_filtered_cards
        } else {
            let mut b = builder.clone();
            b.set_limit(10_000);
            b.set_offset(0);
            match b.build() {
                Ok(filter) => mb_filtered_cards.filter_by(&filter),
                Err(_) => mb_filtered_cards,
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

        session.upkeep(client);
        let Some(session) = session() else {
            toast.error("session expired".to_string(), ToastOptions::default());
            return;
        };

        let scryfall_data_id = card.scryfall_data.id;

        spawn(async move {
            if let Err(e) = client()
                .delete_deck_card(deck_id, scryfall_data_id, &session)
                .await
            {
                tracing::warn!("delete deck card failed: {e}");
                toast.error(e.to_string(), ToastOptions::default());
            }
        });
    };

    let move_card_to_maybeboard = move || {
        let Some(card) = current_card() else {
            return;
        };

        session.upkeep(client);
        let Some(session) = session() else {
            toast.error("session expired".to_string(), ToastOptions::default());
            return;
        };

        let scryfall_data_id = card.scryfall_data.id;
        let request = HttpUpdateDeckCard::new(None, Some(true));

        spawn(async move {
            if let Err(e) = client()
                .update_deck_card(deck_id, scryfall_data_id, &request, &session)
                .await
            {
                tracing::warn!("move card to maybeboard failed: {e}");
                toast.error(e.to_string(), ToastOptions::default());
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

    let mut undo_last_action = move || {
        let Some(action) = action_history.write().pop() else {
            toast.info("nothing to undo".to_string(), ToastOptions::default());
            return;
        };

        match action {
            SwipeAction::Skip(_) => {
                let len = displayed_cards().len();
                if len == 0 {
                    return;
                }
                let idx = current_index();
                let prev = if idx == 0 { len - 1 } else { idx - 1 };
                current_index.set(prev);
                toast.info(
                    "undid skip".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            }
            SwipeAction::Do(card) => {
                // Re-insert into displayed cards so the card reappears
                let card = *card;
                let idx = current_index();
                displayed_cards.write().insert(idx, card.clone());

                // Restore on the backend
                session.upkeep(client);
                let Some(session) = session() else {
                    toast.error("session expired".to_string(), ToastOptions::default());
                    return;
                };

                let oracle_id_str = card.scryfall_data.oracle_id.map(|id| id.to_string()).unwrap_or_default();
                let request =
                    HttpCreateDeckCard::new(&card.scryfall_data.id.to_string(), &oracle_id_str, 1, None);
                spawn(async move {
                    match client().create_deck_card(deck_id, &request, &session).await {
                        Ok(deck_card) => {
                            // Re-add entry to source of truth
                            deck_entries.write().push(DeckEntry {
                                card,
                                deck_card,
                            });
                            toast.success(
                                "undid remove".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo remove (create deck card) failed: {e}");
                            toast.error(
                                format!("failed to undo: {}", e),
                                ToastOptions::default(),
                            );
                        }
                    }
                });
            }
            SwipeAction::Maybeboard(card) => {
                // Re-insert into displayed cards so the card reappears
                let card = *card;
                let idx = current_index();
                displayed_cards.write().insert(idx, card.clone());

                // Move back from maybeboard to active on the backend
                session.upkeep(client);
                let Some(session) = session() else {
                    toast.error("session expired".to_string(), ToastOptions::default());
                    return;
                };

                let scryfall_data_id = card.scryfall_data.id;
                let request = HttpUpdateDeckCard::new(None, Some(false));

                spawn(async move {
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
                                entry.deck_card.maybeboard = false;
                            }
                            toast.success(
                                "undid maybeboard".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo maybeboard (update deck card) failed: {e}");
                            toast.error(
                                format!("failed to undo: {}", e),
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
                div { class: "page-header",
                    h2 { "remove deck card" }
                }

                div { class: "screen-content card-swipe content-enter",

                // Maybeboard filter chips
                div { style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    div { class: "chip-row",
                        span { class: "chip-row-label", "show:" }
                        button {
                            class: if maybeboard_filter() == MaybeboardFilter::Deck { "chip selected" } else { "chip" },
                            onclick: move |_| {
                                maybeboard_filter.set(MaybeboardFilter::Deck);
                                let current = *filter_reset_counter.peek();
                                filter_reset_counter.set(current + 1);
                            },
                            "deck"
                        }
                        button {
                            class: if maybeboard_filter() == MaybeboardFilter::Maybeboard { "chip selected" } else { "chip" },
                            onclick: move |_| {
                                maybeboard_filter.set(MaybeboardFilter::Maybeboard);
                                let current = *filter_reset_counter.peek();
                                filter_reset_counter.set(current + 1);
                            },
                            "maybeboard"
                        }
                        button {
                            class: if maybeboard_filter() == MaybeboardFilter::All { "chip selected" } else { "chip" },
                            onclick: move |_| {
                                maybeboard_filter.set(MaybeboardFilter::All);
                                let current = *filter_reset_counter.peek();
                                filter_reset_counter.set(current + 1);
                            },
                            "all"
                        }
                    }
                }

                div { class: "form-container",
                    if let Some(card) = current_card() {
                        if let Some(ImageUris { large: Some(image_url), .. }) = &card.scryfall_data.image_uris {
                            Swipeable {
                                state: swipe_state,
                                config: swipe_config,
                                on_swipe_left: move |_| {
                                    let Some(card) = current_card() else { return; };
                                    action_history.write().push(SwipeAction::Skip(Box::new(card)));
                                    toast.info(
                                        "skipped".to_string(),
                                        ToastOptions::default().duration(Duration::from_millis(1500)),
                                    );
                                    is_animating.set(true);
                                    animation_direction.set(Direction::Left);
                                },
                                on_swipe_right: move |_| {
                                    let Some(card) = current_card() else { return; };
                                    action_history.write().push(SwipeAction::Do(Box::new(card)));
                                    delete_card_from_deck();
                                    toast.success(
                                        "removed from deck".to_string(),
                                        ToastOptions::default().duration(Duration::from_millis(1500)),
                                    );
                                    is_animating.set(true);
                                    animation_direction.set(Direction::Right);
                                },
                                on_swipe_up: move |_| {
                                    let Some(card) = current_card() else { return; };
                                    action_history.write().push(SwipeAction::Maybeboard(Box::new(card)));
                                    move_card_to_maybeboard();
                                    toast.info("moved to maybeboard".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                    is_animating.set(true);
                                    animation_direction.set(Direction::Up);
                                },
                                on_swipe_down: move |_| {
                                    undo_last_action();
                                },

                                img {
                                    src: "{image_url}",
                                    alt: "{card.scryfall_data.name}",
                                    class: "card-image",
                                    class: if is_animating() { "card-exit-animation" } else { "" },
                                    style: if is_animating() {
                                        format!(
                                            "--card-exit-direction: card-exit-{}",
                                            animation_direction().to_string().to_lowercase(),
                                        )
                                    } else {
                                        String::new()
                                    },
                                    onanimationend: move |_| {
                                        is_animating.set(false);
                                        if matches!(animation_direction(), Direction::Right | Direction::Up) {
                                            remove_current_card();
                                        } else {
                                            let len = displayed_cards().len();
                                            if len > 0 {
                                                current_index.set((current_index() + 1) % len);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        CardInfoDisplay { card }
                    } else {
                        CardSkeleton {}
                    }

                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| navigator.go_back(),
                    "back"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| filters_overlay_open.set(true),
                    "filter"
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
                            "stack refreshed".to_string(),
                            ToastOptions::default().duration(Duration::from_millis(1500)),
                        );
                    },
                    "refresh"
                }
                if !filter_builder.read().is_empty() {
                    button {
                        class: "util-btn util-btn-clear",
                        onclick: move |_| {
                            filter_builder.write().clear();
                            maybeboard_filter.set(MaybeboardFilter::Deck);
                            let current = *filter_reset_counter.peek();
                            filter_reset_counter.set(current + 1);
                            toast.info(
                                "filter cleared".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        },
                        "clear"
                    }
                }
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

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
            components::action_history::{CARDS_WARNING_THRESHOLD, MAX_CARDS_IN_STACK, SwipeAction},
            filter::card_filter_sheet::CardFilterSheet,
        },
    },
    outbound::client::{
        ZwipeClient,
        card::search_cards::ClientSearchCards,
        deck::get_deck::ClientGetDeck,
        deck_card::{
            create_deck_card::ClientCreateDeckCard, delete_deck_card::ClientDeleteDeckCard,
        },
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::collections::HashSet;
use std::time::Duration;
use uuid::Uuid;
use zwipe::domain::card::models::{
    Card, scryfall_data::image_uris::ImageUris,
    search_card::card_filter::builder::CardFilterBuilder,
};
use zwipe_core::domain::deck::format::Format;
use zwipe::inbound::http::handlers::deck_card::create_deck_card::HttpCreateDeckCard;
use zwipe_core::domain::auth::models::session::Session;

#[component]
pub fn Add(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    let mut cards: Signal<Vec<Card>> = use_context();
    let mut last_search_filter: Signal<Option<CardFilterBuilder>> = use_context();
    let is_first_run = use_hook(|| std::cell::Cell::new(true));

    let mut is_animating = use_signal(|| false);
    let mut animation_direction = use_signal(|| Direction::Left);

    let mut deck_cards_ids = use_signal(HashSet::<Uuid>::new);
    let mut deck_format: Signal<Option<Format>> = use_signal(|| None);

    // Undo action history
    let mut action_history: Signal<Vec<SwipeAction>> = use_signal(Vec::new);

    // Filters overlay at bottom of screen state
    let mut filters_overlay_open = use_signal(|| false);

    // Reset counter for collapsing accordions and clearing search queries
    let filter_reset_counter: Signal<u32> = use_signal(|| 0);
    use_context_provider(|| filter_reset_counter);

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    // Card iteration state
    let mut current_index = use_signal(|| 0_usize);

    // Pagination state
    let mut current_offset = use_signal(|| 0_u32);
    let mut is_loading_more = use_signal(|| false);
    let pagination_limit = 100_u32; // Backend default
    let load_more_threshold = 10_usize;
    let mut pagination_exhausted = use_signal(|| false);

    let mut is_loading_cards = use_signal(|| false);

    let current_card = move || {
        let idx = current_index();
        cards().get(idx).cloned()
    };

    // Load more cards with pagination and de-duplication
    let mut load_more_cards = move || {
        // Check if we've hit the card limit
        let current_card_count = cards().len();
        if current_card_count >= MAX_CARDS_IN_STACK {
            toast.warning(
                "card limit reached, please refresh to continue".to_string(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
            return;
        }

        if is_loading_more() {
            return;
        }

        is_loading_more.set(true);

        let mut builder = filter_builder.read().clone();
        builder.set_is_token(false);
        builder.set_limit(pagination_limit);
        builder.set_offset(current_offset());

        let Ok(filter) = builder.build() else {
            is_loading_more.set(false);
            return;
        };

        session.upkeep(client);
        let Some(session) = session() else {
            is_loading_more.set(false);
            return;
        };

        spawn(async move {
            match client().search_cards(&filter, &session).await {
                Ok(new_cards) => {
                    let existing_cards = cards();
                    let deck_ids = deck_cards_ids();

                    // Get existing card IDs for de-duplication
                    let existing_ids: HashSet<Uuid> = existing_cards
                        .iter()
                        .map(|c| c.card_profile.scryfall_data_id)
                        .collect();

                    // Filter out duplicates, deck cards, and cards without images
                    let unique_new_cards: Vec<Card> = new_cards
                        .into_iter()
                        .filter(|card| {
                            !existing_ids.contains(&card.card_profile.scryfall_data_id)
                                && !deck_ids.contains(&card.scryfall_data.id)
                                && card
                                    .scryfall_data
                                    .image_uris
                                    .as_ref()
                                    .and_then(|x| x.large.as_ref())
                                    .is_some()
                        })
                        .collect();

                    // Append unique cards to existing list
                    if !unique_new_cards.is_empty() {
                        let mut updated_cards = existing_cards;
                        updated_cards.extend(unique_new_cards);
                        cards.set(updated_cards);

                        // Update offset for next load
                        current_offset.set(current_offset() + pagination_limit);
                    } else {
                        pagination_exhausted.set(true);
                    }

                    is_loading_more.set(false);
                }
                Err(e) => {
                    tracing::warn!("pagination load failed: {e}");
                    is_loading_more.set(false);
                }
            }
        });
    };

    let mut next_card = move || {
        let idx = current_index();
        let total_cards = cards().len();

        if idx + 1 < total_cards {
            current_index.set(idx + 1);

            // Show warning when approaching limit
            if (CARDS_WARNING_THRESHOLD..MAX_CARDS_IN_STACK).contains(&total_cards)
                && total_cards.is_multiple_of(100)
            {
                // Show every 100 cards after threshold
                toast.info(
                    "approaching card limit, consider refreshing".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(2000)),
                );
            }

            // Check if we should load more cards (within threshold of end)
            if total_cards > 0 && idx + 1 >= total_cards.saturating_sub(load_more_threshold) {
                load_more_cards();
            }
        } else {
            // At the end - check if exhausted or try loading more
            if pagination_exhausted() {
                toast.warning("end of results".to_string(), ToastOptions::default());
            } else {
                load_more_cards();
            }
        }
    };

    // Swipeable state
    let swipe_state = use_signal(SwipeState::new);
    let swipe_config = SwipeConfig::new(
        vec![Direction::Left, Direction::Right, Direction::Down],
        150.0, // 150px to commit swipe
        5.0,   // 5px/ms speed threshold
    );

    let add_card_to_deck = move || {
        let Some(card) = current_card() else {
            return; // No card to add
        };

        session.upkeep(client);
        let Some(session) = session() else {
            toast.error("session expired".to_string(), ToastOptions::default());
            return;
        };

        // For now, always add quantity 1 (will add quantity picker later)
        let request = HttpCreateDeckCard::new(&card.scryfall_data.id.to_string(), 1);
        let card_id = card.scryfall_data.id;

        spawn(async move {
            match client().create_deck_card(deck_id, &request, &session).await {
                Ok(_) => {
                    deck_cards_ids.write().insert(card_id);
                }
                Err(e) => {
                    tracing::warn!("add card to deck failed: {e}");
                    toast.error(e.to_string(), ToastOptions::default());
                }
            }
        });
    };

    let mut undo_last_action = move || {
        // Pop last action from history
        let Some(action) = action_history.write().pop() else {
            toast.info("nothing to undo".to_string(), ToastOptions::default());
            return;
        };

        // Can't undo if we're at the first card
        if current_index() == 0 {
            toast.warning("no previous card".to_string(), ToastOptions::default());
            action_history.write().push(action); // Put it back
            return;
        }

        // Go back one card
        current_index.set(current_index() - 1);

        match action {
            SwipeAction::Skip(_) => {
                // Just showing previous card - done!
                toast.info(
                    "undid skip".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            }
            SwipeAction::Do(ref card) => {
                // Need to delete from backend (undoing the add)
                session.upkeep(client);
                let Some(session) = session() else {
                    toast.error("session expired".to_string(), ToastOptions::default());
                    action_history.write().push(action); // Restore history
                    current_index.set(current_index() + 1); // Restore index
                    return;
                };

                let card_id = card.scryfall_data.id;

                spawn(async move {
                    match client().delete_deck_card(deck_id, card_id, &session).await {
                        Ok(_) => {
                            // Remove from exclusion HashSet
                            deck_cards_ids.write().remove(&card_id);
                            toast.success(
                                "undid add".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo add (delete deck card) failed: {e}");
                            toast.error(format!("failed to undo: {}", e), ToastOptions::default());
                            // Don't restore action or index - user can try again by adding the card
                        }
                    }
                });
            }
        }
    };

    let mut clear_filters = move || {
        let opts = ToastOptions::default().duration(Duration::from_millis(1500));
        if filter_builder.read().is_empty_ignoring_deck_context() {
            toast.warning("filter already cleared".to_string(), opts);
        } else {
            filter_builder.write().clear();
            // Re-apply deck format after clear
            if let Some(fmt) = deck_format() {
                filter_builder.write().set_legalities_contains_any(
                    vec![fmt.to_legality_key().to_string()]
                );
            }
            cards.set(vec![]);
            current_index.set(0);
            toast.info("filter cleared".to_string(), opts);
        }
    };

    // Fetch deck cards on mount for filtering
    use_effect(move || {
        session.upkeep(client);
        let Some(session) = session() else {
            return;
        };

        spawn(async move {
            match client().get_deck(deck_id, &session).await {
                Ok(deck) => {
                    let mut ids: HashSet<_> = deck
                        .entries
                        .iter()
                        .map(|entry| entry.card.scryfall_data.id)
                        .collect();
                    if let Some(commander_id) = deck.deck_profile.commander_id {
                        ids.insert(commander_id);
                    }
                    deck_cards_ids.set(ids);

                    // Pre-populate format filter from deck
                    if let Some(fmt) = deck.deck_profile.format {
                        deck_format.set(Some(fmt));
                        if filter_builder.peek().legalities_contains_any().is_none() {
                            filter_builder.write().set_legalities_contains_any(
                                vec![fmt.to_legality_key().to_string()]
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "deck card filter fetch failed, continuing without filtering: {e}"
                    );
                }
            }
        });
    });

    use_effect(move || {
        let _ = filter_reset_counter();

        let first = is_first_run.get();
        is_first_run.set(false);

        let mut builder = filter_builder.peek().clone();
        builder.set_is_token(false);
        builder.set_limit(pagination_limit);
        builder.set_offset(0);

        let effectively_empty = builder.is_empty_ignoring_deck_context();

        if first {
            // ── Initial mount ─────────────────────────────────────────
            if effectively_empty {
                // Filter is default — clear any stale cards and don't search
                cards.set(vec![]);
                last_search_filter.set(None);
                current_offset.set(0);
                current_index.set(0);
                toast.warning("filter is empty".to_string(), ToastOptions::default().duration(Duration::from_millis(2000)));
                return;
            }
            // Preserve cards if the filter hasn't changed since last search.
            let filter_unchanged = last_search_filter
                .peek()
                .as_ref()
                .map(|prev| {
                    let mut prev_b = prev.clone();
                    prev_b.set_is_token(false);
                    prev_b.set_limit(pagination_limit);
                    prev_b.set_offset(0);
                    prev_b == builder
                })
                .unwrap_or(false);

            if filter_unchanged && !cards.peek().is_empty() {
                // Restore pagination offset so load-more picks up from the right place.
                current_offset.set(cards.peek().len() as u32);
                return;
            }
        }

        // ── Clear and re-fetch ────────────────────────────────────────
        // Reaches here when:
        //   - explicit user action (refresh / apply filter), OR
        //   - initial mount with a different/new filter, OR
        //   - initial mount with no existing cards
        cards.set(vec![]);
        last_search_filter.set(None);
        current_offset.set(0);
        current_index.set(0);
        pagination_exhausted.set(false);

        if effectively_empty {
            // Filter is default (empty or only deck format) — don't search
            return;
        }

        let Ok(filter) = builder.build() else {
            return;
        };

        // Peek session to avoid subscribing this effect to session changes.
        // The interval-based upkeep in Bouncer handles session refresh.
        let Some(session) = session.peek().clone() else {
            toast.error("session expired".to_string(), ToastOptions::default());
            return;
        };

        is_loading_cards.set(true);

        // Snapshot the filter builder state before the async block owns context.
        let filter_snapshot = filter_builder.peek().clone();

        spawn(async move {
            match client().search_cards(&filter, &session).await {
                Ok(cards_from_search) => {
                    let deck_ids = deck_cards_ids();
                    cards.set(
                        cards_from_search
                            .into_iter()
                            .filter(|card| {
                                card.scryfall_data
                                    .image_uris
                                    .as_ref()
                                    .and_then(|x| x.large.as_ref())
                                    .is_some()
                                    && !deck_ids.contains(&card.scryfall_data.id)
                            })
                            .collect(),
                    );
                    // Record the filter that produced these results.
                    last_search_filter.set(Some(filter_snapshot));
                    // Set offset for next page
                    current_offset.set(pagination_limit);
                    is_loading_cards.set(false);
                }
                Err(e) => {
                    tracing::warn!("card search failed: {e}");
                    toast.error(e.to_string(), ToastOptions::default());
                    is_loading_cards.set(false);
                }
            }
        });
    });

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "add deck card" }
                }

                div { class: "screen-content card-swipe content-enter",

                div { class : "form-container",
                    // Show current card with Swipeable
                    if let Some(card) = current_card() {
                        if let Some(ImageUris { large: Some(image_url), ..}) = &card.scryfall_data.image_uris {
                            Swipeable {
                                state: swipe_state,
                                config: swipe_config,
                                on_swipe_left: move |_| {
                                    let Some(card) = current_card() else { return; };
                                    action_history.write().push(SwipeAction::Skip(Box::new(card)));
                                    toast.info("skipped".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                    is_animating.set(true);
                                    animation_direction.set(Direction::Left);
                                },
                                on_swipe_right: move |_| {
                                    let Some(card) = current_card() else { return; };
                                    action_history.write().push(SwipeAction::Do(Box::new(card)));
                                    // Add card to deck then trigger exit animation
                                    add_card_to_deck();
                                    toast.success("added to deck".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                    is_animating.set(true);
                                    animation_direction.set(Direction::Right);
                                },
                                on_swipe_up: move |_| {},     // Not used
                                on_swipe_down: move |_| {
                                    undo_last_action();
                                },

                                img {
                                    src: "{image_url}",
                                    alt: "{card.scryfall_data.name}",
                                    class: "card-image",
                                    class: if is_animating() { "card-exit-animation" } else { "" },
                                    style: if is_animating() {
                                        format!("--card-exit-direction: card-exit-{}", animation_direction().to_string().to_lowercase())
                                    } else {
                                        String::new()
                                    },
                                    onanimationend: move |_| {
                                        is_animating.set(false);
                                        next_card();
                                    }
                                }
                            }
                        }

                        CardInfoDisplay { card }
                    } else if is_loading_cards() {
                        CardSkeleton { is_loading: true }
                    } else {
                        CardSkeleton {}
                    }

                }
            }

            div {
                class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.go_back();
                    },
                    "back"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| filters_overlay_open.set(true),
                    "filter"
                    if !filter_builder.read().is_empty_ignoring_deck_context() {
                        span { class: "filter-dot" }
                    }
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        let mut builder = filter_builder.peek().clone();
                        builder.set_is_token(false);
                        builder.set_limit(pagination_limit);
                        builder.set_offset(0);

                        let Ok(filter) = builder.build() else {
                            toast.warning(
                                "filter is empty".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                            return;
                        };

                        let Some(session) = session.peek().clone() else {
                            toast.error("session expired".to_string(), ToastOptions::default());
                            return;
                        };

                        cards.set(vec![]);
                        last_search_filter.set(None);
                        current_offset.set(0);
                        current_index.set(0);
                        pagination_exhausted.set(false);
                        is_loading_cards.set(true);

                        let filter_snapshot = filter_builder.peek().clone();

                        spawn(async move {
                            match client().search_cards(&filter, &session).await {
                                Ok(cards_from_search) => {
                                    let deck_ids = deck_cards_ids();
                                    cards.set(
                                        cards_from_search
                                            .into_iter()
                                            .filter(|card| {
                                                card.scryfall_data
                                                    .image_uris
                                                    .as_ref()
                                                    .and_then(|x| x.large.as_ref())
                                                    .is_some()
                                                    && !deck_ids.contains(&card.scryfall_data.id)
                                            })
                                            .collect(),
                                    );
                                    last_search_filter.set(Some(filter_snapshot));
                                    current_offset.set(pagination_limit);
                                    is_loading_cards.set(false);
                                }
                                Err(e) => {
                                    tracing::warn!("card search failed: {e}");
                                    toast.error(e.to_string(), ToastOptions::default());
                                    is_loading_cards.set(false);
                                }
                            }
                        });

                        toast.info(
                            "stack refreshed".to_string(),
                            ToastOptions::default().duration(Duration::from_millis(1500)),
                        );
                    },
                    "refresh"
                }
            }

            CardFilterSheet {
                open: filters_overlay_open,
                show_format_filter: true,
                show_active_indicators: true,
                validate_before_apply: true,
                on_clear: move |_| clear_filters(),
            }
            }
        }
    }
}

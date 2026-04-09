use super::components::card_info::{CardInfoDisplay, CardSkeleton};
use crate::{
    inbound::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{SwipeStack, config::SwipeConfig, direction::Direction},
        },
        screens::deck::card::{
            components::action_history::{CARDS_WARNING_THRESHOLD, MAX_CARDS_IN_STACK, SwipeAction},
            filter::card_filter_sheet::CardFilterSheet,
        },
    },
    outbound::client::{
        ZwipeClient,
        card::{get_card::ClientGetCard, search_cards::ClientSearchCards},
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
use zwipe_core::domain::card::{Card, scryfall_data::colors::{Color, Colors}, search_card::card_filter::builder::CardFilterBuilder};
use zwipe_core::domain::deck::format::Format;
use zwipe_core::http::contracts::deck_card::HttpCreateDeckCard;
use zwipe_core::domain::auth::models::session::Session;

#[component]
pub fn Add(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    let mut cards: Signal<Vec<Card>> = use_context();
    let mut last_search_filter: Signal<Option<CardFilterBuilder>> = use_context();
    let is_first_run = use_hook(|| std::cell::Cell::new(true));

    // When Some, the SwipeStack plays a keyframe entering from this direction
    // on the next top card, and clears it on animationend. Set by undo.
    let mut entering_direction: Signal<Option<Direction>> = use_signal(|| None);

    let mut deck_cards_ids = use_signal(HashSet::<Uuid>::new);
    let mut deck_format: Signal<Option<Format>> = use_signal(|| None);
    let mut deck_color_identity: Signal<Option<Colors>> = use_signal(|| None);

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
    // Keep the buffer comfortably ahead of the STACK_DEPTH window so the
    // stack never visibly shrinks while a batch is in flight.
    let load_more_threshold = 15_usize;
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
                                && !card.scryfall_data.oracle_id.is_some_and(|oid| deck_ids.contains(&oid))
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

    // Swipe config — the stack owns its own SwipeState internally.
    // Thresholds tuned for responsive rapid swiping: a short drag (60px) OR
    // a quick flick (1.5 px/ms over the 10px minimum) commits.
    let swipe_config = SwipeConfig::new(
        vec![Direction::Left, Direction::Right, Direction::Up, Direction::Down],
        60.0, // distance threshold in px
        1.5,  // speed threshold in px/ms
    );

    // Advance past the just-committed card. The stack fires its on_swipe_*
    // callbacks after the exit transition, so by now the card is off-screen.
    let mut advance_after_commit = move || {
        let total = cards().len();
        if current_index() + 1 < total {
            current_index.set(current_index() + 1);
            if (CARDS_WARNING_THRESHOLD..MAX_CARDS_IN_STACK).contains(&total)
                && total.is_multiple_of(100)
            {
                toast.info(
                    "approaching card limit, consider refreshing".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(2000)),
                );
            }
            // Trigger a pagination prefetch when we're within the threshold.
            if total > 0 && current_index() + 1 >= total.saturating_sub(load_more_threshold) {
                load_more_cards();
            }
        } else {
            // At the end — try to load more, else inform the user.
            if pagination_exhausted() {
                toast.warning("end of results".to_string(), ToastOptions::default());
            } else {
                load_more_cards();
            }
        }
    };

    let add_card_to_deck = move |card: Card| {
        session.upkeep(client);
        let Some(session) = session() else {
            toast.error("session expired".to_string(), ToastOptions::default());
            return;
        };

        // For now, always add quantity 1 (will add quantity picker later)
        let request = HttpCreateDeckCard::new(&card.scryfall_data, 1, None);
        let oracle_id = card.scryfall_data.oracle_id;

        spawn(async move {
            match client().create_deck_card(deck_id, &request, &session).await {
                Ok(_) => {
                    if let Some(oid) = oracle_id { deck_cards_ids.write().insert(oid); }
                }
                Err(e) => {
                    tracing::warn!("add card to deck failed: {e}");
                    toast.error(e.to_string(), ToastOptions::default());
                }
            }
        });
    };

    let add_card_to_maybeboard = move |card: Card| {
        session.upkeep(client);
        let Some(session) = session() else {
            toast.error("session expired".to_string(), ToastOptions::default());
            return;
        };

        let request = HttpCreateDeckCard::new(&card.scryfall_data, 1, Some("maybeboard".to_string()));
        let oracle_id = card.scryfall_data.oracle_id;

        spawn(async move {
            match client().create_deck_card(deck_id, &request, &session).await {
                Ok(_) => {
                    if let Some(oid) = oracle_id { deck_cards_ids.write().insert(oid); }
                }
                Err(e) => {
                    tracing::warn!("add card to maybeboard failed: {e}");
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

        // Go back one card — the previously-swiped card becomes the new top.
        current_index.set(current_index() - 1);
        // Ask the stack to play the enter animation from the direction the
        // card originally exited.
        entering_direction.set(Some(action.exited().clone()));

        match action {
            SwipeAction::Skip { .. } => {
                // Just showing previous card - done!
                toast.info(
                    "undid skip".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            }
            SwipeAction::Do { ref card, .. } => {
                // Need to delete from backend (undoing the add)
                session.upkeep(client);
                let Some(session) = session() else {
                    toast.error("session expired".to_string(), ToastOptions::default());
                    action_history.write().push(action); // Restore history
                    current_index.set(current_index() + 1); // Restore index
                    entering_direction.set(None);
                    return;
                };

                let card_id = card.scryfall_data.id;
                let oracle_id = card.scryfall_data.oracle_id;

                spawn(async move {
                    match client().delete_deck_card(deck_id, card_id, &session).await {
                        Ok(_) => {
                            // Remove from exclusion HashSet
                            if let Some(oid) = oracle_id { deck_cards_ids.write().remove(&oid); }
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
            SwipeAction::Maybeboard { ref card, .. } => {
                session.upkeep(client);
                let Some(session) = session() else {
                    toast.error("session expired".to_string(), ToastOptions::default());
                    action_history.write().push(action);
                    current_index.set(current_index() + 1);
                    entering_direction.set(None);
                    return;
                };

                let card_id = card.scryfall_data.id;
                let oracle_id = card.scryfall_data.oracle_id;

                spawn(async move {
                    match client().delete_deck_card(deck_id, card_id, &session).await {
                        Ok(_) => {
                            if let Some(oid) = oracle_id { deck_cards_ids.write().remove(&oid); }
                            toast.success(
                                "undid maybeboard".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo maybeboard (delete deck card) failed: {e}");
                            toast.error(format!("failed to undo: {}", e), ToastOptions::default());
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
            // Re-apply deck context defaults after clear
            if let Some(fmt) = deck_format() {
                filter_builder.write().set_legalities_contains_any(
                    vec![fmt.to_legality_key().to_string()]
                );
            }
            if let Some(colors) = deck_color_identity() {
                filter_builder.write().set_color_identity_within(colors);
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
                        .filter_map(|entry| entry.card.scryfall_data.oracle_id)
                        .collect();
                    // Resolve command zone cards to oracle_ids for exclusion
                    // and collect color identities from commander/partner/background
                    let mut identity_colors: Vec<Color> = Vec::new();
                    for cz_id in [
                        deck.deck_profile.commander_id,
                        deck.deck_profile.partner_commander_id,
                        deck.deck_profile.background_id,
                    ]
                    .into_iter()
                    .flatten()
                    {
                        if let Ok(card) = client().get_card(cz_id, &session).await
                            && let Some(oid) = card.scryfall_data.oracle_id
                        {
                            ids.insert(oid);
                            identity_colors.extend(card.scryfall_data.color_identity.iter().cloned());
                        }
                    }
                    // Signature spell: oracle_id exclusion only (doesn't contribute to color identity)
                    if let Some(spell_id) = deck.deck_profile.signature_spell_id
                        && let Ok(card) = client().get_card(spell_id, &session).await
                        && let Some(oid) = card.scryfall_data.oracle_id
                    {
                        ids.insert(oid);
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

                        // Pre-populate color identity filter from commander
                        if fmt.checks_color_identity() && deck.deck_profile.commander_id.is_some() {
                            identity_colors.sort();
                            identity_colors.dedup();
                            let colors: Colors = identity_colors.into();
                            deck_color_identity.set(Some(colors.clone()));
                            if filter_builder.peek().color_identity_within().is_none() {
                                filter_builder.write().set_color_identity_within(colors);
                            }
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
                                    && !card.scryfall_data.oracle_id.is_some_and(|oid| deck_ids.contains(&oid))
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
                    if !cards().is_empty() {
                        SwipeStack {
                            cards: {
                                let all = cards();
                                all.into_iter().skip(current_index()).take(crate::inbound::components::interactions::swipe::STACK_DEPTH).collect::<Vec<_>>()
                            },
                            config: swipe_config,
                            entering: entering_direction,
                            on_swipe_left: move |card: Card| {
                                action_history.write().push(SwipeAction::Skip { card: Box::new(card), exited: Direction::Left });
                                toast.info("skipped".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                advance_after_commit();
                            },
                            on_swipe_right: move |card: Card| {
                                action_history.write().push(SwipeAction::Do { card: Box::new(card.clone()), exited: Direction::Right });
                                add_card_to_deck(card);
                                toast.success("added to deck".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                advance_after_commit();
                            },
                            on_swipe_up: move |card: Card| {
                                action_history.write().push(SwipeAction::Maybeboard { card: Box::new(card.clone()), exited: Direction::Up });
                                add_card_to_maybeboard(card);
                                toast.info("added to maybeboard".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                advance_after_commit();
                            },
                            on_swipe_down: move |_card: Card| {
                                undo_last_action();
                            },
                        }

                        if let Some(card) = current_card() {
                            CardInfoDisplay { card }
                        }
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
                        // Clear auto-populated defaults so view/remove screens start fresh
                        if filter_builder.read().is_empty_ignoring_deck_context() {
                            filter_builder.write().clear();
                        }
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
                                                    && !card.scryfall_data.oracle_id.is_some_and(|oid| deck_ids.contains(&oid))
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

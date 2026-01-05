use crate::{
    inbound::{
        components::{
            accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger},
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{
                config::SwipeConfig, direction::Direction, state::SwipeState, Swipeable,
            },
        },
        screens::deck::card::{
            action_history::{SwipeAction, CARDS_WARNING_THRESHOLD, MAX_CARDS_IN_STACK},
            filter::{
                combat::Combat, mana::Mana, rarity::Rarity, set::Set, sort::Sort, text::Text,
                types::Types,
            },
        },
    },
    outbound::client::{
        card::search_cards::ClientSearchCards,
        deck::get_deck::ClientGetDeck,
        deck_card::{
            create_deck_card::ClientCreateDeckCard, delete_deck_card::ClientDeleteDeckCard,
        },
        ZwipeClient,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::collections::HashSet;
use std::time::Duration;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{
            scryfall_data::image_uris::ImageUris,
            search_card::card_filter::builder::CardFilterBuilder, Card,
        },
    },
    inbound::http::handlers::deck_card::create_deck_card::HttpCreateDeckCard,
};

#[component]
pub fn Add(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    let mut cards: Signal<Vec<Card>> = use_context();

    let mut add_card_error = use_signal(|| None::<String>);
    let mut search_error = use_signal(|| None::<String>);

    let mut is_animating = use_signal(|| false);
    let mut animation_direction = use_signal(|| Direction::Left);

    let mut deck_cards_ids = use_signal(HashSet::<Uuid>::new);

    // Undo action history
    let mut action_history: Signal<Vec<SwipeAction>> = use_signal(Vec::new);

    // Filters overlay at bottom of screen state
    let mut filters_overlay_open = use_signal(|| false);

    // Reset counter for collapsing accordions and clearing search queries
    let mut filter_reset_counter: Signal<u32> = use_signal(|| 0);
    use_context_provider(|| filter_reset_counter);

    // Refresh trigger - incrementing this re-runs the card search effect
    let mut refresh_trigger = use_signal(|| false);

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
        builder.set_limit(pagination_limit);
        builder.set_offset(current_offset());

        let Ok(filter) = builder.build() else {
            is_loading_more.set(false);
            return;
        };

        session.upkeep(client);
        let Some(sesh) = session() else {
            is_loading_more.set(false);
            return;
        };

        spawn(async move {
            match client().search_cards(&filter, &sesh).await {
                Ok(new_cards) => {
                    let existing_cards = cards();
                    let deck_ids = deck_cards_ids();

                    // Get existing card IDs for de-duplication
                    let existing_ids: HashSet<Uuid> =
                        existing_cards.iter().map(|c| c.card_profile.id).collect();

                    // Filter out duplicates, deck cards, and cards without images
                    let unique_new_cards: Vec<Card> = new_cards
                        .into_iter()
                        .filter(|card| {
                            !existing_ids.contains(&card.card_profile.id)
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
                Err(_) => {
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

    let mut add_card_to_deck = move || {
        let Some(card) = current_card() else {
            return; // No card to add
        };

        session.upkeep(client);
        let Some(sesh) = session() else {
            add_card_error.set(Some("session expired".to_string()));
            return;
        };

        // For now, always add quantity 1 (will add quantity picker later)
        let request = HttpCreateDeckCard::new(&card.scryfall_data.id.to_string(), 1);
        let card_id = card.scryfall_data.id;

        spawn(async move {
            match client().create_deck_card(deck_id, &request, &sesh).await {
                Ok(_) => {
                    add_card_error.set(None);
                    deck_cards_ids.write().insert(card_id);
                }
                Err(e) => {
                    add_card_error.set(Some(e.to_string()));
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
            SwipeAction::Skip => {
                // Just showing previous card - done!
                toast.info(
                    "undid skip".to_string(),
                    ToastOptions::default().duration(Duration::from_millis(1500)),
                );
            }
            SwipeAction::Do => {
                // Need to delete from backend (undoing the add)
                let Some(card) = current_card() else {
                    toast.error("card not found".to_string(), ToastOptions::default());
                    return;
                };

                session.upkeep(client);
                let Some(sesh) = session() else {
                    toast.error("session expired".to_string(), ToastOptions::default());
                    action_history.write().push(action); // Restore history
                    current_index.set(current_index() + 1); // Restore index
                    return;
                };

                let card_id = card.scryfall_data.id;

                spawn(async move {
                    match client().delete_deck_card(deck_id, card_id, &sesh).await {
                        Ok(_) => {
                            // Remove from exclusion HashSet
                            deck_cards_ids.write().remove(&card_id);
                            toast.success(
                                "undid add".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            toast.error(format!("failed to undo: {}", e), ToastOptions::default());
                            // Don't restore action or index - user can try again by adding the card
                        }
                    }
                });
            }
        }
    };

    let mut clear_filters = move || {
        filter_builder.write().clear_all();
    };

    // Fetch deck cards on mount for filtering
    use_effect(move || {
        session.upkeep(client);
        let Some(sesh) = session() else {
            return;
        };

        spawn(async move {
            match client().get_deck(deck_id, &sesh).await {
                Ok(deck) => {
                    let ids: HashSet<_> = deck
                        .cards
                        .iter()
                        .map(|card| card.scryfall_data.id)
                        .collect();
                    deck_cards_ids.set(ids);
                }
                Err(_) => {
                    // Continue without deck card filtering if fetch fails
                }
            }
        });
    });

    use_effect(move || {
        // Read refresh_trigger to make effect re-run when button clicked
        let _ = refresh_trigger();
        let _ = filter_reset_counter();

        // Reset pagination when filter changes or refresh triggered
        current_offset.set(0);
        current_index.set(0);
        pagination_exhausted.set(false);

        let mut builder = filter_builder.peek().clone();
        builder.set_limit(pagination_limit);
        builder.set_offset(0);

        let Ok(filter) = builder.build() else {
            return;
        };

        // Don't call session.upkeep here - it creates a loop when session updates
        // The interval-based upkeep in Bouncer handles session refresh
        let Some(sesh) = session() else {
            search_error.set(Some("session expired".to_string()));
            return;
        };

        spawn(async move {
            match client().search_cards(&filter, &sesh).await {
                Ok(cards_from_search) => {
                    search_error.set(None);
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
                    // Set offset for next page
                    current_offset.set(pagination_limit);
                }
                Err(e) => search_error.set(Some(e.to_string())),
            }
        });
    });

    rsx! {
        Bouncer {
            div { class: "page-header",
                h2 { "add deck card" }
            }

            div { class: "sticky top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center; padding-top: 4rem;",

                div { class : "form-container",
                    // Show current card with Swipeable
                    if let Some(card) = current_card() {
                        if let Some(ImageUris { large: Some(image_url), ..}) = &card.scryfall_data.image_uris {
                            Swipeable {
                                state: swipe_state,
                                config: swipe_config,
                                on_swipe_left: move |_| {
                                    action_history.write().push(SwipeAction::Skip);
                                    // Skip card - trigger exit animation
                                    toast.info("skipped".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                                    is_animating.set(true);
                                    animation_direction.set(Direction::Left);
                                },
                                on_swipe_right: move |_| {
                                    action_history.write().push(SwipeAction::Do);
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

                        div { class: "card-info",
                            if card.scryfall_data.prices.usd.is_some()
                                || card.scryfall_data.prices.eur.is_some()
                                || card.scryfall_data.prices.tix.is_some() {
                                    {
                                        let mut display: String = String::from("prices:");
                                        let mut prices_count = 0;
                                        if let Some(usd) = card.scryfall_data.prices.usd {
                                            display.push_str(format!(" ${usd}").as_str());
                                            prices_count += 1;
                                        }
                                        if let Some(eur) = card.scryfall_data.prices.eur {
                                            if prices_count > 0 {
                                                display.push_str(" |");
                                            }
                                            display.push_str(format!(" €{eur}").as_str());
                                            prices_count += 1;
                                        }
                                        if let Some(tix) = card.scryfall_data.prices.tix {
                                            if prices_count > 0 {
                                                display.push_str(" |");
                                            }
                                            display.push_str(format!(" {tix} tix").as_str());
                                        }
                                        rsx! { span { "{display}" } }
                                    }
                            }
                            span { "released: {card.scryfall_data.released_at}" },
                            if let Some(artist) = card.scryfall_data.artist && !artist.is_empty() {
                                span { "artist: {artist}" }
                            }
                        }
                    } else {
                        // Empty state (no cards)
                        div { class : "card-shape flex-center",
                            "no cards"
                        }
                    }

                    if let Some(add_card_error) = add_card_error() {
                        div { class : "message-error", "{add_card_error}"}
                    }

                    if let Some(search_error) = search_error() {
                        div { class : "message-error", "{search_error}" }
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
                    "filters"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        refresh_trigger.set(!refresh_trigger());
                    },
                    "refresh"
                }
            }

            // Modal backdrop (always rendered for CSS animation)
            div {
                class: if filters_overlay_open() { "modal-backdrop show" } else { "modal-backdrop" },
                onclick: move |_| filters_overlay_open.set(false),
            }

            // Bottom sheet (always rendered for CSS animation)
            div {
                class: if filters_overlay_open() { "bottom-sheet show" } else { "bottom-sheet" },

                // Header with apply button
                div { class: "modal-header",
                    button {
                        class: "btn btn-sm",
                        onclick: move |_| {
                            if filter_builder.read().is_empty() {
                                toast.warning("try adding a filter".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
                            } else {
                                filter_reset_counter.set(filter_reset_counter() + 1);
                            }
                            filters_overlay_open.set(false);
                        },
                        "apply"
                    }
                }

                // Content with accordion
                div { class: "modal-content",
                    Accordion {
                        key: "{filter_reset_counter()}",
                        id: "filter-accordion",
                        allow_multiple_open: false,
                        collapsible: true,

                        AccordionItem { index: 1,
                            AccordionTrigger { "text" }
                            AccordionContent { Text {} }
                        }

                        AccordionItem { index: 2,
                            AccordionTrigger { "types" }
                            AccordionContent { Types {} }
                        }

                        AccordionItem { index: 3,
                            AccordionTrigger { "mana" }
                            AccordionContent { Mana {} }
                        }

                        AccordionItem { index: 4,
                            AccordionTrigger { "combat" }
                            AccordionContent { Combat {} }
                        }

                        AccordionItem { index: 5,
                            AccordionTrigger { "rarity" }
                            AccordionContent { Rarity {} }
                        }

                        AccordionItem { index: 6,
                            AccordionTrigger { "set" }
                            AccordionContent { Set {} }
                        }

                        AccordionItem { index: 7,
                            AccordionTrigger { "sort" }
                            AccordionContent { Sort {} }
                        }
                  }
                }

                // Footer with clear button
                div { class: "modal-footer",
                    button {
                        class: "btn btn-sm",
                        onclick: move |_| {
                            clear_filters();
                        },
                        "clear"
                    }
                }
            }
        }
    }
}

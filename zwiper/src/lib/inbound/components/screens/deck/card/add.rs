use crate::{
    inbound::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{
                config::SwipeConfig, direction::Direction, state::SwipeState, Swipeable,
            },
        },
        router::Router,
    },
    outbound::client::{
        card::search_cards::ClientSearchCards, deck_card::create_deck_card::ClientCreateDeckCard,
        ZwipeClient,
    },
};
use dioxus::prelude::*;
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

    let filter_builder: Signal<CardFilterBuilder> = use_context();
    let mut cards: Signal<Vec<Card>> = use_context();

    let mut add_card_error = use_signal(|| None::<String>);
    let mut search_error = use_signal(|| None::<String>);

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    // Card iteration state
    let mut current_index = use_signal(|| 0_usize);

    // Pagination state
    let mut current_offset = use_signal(|| 0_u32);
    let mut is_loading_more = use_signal(|| false);
    let pagination_limit = 100_u32; // Match backend default
    let load_more_threshold = 10_usize; // Trigger load when within 10 cards of end

    let current_card = move || {
        let idx = current_index();
        cards().get(idx).cloned()
    };

    // Load more cards with pagination and de-duplication
    let mut load_more_cards = move || {
        if is_loading_more() {
            return; // Already loading
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

                    // Get existing card IDs for de-duplication
                    let existing_ids: std::collections::HashSet<_> = existing_cards
                        .iter()
                        .map(|c| c.card_profile.id.clone())
                        .collect();

                    // Filter out duplicates and cards without images
                    let unique_new_cards: Vec<Card> = new_cards
                        .into_iter()
                        .filter(|card| {
                            !existing_ids.contains(&card.card_profile.id)
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

            // Check if we should load more cards (within threshold of end)
            if total_cards > 0 && idx + 1 >= total_cards.saturating_sub(load_more_threshold) {
                load_more_cards();
            }
        } else {
            // At the end - try loading more
            load_more_cards();
        }
    };

    // Swipeable state
    let swipe_state = use_signal(SwipeState::new);
    let swipe_config = SwipeConfig::new(
        vec![Direction::Left, Direction::Right],
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

        spawn(async move {
            match client().create_deck_card(deck_id, &request, &sesh).await {
                Ok(_) => {
                    add_card_error.set(None);
                }
                Err(e) => {
                    add_card_error.set(Some(e.to_string()));
                }
            }
        });
    };

    use_effect(move || {
        // Reset pagination when filter changes
        current_offset.set(0);
        current_index.set(0);

        let mut builder = filter_builder.read().clone();
        builder.set_limit(pagination_limit);
        builder.set_offset(0);

        let Ok(filter) = builder.build() else {
            return;
        };

        session.upkeep(client);
        let Some(sesh) = session() else {
            search_error.set(Some("session expired".to_string()));
            return;
        };

        spawn(async move {
            match client().search_cards(&filter, &sesh).await {
                Ok(cards_from_search) => {
                    search_error.set(None);
                    cards.set(
                        cards_from_search
                            .into_iter()
                            .filter(|card| {
                                card.scryfall_data
                                    .image_uris
                                    .as_ref()
                                    .and_then(|x| x.large.as_ref())
                                    .is_some()
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
            div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center;",
                h2 { class: "text-center mb-2 font-light tracking-wider", "add deck card" }

                div { class : "form-container",
                    // Show current card with Swipeable
                    if let Some(card) = current_card() {
                        if let Some(ImageUris { large: Some(image_url), ..}) = &card.scryfall_data.image_uris {
                            Swipeable {
                                state: swipe_state,
                                config: swipe_config.clone(),
                                on_swipe_left: move |_| {
                                    // Skip card - advance to next
                                    next_card();
                                },
                                on_swipe_right: move |_| {
                                    // Add card to deck then advance to next
                                    add_card_to_deck();
                                    next_card();
                                },
                                on_swipe_up: move |_| {},     // Not used
                                on_swipe_down: move |_| {},   // Not used

                                img {
                                    src: "{image_url}",
                                    alt: "{card.scryfall_data.name}",
                                    class: "card-image"
                                }
                            }
                        }
                    } else {
                        // Empty state (no cards)
                        div { class : "card-shape flex-center",
                            "no cards"
                        }
                    }

                    button { class : "btn",
                        onclick : move |_| {
                            navigator.push(Router::Filter { } );
                        },
                        "filters"
                    }

                    if let Some(add_card_error) = add_card_error() {
                        div { class : "error", "{add_card_error}"}
                    }

                    if let Some(search_error) = search_error() {
                        div { class : "message-error", "{search_error}" }
                    }

                    button { class : "btn",
                        onclick: move |_| {
                            navigator.push(Router::EditDeck { deck_id });
                        },
                        "back"
                    }
                }
            }
        }
    }
}

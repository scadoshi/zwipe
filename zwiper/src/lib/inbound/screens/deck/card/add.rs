use crate::{
    inbound::{
        components::{
            accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger},
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{
                config::SwipeConfig, direction::Direction, state::SwipeState, Swipeable,
            },
        },
        router::Router,
        screens::deck::card::filter::{
            combat::Combat, mana::Mana, rarity::Rarity, set::Set, text::Text, types::Types,
        },
    },
    outbound::client::{
        card::search_cards::ClientSearchCards, deck::get_deck::ClientGetDeck,
        deck_card::create_deck_card::ClientCreateDeckCard, ZwipeClient,
    },
};
use dioxus::prelude::*;
use std::collections::HashSet;
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

    // Filters overlay at bottom of screen state
    let mut filters_overlay_open = use_signal(|| false);

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    // Card iteration state
    let mut current_index = use_signal(|| 0_usize);

    // Pagination state
    let mut current_offset = use_signal(|| 0_u32);
    let mut is_loading_more = use_signal(|| false);
    let pagination_limit = 100_u32; // Backend default
    let load_more_threshold = 10_usize;

    let current_card = move || {
        let idx = current_index();
        cards().get(idx).cloned()
    };

    // Load more cards with pagination and de-duplication
    let mut load_more_cards = move || {
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
                    // Extract scryfall_data IDs from deck cards
                    // Note: Deck struct has private fields, need to access via pattern matching
                    let ids: std::collections::HashSet<_> = deck
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

            div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center; padding-top: 4rem;",

                div { class : "form-container",
                    // Show current card with Swipeable
                    if let Some(card) = current_card() {
                        if let Some(ImageUris { large: Some(image_url), ..}) = &card.scryfall_data.image_uris {
                            Swipeable {
                                state: swipe_state,
                                config: swipe_config,
                                on_swipe_left: move |_| {
                                    // Skip card - trigger exit animation
                                    is_animating.set(true);
                                    animation_direction.set(Direction::Left);
                                },
                                on_swipe_right: move |_| {
                                    // Add card to deck then trigger exit animation
                                    add_card_to_deck();
                                    is_animating.set(true);
                                    animation_direction.set(Direction::Right);
                                },
                                on_swipe_up: move |_| {},     // Not used
                                on_swipe_down: move |_| {},   // Not used

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
                        cards.write().clear();
                    },
                    "clear cards"
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
                        onclick: move |_| filters_overlay_open.set(false),
                        "apply filters"
                    }
                }

                // Content with accordion
                div { class: "modal-content",
                    Accordion {
                        id: "filter-accordion",
                        allow_multiple_open: false,
                        collapsible: true,

                        AccordionItem { index: 1,
                            AccordionTrigger { "combat" }
                            AccordionContent { Combat {} }
                        }

                        AccordionItem { index: 2,
                            AccordionTrigger { "mana" }
                            AccordionContent { Mana {} }
                        }

                        AccordionItem { index: 3,
                            AccordionTrigger { "rarity" }
                            AccordionContent { Rarity {} }
                        }

                        AccordionItem { index: 4,
                            AccordionTrigger { "set" }
                            AccordionContent { Set {} }
                        }

                        AccordionItem { index: 5,
                            AccordionTrigger { "text" }
                            AccordionContent { Text {} }
                        }

                        AccordionItem { index: 6,
                            AccordionTrigger { "types" }
                            AccordionContent { Types {} }
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
                        "clear filters"
                    }
                }
            }
        }
    }
}

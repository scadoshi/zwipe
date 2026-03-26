use crate::{
    inbound::{
        components::{
            accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger},
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{
                Swipeable, config::SwipeConfig, direction::Direction, state::SwipeState,
            },
        },
        screens::deck::card::{
            action_history::{CARDS_WARNING_THRESHOLD, MAX_CARDS_IN_STACK, SwipeAction},
            filter::{
                artist::Artist, combat::Combat, config::Config, flavor_text::FlavorText,
                mana::Mana, name::Name, oracle_text::OracleText, rarity::Rarity, set::Set,
                sort::Sort, types::Types,
            },
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
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{
            Card, scryfall_data::image_uris::ImageUris,
            search_card::card_filter::builder::CardFilterBuilder,
        },
    },
    inbound::http::handlers::deck_card::create_deck_card::HttpCreateDeckCard,
};

#[component]
pub fn Add(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();
    let mut cards: Signal<Vec<Card>> = use_context();

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
        if filter_builder.read().is_empty() {
            toast.warning("filter already cleared".to_string(), opts);
        } else {
            filter_builder.write().clear();
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

        // Don't call session.upkeep here - it creates a loop when session updates
        // The interval-based upkeep in Bouncer handles session refresh
        let Some(session) = session() else {
            toast.error("session expired".to_string(), ToastOptions::default());
            return;
        };

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
                    // Set offset for next page
                    current_offset.set(pagination_limit);
                }
                Err(e) => toast.error(e.to_string(), ToastOptions::default()),
            }
        });
    });

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "add deck card" }
                }

                div { class: "screen-content centered",

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
                                span { "artist: {artist.to_lowercase()}" }
                            }
                        }
                    } else {
                        // Empty state (no cards)
                        div { class : "card-shape flex-center",
                            "no cards"
                        }
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
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        if filter_builder.read().is_empty() {
                            toast.warning(
                                "filter is empty".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        } else {
                            toast.info(
                                "search refreshed".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                            refresh_trigger.set(!refresh_trigger());
                        }
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
                                toast.warning("filter is empty".to_string(), ToastOptions::default().duration(Duration::from_millis(1500)));
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
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(1)'); if (el) el.scrollIntoView({ behavior: 'instant', block: 'start' }); }, 300)"); }
                            },
                            AccordionTrigger { "name" }
                            AccordionContent { Name {} }
                        }

                        AccordionItem { index: 2,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(2)'); if (el) el.scrollIntoView({ behavior: 'instant', block: 'start' }); }, 300)"); }
                            },
                            AccordionTrigger { "oracle text" }
                            AccordionContent { OracleText {} }
                        }

                        AccordionItem { index: 3,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(3)'); if (el) el.scrollIntoView({ behavior: 'instant', block: 'start' }); }, 300)"); }
                            },
                            AccordionTrigger { "types" }
                            AccordionContent { Types {} }
                        }

                        AccordionItem { index: 4,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(4)'); if (el) el.scrollIntoView({ behavior: 'instant', block: 'start' }); }, 300)"); }
                            },
                            AccordionTrigger { "mana" }
                            AccordionContent { Mana {} }
                        }

                        AccordionItem { index: 5,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(5)'); if (el) el.scrollIntoView({ behavior: 'instant', block: 'start' }); }, 300)"); }
                            },
                            AccordionTrigger { "combat" }
                            AccordionContent { Combat {} }
                        }

                        AccordionItem { index: 6,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(6)'); if (el) el.scrollIntoView({ behavior: 'instant', block: 'start' }); }, 300)"); }
                            },
                            AccordionTrigger { "flavor text" }
                            AccordionContent { FlavorText {} }
                        }

                        AccordionItem { index: 7,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(7)'); if (el) el.scrollIntoView({ behavior: 'instant', block: 'start' }); }, 300)"); }
                            },
                            AccordionTrigger { "artist" }
                            AccordionContent { Artist {} }
                        }

                        AccordionItem { index: 8,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(8)'); if (el) el.scrollIntoView({ behavior: 'instant', block: 'start' }); }, 300)"); }
                            },
                            AccordionTrigger { "rarity" }
                            AccordionContent { Rarity {} }
                        }

                        AccordionItem { index: 9,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(9)'); if (el) el.scrollIntoView({ behavior: 'instant', block: 'start' }); }, 300)"); }
                            },
                            AccordionTrigger { "set" }
                            AccordionContent { Set {} }
                        }

                        AccordionItem { index: 10,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(10)'); if (el) el.scrollIntoView({ behavior: 'instant', block: 'start' }); }, 300)"); }
                            },
                            AccordionTrigger { "sort" }
                            AccordionContent { Sort {} }
                        }

                        AccordionItem { index: 11,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(11)'); if (el) el.scrollIntoView({ behavior: 'instant', block: 'start' }); }, 300)"); }
                            },
                            AccordionTrigger { "config" }
                            AccordionContent { Config {} }
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
}

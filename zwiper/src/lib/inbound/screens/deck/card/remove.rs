use crate::{
    inbound::{
        components::{
            accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger},
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{
                Swipeable, config::SwipeConfig, direction::Direction, state::SwipeState,
            },
        },
        screens::deck::card::filter::{
            artist::Artist, combat::Combat, config::Config, deck_cards::DeckCards,
            flavor_text::FlavorText, mana::Mana, name::Name, oracle_text::OracleText,
            rarity::Rarity, set::Set, sort::Sort, types::Types,
        },
    },
    outbound::client::{
        ZwipeClient,
        deck::get_deck::ClientGetDeck,
        deck_card::{
            create_deck_card::ClientCreateDeckCard, delete_deck_card::ClientDeleteDeckCard,
        },
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{
            Card,
            scryfall_data::image_uris::ImageUris,
            search_card::{
                card_filter::{builder::CardFilterBuilder, order_by_option::OrderByOption},
                filter_cards::{FilterCards, SortCards},
            },
        },
    },
    inbound::http::handlers::deck_card::create_deck_card::HttpCreateDeckCard,
};

/// Local undo action for the remove screen.
#[derive(Clone)]
enum RemoveAction {
    Skip,
    Removed(Box<Card>),
}

#[component]
pub fn Remove(deck_id: Uuid) -> Element {
    let navigator = use_navigator();

    let mut filter_builder: Signal<CardFilterBuilder> = use_context();

    let mut is_animating = use_signal(|| false);
    let mut animation_direction = use_signal(|| Direction::Left);

    // Local undo stack
    let mut action_history: Signal<Vec<RemoveAction>> = use_signal(Vec::new);

    // Filter overlay state
    let mut filters_overlay_open = use_signal(|| false);

    // Incrementing this re-runs the filter effect
    let mut filter_reset_counter: Signal<u32> = use_signal(|| 0);
    use_context_provider(|| filter_reset_counter);

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    // Source of truth — all cards in the deck
    let mut deck_cards: Signal<Vec<Card>> = use_signal(Vec::new);
    use_context_provider(|| DeckCards(deck_cards));
    // What the swipe UI iterates over (may be a filtered subset)
    let mut displayed_cards: Signal<Vec<Card>> = use_signal(Vec::new);
    // Guards filter effect from running before the deck has loaded
    let mut deck_loaded: Signal<bool> = use_signal(|| false);

    let mut current_index = use_signal(|| 0_usize);

    let swipe_state = use_signal(SwipeState::new);
    let swipe_config = SwipeConfig::new(
        vec![Direction::Left, Direction::Right, Direction::Down],
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
                    let cards: Vec<Card> = deck.entries.into_iter().map(|e| e.card).collect();
                    deck_cards.set(cards.clone());
                    displayed_cards.set(cards);
                    deck_loaded.set(true);
                }
                Err(e) => {
                    tracing::warn!("deck load failed: {e}");
                    toast.error(e.to_string(), ToastOptions::default());
                }
            }
        });
    });

    // Effect 2 — filter (reads `filter_reset_counter` reactively; peeks `deck_cards`)
    use_effect(move || {
        let _ = filter_reset_counter();

        if !*deck_loaded.peek() {
            return;
        }

        let all_cards = deck_cards.peek().clone();
        let builder = filter_builder.peek().clone();

        let mut filtered = if builder.is_empty() {
            all_cards
        } else {
            let mut b = builder.clone();
            b.set_limit(10_000);
            b.set_offset(0);
            match b.build() {
                Ok(filter) => all_cards.filter_by(&filter),
                Err(_) => deck_cards.peek().clone(),
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

    // Called at animation end to physically remove the card from both vecs.
    let mut remove_current_card = move || {
        let idx = current_index();
        let card_id = displayed_cards()
            .get(idx)
            .map(|c| c.card_profile.scryfall_data_id);
        if let Some(id) = card_id {
            deck_cards
                .write()
                .retain(|c| c.card_profile.scryfall_data_id != id);
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
            RemoveAction::Skip => {
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
            RemoveAction::Removed(card) => {
                // Re-insert into both vecs so the card reappears
                let card = *card;
                let idx = current_index();
                deck_cards.write().push(card.clone());
                displayed_cards.write().insert(idx, card.clone());
                // current_index is unchanged — it now points to the restored card

                // Restore on the backend
                session.upkeep(client);
                let Some(session) = session() else {
                    toast.error("session expired".to_string(), ToastOptions::default());
                    return;
                };

                let request = HttpCreateDeckCard::new(&card.scryfall_data.id.to_string(), 1);

                spawn(async move {
                    match client().create_deck_card(deck_id, &request, &session).await {
                        Ok(_) => {
                            toast.success(
                                "undid remove".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
                            tracing::warn!("undo remove (create deck card) failed: {e}");
                            toast.error(format!("failed to undo: {}", e), ToastOptions::default());
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

                div { class: "form-container",
                    if let Some(card) = current_card() {
                        if let Some(ImageUris { large: Some(image_url), .. }) = &card.scryfall_data.image_uris {
                            Swipeable {
                                state: swipe_state,
                                config: swipe_config,
                                on_swipe_left: move |_| {
                                    action_history.write().push(RemoveAction::Skip);
                                    toast.info(
                                        "skipped".to_string(),
                                        ToastOptions::default().duration(Duration::from_millis(1500)),
                                    );
                                    is_animating.set(true);
                                    animation_direction.set(Direction::Left);
                                },
                                on_swipe_right: move |_| {
                                    let Some(card) = current_card() else { return; };
                                    action_history.write().push(RemoveAction::Removed(Box::new(card)));
                                    delete_card_from_deck();
                                    toast.success(
                                        "removed from deck".to_string(),
                                        ToastOptions::default().duration(Duration::from_millis(1500)),
                                    );
                                    is_animating.set(true);
                                    animation_direction.set(Direction::Right);
                                },
                                on_swipe_up: move |_| {},
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
                                        if animation_direction() == Direction::Right {
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

                        div { class: "card-info",
                            if card.scryfall_data.prices.usd.is_some()
                                || card.scryfall_data.prices.eur.is_some()
                                || card.scryfall_data.prices.tix.is_some()
                            {
                                {
                                    let mut display = String::from("prices:");
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
                            span { "released: {card.scryfall_data.released_at}" }
                            if let Some(artist) = card.scryfall_data.artist && !artist.is_empty() {
                                span { "artist: {artist.to_lowercase()}" }
                            }
                        }
                    } else {
                        div { class: "card-shape flex-center",
                            "no cards"
                        }
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
                            "back to start".to_string(),
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

            // Modal backdrop
            div {
                class: if filters_overlay_open() { "modal-backdrop show" } else { "modal-backdrop" },
                onclick: move |_| filters_overlay_open.set(false),
            }

            // Bottom sheet
            div {
                class: if filters_overlay_open() { "bottom-sheet show" } else { "bottom-sheet" },

                div { class: "modal-header",
                    button {
                        class: "btn btn-sm",
                        onclick: move |_| {
                            filter_reset_counter.set(filter_reset_counter() + 1);
                            filters_overlay_open.set(false);
                        },
                        "apply"
                    }
                }

                div { class: "modal-content",
                    Accordion {
                        key: "{filter_reset_counter()}",
                        id: "filter-accordion",
                        allow_multiple_open: false,
                        collapsible: true,

                        AccordionItem { index: 1,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(1)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "name" }
                            AccordionContent { Name {} }
                        }
                        AccordionItem { index: 2,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(2)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "oracle text" }
                            AccordionContent { OracleText {} }
                        }
                        AccordionItem { index: 3,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(3)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "types" }
                            AccordionContent { Types {} }
                        }
                        AccordionItem { index: 4,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(4)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "mana" }
                            AccordionContent { Mana {} }
                        }
                        AccordionItem { index: 5,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(5)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "combat" }
                            AccordionContent { Combat {} }
                        }
                        AccordionItem { index: 6,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(6)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "flavor text" }
                            AccordionContent { FlavorText {} }
                        }
                        AccordionItem { index: 7,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(7)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "artist" }
                            AccordionContent { Artist {} }
                        }
                        AccordionItem { index: 8,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(8)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "rarity" }
                            AccordionContent { Rarity {} }
                        }
                        AccordionItem { index: 9,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(9)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "set" }
                            AccordionContent { Set {} }
                        }
                        AccordionItem { index: 10,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(10)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "sort" }
                            AccordionContent { Sort {} }
                        }
                        AccordionItem { index: 11,
                            on_change: move |is_open| {
                                if is_open { let _ = document::eval("setTimeout(() => { const el = document.querySelector('#filter-accordion .accordion-item:nth-child(11)'); if (el) el.scrollIntoView({ behavior: 'smooth', block: 'start' }); }, 50)"); }
                            },
                            AccordionTrigger { "config" }
                            AccordionContent { Config {} }
                        }
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn btn-sm",
                        onclick: move |_| {
                            filter_builder.write().clear();
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
            }
        }
    }
}

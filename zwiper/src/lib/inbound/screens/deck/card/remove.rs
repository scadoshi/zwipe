use crate::{
    inbound::{
        components::{
            accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger},
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{
                config::SwipeConfig, direction::Direction, state::SwipeState, Swipeable,
            },
        },
        screens::deck::card::filter::{
            combat::Combat, config::Config, mana::Mana, rarity::Rarity, set::Set, sort::Sort,
            text::Text, types::Types,
        },
    },
    outbound::client::{
        deck::get_deck::ClientGetDeck,
        deck_card::{
            create_deck_card::ClientCreateDeckCard, delete_deck_card::ClientDeleteDeckCard,
        },
        ZwipeClient,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{
            scryfall_data::image_uris::ImageUris,
            search_card::{
                card_filter::builder::CardFilterBuilder, filter_cards::FilterCards,
            },
            Card,
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

    let mut remove_card_error = use_signal(|| None::<String>);

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
        let idx = current_index();
        displayed_cards().get(idx).cloned()
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
                    let cards = deck.cards;
                    deck_cards.set(cards.clone());
                    displayed_cards.set(cards);
                    deck_loaded.set(true);
                }
                Err(e) => {
                    remove_card_error.set(Some(e.to_string()));
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

        let filtered = if builder.is_empty() {
            all_cards
        } else {
            let mut b = builder;
            b.set_limit(10_000);
            b.set_offset(0);
            match b.build() {
                Ok(filter) => all_cards.filter_by(&filter),
                Err(_) => deck_cards.peek().clone(),
            }
        };

        displayed_cards.set(filtered);
        current_index.set(0);
    });

    let mut delete_card_from_deck = move || {
        let Some(card) = current_card() else {
            return;
        };

        session.upkeep(client);
        let Some(session) = session() else {
            remove_card_error.set(Some("session expired".to_string()));
            return;
        };

        let scryfall_data_id = card.scryfall_data.id;

        spawn(async move {
            match client()
                .delete_deck_card(deck_id, scryfall_data_id, &session)
                .await
            {
                Ok(_) => {
                    remove_card_error.set(None);
                }
                Err(e) => {
                    remove_card_error.set(Some(e.to_string()));
                }
            }
        });
    };

    // Called at animation end to physically remove the card from both vecs.
    let mut remove_current_card = move || {
        let idx = current_index();
        let card_id = displayed_cards()
            .get(idx)
            .map(|c| c.card_profile.id);
        if let Some(id) = card_id {
            deck_cards.write().retain(|c| c.card_profile.id != id);
            if idx < displayed_cards.read().len() {
                displayed_cards.write().remove(idx);
            }
        }
        // current_index is unchanged — the next card slides into position
    };

    let mut undo_last_action = move || {
        let Some(action) = action_history.write().pop() else {
            toast.info(
                "nothing to undo".to_string(),
                ToastOptions::default(),
            );
            return;
        };

        match action {
            RemoveAction::Skip => {
                if current_index() == 0 {
                    toast.warning(
                        "no previous card".to_string(),
                        ToastOptions::default(),
                    );
                    action_history.write().push(RemoveAction::Skip);
                    return;
                }
                current_index.set(current_index() - 1);
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

                let request =
                    HttpCreateDeckCard::new(&card.scryfall_data.id.to_string(), 1);

                spawn(async move {
                    match client().create_deck_card(deck_id, &request, &session).await {
                        Ok(_) => {
                            toast.success(
                                "undid remove".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                        Err(e) => {
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

    let mut clear_filters = move || {
        filter_builder.write().clear();
    };

    rsx! {
        Bouncer {
            div { class: "page-header",
                h2 { "remove deck card" }
            }

            div { class: "sticky top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center; padding-top: 4rem;",

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
                                    toast.warning(
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
                                            current_index.set(current_index() + 1);
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
                                span { "artist: {artist}" }
                            }
                        }
                    } else {
                        div { class: "card-shape flex-center",
                            "no cards"
                        }
                    }

                    if let Some(err) = remove_card_error() {
                        div { class: "message-error", "{err}" }
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
                    "filters"
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
                            if filter_builder.read().is_empty() {
                                toast.warning(
                                    "try adding a filter".to_string(),
                                    ToastOptions::default().duration(Duration::from_millis(1500)),
                                );
                            } else {
                                filter_reset_counter.set(filter_reset_counter() + 1);
                            }
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
                        AccordionItem { index: 8,
                            AccordionTrigger { "config" }
                            AccordionContent { Config {} }
                        }
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn btn-sm",
                        onclick: move |_| clear_filters(),
                        "clear"
                    }
                }
            }
        }
    }
}

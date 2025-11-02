pub mod filter;

use crate::{
    inbound::ui::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{
                config::SwipeConfig, direction::Direction as Dir, screen_offset::ScreenOffset,
                state::SwipeState, Swipeable,
            },
            screens::app::deck_card::add::filter::Filter,
        },
        router::Router,
    },
    outbound::client::{
        deck::get_deck::ClientGetDeck, deck_card::create_deck_card::ClientCreateDeckCard,
        ZwipeClient,
    },
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{scryfall_data::image_uris::ImageUris, search_card::SearchCards, Card},
        deck::models::deck::Deck,
    },
    inbound::http::{handlers::deck_card::create_deck_card::HttpCreateDeckCard, ApiError},
};

#[component]
pub fn AddDeckCard(deck_id: Uuid) -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Down],
        submission_swipe: None,
        from_main_screen: ScreenOffset::origin(),
    };

    // let card_swipe_state = use_signal(|| SwipeState::new());

    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut add_card_error = use_signal(|| None::<String>);

    let mut add_card = move |card: &Card| {
        session.upkeep(client);
        let Some(sesh) = session() else {
            add_card_error.set(Some("session expired".to_string()));
            return;
        };

        let request = HttpCreateDeckCard::new(&card.card_profile.id.to_string(), 1);

        spawn(async move {
            match client().create_deck_card(&deck_id, &request, &sesh).await {
                Ok(_) => add_card_error.set(None),
                Err(e) => add_card_error.set(Some(e.to_string())),
            }
        });
    };

    let cards: Signal<Vec<Card>> = use_signal(|| Vec::new());

    let card_filter = use_signal(|| SearchCards::default());

    rsx! {
        Bouncer {
            Filter { swipe_state, deck_id, card_filter, cards }

            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "form-container",

                    if cards().is_empty() {
                        p { class : "up-arrow", "↑" }
                        p { "try adding some filters" }
                    } else {
                        if let Some(card) = cards().iter().next() {
                            if let Some(ImageUris { large: Some(image_url), ..}) = &card.scryfall_data.image_uris {
                                img {
                                    src: "{image_url}",
                                    alt: "{card.scryfall_data.name}",
                                    class: "card-image"
                                }
                            }
                        }
                    }

                    if let Some(add_card_error) = add_card_error() {
                        div { class : "error", "{add_card_error}"}
                    }

                    button {
                        onclick: move |_| {
                            navigator.push(Router::EditDeckProfile { deck_id });
                        },
                        "back"
                    }
                }
            }

        }
    }
}

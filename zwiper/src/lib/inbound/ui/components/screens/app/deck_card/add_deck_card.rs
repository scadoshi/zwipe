pub mod card_filter;
use crate::{
    inbound::ui::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        },
        router::Router,
    },
    outbound::client::{deck_card::create_deck_card::ClientCreateDeckCard, ZwipeClient},
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{scryfall_data::image_uris::ImageUris, search_card::SearchCards, Card},
    },
    inbound::http::handlers::deck_card::create_deck_card::HttpCreateDeckCard,
};

#[component]
pub fn AddDeckCard(
    deck_id: Uuid,
    card_filter: Signal<SearchCards>,
    cards: Signal<Vec<Card>>,
) -> Element {
    tracing::debug!("{} cards found", { cards.len() });

    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

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

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                h2 { class: "text-center mb-2 font-light tracking-wider", "add deck card" }

                div { class : "form-container",

                    if !cards().is_empty() {
                        if let Some(card) = cards().iter().next() {
                            if let Some(ImageUris { large: Some(image_url), ..}) = &card.scryfall_data.image_uris {
                                img {
                                    src: "{image_url}",
                                    alt: "{card.scryfall_data.name}",
                                    class: "card-image"
                                }
                            }
                        }
                    } else {
                        div { class : "card-shape",
                            "no cards yet"
                        }
                    }

                    button { class : "btn",
                        onclick : move |_| {
                            navigator.push(Router::AddDeckCardFilter { card_filter, cards, deck_id } );
                        },
                        "adjust card filters"
                    }

                    if let Some(add_card_error) = add_card_error() {
                        div { class : "error", "{add_card_error}"}
                    }

                    button { class : "btn",
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

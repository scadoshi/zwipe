use crate::{
    inbound::ui::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
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
        card::models::{scryfall_data::image_uris::ImageUris, search_card::SearchCards, Card},
    },
    inbound::http::handlers::deck_card::create_deck_card::HttpCreateDeckCard,
};

#[component]
pub fn Add(deck_id: Uuid) -> Element {
    let filter: Signal<SearchCards> = use_context();
    let mut cards: Signal<Vec<Card>> = use_context();

    tracing::debug!("{} cards found", { cards.len() });

    let swipe_state = use_signal(SwipeState::new);
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut add_card_error = use_signal(|| None::<String>);
    let mut search_error = use_signal(|| None::<String>);

    let _add_card = move |card: &Card| {
        session.upkeep(client);
        let Some(sesh) = session() else {
            add_card_error.set(Some("session expired".to_string()));
            return;
        };

        let request = HttpCreateDeckCard::new(&card.card_profile.id.to_string(), 1);

        spawn(async move {
            match client().create_deck_card(deck_id, &request, &sesh).await {
                Ok(_) => add_card_error.set(None),
                Err(e) => add_card_error.set(Some(e.to_string())),
            }
        });
    };

    use_effect(move || {
        if filter.read().is_blank() {
            return;
        }

        session.upkeep(client);
        let Some(sesh) = session() else {
            search_error.set(Some("session expired".to_string()));
            return;
        };

        let filter_clone = filter.read().clone();
        spawn(async move {
            match client().search_cards(&filter_clone, &sesh).await {
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
                }
                Err(e) => search_error.set(Some(e.to_string())),
            }
        });
    });

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                h2 { class: "text-center mb-2 font-light tracking-wider", "add deck card" }

                div { class : "form-container",
                    if !cards.read().is_empty() {
                        if let Some(card) = cards.read().iter().next() {
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

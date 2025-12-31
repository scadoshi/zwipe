use crate::{
    inbound::components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
    outbound::client::{deck::get_deck::ClientGetDeck, ZwipeClient},
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{search_card::card_filter::builder::CardFilterBuilder, Card},
        deck::models::deck::Deck,
    },
    inbound::http::ApiError,
};

#[component]
pub fn Remove(deck_id: Uuid) -> Element {
    let _filter_builder: Signal<CardFilterBuilder> = use_context();
    let _cards: Signal<Vec<Card>> = use_context();

    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let _deck_cards: Resource<Result<Deck, ApiError>> = use_resource(move || async move {
        session.upkeep(client);
        let Some(sesh) = session() else {
            return Err(ApiError::Unauthorized("session expired".to_string()));
        };

        client().get_deck(deck_id, &sesh).await
    });

    rsx! {
        Bouncer {
            div { class: "fixed top-0 left-0 h-screen flex flex-col items-center overflow-y-auto",
                style: "width: 100vw; justify-content: center;",
                div { class : "form-container",
                        p { "still building" }
                    button { class : "btn",
                        onclick : move |_| {
                            navigator.go_back();
                        },
                        "back"
                    }
                }
            }
        }
    }
}

use crate::{
    inbound::ui::components::{
        auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
    },
    outbound::client::{deck::get_deck::ClientGetDeck, ZwipeClient},
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe::{
    domain::{
        auth::models::session::Session,
        card::models::{search_card::SearchCards, Card},
        deck::models::deck::Deck,
    },
    inbound::http::ApiError,
};

#[component]
pub fn Remove(deck_id: Uuid) -> Element {
    let _filter: Signal<SearchCards> = use_context();
    let _cards: Signal<Vec<Card>> = use_context();

    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let _deck_cards: Resource<Result<Deck, ApiError>> = use_resource(move || async move {
        session.upkeep(client);
        let Some(sesh) = session() else {
            return Err(ApiError::Unauthorized("session expired".to_string()));
        };

        client().get_deck(&deck_id, &sesh).await
    });

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
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

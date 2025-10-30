pub mod filter;

use crate::{
    inbound::ui::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
        },
        router::Router,
    },
    outbound::client::{deck::get_deck::ClientGetDeck, ZwipeClient},
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe::{
    domain::{auth::models::session::Session, deck::models::deck::Deck},
    inbound::http::ApiError,
};

#[component]
pub fn RemoveDeckCard(deck_id: Uuid) -> Element {
    let swipe_state = use_signal(|| SwipeState::new());
    let swipe_config = SwipeConfig::blank();

    let navigator = use_navigator();

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let deck_cards: Resource<Result<Deck, ApiError>> = use_resource(move || async move {
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
                    p { "click to go back" }
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

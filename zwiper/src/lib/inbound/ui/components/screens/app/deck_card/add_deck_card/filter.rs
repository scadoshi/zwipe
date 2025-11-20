pub mod mana;
pub mod printing;
pub mod stats;
pub mod text;
pub mod types;

use crate::{
    inbound::ui::{
        components::{
            auth::{bouncer::Bouncer, session_upkeep::Upkeep},
            interactions::swipe::{config::SwipeConfig, state::SwipeState, Swipeable},
            screens::app::deck_card::add_deck_card::filter::{text::Text, types::Types},
        },
        router::Router,
    },
    outbound::client::{card::search_cards::ClientSearchCards, ZwipeClient},
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe::domain::{
    auth::models::session::Session,
    card::models::{search_card::SearchCards, Card},
};

#[component]
pub fn Filter(filter: Signal<SearchCards>, cards: Signal<Vec<Card>>, deck_id: Uuid) -> Element {
    let swipe_config = SwipeConfig::blank();
    let swipe_state = use_signal(|| SwipeState::new());
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let navigator = use_navigator();
    let mut search_error = use_signal(|| None::<String>);

    let mut should_go_back = use_signal(|| false);
    let mut attempt_search_cards = move || {
        if filter.read().is_blank() {
            search_error.set(Some("must search something".to_string()));
            return;
        }

        session.upkeep(client);
        let Some(sesh) = session() else {
            search_error.set(Some("session expired".to_string()));
            return;
        };

        spawn(async move {
            match client().search_cards(&filter.read(), &sesh).await {
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
                    should_go_back.set(true);
                }
                Err(e) => search_error.set(Some(e.to_string())),
            }
        });
    };

    use_effect(move || {
        if should_go_back() {
            tracing::debug!("made it here");
            navigator.go_back();
            // navigator.push(Router::AddDeckCard {
            //     deck_id,
            //     filter,
            //     cards,
            // });
        }
    });

    rsx! {
        Bouncer {
            Swipeable { state: swipe_state, config: swipe_config,
                div { class : "container-sm",
                    h2 { class: "text-center mb-2 font-light tracking-wider", "card filters" }

                    form { class : "flex-col text-center",
                        Text { filter }
                        Types { filter }
                        button { class : "btn",
                            onclick : move |_| attempt_search_cards(),
                            "search"
                        }
                        if let Some(search_error) = search_error() {
                            div { class : "message-error", "{search_error}" }
                        }
                        button { class : "btn",
                            onclick : move |_| {
                                navigator.push(Router::AddDeckCard { deck_id, filter, cards });
                            },
                            "back"
                        }
                    }
                }
            }
        }
    }
}

use crate::{
    inbound::ui::components::{
        auth::session_upkeep::Upkeep,
        interactions::swipe::{
            config::SwipeConfig,
            direction::Direction as Dir,
            screen_offset::{ScreenOffset, ScreenOffsetMethods},
            state::SwipeState,
            Swipeable,
        },
        success_messages::random_success_message,
    },
    outbound::client::{card::search_cards::ClientSearchCards, ZwipeClient},
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe::{
    domain::{auth::models::session::Session, card::models::Card},
    inbound::http::handlers::card::search_card::HttpSearchCards,
};

#[component]
pub fn Filter(
    swipe_state: Signal<SwipeState>,
    deck_id: Uuid,
    card_filter: Signal<HttpSearchCards>,
    cards: Signal<Vec<Card>>,
) -> Element {
    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Up],
        submission_swipe: None,
        from_main_screen: ScreenOffset::up(),
    };

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut search_error = use_signal(|| None::<String>);
    let mut success_message = use_signal(|| None::<String>);

    let mut attempt_search_cards = move || {
        success_message.set(None);

        if card_filter.read().is_blank() {
            search_error.set(Some("must search something".to_string()));
            return;
        }

        session.upkeep(client);
        let Some(sesh) = session() else {
            search_error.set(Some("session expired".to_string()));
            return;
        };

        spawn(async move {
            match client().search_cards(&card_filter.read(), &sesh).await {
                Ok(cards_from_search) => {
                    search_error.set(None);
                    success_message.set(Some(random_success_message()));
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
                    )
                }
                Err(e) => search_error.set(Some(e.to_string())),
            }
        });
    };

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class : "form-container",

                h2 { "card filters" }

                form { class : "form-group",

                    label { r#for : "name-input", "name" }
                    input { class : "form-input",
                        id : "name-input",
                        placeholder : "name",
                        value : if let Some(name) = card_filter.read().name.as_deref() {
                            name
                        } else { "" },
                        r#type : "text",
                        autocapitalize : "none",
                        spellcheck : "false",
                        oninput : move |event| {
                            card_filter.write().name = Some(event.value());
                            if card_filter.read().name == Some("".to_string()) {
                                card_filter.write().name = None;
                            }
                        }
                    }

                    button { class : "search-cards-button",
                        onclick : move |_| attempt_search_cards(),
                        "search"
                    }

                    if let Some(search_error) = search_error() {
                        div { class : "error", "{search_error}" }
                    }

                    if let Some(success_message) = success_message() {
                        div { class: "success-message", "{success_message}" }
                    }
                }
            }
        }
    }
}

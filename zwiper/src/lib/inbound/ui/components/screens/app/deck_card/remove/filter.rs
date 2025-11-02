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
pub fn Filter(
    swipe_state: Signal<SwipeState>,
    deck_id: Uuid,
    card_filter: Signal<SearchCards>,
    cards: Signal<Vec<Card>>,
) -> Element {
    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Up],
        submission_swipe: None,
        from_main_screen: ScreenOffset::up(),
    };

    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut search_error = use_signal(|| String::new());

    let mut attempt_search_cards = move || {
        session.upkeep(client);
        let Some(sesh) = session() else {
            search_error.set("session expired".to_string());
            return;
        };

        spawn(async move {
            match client().search_cards(&card_filter.read(), &sesh).await {
                Ok(cards_from_search) => cards.set(cards_from_search),
                Err(e) => search_error.set(e.to_string()),
            }
        });
    };

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class : "form-container",

                h2 { "card filters" }

                form { class : "form-group",

                    label { r#for : "name-input", "name input" }
                    input { class : "form-input",
                        id : "name-input",
                        placeholder : "name",
                        value : if let Some(name) = card_filter.read().name_contains.as_deref() {
                            name
                        } else { "" },
                        r#type : "text",
                        autocapitalize : "none",
                        spellcheck : "false",
                        oninput : move |event| {
                            card_filter.write().name_contains = Some(event.value());
                        }
                    }

                    button { class : "search-cards-button",
                        onclick : move |_| attempt_search_cards(),
                    }
                }
            }
        }
    }
}

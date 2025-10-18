use crate::{
    inbound::ui::components::interactions::swipe::{
        config::SwipeConfig,
        direction::Direction as Dir,
        screen_offset::{ScreenOffset, ScreenOffsetMethods},
        state::SwipeState,
        Swipeable,
    },
    outbound::client::{
        auth::{session::ActiveSession, AuthClient},
        deck::get_deck_profiles::{GetDeckProfilesError, GetDecks},
    },
};
use dioxus::prelude::*;
use zwipe::domain::{
    auth::models::session::Session, deck::models::deck::deck_profile::DeckProfile,
};

#[component]
pub fn Decks(swipe_state: Signal<SwipeState>) -> Element {
    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Down],
        submission_swipe: None,
        from_main_screen: ScreenOffset::down(),
    };

    let auth_client: Signal<AuthClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let decks: Resource<Result<Vec<DeckProfile>, GetDeckProfilesError>> =
        use_resource(move || async move {
            let Some(current) = session.read().clone() else {
                return Err(GetDeckProfilesError::SessionExpired);
            };

            let Some(active) = auth_client
                .read()
                .infallible_get_active_session(&current)
                .await
            else {
                return Err(GetDeckProfilesError::SessionExpired);
            };

            let result = auth_client.read().get_deck_profiles(&active).await;

            if active != current {
                session.set(Some(active));
            }

            result
        });

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class : "decks-screen",
                {
                    match decks.read().as_ref() {
                        Some(Ok(decks)) => rsx! {
                            if decks.is_empty() {
                                div { class: "no-decks",
                                    p { "no decks yet" }
                                }
                            } else {
                                for deck_profile in decks {
                                    div { class : "deck-item",
                                        h3 { { deck_profile.name.to_string() } }
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            div { class : "deck-error",
                                p { "failed to load decks: {e}" }
                            }
                        },
                        None => rsx! { div { class : "spinning-card" } },
                    }
                }
            }
        }
    }
}

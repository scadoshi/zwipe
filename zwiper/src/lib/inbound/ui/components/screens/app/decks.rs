use crate::{
    inbound::ui::components::interactions::swipe::{
        config::SwipeConfig, direction::Direction as Dir, state::SwipeState, Swipeable,
    },
    outbound::client::{
        auth::{session::ActiveSession, AuthClient},
        deck::get_deck_profiles::{GetDeckProfilesError, GetDecks},
    },
};
use dioxus::prelude::*;
use std::time::Duration;
use tokio::time::interval;
use zwipe::domain::{
    auth::models::session::Session, deck::models::deck::deck_profile::DeckProfile,
};

#[component]
pub fn Decks(swipe_state: Signal<SwipeState>) -> Element {
    let swipe_config = SwipeConfig {
        navigation_swipes: vec![Dir::Down],
        submission_swipe: None,
        from_main_screen: Some(Dir::Down),
    };

    let auth_client: Signal<AuthClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let check_session = move || async move {
        let Some(s) = session.read().clone() else {
            return;
        };
        session.set(auth_client.read().infallible_get_active_session(&s).await);
    };

    let decks: Resource<Result<Vec<DeckProfile>, GetDeckProfilesError>> =
        use_resource(move || async move {
            check_session().await;
            let Some(s) = session.read().clone() else {
                return Err(GetDeckProfilesError::SessionExpired);
            };
            auth_client.read().get_deck_profiles(&s).await
        });

    use_effect(move || {
        spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                check_session().await;
            }
        });
    });

    rsx! {
        Swipeable { state: swipe_state, config: swipe_config,
            div { class : "decks-screen",
                {
                    match decks.read().as_ref() {
                        Some(Ok(decks)) => rsx! {
                            for deck_profile in decks {
                                div { class : "deck-item",
                                    h3 { { deck_profile.name.to_string() } }
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

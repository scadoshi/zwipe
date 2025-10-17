use crate::{
    inbound::ui::components::interactions::swipe::{
        direction::Direction, onswipe::OnMouse, ontouch::OnTouch, state::SwipeState, VH_GAP,
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
    const MOVE_SWIPES: [Direction; 1] = [Direction::Down];

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
            let mut i = interval(Duration::from_secs(60));
            loop {
                i.tick().await;
                check_session().await;
            }
        });
    });

    rsx! {
        div { class : "swipe-able",

            style : format!(
                "transform: translateY(calc({}px + {}vh + {}vh));
                transition: transform {}s;",
                swipe_state.read().dy().from_start,
                VH_GAP,
                swipe_state.read().position.y * VH_GAP,
                swipe_state.read().transition_seconds
            ),

            ontouchstart : move |e: Event<TouchData>| swipe_state.ontouchstart(e),
            ontouchmove : move |e: Event<TouchData>| swipe_state.ontouchmove(e),
            ontouchend : move |e: Event<TouchData>| { swipe_state.ontouchend(e, &MOVE_SWIPES) },

            onmousedown : move |e: Event<MouseData>| swipe_state.onmousedown(e),
            onmousemove : move |e: Event<MouseData>| swipe_state.onmousemove(e),
            onmouseup : move |e: Event<MouseData>| { swipe_state.onmouseup(e, &MOVE_SWIPES) },

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

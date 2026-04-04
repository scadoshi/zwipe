//! Deck list screen.

use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, session_upkeep::Upkeep},
        router::Router,
    },
    outbound::client::{ZwipeClient, deck::get_deck_profiles::ClientGetDeckList, user::get_user::ClientGetUser},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::auth::models::session::Session;

/// Screen displaying all user's decks with navigation to view/edit.
#[component]
pub fn DeckList() -> Element {
    let navigator = use_navigator();
    let auth_client: Signal<ZwipeClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();
    let toast = use_toast();

    // Refresh user on mount so email_verified_at is current without re-login.
    use_effect(move || {
        let Some(s) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            match auth_client().get_user(&s).await {
                Ok(fresh_user) => {
                    let current = session.peek().clone();
                    if let Some(mut current) = current {
                        current.user = fresh_user;
                        session.set(Some(current));
                    }
                }
                Err(e) => {
                    tracing::warn!("deck list user fetch failed: {e}");
                }
            }
        });
    });

    let mut deck_profiles_resource: Resource<Result<Vec<DeckProfile>, ApiError>> =
        use_resource(move || async move {
            session.upkeep(auth_client);
            let Some(session) = session() else {
                return Err(ApiError::Unauthorized("session expired".to_string()));
            };

            auth_client().get_deck_profiles(&session).await
        });

    // Restart resource on component mount to ensure fresh data
    use_effect(move || {
        deck_profiles_resource.restart();
    });

    use_effect(move || {
        if let Some(Err(e)) = &*deck_profiles_resource.read() {
            toast.error(e.to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
        }
    });

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "decks" }
                }

                div { class: "screen-content",
                div { class: "flex-col",
                    style: "max-width: 40rem; width: 100%; padding: 2rem;",

                    match &*deck_profiles_resource.read() {
                        Some(Ok(deck_profiles)) => {
                            if deck_profiles.is_empty() {
                                rsx! {
                                    div { class: "message-empty",
                                        p { "no decks" }
                                    }
                                }
                            } else {
                                rsx! {
                                    for profile in deck_profiles.iter().map(|x| x.to_owned()) {
                                        div { class : "card row-enter",
                                            key : "{profile.id}",
                                            onclick : move |_| {
                                                navigator.push(Router::ViewDeck {
                                                    deck_id: profile.id,
                                                });
                                            },
                                            h3 { class: "font-light text-base tracking-wide",
                                                { profile.name.to_string() }
                                            }
                                            span { class: "text-muted text-sm",
                                                "{profile.card_count} cards"
                                                if let Some(ref fmt) = profile.format {
                                                    " | {fmt.display_name().to_lowercase()}"
                                                }
                                                if let Some(ref cmd) = profile.commander_name {
                                                    " | {cmd.to_lowercase()}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(_)) => rsx!{
                            div { class : "message-empty",
                                p { "could not load decks" }
                            }
                        },
                        None => rsx! {
                            div { class: "message-empty",
                                div { class: "spinner" }
                            }
                        },
                    }

                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::Home {});
                    },
                    "back"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        // Proactive guard: unverified users are limited to 1 deck.
                        // The backend enforces this too, but we surface it here first.
                        let at_limit = session().is_some_and(|s| {
                            s.user.email_verified_at.is_none()
                                && deck_profiles_resource
                                    .read()
                                    .as_ref()
                                    .and_then(|r| r.as_ref().ok())
                                    .is_some_and(|p| !p.is_empty())
                        });
                        if at_limit {
                            toast.warning(
                                "verify your email to create more than 1 deck".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(4000)),
                            );
                        } else {
                            navigator.push(Router::CreateDeck);
                        }
                    },
                    "create"
                }
            }
            }
        }
    }
}

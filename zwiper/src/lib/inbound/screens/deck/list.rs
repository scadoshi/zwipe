//! Deck list screen.

use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, ensure_session::EnsureFresh},
        components::hint_dialog::{HintBullet, HintBullets, HintDialog, HintKey},
        router::Router,
        screens::deck::components::skeletons::DeckListSkeleton,
    },
    outbound::client::{
        ZwipeClient, deck::get_deck_profiles::ClientGetDeckList, user::get_user::ClientGetUser,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::deck_profile::DeckProfile;

/// Screen displaying all user's decks with navigation to view/edit.
#[component]
pub fn DeckList() -> Element {
    let navigator = use_navigator();
    let auth_client: Signal<ZwipeClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();
    let toast = use_toast();
    let mut decks_hint_open = use_signal(|| false);

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
            let session = session.ensure_fresh(auth_client).await?;

            auth_client().get_deck_profiles(&session).await
        });

    // Restart resource on component mount to ensure fresh data
    use_effect(move || {
        deck_profiles_resource.restart();
    });

    use_effect(move || {
        if let Some(Err(e)) = &*deck_profiles_resource.read() {
            toast.error(
                e.to_user_message(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
    });

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header", style: "position: relative;",
                    h2 { "Decks" }
                    button {
                        class: "util-btn",
                        style: "position: absolute; right: 1rem; top: 50%; transform: translateY(-50%); opacity: 0.55; padding: 0.2rem 0.6rem;",
                        onclick: move |_| decks_hint_open.set(true),
                        "?"
                    }
                }

                // On-demand only: no auto-open, no hints_shown key. Here for
                // screen congruence and as a home for future bulk operations.
                HintDialog {
                    open: decks_hint_open,
                    title: "Your decks",
                    HintBullets {
                        HintBullet { "Scroll through your decks and tap one to open it" }
                        HintBullet {
                            "Tap "
                            HintKey { "Create" }
                            " to start a new deck"
                        }
                    }
                }

                div { class: "screen-content",
                div { class: "flex-col",
                    style: "max-width: 40rem; width: 100%; padding: 2rem;",

                    match &*deck_profiles_resource.read() {
                        Some(Ok(deck_profiles)) => {
                            if deck_profiles.is_empty() {
                                rsx! {
                                    div { class: "message-empty",
                                        p { "No decks" }
                                    }
                                }
                            } else {
                                rsx! {
                                    for profile in {
                                        let mut sorted = deck_profiles.clone();
                                        sorted.sort_by_key(|a| a.name.to_lowercase());
                                        sorted
                                    } {
                                        div { class : "card row-enter",
                                            key : "{profile.id}",
                                            onclick : move |_| {
                                                navigator.push(Router::ViewDeck {
                                                    deck_id: profile.id,
                                                });
                                            },
                                            h3 { class: "font-light text-base tracking-wide",
                                                style: "overflow-wrap: anywhere; word-break: break-word;",
                                                { profile.name.to_string() }
                                            }
                                            hr { class: "card-rule" }
                                            span { class: "text-muted text-sm",
                                                {
                                                    let mut count = profile.card_count;
                                                    if profile.format.as_ref().is_some_and(|f| f.has_commander()) && profile.commander_id.is_some() {
                                                        count += 1;
                                                    }
                                                    if profile.partner_commander_id.is_some() { count += 1; }
                                                    if profile.background_id.is_some() { count += 1; }
                                                    if profile.signature_spell_id.is_some() { count += 1; }
                                                    format!("{count} cards")
                                                }
                                                if let Some(ref fmt) = profile.format {
                                                    " | {fmt.display_name()}"
                                                }
                                                if let Some(ref cmd) = profile.commander_name {
                                                    " | {cmd}"
                                                }
                                                if let Some(ref name) = profile.partner_commander_name {
                                                    " | {name}"
                                                }
                                                if let Some(ref name) = profile.background_name {
                                                    " | {name}"
                                                }
                                                if let Some(ref name) = profile.signature_spell_name {
                                                    " | {name}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(_)) => rsx!{
                            div { class : "message-empty",
                                p { "Could not load decks" }
                            }
                        },
                        None => rsx! {
                            DeckListSkeleton {}
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
                    "Back"
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
                                "Verify your email to create more than 1 deck".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(4000)),
                            );
                        } else {
                            navigator.push(Router::CreateDeck);
                        }
                    },
                    "Create"
                }
            }
            }
        }
    }
}

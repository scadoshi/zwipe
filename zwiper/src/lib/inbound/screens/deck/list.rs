//! Deck list screen.

use crate::{
    inbound::{
        components::{
            auth::{bouncer::Bouncer, ensure_session::EnsureFresh},
            hint_dialog::{HintBullet, HintBullets, HintDialog, HintKey},
            screen_header::ScreenHeader,
        },
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
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::domain::{
    auth::models::session::Session, card::scryfall_data::colors::Color,
    deck::deck_profile::DeckProfile,
};

/// Screen displaying all user's decks with navigation to view/edit.
#[component]
pub fn DeckList() -> Element {
    let navigator = use_navigator();
    let auth_client: Signal<ZwipeClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();
    let toast = use_toast();
    let decks_hint_open = use_signal(|| false);

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
                ScreenHeader { title: "Decks", hint: decks_hint_open }

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
                                            {
                                                let mut count = profile.card_count;
                                                if profile.format.as_ref().is_some_and(|f| f.has_commander()) && profile.commander_id.is_some() {
                                                    count += 1;
                                                }
                                                if profile.partner_commander_id.is_some() { count += 1; }
                                                if profile.background_id.is_some() { count += 1; }
                                                if profile.signature_spell_id.is_some() { count += 1; }
                                                // Out of bounds only when the format defines a size rule the count breaks.
                                                let count_bad = profile.format.as_ref().is_some_and(|f| {
                                                    f.min_cards().is_some_and(|m| count < m as i64)
                                                        || f.max_cards().is_some_and(|m| count > m as i64)
                                                });
                                                rsx! {
                                                    div { class: "deck-list-row",
                                                        h3 { class: "font-light text-base tracking-wide deck-list-name",
                                                            { profile.name.to_string() }
                                                        }
                                                        {
                                                            let present: std::collections::HashSet<Color> = profile
                                                                .color_identity
                                                                .iter()
                                                                .filter_map(|s| Color::try_from(s.as_str()).ok())
                                                                .collect();
                                                            (!present.is_empty())
                                                                .then(|| rsx! {
                                                                    span { class: "deck-list-identity",
                                                                        for color in Color::all().into_iter().filter(|c| present.contains(c)) {
                                                                            i {
                                                                                key: "{color.to_short_name()}",
                                                                                class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
                                                                            }
                                                                        }
                                                                    }
                                                                })
                                                        }
                                                        span {
                                                            class: if count_bad { "stat-chip stat-chip-bad" } else { "stat-chip" },
                                                            "{count} cards"
                                                        }
                                                        if let Some(ref fmt) = profile.format {
                                                            span { class: "stat-chip stat-chip-format", "{fmt.display_name()}" }
                                                        }
                                                        if let Some(pl) = profile.power_level {
                                                            span { class: "stat-chip stat-chip-power", "{pl.display_name()}" }
                                                        }
                                                        if let Some(ref cmd) = profile.commander_name {
                                                            span { class: "stat-chip stat-chip-zone", "{cmd}" }
                                                        }
                                                        if let Some(ref name) = profile.partner_commander_name {
                                                            span { class: "stat-chip stat-chip-zone", "{name}" }
                                                        }
                                                        if let Some(ref name) = profile.background_name {
                                                            span { class: "stat-chip stat-chip-zone", "{name}" }
                                                        }
                                                        if let Some(ref name) = profile.signature_spell_name {
                                                            span { class: "stat-chip stat-chip-zone", "{name}" }
                                                        }
                                                        for tag in profile.tags.iter() {
                                                            span { key: "{tag}", class: "stat-chip stat-chip-tag", "{tag.display_name()}" }
                                                        }
                                                        for tag in profile.other_tags.iter() {
                                                            span { key: "{tag}", class: "stat-chip stat-chip-other", "{tag.display_name()}" }
                                                        }
                                                    }
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

            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| {
                        navigator.push(Router::Home {});
                    },
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
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

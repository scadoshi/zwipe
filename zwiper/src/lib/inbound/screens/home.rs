//! Home/landing page screen.

use crate::inbound::components::logout_dialog::LogoutDialog;
use crate::inbound::router::Router;
use crate::{
    inbound::components::auth::bouncer::Bouncer,
    inbound::components::hint_dialog::{
        HintBullet, HintBullets, HintColored, HintDialog, HintKey, use_one_time_hint,
    },
    outbound::client::{
        ZwipeClient,
        card::search_cards::ClientSearchCards,
        user::{get_user::ClientGetUser, preferences::ClientGetPreferences},
    },
};
use crate::inbound::screens::deck::card::components::image_preview::ImagePreview;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::{
    builder::CardFilterBuilder, order_by_option::OrderByOption,
};
use zwipe_core::domain::logo;
use zwipe_core::domain::user::models::hints::HINT_FIRST_LOGIN;
use zwipe_core::domain::user::models::theme::ThemeConfig;

/// Home screen with navigation to main app features.
#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();

    let client: Signal<ZwipeClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let mut show_logout_dialog = use_signal(|| false);
    let toast = use_toast();

    // First-login welcome: auto-opens once per account.
    let first_login_hint_open = use_one_time_hint(HINT_FIRST_LOGIN);

    let logo = logo::ZWIPE;

    let mut theme_config: Signal<ThemeConfig> = use_context();

    // Tap the card name in the flavor quote to preview that card's image.
    let mut preview_card: Signal<Option<ScryfallData>> = use_signal(|| None);
    let preview_dismissing: Signal<bool> = use_signal(|| false);

    // Greet user on mount.
    use_effect(move || {
        if let Some(session) = session.peek().clone() {
            toast.info(
                format!("Hello, {}!", session.user.username),
                ToastOptions::default().duration(Duration::from_millis(1500)),
            );
        }
    });

    // Refresh user on mount so email_verified_at is current without re-login.
    use_effect(move || {
        let Some(s) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            match client().get_user(&s).await {
                Ok(fresh_user) => {
                    let needs_verification = fresh_user.email_verified_at.is_none();
                    let current = session.peek().clone();
                    if let Some(mut current) = current {
                        current.user = fresh_user;
                        session.set(Some(current));
                    }
                    if needs_verification {
                        toast.warning(
                            "Verify your email to enable password recovery!".to_string(),
                            ToastOptions::default().duration(Duration::from_millis(1500)),
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("home user fetch failed: {e}");
                }
            }
        });
    });

    // Fetch fresh preferences on mount so theme stays current mid-session.
    use_effect(move || {
        let Some(s) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            match client().get_preferences(&s).await {
                Ok(prefs) => {
                    theme_config.set(ThemeConfig::from(&prefs));
                }
                Err(e) => {
                    tracing::warn!("preferences fetch failed: {e}");
                }
            }
        });
    });

    // Fetch a random card with flavor text
    let random_flavor_card = use_resource(move || async move {
        let session = session()?;

        let mut builder = CardFilterBuilder::new();
        builder
            .unset_is_playable()
            .set_has_flavor_text(true)
            .set_order_by(OrderByOption::Random)
            .set_limit(1);

        let Ok(filter) = builder.build() else {
            return None;
        };

        match client().search_cards(&filter, &session).await {
            Ok(cards) => cards.into_iter().next(),
            Err(e) => {
                tracing::warn!("flavor card fetch failed: {e}");
                None
            }
        }
    });

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header", h2 { "Home" } }
                div { class: "screen-content centered",
                div { class : "logo", "{logo}" }

                // Display random flavor text
                div { class: "container-sm text-center flex-col home-flavor content-enter-delayed",
                    match &*random_flavor_card.read() {
                        Some(Some(card)) => {
                            if let Some(flavor_text) = card.scryfall_data.flavor_text.as_ref() {
                                let sd = card.scryfall_data.clone();
                                rsx! {
                                    div { class: "flavor-quote",
                                        "{flavor_text} "
                                        span {
                                            class: "flavor-source flavor-source-link",
                                            onclick: move |_| preview_card.set(Some(sd.clone())),
                                            "{card.scryfall_data.name}"
                                        }
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        }
                        Some(None) => rsx! {},
                        None => rsx! {
                            div { class: "skeleton-flavor",
                                div { class: "skeleton-bar skeleton-flavor-line skeleton-flavor-line-1" }
                                div { class: "skeleton-bar skeleton-flavor-line skeleton-flavor-line-2" }
                                div { class: "skeleton-bar skeleton-flavor-line skeleton-flavor-line-3" }
                                div { class: "skeleton-bar skeleton-flavor-source" }
                            }
                        },
                    }
                }
            }
            HintDialog {
                open: first_login_hint_open,
                title: "Welcome to Zwipe",
                HintBullets {
                    HintBullet {
                        "Tap "
                        HintKey { "Decks" }
                        " to create and build your decks"
                    }
                    HintBullet {
                        "Tap "
                        HintKey { "Profile" }
                        " to manage your account or change your theme"
                    }
                    HintBullet {
                        HintColored { color: "--color-warning", "Verify your email" }
                        " in "
                        HintKey { "Profile" }
                        " to unlock full deck and card limits"
                    }
                }
            }

            ImagePreview { card: preview_card, dismissing: preview_dismissing }

            div { class: "util-bar",
                button { class : "util-btn",
                    onclick : move |_| {
                        navigator.push(Router::DeckList {} );
                    }, "Decks"
                }
                button { class : "util-btn",
                    onclick : move |_| {
                        navigator.push(Router::Profile {} );
                    }, "Profile"
                }
                button {
                    class: "util-btn util-btn-danger",
                    onclick: move |_| show_logout_dialog.set(true),
                    "Log out"
                }
            }

            LogoutDialog { open: show_logout_dialog }
            }
        }
    }
}

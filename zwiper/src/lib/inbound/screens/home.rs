//! Home/landing page screen.

use crate::inbound::components::logout_dialog::LogoutDialog;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::router::Router;
use crate::inbound::screens::deck::card::components::image_preview::ImagePreview;
use crate::{
    inbound::components::auth::bouncer::Bouncer,
    inbound::components::auth::session_upkeep::FlavorCard,
    inbound::components::hint_dialog::{
        HintBullet, HintBullets, HintColored, HintDialog, HintKey, use_one_time_hint,
    },
    outbound::client::{
        ZwipeClient,
        card::search_cards::ClientSearchCards,
        user::{get_user::ClientGetUser, preferences::ClientGetPreferences},
    },
};
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

    // Refresh user on mount so email_verified_at is current without re-login,
    // then greet: verified users get "Hello, username!", unverified users get
    // only the verification nudge (no greeting).
    use_effect(move || {
        let Some(s) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            match client().get_user(&s).await {
                Ok(fresh_user) => {
                    let needs_verification = fresh_user.email_verified_at.is_none();
                    let username = fresh_user.username.clone();
                    let current = session.peek().clone();
                    if let Some(mut current) = current {
                        current.user = fresh_user;
                        session.set(Some(current));
                    }
                    if needs_verification {
                        toast.warning(
                            "Verify your email!".to_string(),
                            ToastOptions::default().duration(Duration::from_millis(1500)),
                        );
                    } else {
                        toast.info(
                            format!("Hello, {username}!"),
                            ToastOptions::default().duration(Duration::from_millis(1500)),
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("home user fetch failed: {e}");
                    // Fall back to the cached session's verification status so a
                    // failed refresh still greets verified users.
                    if let Some(session) = session.peek().clone() {
                        if session.user.email_verified_at.is_some() {
                            toast.info(
                                format!("Hello, {}!", session.user.username),
                                ToastOptions::default().duration(Duration::from_millis(1500)),
                            );
                        }
                    }
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

    // Random flavor card, cached app-wide with a TTL (FlavorCard context).
    // Refetch only when the cache is empty or expired, and overwrite only on
    // success — so rapid navigation reuses the cached quote and a failed /
    // rate-limited refetch keeps the last one instead of blanking.
    let mut flavor: Signal<Option<FlavorCard>> = use_context();
    use_effect(move || {
        // Subscribe to the session so expiry is also re-checked when it
        // refreshes (~60s), not only on mount.
        let current_session = session();
        let needs_refresh = flavor.peek().as_ref().is_none_or(FlavorCard::is_expired);
        if !needs_refresh {
            return;
        }
        let Some(session_val) = current_session else {
            return;
        };
        spawn(async move {
            let mut builder = CardFilterBuilder::new();
            builder
                .unset_is_playable()
                .set_has_flavor_text(true)
                .set_order_by(OrderByOption::Random)
                .set_limit(1);
            let Ok(filter) = builder.build() else {
                return;
            };
            match client().search_cards(&filter, &session_val).await {
                Ok(cards) => {
                    if let Some(card) = cards.into_iter().next() {
                        flavor.set(Some(FlavorCard::new(card)));
                    }
                }
                Err(e) => {
                    // Keep the existing cached card rather than blanking.
                    tracing::warn!("flavor card fetch failed: {e}");
                }
            }
        });
    });

    rsx! {
        Bouncer {
            div { class: "screen",
                ScreenHeader { title: "Home" }
                div { class: "screen-content centered",
                div { class : "logo", "{logo}" }

                // Display random flavor text
                div { class: "container-sm text-center flex-col home-flavor content-enter-delayed",
                    match &*flavor.read() {
                        Some(entry) => {
                            if let Some(flavor_text) = entry.card.scryfall_data.flavor_text.as_ref() {
                                let sd = entry.card.scryfall_data.clone();
                                rsx! {
                                    div { class: "card-header", style: "justify-content: center;",
                                        span { class: "card-title", "Flavor of the hour" }
                                    }
                                    div { class: "flavor-quote", style: "padding: 1rem;",
                                        "{flavor_text} "
                                        span {
                                            class: "flavor-source flavor-source-link",
                                            onclick: move |_| preview_card.set(Some(sd.clone())),
                                            "{entry.card.scryfall_data.name}"
                                        }
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        }
                        None => rsx! {
                            div { class: "skeleton-flavor", style: "padding: 1rem;",
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

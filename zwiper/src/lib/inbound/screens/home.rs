//! Home/landing page screen.

use crate::domain::theme::ThemeConfig;
use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
use crate::inbound::router::Router;
use crate::{
    inbound::components::auth::{bouncer::Bouncer, signal_logout::SignalLogout},
    outbound::client::{
        card::search_cards::ClientSearchCards,
        user::{get_user::ClientGetUser, preferences::ClientGetPreferences},
        ZwipeClient,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe::domain::{
    auth::models::session::Session,
    card::models::search_card::card_filter::{
        builder::CardFilterBuilder, order_by_option::OrderByOption,
    },
};
use zwipe_core::domain::logo;

/// Home screen with navigation to main app features.
#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();

    let client: Signal<ZwipeClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();

    let mut show_logout_dialog = use_signal(|| false);
    let toast = use_toast();

    let logo = logo::ZWIPE;

    let mut theme_config: Signal<ThemeConfig> = use_context();

    // Greet user on mount.
    use_effect(move || {
        if let Some(session) = session.peek().clone() {
            toast.info(
                format!("hello, {}!", session.user.username),
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
                            "verify your email to enable password recovery!".to_string(),
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
                div { class: "screen-content centered",
                div { class : "logo", "{logo}" }

                // Display random flavor text
                div { class: "container-sm text-center flex-col home-flavor content-enter-delayed",
                    match &*random_flavor_card.read() {
                        Some(Some(card)) => {
                            if let Some(flavor_text) = card.scryfall_data.flavor_text.as_ref() {
                                rsx! {
                                    div { class: "flavor-quote",
                                        "{flavor_text} "
                                        span { class: "flavor-source", "[{card.scryfall_data.name}]" }
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        }
                        Some(None) => rsx! {},
                        None => rsx! { div { class: "spinner" } },
                    }
                }
            }
            div { class: "util-bar",
                button { class : "util-btn",
                    onclick : move |_| {
                        navigator.push(Router::DeckList {} );
                    }, "decks"
                }
                button { class : "util-btn",
                    onclick : move |_| {
                        navigator.push(Router::Profile {} );
                    }, "profile"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| show_logout_dialog.set(true),
                    "logout"
                }
            }

            AlertDialogRoot {
                open: show_logout_dialog(),
                on_open_change: move |open| show_logout_dialog.set(open),
                AlertDialogContent {
                    AlertDialogTitle { "logout" }
                    AlertDialogDescription { "are you sure you want to logout?" }
                    AlertDialogActions {
                        AlertDialogCancel {
                            on_click: move |_| show_logout_dialog.set(false),
                            "cancel"
                        }
                        AlertDialogAction {
                            on_click: move |_| session.logout(client),
                            "logout"
                        }
                    }
                }
            }
            }
        }
    }
}

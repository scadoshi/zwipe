//! User profile management screens.
//!
//! Provides screens for viewing and updating user profile information.

/// Change email screen.
pub mod change_email;
/// Change password screen.
pub mod change_password;
/// Change username screen.
pub mod change_username;
/// User preferences screen.
pub mod preferences;

use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, session_upkeep::Upkeep, signal_logout::SignalLogout},
        router::Router,
    },
    outbound::client::{
        auth::resend_verification::ClientResendEmailVerification,
        user::delete_user::ClientDeleteUser,
        user::get_user::ClientGetUser,
        ZwipeClient,
    },
};
use zwipe::inbound::http::handlers::auth::delete_user::HttpDeleteUser;
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe::domain::auth::models::session::Session;

/// User profile screen showing account details and management options.
#[component]
pub fn Profile() -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut show_logout_dialog = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| false);
    let mut delete_countdown = use_signal(|| 5u8);
    let mut delete_password = use_signal(String::new);
    let mut is_deleting = use_signal(|| false);
    let mut is_resending = use_signal(|| false);
    let toast = use_toast();

    // Refresh user on every open so email_verified_at is current without re-login.
    // peek() avoids a reactive subscription — runs once on mount, not on every session update.
    use_effect(move || {
        let Some(s) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            match client().get_user(&s).await {
                Ok(fresh_user) => {
                    let current = session.peek().clone();
                    if let Some(mut current) = current {
                        current.user = fresh_user;
                        session.set(Some(current));
                    }
                }
                Err(e) => {
                    tracing::warn!("profile user fetch failed: {e}");
                }
            }
        });
    });

    let navigator = use_navigator();

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "profile" }
                }

                div { class: "screen-content centered content-enter",
                    if let Some(s) = session().as_ref() {
                        div { class: "profile-list", style: "width: calc(100% - 4rem);",

                            div {
                                class: "profile-row",
                                span { class: "profile-row-label", "username" }
                                div { class: "profile-row-value",
                                    span { { s.user.username.to_string() } }
                                    button {
                                        class: "util-btn",
                                        onclick: move |_| { navigator.push(Router::ChangeUsername {}); },
                                        "change"
                                    }
                                }
                            }

                            div {
                                class: "profile-row",
                                span { class: "profile-row-label", "email" }
                                div { class: "profile-row-value",
                                    span { { s.user.email.to_string() } }
                                    if s.user.email_verified_at.is_some() {
                                        span { class: "badge-verified", "verified" }
                                    } else {
                                        span { class: "badge-unverified", "unverified" }
                                        button {
                                            class: "util-btn",
                                            disabled: is_resending(),
                                            onclick: move |evt| {
                                                evt.stop_propagation();
                                                is_resending.set(true);
                                                spawn(async move {
                                                    session.upkeep(client);
                                                    let Some(s) = session() else {
                                                        toast.error(
                                                            "session expired — please log in again".to_string(),
                                                            ToastOptions::default().duration(Duration::from_millis(5000)),
                                                        );
                                                        is_resending.set(false);
                                                        return;
                                                    };
                                                    match client().resend_verification(&s).await {
                                                        Ok(()) => toast.success(
                                                            "verification email sent".to_string(),
                                                            ToastOptions::default().duration(Duration::from_millis(3000)),
                                                        ),
                                                        Err(e) => toast.error(
                                                            e.to_user_message(),
                                                            ToastOptions::default().duration(Duration::from_millis(5000)),
                                                        ),
                                                    }
                                                    is_resending.set(false);
                                                });
                                            },
                                            if is_resending() { "sending..." } else { "resend" }
                                        }
                                    }
                                    button {
                                        class: "util-btn",
                                        onclick: move |_| { navigator.push(Router::ChangeEmail {}); },
                                        "change"
                                    }
                                }
                            }

                            div {
                                class: "profile-row",
                                span { class: "profile-row-label", "password" }
                                div { class: "profile-row-value",
                                    span { "•••••••" }
                                    button {
                                        class: "util-btn",
                                        onclick: move |_| { navigator.push(Router::ChangePassword {}); },
                                        "change"
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "util-bar",
                    button {
                        class: "util-btn",
                        onclick: move |_| navigator.go_back(),
                        "back"
                    }
                    button {
                        class: "util-btn",
                        onclick: move |_| { navigator.push(Router::Preferences {}); },
                        "preferences"
                    }
                    button {
                        class: "util-btn",
                        onclick: move |_| show_logout_dialog.set(true),
                        "logout"
                    }
                    button {
                        class: "util-btn",
                        onclick: move |_| {
                            delete_password.set(String::new());
                            delete_countdown.set(5);
                            show_delete_dialog.set(true);
                            spawn(async move {
                                for i in (0..5u8).rev() {
                                    tokio::time::sleep(Duration::from_secs(1)).await;
                                    delete_countdown.set(i);
                                }
                            });
                        },
                        "delete account"
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

                AlertDialogRoot {
                    open: show_delete_dialog(),
                    on_open_change: move |open: bool| {
                        if !open { show_delete_dialog.set(false); }
                    },
                    AlertDialogContent {
                        AlertDialogTitle { "delete account" }
                        AlertDialogDescription {
                            "this will permanently delete your account, all decks, and all card data. this cannot be undone."
                        }
                        input {
                            r#type: "password",
                            class: "input",
                            placeholder: "confirm your password",
                            value: "{delete_password}",
                            oninput: move |evt| {
                                delete_password.set(evt.value());
                            },
                        }
                        AlertDialogActions {
                            AlertDialogCancel {
                                on_click: move |_| show_delete_dialog.set(false),
                                "cancel"
                            }
                            button {
                                class: "alert-dialog-action-danger",
                                disabled: delete_countdown() > 0 || is_deleting(),
                                onclick: move |_| {
                                    is_deleting.set(true);
                                    let password = delete_password();
                                    spawn(async move {
                                        session.upkeep(client);
                                        let Some(s) = session() else {
                                            toast.error("session expired — please log in again".to_string(), ToastOptions::default().duration(Duration::from_millis(3000)));
                                            is_deleting.set(false);
                                            return;
                                        };
                                        match client().delete_user(HttpDeleteUser { password }, &s).await {
                                            Ok(()) => {
                                                show_delete_dialog.set(false);
                                                session.logout(client);
                                            }
                                            Err(e) => {
                                                toast.error(e.to_user_message(), ToastOptions::default().duration(Duration::from_millis(3000)));
                                                is_deleting.set(false);
                                            }
                                        }
                                    });
                                },
                                if delete_countdown() > 0 {
                                    "delete ({delete_countdown()})"
                                } else if is_deleting() {
                                    "deleting..."
                                } else {
                                    "delete"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

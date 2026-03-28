//! User profile management screens.
//!
//! Provides screens for viewing and updating user profile information.

/// Change email screen.
pub mod change_email;
/// Change password screen.
pub mod change_password;
/// Change username screen.
pub mod change_username;

use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
use crate::{
    inbound::{
        components::auth::{bouncer::Bouncer, session_upkeep::Upkeep, signal_logout::SignalLogout},
        router::Router,
    },
    outbound::client::{auth::resend_verification::ClientResendEmailVerification, ZwipeClient},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe::domain::auth::models::session::Session;

/// User profile screen showing account details and management options.
#[component]
pub fn Profile() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut show_logout_dialog = use_signal(|| false);
    let mut is_resending = use_signal(|| false);
    let toast = use_toast();

    let navigator = use_navigator();

    rsx! {
        Bouncer {
            div { class: "screen",
                div { class: "page-header",
                    h2 { "profile" }
                }

                div { class: "screen-content centered",
                if let Some(session) = session().as_ref() {
                    div { style: "max-width: 40rem; width: 100%; padding: 0 1rem;",

                        div { class : "flex items-center flex-between mb-2 gap-2",
                            div { class : "flex-1",
                                label { class: "label", "username" }
                                p { class: "text-base font-light mb-1",
                                    { session.user.username.to_string() }
                                }
                            }
                        }

                        div { class : "flex items-center flex-between mb-2 gap-2",
                            div { class : "flex-1",
                                label { class: "label", "email" }
                                p { class: "text-base font-light mb-1",
                                    { session.user.email.to_string() }
                                }
                                if session.user.email_verified_at.is_some() {
                                    span { class: "badge-verified", "verified" }
                                } else {
                                    span { class: "badge-unverified", "unverified" }
                                }
                            }
                        }

                        div { class : "flex items-center flex-between mb-2 gap-2",
                            div { class : "flex-1",
                                label { class: "label", "password" }
                                p { class: "text-base font-light mb-1", "•••••••" }
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
                if session().as_ref().map_or(false, |s| s.user.email_verified_at.is_none()) {
                    button {
                        class: "util-btn",
                        disabled: is_resending(),
                        onclick: move |_| {
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
                                        e.to_string().to_lowercase(),
                                        ToastOptions::default().duration(Duration::from_millis(5000)),
                                    ),
                                }
                                is_resending.set(false);
                            });
                        },
                        if is_resending() { "sending..." } else { "resend verification" }
                    }
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::ChangeUsername {});
                    },
                    "username"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::ChangeEmail {});
                    },
                    "email"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| {
                        navigator.push(Router::ChangePassword {});
                    },
                    "password"
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

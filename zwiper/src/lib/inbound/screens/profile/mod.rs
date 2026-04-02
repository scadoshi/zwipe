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
/// Extracted components for the profile screen.
mod components;

use components::delete_account_dialog::DeleteAccountDialog;
use components::email_verification::EmailVerification;
use components::logout_dialog::LogoutDialog;
use crate::{
    inbound::{
        components::auth::bouncer::Bouncer,
        router::Router,
    },
    outbound::client::{
        user::get_user::ClientGetUser,
        ZwipeClient,
    },
};
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

/// User profile screen showing account details and management options.
#[component]
pub fn Profile() -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut show_logout_dialog = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| false);

    // Refresh user on every open so email_verified_at is current without re-login.
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
                                    EmailVerification {
                                        email: s.user.email.to_string(),
                                        is_verified: s.user.email_verified_at.is_some(),
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
                        onclick: move |_| show_delete_dialog.set(true),
                        "delete account"
                    }
                }

                LogoutDialog { open: show_logout_dialog }
                DeleteAccountDialog { open: show_delete_dialog }
            }
        }
    }
}

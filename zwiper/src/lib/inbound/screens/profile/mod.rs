//! User profile management screen and its edit sheets.
//!
//! Provides the profile overview plus bottom sheets for updating user info.

/// Change email bottom sheet.
pub mod change_email;
/// Change password bottom sheet.
pub mod change_password;
/// Change username bottom sheet.
pub mod change_username;
/// User preferences bottom sheet.
pub mod preferences;
/// Extracted components for the profile screen.
mod components;

use components::delete_account_dialog::DeleteAccountDialog;
use components::email_verification::EmailVerification;
use crate::inbound::components::logout_dialog::LogoutDialog;
use change_email::ChangeEmailSheet;
use change_password::ChangePasswordSheet;
use change_username::ChangeUsernameSheet;
use preferences::PreferencesSheet;
use crate::{
    inbound::{
        components::auth::bouncer::Bouncer,
        components::hint_dialog::{
            HintBullet, HintBullets, HintDialog, HintKey, use_one_time_hint,
        },
    },
    outbound::client::{
        user::get_user::ClientGetUser,
        ZwipeClient,
    },
};
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::hints::HINT_PROFILE;

/// User profile screen showing account details and management options.
#[component]
pub fn Profile() -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();

    let mut show_logout_dialog = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| false);
    let mut preferences_open = use_signal(|| false);
    let mut change_username_open = use_signal(|| false);
    let mut change_email_open = use_signal(|| false);
    let mut change_password_open = use_signal(|| false);

    // Account management hint: auto-opens on this user's first visit, the
    // grayed "?" in the header reopens it on demand.
    let mut profile_hint_open = use_one_time_hint(HINT_PROFILE);

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
                div { class: "page-header", style: "position: relative;",
                    h2 { "Profile" }
                    button {
                        class: "util-btn",
                        style: "position: absolute; right: 1rem; top: 50%; transform: translateY(-50%); opacity: 0.55; padding: 0.2rem 0.6rem;",
                        onclick: move |_| profile_hint_open.set(true),
                        "?"
                    }
                }

                HintDialog {
                    open: profile_hint_open,
                    title: "Your profile",
                    HintBullets {
                        HintBullet {
                            "Tap "
                            HintKey { "Change" }
                            " to update your username, email or password"
                        }
                        HintBullet {
                            "Tap "
                            HintKey { "Preferences" }
                            " to change your theme"
                        }
                        HintBullet {
                            "Tap "
                            HintKey { "Delete account" }
                            " to erase your account for good"
                        }
                    }
                }

                div { class: "screen-content centered content-enter",
                    if let Some(s) = session().as_ref() {
                        div { class: "profile-list", style: "width: calc(100% - 4rem);",

                            div {
                                class: "profile-row",
                                span { class: "profile-row-label", "Username" }
                                div { class: "profile-row-value",
                                    span { { s.user.username.to_string() } }
                                    button {
                                        class: "util-btn",
                                        onclick: move |_| change_username_open.set(true),
                                        "Change"
                                    }
                                }
                            }

                            div {
                                class: "profile-row",
                                span { class: "profile-row-label", "Email" }
                                div { class: "profile-row-value",
                                    EmailVerification {
                                        email: s.user.email.to_string(),
                                        is_verified: s.user.email_verified_at.is_some(),
                                    }
                                    button {
                                        class: "util-btn",
                                        onclick: move |_| change_email_open.set(true),
                                        "Change"
                                    }
                                }
                            }

                            div {
                                class: "profile-row",
                                span { class: "profile-row-label", "Password" }
                                div { class: "profile-row-value",
                                    span { "•••••••" }
                                    button {
                                        class: "util-btn",
                                        onclick: move |_| change_password_open.set(true),
                                        "Change"
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
                        "Back"
                    }
                    button {
                        class: "util-btn",
                        onclick: move |_| preferences_open.set(true),
                        "Preferences"
                    }
                    button {
                        class: "util-btn util-btn-danger",
                        onclick: move |_| show_logout_dialog.set(true),
                        "Log out"
                    }
                    button {
                        class: "util-btn util-btn-danger",
                        onclick: move |_| show_delete_dialog.set(true),
                        "Delete account"
                    }
                }

                LogoutDialog { open: show_logout_dialog }
                DeleteAccountDialog { open: show_delete_dialog }
                PreferencesSheet { open: preferences_open }
                ChangeUsernameSheet { open: change_username_open }
                ChangeEmailSheet { open: change_email_open }
                ChangePasswordSheet { open: change_password_open }
            }
        }
    }
}

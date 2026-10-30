//! Account screen and its edit sheets.
//!
//! Shows username, email and password with bottom sheets to change them,
//! plus the About rows, logging out and deleting the account.

/// Change email bottom sheet.
pub mod change_email;
/// Change password bottom sheet.
pub mod change_password;
/// Change username bottom sheet.
pub mod change_username;
/// Extracted components for the Account screen.
mod components;

use ::components::{
    ActionBar, BottomSheet, Button, ButtonVariant, HintBullet, HintBullets, HintDialog, HintKey,
};
use change_email::ChangeEmailSheet;
use change_password::ChangePasswordSheet;
use change_username::ChangeUsernameSheet;
use components::delete_account_dialog::DeleteAccountDialog;
use components::email_verification::{EmailVerification, PendingEmail, VerificationActions};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::site::WEB_BASE;
use zwipe_core::domain::user::models::hints::HINT_PROFILE;

use crate::inbound::components::hint_dialog::use_one_time_hint;
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::components::logout_dialog::LogoutDialog;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::router::Router;
use crate::outbound::open_url;

/// Client version baked in at compile time, shown at the bottom of the screen
/// so users can report which build they're on.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Account screen showing account details and management options.
#[component]
pub fn Account() -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut show_logout_dialog = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| false);
    let mut show_more_sheet = use_signal(|| false);
    let mut change_username_open = use_signal(|| false);
    let mut change_email_open = use_signal(|| false);
    let mut change_password_open = use_signal(|| false);

    // Account management hint: auto-opens on this user's first visit, the
    // grayed "?" in the header reopens it on demand.
    let account_hint_open = use_one_time_hint(HINT_PROFILE);

    // Refresh user on every open so email_verified_at is current without re-login.
    use_effect(move || {
        let Some(s) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            match client().fetch_user(&s).await {
                Ok(fresh_user) => {
                    let current = session.peek().clone();
                    if let Some(mut current) = current {
                        current.user = fresh_user;
                        session.set(Some(current));
                    }
                }
                Err(e) => {
                    tracing::warn!("account user fetch failed: {e}");
                }
            }
        });
    });

    let navigator = use_navigator();

    rsx! {
            div { class: "screen",
                ScreenHeader { title: "Account", hint: account_hint_open }

                HintDialog {
                    open: account_hint_open,
                    title: "Your account",
                    HintBullets {
                        HintBullet { "Tap a row's " HintKey { color: "--palette-6", "?" } " for what it does" }
                        HintBullet { HintKey { "More" } " deletes your account" }
                    }
                }

                div { class: "screen-content content-enter",
                    if let Some(s) = session().as_ref() {
                        div { class: "settings-sections",

                            div { class: "settings-list",

                                div { class: "settings-list-header",
                                    span { class: "settings-list-title", "Account" }
                                }

                                div {
                                    class: "settings-row",
                                    span { class: "settings-row-label", "Username" }
                                    div { class: "settings-row-value",
                                        span { { s.user.username.to_string() } }
                                        Button {
                                            variant: ButtonVariant::Action,
                                            onclick: move |_| change_username_open.set(true),
                                            "Change"
                                        }
                                    }
                                }

                                div {
                                    class: "settings-row",
                                    span { style: "display:flex;align-items:center;",
                                        span { class: "settings-row-label", "Email" }
                                        InfoButton { topic: HintTopic::Email }
                                    }
                                    div { class: "settings-row-value",
                                        EmailVerification {
                                            email: s.user.email.to_string(),
                                            is_verified: s.user.email_verified_at.is_some(),
                                            on_change: move |()| change_email_open.set(true),
                                        }
                                    }
                                }

                                if let Some(pending) = s.user.pending_email.as_ref() {
                                    div {
                                        class: "settings-row",
                                        span { class: "settings-row-label", "Pending" }
                                        div { class: "settings-row-value",
                                            PendingEmail {
                                                email: pending.to_string(),
                                                expires_at: s.user.pending_email_expires_at,
                                            }
                                        }
                                    }
                                }

                                if s.user.email_verified_at.is_none() {
                                    div {
                                        class: "settings-row",
                                        span { style: "display:flex;align-items:center;",
                                        span { class: "settings-row-label", "Verification" }
                                        InfoButton { topic: HintTopic::Verification }
                                    }
                                        div { class: "settings-row-value",
                                            VerificationActions {}
                                        }
                                    }
                                }

                                div {
                                    class: "settings-row",
                                    span { class: "settings-row-label", "Password" }
                                    div { class: "settings-row-value",
                                        span { "•••••••" }
                                        Button {
                                            variant: ButtonVariant::Action,
                                            onclick: move |_| change_password_open.set(true),
                                            "Change"
                                        }
                                    }
                                }
                            }

                            div { class: "settings-list",

                                div { class: "settings-list-header",
                                    span { class: "settings-list-title", "About" }
                                }

                                div {
                                    class: "settings-row",
                                    span { style: "display:flex;align-items:center;",
                                        span { class: "settings-row-label", "Website" }
                                        InfoButton { topic: HintTopic::Website }
                                    }
                                    div { class: "settings-row-value",
                                        Button {
                                            variant: ButtonVariant::Action,
                                            onclick: move |_| open_url::open(WEB_BASE),
                                            "zwipe.net \u{2197}"
                                        }
                                    }
                                }

                                div {
                                    class: "settings-row",
                                    span { class: "settings-row-label", "Privacy policy" }
                                    div { class: "settings-row-value",
                                        Button {
                                            variant: ButtonVariant::Action,
                                            onclick: move |_| {
                                                navigator.push(Router::PrivacyPolicy {});
                                            },
                                            "View"
                                        }
                                    }
                                }

                                div {
                                    class: "settings-row",
                                    span { style: "display:flex;align-items:center;",
                                        span { class: "settings-row-label", "Version" }
                                        InfoButton { topic: HintTopic::Version }
                                    }
                                    div { class: "settings-row-value",
                                        span { "v{APP_VERSION}" }
                                        Button {
                                            variant: ButtonVariant::Action,
                                            onclick: move |_| {
                                                navigator.push(Router::Changelog {});
                                            },
                                            "Changelog"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                ActionBar {
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| navigator.go_back(),
                        "Back"
                    }
                    Button {
                        variant: ButtonVariant::Action,
                        danger: true,
                        onclick: move |_| show_logout_dialog.set(true),
                        "Log out"
                    }
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| show_more_sheet.set(true),
                        "More"
                    }
                }

                BottomSheet { open: show_more_sheet, title: "More".to_string(),
                    Button {
                        danger: true,
                        onclick: move |_| show_delete_dialog.set(true),
                        "Delete account"
                    }
                }

                LogoutDialog { open: show_logout_dialog }
                DeleteAccountDialog { open: show_delete_dialog }
                ChangeUsernameSheet { open: change_username_open }
                ChangeEmailSheet { open: change_email_open }
                ChangePasswordSheet { open: change_password_open }
            }
    }
}

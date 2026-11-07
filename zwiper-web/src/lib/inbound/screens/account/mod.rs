//! Account: the `Account` list, the change sheets as centered modals, and
//! a Danger zone panel at the bottom with Log out and Delete account. The
//! version, website and privacy policy live on About.

mod change_email;
mod change_password;
mod change_username;
mod delete_account;
mod verification;

use components::{Button, ButtonVariant};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_ui::screens::account::{self as model, MoreItem};

use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::logout_dialog::LogoutDialog;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::screens::account::change_email::ChangeEmailSheet;
use crate::inbound::screens::account::change_password::ChangePasswordSheet;
use crate::inbound::screens::account::change_username::ChangeUsernameSheet;
use crate::inbound::screens::account::delete_account::DeleteAccountDialog;
use crate::inbound::screens::account::verification::{
    EmailValue, PendingEmail, VerificationActions,
};
use crate::inbound::session::logout;
use crate::inbound::shell::ScreenFrame;

/// The account screen. Every visit fetches the user once with the session as
/// it is, so the verification badge is current without logging in again; a
/// failed fetch is only logged.
#[component]
pub fn Account() -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let hint_open = use_one_time_hint(OneTimeHint::Profile);
    let mut logout_open = use_signal(|| false);
    let mut delete_open = use_signal(|| false);
    let mut username_open = use_signal(|| false);
    let mut email_open = use_signal(|| false);
    let mut password_open = use_signal(|| false);
    let log_out = use_callback(move |()| logout::logout(session, client));

    use_hook(move || {
        let Some(current) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            let http = client.peek().clone();
            match http.fetch_user(&current).await {
                Ok(user) => {
                    let latest = session.peek().clone();
                    if let Some(mut latest) = latest {
                        latest.user = user;
                        session.set(Some(latest));
                    }
                }
                Err(e) => tracing::warn!("account user fetch failed: {e}"),
            }
        });
    });

    rsx! {
        ScreenFrame {
            title: "Account",
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div { class: "screen-content content-enter",
                if let Some(s) = session().as_ref() {
                    div { class: "settings-sections web-settings-pair",
                        div { class: "settings-list",
                            div { class: "settings-list-header",
                                span { class: "settings-list-title", "Account" }
                            }
                            div { class: "settings-row",
                                span { class: "settings-row-label", "Username" }
                                div { class: "settings-row-value",
                                    span { "{s.user.username}" }
                                    Button {
                                        variant: ButtonVariant::Action,
                                        onclick: move |_| username_open.set(true),
                                        "Change"
                                    }
                                }
                            }
                            div { class: "settings-row",
                                RowLabel { label: "Email", topic: HintTopic::Email }
                                div { class: "settings-row-value",
                                    EmailValue {
                                        email: s.user.email.to_string(),
                                        is_verified: s.user.email_verified_at.is_some(),
                                        on_change: move |()| email_open.set(true),
                                    }
                                }
                            }
                            if let Some(pending) = s.user.pending_email.as_ref() {
                                div { class: "settings-row",
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
                                div { class: "settings-row",
                                    RowLabel { label: "Verification", topic: HintTopic::Verification }
                                    div { class: "settings-row-value", VerificationActions {} }
                                }
                            }
                            div { class: "settings-row",
                                span { class: "settings-row-label", "Password" }
                                div { class: "settings-row-value",
                                    span { "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}" }
                                    Button {
                                        variant: ButtonVariant::Action,
                                        onclick: move |_| password_open.set(true),
                                        "Change"
                                    }
                                }
                            }
                        }
                        div { class: "settings-list danger-zone",
                            div { class: "settings-list-header",
                                span { class: "settings-list-title danger-zone-title", {model::DANGER_ZONE} }
                            }
                            div { class: "settings-row",
                                span { class: "settings-row-label", {model::LOG_OUT} }
                                div { class: "settings-row-value",
                                    Button {
                                        variant: ButtonVariant::Action,
                                        danger: true,
                                        onclick: move |_| logout_open.set(true),
                                        {model::LOG_OUT}
                                    }
                                }
                            }
                            div { class: "settings-row",
                                span { class: "settings-row-label", {MoreItem::DeleteAccount.label()} }
                                div { class: "settings-row-value",
                                    Button {
                                        variant: ButtonVariant::Action,
                                        danger: true,
                                        onclick: move |_| delete_open.set(true),
                                        {model::DELETE}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::Profile, open: hint_open }
        LogoutDialog { open: logout_open, on_confirm: log_out }
        DeleteAccountDialog { open: delete_open }
        ChangeUsernameSheet { open: username_open }
        ChangeEmailSheet { open: email_open }
        ChangePasswordSheet { open: password_open }
    }
}

/// A row label with its `?`.
#[component]
fn RowLabel(label: String, topic: HintTopic) -> Element {
    rsx! {
        span { style: "display:flex;align-items:center;",
            span { class: "settings-row-label", "{label}" }
            InfoButton { topic }
        }
    }
}

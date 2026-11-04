//! Account screen and its edit sheets.
//!
//! Shows username, email and password with bottom sheets to change them,
//! plus logging out and deleting the account.
//!
//! What the screen knows and does is `zwipe_ui::screens::account`; these
//! files draw its state and perform its effects.

/// Change email bottom sheet.
pub mod change_email;
/// Change password bottom sheet.
pub mod change_password;
/// Change username bottom sheet.
pub mod change_username;
/// Extracted components for the Account screen.
mod components;
/// The model with the handles that perform its effects.
mod driver;

use ::components::{Button, ButtonVariant, HintBullet, HintBullets, HintDialog, HintKey};
use change_email::ChangeEmailSheet;
use change_password::ChangePasswordSheet;
use change_username::ChangeUsernameSheet;
use chrono::Utc;
use components::delete_account_dialog::DeleteAccountDialog;
use components::email_verification::{EmailVerification, PendingEmail, VerificationActions};
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::hints::HINT_PROFILE;
use zwipe_ui::screens::account::{self as model, Action, MoreItem, Row, Sheet};

use self::driver::{Driver, use_driver};
use crate::inbound::components::guide_link;
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::components::logout_dialog::LogoutDialog;
use crate::inbound::components::screen_header::ScreenHeader;

/// Account screen showing account details and management options.
#[component]
pub fn Account() -> Element {
    let driver: Driver = use_driver();
    use_context_provider(|| driver);
    let session: Signal<Option<Session>> = use_context();
    // Offers the hint and refreshes the user once per mount, while the screen
    // first draws, so the hint opens with the screen and the verification is
    // current without signing in again.
    use_hook(move || {
        let user = session.peek().as_ref().map(|s| Box::new(s.user.clone()));
        driver.dispatch(Action::Load(user));
    });
    let account_hint_open = driver.overlays.hint;

    let state = driver.model.read();
    let user = state.user().cloned();
    let rows = state.rows();
    let pending = state.pending(Utc::now());
    let is_verified = state.is_verified();
    drop(state);

    rsx! {
            div { class: "screen",
                ScreenHeader { title: model::TITLE, hint: account_hint_open, back: move |()| driver.dispatch(Action::Back) }

                HintDialog {
                    open: account_hint_open,
                    actions: guide_link::one_time(HINT_PROFILE),
                    title: "Your account",
                    HintBullets {
                        HintBullet { "Tap a row's " HintKey { color: "--palette-6", "?" } " for what it does" }
                        HintBullet { HintKey { "Danger zone" } " at the bottom logs you out or deletes your account" }
                    }
                }

                div { class: "screen-content content-enter",
                    if let Some(user) = user.as_ref() {
                        div { class: "settings-sections",

                            div { class: "settings-list",

                                div { class: "settings-list-header",
                                    span { class: "settings-list-title", {model::ACCOUNT} }
                                }

                                div {
                                    class: "settings-row",
                                    span { class: "settings-row-label", {Row::Username.label()} }
                                    div { class: "settings-row-value",
                                        span { { user.username.to_string() } }
                                        Button {
                                            variant: ButtonVariant::Action,
                                            onclick: move |_| driver.dispatch(Action::OpenSheet(Sheet::Username)),
                                            {model::CHANGE}
                                        }
                                    }
                                }

                                div {
                                    class: "settings-row",
                                    span { style: "display:flex;align-items:center;",
                                        span { class: "settings-row-label", {Row::Email.label()} }
                                        InfoButton { topic: HintTopic::Email }
                                    }
                                    div { class: "settings-row-value",
                                        EmailVerification {
                                            email: user.email.to_string(),
                                            is_verified,
                                            on_change: move |()| driver.dispatch(Action::OpenSheet(Sheet::Email)),
                                        }
                                    }
                                }

                                if let Some(pending) = pending {
                                    div {
                                        class: "settings-row",
                                        span { class: "settings-row-label", {Row::Pending.label()} }
                                        div { class: "settings-row-value",
                                            PendingEmail {
                                                email: pending.email,
                                                note: pending.note,
                                            }
                                        }
                                    }
                                }

                                if rows.contains(&Row::Verification) {
                                    div {
                                        class: "settings-row",
                                        span { style: "display:flex;align-items:center;",
                                        span { class: "settings-row-label", {Row::Verification.label()} }
                                        InfoButton { topic: HintTopic::Verification }
                                    }
                                        div { class: "settings-row-value",
                                            VerificationActions {}
                                        }
                                    }
                                }

                                div {
                                    class: "settings-row",
                                    span { class: "settings-row-label", {Row::Password.label()} }
                                    div { class: "settings-row-value",
                                        span { {model::PASSWORD_MASK} }
                                        Button {
                                            variant: ButtonVariant::Action,
                                            onclick: move |_| driver.dispatch(Action::OpenSheet(Sheet::Password)),
                                            {model::CHANGE}
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
                                            onclick: move |_| driver.dispatch(Action::LogOut),
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
                                            onclick: move |_| driver.dispatch(Action::More(MoreItem::DeleteAccount)),
                                            {model::DELETE}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                LogoutDialog {
                    open: driver.overlays.log_out,
                    on_confirm: move |()| driver.dispatch(Action::ConfirmLogOut),
                }
                DeleteAccountDialog { open: driver.overlays.delete }
                ChangeUsernameSheet { open: driver.overlays.username }
                ChangeEmailSheet { open: driver.overlays.email }
                ChangePasswordSheet { open: driver.overlays.password }
            }
    }
}

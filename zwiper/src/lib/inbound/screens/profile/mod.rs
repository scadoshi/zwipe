//! User profile management screen and its edit sheets.
//!
//! Provides the profile overview plus bottom sheets for updating user info.

/// Change email bottom sheet.
pub mod change_email;
/// Change password bottom sheet.
pub mod change_password;
/// Change username bottom sheet.
pub mod change_username;
/// Extracted components for the profile screen.
mod components;
/// User preferences bottom sheet.
pub mod preferences;

use crate::inbound::components::bottom_sheet::BottomSheet;
use crate::inbound::components::logout_dialog::LogoutDialog;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::router::Router;
use crate::{
    inbound::{
        components::auth::bouncer::Bouncer,
        components::hint_dialog::{
            HintBullet, HintBullets, HintDialog, HintKey, use_one_time_hint,
        },
    },
    outbound::client::{ZwipeClient, user::get_user::ClientGetUser},
    outbound::open_url,
};
use change_email::ChangeEmailSheet;
use change_password::ChangePasswordSheet;
use change_username::ChangeUsernameSheet;
use components::delete_account_dialog::DeleteAccountDialog;
use components::email_verification::{EmailVerification, VerificationActions};
use dioxus::prelude::*;
use preferences::{PreferencesSheet, display_theme_name};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::hints::HINT_PROFILE;
use zwipe_core::domain::user::models::theme::ThemeConfig;

/// Client version baked in at compile time, shown at the bottom of the screen
/// so users can report which build they're on.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Marketing site, opened externally from the About section.
const WEB_DOMAIN: &str = "https://zwipe.net";

/// User profile screen showing account details and management options.
#[component]
pub fn Profile() -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let theme_config: Signal<ThemeConfig> = use_context();

    let mut show_logout_dialog = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| false);
    let mut show_more_sheet = use_signal(|| false);
    let mut preferences_open = use_signal(|| false);
    let mut change_username_open = use_signal(|| false);
    let mut change_email_open = use_signal(|| false);
    let mut change_password_open = use_signal(|| false);

    // Account management hint: auto-opens on this user's first visit, the
    // grayed "?" in the header reopens it on demand.
    let profile_hint_open = use_one_time_hint(HINT_PROFILE);

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
                ScreenHeader { title: "Profile", hint: profile_hint_open }

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
                            "Find your theme and dark mode under "
                            HintKey { "Preferences" }
                        }
                        HintBullet {
                            "Tap "
                            HintKey { "More" }
                            " for account actions like deleting your account"
                        }
                    }
                }

                div { class: "screen-content content-enter",
                    if let Some(s) = session().as_ref() {
                        div { class: "profile-sections",

                            div { class: "profile-list",

                                div { class: "card-header",
                                    span { class: "card-title", "Account" }
                                }

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
                                            on_change: move |_| change_email_open.set(true),
                                        }
                                    }
                                }

                                if s.user.email_verified_at.is_none() {
                                    div {
                                        class: "profile-row",
                                        span { class: "profile-row-label", "Verification" }
                                        div { class: "profile-row-value",
                                            VerificationActions {}
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

                            div { class: "profile-list",

                                div { class: "card-header",
                                    span { class: "card-title", "Preferences" }
                                    button {
                                        class: "util-btn",
                                        onclick: move |_| preferences_open.set(true),
                                        "Change"
                                    }
                                }

                                div {
                                    class: "profile-row",
                                    span { class: "profile-row-label", "Theme" }
                                    div { class: "profile-row-value",
                                        span { { display_theme_name(&theme_config().name) } }
                                    }
                                }

                                div {
                                    class: "profile-row",
                                    span { class: "profile-row-label", "Dark mode" }
                                    div { class: "profile-row-value",
                                        span { if theme_config().is_dark { "On" } else { "Off" } }
                                    }
                                }
                            }

                            div { class: "profile-list",

                                div { class: "card-header",
                                    span { class: "card-title", "About" }
                                }

                                div {
                                    class: "profile-row",
                                    span { class: "profile-row-label", "Website" }
                                    div { class: "profile-row-value",
                                        button {
                                            class: "util-btn",
                                            onclick: move |_| open_url::open(WEB_DOMAIN),
                                            "zwipe.net \u{2197}"
                                        }
                                    }
                                }

                                div {
                                    class: "profile-row",
                                    span { class: "profile-row-label", "Privacy Policy" }
                                    div { class: "profile-row-value",
                                        button {
                                            class: "util-btn",
                                            onclick: move |_| {
                                                navigator.push(Router::PrivacyPolicy {});
                                            },
                                            "View"
                                        }
                                    }
                                }

                                div {
                                    class: "profile-row",
                                    span { class: "profile-row-label", "Version" }
                                    div { class: "profile-row-value",
                                        span { "v{APP_VERSION}" }
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
                        class: "util-btn util-btn-danger",
                        onclick: move |_| show_logout_dialog.set(true),
                        "Log out"
                    }
                    button {
                        class: "util-btn",
                        onclick: move |_| show_more_sheet.set(true),
                        "More"
                    }
                }

                BottomSheet { open: show_more_sheet, title: "More".to_string(),
                    button {
                        class: "btn btn-danger",
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

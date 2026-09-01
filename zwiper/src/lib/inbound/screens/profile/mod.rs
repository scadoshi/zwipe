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
/// Universes Beyond preference bottom sheet.
pub mod universes_beyond;

use crate::{
    inbound::{
        components::{
            auth::ensure_session::EnsureFresh,
            bottom_sheet::BottomSheet,
            hint_dialog::{HintBullet, HintBullets, HintDialog, HintKey, use_one_time_hint},
            logout_dialog::LogoutDialog,
            screen_header::ScreenHeader,
            telemetry::{
                usage_buffer::UsageBuffer,
                vocabulary::{component, screen},
            },
        },
        router::Router,
    },
    outbound::{
        client::{
            ZwipeClient,
            user::{
                get_user::ClientGetUser,
                preferences::{ClientGetPreferences, ClientUpdatePreferences},
            },
        },
        open_url,
    },
};
use change_email::ChangeEmailSheet;
use change_password::ChangePasswordSheet;
use change_username::ChangeUsernameSheet;
use components::{
    delete_account_dialog::DeleteAccountDialog,
    email_verification::{EmailVerification, VerificationActions},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use preferences::{PreferencesSheet, display_theme_name};
use std::time::Duration;
use universes_beyond::UniversesBeyondExceptionsSheet;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::scryfall_data::universe::franchise_by_slug,
        site::WEB_BASE,
        user::models::{hints::HINT_PROFILE, theme::ThemeConfig},
    },
    http::contracts::user::HttpUpdatePreferences,
};

/// Client version baked in at compile time, shown at the bottom of the screen
/// so users can report which build they're on.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// User profile screen showing account details and management options.
#[component]
pub fn Profile() -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut theme_config: Signal<ThemeConfig> = use_context();
    let toast = use_toast();
    let usage_buffer: Signal<UsageBuffer> = use_context();

    let mut show_logout_dialog = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| false);
    let mut show_more_sheet = use_signal(|| false);
    let mut preferences_open = use_signal(|| false);
    let mut ub_exceptions_open = use_signal(|| false);

    // Universes Beyond preference, fetched once on mount. The toggle writes
    // through immediately (dark-mode pattern); the exceptions sheet edits and
    // writes back the whitelist.
    let mut ub_hide = use_signal(|| false);
    let mut ub_exceptions: Signal<Vec<String>> = use_signal(Vec::new);
    let mut ub_loaded = use_signal(|| false);
    // Exceptions explainer, shared with the sheet: the row's "?" and the
    // sheet's header "?" both open it.
    let mut ub_hint_open = use_signal(|| false);
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

    // Load the stored Universes Beyond preference once on mount.
    use_effect(move || {
        spawn(async move {
            let Ok(session_val) = session.ensure_fresh(client).await else {
                return;
            };
            match client().get_preferences(&session_val).await {
                Ok(prefs) => {
                    ub_hide.set(prefs.exclude_universes_beyond);
                    ub_exceptions.set(prefs.universes_beyond_exceptions);
                    ub_loaded.set(true);
                }
                Err(e) => {
                    tracing::warn!("get preferences failed: {e}");
                }
            }
        });
    });

    // Show/Hide toggle for Universes Beyond: flips optimistically and persists
    // immediately, reverting on failure. Same shape as the dark-mode toggle.
    let mut toggle_universes_beyond = move || {
        let prev = *ub_hide.peek();
        let next = !prev;
        ub_hide.set(next);
        let request = HttpUpdatePreferences {
            theme: None,
            dark_mode: None,
            exclude_universes_beyond: Some(next),
            universes_beyond_exceptions: None,
        };
        spawn(async move {
            let session_val = match session.ensure_fresh(client).await {
                Ok(session_val) => session_val,
                Err(e) => {
                    ub_hide.set(prev);
                    usage_buffer.peek().report_error(
                        screen::PROFILE,
                        component::NONE,
                        "update_preferences",
                        &e,
                    );
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    return;
                }
            };
            match client().update_preferences(request, &session_val).await {
                Ok(prefs) => ub_hide.set(prefs.exclude_universes_beyond),
                Err(e) => {
                    ub_hide.set(prev);
                    tracing::warn!("update universes beyond failed: {e}");
                    usage_buffer.peek().report_error(
                        screen::PROFILE,
                        component::NONE,
                        "update_preferences",
                        &e,
                    );
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                }
            }
        });
    };

    let navigator = use_navigator();

    // Dark-mode toggle that lives on the profile page: flips the theme live and
    // persists it immediately, reverting the UI if the save fails.
    let mut toggle_dark_mode = move || {
        let prev = theme_config.peek().clone();
        let next = ThemeConfig {
            name: prev.name.clone(),
            is_dark: !prev.is_dark,
        };
        theme_config.set(next.clone());
        let request = HttpUpdatePreferences {
            theme: Some(next.name.clone()),
            dark_mode: Some(next.is_dark),
            exclude_universes_beyond: None,
            universes_beyond_exceptions: None,
        };
        spawn(async move {
            let session_val = match session.ensure_fresh(client).await {
                Ok(session_val) => session_val,
                Err(e) => {
                    theme_config.set(prev);
                    usage_buffer.peek().report_error(
                        screen::PROFILE,
                        component::NONE,
                        "load_profile",
                        &e,
                    );
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    return;
                }
            };
            match client().update_preferences(request, &session_val).await {
                Ok(prefs) => theme_config.set(ThemeConfig::from(&prefs)),
                Err(e) => {
                    theme_config.set(prev);
                    tracing::warn!("update dark mode failed: {e}");
                    usage_buffer.peek().report_error(
                        screen::PROFILE,
                        component::NONE,
                        "update_preferences",
                        &e,
                    );
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                }
            }
        });
    };

    rsx! {
            div { class: "screen",
                ScreenHeader { title: "Profile", hint: profile_hint_open }

                HintDialog {
                    open: profile_hint_open,
                    title: "Your profile",
                    HintBullets {
                        HintBullet {
                            "Tap "
                            HintKey { color: "--accent-primary", "Change" }
                            " to update your username, email or password"
                        }
                        HintBullet {
                            "Toggle "
                            HintKey { color: "--accent-secondary", "Dark mode" }
                            " right here, or tap "
                            HintKey { color: "--accent-primary", "Change" }
                            " on Theme to pick a palette"
                        }
                        HintBullet {
                            "Set "
                            HintKey { color: "--accent-secondary", "Universes Beyond" }
                            " to Hide to keep crossover cards out of searches and commander picks"
                        }
                        HintBullet {
                            HintKey { color: "--accent-secondary", "Exceptions" }
                            " picks franchises that still show up while hidden"
                        }
                        HintBullet {
                            "Tap "
                            HintKey { color: "--accent-tertiary", "More" }
                            " to delete your account"
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
                                        Button {
                                            variant: ButtonVariant::Util,
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
                                        Button {
                                            variant: ButtonVariant::Util,
                                            onclick: move |_| change_password_open.set(true),
                                            "Change"
                                        }
                                    }
                                }
                            }

                            div { class: "profile-list",

                                div { class: "card-header",
                                    span { class: "card-title", "Preferences" }
                                }

                                div {
                                    class: "profile-row",
                                    span { class: "profile-row-label", "Theme" }
                                    div { class: "profile-row-value",
                                        span { { display_theme_name(&theme_config().name) } }
                                        Button {
                                            variant: ButtonVariant::Util,
                                            onclick: move |_| preferences_open.set(true),
                                            "Change"
                                        }
                                    }
                                }

                                div {
                                    class: "profile-row",
                                    span { class: "profile-row-label", "Dark mode" }
                                    div { class: "profile-row-value",
                                        Button {
                                            variant: ButtonVariant::Util,
                                            onclick: move |_| toggle_dark_mode(),
                                            if theme_config().is_dark { "On" } else { "Off" }
                                        }
                                    }
                                }

                                div {
                                    class: "profile-row",
                                    span { class: "profile-row-label", "Universes Beyond" }
                                    div { class: "profile-row-value",
                                        Button {
                                            variant: ButtonVariant::Util,
                                            disabled: !ub_loaded(),
                                            onclick: move |_| toggle_universes_beyond(),
                                            if ub_hide() { "Hide" } else { "Show" }
                                        }
                                    }
                                }

                                div {
                                    class: if ub_hide() { "profile-row ub-exceptions-row" } else { "profile-row ub-exceptions-row ub-chips-disabled" },
                                    span { style: "display:flex;align-items:center;gap:0.35rem;",
                                        button {
                                            class: "info-button",
                                            r#type: "button",
                                            onclick: move |evt| {
                                                evt.stop_propagation();
                                                ub_hint_open.set(true);
                                            },
                                            "?"
                                        }
                                        span { class: "profile-row-label", "Exceptions" }
                                    }
                                    div { class: "profile-row-value",
                                        Button {
                                            variant: ButtonVariant::Util,
                                            disabled: !ub_loaded() || !ub_hide(),
                                            onclick: move |_| ub_exceptions_open.set(true),
                                            "Change"
                                        }
                                    }
                                    if !ub_exceptions().is_empty() {
                                        div { class: "ub-exception-chips",
                                            {
                                                let mut names: Vec<&str> = ub_exceptions()
                                                    .iter()
                                                    .filter_map(|s| franchise_by_slug(s))
                                                    .map(|f| f.name)
                                                    .collect();
                                                names.sort_unstable();
                                                rsx! {
                                                    for name in names {
                                                        span { class: "stat-chip stat-chip-tag", { name } }
                                                    }
                                                }
                                            }
                                        }
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
                                        Button {
                                            variant: ButtonVariant::Util,
                                            onclick: move |_| open_url::open(WEB_BASE),
                                            "zwipe.net \u{2197}"
                                        }
                                    }
                                }

                                div {
                                    class: "profile-row",
                                    span { class: "profile-row-label", "Privacy Policy" }
                                    div { class: "profile-row-value",
                                        Button {
                                            variant: ButtonVariant::Util,
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
                                        Button {
                                            variant: ButtonVariant::Util,
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
                        variant: ButtonVariant::Util,
                        onclick: move |_| navigator.go_back(),
                        "Back"
                    }
                    Button {
                        variant: ButtonVariant::Util,
                        danger: true,
                        onclick: move |_| show_logout_dialog.set(true),
                        "Log out"
                    }
                    Button {
                        variant: ButtonVariant::Util,
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
                PreferencesSheet { open: preferences_open }
                UniversesBeyondExceptionsSheet {
                    open: ub_exceptions_open,
                    exceptions: ub_exceptions,
                    hint_open: ub_hint_open,
                }
                ChangeUsernameSheet { open: change_username_open }
                ChangeEmailSheet { open: change_email_open }
                ChangePasswordSheet { open: change_password_open }
            }
    }
}

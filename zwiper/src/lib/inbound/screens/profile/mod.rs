//! User profile management screen and its edit sheets.
//!
//! Provides the profile overview plus bottom sheets for updating user info.

/// Change email bottom sheet.
use zwipe_client::ZwipeClient;
/// App icon bottom sheet.
pub mod app_icon_sheet;
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
            auth::authed::use_authed,
            hint_dialog::{HintDialog, use_one_time_hint},
            hint_host::HintTopic,
            info_button::InfoButton,
            logout_dialog::LogoutDialog,
            screen_header::ScreenHeader,
            telemetry::vocabulary::{ProfileScreen, Screen},
        },
        router::Router,
    },
    outbound::{app_icon, open_url},
};
use ::components::{
    ActionBar, BottomSheet, Button, ButtonVariant, DarkModeButton, HintBullet, HintBullets,
    HintKey, TOAST_NORMAL, ThemeConfig, theme_from_preferences,
};
use app_icon_sheet::AppIconSheet;
use change_email::ChangeEmailSheet;
use change_password::ChangePasswordSheet;
use change_username::ChangeUsernameSheet;
use components::{
    delete_account_dialog::DeleteAccountDialog,
    email_verification::{EmailVerification, VerificationActions},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use preferences::PreferencesSheet;
use universes_beyond::UniversesBeyondExceptionsSheet;
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::scryfall_data::universe::franchise_by_slug,
        site::WEB_BASE,
        user::{models::hints::HINT_PROFILE, preferences::display_theme_name},
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
    let authed = use_authed(Screen::Profile(ProfileScreen::Main));
    let toast = use_toast();

    // Every toggle on this screen says what it did, in one voice: a setting
    // that changes something you cannot see from here (which cards get
    // served) is otherwise a button that looks like it did nothing. Sheets
    // raise their own on Save, since they carry the value that was committed.
    let said = use_callback(move |what: &'static str| {
        toast.success(
            what.to_string(),
            ToastOptions::default().duration(TOAST_NORMAL),
        );
    });

    let mut show_logout_dialog = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| false);
    let mut show_more_sheet = use_signal(|| false);
    let mut preferences_open = use_signal(|| false);
    let mut app_icon_open = use_signal(|| false);
    let mut app_icon_theme = use_signal(app_icon::current);
    let app_icon_follows = use_signal(app_icon::follows_theme);
    let mut ub_exceptions_open = use_signal(|| false);

    // The home-screen icon's own mode, apart from the app's.
    let toggle_app_icon_dark = move |_| {
        let prev = app_icon_theme.read().clone();
        let next = ThemeConfig {
            name: prev.name,
            is_dark: !prev.is_dark,
        };
        match app_icon::set(&next) {
            Ok(()) => {
                toast.success(
                    format!("App icon {}", app_icon::mode(next.is_dark)),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
                app_icon_theme.set(next);
            }
            Err(e) => {
                tracing::error!("failed to set app icon: {e}");
                toast.error(e.to_string(), ToastOptions::default());
            }
        }
    };

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

    // Load the stored Universes Beyond preference once on mount. Quiet on
    // failure: the rows stay disabled until loaded, and a toast on every
    // flaky profile open would be noise (the failure still reaches
    // telemetry through the facade).
    use_effect(move || {
        spawn(async move {
            if let Some(prefs) = authed
                .run_quiet("get_preferences", |c, s| async move {
                    c.get_preferences(&s).await
                })
                .await
            {
                ub_hide.set(prefs.exclude_universes_beyond);
                ub_exceptions.set(prefs.universes_beyond_exceptions);
                ub_loaded.set(true);
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
            // try_run: the facade reports and toasts; the Err back here only
            // drives the optimistic revert.
            match authed
                .try_run("update_preferences", |c, s| async move {
                    c.update_preferences(request, &s).await
                })
                .await
            {
                Ok(prefs) => {
                    ub_hide.set(prefs.exclude_universes_beyond);
                    said.call(if prefs.exclude_universes_beyond {
                        "Universes Beyond hidden"
                    } else {
                        "Universes Beyond shown"
                    });
                }
                Err(_) => ub_hide.set(prev),
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
            // try_run: reporting and toasts live in the facade; Err drives
            // the optimistic revert. (This also normalizes the old refresh
            // path's "load_profile" op to "update_preferences".)
            match authed
                .try_run("update_preferences", |c, s| async move {
                    c.update_preferences(request, &s).await
                })
                .await
            {
                Ok(prefs) => {
                    let theme = theme_from_preferences(&prefs);
                    app_icon::follow(&theme);
                    app_icon_theme.set(app_icon::current());
                    theme_config.set(theme);
                    said.call(if prefs.dark_mode {
                        "Dark mode on"
                    } else {
                        "Dark mode off"
                    });
                }
                Err(_) => theme_config.set(prev),
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
                                            on_change: move |_| change_email_open.set(true),
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
                                    span { class: "settings-list-title", "Preferences" }
                                }

                                div {
                                    class: "settings-row",
                                    span { style: "display:flex;align-items:center;",
                                        span { class: "settings-row-label", "Theme" }
                                        InfoButton { topic: HintTopic::Theme }
                                    }
                                    div { class: "settings-row-value",
                                        span { { display_theme_name(&theme_config().name) } }
                                        DarkModeButton {
                                            is_dark: theme_config().is_dark,
                                            onclick: move |_| toggle_dark_mode(),
                                        }
                                        Button {
                                            variant: ButtonVariant::Action,
                                            onclick: move |_| preferences_open.set(true),
                                            "Change"
                                        }
                                    }
                                }

                                if app_icon::SUPPORTED {
                                    div {
                                        class: "settings-row",
                                        span { style: "display:flex;align-items:center;",
                                            span { class: "settings-row-label", "App icon" }
                                            InfoButton { topic: HintTopic::AppIcon }
                                        }
                                        div { class: "settings-row-value",
                                            if app_icon_follows() {
                                                span { "Matches theme" }
                                            } else {
                                                span { { display_theme_name(&app_icon_theme().name) } }
                                                DarkModeButton {
                                                    is_dark: app_icon_theme().is_dark,
                                                    onclick: toggle_app_icon_dark,
                                                }
                                            }
                                            Button {
                                                variant: ButtonVariant::Action,
                                                onclick: move |_| app_icon_open.set(true),
                                                "Change"
                                            }
                                        }
                                    }
                                }

                                div {
                                    class: "settings-row",
                                    span { style: "display:flex;align-items:center;",
                                        span { class: "settings-row-label", "Universes Beyond" }
                                        InfoButton { topic: HintTopic::UniversesBeyond }
                                    }
                                    div { class: "settings-row-value",
                                        Button {
                                            variant: ButtonVariant::Action,
                                            disabled: !ub_loaded(),
                                            onclick: move |_| toggle_universes_beyond(),
                                            if ub_hide() { "Hide" } else { "Show" }
                                        }
                                    }
                                }

                                div {
                                    class: if ub_hide() { "settings-row ub-exceptions-row" } else { "settings-row ub-exceptions-row ub-chips-disabled" },
                                    span { style: "display:flex;align-items:center;",
                                        span { class: "settings-row-label", "Exceptions" }
                                        button {
                                            class: "info-button",
                                            r#type: "button",
                                            onclick: move |evt| {
                                                evt.stop_propagation();
                                                ub_hint_open.set(true);
                                            },
                                            "?"
                                        }
                                    }
                                    div { class: "settings-row-value",
                                        Button {
                                            variant: ButtonVariant::Action,
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
                                                    // Each chip eases in a beat after the one before it.
                                                    for (i, name) in names.into_iter().enumerate() {
                                                        span {
                                                            key: "{name}",
                                                            class: "stat-chip stat-chip-muted ub-exception-chip chip-in",
                                                            style: "animation-delay: {i * 45}ms",
                                                            { name }
                                                        }
                                                    }
                                                }
                                            }
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
                PreferencesSheet { open: preferences_open }
                AppIconSheet {
                    open: app_icon_open,
                    icon: app_icon_theme,
                    follows: app_icon_follows,
                }
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

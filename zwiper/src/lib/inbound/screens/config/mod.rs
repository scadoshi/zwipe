//! Config screen and its sheets: how the app looks (theme, pips, app icon)
//! and which cards it serves (Universes Beyond and its exceptions).

/// App icon bottom sheet.
pub mod app_icon_sheet;
/// Theme bottom sheet, saving to the account's preferences.
pub mod preferences;
/// Universes Beyond exceptions bottom sheet.
pub mod universes_beyond;

use app_icon_sheet::AppIconSheet;
use components::{
    ActionBar, Button, ButtonVariant, DarkModeButton, HintBullet, HintBullets, HintDialog, HintKey,
    TOAST_NORMAL, ThemeConfig,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use preferences::PreferencesSheet;
use universes_beyond::UniversesBeyondExceptionsSheet;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::scryfall_data::universe;
use zwipe_core::domain::user;
use zwipe_core::domain::user::models::hints::HINT_CONFIG;
use zwipe_core::domain::user::preferences::PipStyle;
use zwipe_core::http::contracts::user::HttpUpdatePreferences;

use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::hint_dialog::use_one_time_hint;
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::{ProfileScreen, Screen};
use crate::outbound::app_icon;
use crate::outbound::session::Persist;

/// Numbers one toggle's requests so a reply can tell whether a newer tap
/// has gone out since.
#[derive(Debug, Default, Clone, Copy)]
struct RequestSeq {
    latest: u64,
}

impl RequestSeq {
    /// Numbers a new request; it is now the latest.
    fn begin(&mut self) -> u64 {
        self.latest = self.latest.wrapping_add(1);
        self.latest
    }
}

/// What a toggle shows once request `mine` answers: the server's value, or
/// `prev` (the value from before that tap) on failure. `None` leaves the
/// screen alone, since a newer tap's request owns it.
fn settle<T>(seq: RequestSeq, mine: u64, reply: Option<T>, prev: T) -> Option<T> {
    if seq.latest != mine {
        return None;
    }
    Some(reply.unwrap_or(prev))
}

/// Config screen: the account's appearance and card preferences. Theme, pips
/// and Universes Beyond save through the preferences API; the app icon is a
/// device setting and only shows where the platform supports it.
#[component]
pub fn Config() -> Element {
    let mut theme_config: Signal<ThemeConfig> = use_context();
    let mut pip_style: Signal<PipStyle> = use_context();
    let mut session: Signal<Option<Session>> = use_context();
    let authed = use_authed(Screen::Profile(ProfileScreen::Preferences));
    let toast = use_toast();
    let navigator = use_navigator();

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

    // Config explainer: auto-opens on this user's first visit, the grayed
    // "?" in the header reopens it on demand.
    let config_hint_open = use_one_time_hint(HINT_CONFIG);

    // Load the stored Universes Beyond preference once on mount. Quiet on
    // failure: the rows stay disabled until loaded, and a toast on every
    // flaky open would be noise (the failure still reaches telemetry
    // through the facade).
    use_effect(move || {
        spawn(async move {
            if let Some(prefs) = authed
                .run_quiet("load_preferences", |c, s| async move {
                    c.fetch_preferences(&s).await
                })
                .await
            {
                ub_hide.set(prefs.exclude_universes_beyond);
                ub_exceptions.set(prefs.universes_beyond_exceptions);
                ub_loaded.set(true);
                pip_style.set(prefs.pip_style);
            }
        });
    });

    // Show/Hide toggle for Universes Beyond: flips optimistically and persists
    // immediately, reverting on failure. Same shape as the dark-mode toggle.
    let mut ub_seq = use_signal(RequestSeq::default);
    let mut toggle_universes_beyond = move || {
        let prev = *ub_hide.peek();
        let next = !prev;
        ub_hide.set(next);
        let mine = ub_seq.write().begin();
        let request = HttpUpdatePreferences {
            theme: None,
            dark_mode: None,
            exclude_universes_beyond: Some(next),
            universes_beyond_exceptions: None,
            pip_style: None,
        };
        spawn(async move {
            // try_run: the facade reports and toasts; the Err back here only
            // drives the optimistic revert.
            let reply = authed
                .try_run("update_preferences", |c, s| async move {
                    c.update_preferences(request, &s).await
                })
                .await
                .ok();
            let saved = reply.map(|prefs| prefs.exclude_universes_beyond);
            let Some(shown) = settle(*ub_seq.peek(), mine, saved, prev) else {
                return;
            };
            ub_hide.set(shown);
            if saved.is_some() {
                said.call(if shown {
                    "Universes Beyond hidden"
                } else {
                    "Universes Beyond shown"
                });
            }
        });
    };

    // Theme/Natural toggle for pips: recolors every pip at once, persists
    // immediately and reverts on failure. The stored session keeps the
    // saved style so the next launch opens with it.
    let mut pip_seq = use_signal(RequestSeq::default);
    let mut toggle_pip_style = move || {
        let prev = *pip_style.peek();
        let next = prev.toggled();
        pip_style.set(next);
        let mine = pip_seq.write().begin();
        let request = HttpUpdatePreferences {
            theme: None,
            dark_mode: None,
            exclude_universes_beyond: None,
            universes_beyond_exceptions: None,
            pip_style: Some(next),
        };
        spawn(async move {
            // try_run: the facade reports and toasts; the Err back here only
            // drives the optimistic revert.
            let reply = authed
                .try_run("update_preferences", |c, s| async move {
                    c.update_preferences(request, &s).await
                })
                .await
                .ok();
            let saved = reply.map(|prefs| prefs.pip_style);
            let Some(shown) = settle(*pip_seq.peek(), mine, saved, prev) else {
                return;
            };
            pip_style.set(shown);
            if saved.is_some() {
                let current = session.peek().clone();
                if let Some(mut current) = current {
                    current.preferences.pip_style = shown;
                    current.infallible_save();
                    session.set(Some(current));
                }
                said.call(match shown {
                    PipStyle::Theme => "Pips in theme colors",
                    PipStyle::Natural => "Pips in natural colors",
                });
            }
        });
    };

    // Dark-mode toggle in the Theme row: flips the theme live and persists it
    // immediately, reverting the UI if the save fails.
    let mut dark_seq = use_signal(RequestSeq::default);
    let mut toggle_dark_mode = move || {
        let prev = theme_config.peek().clone();
        let next = ThemeConfig {
            name: prev.name.clone(),
            is_dark: !prev.is_dark,
        };
        theme_config.set(next.clone());
        let mine = dark_seq.write().begin();
        let request = HttpUpdatePreferences {
            theme: Some(next.name.clone()),
            dark_mode: Some(next.is_dark),
            exclude_universes_beyond: None,
            universes_beyond_exceptions: None,
            pip_style: None,
        };
        spawn(async move {
            // try_run: reporting and toasts live in the facade; Err drives
            // the optimistic revert.
            let reply = authed
                .try_run("update_preferences", |c, s| async move {
                    c.update_preferences(request, &s).await
                })
                .await
                .ok();
            let saved_dark = reply.as_ref().map(|prefs| prefs.dark_mode);
            let saved = reply.map(|prefs| components::theme_from_preferences(&prefs));
            let Some(theme) = settle(*dark_seq.peek(), mine, saved, prev) else {
                return;
            };
            if saved_dark.is_some() {
                app_icon::follow(&theme);
                app_icon_theme.set(app_icon::current());
            }
            theme_config.set(theme);
            if let Some(dark_mode) = saved_dark {
                said.call(if dark_mode {
                    "Dark mode on"
                } else {
                    "Dark mode off"
                });
            }
        });
    };

    rsx! {
            div { class: "screen",
                ScreenHeader { title: "Config", hint: config_hint_open }

                HintDialog {
                    open: config_hint_open,
                    title: "Config",
                    HintBullets {
                        HintBullet { "Set how Zwipe looks and which cards it shows" }
                        HintBullet { "Tap a row's " HintKey { color: "--palette-6", "?" } " for what it does" }
                    }
                }

                div { class: "screen-content content-enter",
                    div { class: "settings-sections",

                        div { class: "settings-list",

                            div { class: "settings-list-header",
                                span { class: "settings-list-title", "Appearance" }
                            }

                            div {
                                class: "settings-row",
                                span { style: "display:flex;align-items:center;",
                                    span { class: "settings-row-label", "Theme" }
                                    InfoButton { topic: HintTopic::Theme }
                                }
                                div { class: "settings-row-value",
                                    span { { user::preferences::display_theme_name(&theme_config().name) } }
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

                            div {
                                class: "settings-row",
                                span { style: "display:flex;align-items:center;",
                                    span { class: "settings-row-label", "Pips" }
                                    InfoButton { topic: HintTopic::Pips }
                                }
                                div { class: "settings-row-value",
                                    Button {
                                        variant: ButtonVariant::Action,
                                        onclick: move |_| toggle_pip_style(),
                                        match pip_style() {
                                            PipStyle::Theme => "Theme",
                                            PipStyle::Natural => "Natural",
                                        }
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
                                            span { { user::preferences::display_theme_name(&app_icon_theme().name) } }
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
                        }

                        div { class: "settings-list",

                            div { class: "settings-list-header",
                                span { class: "settings-list-title", "Cards" }
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
                                                .filter_map(|s| universe::franchise_by_slug(s))
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
                    }
                }

                ActionBar {
                    Button {
                        variant: ButtonVariant::Action,
                        onclick: move |_| navigator.go_back(),
                        "Back"
                    }
                }

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
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reply_overtaken_by_a_newer_tap_changes_nothing() {
        let mut seq = RequestSeq::default();
        // Hide, then show again before the first reply.
        let first = seq.begin();
        let second = seq.begin();

        assert_eq!(
            settle(seq, second, Some(false), true),
            Some(false),
            "the newest reply lands"
        );
        assert_eq!(
            settle(seq, first, Some(true), false),
            None,
            "a late older reply is dropped"
        );
        assert_eq!(
            settle(seq, first, None, false),
            None,
            "and so is a late older failure"
        );
    }

    #[test]
    fn the_latest_failure_rolls_back_to_the_value_before_its_tap() {
        let mut seq = RequestSeq::default();
        let _first = seq.begin();
        let second = seq.begin();
        // The second tap flipped true back to false; its failure restores true.
        assert_eq!(settle(seq, second, None, true), Some(true));
    }
}

//! Config: the `Appearance` and `Cards` lists side by side on a wide window,
//! Themes as a side panel and Exceptions as a modal. The app's `App icon`
//! row is left out, since a browser has no app icon.

mod exceptions;
mod themes;

use components::{ActionBar, Button, ButtonVariant, DarkModeButton, TOAST_NORMAL, ThemeConfig};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::preferences::{self as user_preferences, PipStyle};
use zwipe_core::http::contracts::user::HttpUpdatePreferences;

use crate::domain::preferences::{self, RequestSeq};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::screens::config::exceptions::ExceptionsSheet;
use crate::inbound::screens::config::themes::ThemesSheet;
use crate::inbound::shell::ScreenFrame;
use crate::outbound::session_store::SessionStore;

/// The update that changes only what `edit` sets.
fn update(edit: impl FnOnce(&mut HttpUpdatePreferences)) -> HttpUpdatePreferences {
    let mut request = HttpUpdatePreferences {
        theme: None,
        dark_mode: None,
        exclude_universes_beyond: None,
        universes_beyond_exceptions: None,
        pip_style: None,
    };
    edit(&mut request);
    request
}

/// The config screen. Theme and pips show at once from what this browser
/// has; the Universes Beyond controls wait for the account's preferences,
/// and stay disabled for the visit if that load fails. Every toggle flips at
/// once, saves, and says what it did; a failure puts back the value from
/// before that click, and a reply overtaken by a newer click is ignored.
#[component]
pub fn Config() -> Element {
    let mut theme: Signal<ThemeConfig> = use_context();
    let mut pip_style: Signal<PipStyle> = use_context();
    let mut session: Signal<Option<Session>> = use_context();
    let authed = use_authed();
    let toast = use_toast();
    let navigator = use_navigator();
    let hint_open = use_one_time_hint(OneTimeHint::Config);

    let said = use_callback(move |what: &'static str| {
        toast.success(
            what.to_string(),
            ToastOptions::default().duration(TOAST_NORMAL),
        );
    });

    let mut themes_open = use_signal(|| false);
    let mut exceptions_open = use_signal(|| false);
    let mut ub_hint_open = use_signal(|| false);
    let mut ub_hide = use_signal(|| false);
    let mut ub_exceptions: Signal<Vec<String>> = use_signal(Vec::new);
    let mut ub_loaded = use_signal(|| false);

    use_hook(move || {
        spawn(async move {
            if let Some(loaded) = authed
                .run_quiet("load_preferences", |c, s| async move {
                    c.fetch_preferences(&s).await
                })
                .await
            {
                ub_hide.set(loaded.exclude_universes_beyond);
                ub_exceptions.set(loaded.universes_beyond_exceptions);
                ub_loaded.set(true);
                pip_style.set(loaded.pip_style);
            }
        });
    });

    let mut dark_seq = use_signal(RequestSeq::default);
    let toggle_dark_mode = move |_| {
        let prev = theme.peek().clone();
        let next = ThemeConfig {
            name: prev.name.clone(),
            is_dark: !prev.is_dark,
        };
        theme.set(next.clone());
        let mine = dark_seq.write().begin();
        let request = update(|r| {
            r.theme = Some(next.name);
            r.dark_mode = Some(next.is_dark);
        });
        spawn(async move {
            let reply = authed
                .try_run("update_preferences", |c, s| async move {
                    c.update_preferences(request, &s).await
                })
                .await
                .ok();
            let saved_dark = reply.as_ref().map(|r| r.dark_mode);
            let saved = reply.map(|r| components::theme_from_preferences(&r));
            let Some(shown) = preferences::settle(*dark_seq.peek(), mine, saved, prev) else {
                return;
            };
            theme.set(shown);
            if let Some(dark) = saved_dark {
                said.call(if dark {
                    "Dark mode on"
                } else {
                    "Dark mode off"
                });
            }
        });
    };

    let mut pip_seq = use_signal(RequestSeq::default);
    let toggle_pips = move |_| {
        let prev = *pip_style.peek();
        let next = prev.toggled();
        pip_style.set(next);
        let mine = pip_seq.write().begin();
        let request = update(|r| r.pip_style = Some(next));
        spawn(async move {
            let saved = authed
                .try_run("update_preferences", |c, s| async move {
                    c.update_preferences(request, &s).await
                })
                .await
                .ok()
                .map(|r| r.pip_style);
            let Some(shown) = preferences::settle(*pip_seq.peek(), mine, saved, prev) else {
                return;
            };
            pip_style.set(shown);
            if saved.is_some() {
                let latest = session.peek().clone();
                if let Some(mut latest) = latest {
                    latest.preferences.pip_style = shown;
                    SessionStore::browser().infallible_save(&latest);
                    session.set(Some(latest));
                }
                said.call(match shown {
                    PipStyle::Theme => "Pips in theme colors",
                    PipStyle::Natural => "Pips in natural colors",
                });
            }
        });
    };

    let mut ub_seq = use_signal(RequestSeq::default);
    let toggle_universes_beyond = move |_| {
        let prev = *ub_hide.peek();
        let next = !prev;
        ub_hide.set(next);
        let mine = ub_seq.write().begin();
        let request = update(|r| r.exclude_universes_beyond = Some(next));
        spawn(async move {
            let saved = authed
                .try_run("update_preferences", |c, s| async move {
                    c.update_preferences(request, &s).await
                })
                .await
                .ok()
                .map(|r| r.exclude_universes_beyond);
            let Some(shown) = preferences::settle(*ub_seq.peek(), mine, saved, prev) else {
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

    let names = preferences::exception_names(&ub_exceptions());

    rsx! {
        ScreenFrame {
            title: "Config",
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div { class: "screen-content content-enter",
                div { class: "settings-sections web-settings-pair",
                    div { class: "settings-list",
                        div { class: "settings-list-header",
                            span { class: "settings-list-title", "Appearance" }
                        }
                        div { class: "settings-row",
                            RowLabel { label: "Theme", topic: HintTopic::Theme }
                            div { class: "settings-row-value",
                                span { {user_preferences::display_theme_name(&theme().name)} }
                                DarkModeButton { is_dark: theme().is_dark, onclick: toggle_dark_mode }
                                Button {
                                    variant: ButtonVariant::Action,
                                    onclick: move |_| themes_open.set(true),
                                    "Change"
                                }
                            }
                        }
                        div { class: "settings-row",
                            RowLabel { label: "Pips", topic: HintTopic::Pips }
                            div { class: "settings-row-value",
                                Button {
                                    variant: ButtonVariant::Action,
                                    onclick: toggle_pips,
                                    match pip_style() {
                                        PipStyle::Theme => "Theme",
                                        PipStyle::Natural => "Natural",
                                    }
                                }
                            }
                        }
                    }
                    div { class: "settings-list",
                        div { class: "settings-list-header",
                            span { class: "settings-list-title", "Cards" }
                        }
                        div { class: "settings-row",
                            RowLabel { label: "Universes Beyond", topic: HintTopic::UniversesBeyond }
                            div { class: "settings-row-value",
                                Button {
                                    variant: ButtonVariant::Action,
                                    disabled: !ub_loaded(),
                                    onclick: toggle_universes_beyond,
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
                                    "aria-label": "What is this?",
                                    onclick: move |event| {
                                        event.stop_propagation();
                                        ub_hint_open.set(true);
                                    },
                                    "?"
                                }
                            }
                            div { class: "settings-row-value",
                                Button {
                                    variant: ButtonVariant::Action,
                                    disabled: !ub_loaded() || !ub_hide(),
                                    onclick: move |_| exceptions_open.set(true),
                                    "Change"
                                }
                            }
                            if !names.is_empty() {
                                div { class: "ub-exception-chips",
                                    for (i , name) in names.into_iter().enumerate() {
                                        span {
                                            key: "{name}",
                                            class: "stat-chip stat-chip-muted ub-exception-chip chip-in",
                                            style: "animation-delay: {i.saturating_mul(45)}ms",
                                            "{name}"
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
        }
        OneTimeHintDialog { hint: OneTimeHint::Config, open: hint_open }
        OneTimeHintDialog { hint: OneTimeHint::UniversesBeyondExceptions, open: ub_hint_open }
        ThemesSheet { open: themes_open }
        ExceptionsSheet {
            open: exceptions_open,
            exceptions: ub_exceptions,
            hint_open: ub_hint_open,
        }
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

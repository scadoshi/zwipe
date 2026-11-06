//! Config screen and its sheets: how the app looks (theme, pips, app icon)
//! and which cards it serves (Universes Beyond and its exceptions).

/// App icon bottom sheet.
pub mod app_icon_sheet;
/// Universes Beyond exceptions bottom sheet.
pub mod universes_beyond;

use app_icon_sheet::AppIconSheet;
use components::{
    Button, ButtonVariant, DarkModeButton, PaletteIcon, TOAST_NORMAL, ThemeConfig, ToastOptions,
    Toasts, use_toast,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use universes_beyond::UniversesBeyondExceptionsSheet;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::domain::user::settings::PipStyle;
use zwipe_core::text::sentence_case;
use zwipe_ui::screens::config::{self as model, Action, Destination, Effect, Sheet, State};

use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::hint_dialog::OneTimeHintDialog;
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::{ProfileScreen, Screen};
use crate::inbound::components::theme_save::{self, ThemeSaveSheet, theme_config_of, theme_of};
use crate::inbound::components::{hint_dialog, ui_toast};
use crate::outbound::app_icon;
use crate::outbound::session::Persist;

/// Config's model with the handles that perform its effects. Copy, so every
/// handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    navigator: Navigator,
    toast: Toasts,
    authed: Authed,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    theme_config: Signal<ThemeConfig>,
    pip_style: Signal<PipStyle>,
    app_icon_theme: Signal<ThemeConfig>,
    config_hint_open: Signal<bool>,
    ub_hint_open: Signal<bool>,
    settings_open: Signal<bool>,
    ub_exceptions_open: Signal<bool>,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    fn perform(self, effect: Effect) {
        let Self {
            navigator,
            toast,
            authed,
            mut session,
            client,
            mut theme_config,
            mut pip_style,
            mut app_icon_theme,
            ..
        } = self;
        match effect {
            Effect::OfferHint(hint) => {
                let open = match hint {
                    OneTimeHint::UniversesBeyondExceptions => self.ub_hint_open,
                    _ => self.config_hint_open,
                };
                hint_dialog::open_and_record_hint(hint.key(), session, client, open);
            }
            Effect::FetchSettings(ticket) => {
                spawn(async move {
                    // Quiet on failure: the Universes Beyond rows stay off, and
                    // the failure still reaches telemetry through the facade.
                    let reply = authed
                        .run_quiet("load_preferences", |c, s| async move {
                            c.fetch_settings(&s).await
                        })
                        .await;
                    self.dispatch(match reply {
                        Some(settings) => Action::Loaded {
                            ticket,
                            settings: Box::new(settings),
                        },
                        None => Action::Failed { ticket },
                    });
                });
            }
            Effect::Save { ticket, change } => {
                theme_save::save(authed, session, ticket, change, move |action| {
                    self.dispatch(action);
                });
            }
            Effect::ShowTheme(theme) => theme_config.set(theme_config_of(theme)),
            Effect::ShowPips(style) => pip_style.set(style),
            Effect::ThemeSaved(theme) => {
                theme_save::saved(theme_config, theme);
                app_icon_theme.set(app_icon::current());
            }
            Effect::PipsSaved(style) => {
                pip_style.set(style);
                // The stored session keeps the saved style so the next launch
                // opens with it.
                let current = session.peek().clone();
                if let Some(mut current) = current {
                    current.settings.pip_style = style;
                    current.infallible_save();
                    session.set(Some(current));
                }
            }
            Effect::OpenSheet(sheet) => self.show_sheet(sheet, /* open */ true),
            Effect::CloseSheet(sheet) => self.show_sheet(sheet, /* open */ false),
            Effect::Navigate(Destination::Previous) => navigator.go_back(),
            Effect::Toast(raised) => ui_toast::raise(toast, raised),
        }
    }

    /// Opens or closes `sheet`. The theme sheet closes itself before it hands
    /// on a pick, so a close that is already done writes nothing.
    fn show_sheet(self, sheet: Sheet, open: bool) {
        let mut signal = match sheet {
            Sheet::Themes => self.settings_open,
            Sheet::Exceptions => self.ub_exceptions_open,
        };
        if *signal.peek() != open {
            signal.set(open);
        }
    }
}

/// Config screen: the account's appearance and card settings. Theme, pips
/// and Universes Beyond save through the settings API; the app icon is a
/// device setting and only shows where the platform supports it.
///
/// What the screen knows and does is `zwipe_ui::screens::config`; this file
/// draws its state and performs its effects. The app icon row is the app's
/// own.
#[component]
pub fn Config() -> Element {
    let theme_config: Signal<ThemeConfig> = use_context();
    let pip_style: Signal<PipStyle> = use_context();
    let toast = use_toast();

    let mut app_icon_open = use_signal(|| false);
    let mut app_icon_theme = use_signal(app_icon::current);
    let app_icon_follows = use_signal(app_icon::follows_theme);

    let driver = Driver {
        model: use_signal(State::new),
        navigator: use_navigator(),
        toast,
        authed: use_authed(Screen::Profile(ProfileScreen::Settings)),
        session: use_context(),
        client: use_context(),
        theme_config,
        pip_style,
        app_icon_theme,
        // Config explainer: auto-opens on this user's first visit, the grayed
        // "?" in the header reopens it on demand.
        config_hint_open: use_signal(|| false),
        // Exceptions explainer, shared with the sheet: the row's "?" and the
        // sheet's header "?" both open it.
        ub_hint_open: use_signal(|| false),
        settings_open: use_signal(|| false),
        ub_exceptions_open: use_signal(|| false),
    };
    let config_hint_open = driver.config_hint_open;
    let mut ub_hint_open = driver.ub_hint_open;
    use_hook(move || driver.dispatch(Action::Load));

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
                toast.error(sentence_case(&e.to_string()), ToastOptions::default());
            }
        }
    };

    // Snapshot the model once per render.
    let state = driver.model.read();
    let ub_enabled = state.universes_beyond_enabled();
    let ub_label = state.universes_beyond_label();
    let exceptions_enabled = state.exceptions_enabled();
    let exceptions_muted = state.exceptions_muted();
    let shows_chips = state.shows_exception_chips();
    let names = state.exception_names();
    let draft = state.draft().to_vec();
    let draft_unchanged = state.draft_unchanged();
    drop(state);

    rsx! {
            div { class: "screen",
                ScreenHeader { title: model::TITLE, hint: config_hint_open, back: move |()| driver.dispatch(Action::Back) }

                OneTimeHintDialog { open: config_hint_open, hint: OneTimeHint::Config }

                div { class: "screen-content content-enter",
                    div { class: "settings-sections",

                        div { class: "settings-list",

                            div { class: "settings-list-header",
                                span { class: "settings-list-title", {model::APPEARANCE} }
                            }

                            div {
                                class: "settings-row",
                                span { style: "display:flex;align-items:center;",
                                    span { class: "settings-row-label", {model::THEME} }
                                    InfoButton { topic: HintTopic::Theme }
                                }
                                div { class: "settings-row-value",
                                    span { { user::settings::display_theme_name(&theme_config().name) } }
                                    DarkModeButton {
                                        is_dark: theme_config().is_dark,
                                        onclick: move |_| {
                                            let shown = theme_of(theme_config.peek().clone());
                                            driver.dispatch(Action::ToggleDarkMode { shown });
                                        },
                                    }
                                    Button {
                                        variant: ButtonVariant::Action,
                                        onclick: move |_| driver.dispatch(Action::OpenThemes),
                                        PaletteIcon {}
                                        " "
                                        {model::CHANGE}
                                    }
                                }
                            }

                            div {
                                class: "settings-row",
                                span { style: "display:flex;align-items:center;",
                                    span { class: "settings-row-label", {model::PIPS} }
                                    InfoButton { topic: HintTopic::Pips }
                                }
                                div { class: "settings-row-value",
                                    Button {
                                        variant: ButtonVariant::Action,
                                        onclick: move |_| {
                                            let shown = *pip_style.peek();
                                            driver.dispatch(Action::TogglePips { shown });
                                        },
                                        {model::pips_label(pip_style())}
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
                                            span { { user::settings::display_theme_name(&app_icon_theme().name) } }
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
                                span { class: "settings-list-title", {model::CARDS} }
                            }

                            div {
                                class: "settings-row",
                                span { style: "display:flex;align-items:center;",
                                    span { class: "settings-row-label", {model::UNIVERSES_BEYOND} }
                                    InfoButton { topic: HintTopic::UniversesBeyond }
                                }
                                div { class: "settings-row-value",
                                    Button {
                                        variant: ButtonVariant::Action,
                                        disabled: !ub_enabled,
                                        onclick: move |_| driver.dispatch(Action::ToggleUniversesBeyond),
                                        {ub_label}
                                    }
                                }
                            }

                            div {
                                class: if exceptions_muted { "settings-row ub-exceptions-row ub-chips-disabled" } else { "settings-row ub-exceptions-row" },
                                span { style: "display:flex;align-items:center;",
                                    span { class: "settings-row-label", {model::EXCEPTIONS} }
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
                                        disabled: !exceptions_enabled,
                                        onclick: move |_| driver.dispatch(Action::OpenExceptions),
                                        {model::CHANGE}
                                    }
                                }
                                if shows_chips {
                                    div { class: "ub-exception-chips",
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


                ThemeSaveSheet {
                    open: driver.settings_open,
                    on: move |action| driver.dispatch(action),
                }
                AppIconSheet {
                    open: app_icon_open,
                    icon: app_icon_theme,
                    follows: app_icon_follows,
                }
                UniversesBeyondExceptionsSheet {
                    open: driver.ub_exceptions_open,
                    hint_open: ub_hint_open,
                    draft,
                    unchanged: draft_unchanged,
                    on: move |action| driver.dispatch(action),
                }
            }
    }
}

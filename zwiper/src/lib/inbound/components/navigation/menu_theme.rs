//! The main menu's theme buttons, after the footer's icons: a palette
//! that folds the menu and raises the Themes sheet, and the mode's sun or
//! moon, which flips Dark/Light.
//!
//! The pair and the sheet each drive a `zwipe_ui::screens::config` model and
//! save through the same path as Settings' Theme row (see
//! [`theme_save`](crate::inbound::components::theme_save)), so none of them
//! disagree: all draw the app-wide theme and all persist to the account's
//! settings. The sheet mounts at the app root as
//! [`MenuThemeSheet`](crate::inbound::components::navigation::menu_theme::MenuThemeSheet),
//! beside the router, and
//! [`ThemeSheetOpen`](crate::inbound::components::navigation::menu_theme::ThemeSheetOpen)
//! joins it to the palette. The header
//! holds no fixed layer of its own: on iOS a header holding one is drawn a
//! status bar inset low on the frame a theme wipe captures.

use components::{ModeIcon, PaletteIcon, ThemeConfig, Toasts, use_toast};
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_ui::screens::config::{Action, Effect, Sheet, State};

use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::navigation::menu::MenuOpen;
use crate::inbound::components::telemetry::vocabulary::{ProfileScreen, Screen};
use crate::inbound::components::theme_save::{self, ThemeSaveSheet, theme_config_of, theme_of};
use crate::inbound::components::ui_toast;

/// Whether the menu's Themes sheet is up. Provided at the app root, above the
/// router, so the palette in every screen's header and the sheet beside the
/// router share it.
#[derive(Debug, Clone, Copy)]
pub struct ThemeSheetOpen(pub Signal<bool>);

/// A theme model with the handles that perform its effects. Copy, so every
/// handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    authed: Authed,
    session: Signal<Option<Session>>,
    toast: Toasts,
    theme_config: Signal<ThemeConfig>,
    /// The menu's open state, folded as the sheet opens.
    menu_open: Signal<bool>,
    sheet_open: Signal<bool>,
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

    /// Performs one theme effect. The pair sends only theme actions, so the
    /// model never asks for a fetch, a hint, a pip change, the Exceptions
    /// sheet or a navigation here.
    fn perform(self, effect: Effect) {
        let Self {
            authed,
            session,
            toast,
            mut theme_config,
            mut menu_open,
            mut sheet_open,
            ..
        } = self;
        match effect {
            Effect::Save { ticket, change } => {
                theme_save::save(authed, session, ticket, change, move |action| {
                    self.dispatch(action);
                });
            }
            Effect::ShowTheme(theme) => theme_config.set(theme_config_of(theme)),
            Effect::ThemeSaved(theme) => theme_save::saved(theme_config, theme),
            Effect::OpenSheet(Sheet::Themes) => {
                menu_open.set(false);
                sheet_open.set(true);
            }
            // The sheet closes itself before it hands on a pick, so a close
            // that is already done writes nothing.
            Effect::CloseSheet(Sheet::Themes) => {
                if *sheet_open.peek() {
                    sheet_open.set(false);
                }
            }
            Effect::Toast(raised) => ui_toast::raise(toast, raised),
            Effect::OfferHint(_)
            | Effect::FetchSettings(_)
            | Effect::ShowPips(_)
            | Effect::PipsSaved(_)
            | Effect::SwipeConfirmationsSaved(_)
            | Effect::OpenSheet(Sheet::Exceptions)
            | Effect::CloseSheet(Sheet::Exceptions)
            | Effect::Navigate(_) => {}
        }
    }
}

/// A driver over a fresh model, with the app-wide menu and sheet state.
fn use_driver() -> Driver {
    Driver {
        model: use_signal(State::new),
        authed: use_authed(Screen::Profile(ProfileScreen::Settings)),
        session: use_context(),
        toast: use_toast(),
        theme_config: use_context(),
        menu_open: use_context::<MenuOpen>().0,
        sheet_open: use_context::<ThemeSheetOpen>().0,
    }
}

/// The menu's theme buttons, two footer icons: the palette folds the menu
/// and raises the Themes sheet, and the sun or moon names the mode in use and
/// flips it, saving at once, to the profile or, signed out, to the device.
#[component]
pub fn MenuTheme() -> Element {
    let theme_config: Signal<ThemeConfig> = use_context();
    let driver = use_driver();
    let is_dark = theme_config.read().is_dark;
    rsx! {
        button {
            class: "btn-action nav-icon",
            "aria-label": "Theme",
            onclick: move |_| driver.dispatch(Action::OpenThemes),
            PaletteIcon {}
        }
        button {
            class: "btn-action nav-icon",
            "aria-label": if is_dark { "Switch to light" } else { "Switch to dark" },
            onclick: move |_| {
                let shown = theme_of(theme_config.peek().clone());
                driver.dispatch(Action::ToggleDarkMode { shown });
            },
            ModeIcon { dark: is_dark }
        }
    }
}

/// The Themes sheet the menu's palette raises, mounted at the app root beside
/// the router. A pick saves as Settings' does.
#[component]
pub fn MenuThemeSheet() -> Element {
    let driver = use_driver();
    rsx! {
        // The kit styles the sheet's rows under this class.
        div { class: "theme-picker-sheet",
            ThemeSaveSheet {
                open: driver.sheet_open,
                on: move |action| driver.dispatch(action),
            }
        }
    }
}

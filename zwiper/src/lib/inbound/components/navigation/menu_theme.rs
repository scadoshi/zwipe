//! The main menu's theme buttons, after the footer's icons: a palette
//! that opens the Themes sheet and the mode's sun or moon, which flips
//! Dark/Light.
//!
//! The pair drives its own `zwipe_ui::screens::config` model and saves
//! through the same path as Settings' Theme row (see
//! [`theme_save`](crate::inbound::components::theme_save)), so the pair and
//! the row never disagree: both draw the app-wide theme and both persist to
//! the account's preferences. The sheet is the menu's descendant, so the menu
//! stays painted and folds as the sheet rises, the way the kit's
//! `ThemePicker` folds a site's menu.

use components::{ModeIcon, ThemeConfig, Toasts, use_toast};
use dioxus::prelude::*;
use zwipe_ui::screens::config::{Action, Effect, Sheet, State};

use crate::inbound::components::action_dock::{DockIcon, PALETTE};
use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::telemetry::vocabulary::{ProfileScreen, Screen};
use crate::inbound::components::theme_save::{self, ThemeSaveSheet, theme_config_of, theme_of};
use crate::inbound::components::ui_toast;

/// The pair's model with the handles that perform its theme effects. Copy, so
/// every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    authed: Authed,
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
            toast,
            mut theme_config,
            mut menu_open,
            mut sheet_open,
            ..
        } = self;
        match effect {
            Effect::Save { ticket, change } => {
                theme_save::save(authed, ticket, change, move |action| self.dispatch(action));
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
            | Effect::FetchPreferences(_)
            | Effect::ShowPips(_)
            | Effect::PipsSaved(_)
            | Effect::OpenSheet(Sheet::Exceptions)
            | Effect::CloseSheet(Sheet::Exceptions)
            | Effect::Navigate(_) => {}
        }
    }
}

/// The signed-in menu's theme buttons, two footer icons: the palette
/// opens the Themes sheet and folds the menu, and the sun or moon names the
/// mode in use and flips it, saving at once.
#[component]
pub fn MenuTheme(open: Signal<bool>) -> Element {
    let theme_config: Signal<ThemeConfig> = use_context();
    let driver = Driver {
        model: use_signal(State::new),
        authed: use_authed(Screen::Profile(ProfileScreen::Preferences)),
        toast: use_toast(),
        theme_config,
        menu_open: open,
        sheet_open: use_signal(|| false),
    };
    let is_dark = theme_config.read().is_dark;
    rsx! {
        button {
            class: "btn-action app-nav-icon",
            "aria-label": "Theme",
            onclick: move |_| driver.dispatch(Action::OpenThemes),
            DockIcon { path: PALETTE }
        }
        button {
            class: "btn-action app-nav-icon",
            "aria-label": if is_dark { "Switch to light" } else { "Switch to dark" },
            onclick: move |_| {
                let shown = theme_of(theme_config.peek().clone());
                driver.dispatch(Action::ToggleDarkMode { shown });
            },
            ModeIcon { dark: is_dark }
        }
        // The kit's class keeps the folding menu painted while the sheet is up.
        div { class: "theme-picker-sheet",
            ThemeSaveSheet {
                open: driver.sheet_open,
                on: move |action| driver.dispatch(action),
            }
        }
    }
}

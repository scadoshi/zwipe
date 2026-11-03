//! User preferences screen for theme and dark mode selection.

use components::{ThemeConfig, ThemeSheet};
use dioxus::prelude::*;
use zwipe_ui::screens::config::{Action, Theme};

/// The shared theme sheet, handing a kept pick to Config. Selections
/// live-preview against the whole app; Save hands the pick on as
/// [`Action::KeepTheme`], which persists it and toasts once the server
/// answers, and a discarded pick hands on [`Action::DiscardTheme`] once it has
/// wiped back out.
#[component]
pub fn PreferencesSheet(open: Signal<bool>, on: EventHandler<Action>) -> Element {
    let theme_config: Signal<ThemeConfig> = use_context();

    rsx! {
        ThemeSheet {
            open,
            theme: theme_config,
            on_save: move |picked: ThemeConfig| {
                on.call(Action::KeepTheme(Theme {
                    name: picked.name,
                    dark_mode: picked.is_dark,
                }));
            },
            on_unchanged: move |()| on.call(Action::DiscardTheme { picked: true }),
        }
    }
}

//! The App icon sheet: Match theme first, then every theme as a row, each with
//! the icon drawn in that theme's colors in the mode the icon is in now
//! (Profile's App icon row flips the mode), laid out like the theme sheet.

use crate::outbound::app_icon;
use components::{
    BottomSheet, Button, ButtonVariant, THEME_GROUPS, TOAST_NORMAL, TOAST_QUICK, ThemeConfig,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::{logo::Z, user::preferences::display_theme_name};

/// The tile: the mark as text in a theme's own class, so it is the same glyph
/// and colors `zcripts/ios/icon.py` renders, with nothing bundled to show it.
#[component]
fn IconTile(theme: String, mode: String) -> Element {
    rsx! {
        div { class: "app-icon-tile theme-{theme}-{mode}",
            pre { "{Z.trim_end()}" }
        }
    }
}

/// One selectable row: label on the left, the icon on the right. `pick` is
/// the draft it stands for, `None` for Match theme.
#[component]
fn IconRow(
    label: String,
    pick: Option<String>,
    tile: ThemeConfig,
    mut draft: Signal<Option<String>>,
) -> Element {
    let is_selected = draft() == pick;
    rsx! {
        button {
            class: if is_selected { "pref-row selected" } else { "pref-row" },
            onclick: move |_| draft.set(pick.clone()),
            div { class: "pref-row-inner",
                span { "{label}" }
                IconTile { theme: tile.name, mode: app_icon::mode(tile.is_dark) }
            }
        }
    }
}

/// Bottom sheet for picking the home-screen icon's theme, or Match theme. The
/// pick is held until Save, which switches the icon and writes it to `icon`
/// and `follows`.
#[component]
pub fn AppIconSheet(
    mut open: Signal<bool>,
    mut icon: Signal<ThemeConfig>,
    mut follows: Signal<bool>,
) -> Element {
    let toast = use_toast();
    let app_theme: Signal<ThemeConfig> = use_context();
    // `None` is Match theme.
    let saved = move || (!follows()).then(|| icon().name);
    let mut draft = use_signal(saved);

    // Every open starts from the icon actually showing.
    use_effect(move || {
        if open() {
            icon.set(app_icon::current());
            follows.set(app_icon::follows_theme());
            draft.set(saved());
        }
    });

    let discard = use_callback(move |()| {
        if *draft.peek() != saved() {
            toast.info(
                "App icon unchanged".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
        open.set(false);
    });

    let save = move |_| {
        let picked = draft();
        let (next, message) = match &picked {
            None => (app_theme(), "App icon matches your theme".to_string()),
            Some(name) => (
                ThemeConfig {
                    name: name.clone(),
                    is_dark: icon.peek().is_dark,
                },
                format!("App icon: {}", display_theme_name(name)),
            ),
        };
        let switched = app_icon::set_follows_theme(picked.is_none()).and_then(|()| {
            if app_icon::current() == next {
                Ok(())
            } else {
                app_icon::set(&next)
            }
        });
        match switched {
            Ok(()) => {
                toast.success(message, ToastOptions::default().duration(TOAST_NORMAL));
                icon.set(next);
                follows.set(picked.is_none());
            }
            Err(e) => {
                tracing::error!("failed to set app icon: {e}");
                toast.error(e.to_string(), ToastOptions::default());
            }
        }
        open.set(false);
    };

    let icon_mode = icon().is_dark;

    rsx! {
        BottomSheet {
            open,
            title: "App icon",
            on_dismiss: move |()| discard.call(()),
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| discard.call(()),
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Action,
                    disabled: draft() == saved(),
                    onclick: save,
                    "Save"
                }
            },
            IconRow { label: "Match theme", pick: None, tile: app_theme(), draft }
            for group in THEME_GROUPS {
                div { class: "pref-section-label", "{group.label}" }
                for t in group.themes.iter().copied() {
                    IconRow {
                        label: display_theme_name(t),
                        pick: Some(t.to_string()),
                        tile: ThemeConfig { name: t.to_string(), is_dark: icon_mode },
                        draft,
                    }
                }
            }
        }
    }
}

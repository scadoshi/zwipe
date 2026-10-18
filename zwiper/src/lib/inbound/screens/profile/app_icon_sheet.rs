//! The App icon sheet: every theme as a row, each with the icon drawn in that
//! theme's dark colors, laid out like the theme sheet.

use crate::outbound::app_icon;
use components::{
    ALLOWED_THEMES, BottomSheet, Button, ButtonVariant, COLORBLIND_THEMES, TOAST_NORMAL,
    TOAST_QUICK,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::{logo::Z, user::preferences::display_theme_name};

/// One selectable icon row: name on the left, the icon on the right. The tile
/// draws the mark as text in the theme's own class, so it is the same glyph
/// and colors `zcripts/ios/icon.py` renders, with nothing bundled to show it.
#[component]
fn IconRow(theme: String, mut draft: Signal<String>) -> Element {
    let is_selected = draft() == theme;
    let click_theme = theme.clone();
    rsx! {
        button {
            class: if is_selected { "pref-row selected" } else { "pref-row" },
            onclick: move |_| draft.set(click_theme.clone()),
            div { class: "pref-row-inner",
                span { "{display_theme_name(&theme)}" }
                div { class: "app-icon-tile theme-{theme}-dark",
                    pre { "{Z.trim_end()}" }
                }
            }
        }
    }
}

/// Bottom sheet for picking the home-screen icon. The pick is held until Save,
/// which switches the icon and writes the theme to `icon`.
#[component]
pub fn AppIconSheet(mut open: Signal<bool>, mut icon: Signal<String>) -> Element {
    let toast = use_toast();
    let mut draft = use_signal(|| icon.peek().clone());

    // Every open starts from the icon actually showing.
    use_effect(move || {
        if open() {
            let now = app_icon::current();
            icon.set(now.clone());
            draft.set(now);
        }
    });

    let discard = use_callback(move |()| {
        if *draft.peek() != *icon.peek() {
            toast.info(
                "App icon unchanged".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
        open.set(false);
    });

    let regular = ALLOWED_THEMES
        .iter()
        .copied()
        .filter(|t| !COLORBLIND_THEMES.contains(t));
    let colorblind = ALLOWED_THEMES
        .iter()
        .copied()
        .filter(|t| COLORBLIND_THEMES.contains(t));

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
                    disabled: draft() == icon(),
                    onclick: move |_| {
                        let picked = draft();
                        match app_icon::set(&picked) {
                            Ok(()) => {
                                toast.success(
                                    format!("App icon: {}", display_theme_name(&picked)),
                                    ToastOptions::default().duration(TOAST_NORMAL),
                                );
                                icon.set(picked);
                            }
                            Err(e) => {
                                tracing::error!("failed to set app icon: {e}");
                                toast.error(e.to_string(), ToastOptions::default());
                            }
                        }
                        open.set(false);
                    },
                    "Save"
                }
            },
            for t in regular {
                IconRow { theme: t.to_string(), draft }
            }
            div { class: "pref-section-label", "Color blind" }
            for t in colorblind {
                IconRow { theme: t.to_string(), draft }
            }
        }
    }
}

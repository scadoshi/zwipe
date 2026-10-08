//! The app's theme sheet: every allowed theme as a row, previewed live as it
//! is picked. Styled by `assets/app.css`.
//!
//! What a save means (an API call, a local store) and what each outcome says
//! (a toast) stay the host's, through [`ThemeSheet`]'s callbacks.

use dioxus::prelude::*;
use zwipe_core::domain::user::{
    models::theme::ThemeConfig,
    preferences::{ALLOWED_THEMES, COLORBLIND_THEMES, display_theme_name},
};

use crate::{BottomSheet, Button, ButtonVariant, ThemeFollow};

/// One selectable theme row: name on the left, swatch dots on the right. The
/// dots take their colors from the theme's own class, so colors stay defined
/// only in themes.css.
#[component]
fn ThemeRow(
    theme: String,
    mode: String,
    mut selected: Signal<String>,
    mut live: Signal<ThemeConfig>,
    dark: Signal<bool>,
) -> Element {
    let is_selected = selected() == theme;
    let click_theme = theme.clone();
    rsx! {
        button {
            class: if is_selected { "pref-row selected" } else { "pref-row" },
            onclick: move |_| {
                selected.set(click_theme.clone());
                live.set(ThemeConfig {
                    name: click_theme.clone(),
                    is_dark: dark(),
                });
            },
            div { class: "pref-row-inner",
                span { "{display_theme_name(&theme)}" }
                div { class: "theme-swatches theme-{theme}-{mode}",
                    span { class: "theme-dot", style: "background:var(--palette-1)" }
                    span { class: "theme-dot", style: "background:var(--palette-2)" }
                    span { class: "theme-dot", style: "background:var(--palette-3)" }
                    span { class: "theme-dot", style: "background:var(--palette-4)" }
                    span { class: "theme-dot", style: "background:var(--palette-5)" }
                    span { class: "theme-dot", style: "background:var(--palette-6)" }
                }
            }
        }
    }
}

/// Bottom sheet for picking a theme. A pick previews at once by setting
/// `theme`, the app's live theme. Save closes the sheet and hands the pick to
/// `on_save`, and is greyed until the pick differs from what the sheet opened
/// on. Back and the backdrop restore the theme that was active when the sheet
/// opened: the sheet holds still while that theme wipes back in, leaves
/// inside the wipe, and calls `on_unchanged` once the sweep is done.
///
/// Reads the [`ThemeFollow`] the host provides from
/// [`use_theme_follow`](crate::use_theme_follow), and registers with the
/// [`OverlayBackStack`](crate::OverlayBackStack) through [`BottomSheet`].
#[component]
pub fn ThemeSheet(
    mut open: Signal<bool>,
    /// The app's live theme. Each pick is written here.
    mut theme: Signal<ThemeConfig>,
    /// Called with the kept theme after Save closes the sheet.
    on_save: EventHandler<ThemeConfig>,
    /// Called once a discarded pick has wiped back out and the sheet is gone.
    on_unchanged: EventHandler<()>,
    /// A hint's open signal, for the sheet header's "?".
    #[props(default)]
    hint: Option<Signal<bool>>,
) -> Element {
    let follow: ThemeFollow = use_context();
    let mut original = use_signal(|| theme.peek().clone());
    let mut selected = use_signal(|| theme.peek().name.clone());
    let mut dark = use_signal(|| theme.peek().is_dark);

    // Snapshot the active theme each time the sheet opens and sync the
    // selection to it, so every open starts from the live theme.
    use_effect(move || {
        if open() {
            let current = theme.peek().clone();
            original.set(current.clone());
            selected.set(current.name.clone());
            dark.set(current.is_dark);
        }
    });

    // Back and the backdrop are the same act. With nothing to throw away the
    // sheet just slides off. With a pick on screen, the original wipes back
    // in and the sheet waits for it: sliding down under a wipe stutters, and
    // the old theme would otherwise look like it took.
    let mut restoring = use_signal(|| false);
    let discard = use_callback(move |()| {
        let original = original.peek().clone();
        let changed = *selected.peek() != original.name || *dark.peek() != original.is_dark;
        if changed {
            restoring.set(true);
            theme.set(original);
        } else {
            open.set(false);
        }
    });

    // Once the shell shows the original again the wipe has taken its new
    // snapshot's contents, so the sheet is pulled from the page here and the
    // sweep reveals the screen without it. The callback waits for the sweep.
    let hidden = use_memo(move || restoring() && *follow.shown.read() == *original.read());
    use_effect(move || {
        if restoring() && hidden() && !follow.wiping() {
            open.set(false);
            restoring.set(false);
            on_unchanged.call(());
        }
    });

    let unchanged = selected() == original().name && dark() == original().is_dark;
    let mode = (if dark() { "dark" } else { "light" }).to_string();
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
            title: "Themes".to_string(),
            hint,
            on_dismiss: move |()| discard.call(()),
            hidden: hidden(),
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| discard.call(()),
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    disabled: unchanged,
                    onclick: move |_| {
                        open.set(false);
                        on_save.call(ThemeConfig {
                            name: selected(),
                            is_dark: dark(),
                        });
                    },
                    "Save"
                }
            },
            for t in regular {
                ThemeRow {
                    theme: t.to_string(),
                    mode: mode.clone(),
                    selected,
                    live: theme,
                    dark,
                }
            }
            div { class: "pref-section-label", "Color blind" }
            for t in colorblind {
                ThemeRow {
                    theme: t.to_string(),
                    mode: mode.clone(),
                    selected,
                    live: theme,
                    dark,
                }
            }
        }
    }
}

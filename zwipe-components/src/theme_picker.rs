//! Site-style theme picker: a [`NavDropdown`] of every allowed theme
//! (color-blind themes grouped in their own bottom section) plus a dark/light
//! mode toggle.
//!
//! This is the canonical copy for every site; the host passes its `Signal<ThemeConfig>` in,
//! so how the theme is provided (context, prop drilling) and applied (body
//! class, wrapper div) stays the host's business.

use dioxus::prelude::*;
use zwipe_core::domain::user::{
    models::theme::ThemeConfig,
    preferences::{ALLOWED_THEMES, COLORBLIND_THEMES, display_theme_name},
};

use crate::{Button, ButtonVariant, NavDropdown};

/// A mode's icon, solid in the text color: the moon for dark, the sun for
/// light. Drawn as SVG rather than the ☀/☾ glyphs, which most fonts outline
/// and iOS turns into a color emoji.
#[component]
pub fn ModeIcon(dark: bool) -> Element {
    rsx! {
        svg {
            class: "mode-icon",
            view_box: "0 0 24 24",
            "aria-hidden": "true",
            if dark {
                path {
                    fill: "currentColor",
                    d: "M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z",
                }
            } else {
                circle { cx: "12", cy: "12", r: "5", fill: "currentColor" }
                g {
                    stroke: "currentColor",
                    stroke_width: "2.2",
                    stroke_linecap: "round",
                    line { x1: "12", y1: "1.5", x2: "12", y2: "3.5" }
                    line { x1: "12", y1: "20.5", x2: "12", y2: "22.5" }
                    line { x1: "1.5", y1: "12", x2: "3.5", y2: "12" }
                    line { x1: "20.5", y1: "12", x2: "22.5", y2: "12" }
                    line { x1: "4.6", y1: "4.6", x2: "6", y2: "6" }
                    line { x1: "18", y1: "18", x2: "19.4", y2: "19.4" }
                    line { x1: "4.6", y1: "19.4", x2: "6", y2: "18" }
                    line { x1: "18", y1: "6", x2: "19.4", y2: "4.6" }
                }
            }
        }
    }
}

/// A mode's icon followed by its name, "dark" or "light".
#[component]
fn ModeLabel(dark: bool) -> Element {
    rsx! {
        span { class: "mode-label",
            ModeIcon { dark }
            if dark { "dark" } else { "light" }
        }
    }
}

/// The dark/light toggle that sits in a settings screen's theme row, beside
/// its `Change` button. It names the mode in use; the host flips and saves
/// the mode in `onclick`.
#[component]
pub fn DarkModeButton(is_dark: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Util,
            onclick: move |evt| onclick.call(evt),
            ModeLabel { dark: is_dark }
        }
    }
}

/// Theme dropdown + dark/light toggle. Every theme has both modes, so the
/// toggle is always shown.
#[component]
pub fn ThemePicker(
    theme: Signal<ThemeConfig>,
    /// What the picker shows as current (its label, the active row, the mode
    /// toggle) when that should lag behind `theme`, as during a wipe. Defaults
    /// to `theme`.
    #[props(default)]
    shown: Option<Signal<ThemeConfig>>,
    /// The menu's open state, when the host wants it: a nav with menus of its
    /// own passes one so it can keep a single menu open at a time. Defaults to
    /// the picker's own.
    #[props(default)]
    open: Option<Signal<bool>>,
) -> Element {
    let mut theme = theme;
    // Always taken, so the hook order holds whether or not the host passes one.
    let own_open = use_signal(|| false);
    let mut open = open.unwrap_or(own_open);
    let shown = shown.unwrap_or(theme);
    let current = shown.read().name.clone();
    let is_dark = shown.read().is_dark;
    let mode = if is_dark { "dark" } else { "light" };
    // ALLOWED_THEMES is already alphabetical; filtering preserves that order
    // for the main group and pulls the color-blind themes into a bottom section.
    let regular_themes = ALLOWED_THEMES
        .iter()
        .copied()
        .filter(|t| !COLORBLIND_THEMES.contains(t));
    let colorblind_themes = ALLOWED_THEMES
        .iter()
        .copied()
        .filter(|t| COLORBLIND_THEMES.contains(t));

    rsx! {
        // Its children glide aside when the label changes width (NAV_GLIDE_JS).
        div { class: "theme-switcher", "data-nav-glide": "true",
            NavDropdown {
                open,
                label: display_theme_name(&current),
                div { class: "nav-dropdown-label", "Themes" }
                for name in regular_themes {
                    button {
                        class: if current == *name { "nav-dropdown-item active" } else { "nav-dropdown-item" },
                        onclick: move |_| {
                            let dark = theme.read().is_dark;
                            theme.set(ThemeConfig {
                                name: name.to_string(),
                                is_dark: dark,
                            });
                            open.set(false);
                        },
                        span { "{display_theme_name(name)}" }
                        ThemeDots { name, mode }
                    }
                }
                div { class: "nav-dropdown-label", "Color blind" }
                for name in colorblind_themes {
                    button {
                        class: if current == *name { "nav-dropdown-item active" } else { "nav-dropdown-item" },
                        onclick: move |_| {
                            let dark = theme.read().is_dark;
                            theme.set(ThemeConfig {
                                name: name.to_string(),
                                is_dark: dark,
                            });
                            open.set(false);
                        },
                        span { "{display_theme_name(name)}" }
                        ThemeDots { name, mode }
                    }
                }
            }
            button {
                class: "mode-toggle",
                // It names the mode in use, as the apps' settings row does;
                // the tooltip names the one a click switches to.
                title: if is_dark { "Switch to light" } else { "Switch to dark" },
                onclick: move |_| {
                    let current = theme.read().clone();
                    theme.set(ThemeConfig {
                        name: current.name,
                        is_dark: !current.is_dark,
                    });
                },
                ModeLabel { dark: is_dark }
            }
        }
    }
}

/// A theme's colors as a strip of dots: background, text, the three accents
/// and the error color. The strip carries the theme's own class, so the dots
/// read its CSS variables and the colors stay defined only in themes.css.
#[component]
fn ThemeDots(name: &'static str, mode: &'static str) -> Element {
    rsx! {
        span { class: "theme-swatches theme-{name}-{mode}",
            span { class: "theme-dot", style: "background:var(--palette-1)" }
            span { class: "theme-dot", style: "background:var(--palette-2)" }
            span { class: "theme-dot", style: "background:var(--palette-3)" }
            span { class: "theme-dot", style: "background:var(--palette-4)" }
            span { class: "theme-dot", style: "background:var(--palette-5)" }
            span { class: "theme-dot", style: "background:var(--palette-6)" }
        }
    }
}

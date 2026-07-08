//! Shared button component.
//!
//! The single source for the app's push buttons — the full-width form/dialog
//! `.btn`, the compact bar `.util-btn`, and the small `.btn-xs` — so every
//! button looks and behaves the same across `zwiper` and `zite`. Styling lives
//! in `assets/components.css` (copied into each app's bundle at build time).

use dioxus::prelude::*;

/// Which button style to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// Full-width form/dialog button (`.btn`).
    #[default]
    Primary,
    /// Small inline button (`.btn-xs`).
    Small,
    /// Compact action-bar button (`.util-btn`).
    Util,
}

impl ButtonVariant {
    /// Base CSS class for the variant.
    fn base_class(self) -> &'static str {
        match self {
            ButtonVariant::Primary => "btn",
            ButtonVariant::Small => "btn-xs",
            ButtonVariant::Util => "util-btn",
        }
    }

    /// Danger modifier class for the variant (destructive actions).
    fn danger_class(self) -> &'static str {
        match self {
            ButtonVariant::Util => "util-btn-danger",
            // `.btn` and `.btn-xs` share the same danger treatment.
            ButtonVariant::Primary | ButtonVariant::Small => "btn-danger",
        }
    }
}

/// A push button.
///
/// `variant` picks the style, `danger` applies the destructive treatment,
/// `disabled` greys it out. `class` appends extra classes (e.g. an animation
/// or a one-off layout modifier) and `style` covers the rare inline-styled
/// site — both keep the handful of special call sites migratable without a new
/// prop each.
#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default = false)] danger: bool,
    #[props(default = false)] disabled: bool,
    #[props(default)] class: Option<String>,
    #[props(default)] style: Option<String>,
    onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let mut full = String::from(variant.base_class());
    if danger {
        full.push(' ');
        full.push_str(variant.danger_class());
    }
    if let Some(extra) = &class {
        full.push(' ');
        full.push_str(extra);
    }

    rsx! {
        button {
            class: "{full}",
            disabled,
            style: style.unwrap_or_default(),
            onclick: move |evt| onclick.call(evt),
            {children}
        }
    }
}

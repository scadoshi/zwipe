//! Push buttons: the full-width `.btn`, the compact bar `.util-btn`, and the
//! small `.btn-xs`.

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

    /// Modifier class for destructive actions.
    fn danger_class(self) -> &'static str {
        match self {
            ButtonVariant::Util => "util-btn-danger",
            ButtonVariant::Primary | ButtonVariant::Small => "btn-danger",
        }
    }
}

/// A push button. `class` appends extra classes; `style` is for the rare
/// inline-styled call site.
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

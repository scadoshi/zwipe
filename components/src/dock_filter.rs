//! The action dock's filter button.

use dioxus::prelude::*;

use crate::icons::{DockIcon, FUNNEL};

/// The dock's filter button: the funnel, its dot while a filter or a sort is
/// on, and, when `on_reset` is given and `active`, a small reset on its corner.
#[component]
pub fn DockFilter(
    label: &'static str,
    dot: bool,
    #[props(default)] active: bool,
    onclick: EventHandler<()>,
    #[props(default)] on_reset: Option<EventHandler<()>>,
    #[props(default)] reset_label: &'static str,
) -> Element {
    rsx! {
        div { class: "action-dock-group",
            button {
                class: "action-dock-btn",
                "aria-label": label,
                onclick: move |_| onclick.call(()),
                DockIcon { path: FUNNEL }
                if dot {
                    span { class: "filter-dot" }
                }
            }
            if let (true, Some(on_reset)) = (active, on_reset) {
                button {
                    class: "action-dock-reset",
                    "aria-label": reset_label,
                    onclick: move |_| on_reset.call(()),
                    "\u{d7}"
                }
            }
        }
    }
}

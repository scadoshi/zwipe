//! The action dock: a screen's icon actions in one floating squircle bar at
//! the bottom right, on the kit's `.action-dock` styles. The icons and their
//! paths are [`components::icons`], re-exported here for the screens.

pub use components::icons::{
    ARROW, CHECK, COG, COPY, DockIcon, ERASER, EyeIcon, FUNNEL, HOUSE, INFO, LINK, MINUS, PALETTE,
    PERSON, PLUS, REFRESH, SEND, UNDO,
};
use dioxus::prelude::*;

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

//! The action dock: a screen's icon actions in one floating squircle bar at
//! the bottom right. Styled by the "action dock" block of `main.css`.

use dioxus::prelude::*;

/// The add icon: a plus.
pub const PLUS: &str = "M12 5v14M5 12h14";
/// The remove icon: a minus.
pub const MINUS: &str = "M5 12h14";
/// The filter icon: a funnel.
pub const FUNNEL: &str = "M4 5h16l-6 7.5V19l-4 1v-7.5z";
/// The undo icon: an arrow turning back.
pub const UNDO: &str = "M9 14L4 9l5-5M4 9h10a6 6 0 0 1 0 12h-3";
/// The refresh icon: an arrow circling round.
pub const REFRESH: &str = "M20 12a8 8 0 1 1-2.34-5.66M20 4v5h-5";

/// The save icon: a checkmark.
pub const CHECK: &str = "M5 12.5l4.5 4.5L19 7";
/// The synergy icon: two chain links.
pub const LINK: &str = "M10 14a4 4 0 0 0 5.66 0l3-3a4 4 0 0 0-5.66-5.66l-1 1M14 10a4 4 0 0 0-5.66 0l-3 3a4 4 0 0 0 5.66 5.66l1-1";

/// One stroke icon for a dock button, drawn from a single SVG path.
#[component]
pub fn DockIcon(path: &'static str) -> Element {
    rsx! {
        svg {
            class: "icon-pill-icon",
            view_box: "0 0 24 24",
            xmlns: "http://www.w3.org/2000/svg",
            "aria-hidden": "true",
            path { d: path }
        }
    }
}

/// The card details icon: an eye.
#[component]
pub fn EyeIcon() -> Element {
    rsx! {
        svg {
            class: "icon-pill-icon",
            view_box: "0 0 24 24",
            xmlns: "http://www.w3.org/2000/svg",
            "aria-hidden": "true",
            path { d: "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" }
            circle { cx: "12", cy: "12", r: "3" }
        }
    }
}

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
        div { class: "icon-pill-filter",
            button {
                class: "icon-pill-btn",
                "aria-label": label,
                onclick: move |_| onclick.call(()),
                DockIcon { path: FUNNEL }
                if dot {
                    span { class: "filter-dot" }
                }
            }
            if let (true, Some(on_reset)) = (active, on_reset) {
                button {
                    class: "icon-pill-reset",
                    "aria-label": reset_label,
                    onclick: move |_| on_reset.call(()),
                    "\u{d7}"
                }
            }
        }
    }
}

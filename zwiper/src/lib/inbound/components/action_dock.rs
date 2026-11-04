//! The action dock: a screen's icon actions in one floating squircle bar at
//! the bottom right, on the kit's `.action-dock` styles.

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
/// The copy icon: two stacked sheets.
pub const COPY: &str = "M9 9h10v11H9zM5 15V4h10";
/// The go icon: an arrow pointing right, for Log in and Create profile.
pub const ARROW: &str = "M5 12h14M13 6l6 6-6 6";
/// The send icon: a paper plane.
pub const SEND: &str = "M22 2L11 13M22 2l-7 20-4-9-9-4z";
/// The Home icon: a house.
pub const HOUSE: &str = "M3 11.5l9-8 9 8M5 10v10h5v-6h4v6h5V10";
/// The Account icon: a person.
pub const PERSON: &str = "M20 21a8 8 0 0 0-16 0M12 4a4 4 0 1 0 0 8 4 4 0 1 0 0-8";
/// The Settings icon: a cog, an eight-toothed gear around a hole.
pub const COG: &str = "M19.4 10.3L21.9 10.7L21.9 13.3L19.4 13.7L18.4 16.1L19.9 18.1L18.1 19.9L16.1 18.4L13.7 19.4L13.3 21.9L10.7 21.9L10.3 19.4L7.9 18.4L5.9 19.9L4.1 18.1L5.6 16.1L4.6 13.7L2.1 13.3L2.1 10.7L4.6 10.3L5.6 7.9L4.1 5.9L5.9 4.1L7.9 5.6L10.3 4.6L10.7 2.1L13.3 2.1L13.7 4.6L16.1 5.6L18.1 4.1L19.9 5.9L18.4 7.9zM12 8.8a3.2 3.2 0 1 0 0 6.4 3.2 3.2 0 1 0 0-6.4";
/// The About icon: a circled i.
pub const INFO: &str = "M12 3a9 9 0 1 0 0 18 9 9 0 1 0 0-18M12 11v5M12 8v0.01";
/// A palette with four paint dabs, the main menu's Theme button.
pub const PALETTE: &str = "M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.93 0 1.65-.75 1.65-1.69 0-.44-.18-.84-.44-1.13-.29-.29-.44-.65-.44-1.12a1.64 1.64 0 0 1 1.67-1.67h2c3.05 0 5.55-2.5 5.55-5.55C21.97 6.01 17.46 2 12 2ZM13.5 6.5v.01M17.5 10.5v.01M8.5 7.5v.01M6.5 12.5v.01";
/// The synergy icon: two chain links.
pub const LINK: &str = "M10 14a4 4 0 0 0 5.66 0l3-3a4 4 0 0 0-5.66-5.66l-1 1M14 10a4 4 0 0 0-5.66 0l-3 3a4 4 0 0 0 5.66 5.66l1-1";

/// One stroke icon for a dock button, drawn from a single SVG path.
#[component]
pub fn DockIcon(path: &'static str) -> Element {
    rsx! {
        svg {
            class: "action-dock-icon",
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
            class: "action-dock-icon",
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

//! Shared screen header.
//!
//! Every screen opens with the same bar, the kit's `NavBar` folded behind
//! its hamburger: the Z mark on the left goes Home, the screen's title and,
//! on screens with a hint dialog, its `?` sit in the middle, and the hamburger
//! on the right opens the [main menu](crate::inbound::components::navigation::menu).
//! The screen still owns its `HintDialog` (with screen-specific content);
//! passing its `open` signal as `hint` wires up the `?`.

use components::{InfoButton, NavBar};
use dioxus::prelude::*;
use zwipe_core::domain::logo;

use crate::inbound::components::navigation::menu::{MenuFooter, MenuLinks};
use crate::inbound::router::Router;

/// The header bar with the main menu, titled `title`.
///
/// Omit `hint` for screens without a hint dialog (the `?` isn't rendered).
#[component]
pub fn ScreenHeader(title: String, hint: Option<Signal<bool>>) -> Element {
    let route: Router = use_route();
    let mut open = use_signal(|| false);

    // A new route closes the menu.
    use_effect(use_reactive!(|route| {
        let _ = route;
        open.set(false);
    }));

    rsx! {
        div { class: "app-nav",
            NavBar {
                open,
                brand: rsx! {
                    Link {
                        to: Router::Home {},
                        class: "nav-brand",
                        "aria-label": "Home",
                        onclick: move |_| open.set(false),
                        span { class: "nav-logo app-nav-logo", "{logo::Z}" }
                    }
                },
                persistent: rsx! {
                    div { class: "app-nav-title",
                        h2 { class: "app-nav-title-text", "{title}" }
                        if let Some(mut hint) = hint {
                            InfoButton { onclick: move |_| hint.set(true) }
                        }
                    }
                },
                links: rsx! {
                    MenuLinks { open, route }
                },
                trailing: rsx! {
                    MenuFooter { open }
                },
            }
        }
    }
}

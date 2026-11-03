//! Shared screen header.
//!
//! Every screen opens with the same bar, the kit's `NavBar` folded behind
//! its hamburger: the Z mark on the left goes Home (a back arrow instead on a
//! screen below a section), the screen's title and,
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
/// Pass `back` on a screen below a section: the left corner then shows a back
/// arrow that calls it, where the Z mark sits elsewhere.
#[component]
pub fn ScreenHeader(
    title: String,
    hint: Option<Signal<bool>>,
    back: Option<Callback<()>>,
) -> Element {
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
                    if let Some(back) = back {
                        button {
                            class: "nav-brand app-nav-back",
                            "aria-label": "Back",
                            onclick: move |_| {
                                open.set(false);
                                back.call(());
                            },
                            BackArrow {}
                        }
                    } else {
                        Link {
                            to: Router::Home {},
                            class: "nav-brand",
                            "aria-label": "Home",
                            onclick: move |_| open.set(false),
                            span { class: "nav-logo app-nav-logo", "{logo::Z}" }
                        }
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

/// The back arrow: a left chevron drawn in the text color.
#[component]
fn BackArrow() -> Element {
    rsx! {
        svg {
            class: "app-nav-back-icon",
            view_box: "0 0 24 24",
            xmlns: "http://www.w3.org/2000/svg",
            "aria-hidden": "true",
            path { d: "M15 5 8 12l7 7" }
        }
    }
}

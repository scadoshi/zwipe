//! The page for a path no route matches: zite's dead end, under the nav
//! the user would see anyway, so the way on is right there. A shared deck's
//! link that no longer opens a deck ends on the same card in its own words.

use components::Panel;
use dioxus::prelude::*;

use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;

/// A path no route matches, said as zite says it.
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let _ = segments;
    rsx! {
        ScreenFrame { title: "Page not found",
            div { class: "screen-content",
                DeadEnd {
                    title: "Page not found",
                    line: "Nothing lives at this address. It may have moved with a site update, or the link may be incomplete.",
                }
            }
        }
    }
}

/// A dead end: a small centered card with `title` pulsing in the error color,
/// one `line` of copy and a Home button.
#[component]
pub(crate) fn DeadEnd(title: &'static str, line: &'static str) -> Element {
    rsx! {
        div { class: "web-dead-end content-enter",
            Panel {
                title,
                title_h1: true,
                actions: rsx! {
                    Link { class: "panel-action", to: Route::Home {}, "Home" }
                },
                p { class: "web-dead-end-line", "{line}" }
            }
        }
    }
}

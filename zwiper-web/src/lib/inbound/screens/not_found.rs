//! The page for a path no route matches: zite's dead end, under the nav
//! the user would see anyway, so the way on is right there.

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
                div { class: "web-dead-end content-enter",
                    Panel {
                        title: "Page not found",
                        title_h1: true,
                        actions: rsx! {
                            Link { class: "panel-action", to: Route::Home {}, "Home" }
                        },
                        p { class: "web-dead-end-line",
                            "Nothing lives at this address. It may have moved with a site update, or the link may be incomplete."
                        }
                    }
                }
            }
        }
    }
}

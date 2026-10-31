//! The page for a path no route matches.

use components::Panel;
use dioxus::prelude::*;

/// A path no route matches.
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let path = segments.join("/");
    rsx! {
        div { class: "web-public",
            div { class: "web-placeholder content-enter",
                Panel { title: "Not found",
                    p { "Nothing lives at /{path}." }
                    Link { to: "/", "Home" }
                }
            }
        }
    }
}

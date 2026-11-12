//! The back arrow: the app's left chevron in a small soft box, for the
//! folded nav's left corner and an overlay's header.

use dioxus::prelude::*;

/// A back button boxed like the hamburger, drawn with the app's
/// `.app-nav-back` rule. `class` adds the host's own, such as where it sits
/// in a header.
#[component]
pub fn BackArrow(onclick: EventHandler<()>, #[props(default)] class: Option<String>) -> Element {
    let class = class.map_or_else(
        || "nav-brand app-nav-back".to_owned(),
        |extra| format!("nav-brand app-nav-back {extra}"),
    );
    rsx! {
        button {
            class,
            "aria-label": "Back",
            onclick: move |_| onclick.call(()),
            svg {
                class: "app-nav-back-icon",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                "aria-hidden": "true",
                path { d: "M15 5 8 12l7 7" }
            }
        }
    }
}

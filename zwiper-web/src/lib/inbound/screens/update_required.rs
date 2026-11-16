//! The blocking screen for a build below the server's minimum.

use dioxus::prelude::*;

/// Shown in place of the router while the gate is closed. The app's markup
/// and copy, except the last button: the app links to its store, and a web
/// build updates by loading the page again.
#[component]
pub fn UpdateRequired() -> Element {
    rsx! {
        div { class: "screen web-screen",
            div { class: "screen-content centered content-enter",
                div { class: "container-sm text-center",
                    div { class: "card", style: "cursor: default;",
                        span {
                            class: "settings-list-title update-required-title",
                            "Update required"
                        }
                        hr { class: "box-rule", style: "margin-left: calc(-1 * var(--space-4)); margin-right: calc(-1 * var(--space-4));" }
                        p { class: "text-muted",
                            "This version of "
                            strong { style: "color: var(--palette-3);", "Zwipe" }
                            " is no longer supported. "
                            strong { style: "color: var(--accent-primary);", "Update to keep building." }
                        }
                    }
                    button {
                        class: "btn",
                        style: "display: block; width: 100%; margin-top: var(--space-4);",
                        onclick: move |_| {
                            document::eval("window.location.reload()");
                        },
                        "Reload"
                    }
                }
            }
        }
    }
}

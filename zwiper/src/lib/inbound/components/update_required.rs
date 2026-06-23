//! Full-screen blocking view shown when this build is below the server's
//! minimum supported version. Rendered by the root component in place of the
//! router — deliberately no dismiss affordance.

use dioxus::prelude::*;

// Both platforms route through zwipe.net/download/* so the destination is
// controlled from the site (a push) and never needs an app update. zite handles
// the actual redirect: ios -> App Store; android -> Play (a "pending" landing
// page until the public listing exists).
#[cfg(target_os = "android")]
const STORE_URL: &str = "https://zwipe.net/download/android";
#[cfg(target_os = "android")]
const STORE_LABEL: &str = "Open Play Store";

#[cfg(not(target_os = "android"))]
const STORE_URL: &str = "https://zwipe.net/download/ios";
#[cfg(not(target_os = "android"))]
const STORE_LABEL: &str = "Open App Store";

/// Blocking "Update required" screen with a link to the App Store.
#[component]
pub fn UpdateRequired() -> Element {
    rsx! {
        div { class: "screen",
            div { class: "page-header", h2 { "Update required" } }
            div { class: "screen-content centered content-enter",
                div { class: "container-sm text-center",
                    div { class: "card", style: "cursor: default;",
                        span {
                            class: "card-title update-required-title",
                            style: "display: block; text-transform: uppercase; color: #ff3030;",
                            "Update required"
                        }
                        hr { class: "box-rule", style: "margin-left: -1rem; margin-right: -1rem;" }
                        p { class: "text-muted",
                            "This version of Zwipe is no longer supported. Update to keep building."
                        }
                    }
                    a {
                        class: "btn",
                        href: "{STORE_URL}",
                        target: "_blank",
                        style: "text-decoration: none; display: block; margin-top: 1rem;",
                        "{STORE_LABEL}"
                    }
                }
            }
            div { class: "util-bar" }
        }
    }
}

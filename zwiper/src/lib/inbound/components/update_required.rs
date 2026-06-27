//! Full-screen blocking view shown when this build is below the server's
//! minimum supported version. Rendered by the root component in place of the
//! router — deliberately no dismiss affordance.

use dioxus::prelude::*;

// Both platforms route through the site's /download/* so the destination is
// controlled from the site (a push) and never needs an app update. zite handles
// the actual redirect: ios -> App Store; android -> Play (a "pending" landing
// page until the public listing exists). The domain is centralized here so a
// future domain change is a single edit.
const WEB_DOMAIN: &str = "https://zwipe.net";

#[cfg(target_os = "android")]
const STORE_PATH: &str = "/download/android";
#[cfg(target_os = "android")]
const STORE_LABEL: &str = "Open Play Store";

#[cfg(not(target_os = "android"))]
const STORE_PATH: &str = "/download/ios";
#[cfg(not(target_os = "android"))]
const STORE_LABEL: &str = "Open App Store";

/// Blocking "Update required" screen with a link to the App Store.
#[component]
pub fn UpdateRequired() -> Element {
    rsx! {
        div { class: "screen",
            // Empty header bar — frames the top symmetrically with the footer.
            // The red card title below is the single "Update required" headline.
            div { class: "page-header" }
            div { class: "screen-content centered content-enter",
                div { class: "container-sm text-center",
                    div { class: "card", style: "cursor: default;",
                        span {
                            class: "card-title update-required-title",
                            style: "display: block; color: #ff3030; font-weight: bold; text-transform: uppercase;",
                            "Update required"
                        }
                        hr { class: "box-rule", style: "margin-left: -1rem; margin-right: -1rem;" }
                        p { class: "text-muted",
                            "This version of "
                            strong { style: "color: var(--accent-tertiary);", "Zwipe" }
                            " is no longer supported. "
                            strong { style: "color: var(--accent-primary);", "Update to keep building." }
                        }
                    }
                    a {
                        class: "btn",
                        href: "{WEB_DOMAIN}{STORE_PATH}",
                        target: "_blank",
                        style: "text-decoration: none; display: block; margin-top: 1rem;",
                        "{STORE_LABEL} ↗"
                    }
                }
            }
            div { class: "util-bar" }
        }
    }
}

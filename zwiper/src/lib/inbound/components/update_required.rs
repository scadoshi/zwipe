//! Full-screen blocking view shown when this build is below the server's
//! minimum supported version. Rendered by the root component in place of the
//! router — deliberately no dismiss affordance.

use dioxus::prelude::*;

/// App Store listing for Zwipe TCG.
const APP_STORE_URL: &str = "https://apps.apple.com/us/app/zwipe-tcg/id6761341603";

/// Blocking "Update required" screen with a link to the App Store.
#[component]
pub fn UpdateRequired() -> Element {
    rsx! {
        div { class: "screen",
            div { class: "screen-content centered content-enter",
                div { class: "container-sm", style: "text-align: center;",
                    h2 { class: "font-light tracking-wide update-required-title", "Update required" }
                    p { class: "text-muted",
                        "This version of Zwipe is no longer supported. Update to keep building."
                    }
                    a {
                        class: "btn",
                        href: "{APP_STORE_URL}",
                        target: "_blank",
                        style: "text-decoration: none; text-align: center; display: block; margin-top: 2rem;",
                        "Open App Store"
                    }
                }
            }
        }
    }
}

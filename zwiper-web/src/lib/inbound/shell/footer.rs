//! The footer under every screen, zite's: the copyright line with the app's
//! links, then the Scryfall credit and the Fan Content notice.

use dioxus::prelude::*;
use zwipe_core::domain::site::{SUPPORT_EMAIL, WEB_BASE};

use crate::inbound::router::Route;

/// The Scryfall credit's link.
const SCRYFALL_URL: &str = "https://scryfall.com";
/// The Fan Content Policy's link.
const FAN_CONTENT_URL: &str = "https://company.wizards.com/en/legal/fancontentpolicy";

/// The footer. Signed in, `Privacy policy` and `Changelog` are the app's
/// own screens; signed out they are zwipe.net's, in a new tab, since the
/// app's need a session. Every other outside link opens a new tab too.
#[component]
pub fn SiteFooter(signed_in: bool) -> Element {
    rsx! {
        footer { class: "site-footer web-footer",
            p {
                "\u{a9} 2026 Zwipe | "
                if signed_in {
                    Link { to: Route::PrivacyPolicy {}, "Privacy policy" }
                    " | "
                    Link { to: Route::Changelog {}, "Changelog" }
                } else {
                    OutLink { href: format!("{WEB_BASE}/privacy"), label: "Privacy policy" }
                    " | "
                    OutLink { href: format!("{WEB_BASE}/changelog"), label: "Changelog" }
                }
                " | "
                OutLink { href: format!("{WEB_BASE}/guides"), label: "Guides" }
                " | "
                OutLink { href: format!("{WEB_BASE}/community"), label: "Community" }
                " | "
                a { href: "mailto:{SUPPORT_EMAIL}", "Support" }
                " | "
                OutLink { href: WEB_BASE.to_string(), label: "zwipe.net" }
            }
            p { class: "site-footer-note",
                "Card data and images from "
                OutLink { href: SCRYFALL_URL.to_string(), label: "Scryfall", arrow: false }
                "."
            }
            p { class: "site-footer-note",
                "Zwipe is unofficial Fan Content permitted under the "
                OutLink { href: FAN_CONTENT_URL.to_string(), label: "Fan Content Policy", arrow: false }
                ". Not approved/endorsed by Wizards. Portions of the materials used are property "
                "of Wizards of the Coast. \u{a9}Wizards of the Coast LLC."
            }
        }
    }
}

/// A link that opens a new tab, with zite's `↗` unless `arrow` is off.
#[component]
fn OutLink(href: String, label: &'static str, #[props(default = true)] arrow: bool) -> Element {
    rsx! {
        a { href, target: "_blank", rel: "noopener noreferrer",
            "{label}"
            if arrow {
                span { class: "web-footer-ext", " \u{2197}" }
            }
        }
    }
}

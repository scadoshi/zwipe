//! zwipe.net's footer, which the web app draws too.

use dioxus::prelude::*;

/// The year the footer's copyright names.
const COPYRIGHT_YEAR: u16 = 2026;

/// The footer: the copyright and `children` (the host's links, as
/// `nav-cta` pills) in one centered row, then the Scryfall credit and the
/// Fan Content notice as one note across the footer's width. `class` adds
/// the host's own classes to the footer.
#[component]
pub fn SiteFooter(#[props(default)] class: Option<String>, children: Element) -> Element {
    let class = class.map_or_else(
        || "site-footer content-enter-delayed".to_owned(),
        |extra| format!("site-footer content-enter-delayed {extra}"),
    );
    rsx! {
        footer { class,
            div { class: "site-footer-row",
                p { class: "site-footer-copyright", "© {COPYRIGHT_YEAR} Zwipe" }
                div { class: "footer-links", {children} }
            }
            p { class: "site-footer-note",
                "Card data and images from "
                a { href: "https://scryfall.com", "Scryfall" }
                ". Zwipe is unofficial Fan Content permitted under the "
                a {
                    href: "https://company.wizards.com/en/legal/fancontentpolicy",
                    "Fan Content Policy"
                }
                ". Not approved/endorsed by Wizards. Portions of the materials used are property "
                "of Wizards of the Coast. ©Wizards of the Coast LLC."
            }
        }
    }
}

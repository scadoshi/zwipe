use dioxus::prelude::*;
use crate::WEB_BASE;

/// Per-route metadata: page title, description, canonical URL, and OG/Twitter
/// tags. Rendered into `<head>` by Dioxus's document API. Picked up by SSG so
/// the prerendered HTML carries these tags before any JS runs — which is the
/// whole point of the SSG pass for SEO and link unfurling.
///
/// `path` should start with `/` and is appended to [`WEB_BASE`] for
/// the canonical and og:url. Pass the empty string for the home page.
#[component]
pub fn PageMeta(title: String, description: String, path: String) -> Element {
    let canonical = format!("{WEB_BASE}{path}");
    let og_image = format!("{WEB_BASE}/assets/og-default.png");
    let full_title = if title == "Zwipe" {
        "Zwipe".to_string()
    } else {
        format!("{title} | Zwipe")
    };

    rsx! {
        document::Title { "{full_title}" }
        document::Meta { name: "description", content: "{description}" }
        document::Link { rel: "canonical", href: "{canonical}" }

        // Open Graph
        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:site_name", content: "Zwipe" }
        document::Meta { property: "og:title", content: "{full_title}" }
        document::Meta { property: "og:description", content: "{description}" }
        document::Meta { property: "og:url", content: "{canonical}" }
        document::Meta { property: "og:image", content: "{og_image}" }

        // Twitter / X
        document::Meta { name: "twitter:card", content: "summary_large_image" }
        document::Meta { name: "twitter:title", content: "{full_title}" }
        document::Meta { name: "twitter:description", content: "{description}" }
        document::Meta { name: "twitter:image", content: "{og_image}" }
    }
}

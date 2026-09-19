//! Per-route `<head>` metadata: page title, description, canonical URL, and
//! OG/Twitter tags. Rendered via Dioxus's document API and picked up by SSG so
//! the prerendered HTML carries these tags before any JS runs — the point of
//! the SSG pass for SEO and link unfurling.

use dioxus::prelude::*;

/// Site-wide constants a host plugs into [`PageMeta`]. Each site defines one
/// (typically a `const`) and wraps the component so its pages never repeat it.
#[derive(Clone, PartialEq)]
pub struct SiteMeta {
    /// Public base URL, no trailing slash (e.g. `https://zwipe.net`).
    pub base_url: &'static str,
    /// Brand name: the `og:site_name` and the `<title>` suffix.
    pub site_name: &'static str,
    /// Site-wide OG image path (e.g. `/assets/og-default.png`), if the site
    /// has one. Also decides the Twitter card type: `summary_large_image`
    /// with an image, `summary` without.
    pub og_image_path: Option<&'static str>,
}

/// Head metadata for one page.
///
/// `path` should start with `/` and is appended to the site's base URL for
/// the canonical and og:url; pass the empty string for the home page. Pages
/// pass a descriptive, keyword-bearing title and get the ` | {site_name}`
/// brand suffix appended — except a title that *is* the bare site name (a
/// home page introducing the brand itself), which renders unsuffixed.
#[component]
pub fn PageMeta(site: SiteMeta, title: String, description: String, path: String) -> Element {
    let canonical = format!("{}{path}", site.base_url);
    let full_title = if title == site.site_name {
        title
    } else {
        format!("{title} | {}", site.site_name)
    };
    let og_image = site.og_image_path.map(|p| format!("{}{p}", site.base_url));
    let twitter_card = if og_image.is_some() {
        "summary_large_image"
    } else {
        "summary"
    };

    rsx! {
        document::Title { "{full_title}" }
        document::Meta { name: "description", content: "{description}" }
        document::Link { rel: "canonical", href: "{canonical}" }

        // Open Graph
        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:site_name", content: "{site.site_name}" }
        document::Meta { property: "og:title", content: "{full_title}" }
        document::Meta { property: "og:description", content: "{description}" }
        document::Meta { property: "og:url", content: "{canonical}" }
        if let Some(image) = og_image.clone() {
            document::Meta { property: "og:image", content: "{image}" }
        }

        // Twitter / X
        document::Meta { name: "twitter:card", content: "{twitter_card}" }
        document::Meta { name: "twitter:title", content: "{full_title}" }
        document::Meta { name: "twitter:description", content: "{description}" }
        if let Some(image) = og_image {
            document::Meta { name: "twitter:image", content: "{image}" }
        }
    }
}

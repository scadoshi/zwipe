//! Per-route `<head>` metadata, picked up by SSG so the prerendered HTML
//! carries it before any JS runs. That is what makes link unfurling work.

use dioxus::prelude::*;

/// Site-wide constants a host plugs into [`PageMeta`].
#[derive(Clone, PartialEq)]
pub struct SiteMeta {
    /// Public base URL, no trailing slash.
    pub base_url: &'static str,
    /// The `og:site_name` and the `<title>` suffix.
    pub site_name: &'static str,
    /// Site-wide OG image path. Its presence also picks the Twitter card type.
    pub og_image_path: Option<&'static str>,
}

/// Head metadata for one page. `path` starts with `/`, or is empty for the
/// home page. Titles get a ` | {site_name}` suffix unless they are the bare
/// site name.
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

        document::Meta { property: "og:type", content: "website" }
        document::Meta { property: "og:site_name", content: "{site.site_name}" }
        document::Meta { property: "og:title", content: "{full_title}" }
        document::Meta { property: "og:description", content: "{description}" }
        document::Meta { property: "og:url", content: "{canonical}" }
        if let Some(image) = og_image.clone() {
            document::Meta { property: "og:image", content: "{image}" }
        }

        document::Meta { name: "twitter:card", content: "{twitter_card}" }
        document::Meta { name: "twitter:title", content: "{full_title}" }
        document::Meta { name: "twitter:description", content: "{description}" }
        if let Some(image) = og_image {
            document::Meta { name: "twitter:image", content: "{image}" }
        }
    }
}

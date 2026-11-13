//! zwipe.net's guide pages: the index and one page per guide, drawn by
//! components' [`GuideIndex()`] and [`GuideArticle()`] inside the site's page
//! shell, with zwipe.net's own title, description and Article JSON-LD. Every guide link stays on zwipe.net.
//!
//! The guides are `zwipe_core::content::guides`. `/guides/:slug` is one
//! dynamic route, and `slugs` feeds the SSG pass so each article is
//! prerendered as its own page: a dynamic route is excluded from
//! `Route::static_routes()`, and an unprerendered path is served by
//! `404.html` with a 404 status.

#[cfg(test)]
mod served;

use components::{GuideArticle, GuideIndex, GuideMissing};
use dioxus::prelude::*;
#[cfg(any(feature = "server", test))]
use zwipe_core::content::guides::GUIDES;
use zwipe_core::content::guides::find;

use crate::components::PageMeta;
use crate::{Footer, Nav, Route, WEB_BASE};

/// Every guide slug, for the SSG prerender list in `main.rs`. Reads `GUIDES`
/// so the prerendered set, the rendered articles and `build.rs`'s sitemap all
/// resolve to one source; `build.rs` asserts its own copy against it.
#[cfg(any(feature = "server", test))]
pub(crate) fn slugs() -> impl Iterator<Item = &'static str> {
    GUIDES.iter().map(|g| g.slug)
}

/// A guide's page on zwipe.net.
fn guide_href(slug: &'static str) -> String {
    Route::GuidePage {
        slug: slug.to_owned(),
    }
    .to_string()
}

#[component]
pub fn Guides() -> Element {
    rsx! {
        PageMeta {
            title: "Guides",
            description: "How-to guides for the Zwipe Magic: The Gathering deck builder: swiping, filtering, budgeting, commanders, stats, and more.",
            path: "/guides/",
        }
        Nav {}
        main { id: "content", class: "page content-enter",
            GuideIndex { href: guide_href }
        }
        Footer {}
    }
}

#[component]
pub fn GuidePage(slug: String) -> Element {
    let Some(g) = find(&slug) else {
        return rsx! {
            PageMeta {
                title: "Guide not found",
                description: "That guide doesn't exist. Browse all Zwipe guides.",
                path: "/guides/",
            }
            Nav {}
            main { id: "content", class: "page content-enter",
                GuideMissing { index: Route::Guides {}.to_string() }
            }
            Footer {}
        };
    };

    // The primary tag is the category for the breadcrumb and JSON-LD.
    let primary = g.tags.first().copied().unwrap_or("Guides");

    // Article JSON-LD for rich results: headline/description/section straight
    // from the guide, with Zwipe as the publisher.
    let json_ld = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": g.title,
        "description": g.summary,
        "articleSection": primary,
        "url": format!("{WEB_BASE}/guides/{}/", g.slug),
        "publisher": {
            "@type": "Organization",
            "name": "Zwipe",
            "url": WEB_BASE,
        },
    })
    .to_string();

    rsx! {
        PageMeta { title: "{g.title}", description: "{g.summary}", path: "/guides/{g.slug}/" }
        document::Script { r#type: "application/ld+json", "{json_ld}" }
        Nav {}
        main { id: "content", class: "page content-enter guide-page",
            GuideArticle { guide: g, href: guide_href }
        }
        Footer {}
    }
}

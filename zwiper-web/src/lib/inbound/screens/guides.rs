//! The guides, open to everyone: the index at `/guides` and one guide at
//! `/guides/<slug>`, under the signed-in nav or the public one. They are
//! components' [`GuideIndex`] and [`GuideArticle`], the pages zwipe.net
//! draws, with every guide link staying in the web app. Each page names its
//! zwipe.net twin as canonical, so search engines credit zwipe.net.

use components::{GuideArticle, GuideIndex, GuideMissing};
use dioxus::prelude::*;
use zwipe_core::content::guides::find;
use zwipe_core::domain::site::WEB_BASE;

use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;

/// A guide's page in the web app.
fn guide_href(slug: &'static str) -> String {
    Route::GuidePage {
        slug: slug.to_owned(),
    }
    .to_string()
}

/// The zwipe.net address of the page at `path` (`/guides/` and the like),
/// the canonical copy.
fn canonical(path: &str) -> String {
    format!("{WEB_BASE}{path}")
}

/// The guides index.
#[component]
pub fn Guides() -> Element {
    rsx! {
        document::Link { rel: "canonical", href: canonical("/guides/") }
        ScreenFrame { title: "Guides", titled: true,
            div { class: "screen-content content-enter",
                div { class: "web-guides", GuideIndex { href: guide_href } }
            }
        }
    }
}

/// The guide named `slug`, or a page saying there is none.
#[component]
pub fn GuidePage(slug: String) -> Element {
    let Some(guide) = find(&slug) else {
        return rsx! {
            ScreenFrame { title: "Guide not found", titled: true,
                div { class: "screen-content content-enter",
                    div { class: "web-guides", GuideMissing { index: Route::Guides {}.to_string() } }
                }
            }
        };
    };
    rsx! {
        document::Link { rel: "canonical", href: canonical(&format!("/guides/{}/", guide.slug)) }
        ScreenFrame { title: guide.title, titled: true,
            div { class: "screen-content content-enter",
                div { class: "web-guides guide-page",
                    GuideArticle { guide, href: guide_href, index: Route::Guides {}.to_string() }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_guide_s_canonical_is_its_zwipe_net_page() {
        assert_eq!(
            canonical("/guides/filtering/"),
            format!("{WEB_BASE}/guides/filtering/")
        );
        assert_eq!(guide_href("filtering"), "/guides/filtering");
    }
}

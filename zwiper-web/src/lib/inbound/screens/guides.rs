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

/// Sets the page's one `rel=canonical` link to `href` while the screen is
/// up, and takes it out when the screen goes, so moving between guides in
/// the app never leaves two in the head.
fn use_canonical(href: &str) {
    let href = href.to_owned();
    use_effect(use_reactive!(|href| {
        document::eval(&set_canonical_js(&href));
    }));
    use_drop(|| {
        document::eval(DROP_CANONICAL_JS);
    });
}

/// A script that makes `href` the page's one canonical link. `href` is a
/// zwipe.net address built from a guide's slug, so it carries no quote.
fn set_canonical_js(href: &str) -> String {
    format!(
        "let link = document.head.querySelector('link[rel=canonical]'); if (!link) {{ link = document.createElement('link'); link.rel = 'canonical'; document.head.appendChild(link); }} link.href = '{href}';"
    )
}

/// Takes the page's canonical link out.
const DROP_CANONICAL_JS: &str =
    "document.head.querySelectorAll('link[rel=canonical]').forEach((link) => link.remove());";

/// The guides index.
#[component]
pub fn Guides() -> Element {
    use_canonical(&canonical("/guides/"));
    rsx! {
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
    let guide = find(&slug);
    use_canonical(&canonical(&guide.map_or_else(
        || "/guides/".to_owned(),
        |g| format!("/guides/{}/", g.slug),
    )));
    let Some(guide) = guide else {
        return rsx! {
            ScreenFrame { title: "Guide not found", titled: true,
                div { class: "screen-content content-enter",
                    div { class: "web-guides", GuideMissing { index: Route::Guides {}.to_string() } }
                }
            }
        };
    };
    rsx! {
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
        let js = set_canonical_js(&canonical("/guides/filtering/"));
        assert!(js.contains("querySelector('link[rel=canonical]')"), "{js}");
        assert!(
            js.contains(&format!("link.href = '{WEB_BASE}/guides/filtering/'")),
            "{js}"
        );
    }
}

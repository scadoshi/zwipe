//! Knowledge-base guide pages, rendered from a data-driven content model so
//! articles stay uniform and easy to iterate on. `GUIDES` is the article set;
//! `Guides` is the index and `GuidePage` renders one article by slug.
//!
//! Content lives in `content.rs`. Routing for `/guides/:slug` is dynamic for
//! now (client-hydrated); SSG prerender per guide is a later step (see
//! `context/plans/seo_guides.md`).

mod content;

use crate::components::PageMeta;
use crate::{Footer, Nav, Route};
use content::{Block, GUIDES};
use dioxus::prelude::*;

/// Maps a swipe direction to the app's gesture color class (shared with the
/// home hero), so guide swipe legends match the in-app hint coloring.
fn swipe_class(dir: &str) -> &'static str {
    match dir {
        "right" => "swipe-add",
        "left" => "swipe-skip",
        "up" => "swipe-maybe",
        "down" => "swipe-undo",
        _ => "",
    }
}

/// Renders body text, turning `backtick`-delimited tokens into highlighted
/// keyword spans (filter names, option values, enum members, and the like).
fn inline(text: &str) -> Element {
    let parts: Vec<(bool, String)> = text
        .split('`')
        .enumerate()
        .map(|(i, s)| (i % 2 == 1, s.to_string()))
        .collect();
    rsx! {
        for (kw , s) in parts {
            if kw {
                span { class: "guide-kw", "{s}" }
            } else {
                "{s}"
            }
        }
    }
}

fn render_block(b: &'static Block) -> Element {
    match b {
        Block::Lead(t) => rsx! { p { class: "guide-lead", {inline(t)} } },
        Block::H2(t) => rsx! { h2 { class: "guide-h2", "{t}" } },
        Block::P(t) => rsx! { p { class: "guide-p", {inline(t)} } },
        Block::Steps(items) => rsx! {
            ol { class: "guide-steps",
                for it in items.iter() {
                    li { {inline(it)} }
                }
            }
        },
        Block::Bullets(items) => rsx! {
            ul { class: "guide-bullets",
                for it in items.iter() {
                    li { {inline(it)} }
                }
            }
        },
        Block::Swipe(rows) => rsx! {
            ul { class: "guide-swipe",
                for (dir , meaning) in rows.iter() {
                    li {
                        "Swipe "
                        span { class: "{swipe_class(dir)}", "{dir}" }
                        " to {meaning}."
                    }
                }
            }
        },
        Block::Note(t) => rsx! { aside { class: "guide-note", {inline(t)} } },
    }
}

/// Category order for the index. Guides are grouped under these headings; any
/// category not listed here is skipped, so keep it in sync with `content.rs`.
const CATEGORY_ORDER: &[&str] = &["Start", "Build", "Cards", "Decks"];

#[component]
pub fn Guides() -> Element {
    rsx! {
        PageMeta {
            title: "Guides",
            description: "How-to guides for building Magic: The Gathering decks on mobile with Zwipe: swiping to build, filtering, budgeting, land targets, deck stats, commanders, and more.",
            path: "/guides",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "section",
                h1 { "Guides" }
                p { class: "arch-subtitle", "How Zwipe works, one feature at a time." }
            }
            for cat in CATEGORY_ORDER.iter() {
                section { class: "section guide-cat",
                    h2 { class: "guide-cat-heading", "{cat}" }
                    div { class: "card-grid",
                        for g in GUIDES.iter().filter(|g| g.category == *cat) {
                            Link {
                                to: Route::GuidePage { slug: g.slug.to_string() },
                                class: "card guide-card",
                                h3 { class: "card-title", "{g.title}" }
                                p { class: "card-summary", "{g.summary}" }
                            }
                        }
                    }
                }
            }
        }
        Footer {}
    }
}

#[component]
pub fn GuidePage(slug: String) -> Element {
    let Some(g) = GUIDES.iter().find(|g| g.slug == slug) else {
        return rsx! {
            PageMeta {
                title: "Guide not found",
                description: "That guide doesn't exist. Browse all Zwipe guides.",
                path: "/guides",
            }
            Nav {}
            div { class: "page content-enter",
                div { class: "section",
                    h1 { "Guide not found" }
                    p { class: "guide-p",
                        "That guide doesn't exist. "
                        Link { to: Route::Guides {}, "Back to all guides" }
                        "."
                    }
                }
            }
            Footer {}
        };
    };

    rsx! {
        PageMeta { title: "{g.title}", description: "{g.summary}", path: "/guides/{g.slug}" }
        Nav {}
        div { class: "page content-enter",
            div { class: "section panel",
                div { class: "guide-breadcrumb",
                    Link { to: Route::Guides {}, "Guides" }
                    span { class: "crumb-sep", "→" }
                    span { class: "crumb-cat", "{g.category}" }
                    span { class: "crumb-sep", "→" }
                    span { "{g.title}" }
                }
                h1 { class: "guide-title", "{g.title}" }
            }
            div { class: "guide-content section panel",
                for b in g.blocks.iter() {
                    {render_block(b)}
                }
                div { class: "guide-cta",
                    a {
                        class: "store-link",
                        href: "https://apps.apple.com/us/app/zwipe-tcg/id6761341603",
                        target: "_blank",
                        rel: "noopener",
                        "Get Zwipe on the App Store ↗"
                    }
                    Link { to: Route::Android {}, class: "store-link", "Get it on Google Play ↗" }
                }
            }
        }
        Footer {}
    }
}

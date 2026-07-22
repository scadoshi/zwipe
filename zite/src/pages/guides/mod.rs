//! Knowledge-base guide pages, rendered from a data-driven content model so
//! articles stay uniform and easy to iterate on. `GUIDES` is the article set;
//! `Guides` is the index and `GuidePage` renders one article by slug.
//!
//! Content lives in `content.rs`. Routing for `/guides/:slug` is dynamic for
//! now (client-hydrated); SSG prerender per guide is a later step (see
//! `context/plans/seo_guides.md`).

mod content;

use crate::{Footer, Nav, Route, WEB_BASE, components::PageMeta};
use content::{Block, GUIDES};
use dioxus::prelude::*;
use zwipe_components::Panel;

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
    let parts = text
        .split('`')
        .enumerate()
        .map(|(i, s)| (i % 2 == 1, s.to_string()));
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
        Block::Diagram(t) => rsx! { pre { class: "guide-diagram", "{t}" } },
    }
}

/// Tag vocabulary for the index filter row, in display order. Each guide is
/// tagged with 1-3 of these in `content.rs`.
const GUIDE_TAGS: &[&str] = &[
    "Getting started",
    "Swiping",
    "Filtering",
    "Cards",
    "Commander",
    "Oracle tags",
    "Deck building",
    "Deck stats",
    "Importing",
];

#[component]
pub fn Guides() -> Element {
    let mut selected = use_signal(|| Option::<&'static str>::None);
    rsx! {
        PageMeta {
            title: "Guides",
            description: "How-to guides for building Magic: The Gathering decks on mobile with Zwipe: swiping to build, filtering, budgeting, land targets, deck stats, commanders, and more.",
            path: "/guides",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "page-header section panel",
                h1 { "Guides" }
                p { class: "tagline", "How Zwipe works, one feature at a time." }
            }
            div { class: "guide-filter",
                button {
                    class: if selected().is_none() { "chip selected" } else { "chip" },
                    onclick: move |_| selected.set(None),
                    "All"
                }
                for tag in GUIDE_TAGS.iter().copied() {
                    button {
                        class: if selected() == Some(tag) { "chip selected" } else { "chip" },
                        onclick: move |_| {
                            if selected() == Some(tag) {
                                selected.set(None);
                            } else {
                                selected.set(Some(tag));
                            }
                        },
                        "{tag}"
                    }
                }
            }
            div { class: "card-grid",
                for g in GUIDES.iter().filter(|g| selected().is_none_or(|t| g.tags.contains(&t))) {
                    Link {
                        to: Route::GuidePage { slug: g.slug.to_string() },
                        class: "guide-card",
                        Panel { title: "{g.title}",
                            p { class: "card-summary", "{g.summary}" }
                            div { class: "guide-tags",
                                for t in g.tags.iter().copied() {
                                    span { class: "chip", "{t}" }
                                }
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

    // The primary tag stands in for the old category (breadcrumb + JSON-LD).
    let primary = g.tags.first().copied().unwrap_or("Guides");

    // Article JSON-LD for rich results: headline/description/section straight
    // from the guide, with Zwipe as the publisher.
    let json_ld = serde_json::json!({
        "@context": "https://schema.org",
        "@type": "Article",
        "headline": g.title,
        "description": g.summary,
        "articleSection": primary,
        "url": format!("{WEB_BASE}/guides/{}", g.slug),
        "publisher": {
            "@type": "Organization",
            "name": "Zwipe",
            "url": WEB_BASE,
        },
    })
    .to_string();

    rsx! {
        PageMeta { title: "{g.title}", description: "{g.summary}", path: "/guides/{g.slug}" }
        document::Script { r#type: "application/ld+json", "{json_ld}" }
        Nav {}
        div { class: "page content-enter guide-page",
            div { class: "section panel",
                div { class: "guide-breadcrumb",
                    Link { to: Route::Guides {}, "Guides" }
                    span { class: "crumb-sep", "→" }
                    span { class: "crumb-cat", "{primary}" }
                    span { class: "crumb-sep", "→" }
                    span { "{g.title}" }
                }
                h1 { class: "guide-title", "{g.title}" }
            }
            div { class: "guide-content section panel",
                for b in g.blocks.iter() {
                    {render_block(b)}
                }
            }
            if !g.related.is_empty() {
                div { class: "guide-related section panel",
                    h2 { class: "guide-related-heading", "Related guides" }
                    div { class: "guide-related-list",
                        for rel in g.related.iter().copied() {
                            if let Some(rg) = GUIDES.iter().find(|x| x.slug == rel) {
                                Link {
                                    to: Route::GuidePage { slug: rg.slug.to_string() },
                                    class: "guide-related-link",
                                    "{rg.title}"
                                }
                            }
                        }
                    }
                }
            }
        }
        Footer {}
    }
}

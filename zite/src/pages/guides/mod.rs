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

/// Renders a guide's content. With screenshots present, the prose runs in
/// one column with a single phone-shaped gallery beside it (the home page's
/// prev/next viewer) — one tall capture on screen at a time, sticky while
/// the text scrolls. Without screenshots, blocks render linearly as ever.
fn render_content(blocks: &'static [Block]) -> Element {
    let shots: Vec<(&'static str, &'static str, Option<&'static str>)> = blocks
        .iter()
        .filter_map(|b| match b {
            Block::Image { file, alt, caption } => Some((*file, *alt, *caption)),
            _ => None,
        })
        .collect();
    let text = blocks.iter().filter(|b| !matches!(b, Block::Image { .. }));

    if shots.is_empty() {
        return rsx! {
            for b in text {
                {render_block(b)}
            }
        };
    }
    rsx! {
        div { class: "guide-gallery-layout",
            div {
                for b in text {
                    {render_block(b)}
                }
            }
            div { class: "guide-gallery-col", GuideGallery { shots } }
        }
    }
}

/// The guide's screenshot viewer: one image at a time with the home
/// gallery's prev/next chrome, caption from the shot's own text.
#[component]
fn GuideGallery(shots: Vec<(&'static str, &'static str, Option<&'static str>)>) -> Element {
    let mut index = use_signal(|| 0usize);
    let total = shots.len();
    let i = index().min(total.saturating_sub(1));
    let (file, alt, caption) = shots[i];
    let caption_text = caption.unwrap_or(alt);
    let src = content::guide_image(file);

    rsx! {
        div { class: "gallery-body guide-gallery-body",
            if let Some(src) = src {
                img {
                    key: "{i}",
                    class: "gallery-video",
                    src: "{src}",
                    alt: "{alt}",
                    loading: "lazy",
                    draggable: false,
                }
            }
            if total > 1 {
                button {
                    class: "gallery-nav gallery-prev",
                    aria_label: "Previous screenshot",
                    onclick: move |_| {
                        let i = index();
                        index.set(if i == 0 { total - 1 } else { i - 1 });
                    },
                    "←"
                }
                button {
                    class: "gallery-nav gallery-next",
                    aria_label: "Next screenshot",
                    onclick: move |_| {
                        let i = index();
                        index.set((i + 1) % total);
                    },
                    "→"
                }
            }
        }
        hr { class: "gallery-rule" }
        div { class: "gallery-footer",
            span { key: "{i}", class: "gallery-caption", "{caption_text}" }
            if total > 1 {
                span { class: "gallery-counter", "{i + 1} / {total}" }
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
        Block::Image { file, alt, caption } => rsx! {
            if let Some(src) = content::guide_image(file) {
                figure { class: "guide-figure",
                    img {
                        class: "guide-img",
                        src: "{src}",
                        alt: "{alt}",
                        loading: "lazy",
                        draggable: false,
                    }
                    if let Some(c) = caption {
                        figcaption { class: "guide-figcaption", "{c}" }
                    }
                }
            }
        },
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

/// Color class for a guide tag, keyed by its position in [`GUIDE_TAGS`] so the
/// same tag reads the same color on every card (the bare `.tag` cycle is
/// positional and would recolor a tag per card).
fn tag_color_class(tag: &str) -> &'static str {
    const CLASSES: [&str; 6] = ["tag-c1", "tag-c2", "tag-c3", "tag-c4", "tag-c5", "tag-c6"];
    let idx = GUIDE_TAGS.iter().position(|t| *t == tag).unwrap_or(0);
    CLASSES.get(idx % CLASSES.len()).unwrap_or(&"tag-c1")
}

#[component]
pub fn Guides() -> Element {
    let mut selected = use_signal(|| Option::<&'static str>::None);
    rsx! {
        PageMeta {
            title: "Guides",
            description: "How-to guides for the Zwipe Magic: The Gathering deck builder: swiping, filtering, budgeting, commanders, stats, and more.",
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
                                    span { class: "tag {tag_color_class(t)}", "{t}" }
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
                {render_content(g.blocks)}
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

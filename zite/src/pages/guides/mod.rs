//! Knowledge-base guide pages, rendered from a data-driven content model so
//! articles stay uniform and easy to iterate on. `GUIDES` is the article set;
//! `Guides` is the index and `GuidePage` renders one article by slug.
//!
//! Content lives in `content.rs`. Routing for `/guides/:slug` is dynamic for
//! now (client-hydrated); SSG prerender per guide is a later step (see
//! `context/plans/seo_guides.md`).

mod content;

use crate::{Footer, Nav, Route, WEB_BASE, components::PageMeta};
use content::{Block, GUIDES, Guide};
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

/// A guide's screenshots, harvested for the sidecar gallery panel.
fn guide_shots(
    blocks: &'static [Block],
) -> Vec<(&'static str, &'static str, Option<&'static str>)> {
    blocks
        .iter()
        .filter_map(|b| match b {
            Block::Image { file, alt, caption } => Some((*file, *alt, *caption)),
            _ => None,
        })
        .collect()
}

/// Renders a guide's prose blocks linearly (screenshots are harvested into
/// the sidecar gallery panel instead of rendering inline).
fn render_text(blocks: &'static [Block]) -> Element {
    rsx! {
        for b in blocks.iter().filter(|b| !matches!(b, Block::Image { .. })) {
            {render_block(b)}
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

/// Everything about a guide the index search can match: title, summary, tags,
/// and the article's prose. Lowercased, with the `backtick` keyword markers
/// stripped so a search for "group by" hits `Group by` in the body too.
///
/// Built per keystroke over the compiled [`GUIDES`] array (19 articles, no
/// index, no network) — cheap enough that caching would cost more than it saves.
fn haystack(g: &Guide) -> String {
    let mut s = String::new();
    let mut push = |t: &str| {
        s.push_str(t);
        s.push(' ');
    };
    push(g.title);
    push(g.summary);
    for t in g.tags.iter().copied() {
        push(t);
    }
    for b in g.blocks.iter() {
        match b {
            Block::Lead(t) | Block::H2(t) | Block::P(t) | Block::Note(t) => push(t),
            Block::Steps(items) | Block::Bullets(items) => {
                for it in items.iter().copied() {
                    push(it);
                }
            }
            Block::Swipe(rows) => {
                for (dir, meaning) in rows.iter() {
                    push(dir);
                    push(meaning);
                }
            }
            // Diagrams are ASCII flow art and images live in the gallery —
            // neither reads as prose a searcher would type.
            Block::Diagram(_) | Block::Image { .. } => {}
        }
    }
    s.retain(|c| c != '`');
    s.to_lowercase()
}

/// Whether a guide matches the query: every whitespace-separated term has to
/// appear somewhere in its [`haystack`], so extra words narrow rather than
/// widen the result set.
fn matches_query(g: &Guide, query: &str) -> bool {
    let hay = haystack(g);
    query.split_whitespace().all(|term| hay.contains(term))
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
    let mut query = use_signal(String::new);
    // Search and the tag chips narrow the same list: a guide shows when it
    // passes both.
    let q = query().trim().to_lowercase();
    let hits: Vec<&Guide> = GUIDES
        .iter()
        .filter(|g| selected().is_none_or(|t| g.tags.contains(&t)))
        .filter(|g| q.is_empty() || matches_query(g, &q))
        .collect();
    // Signature of the visible set, prefixed onto each card's key so the cards
    // remount — and replay their entrance — only when the results actually
    // change, not on every keystroke that leaves the same list standing.
    let sig = hits.iter().map(|g| g.slug).collect::<Vec<_>>().join(",");
    rsx! {
        PageMeta {
            title: "Guides",
            description: "How-to guides for the Zwipe Magic: The Gathering deck builder: swiping, filtering, budgeting, commanders, stats, and more.",
            path: "/guides",
        }
        Nav {}
        div { class: "page content-enter",
            div { class: "page-header",
                Panel { eyebrow: "Explore", title: "Guides", title_h1: true,
                    p { class: "tagline", "How Zwipe works, one feature at a time." }
                }
            }
            // One console row, the app's chip-row anatomy: inline label, the
            // search input with its clear button, then the tag chips.
            div { class: "guide-filter",
                span { class: "guide-filter-label", "Filter:" }
                input {
                    class: "guide-search-input",
                    r#type: "text",
                    placeholder: "Search guides",
                    aria_label: "Search guides",
                    value: "{query}",
                    autocapitalize: "none",
                    autocorrect: "off",
                    spellcheck: "false",
                    oninput: move |evt| query.set(evt.value()),
                }
                if !query().is_empty() {
                    button {
                        class: "guide-search-clear",
                        aria_label: "Clear search",
                        onclick: move |_| query.set(String::new()),
                        "\u{00d7}"
                    }
                }
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
            if hits.is_empty() {
                div { class: "guide-empty",
                    p {
                        if q.is_empty() {
                            "No guides match that tag."
                        } else {
                            "No guides match \u{201c}{query().trim()}\u{201d}."
                        }
                    }
                    button {
                        class: "chip",
                        onclick: move |_| {
                            query.set(String::new());
                            selected.set(None);
                        },
                        "Clear search"
                    }
                }
            }
            div { class: "card-grid",
                for (i , g) in hits.iter().enumerate() {
                    Link {
                        key: "{sig}|{g.slug}",
                        to: Route::GuidePage { slug: g.slug.to_string() },
                        class: "guide-card guide-card-in",
                        // Stagger the deal-in, capped so a wide result set
                        // doesn't trail on for a second.
                        style: "animation-delay: {i.min(8) * 35}ms;",
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
                    Panel { eyebrow: "Guides", title: "Guide not found", title_h1: true,
                        p { class: "guide-p",
                            "That guide doesn't exist. "
                            Link { to: Route::Guides {}, "Back to all guides" }
                            "."
                        }
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
            // With screenshots, the article and a sidecar gallery Panel sit
            // as siblings: prose panel beside the phone viewer on wide
            // screens, gallery first when stacked on phones. The article
            // panel carries its own eyebrow (the primary category) + h1.
            {
                let shots = guide_shots(g.blocks);
                let article = rsx! {
                    div { class: "guide-content section panel",
                        // Panel-header anatomy (title + rule) on an h1 so
                        // the page keeps its semantic heading.
                        h1 { class: "panel-title guide-title", "{g.title}" }
                        div { class: "guide-title-tags",
                            for t in g.tags.iter().copied() {
                                span { class: "tag {tag_color_class(t)}", "{t}" }
                            }
                        }
                        hr { class: "panel-rule" }
                        {render_text(g.blocks)}
                    }
                };
                if shots.is_empty() {
                    rsx! {
                        {article}
                    }
                } else {
                    rsx! {
                        div { class: "guide-with-gallery",
                            div { class: "guide-gallery-col",
                                Panel { eyebrow: "Screens", title: "In the app",
                                    GuideGallery { shots }
                                }
                            }
                            {article}
                        }
                    }
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

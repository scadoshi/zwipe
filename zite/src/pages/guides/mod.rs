//! Knowledge-base guide pages, rendered from a data-driven content model so
//! articles stay uniform and easy to iterate on. `GUIDES` is the article set;
//! `Guides` is the index and `GuidePage` renders one article by slug.
//!
//! Content lives in `content.rs`. `/guides/:slug` is one dynamic route, and
//! `slugs` feeds the SSG pass so each article is prerendered as its own
//! page: a dynamic route is excluded from `Route::static_routes()`, and an
//! unprerendered path is served by `404.html` with a 404 status.

mod content;

use components::hint::KeyGlyph;
use components::{ClearWrap, GalleryFooter, GalleryFrame, HintKey, Panel};
use dioxus::prelude::*;
use zwipe_core::content::guides::{Block, GUIDES, Guide};
use zwipe_ui::hint::KeyIcon;

use crate::components::{MediaFrame, PageMeta};
use crate::{Footer, Nav, Route, WEB_BASE};

/// Every guide slug, for the SSG prerender list in `main.rs`. Reads `GUIDES`
/// so the prerendered set, the rendered articles and `build.rs`'s sitemap all
/// resolve to one source; `build.rs` asserts its own copy against it.
#[cfg(feature = "server")]
pub(crate) fn slugs() -> impl Iterator<Item = &'static str> {
    GUIDES.iter().map(|g| g.slug)
}

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

/// One run of a guide line: plain text, a `backtick` keyword, or an
/// `[[icon]]` naming an icon button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Inline<'line> {
    /// Text drawn as is.
    Text(&'line str),
    /// A highlighted keyword: a filter name, an option value, a button's
    /// words.
    Keyword(&'line str),
    /// An icon button, named as [`KeyIcon::name`] lists them, drawn as the
    /// button's own icon in a hint key. A name no icon has renders as the
    /// literal token so a typo shows.
    Icon(&'line str),
}

/// Splits a guide line into its runs: `backtick` pairs are keywords and
/// `[[name]]` is an icon; an unclosed `[[` stays text.
fn tokens(text: &str) -> Vec<Inline<'_>> {
    let mut runs = Vec::new();
    for (i, part) in text.split('`').enumerate() {
        if i % 2 == 1 {
            runs.push(Inline::Keyword(part));
            continue;
        }
        let mut rest = part;
        while let Some((before, after)) = rest.split_once("[[") {
            let Some((name, tail)) = after.split_once("]]") else {
                break;
            };
            if !before.is_empty() {
                runs.push(Inline::Text(before));
            }
            runs.push(Inline::Icon(name));
            rest = tail;
        }
        if !rest.is_empty() {
            runs.push(Inline::Text(rest));
        }
    }
    runs
}

/// Renders body text: `backtick`-delimited tokens become highlighted keyword
/// spans (filter names, option values, enum members, and the like) and
/// `[[icon]]` tokens become the icon button's own icon in a hint key, so a
/// guide never describes an icon in words.
fn inline(text: &str) -> Element {
    let runs = tokens(text);
    rsx! {
        for run in runs {
            match run {
                Inline::Keyword(s) => rsx! { span { class: "guide-kw", "{s}" } },
                Inline::Text(s) => rsx! { "{s}" },
                Inline::Icon(name) => match KeyIcon::from_name(name) {
                    Some(icon) => rsx! { HintKey { KeyGlyph { icon } } },
                    None => rsx! { "[[{name}]]" },
                },
            }
        }
    }
}

/// A guide line's words alone: keywords without their backticks, icon
/// tokens dropped.
fn prose(text: &str) -> String {
    tokens(text)
        .into_iter()
        .filter_map(|run| match run {
            Inline::Text(s) | Inline::Keyword(s) => Some(s),
            Inline::Icon(_) => None,
        })
        .collect()
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
    let index = use_signal(|| 0usize);
    let total = shots.len();
    let i = index().min(total.saturating_sub(1));
    let Some(&(file, alt, caption)) = shots.get(i) else {
        return rsx! {};
    };
    let caption_text = caption.unwrap_or(alt);
    let src = content::guide_image(file);

    rsx! {
        GalleryFrame { index, total, noun: "screenshot", class: "guide-gallery-body",
            if let Some(src) = src {
                MediaFrame { key: "{i}", size: content::guide_image_size(file),
                    img {
                        src: "{src}",
                        alt: "{alt}",
                        loading: "lazy",
                        draggable: false,
                    }
                }
            }
        }
        hr { class: "gallery-rule" }
        GalleryFooter { index: i, total, caption: caption_text.to_string() }
    }
}

/// A section heading's id, so a link can land on it: the heading in
/// lowercase, its letters and digits kept and every run of anything else a
/// single `-`, with none at either end.
fn heading_anchor(heading: &str) -> String {
    let mut id = String::new();
    let mut dash = false;
    for c in heading.chars() {
        if c.is_alphanumeric() {
            if dash && !id.is_empty() {
                id.push('-');
            }
            dash = false;
            id.extend(c.to_lowercase());
        } else {
            dash = true;
        }
    }
    id
}

fn render_block(b: &'static Block) -> Element {
    match b {
        Block::Lead(t) => rsx! { p { class: "guide-lead", {inline(t)} } },
        Block::H2(t) => rsx! { h2 { class: "guide-h2", id: heading_anchor(t), "{t}" } },
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
        Block::Diagram(t) => rsx! {
            div { class: "guide-diagram",
                pre { class: "scroll-fade-x", "{t}" }
            }
        },
        Block::Image { file, alt, caption } => rsx! {
            if let Some(src) = content::guide_image(file) {
                figure { class: "guide-figure",
                    MediaFrame { size: content::guide_image_size(file), class: "guide-img-frame",
                        img {
                            class: "guide-img",
                            src: "{src}",
                            alt: "{alt}",
                            loading: "lazy",
                            draggable: false,
                        }
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
/// stripped so a search for "group by" hits `Group by` in the body too, and
/// the `[[icon]]` tokens dropped, since nobody types them.
///
/// Built per keystroke over the compiled [`GUIDES`] array (21 articles, no
/// index, no network), cheap enough that caching would cost more than it saves.
fn haystack(g: &Guide) -> String {
    let mut s = String::new();
    let mut push = |t: &str| {
        s.push_str(&prose(t));
        s.push(' ');
    };
    push(g.title);
    push(g.summary);
    for t in g.tags.iter().copied() {
        push(t);
    }
    for b in g.blocks {
        match b {
            Block::Lead(t) | Block::H2(t) | Block::P(t) | Block::Note(t) => push(t),
            Block::Steps(items) | Block::Bullets(items) => {
                for it in items.iter().copied() {
                    push(it);
                }
            }
            Block::Swipe(rows) => {
                for (dir, meaning) in *rows {
                    push(dir);
                    push(meaning);
                }
            }
            // Diagrams are ASCII flow art and images live in the gallery;
            // neither reads as prose a searcher would type.
            Block::Diagram(_) | Block::Image { .. } => {}
        }
    }
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
/// same tag reads the same color on every card.
fn tag_color_class(tag: &str) -> &'static str {
    const CLASSES: [&str; 6] = ["tag-p1", "tag-p2", "tag-p3", "tag-p4", "tag-p5", "tag-p6"];
    let idx = GUIDE_TAGS.iter().position(|t| *t == tag).unwrap_or(0);
    idx.checked_rem(CLASSES.len())
        .and_then(|i| CLASSES.get(i))
        .unwrap_or(&"tag-p1")
}

/// The filter row above the guides, in the app's chip-row anatomy: the
/// "Filter" label chip, the search box with the shared clear button, then a
/// chip per tag. Search and tag narrow the same list.
#[component]
fn GuideFilter(query: Signal<String>, selected: Signal<Option<&'static str>>) -> Element {
    rsx! {
        // A tag or a search settles the guides into place (`components::settle`).
        div { class: "guide-filter", "data-settle": "",
            span { class: "stat-chip stat-chip-p1 chip-row-label", "Filter" }
            ClearWrap {
                filled: !query().is_empty(),
                on_clear: move |()| query.set(String::new()),
                label: "Clear search".to_string(),
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
    }
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
    rsx! {
        PageMeta {
            title: "Guides",
            description: "How-to guides for the Zwipe Magic: The Gathering deck builder: swiping, filtering, budgeting, commanders, stats, and more.",
            path: "/guides/",
        }
        Nav {}
        main { id: "content", class: "page content-enter",
            div { class: "page-header",
                Panel { title: "Guides", title_h1: true,
                    p { class: "tagline", "How Zwipe works, one feature at a time." }
                }
            }
            GuideFilter { query, selected }
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
            div { class: "card-grid", "data-settle-scope": "",
                for (i , g) in hits.iter().enumerate() {
                    Link {
                        key: "{g.slug}",
                        "data-settle-key": "{g.slug}",
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
                path: "/guides/",
            }
            Nav {}
            main { id: "content", class: "page content-enter",
                div { class: "section",
                    Panel { title: "Guide not found", title_h1: true,
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
                                Panel { title: "In the app",
                                    GuideGallery { shots }
                                }
                            }
                            {article}
                        }
                    }
                }
            }
            if !g.related.is_empty() {
                div { class: "guide-related section",
                    Panel { title: "Related guides",
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
        }
        Footer {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The filter row's HTML with `typed` in the search box.
    fn filter_html(typed: &'static str) -> String {
        #[component]
        fn Host(typed: &'static str) -> Element {
            let query = use_signal(|| typed.to_owned());
            let selected = use_signal(|| None);
            rsx! {
                GuideFilter { query, selected }
            }
        }
        let mut dom = VirtualDom::new_with_props(Host, HostProps { typed });
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn the_filter_labels_with_a_chip_and_clears_through_the_shared_wrap() {
        let empty = filter_html("");
        assert!(
            empty.contains(r#"class="stat-chip stat-chip-p1 chip-row-label">Filter<"#),
            "{empty}"
        );
        assert!(empty.contains(r#"class="input-clear-wrap""#), "{empty}");
        assert!(!empty.contains("guide-search-clear"), "{empty}");
        let typed = filter_html("mana");
        assert!(typed.contains("input-clear-wrap has-text"), "{typed}");
        assert!(typed.contains(r#"aria-label="Clear search""#), "{typed}");
    }

    #[test]
    fn a_heading_anchor_is_its_words_joined_by_dashes() {
        assert_eq!(heading_anchor("A skip lasts a week"), "a-skip-lasts-a-week");
        assert_eq!(
            heading_anchor("Profile, Budget & Tags"),
            "profile-budget-tags"
        );
        assert_eq!(heading_anchor("  Drilling down! "), "drilling-down");
        assert_eq!(heading_anchor("Mana (and lands)"), "mana-and-lands");
    }

    #[test]
    fn a_line_splits_into_text_keywords_and_icons() {
        assert_eq!(
            tokens("Tap the [[funnel]], then `Apply`."),
            vec![
                Inline::Text("Tap the "),
                Inline::Icon("funnel"),
                Inline::Text(", then "),
                Inline::Keyword("Apply"),
                Inline::Text("."),
            ]
        );
        assert_eq!(
            tokens("[[plus]] starts a deck"),
            vec![Inline::Icon("plus"), Inline::Text(" starts a deck")]
        );
        assert_eq!(
            tokens("an unclosed [[ stays"),
            vec![Inline::Text("an unclosed [[ stays")],
            "an unclosed token is text"
        );
        assert_eq!(tokens(""), vec![]);
    }

    #[test]
    fn every_icon_token_in_the_guides_names_a_real_icon() {
        let mut lines: Vec<&str> = Vec::new();
        for g in GUIDES {
            for b in g.blocks {
                match b {
                    Block::Lead(t) | Block::H2(t) | Block::P(t) | Block::Note(t) => lines.push(t),
                    Block::Steps(items) | Block::Bullets(items) => lines.extend(items.iter()),
                    Block::Swipe(rows) => lines.extend(rows.iter().map(|(_, m)| m)),
                    Block::Diagram(_) | Block::Image { .. } => {}
                }
            }
        }
        for line in lines {
            for run in tokens(line) {
                if let Inline::Icon(name) = run {
                    assert!(
                        KeyIcon::from_name(name).is_some(),
                        "unknown icon [[{name}]] in {line:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_search_haystack_drops_icon_tokens_and_backticks() {
        assert_eq!(
            prose("Tap the [[funnel]], then `Apply`."),
            "Tap the , then Apply."
        );
        let guide = GUIDES
            .iter()
            .find(|g| g.slug == "swipe-to-build")
            .expect("the swipe guide ships");
        let hay = haystack(guide);
        assert!(!hay.contains("[["), "{hay}");
        assert!(!hay.contains('`'), "{hay}");
        assert!(hay.contains("synergy"), "{hay}");
    }
}

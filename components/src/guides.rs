//! The guide pages zwipe.net and the web app both draw: [`GuideIndex`] (the
//! search, the tag chips and a card per guide), [`GuideArticle`] (the
//! article, its "In the app" gallery and its related guides) and
//! [`GuideMissing`]. The guides are [`zwipe_core::content::guides`] and what
//! they show is [`zwipe_ui::guides`]. Each host passes `href`, a guide's
//! path on its own site, so a guide link stays on the site it was clicked
//! on; the screenshots load from zwipe.net on both.

use dioxus::prelude::*;
use orangui::{ClearWrap, GalleryFooter, GalleryFrame, HintKey, Panel};
use zwipe_core::content::guides::{Block, Guide, image_size, image_url};
use zwipe_ui::guides::{self, GALLERY_KEYS, IndexFilter, Inline, Shot};
use zwipe_ui::hint::KeyIcon;

use crate::hint::KeyGlyph;
use crate::keys::use_scoped_keys;
use crate::media_frame::MediaFrame;

/// The cards' deal-in stagger per card, in milliseconds.
const DEAL_STAGGER_MS: usize = 35;

/// The last card whose deal-in waits longer than the one before it, so a
/// long list doesn't trail on for a second.
const DEAL_STAGGER_CAP: usize = 8;

/// A guide line: `backtick` keywords as highlighted spans and `[[icon]]`
/// tokens as the icon button's own icon in a hint key, so a guide never
/// describes an icon in words. An unknown icon shows its token.
fn inline(text: &str) -> Element {
    let runs = guides::tokens(text);
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

/// One block of an article's text.
fn text_block(block: &'static Block) -> Element {
    match block {
        Block::Lead(t) => rsx! { p { class: "guide-lead", {inline(t)} } },
        Block::H2(t) => rsx! { h2 { class: "guide-h2", id: guides::heading_anchor(t), "{t}" } },
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
                        span { class: "{guides::swipe_class(dir)}", "{dir}" }
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
        // The screenshots show in the gallery.
        Block::Image { .. } => rsx! {},
    }
}

/// The guide's screenshots, one at a time with the gallery's previous and
/// next chrome and the shot's caption under it. The arrow keys, and `h` and
/// `l`, step it while focus is in the gallery or on nothing.
#[component]
fn GuideGallery(shots: Vec<Shot>) -> Element {
    let mut index = use_signal(|| 0usize);
    let total = shots.len();
    use_scoped_keys(
        ".guide-gallery-col",
        &GALLERY_KEYS,
        Callback::new(move |key: String| {
            if let Some(step) = guides::gallery_step(&key) {
                let at = index();
                index.set(guides::stepped(at, total, step));
            }
        }),
    );
    let i = index().min(total.saturating_sub(1));
    let Some(&Shot { file, alt, caption }) = shots.get(i) else {
        return rsx! {};
    };
    let src = image_url(file);
    rsx! {
        GalleryFrame { index, total, noun: "screenshot", class: "guide-gallery-body",
            MediaFrame { key: "{i}", size: image_size(file).into(),
                img {
                    src: "{src}",
                    alt: "{alt}",
                    loading: "lazy",
                    draggable: false,
                }
            }
        }
        hr { class: "gallery-rule" }
        GalleryFooter { index: i, total, caption: caption.to_owned() }
    }
}

/// The filter row above the guides, in the app's chip-row anatomy: the
/// "Filter" label chip, the search box with the shared clear button, then a
/// chip per tag. Search and tag narrow the same list.
#[component]
fn GuideFilter(query: Signal<String>, selected: Signal<Option<&'static str>>) -> Element {
    rsx! {
        // A tag or a search settles the guides into place (`crate::settle`).
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
            for tag in guides::TAGS.iter().copied() {
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

/// A guide's tags as colored chips.
fn tag_chips(guide: &'static Guide, class: &'static str) -> Element {
    rsx! {
        div { class,
            for t in guide.tags.iter().copied() {
                span { class: "tag {guides::tag_color_class(t)}", "{t}" }
            }
        }
    }
}

/// A guide's tags as colored chips, each linking to `index`, the host's
/// guides index, filtered to that tag.
fn tag_links(guide: &'static Guide, class: &'static str, index: &str) -> Element {
    rsx! {
        div { class,
            for t in guide.tags.iter().copied() {
                Link {
                    to: format!("{index}{}", IndexFilter { tag: Some(t), query: String::new() }.to_search()),
                    class: "tag {guides::tag_color_class(t)}",
                    "{t}"
                }
            }
        }
    }
}

/// A script that puts `search` (an [`IndexFilter::to_search`], percent-encoded
/// so it needs no quoting beyond its quotes) as the page address's query,
/// keeping the path, the hash and the history entry.
fn replace_search_js(search: &str) -> String {
    format!(
        "history.replaceState(history.state, '', location.pathname + '{search}' + location.hash);"
    )
}

/// The guides index: the page's heading, the filter row, and a card per
/// guide that passes it, each linking to `href` of its slug. With none, a
/// line saying so and a chip that clears the filter.
///
/// The filter lives in the page address's query ([`IndexFilter`]): the index
/// opens on the one there, so Back from a guide and a shared link both show
/// the same list, and each change replaces it in place.
#[component]
pub fn GuideIndex(href: Callback<&'static str, String>) -> Element {
    let mut selected = use_signal(|| Option::<&'static str>::None);
    let mut query = use_signal(String::new);
    // Whether the address's filter is read yet; until then nothing is written over it.
    let mut read = use_signal(|| false);
    use_effect(move || {
        spawn(async move {
            if let Ok(search) = document::eval("return location.search")
                .join::<String>()
                .await
            {
                let filter = IndexFilter::from_search(&search);
                selected.set(filter.tag);
                query.set(filter.query);
            }
            read.set(true);
        });
    });
    use_effect(move || {
        let search = IndexFilter {
            tag: selected(),
            query: query(),
        }
        .to_search();
        if read() {
            document::eval(&replace_search_js(&search));
        }
    });
    let hits = guides::hits(selected(), &query());
    let typed = query().trim().to_owned();
    rsx! {
        div { class: "guide-index-head",
            Panel { title: "Guides", title_h1: true,
                p { class: "tagline", "How Zwipe works, one feature at a time." }
            }
        }
        GuideFilter { query, selected }
        if hits.is_empty() {
            div { class: "guide-empty",
                p {
                    if typed.is_empty() {
                        "No guides match that tag."
                    } else {
                        "No guides match \u{201c}{typed}\u{201d}."
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
            for (i , g) in hits.into_iter().enumerate() {
                Link {
                    key: "{g.slug}",
                    "data-settle-key": "{g.slug}",
                    to: href(g.slug),
                    class: "guide-card guide-card-in",
                    style: "animation-delay: {i.min(DEAL_STAGGER_CAP).saturating_mul(DEAL_STAGGER_MS)}ms;",
                    Panel { title: "{g.title}",
                        p { class: "card-summary", "{g.summary}" }
                        {tag_chips(g, "guide-tags")}
                    }
                }
            }
        }
    }
}

/// One guide: its article panel (the title as the page's `h1`, its tags and
/// its text), with its screenshots in an "In the app" gallery beside it
/// when it has any (above it on a phone), then its related guides, each
/// linking to `href` of its slug. Each tag links to `index`, the host's
/// guides index, filtered to that tag.
#[component]
pub fn GuideArticle(
    guide: &'static Guide,
    href: Callback<&'static str, String>,
    index: String,
) -> Element {
    let shots = guides::shots(guide);
    let related: Vec<&'static Guide> = guides::related(guide).collect();
    let article = rsx! {
        div { class: "guide-content section panel",
            // The panel's header anatomy (title and rule) on an h1, so the
            // page keeps its heading.
            h1 { class: "panel-title guide-title", "{guide.title}" }
            {tag_links(guide, "guide-title-tags", &index)}
            hr { class: "panel-rule" }
            for block in guide.blocks.iter() {
                {text_block(block)}
            }
        }
    };
    rsx! {
        if shots.is_empty() {
            {article}
        } else {
            div { class: "guide-with-gallery",
                div { class: "guide-gallery-col",
                    Panel { title: "In the app",
                        GuideGallery { shots }
                    }
                }
                {article}
            }
        }
        if !related.is_empty() {
            div { class: "guide-related section",
                Panel { title: "Related guides",
                    div { class: "guide-related-list",
                        for rg in related {
                            Link { to: href(rg.slug), class: "guide-related-link", "{rg.title}" }
                        }
                    }
                }
            }
        }
    }
}

/// The page for a slug no guide has, linking back to `index`, the host's
/// guides index.
#[component]
pub fn GuideMissing(index: String) -> Element {
    rsx! {
        div { class: "section",
            Panel { title: "Guide not found", title_h1: true,
                p { class: "guide-p",
                    "That guide doesn't exist. "
                    Link { to: index, "Back to all guides" }
                    "."
                }
            }
        }
    }
}

#[cfg(test)]
// The test site's Routable derive builds each `String` field's parse error
// from an `Infallible`, which rustc reads as an unreachable call; the derive's
// items sit beside the enum, so only a module-level expect reaches them.
#[expect(
    unreachable_code,
    reason = "Routable's derive wraps String fields' Infallible parse errors"
)]
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

    /// A host site's routes: its index draws the [`Page`] in context, and
    /// each guide sits under `/base/`.
    #[derive(Clone, PartialEq, Routable)]
    enum Site {
        #[route("/")]
        Index {},
        #[route("/base/:slug")]
        Base { slug: String },
    }

    #[component]
    fn Index() -> Element {
        match consume_context::<Page>() {
            Page::List => rsx! {
                GuideIndex { href: |slug| Site::Base { slug: String::from(slug) }.to_string() }
            },
            Page::Article(guide) => rsx! {
                GuideArticle {
                    guide,
                    href: |slug| Site::Base { slug: String::from(slug) }.to_string(),
                    index: "/".to_owned(),
                }
            },
        }
    }

    #[component]
    fn Base(slug: String) -> Element {
        rsx! { "{slug}" }
    }

    /// What the host's index draws.
    #[derive(Clone, Copy)]
    enum Page {
        List,
        Article(&'static Guide),
    }

    /// The host site's index HTML, drawing `page`.
    fn html(page: Page) -> String {
        fn app() -> Element {
            dioxus::history::provide_history_context(std::rc::Rc::new(
                dioxus::history::MemoryHistory::with_initial_path("/"),
            ));
            rsx! { Router::<Site> {} }
        }
        let mut dom = VirtualDom::new(app);
        dom.provide_root_context(page);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn the_index_links_every_guide_through_the_host() {
        let page = html(Page::List);
        for g in zwipe_core::content::guides::GUIDES {
            assert!(
                page.contains(&format!(r#"href="/base/{}""#, g.slug)),
                "{}: {page}",
                g.slug
            );
        }
    }

    #[test]
    fn an_article_links_its_related_guides_through_the_host_and_its_shots_from_zwipe_net() {
        let guide = zwipe_core::content::guides::GUIDES
            .iter()
            .find(|g| !g.related.is_empty() && !guides::shots(g).is_empty())
            .unwrap();
        let page = html(Page::Article(guide));
        assert!(page.contains("guide-with-gallery"), "{page}");
        assert!(page.contains(&image_url("")), "{page}");
        assert!(page.contains(r#"href="/base/"#), "{page}");
        assert!(page.contains("<h1"), "{page}");
        let tag = guide.tags.first().unwrap();
        let filtered = IndexFilter {
            tag: Some(tag),
            query: String::new(),
        }
        .to_search();
        assert!(
            page.contains(&format!(r#"href="/{filtered}""#)),
            "a tag links to the index filtered to it: {page}"
        );
    }
}

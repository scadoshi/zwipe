//! Oracle-tag dictionary screen.
//!
//! In-app, read-only reference of the full ~4,500-tag oracle-tag catalog with our
//! authored descriptions. **Letter-first browse**: a horizontally scrolling A–Z
//! rail where only the selected letter's tags mount, with optional search over
//! slug + label + description. Reached from the oracle-tag picker and its hint;
//! authed like every app screen (behind the router `AuthGate`).
//!
//! The catalog is read from the shared app-wide [`CatalogCache`] (prefetched at
//! startup, 1-day TTL, stale-while-revalidate), the same copy the oracle-tag
//! picker and card filter read — one fetch of the ~4,500-row list per session.

use crate::{
    inbound::{
        components::{
            catalog_cache::{CatalogCache, CatalogCell},
            hint_dialog::{
                HintBullet, HintBullets, HintColored, HintDialog, HintKey, use_one_time_hint,
            },
            navigation::overlay_stack::use_overlay_back,
            screen_header::ScreenHeader,
        },
        screens::oracle_tag_examples::OracleTagExamples,
    },
    outbound::client::ZwipeClient,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::domain::{
    card::oracle_tag::OracleTag, user::models::hints::HINT_ORACLE_TAG_DICTIONARY,
};

/// Cap on how many rows a search renders, so a broad query can't mount thousands
/// of DOM nodes. Matches are sorted by slug, so the shown set is deterministic.
const SEARCH_RESULT_CAP: usize = 100;

/// The letter bucket a slug falls under: its first character lowercased, or `#`
/// for anything not starting with a letter (e.g. `40k-model`, `5c-...`).
fn bucket_of(slug: &str) -> char {
    match slug.chars().next() {
        Some(c) if c.is_ascii_alphabetic() => c.to_ascii_lowercase(),
        _ => '#',
    }
}

/// Read-only, searchable dictionary of every oracle tag and its description. An
/// in-place overlay (the host renders it while `open`); each row offers **Examples**
/// (opens the example-cards browse, stacked on top) and **Use** (`on_use`, which the
/// host wires to adopt the tag into the deck or filter without leaving the picker).
#[component]
pub fn OracleTagDictionary(mut open: Signal<bool>, on_use: EventHandler<String>) -> Element {
    use_overlay_back(open);
    let client: Signal<ZwipeClient> = use_context();
    let cache: CatalogCache = use_context();
    let toast = use_toast();

    let hint = use_one_time_hint(HINT_ORACLE_TAG_DICTIONARY);
    let mut selected_letter = use_signal(|| 'a');
    let mut query = use_signal(String::new);

    // Nested example-cards browse, stacked above the dictionary. Owned here so the
    // dictionary stays mounted underneath while examples are open.
    let mut examples_open = use_signal(|| false);
    let mut examples_slug = use_signal(String::new);

    // Warm / revalidate the shared oracle-tag catalog. Single-flight and usually
    // already Loaded from the startup prefetch, so this is a cheap no-op on open.
    use_effect(move || {
        cache.ensure_oracle_tags(client);
    });
    let cell = cache.oracle_tags.cell();

    // Surface a hard failure once (no cached data and the fetch failed).
    use_effect(move || {
        if matches!(&*cell.read(), CatalogCell::Failed) {
            toast.error(
                "Couldn't load the oracle-tag dictionary".to_string(),
                ToastOptions::default().duration(Duration::from_millis(3000)),
            );
        }
    });

    let cell_read = cell.read();

    rsx! {
        div { class: "screen dict-overlay",
            ScreenHeader { title: "Oracle tags", hint }

            div { class: "screen-content content-enter",
                match &*cell_read {
                    CatalogCell::Loading => rsx! { DictionarySkeleton {} },
                    CatalogCell::Failed => rsx! {
                        div { class: "dict-empty", "Couldn't load the dictionary. Pull back and try again." }
                    },
                    CatalogCell::Loaded { data: tags, .. } => {
                        let q = query().trim().to_lowercase();
                        let searching = !q.is_empty();
                        let has_hash = tags.iter().any(|t| bucket_of(&t.slug) == '#');

                        // Empty query -> the selected letter's tags. Non-empty -> whole
                        // catalog matched on slug + label + description, capped.
                        let letter = selected_letter();
                        let mut rows: Vec<OracleTag> = if searching {
                            tags.iter()
                                .filter(|t| {
                                    t.slug.to_lowercase().contains(&q)
                                        || t.label.to_lowercase().contains(&q)
                                        || t
                                            .description
                                            .as_deref()
                                            .is_some_and(|d| d.to_lowercase().contains(&q))
                                })
                                .cloned()
                                .collect()
                        } else {
                            tags.iter()
                                .filter(|t| bucket_of(&t.slug) == letter)
                                .cloned()
                                .collect()
                        };
                        rows.sort_by(|a, b| a.slug.cmp(&b.slug));
                        if searching {
                            rows.truncate(SEARCH_RESULT_CAP);
                        }

                        // Switching letters / searching changes every row's key, so
                        // Dioxus remounts the rows and their entrance animation replays
                        // (mirrors the changelog filter pattern, per-row not per-list).
                        rsx! {
                            div { class: "dict-controls",
                                input {
                                    class: "input",
                                    id: "oracle-tag-dictionary-search",
                                    r#type: "text",
                                    placeholder: "Search tags or descriptions",
                                    value: "{query()}",
                                    autocapitalize: "none",
                                    autocorrect: "off",
                                    spellcheck: "false",
                                    oninput: move |event| query.set(event.value()),
                                }

                                div { class: "dict-letter-rail",
                                    for c in ('a'..='z').chain(has_hash.then_some('#')) {
                                        {
                                            let is_sel = !searching && c == letter;
                                            let label = if c == '#' { "#".to_string() } else { c.to_ascii_uppercase().to_string() };
                                            rsx! {
                                                button {
                                                    key: "{c}",
                                                    class: if is_sel { "chip selected" } else { "chip" },
                                                    onclick: move |_| {
                                                        query.set(String::new());
                                                        selected_letter.set(c);
                                                    },
                                                    "{label}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "dict-list",
                                if rows.is_empty() {
                                    if searching {
                                        div { class: "dict-empty", "No tags match \u{201c}{q}\u{201d}." }
                                    } else {
                                        div { class: "dict-empty", "No tags start with {letter.to_ascii_uppercase()}." }
                                    }
                                } else {
                                    for t in rows {
                                        {
                                            let slug_ex = t.slug.clone();
                                            let slug_use = t.slug.clone();
                                            rsx! {
                                                div {
                                                    key: "{t.slug}",
                                                    class: "dict-row",
                                                    div { class: "dict-slug", "{t.slug}" }
                                                    div { class: "dict-desc",
                                                        if let Some(desc) = t.description.clone() {
                                                            "{desc}"
                                                        } else {
                                                            span { class: "dict-desc-missing", "No description yet" }
                                                        }
                                                    }
                                                    if !t.parent_slugs.is_empty() {
                                                        div { class: "dict-parents",
                                                            for parent in t.parent_slugs.iter() {
                                                                span { key: "{parent}", class: "dict-parent", "{parent}" }
                                                            }
                                                        }
                                                    }
                                                    hr { class: "dict-row-divider" }
                                                    div { class: "dict-row-actions",
                                                        button {
                                                            class: "chip",
                                                            onclick: move |_| {
                                                                examples_slug.set(slug_ex.clone());
                                                                examples_open.set(true);
                                                            },
                                                            "Examples"
                                                        }
                                                        button {
                                                            class: "chip",
                                                            onclick: move |_| on_use.call(slug_use.clone()),
                                                            "Use"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| open.set(false),
                    "Back"
                }
            }

            HintDialog { open: hint, title: "Oracle tags",
                HintBullets {
                    HintBullet {
                        "Tap a "
                        HintKey { color: "--accent-tertiary", "letter" }
                        " to browse tags, or "
                        HintColored { color: "--accent-secondary", "search" }
                        " by name or description."
                    }
                    HintBullet {
                        "Tap "
                        HintKey { color: "--accent-primary", "Examples" }
                        " to see real cards that carry a tag."
                    }
                    HintBullet {
                        "Tap "
                        HintKey { color: "--color-success", "Use" }
                        " to add a tag where you came from."
                    }
                    HintBullet {
                        "Descriptions are written by hand over time, so some tags still show "
                        HintColored { color: "--text-muted", "\u{201c}No description yet\u{201d}" }
                        "."
                    }
                }
            }

            if examples_open() {
                OracleTagExamples { open: examples_open, slug: examples_slug() }
            }
        }
    }
}

/// Placeholder shown while the catalog fetch is in flight: a fake letter rail plus
/// a few entry bars, in the app's skeleton language.
#[component]
fn DictionarySkeleton() -> Element {
    rsx! {
        div { class: "dict-controls",
            div { class: "skeleton-bar dict-skeleton-search" }
            div { class: "dict-letter-rail",
                for i in 0..12 {
                    div { key: "{i}", class: "skeleton-bar dict-skeleton-letter" }
                }
            }
        }
        div { class: "dict-list",
            for i in 0..7 {
                div { key: "{i}", class: "dict-row",
                    div { class: "skeleton-bar dict-skeleton-slug" }
                    div { class: "skeleton-bar dict-skeleton-desc" }
                }
            }
        }
    }
}

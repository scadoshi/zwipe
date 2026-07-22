//! Full-screen oracle-tag picker for a deck's declared strategy tags.
//!
//! Mirrors the [`TagSelect`](super::tag_select::TagSelect) overlay, but for the
//! granular oracle tags: it fetches the ~4,500-tag catalog, shows a curated
//! default grid up front (only entries the backend still serves), exposes the
//! rest through search, and writes selected slugs into the deck. Tapping a tag
//! reveals its definition in the pinned bar. Picking a Deck Tag pre-seeds some of
//! these (handled by the host screen); here the user tunes the set.

use crate::{
    inbound::{
        components::{
            catalog_cache::CatalogCache, concept_explainers::OracleTagsExplainer,
            hint_dialog::HintDialog, navigation::overlay_stack::use_overlay_back,
            screen_header::ScreenHeader,
        },
        screens::oracle_tag_dictionary::OracleTagDictionary,
    },
    outbound::client::ZwipeClient,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::domain::{
    card::oracle_tag::{CURATED_ORACLE_TAGS, OracleTag, search_oracle_tags},
    deck::MAX_DECK_ORACLE_TAGS,
};

/// In-place oracle-tag picker. Toggled by `open`; mutates `selected` (slugs)
/// directly. `on_close` returns to the form.
#[component]
pub(crate) fn OracleTagSelect(
    open: Signal<bool>,
    mut selected: Signal<Vec<String>>,
    on_close: EventHandler<()>,
) -> Element {
    // OS back gesture closes this overlay before touching the router.
    use_overlay_back(open);
    let client: Signal<ZwipeClient> = use_context();
    let cache: CatalogCache = use_context();
    let toast = use_toast();
    let mut query = use_signal(String::new);
    let mut focused = use_signal(|| Option::<OracleTag>::None);
    let mut hint_open = use_signal(|| false);
    // Dictionary overlay, stacked above this picker; its Use button adopts a tag.
    let mut dict_open = use_signal(|| false);

    // Adopt a tag chosen via the dictionary's Use button, respecting the deck cap.
    // Stays in place (no return to the form), toasting the outcome.
    let adopt_tag = move |slug: String| {
        let mut current = selected();
        if current.contains(&slug) {
            toast.info(
                "Tag already added".to_string(),
                ToastOptions::default().duration(Duration::from_millis(2000)),
            );
        } else if current.len() < MAX_DECK_ORACLE_TAGS {
            current.push(slug);
            selected.set(current);
            toast.success(
                "Tag added to deck".to_string(),
                ToastOptions::default().duration(Duration::from_millis(2000)),
            );
        } else {
            toast.warning(
                format!("You may only choose up to {MAX_DECK_ORACLE_TAGS} oracle tags"),
                ToastOptions::default().duration(Duration::from_millis(2000)),
            );
        }
    };
    // Snapshot the selection when the picker opens, so Cancel can revert to it.
    let mut snapshot = use_signal(Vec::<String>::new);
    use_effect(move || {
        if open() {
            snapshot.set(selected.peek().clone());
        }
    });

    // Read the shared app-wide oracle-tag catalog (prefetched at startup), warming
    // / revalidating it on open. One copy is shared with the dictionary + filter.
    use_effect(move || {
        cache.ensure_oracle_tags(client);
    });
    let cell = cache.oracle_tags.cell();

    let screen_class = if open() {
        "screen swipe-select-screen show"
    } else {
        "screen swipe-select-screen"
    };

    let cell_read = cell.read();
    let tags: &[OracleTag] = cell_read.loaded().map(Vec::as_slice).unwrap_or(&[]);
    let sel = selected();

    // Empty search → the curated default grid (entries the backend still serves)
    // plus any selected slug not already in it, alphabetical. Non-empty search →
    // the shared ranked catalog search, capped to the 40 best matches.
    let q = query();
    let results: Vec<OracleTag> = if !open() || tags.is_empty() {
        Vec::new()
    } else if q.trim().is_empty() {
        let mut slugs: Vec<String> = CURATED_ORACLE_TAGS
            .iter()
            .filter(|s| tags.iter().any(|t| &t.slug == *s))
            .map(|s| (*s).to_string())
            .collect();
        for s in &sel {
            if !slugs.contains(s) {
                slugs.push(s.clone());
            }
        }
        let mut curated: Vec<OracleTag> = slugs
            .iter()
            .map(|s| {
                tags.iter()
                    .find(|t| &t.slug == s)
                    .cloned()
                    .unwrap_or_else(|| OracleTag {
                        slug: s.clone(),
                        label: s.clone(),
                        description: None,
                        parent_slugs: Vec::new(),
                    })
            })
            .collect();
        curated.sort_by(|a, b| a.slug.cmp(&b.slug));
        curated
    } else {
        let mut matches = search_oracle_tags(tags, &q);
        matches.truncate(40);
        matches
    };

    rsx! {
        div { class: "{screen_class}",
            if open() {
                ScreenHeader { title: "Oracle tags", hint: hint_open }

                div { class: "screen-content content-enter tag-screen",
                    div { class: "tag-controls",
                        div { class: "tag-controls-head",
                            label { class: "tag-search-label", "Search" }
                            Button {
                                variant: ButtonVariant::Small,
                                class: "dict-btn-fill",
                                onclick: move |_| dict_open.set(true),
                                "Dictionary"
                            }
                            span { class: "tag-count", "{sel.len()}/{MAX_DECK_ORACLE_TAGS}" }
                            if !sel.is_empty() {
                                button {
                                    class: "clear-btn",
                                    onclick: move |_| {
                                        selected.set(Vec::new());
                                        focused.set(None);
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }

                        input { class: "input",
                            id: "oracle-tag-search",
                            r#type: "text",
                            placeholder: "Search all oracle tags",
                            value: "{query()}",
                            autocapitalize: "none",
                            autocorrect: "off",
                            spellcheck: "false",
                            oninput: move |event| query.set(event.value()),
                        }

                        div { class: "tag-def-bar",
                            if let Some(t) = focused() {
                                div { class: "tag-def-name", "{t.slug}" }
                                div { class: "tag-def-text",
                                    { t.description.unwrap_or_else(|| "No description for this tag.".to_string()) }
                                }
                            } else {
                                div { class: "tag-def-name", "Hint" }
                                div { class: "tag-def-text",
                                    "Tap a tag to see what it means. Picking a deck tag pre-selects some of these for you."
                                }
                            }
                        }
                    }

                    div { class: "tag-grid",
                        if tags.is_empty() {
                            div { class: "chip-unselected", "Loading tags\u{2026}" }
                        } else if results.is_empty() {
                            div { class: "chip-unselected", "No results" }
                        } else {
                            for t in results {
                                {
                                    let slug = t.slug.clone();
                                    let is_sel = sel.contains(&slug);
                                    let tag_for_focus = t.clone();
                                    rsx! {
                                        div {
                                            key: "{slug}",
                                            class: if is_sel { "chip selected" } else { "chip" },
                                            onclick: move |_| {
                                                focused.set(Some(tag_for_focus.clone()));
                                                // Clear the search so the curated grid returns after picking.
                                                query.set(String::new());
                                                let mut current = selected();
                                                if let Some(pos) = current.iter().position(|s| s == &slug) {
                                                    current.remove(pos);
                                                    selected.set(current);
                                                } else if current.len() < MAX_DECK_ORACLE_TAGS {
                                                    current.push(slug.clone());
                                                    selected.set(current);
                                                } else {
                                                    toast.warning(
                                                        format!("You may only choose up to {MAX_DECK_ORACLE_TAGS} oracle tags"),
                                                        ToastOptions::default().duration(Duration::from_millis(2000)),
                                                    );
                                                }
                                            },
                                            "{t.slug}"
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
                        onclick: move |_| {
                            selected.set(snapshot());
                            on_close.call(());
                        },
                        "Cancel"
                    }
                    Button {
                        variant: ButtonVariant::Util,
                        onclick: move |_| on_close.call(()),
                        "Done"
                    }
                }

                HintDialog {
                    open: hint_open,
                    title: "Oracle tags",
                    actions: rsx! {
                        Button {
                            variant: ButtonVariant::Util,
                            onclick: move |_| {
                                hint_open.set(false);
                                dict_open.set(true);
                            },
                            "Dictionary"
                        }
                    },
                    OracleTagsExplainer {}
                }

                if dict_open() {
                    OracleTagDictionary { open: dict_open, on_use: adopt_tag }
                }
            }
        }
    }
}

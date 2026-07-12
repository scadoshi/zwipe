//! Full-screen oracle-tag picker for a deck's declared strategy tags.
//!
//! Mirrors the [`TagSelect`](super::tag_select::TagSelect) overlay, but for the
//! granular oracle tags: it fetches the ~4,500-tag catalog, shows a curated
//! default grid up front (only entries the backend still serves), exposes the
//! rest through search, and writes selected slugs into the deck. Tapping a tag
//! reveals its definition in the pinned bar. Picking a Deck Tag pre-seeds some of
//! these (handled by the host screen); here the user tunes the set.

use crate::{
    inbound::components::{
        concept_explainers::OracleTagsExplainer, hint_dialog::HintDialog,
        screen_header::ScreenHeader,
    },
    outbound::client::{ZwipeClient, card::get_oracle_tags::ClientGetOracleTags},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe::inbound::http::ApiError;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::domain::{
    card::oracle_tag::{CURATED_ORACLE_TAGS, OracleTag},
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
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();
    let mut query = use_signal(String::new);
    let mut focused = use_signal(|| Option::<OracleTag>::None);
    let hint_open = use_signal(|| false);
    // Snapshot the selection when the picker opens, so Cancel can revert to it.
    let mut snapshot = use_signal(Vec::<String>::new);
    use_effect(move || {
        if open() {
            snapshot.set(selected.peek().clone());
        }
    });

    let catalog: Resource<Result<Vec<OracleTag>, ApiError>> =
        use_resource(move || async move { client().get_oracle_tags().await });

    let screen_class = if open() {
        "screen swipe-select-screen show"
    } else {
        "screen swipe-select-screen"
    };

    let catalog_read = catalog.read();
    let tags: &[OracleTag] = match catalog_read.as_ref() {
        Some(Ok(t)) => t,
        _ => &[],
    };
    let sel = selected();

    // Empty search → the curated default grid (entries the backend still serves)
    // plus any selected slug not already in it. Non-empty search → catalog matches.
    let q = query().to_lowercase();
    let results: Vec<OracleTag> = if !open() || tags.is_empty() {
        Vec::new()
    } else if q.is_empty() {
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
        slugs
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
            .collect()
    } else {
        tags.iter()
            .filter(|t| t.label.to_lowercase().contains(&q) || t.slug.contains(&q))
            .take(40)
            .cloned()
            .collect()
    };

    rsx! {
        div { class: "{screen_class}",
            if open() {
                ScreenHeader { title: "Oracle tags", hint: hint_open }

                div { class: "screen-content content-enter tag-screen",
                    div { class: "tag-controls",
                        div { class: "tag-controls-head",
                            label { class: "tag-search-label", "Search" }
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
                    OracleTagsExplainer {}
                }
            }
        }
    }
}

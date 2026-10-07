//! Full-screen tag picker.
//!
//! Rendered as a sibling overlay above the create/edit form and toggled by
//! `open`: it stays mounted while closed so the search query and scroll
//! position persist. Every [`DeckTag`] shows as a chip in alphabetical order;
//! tapping one toggles selection and reveals its definition in the bar pinned
//! at the top, so users learn what a tag does while picking it. The search box
//! filters the grid by name for quick jumps.

use crate::inbound::components::{
    concept_explainers::DeckTagsExplainer, hint_dialog::HintDialog, screen_header::ScreenHeader,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_components::{
    ActionBar, Button, ButtonVariant, HintBullet, HintBullets, TOAST_QUICK, use_overlay_back_action,
};
use zwipe_core::domain::deck::{DeckTagView, MAX_DECK_TAGS};

/// Whether two slug lists pick the same tags, ignoring tap order.
fn same_set(a: &[String], b: &[String]) -> bool {
    let mut a: Vec<&str> = a.iter().map(String::as_str).collect();
    let mut b: Vec<&str> = b.iter().map(String::as_str).collect();
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

/// In-place tag picker. Toggled by `open`; mutates `selected_tags` (slugs)
/// directly, with Save greyed until the set differs from the one it opened
/// on and Back reverting to it. `on_close` returns to the form. Options come from the server-delivered
/// `catalog` (`GET /api/deck/tags`), so a new deck tag is selectable without a
/// client release.
#[component]
pub(crate) fn TagSelect(
    open: Signal<bool>,
    mut selected_tags: Signal<Vec<String>>,
    catalog: Vec<DeckTagView>,
    on_close: EventHandler<()>,
) -> Element {
    let toast = use_toast();
    let mut query = use_signal(String::new);
    let mut focused = use_signal(|| Option::<DeckTagView>::None);
    let hint_open = use_signal(|| false);
    // Snapshot the selection (tag slugs) when the picker opens, so Cancel reverts.
    let mut snapshot = use_signal(Vec::<String>::new);
    use_effect(move || {
        if open() {
            snapshot.set(selected_tags.peek().clone());
        }
    });

    // OS back gesture closes this picker before the router sees it, and maps to
    // Back rather than Save: it reverts to the snapshot then closes, the same
    // as the Back button below, saying so when there was something to revert.
    let back = use_callback(move |_: ()| {
        let mut selected_tags = selected_tags;
        if !same_set(&selected_tags.peek(), &snapshot.peek()) {
            toast.info(
                "Deck tags unchanged".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
        selected_tags.set(snapshot());
        on_close.call(());
    });
    use_overlay_back_action(open.into(), back);
    let unchanged = same_set(&selected_tags(), &snapshot());

    let screen_class = if open() {
        "screen swipe-select-screen show"
    } else {
        "screen swipe-select-screen"
    };

    let results: Vec<DeckTagView> = if open() {
        let q = query().to_lowercase();
        catalog
            .iter()
            .filter(|t| q.is_empty() || t.display_name.to_lowercase().contains(&q))
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    rsx! {
        div { class: "{screen_class}",
            if open() {
                ScreenHeader { title: "Deck tags", hint: hint_open }

                div { class: "screen-content content-enter tag-screen",
                    div { class: "tag-controls",
                        div { class: "tag-controls-head",
                            label { class: "tag-search-label", "Search" }
                            span { class: "tag-count", "{selected_tags().len()}/{MAX_DECK_TAGS}" }
                            if !selected_tags().is_empty() {
                                button {
                                    class: "clear-btn",
                                    onclick: move |_| {
                                        selected_tags.set(Vec::new());
                                        focused.set(None);
                                    },
                                    "\u{00d7}"
                                }
                            }
                        }

                        input { class: "input",
                            id: "tag-search",
                            r#type: "text",
                            placeholder: "Search tags",
                            value: "{query()}",
                            autocapitalize: "none",
                            autocorrect: "off",
                            spellcheck: "false",
                            oninput: move |event| query.set(event.value()),
                        }

                        div { class: "tag-def-bar",
                            if let Some(tag) = focused() {
                                div { class: "tag-def-name", "{tag.display_name}" }
                                div { class: "tag-def-text", "{tag.description}" }
                            } else {
                                div { class: "tag-def-name", "Hint" }
                                div { class: "tag-def-text", "Tap a tag to see its definition here." }
                            }
                        }
                    }

                    div { class: "tag-grid",
                        if results.is_empty() {
                            div { class: "chip-unselected", "No results" }
                        } else {
                            for tag in results {
                                {
                                    let slug = tag.slug.clone();
                                    let label = tag.display_name.clone();
                                    let view = tag;
                                    let is_selected = selected_tags().contains(&slug);
                                    rsx! {
                                        div {
                                            key: "{slug}",
                                            class: if is_selected { "chip selected" } else { "chip" },
                                            onclick: move |_| {
                                                focused.set(Some(view.clone()));
                                                // Clear the search so the full grid returns after picking.
                                                query.set(String::new());
                                                let mut current = selected_tags();
                                                if let Some(pos) = current.iter().position(|t| *t == slug) {
                                                    current.remove(pos);
                                                    selected_tags.set(current);
                                                } else if current.len() < MAX_DECK_TAGS {
                                                    current.push(slug.clone());
                                                    selected_tags.set(current);
                                                } else {
                                                    toast.warning(
                                                        format!("You may only choose up to {MAX_DECK_TAGS} tags"),
                                                        ToastOptions::default().duration(TOAST_QUICK),
                                                    );
                                                }
                                            },
                                            "{label}"
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
                        onclick: move |_| back.call(()),
                        "Back"
                    }
                    Button {
                        variant: ButtonVariant::Util,
                        disabled: unchanged,
                        onclick: move |_| on_close.call(()),
                        "Save"
                    }
                }

                HintDialog {
                    open: hint_open,
                    title: "Deck tags",
                    DeckTagsExplainer {}
                    HintBullets {
                        HintBullet { "Tap a tag to add or remove it. The bar up top shows what it means" }
                    }
                }
            }
        }
    }
}

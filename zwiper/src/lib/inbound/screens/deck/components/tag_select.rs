//! Full-screen tag picker.
//!
//! Rendered as a sibling overlay above the create/edit form and toggled by
//! `open`: it stays mounted while closed so the search query and scroll
//! position persist. Every [`DeckTag`] shows as a chip in alphabetical order;
//! tapping one toggles selection and reveals its definition in the bar pinned
//! at the top, so users learn what a tag does while picking it. The search box
//! filters the grid by name for quick jumps.

use components::{HintBullet, HintBullets, HintDialog, use_overlay_back_action};
use dioxus::prelude::*;
use zwipe_core::domain::deck::{DeckTagView, MAX_DECK_TAGS};
use zwipe_ui::deck::pickers::{self, Picker};

use crate::inbound::components::action_dock::{self, DockIcon};
use crate::inbound::components::concept_explainers::DeckTagsExplainer;
use crate::inbound::components::guide_link;
use crate::inbound::components::screen_header::ScreenHeader;

/// In-place tag picker, up while `open`, over the server-delivered `catalog` (`GET /api/deck/tags`), so a new deck tag is selectable without a client release. `selected` holds the deck's tag slugs; a chip tap goes to `on_toggle` and the × to `on_clear`. The dock's checkmark (`on_save`) stays off while `unchanged`; the header's back arrow (`on_back`) and the OS back gesture put back the tags it opened on.
#[component]
pub(crate) fn TagSelect(
    open: ReadSignal<bool>,
    selected: Vec<String>,
    catalog: Vec<DeckTagView>,
    unchanged: bool,
    on_toggle: EventHandler<String>,
    on_clear: EventHandler<()>,
    on_save: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element {
    let mut query = use_signal(String::new);
    let mut focused = use_signal(|| Option::<DeckTagView>::None);
    let hint_open = use_signal(|| false);

    // OS back gesture closes this picker before the router sees it, and maps to
    // Back rather than Save, the same as the header's back arrow.
    let back = use_callback(move |(): ()| on_back.call(()));
    use_overlay_back_action(open, back);

    let screen_class = if open() {
        "screen swipe-select-screen show"
    } else {
        "screen swipe-select-screen"
    };

    let results: Vec<DeckTagView> = if open() {
        pickers::deck_tags_matching(&catalog, &query())
            .into_iter()
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    rsx! {
        div { class: "{screen_class}",
            // A screen over another paints its own backdrop, since .screen is
            // clear and the one beneath would show through.
            ::components::GridCells { still: true }
            if open() {
                ScreenHeader { title: Picker::DeckTags.title(), hint: hint_open, back: move |()| back.call(()) }

                div { class: "screen-content content-enter tag-screen",
                    div { class: "tag-controls",
                        div { class: "tag-controls-head",
                            label { class: "tag-search-label", "Search" }
                            span { class: "tag-count", "{selected.len()}/{MAX_DECK_TAGS}" }
                            if !selected.is_empty() {
                                button {
                                    class: "clear-btn",
                                    onclick: move |_| {
                                        on_clear.call(());
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
                                div { class: "tag-def-name", "{pickers::PICKER_BAR_HINT}" }
                                div { class: "tag-def-text", "{Picker::DeckTags.bar_empty()}" }
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
                                    let is_selected = selected.contains(&slug);
                                    rsx! {
                                        div {
                                            key: "{slug}",
                                            class: if is_selected { "chip selected" } else { "chip" },
                                            onclick: move |_| {
                                                focused.set(Some(view.clone()));
                                                // Clear the search so the full grid returns after picking.
                                                query.set(String::new());
                                                on_toggle.call(slug.clone());
                                            },
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Save as the action dock's checkmark, pinned bottom right.
                div { class: "action-dock action-dock-right",
                    button {
                        class: "action-dock-btn",
                        "aria-label": "Save",
                        disabled: unchanged,
                        onclick: move |_| on_save.call(()),
                        DockIcon { path: action_dock::CHECK }
                    }
                }

                HintDialog {
                    open: hint_open,
                    actions: guide_link::screen(zwipe_ui::hint::HintTopic::DeckTags),
                    title: Picker::DeckTags.title(),
                    DeckTagsExplainer {}
                    HintBullets {
                        HintBullet { "Tap a tag to add or remove it. The bar up top shows what it means" }
                    }
                }
            }
        }
    }
}

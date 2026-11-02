//! Full-screen oracle-tag picker for a deck's declared strategy tags.
//!
//! Mirrors the [`TagSelect`](super::tag_select::TagSelect) overlay, but for the
//! granular oracle tags: it fetches the ~4,500-tag catalog, shows a curated
//! default grid up front (only entries the backend still serves), exposes the
//! rest through search, and writes selected slugs into the deck. Tapping a tag
//! reveals its definition in the pinned bar. Picking a Deck Tag pre-seeds some of
//! these (handled by the host screen); here the user tunes the set.

use components::{
    ActionBar, Button, ButtonVariant, DialogAction, HintDialog, use_overlay_back_action,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::use_toast;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::deck::MAX_DECK_ORACLE_TAGS;
use zwipe_ui::deck::pickers::{self, Adopted, Draft, Picker, TagPicker};

use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::concept_explainers::OracleTagsExplainer;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::{guide_link, ui_toast};
use crate::inbound::screens::oracle_tag_dictionary::OracleTagDictionary;

/// In-place oracle-tag picker. Toggled by `open`; mutates `selected` (slugs)
/// directly, with Save grayed until the set differs from the one it opened
/// on and Back reverting to it. `on_close` returns to the form.
#[component]
pub(crate) fn OracleTagSelect(
    open: Signal<bool>,
    mut selected: Signal<Vec<String>>,
    on_close: EventHandler<()>,
) -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let cache: CatalogCache = use_context();
    let toast = use_toast();
    let mut query = use_signal(String::new);
    let mut focused = use_signal(|| Option::<OracleTag>::None);
    let hint_open = use_signal(|| false);
    // Dictionary overlay, stacked above this picker; its Use button adopts a tag.
    let mut dict_open = use_signal(|| false);
    let open_dictionary = use_callback(move |()| dict_open.set(true));

    // Adopt a tag chosen via the dictionary's Use button, respecting the deck cap.
    // Stays in place (no return to the form), toasting the outcome.
    let adopt_tag = move |slug: String| {
        let mut current = selected();
        let adopted = pickers::adopt_oracle_tag(&mut current, slug);
        if adopted == Adopted::Added {
            selected.set(current);
        }
        ui_toast::raise(toast, adopted.toast());
    };
    // Snapshot the selection when the picker opens, so Back can revert to it.
    let mut draft = use_signal(|| Draft::tags(TagPicker::OracleTags, Vec::new()));
    use_effect(move || {
        if open() {
            draft.set(Draft::tags(TagPicker::OracleTags, selected.peek().clone()));
        }
    });

    // Back, and the OS back gesture, revert to the snapshot and say so when
    // there was something to revert.
    let back = use_callback(move |(): ()| {
        let mut selected = selected;
        if let Some(discarded) = draft.peek().back_toast(&selected.peek()) {
            ui_toast::raise(toast, discarded);
        }
        selected.set(draft().into_snapshot());
        on_close.call(());
    });
    use_overlay_back_action(open.into(), back);
    let unchanged = draft.read().is_unchanged(&selected());

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
    let tags: &[OracleTag] = cell_read.loaded().map_or(&[], Vec::as_slice);
    let sel = selected();

    // Empty search shows the server-marked curated grid plus any selected
    // slug, alphabetical; a search shows the best ranked matches.
    let q = query();
    let results: Vec<OracleTag> = if !open() || tags.is_empty() {
        Vec::new()
    } else {
        pickers::oracle_tag_results(tags, &sel, &q)
    };

    rsx! {
        div { class: "{screen_class}",
            // A screen over another paints its own backdrop, since .screen is
            // clear and the one beneath would show through.
            ::components::GridCells { still: true }
            if open() {
                ScreenHeader { title: Picker::OracleTags.title(), hint: hint_open }

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
                                    { t.description.unwrap_or_else(|| pickers::ORACLE_TAG_NO_DESCRIPTION.to_string()) }
                                }
                            } else {
                                div { class: "tag-def-name", "{pickers::PICKER_BAR_HINT}" }
                                div { class: "tag-def-text", "{Picker::OracleTags.bar_empty()}" }
                            }
                        }
                    }

                    div { class: "tag-grid",
                        if tags.is_empty() {
                            // Bare chips of mixed widths, like a page of tags.
                            for (i , size) in ["lg", "md", "xl", "sm", "md", "lg", "md", "sm", "xl", "md", "lg", "sm"].iter().enumerate() {
                                div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-{size}" }
                            }
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
                                                match TagPicker::OracleTags.toggle(&mut current, &slug) {
                                                    Some(cap) => ui_toast::raise(toast, cap),
                                                    None => selected.set(current),
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
                        variant: ButtonVariant::Action,
                        onclick: move |_| back.call(()),
                        "Back"
                    }
                    Button {
                        variant: ButtonVariant::Action,
                        disabled: unchanged,
                        onclick: move |_| on_close.call(()),
                        "Save"
                    }
                }

                HintDialog {
                    open: hint_open,
                    title: Picker::OracleTags.title(),
                    actions: [DialogAction::Confirm {
                        label: "Dictionary".into(),
                        danger: false,
                        on_confirm: open_dictionary,
                    }]
                    .into_iter()
                    .chain(guide_link::screen(zwipe_ui::hint::HintTopic::OracleTags))
                    .collect(),
                    OracleTagsExplainer {}
                }

                if dict_open() {
                    OracleTagDictionary { open: dict_open, on_use: adopt_tag }
                }
            }
        }
    }
}

//! The form's three pickers: format, deck tags and oracle tags. Each is the
//! app's full-screen picker below the breakpoint and a side panel over the
//! form's right column on a wide window: the header's back arrow, the
//! search, the definition bar, the chip grid, and Save as the checkmark in
//! the dock at its bottom right.
//!
//! Each opens on a snapshot. The back arrow (and Esc) puts the snapshot
//! back and says so when something changed; the checkmark keeps the change
//! and stays off while nothing has. `/` focuses the search.

use components::hint::{ContentHintDialog, HintBody};
use components::icons::{CHECK, DockIcon};
use components::{
    Button, ButtonVariant, DialogAction, HintDialog, PageHeader, TOAST_NORMAL, TOAST_QUICK,
    use_overlay_back_action,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::oracle_tag::{self, OracleTag};
use zwipe_core::domain::deck::{
    DeckTagView, Format, MAX_DECK_ORACLE_TAGS, MAX_DECK_TAGS, seed_oracle_tags_from_catalog,
};
use zwipe_ui::deck::form::{format_details, reseed_oracle_tags};
use zwipe_ui::filter::fields::same_set;
use zwipe_ui::timing::ORACLE_TAG_PICKER_LIMIT;

use super::DeckForm;
use crate::domain::deck::{
    self, DECK_TAG_BAR_EMPTY, FORMAT_BAR_EMPTY, ORACLE_TAG_BAR_EMPTY, ORACLE_TAG_NO_DESCRIPTION,
    PICKER_BAR_HINT,
};
use crate::inbound::catalog::CatalogCache;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::back_arrow::BackArrow;
use crate::inbound::chrome::page_keys::{focus_by_id, use_page_keys};
use crate::inbound::screens::oracle_tag_dictionary::OracleTagDictionary;

/// The picker's shell: hidden and click-through while closed, its header,
/// body and dock while open. Mounted only while open, so its keys never
/// answer under a closed picker. The header's back arrow calls `on_back`;
/// the dock's checkmark calls `on_save` and stays off while `unchanged`.
#[component]
fn PickerFrame(
    open: Signal<bool>,
    title: &'static str,
    search_id: &'static str,
    hint_open: Signal<bool>,
    unchanged: bool,
    on_back: EventHandler<()>,
    on_save: EventHandler<()>,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: if open() { "screen swipe-select-screen web-picker show" } else { "screen swipe-select-screen web-picker" },
            components::GridCells { still: true }
            if open() {
                PickerBody { title, search_id, hint_open, unchanged, on_back, on_save, {children} }
            }
        }
    }
}

/// The open picker's parts, with `/` focusing its search.
#[component]
fn PickerBody(
    title: &'static str,
    search_id: &'static str,
    mut hint_open: Signal<bool>,
    unchanged: bool,
    on_back: EventHandler<()>,
    on_save: EventHandler<()>,
    children: Element,
) -> Element {
    use_page_keys(Callback::new(move |key: String| {
        if key == "/" {
            focus_by_id(search_id);
        }
    }));
    rsx! {
        PageHeader {
            title,
            leading: rsx! {
                BackArrow { class: "page-header-support", onclick: move |()| on_back.call(()) }
            },
            on_hint: move |()| hint_open.set(true),
        }
        div { class: "screen-content content-enter tag-screen", {children} }
        // Save as the action dock's checkmark, pinned bottom right.
        div { class: "action-dock action-dock-right",
            button {
                class: "action-dock-btn",
                "aria-label": "Save",
                disabled: unchanged,
                onclick: move |_| on_save.call(()),
                DockIcon { path: CHECK }
            }
        }
    }
}

/// The search input every picker leads with.
#[component]
fn PickerSearch(id: &'static str, placeholder: &'static str, mut query: Signal<String>) -> Element {
    rsx! {
        input {
            class: "input",
            id,
            r#type: "text",
            placeholder,
            value: "{query}",
            autocapitalize: "none",
            autocorrect: "off",
            spellcheck: "false",
            oninput: move |event| query.set(event.value()),
        }
    }
}

/// What the format picker opened on: the format and what its cascade
/// clears, so `Back` can put them back.
#[derive(Debug, Clone, PartialEq, Default)]
struct FormatSnapshot {
    format: Option<Format>,
    commander: Option<Card>,
    commander_text: String,
    spell: Option<Card>,
    spell_text: String,
}

/// The format picker: single select, with the format's details in the bar.
/// Picking runs the form's cascade at once; tapping the picked format clears
/// it.
#[component]
pub fn FormatPicker(form: DeckForm) -> Element {
    let toast = use_toast();
    let mut open = form.format_open;
    let mut snapshot = use_signal(FormatSnapshot::default);
    let query = use_signal(String::new);
    let mut focused: Signal<Option<Format>> = use_signal(|| None);
    let hint_open = use_signal(|| false);
    use_effect(move || {
        if open() {
            snapshot.set(FormatSnapshot {
                format: *form.format.peek(),
                commander: form.commander.card.peek().clone(),
                commander_text: form.commander.text.peek().clone(),
                spell: form.signature_spell.card.peek().clone(),
                spell_text: form.signature_spell.text.peek().clone(),
            });
        }
    });
    let back = use_callback(move |(): ()| {
        let held = snapshot.peek().clone();
        if *form.format.peek() != held.format {
            toast.info(
                "Format unchanged".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
        let mut format = form.format;
        format.set(held.format);
        let (mut card, mut text) = (form.commander.card, form.commander.text);
        card.set(held.commander);
        text.set(held.commander_text);
        let (mut card, mut text) = (form.signature_spell.card, form.signature_spell.text);
        card.set(held.spell);
        text.set(held.spell_text);
        open.set(false);
    });
    use_overlay_back_action(open.into(), back);
    let unchanged = (form.format)() == snapshot.read().format;
    let typed = query().to_lowercase();
    let results: Vec<Format> = Format::all()
        .iter()
        .copied()
        .filter(|f| typed.is_empty() || f.display_name().to_lowercase().contains(&typed))
        .collect();
    rsx! {
        PickerFrame {
            open,
            title: "Format",
            search_id: "format-search",
            hint_open,
            unchanged,
            on_back: move |()| back.call(()),
            on_save: move |()| open.set(false),
            div { class: "tag-controls",
                div { class: "tag-controls-head",
                    label { class: "tag-search-label", r#for: "format-search", "Search" }
                }
                PickerSearch { id: "format-search", placeholder: "Search formats", query }
                div { class: "tag-def-bar",
                    if let Some(format) = focused() {
                        div { class: "tag-def-name", "{format.display_name()}" }
                        ul { class: "tag-def-list",
                            for line in format_details(format) {
                                li { "{line}" }
                            }
                        }
                    } else {
                        div { class: "tag-def-name", "{PICKER_BAR_HINT}" }
                        div { class: "tag-def-text", "{FORMAT_BAR_EMPTY}" }
                    }
                }
            }
            div { class: "tag-grid",
                if results.is_empty() {
                    div { class: "chip-unselected", "No results" }
                } else {
                    for format in results {
                        div {
                            key: "{format.display_name()}",
                            class: if (form.format)() == Some(format) { "chip selected" } else { "chip" },
                            onclick: move |_| {
                                focused.set(Some(format));
                                if *form.format.peek() == Some(format) {
                                    form.clear_format();
                                } else {
                                    form.select_format(format);
                                }
                            },
                            "{format.display_name()}"
                        }
                    }
                }
            }
        }
        ContentHintDialog { open: hint_open, content: deck::format_picker_hint() }
    }
}

/// The deck tag picker over the server's catalog, at most five. On close,
/// either way, the oracle tags follow: the old seed goes, hand picks stay,
/// and the new seed is added while there is room.
#[component]
pub fn DeckTagPicker(form: DeckForm) -> Element {
    let toast = use_toast();
    let authed = use_authed();
    let cache: CatalogCache = use_context();
    use_hook(move || cache.ensure_deck_tags(authed));
    let catalog = use_memo(move || -> Vec<DeckTagView> {
        cache
            .0
            .read()
            .deck_tags
            .ready()
            .cloned()
            .unwrap_or_default()
    });
    let mut open = form.tags_open;
    let mut tags = form.tags;
    let mut snapshot: Signal<Vec<String>> = use_signal(Vec::new);
    let query = use_signal(String::new);
    let mut focused: Signal<Option<DeckTagView>> = use_signal(|| None);
    let hint_open = use_signal(|| false);
    use_effect(move || {
        if open() {
            snapshot.set(tags.peek().clone());
        }
    });

    let reconcile = move || {
        let new_seed = seed_oracle_tags_from_catalog(&tags.peek(), &catalog.peek());
        let (kept, added) = reseed_oracle_tags(
            &form.oracle_tags.peek(),
            &form.applied_seed.peek(),
            &new_seed,
        );
        let (mut oracle_tags, mut applied) = (form.oracle_tags, form.applied_seed);
        oracle_tags.set(kept);
        applied.set(new_seed);
        if added {
            toast.info(
                "Oracle tags derived from your deck tags".to_string(),
                ToastOptions::default().duration(TOAST_NORMAL),
            );
        }
    };
    let back = use_callback(move |(): ()| {
        if !same_set(&tags.peek(), &snapshot.peek()) {
            toast.info(
                "Deck tags unchanged".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
        tags.set(snapshot());
        open.set(false);
        reconcile();
    });
    use_overlay_back_action(open.into(), back);
    let unchanged = same_set(&tags(), &snapshot());
    let typed = query().to_lowercase();
    let results: Vec<DeckTagView> = catalog()
        .into_iter()
        .filter(|t| typed.is_empty() || t.display_name.to_lowercase().contains(&typed))
        .collect();
    let count = tags.read().len();
    rsx! {
        PickerFrame {
            open,
            title: "Deck tags",
            search_id: "tag-search",
            hint_open,
            unchanged,
            on_back: move |()| back.call(()),
            on_save: move |()| {
                open.set(false);
                reconcile();
            },
            div { class: "tag-controls",
                div { class: "tag-controls-head",
                    label { class: "tag-search-label", r#for: "tag-search", "Search" }
                    span { class: "tag-count", "{count}/{MAX_DECK_TAGS}" }
                    if count > 0 {
                        button {
                            class: "clear-btn",
                            onclick: move |_| {
                                tags.set(Vec::new());
                                focused.set(None);
                            },
                            "\u{00d7}"
                        }
                    }
                }
                PickerSearch { id: "tag-search", placeholder: "Search tags", query }
                div { class: "tag-def-bar",
                    if let Some(tag) = focused() {
                        div { class: "tag-def-name", "{tag.display_name}" }
                        div { class: "tag-def-text", "{tag.description}" }
                    } else {
                        div { class: "tag-def-name", "{PICKER_BAR_HINT}" }
                        div { class: "tag-def-text", "{DECK_TAG_BAR_EMPTY}" }
                    }
                }
            }
            div { class: "tag-grid",
                if results.is_empty() {
                    div { class: "chip-unselected", "No results" }
                } else {
                    for tag in results {
                        DeckTagChip { key: "{tag.slug}", tag, tags, focused, query }
                    }
                }
            }
        }
        ContentHintDialog { open: hint_open, content: deck::deck_tag_picker_hint() }
    }
}

/// One deck tag in the grid: a tap shows its definition, clears the search
/// and toggles it, warning at the cap.
#[component]
fn DeckTagChip(
    tag: DeckTagView,
    mut tags: Signal<Vec<String>>,
    mut focused: Signal<Option<DeckTagView>>,
    mut query: Signal<String>,
) -> Element {
    let toast = use_toast();
    let picked = tags.read().contains(&tag.slug);
    let label = tag.display_name.clone();
    rsx! {
        div {
            class: if picked { "chip selected" } else { "chip" },
            onclick: move |_| {
                focused.set(Some(tag.clone()));
                query.set(String::new());
                let mut current = tags.write();
                if let Some(at) = current.iter().position(|t| *t == tag.slug) {
                    current.remove(at);
                } else if current.len() < MAX_DECK_TAGS {
                    current.push(tag.slug.clone());
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

/// The oracle tag picker over the oracle tag catalog, at most thirty. With
/// no search it shows the curated tags and every picked one; a search ranks
/// the whole catalog. `Dictionary` opens the dictionary, whose `Use` adds a
/// tag without leaving the picker.
#[component]
pub fn OracleTagPicker(form: DeckForm) -> Element {
    let toast = use_toast();
    let client: Signal<ZwipeClient> = use_context();
    let cache: CatalogCache = use_context();
    use_hook(move || cache.ensure_oracle_tags(client));
    let mut open = form.oracle_tags_open;
    let mut selected = form.oracle_tags;
    let mut snapshot: Signal<Vec<String>> = use_signal(Vec::new);
    let query = use_signal(String::new);
    let mut focused: Signal<Option<OracleTag>> = use_signal(|| None);
    let hint_open = use_signal(|| false);
    let mut dictionary_open = use_signal(|| false);
    use_effect(move || {
        if open() {
            snapshot.set(selected.peek().clone());
        }
    });
    let back = use_callback(move |(): ()| {
        if !same_set(&selected.peek(), &snapshot.peek()) {
            toast.info(
                "Oracle tags unchanged".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
        selected.set(snapshot());
        open.set(false);
    });
    use_overlay_back_action(open.into(), back);
    let adopt = move |slug: String| {
        let mut current = selected.write();
        if current.contains(&slug) {
            toast.info(
                "Tag already added".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        } else if current.len() < MAX_DECK_ORACLE_TAGS {
            current.push(slug);
            toast.success(
                "Tag added to deck".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        } else {
            toast.warning(
                format!("You may only choose up to {MAX_DECK_ORACLE_TAGS} oracle tags"),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
    };

    let unchanged = same_set(&selected(), &snapshot());
    let picked = selected();
    let typed = query();
    let catalog = cache.0.read();
    let all: &[OracleTag] = catalog.oracle_tags.ready().map_or(&[], Vec::as_slice);
    let loaded = !all.is_empty();
    let results: Vec<OracleTag> = if !open() || !loaded {
        Vec::new()
    } else if typed.trim().is_empty() {
        let mut curated: Vec<OracleTag> = all
            .iter()
            .filter(|t| t.curated || picked.contains(&t.slug))
            .cloned()
            .collect();
        curated.sort_by(|a, b| a.slug.cmp(&b.slug));
        curated
    } else {
        let mut ranked = oracle_tag::search_oracle_tags(all, &typed);
        ranked.truncate(ORACLE_TAG_PICKER_LIMIT);
        ranked
    };
    drop(catalog);
    let content = deck::oracle_tag_picker_hint();
    rsx! {
        PickerFrame {
            open,
            title: "Oracle tags",
            search_id: "oracle-tag-search",
            hint_open,
            unchanged,
            on_back: move |()| back.call(()),
            on_save: move |()| open.set(false),
            div { class: "tag-controls",
                div { class: "tag-controls-head",
                    label { class: "tag-search-label", r#for: "oracle-tag-search", "Search" }
                    Button {
                        variant: ButtonVariant::Small,
                        class: "dict-btn-fill",
                        onclick: move |_| dictionary_open.set(true),
                        "Dictionary"
                    }
                    span { class: "tag-count", "{picked.len()}/{MAX_DECK_ORACLE_TAGS}" }
                    if !picked.is_empty() {
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
                PickerSearch { id: "oracle-tag-search", placeholder: "Search all oracle tags", query }
                div { class: "tag-def-bar",
                    if let Some(tag) = focused() {
                        div { class: "tag-def-name", "{tag.slug}" }
                        div { class: "tag-def-text",
                            {tag.description.unwrap_or_else(|| ORACLE_TAG_NO_DESCRIPTION.to_string())}
                        }
                    } else {
                        div { class: "tag-def-name", "{PICKER_BAR_HINT}" }
                        div { class: "tag-def-text", "{ORACLE_TAG_BAR_EMPTY}" }
                    }
                }
            }
            div { class: "tag-grid",
                if !loaded {
                    for (i, size) in ["lg", "md", "xl", "sm", "md", "lg", "md", "sm", "xl", "md", "lg", "sm"].iter().enumerate() {
                        div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-{size}" }
                    }
                } else if results.is_empty() {
                    div { class: "chip-unselected", "No results" }
                } else {
                    for tag in results {
                        OracleTagChip { key: "{tag.slug}", tag, selected, focused, query }
                    }
                }
            }
        }
        HintDialog {
            open: hint_open,
            title: content.title,
            actions: vec![
                DialogAction::Confirm {
                    label: "Dictionary".into(),
                    danger: false,
                    on_confirm: Callback::new(move |()| dictionary_open.set(true)),
                },
            ],
            HintBody { blocks: content.blocks }
        }
        if dictionary_open() {
            OracleTagDictionary { open: dictionary_open, on_use: adopt }
        }
    }
}

/// One oracle tag in the grid: a tap shows its definition, clears the
/// search and toggles it, warning at the cap.
#[component]
fn OracleTagChip(
    tag: OracleTag,
    mut selected: Signal<Vec<String>>,
    mut focused: Signal<Option<OracleTag>>,
    mut query: Signal<String>,
) -> Element {
    let toast = use_toast();
    let picked = selected.read().contains(&tag.slug);
    let slug = tag.slug.clone();
    rsx! {
        div {
            class: if picked { "chip selected" } else { "chip" },
            onclick: move |_| {
                focused.set(Some(tag.clone()));
                query.set(String::new());
                let mut current = selected.write();
                if let Some(at) = current.iter().position(|s| *s == tag.slug) {
                    current.remove(at);
                } else if current.len() < MAX_DECK_ORACLE_TAGS {
                    current.push(tag.slug.clone());
                } else {
                    toast.warning(
                        format!("You may only choose up to {MAX_DECK_ORACLE_TAGS} oracle tags"),
                        ToastOptions::default().duration(TOAST_QUICK),
                    );
                }
            },
            "{slug}"
        }
    }
}

//! The form's three pickers: format, deck tags and oracle tags. Each is the
//! kit's sheet as the card filter draws it: a drawer along the right edge on
//! a wide window and a bottom sheet below the breakpoint, with the title
//! bar's ×, the search, the definition bar, the chip grid, and Save as the
//! checkmark in the dock at its bottom right.
//!
//! Each opens on a snapshot. The ×, a backdrop tap and Esc close quietly
//! when nothing changed, and ask first when something did: Keep stays in
//! the picker, Discard puts the snapshot back. The checkmark keeps the
//! change and stays off while nothing has. `/` focuses the search.

use components::hint::{ContentHintDialog, HintBody, guide_for_screen};
use components::icons::CHECK;
use components::{
    AlertDialog, Button, ButtonVariant, ClearWrap, DialogAction, DockAction, DrawerSheet,
    HintDialog, TOAST_NORMAL, TOAST_QUICK, use_overlay_back_action,
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
use zwipe_ui::deck::pickers::{
    DECK_TAG_BAR_EMPTY, DISCARD, DISCARD_BODY, FORMAT_BAR_EMPTY, KEEP, ORACLE_TAG_BAR_EMPTY,
    ORACLE_TAG_NO_DESCRIPTION, PICKER_BAR_HINT, Picker,
};
use zwipe_ui::filter::fields::same_set;
use zwipe_ui::hint::HintTopic;
use zwipe_ui::timing::ORACLE_TAG_PICKER_LIMIT;

use super::DeckForm;
use crate::inbound::catalog::CatalogCache;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::page_keys::{focus_by_id, use_page_keys};
use crate::inbound::screens::oracle_tag_dictionary::OracleTagDictionary;

/// The picker's sheet, components' [`DrawerSheet`] as a drawer at every
/// width, sliding in from the right on `open`: the kit's drawer on a wide
/// window, the window's height along the right edge below it, and the whole
/// window over the grid backdrop where a drawer no longer fits. The
/// title bar's × and a backdrop tap call `on_back`; the dock's checkmark
/// calls `on_save` and stays off while `unchanged`. The body mounts only
/// while open, so its keys never answer under a closed picker.
#[component]
fn PickerSheet(
    open: Signal<bool>,
    title: &'static str,
    search_id: &'static str,
    mut hint_open: Signal<bool>,
    unchanged: bool,
    on_back: EventHandler<()>,
    on_save: EventHandler<()>,
    children: Element,
) -> Element {
    rsx! {
        DrawerSheet {
            open,
            title,
            on_close: move |()| on_back.call(()),
            on_hint: move |()| hint_open.set(true),
            class: "web-picker-sheet",
            every_width: true,
            dock: rsx! {
                DockAction {
                    label: "Save",
                    icon: CHECK,
                    disabled: unchanged,
                    onclick: move |()| on_save.call(()),
                }
            },
            PickerBody { search_id, {children} }
        }
    }
}

/// The open picker's body, with `/` focusing its search.
#[component]
fn PickerBody(search_id: &'static str, children: Element) -> Element {
    use_page_keys(Callback::new(move |key: String| {
        if key == "/" {
            focus_by_id(search_id);
        }
    }));
    rsx! {
        div { class: "web-picker-body",
            components::GridCells { still: true }
            {children}
        }
    }
}

/// The search input every picker leads with, its `×` easing in as text is
/// typed.
#[component]
fn PickerSearch(id: &'static str, placeholder: &'static str, mut query: Signal<String>) -> Element {
    rsx! {
        ClearWrap {
            filled: !query.read().is_empty(),
            on_clear: move |()| query.set(String::new()),
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
}

/// Asks before a picker's ×, backdrop tap or Esc throws its picks away. Keep,
/// a tap on the dim and Esc leave the picker open; Discard calls
/// `on_discard`.
#[component]
fn DiscardDialog(open: Signal<bool>, picker: Picker, on_discard: Callback<()>) -> Element {
    rsx! {
        AlertDialog {
            open,
            title: picker.discard_title().to_owned(),
            actions: vec![
                DialogAction::Dismiss(KEEP.into()),
                DialogAction::Confirm {
                    label: DISCARD.into(),
                    danger: true,
                    on_confirm: on_discard,
                },
            ],
            {DISCARD_BODY}
        }
    }
}

/// What the format picker opened on: the format, the land target a pick
/// moves and what its cascade clears, so `Back` can put them back.
#[derive(Debug, Clone, PartialEq, Default)]
struct FormatSnapshot {
    format: Option<Format>,
    land_target: Option<i32>,
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
    let mut open = form.format_open;
    let mut snapshot = use_signal(FormatSnapshot::default);
    let query = use_signal(String::new);
    let mut focused: Signal<Option<Format>> = use_signal(|| None);
    let hint_open = use_signal(|| false);
    let mut discard_open = use_signal(|| false);
    use_effect(move || {
        if open() {
            focused.set(None);
            snapshot.set(FormatSnapshot {
                format: *form.format.peek(),
                land_target: *form.land_target.peek(),
                commander: form.commander.card.peek().clone(),
                commander_text: form.commander.text.peek().clone(),
                spell: form.signature_spell.card.peek().clone(),
                spell_text: form.signature_spell.text.peek().clone(),
            });
        }
    });
    let revert = use_callback(move |(): ()| {
        let held = snapshot.peek().clone();
        let mut format = form.format;
        format.set(held.format);
        let mut land_target = form.land_target;
        land_target.set(held.land_target);
        let (mut card, mut text) = (form.commander.card, form.commander.text);
        card.set(held.commander);
        text.set(held.commander_text);
        let (mut card, mut text) = (form.signature_spell.card, form.signature_spell.text);
        card.set(held.spell);
        text.set(held.spell_text);
        open.set(false);
    });
    let back = use_callback(move |(): ()| {
        if *form.format.peek() == snapshot.peek().format {
            revert.call(());
        } else {
            discard_open.set(true);
        }
    });
    // Off the stack while it asks, so Keep puts it back on top.
    let listening = use_memo(move || open() && !discard_open());
    use_overlay_back_action(listening.into(), back);
    let unchanged = (form.format)() == snapshot.read().format;
    let typed = query().to_lowercase();
    let results: Vec<Format> = Format::all()
        .iter()
        .copied()
        .filter(|f| typed.is_empty() || f.display_name().to_lowercase().contains(&typed))
        .collect();
    rsx! {
        PickerSheet {
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
                div { class: "tag-def-bar format-def-bar",
                    div { class: "format-def-scroll",
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
        ContentHintDialog {
            open: hint_open,
            content: Picker::Format.hint(),
            actions: guide_for_screen(HintTopic::Format),
        }
        DiscardDialog { open: discard_open, picker: Picker::Format, on_discard: revert }
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
    let mut discard_open = use_signal(|| false);
    use_effect(move || {
        if open() {
            focused.set(None);
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
    let revert = use_callback(move |(): ()| {
        tags.set(snapshot());
        open.set(false);
        reconcile();
    });
    let back = use_callback(move |(): ()| {
        if same_set(&tags.peek(), &snapshot.peek()) {
            revert.call(());
        } else {
            discard_open.set(true);
        }
    });
    // Off the stack while it asks, so Keep puts it back on top.
    let listening = use_memo(move || open() && !discard_open());
    use_overlay_back_action(listening.into(), back);
    let unchanged = same_set(&tags(), &snapshot());
    let typed = query().to_lowercase();
    let results: Vec<DeckTagView> = catalog()
        .into_iter()
        .filter(|t| typed.is_empty() || t.display_name.to_lowercase().contains(&typed))
        .collect();
    let count = tags.read().len();
    rsx! {
        PickerSheet {
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
        ContentHintDialog {
            open: hint_open,
            content: Picker::DeckTags.hint(),
            actions: guide_for_screen(HintTopic::DeckTags),
        }
        DiscardDialog { open: discard_open, picker: Picker::DeckTags, on_discard: revert }
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
    let mut discard_open = use_signal(|| false);
    use_effect(move || {
        if open() {
            focused.set(None);
            snapshot.set(selected.peek().clone());
        }
    });
    let revert = use_callback(move |(): ()| {
        selected.set(snapshot());
        open.set(false);
    });
    let back = use_callback(move |(): ()| {
        if same_set(&selected.peek(), &snapshot.peek()) {
            revert.call(());
        } else {
            discard_open.set(true);
        }
    });
    // Off the stack while it asks, so Keep puts it back on top.
    let listening = use_memo(move || open() && !discard_open());
    use_overlay_back_action(listening.into(), back);
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
    let content = Picker::OracleTags.hint();
    rsx! {
        PickerSheet {
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
            actions: std::iter::once(DialogAction::Confirm {
                label: "Dictionary".into(),
                danger: false,
                on_confirm: Callback::new(move |()| dictionary_open.set(true)),
            })
            .chain(guide_for_screen(HintTopic::OracleTags))
            .collect::<Vec<_>>(),
            HintBody { blocks: content.blocks }
        }
        if dictionary_open() {
            OracleTagDictionary { open: dictionary_open, on_use: adopt }
        }
        DiscardDialog { open: discard_open, picker: Picker::OracleTags, on_discard: revert }
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

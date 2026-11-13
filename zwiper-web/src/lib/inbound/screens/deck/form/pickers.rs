//! The form's three pickers: format, deck tags and oracle tags. Each is a
//! drawer along the right edge with the title bar's × from 600px, and a
//! screen over the form with a back arrow narrower, as the app's are; either
//! has the search, the definition bar, the chip grid, and Save as the
//! checkmark in the dock at its bottom right.
//!
//! Each opens on a snapshot. The ×, a backdrop tap and Esc close quietly
//! when nothing changed, and ask first when something did: Keep stays in
//! the picker, Discard puts the snapshot back. The checkmark keeps the
//! change and stays off while nothing has. `/` focuses the search.

use components::hint::{ContentHintDialog, HintBody, guide_for_screen};
use components::icons::CHECK;
use components::keys::{SHEET_ENTER_OR_CLOSE, focus_by_id, use_page_keys, use_sheet_focus};
use components::{
    ActionDock, AlertDialog, BackArrow, Button, ButtonVariant, Chip, ClearWrap, DialogAction,
    DockAction, DrawerSheet, HintDialog, PageHeader, TOAST_NORMAL, TOAST_QUICK, use_min_width,
    use_overlay_back_action, use_picker_search,
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

use super::{DeckForm, SlotState};
use crate::inbound::catalog::CatalogCache;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::screens::oracle_tag_dictionary::OracleTagDictionary;

/// The width, in pixels, from which a picker is a drawer beside the form; narrower it is a
/// screen over it.
const PICKER_DRAWER_PX: u32 = 600;

/// Whether an open picker covers the form, as a screen over another does: on a window
/// narrower than [`PICKER_DRAWER_PX`] (`narrow`) while any of the three (`open`) is open.
pub(crate) fn covers_form(narrow: bool, open: [bool; 3]) -> bool {
    narrow && open.contains(&true)
}

/// Whether the form's pickers cover it now ([`covers_form`]), kept current.
pub(crate) fn use_pickers_cover(form: &DeckForm) -> bool {
    let wide = use_min_width(PICKER_DRAWER_PX);
    covers_form(
        !wide(),
        [
            (form.format_open)(),
            (form.tags_open)(),
            (form.oracle_tags_open)(),
        ],
    )
}

/// What takes focus as a picker opens as a screen: its back arrow, so no field raises a phone's
/// keyboard.
const SCREEN_ENTRY: &str = ".web-picker-screen.show .page-header-support";

/// The picker, open on `open`. From [`PICKER_DRAWER_PX`] it is components' [`DrawerSheet`] as
/// a drawer at every width, sliding in from the right: the kit's drawer on a wide window, the
/// window's height along the right edge below it, its title bar's × and a backdrop tap calling
/// `on_back`. Narrower it is a screen over the form, as the app's pickers are: it fades in with
/// no slide, its header's back arrow calls `on_back`, and the form under it is hidden
/// ([`use_pickers_cover`]). The dock's checkmark calls `on_save` and stays off while
/// `unchanged`. The body mounts while open (and, as a drawer, through the slide out), so its
/// keys stop answering once the picker is gone. As a screen its back arrow takes focus as it
/// opens and the opener gets it back as it closes; as a drawer, its ×.
#[component]
fn PickerSheet(
    open: Signal<bool>,
    title: &'static str,
    search_id: &'static str,
    mut hint_open: Signal<bool>,
    unchanged: bool,
    on_back: EventHandler<()>,
    on_save: EventHandler<()>,
    layer: Option<u32>,
    children: Element,
) -> Element {
    let wide = use_min_width(PICKER_DRAWER_PX);
    let as_screen = use_memo(move || open() && !wide());
    use_sheet_focus(as_screen.into(), SCREEN_ENTRY);
    let save = rsx! {
        DockAction {
            label: "Save",
            icon: CHECK,
            disabled: unchanged,
            class: SHEET_ENTER_OR_CLOSE.to_owned(),
            onclick: move |()| on_save.call(()),
        }
    };
    if !wide() {
        let shown = open();
        return rsx! {
            div {
                class: if shown { "screen swipe-select-screen web-picker-screen show" } else { "screen swipe-select-screen web-picker-screen" },
                style: layer.map(|z| format!("--overlay-z: {z};")),
                components::GridCells { still: true }
                if shown {
                    PageHeader {
                        title,
                        leading: rsx! {
                            BackArrow { class: "page-header-support", onclick: move |()| on_back.call(()) }
                        },
                        on_hint: move |()| hint_open.set(true),
                    }
                    div { class: "screen-content content-enter",
                        PickerBody { search_id, {children} }
                    }
                    ActionDock { {save} }
                }
            }
        };
    }
    rsx! {
        DrawerSheet {
            open,
            layer,
            title,
            on_close: move |()| on_back.call(()),
            on_hint: move |()| hint_open.set(true),
            class: "web-picker-sheet",
            every_width: true,
            dock: save,
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
        div { class: "web-picker-body", {children} }
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

/// A command-zone slot's card, input text and search, as a picker opened on
/// them.
#[derive(Debug, Clone, PartialEq, Default)]
struct SlotSnapshot {
    card: Option<Card>,
    text: String,
    query: String,
}

impl SlotSnapshot {
    fn take(slot: SlotState) -> Self {
        Self {
            card: slot.card.peek().clone(),
            text: slot.text.peek().clone(),
            query: slot.query.peek().clone(),
        }
    }

    fn restore(self, slot: SlotState) {
        let (mut card, mut text, mut query) = (slot.card, slot.text, slot.query);
        card.set(self.card);
        text.set(self.text);
        query.set(self.query);
    }
}

/// What the format picker opened on: the format, the land target a pick
/// moves and the command zone its cascade clears (the commander, and with it
/// the partner and the background), so Discard puts the form back as it was.
#[derive(Debug, Clone, PartialEq, Default)]
struct FormatSnapshot {
    format: Option<Format>,
    land_target: Option<i32>,
    commander: SlotSnapshot,
    partner: SlotSnapshot,
    background: SlotSnapshot,
    spell: SlotSnapshot,
}

impl FormatSnapshot {
    fn take(form: &DeckForm) -> Self {
        Self {
            format: *form.format.peek(),
            land_target: *form.land_target.peek(),
            commander: SlotSnapshot::take(form.commander),
            partner: SlotSnapshot::take(form.partner),
            background: SlotSnapshot::take(form.background),
            spell: SlotSnapshot::take(form.signature_spell),
        }
    }

    fn restore(self, form: &DeckForm) {
        let (mut format, mut land_target) = (form.format, form.land_target);
        format.set(self.format);
        land_target.set(self.land_target);
        self.commander.restore(form.commander);
        self.partner.restore(form.partner);
        self.background.restore(form.background);
        self.spell.restore(form.signature_spell);
    }
}

/// What the deck tag picker opened on: the deck tags, and the oracle tags
/// and seed a kept pick reseeds, so Discard puts all three back.
#[derive(Debug, Clone, PartialEq, Default)]
struct TagSnapshot {
    tags: Vec<String>,
    oracle_tags: Vec<String>,
    applied_seed: Vec<String>,
}

impl TagSnapshot {
    fn take(form: &DeckForm) -> Self {
        Self {
            tags: form.tags.peek().clone(),
            oracle_tags: form.oracle_tags.peek().clone(),
            applied_seed: form.applied_seed.peek().clone(),
        }
    }

    fn restore(self, form: &DeckForm) {
        let (mut tags, mut oracle_tags, mut applied_seed) =
            (form.tags, form.oracle_tags, form.applied_seed);
        tags.set(self.tags);
        oracle_tags.set(self.oracle_tags);
        applied_seed.set(self.applied_seed);
    }
}

/// The format picker: single select, with the format's details in the bar.
/// Picking runs the form's cascade at once; tapping the picked format clears
/// it.
#[component]
pub fn FormatPicker(form: DeckForm) -> Element {
    let mut open = form.format_open;
    let mut snapshot = use_signal(FormatSnapshot::default);
    let query = use_picker_search(open.into());
    let mut focused: Signal<Option<Format>> = use_signal(|| None);
    let hint_open = use_signal(|| false);
    let mut discard_open = use_signal(|| false);
    use_effect(move || {
        if open() {
            focused.set(None);
            snapshot.set(FormatSnapshot::take(&form));
        }
    });
    let revert = use_callback(move |(): ()| {
        snapshot.peek().clone().restore(&form);
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
    let layer = use_overlay_back_action(listening.into(), back);
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
            layer: layer(),
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
                        Chip {
                            key: "{format.display_name()}",
                            selected: (form.format)() == Some(format),
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
    let mut snapshot: Signal<TagSnapshot> = use_signal(TagSnapshot::default);
    let query = use_picker_search(open.into());
    let mut focused: Signal<Option<DeckTagView>> = use_signal(|| None);
    let hint_open = use_signal(|| false);
    let mut discard_open = use_signal(|| false);
    use_effect(move || {
        if open() {
            focused.set(None);
            snapshot.set(TagSnapshot::take(&form));
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
        snapshot.peek().clone().restore(&form);
        open.set(false);
    });
    let back = use_callback(move |(): ()| {
        if same_set(&tags.peek(), &snapshot.peek().tags) {
            revert.call(());
        } else {
            discard_open.set(true);
        }
    });
    // Off the stack while it asks, so Keep puts it back on top.
    let listening = use_memo(move || open() && !discard_open());
    let layer = use_overlay_back_action(listening.into(), back);
    let unchanged = same_set(&tags(), &snapshot.read().tags);
    let typed = query().to_lowercase();
    let results: Vec<DeckTagView> = catalog()
        .into_iter()
        .filter(|t| typed.is_empty() || t.display_name.to_lowercase().contains(&typed))
        .collect();
    let count = tags.read().len();
    rsx! {
        PickerSheet {
            open,
            layer: layer(),
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
        Chip {
            selected: picked,
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
                        format!("Up to {MAX_DECK_TAGS} deck tags"),
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
    let query = use_picker_search(open.into());
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
    let layer = use_overlay_back_action(listening.into(), back);
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
                format!("Up to {MAX_DECK_ORACLE_TAGS} Oracle tags"),
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
            layer: layer(),
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
        Chip {
            selected: picked,
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
                        format!("Up to {MAX_DECK_ORACLE_TAGS} Oracle tags"),
                        ToastOptions::default().duration(TOAST_QUICK),
                    );
                }
            },
            "{slug}"
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use uuid::Uuid;

    use super::*;
    use crate::inbound::screens::deck::form::use_deck_form;

    thread_local! {
        static FORM: RefCell<Option<DeckForm>> = const { RefCell::new(None) };
    }

    #[component]
    fn Host() -> Element {
        let form = use_deck_form();
        FORM.with(|f| *f.borrow_mut() = Some(form));
        rsx! {}
    }

    /// A real card renamed to `name`, with a fresh printing id.
    fn card(name: &str) -> Card {
        let mut card: Card = serde_json::from_str(include_str!(
            "../../../../../../../zwipe-ui/src/test_support/card.json"
        ))
        .unwrap();
        let id = Uuid::new_v4();
        card.card_profile.scryfall_data_id = id;
        card.scryfall_data.id = id;
        card.scryfall_data.name = name.to_owned();
        card
    }

    fn fill(slot: SlotState, name: &str) {
        let (mut card_signal, mut text) = (slot.card, slot.text);
        card_signal.set(Some(card(name)));
        text.set(name.to_owned());
    }

    /// A form as Edit holds a Commander deck led by a partner pair with a
    /// Background, and its tags.
    fn edit_form(dom: &VirtualDom) -> DeckForm {
        let form = FORM.with(|f| *f.borrow()).unwrap();
        dom.in_runtime(|| {
            let (mut format, mut tags, mut oracle_tags, mut seed) =
                (form.format, form.tags, form.oracle_tags, form.applied_seed);
            format.set(Some(Format::Commander));
            fill(form.commander, "Tymna the Weaver");
            fill(form.partner, "Thrasios, Triton Hero");
            fill(form.background, "Raised by Giants");
            tags.set(vec!["ramp".to_owned()]);
            oracle_tags.set(Vec::new());
            seed.set(vec!["ramp".to_owned()]);
        });
        form
    }

    /// An open picker as a narrow window draws it (off the browser no width matches).
    #[component]
    fn NarrowPicker() -> Element {
        rsx! {
            PickerSheet {
                open: use_signal(|| true),
                title: "Format",
                search_id: "format-search",
                hint_open: use_signal(|| false),
                unchanged: true,
                on_back: |()| {},
                on_save: |()| {},
                layer: Some(205),
                div { class: "picker-probe" }
            }
        }
    }

    #[test]
    fn a_narrow_picker_is_a_screen_with_a_back_arrow_not_a_sliding_sheet() {
        let mut dom = VirtualDom::new(NarrowPicker);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(
            html.contains("class=\"screen swipe-select-screen web-picker-screen show\""),
            "the picker shows as a screen: {html}"
        );
        assert!(html.contains("--overlay-z: 205;"), "at its layer: {html}");
        assert!(
            html.contains("aria-label=\"Back\""),
            "a back arrow leads: {html}"
        );
        assert!(html.contains("picker-probe"), "its body shows: {html}");
        assert!(
            !html.contains("bottom-sheet") && !html.contains("sheet-close"),
            "no sheet and no x: {html}"
        );
    }

    #[test]
    fn a_narrow_picker_covers_the_form_while_open() {
        assert!(
            !covers_form(false, [true, false, false]),
            "a wide window keeps the form beside it"
        );
        assert!(!covers_form(true, [false, false, false]), "nothing open");
        for open in [
            [true, false, false],
            [false, true, false],
            [false, false, true],
        ] {
            assert!(covers_form(true, open), "{open:?}");
        }
    }

    #[test]
    fn discarding_a_format_pick_puts_the_whole_command_zone_back() {
        let mut dom = VirtualDom::new(Host);
        dom.rebuild_in_place();
        let form = edit_form(&dom);
        dom.in_runtime(|| {
            let before = form.values();
            let snapshot = FormatSnapshot::take(&form);
            form.select_format(Format::Pauper);
            // The form's commander effect clears the mates on the change.
            form.partner.clear_all();
            form.background.clear_all();
            snapshot.restore(&form);
            assert_eq!(form.values(), before);
        });
    }

    #[test]
    fn discarding_a_deck_tag_change_puts_the_oracle_tags_and_seed_back() {
        let mut dom = VirtualDom::new(Host);
        dom.rebuild_in_place();
        let form = edit_form(&dom);
        dom.in_runtime(|| {
            let before = form.values();
            let seed_before = form.applied_seed.peek().clone();
            let snapshot = TagSnapshot::take(&form);
            let (mut tags, mut oracle_tags, mut seed) =
                (form.tags, form.oracle_tags, form.applied_seed);
            tags.set(Vec::new());
            oracle_tags.set(vec!["mana-rock".to_owned()]);
            seed.set(vec!["mana-rock".to_owned()]);
            snapshot.restore(&form);
            assert_eq!(form.values(), before);
            assert_eq!(*form.applied_seed.peek(), seed_before);
        });
    }
}

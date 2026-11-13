//! The card filter: the app's filter sheet, the kit's drawer on a wide
//! window and the bottom sheet below the breakpoint, opened from the host's
//! funnel.
//!
//! The panel edits a draft over the host's applied filter (zwipe-ui's
//! [`FilterDraft`]) and hands it back from the checkmark in its dock, where
//! the undo icon stages the default. The sheet closes from its title bar's
//! ×, a backdrop tap, Esc or a drag down, dropping the edits with `Filter
//! unchanged`; leaving the screen with edits drops them the same way, since
//! the host's filter never saw them.
//!
//! The host hands in what it owns: the catalog lists and a way to fetch the
//! ones a source needs, a way to raise toasts, and the optional chrome (the
//! title bar's `?`, the Oracle tags info button, a hint on the first section
//! opened, a hint dialog and the oracle tag dictionary). No fetch happens
//! here.
//!
//! ```ignore
//! CardFilterPanel {
//!     open: filter_open,
//!     filter,                        // the host's Signal<CardQueryBuilder>
//!     config: SheetConfig { show_format: true, ..SheetConfig::default() },
//!     on_apply: move |applied| { filter.set(applied); bump(); },
//!     catalog,                       // the host's Signal<Catalog>
//!     on_need: move |kinds| fetch(kinds),
//! }
//! ```

mod rows;

use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_ui::filter::draft::{ApplyOutcome, FilterDraft, SheetConfig};
use zwipe_ui::filter::picker::PickerField;
use zwipe_ui::filter::sections::Section;
use zwipe_ui::screens::card_filter;
use zwipe_ui::stores::catalog::{Catalog, Kind};
use zwipe_ui::toast::{Toast, ToastKind};

use self::rows::{SectionRows, bump};
use crate::icons::{CHECK, ERASER};
use crate::keys::SHEET_ENTER_OR_CLOSE;
use crate::{DockAction, DrawerSheet, SheetDim, use_overlay_back};

/// What the panel hands the host's dictionary: whether it is open, and
/// where a tag the user picks in it goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DictionarySlot {
    /// Open while the dictionary shows; the dictionary closes by setting it
    /// false.
    pub open: Signal<bool>,
    /// Takes the slug of the tag the user picks.
    pub on_use: Callback<String>,
}

/// The filter panel. `filter` is the host's applied filter; `on_apply` takes
/// the committed draft. `catalog` holds the lists the pickers search, and
/// `on_need` fetches the kinds the source needs each time the panel opens.
/// `quiet_reset` stages the blank filter on `Reset` without a toast, as the
/// commander maybeboard and Swipe select do.
///
/// The rest is optional chrome: `on_toast` raises the panel's toasts (none
/// show without it), `on_hint` puts a `?` in the title bar,
/// `on_first_open` runs the first time a section opens, `oracle_tags_info`
/// sits in the Oracle tags trigger, `hint` is drawn after the sheet, and
/// `dictionary` draws the oracle tag dictionary the `Dictionary` chips open
/// (no chip shows without it).
#[component]
pub fn CardFilterPanel(
    open: Signal<bool>,
    filter: ReadSignal<CardQueryBuilder>,
    config: SheetConfig,
    on_apply: EventHandler<CardQueryBuilder>,
    catalog: ReadSignal<Catalog>,
    on_need: EventHandler<&'static [Kind]>,
    #[props(default)] quiet_reset: bool,
    #[props(default)] on_toast: Option<EventHandler<Toast>>,
    #[props(default)] on_hint: Option<EventHandler<()>>,
    #[props(default)] on_first_open: Option<EventHandler<()>>,
    #[props(default)] oracle_tags_info: Option<Element>,
    #[props(default)] hint: Option<Element>,
    #[props(default)] dictionary: Option<Callback<DictionarySlot, Element>>,
) -> Element {
    let layer = use_overlay_back(open);

    let config_for_draft = config.clone();
    let mut draft = use_signal(|| FilterDraft::new(&filter.peek(), config_for_draft));
    // Whether `draft` belongs to an opening.
    let mut armed = use_signal(|| false);
    let epoch = use_signal(|| 0u32);
    let mut open_section: Signal<Option<Section>> = use_signal(|| None);
    // The section whose rows stay mounted while it eases shut.
    let closing: Signal<Option<Section>> = use_signal(|| None);
    // Opens `next` (or none), keeping the section it replaces drawn while it
    // closes, so the collapse eases its rows away instead of dropping them.
    let mut set_open = move |next: Option<Section>| {
        let before = *open_section.peek();
        if let Some(prev) = before.filter(|prev| Some(*prev) != next) {
            keep_while_closing(closing, prev);
        }
        open_section.set(next);
    };
    let mut first_fired = use_signal(|| false);
    let mut dict_open = use_signal(|| false);
    let mut dict_field: Signal<Option<PickerField>> = use_signal(|| None);

    let config_for_open = config.clone();
    let mut reseed_for_open = move |applied: &CardQueryBuilder| {
        draft.set(FilterDraft::new(applied, config_for_open.clone()));
        bump(epoch);
    };
    let needs = config.sources.needs();

    // Opening and closing. A close that was not Apply drops the edits with
    // `Filter unchanged`.
    use_effect(move || {
        let live = open();
        if live && !*armed.peek() {
            armed.set(true);
            reseed_for_open(&filter.peek());
            if catalog.peek().any_public_failed() {
                raise(
                    on_toast,
                    Toast::normal(ToastKind::Warning, "Filter lists didn't load, trying again"),
                );
            }
            on_need.call(needs);
        } else if !live && *armed.peek() {
            armed.set(false);
            let dropped = draft.peek().back();
            if let Some(t) = dropped {
                raise(on_toast, t);
            }
            set_open(None);
        }
    });

    // The host changed its filter while the sheet is open (Synergy, a
    // source switch): an untouched draft follows it; an edited one keeps
    // its edits over the new filter, with the host's synergy.
    let config_for_follow = config.clone();
    use_effect(move || {
        let applied = filter();
        if !*armed.peek() {
            return;
        }
        let old = draft.peek().clone();
        if old.unchanged() {
            draft.set(FilterDraft::new(&applied, config_for_follow.clone()));
            bump(epoch);
            return;
        }
        let mut next = FilterDraft::new(&applied, config_for_follow.clone());
        next.draft = old.draft;
        next.draft.set_synergy(applied.synergy());
        next.modes = old.modes;
        draft.set(next);
    });

    let mut apply_outcome = move |outcome: ApplyOutcome| match outcome {
        ApplyOutcome::Disabled => {}
        ApplyOutcome::Refused(t) => raise(on_toast, t),
        ApplyOutcome::Applied {
            filter: applied,
            toast: t,
        } => {
            let applied = *applied;
            let config = draft.peek().config.clone();
            draft.set(FilterDraft::new(&applied, config));
            bump(epoch);
            on_apply.call(applied);
            raise(on_toast, t);
            armed.set(false);
            set_open(None);
            open.set(false);
        }
    };

    let mut apply = move || {
        let outcome = draft.peek().apply();
        apply_outcome(outcome);
    };

    let mut reset = move || {
        if quiet_reset {
            draft.write().draft.clear();
        } else {
            let t = draft.write().reset();
            raise(on_toast, t);
        }
        bump(epoch);
    };

    let mut toggle_section = move |section: Section| {
        if open_section.peek().as_ref() == Some(&section) {
            set_open(None);
            return;
        }
        set_open(Some(section));
        if !*first_fired.peek() {
            first_fired.set(true);
            if let Some(handler) = on_first_open {
                handler.call(());
            }
        }
        document::eval(OPEN_SECTION_SCROLL_JS);
    };

    let on_use = use_callback(move |slug: String| {
        let Some(field) = *dict_field.peek() else {
            return;
        };
        let t = draft.write().adopt_oracle_tag(field, &slug);
        bump(epoch);
        raise(on_toast, t);
    });
    let on_dictionary = dictionary.map(|_| {
        EventHandler::new(move |field: PickerField| {
            dict_field.set(Some(field));
            dict_open.set(true);
        })
    });

    let unchanged = draft.read().unchanged();
    let shown_sections = Section::shown(config.show_format);
    let sources = config.sources.clone();
    let body = rsx! {
        div { class: "accordion card-filter-accordion", id: "filter-accordion",
            for section in shown_sections {
                {
                    let is_open = open_section() == Some(section);
                    let active = section.is_active(&draft.read().draft);
                    let info = if section == Section::OracleTags { oracle_tags_info.clone() } else { None };
                    rsx! {
                        div {
                            key: "{section:?}",
                            class: "accordion-item",
                            id: section_id(section),
                            "data-open": "{is_open}",
                            div {
                                class: "accordion-trigger",
                                role: "button",
                                tabindex: "0",
                                "aria-expanded": "{is_open}",
                                onclick: move |_| toggle_section(section),
                                onkeydown: move |e: Event<KeyboardData>| {
                                    if e.key() == Key::Enter || e.key() == Key::Character(" ".into()) {
                                        e.prevent_default();
                                        toggle_section(section);
                                    }
                                },
                                "{section.title()}"
                                if let Some(info) = info {
                                    {info}
                                }
                                svg {
                                    class: "accordion-expand-icon",
                                    view_box: "0 0 24 24",
                                    xmlns: "http://www.w3.org/2000/svg",
                                    polyline { points: "6 9 12 15 18 9" }
                                }
                            }
                            // Beside the trigger, not in it, so a tap on it never
                            // toggles the section.
                            if active {
                                button {
                                    class: "clear-btn section-clear",
                                    "aria-label": "Clear",
                                    onclick: move |_| {
                                        section.clear(&mut draft.write().draft);
                                        bump(epoch);
                                    },
                                    "\u{00d7}"
                                }
                            }
                            div { class: "accordion-content", "data-open": "{is_open}",
                                div {
                                    if is_open || closing() == Some(section) {
                                        SectionRows {
                                            section,
                                            draft,
                                            epoch,
                                            sources: sources.clone(),
                                            catalog,
                                            on_toast,
                                            on_dictionary,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    // Reset and Apply float at the panel's bottom right, as in the app.
    let dock = rsx! {
        DockAction { label: card_filter::RESET, icon: ERASER, onclick: move |()| reset() }
        DockAction {
            label: card_filter::APPLY,
            icon: CHECK,
            disabled: unchanged,
            class: SHEET_ENTER_OR_CLOSE.to_owned(),
            onclick: move |()| apply(),
        }
    };

    rsx! {
        DrawerSheet {
            layer: layer(),
            dim: SheetDim::Phone,
            open,
            title: card_filter::TITLE,
            on_close: move |()| open.set(false),
            on_hint,
            class: "card-filter-sheet",
            dock,
            {body}
        }
        if let Some(hint) = hint {
            {hint}
        }
        if dict_open() {
            if let Some(dictionary) = dictionary {
                {
                    dictionary.call(DictionarySlot { open: dict_open, on_use })
                }
            }
        }
    }
}

/// Hands `t` to the host's toasts, if it has any.
fn raise(on_toast: Option<EventHandler<Toast>>, t: Toast) {
    if let Some(handler) = on_toast {
        handler.call(t);
    }
}

/// The element id of a section's accordion item.
fn section_id(section: Section) -> String {
    format!("filter-section-{section:?}")
}

/// Scrolls the filter's open section to the top of its sheet, as the app does: once, instantly,
/// at 300 ms, as the height ease ends and the section's rows start to fade in (`filter.css`,
/// `accordion-content-in`), so the sheet moves once, before the rows show, and never under the
/// pointer. By then the section has its full height, so even one near the sheet's end reaches the
/// top. One section opens at a time, so the open one is the one that asked.
pub const OPEN_SECTION_SCROLL_JS: &str = "(() => { const go = () => { const el = document.querySelector('#filter-accordion .accordion-item[data-open=\"true\"]'); if (el) el.scrollIntoView({ behavior: 'instant', block: 'start' }); }; setTimeout(go, 300); })()";

/// How long a section takes to ease shut, in milliseconds: the accordion's
/// 300ms collapse.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(dead_code, reason = "only the browser times the collapse")
)]
const SECTION_CLOSE_MS: u32 = 300;

/// Keeps `section`'s rows drawn for its collapse, unless another section
/// starts closing meanwhile.
#[cfg(target_arch = "wasm32")]
fn keep_while_closing(mut closing: Signal<Option<Section>>, section: Section) {
    closing.set(Some(section));
    spawn(async move {
        gloo_timers::future::TimeoutFuture::new(SECTION_CLOSE_MS).await;
        if *closing.peek() == Some(section) {
            closing.set(None);
        }
    });
}

/// Off the browser nothing is timed: the rows drop at once.
#[cfg(not(target_arch = "wasm32"))]
fn keep_while_closing(_closing: Signal<Option<Section>>, _section: Section) {}

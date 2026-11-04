//! The card filter: the app's filter sheet, as a right side panel on a wide
//! window, the app's bottom sheet below the breakpoint, and on Add and
//! Remove at three columns a panel docked open beside the card.
//!
//! The panel edits a draft over the host's applied filter (zwipe-ui's
//! [`FilterDraft`]) and hands it back from the checkmark in its dock, where
//! the undo icon stages the default. The sheet closes from its title bar's
//! ×, a backdrop tap, Esc or a drag down, dropping the edits with `Filter
//! unchanged`; leaving the screen with edits drops them the same way, since
//! the host's filter never saw them. A host mounts one beside its frame:
//!
//! ```ignore
//! CardFilterPanel {
//!     open: filter_open,
//!     filter,                        // the host's Signal<CardQueryBuilder>
//!     config: SheetConfig { show_format: true, ..SheetConfig::default() },
//!     on_apply: move |applied| { filter.set(applied); bump(); },
//! }
//! ```
//!
//! The one-time `filter` hint opens the first time the user opens a
//! section, a kept deviation: a docked panel is open from the start.

mod rows;

use components::icons::{CHECK, UNDO};
use components::{Button, ButtonVariant, DockIcon, TOAST_NORMAL, use_overlay_back};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_ui::filter::draft::{ApplyOutcome, FilterDraft, SheetConfig};
use zwipe_ui::filter::picker::PickerField;
use zwipe_ui::filter::sections::Section;

use self::rows::{SectionRows, bump};
use crate::inbound::catalog::CatalogCache;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::media::{THREE_COLUMNS_PX, use_min_width};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, open_and_record_hint};
use crate::inbound::chrome::sheet::{SheetForm, SheetFrame};
use crate::inbound::chrome::toast::raise;
use crate::inbound::screens::oracle_tag_dictionary::OracleTagDictionary;

/// Whether the panel is docked open: a host that docks it, on a window at
/// least three columns wide. The host reads it to hide its `Filter` button.
pub fn use_filter_docked(dockable: bool) -> Signal<bool> {
    let wide = use_min_width(THREE_COLUMNS_PX);
    let mut docked = use_signal(|| false);
    use_effect(move || docked.set(dockable && wide()));
    docked
}

/// The filter panel. `filter` is the host's applied filter; `on_apply` takes
/// the committed draft. `docked` (from [`use_filter_docked`]) draws it as an
/// open column instead of a sheet. `quiet_reset` stages the blank filter on
/// `Reset` without a toast, as the commander maybeboard and Swipe select do.
#[component]
pub fn CardFilterPanel(
    open: Signal<bool>,
    filter: ReadSignal<CardQueryBuilder>,
    config: SheetConfig,
    on_apply: EventHandler<CardQueryBuilder>,
    #[props(default)] docked: Option<Signal<bool>>,
    #[props(default)] quiet_reset: bool,
) -> Element {
    use_overlay_back(open);
    let toast = use_toast();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let cache: CatalogCache = use_context();
    let is_docked = move || docked.is_some_and(|d| d());

    let config_for_draft = config.clone();
    let mut draft = use_signal(|| FilterDraft::new(&filter.peek(), config_for_draft));
    // Whether `draft` belongs to an opening: the sheet is open, or docked.
    let mut armed = use_signal(|| false);
    let epoch = use_signal(|| 0u32);
    let mut open_section: Signal<Option<Section>> = use_signal(|| None);
    let hint_open = use_signal(|| false);
    let mut hint_fired = use_signal(|| false);
    let mut dict_open = use_signal(|| false);
    let mut dict_field: Signal<Option<PickerField>> = use_signal(|| None);

    let config_for_open = config.clone();
    let mut reseed_for_open = move |applied: &CardQueryBuilder| {
        draft.set(FilterDraft::new(applied, config_for_open.clone()));
        bump(epoch);
    };
    let needs = config.sources.needs();

    // Opening, docking and closing. A close that was not Apply drops the
    // edits with `Filter unchanged`.
    use_effect(move || {
        let live = open() || is_docked();
        if live && !*armed.peek() {
            armed.set(true);
            reseed_for_open(&filter.peek());
            if cache.0.peek().any_public_failed() {
                toast.warning(
                    "Filter lists didn't load, trying again".to_string(),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
            }
            cache.ensure(needs, client);
        } else if !live && *armed.peek() {
            armed.set(false);
            let dropped = draft.peek().back();
            if let Some(t) = dropped {
                raise(toast, &t);
            }
            open_section.set(None);
        }
    });

    // The host changed its filter while docked (Synergy, a source switch):
    // an untouched draft follows it; an edited one keeps its edits over the
    // new filter, with the host's synergy.
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
        ApplyOutcome::Refused(t) => raise(toast, &t),
        ApplyOutcome::Applied {
            filter: applied,
            toast: t,
        } => {
            let applied = *applied;
            let config = draft.peek().config.clone();
            draft.set(FilterDraft::new(&applied, config));
            bump(epoch);
            on_apply.call(applied);
            raise(toast, &t);
            if !is_docked() {
                armed.set(false);
                open_section.set(None);
                open.set(false);
            }
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
            raise(toast, &t);
        }
        bump(epoch);
    };

    let mut toggle_section = move |section: Section| {
        if open_section.peek().as_ref() == Some(&section) {
            open_section.set(None);
            return;
        }
        open_section.set(Some(section));
        if !*hint_fired.peek() {
            hint_fired.set(true);
            open_and_record_hint(OneTimeHint::Filter, session, client, hint_open);
        }
        scroll_section_into_view(section);
    };

    let unchanged = draft.read().unchanged();
    let shown_sections = Section::shown(config.show_format);
    let sources = config.sources.clone();
    let body = rsx! {
        div { class: "accordion web-filter-accordion", id: "filter-accordion",
            for section in shown_sections {
                {
                    let is_open = open_section() == Some(section);
                    let active = section.is_active(&draft.read().draft);
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
                                if section == Section::OracleTags {
                                    InfoButton { topic: HintTopic::OracleTags }
                                }
                                if active {
                                    button {
                                        class: "clear-btn",
                                        onclick: move |e| {
                                            e.stop_propagation();
                                            section.clear(&mut draft.write().draft);
                                            bump(epoch);
                                        },
                                        "\u{00d7}"
                                    }
                                }
                                svg {
                                    class: "accordion-expand-icon",
                                    view_box: "0 0 24 24",
                                    xmlns: "http://www.w3.org/2000/svg",
                                    polyline { points: "6 9 12 15 18 9" }
                                }
                            }
                            div { class: "accordion-content", "data-open": "{is_open}",
                                div {
                                    if is_open {
                                        SectionRows {
                                            section,
                                            draft,
                                            epoch,
                                            sources: sources.clone(),
                                            on_dictionary: move |field: PickerField| {
                                                dict_field.set(Some(field));
                                                dict_open.set(true);
                                            },
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
    let header = rsx! {
        div { class: "sheet-header", style: "position: relative;",
            if !is_docked() {
                button {
                    class: "sheet-close",
                    "aria-label": "Close",
                    onclick: move |_| open.set(false),
                    "\u{00d7}"
                }
            }
            span { style: "font-size: 1rem; color: var(--accent-primary);", "Filter" }
            Button {
                variant: ButtonVariant::Action,
                style: "position: absolute; right: 1rem; top: 50%; transform: translateY(-50%); opacity: 0.55; padding: 0.2rem 0.6rem;",
                onclick: move |_| {
                    let mut hint_open = hint_open;
                    hint_open.set(true);
                },
                "?"
            }
        }
    };
    // Reset and Apply float at the panel's bottom right, as in the app.
    let dock = rsx! {
        div { class: "action-dock action-dock-right sheet-dock",
            button {
                class: "action-dock-btn",
                "aria-label": "Reset",
                onclick: move |_| reset(),
                DockIcon { path: UNDO }
            }
            button {
                class: "action-dock-btn",
                "aria-label": "Apply",
                disabled: unchanged,
                onclick: move |_| apply(),
                DockIcon { path: CHECK }
            }
        }
    };

    rsx! {
        if is_docked() {
            div { class: "web-filter-dock",
                components::GridCells { still: true }
                {header}
                div { class: "sheet-content web-filter-dock-content", {body} }
                {dock}
            }
        } else {
            SheetFrame { form: SheetForm::SidePanel,
                div {
                    class: if open() { "sheet-backdrop show" } else { "sheet-backdrop" },
                    onclick: move |_| open.set(false),
                }
                div { class: if open() { "bottom-sheet show web-filter-sheet" } else { "bottom-sheet web-filter-sheet" },
                    components::GridCells { still: true }
                    {header}
                    div { class: "sheet-content",
                        if open() {
                            {body}
                        }
                    }
                    {dock}
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::Filter, open: hint_open }
        if dict_open() {
            OracleTagDictionary {
                open: dict_open,
                on_use: move |slug: String| {
                    let Some(field) = *dict_field.peek() else {
                        return;
                    };
                    let t = draft.write().adopt_oracle_tag(field, &slug);
                    bump(epoch);
                    raise(toast, &t);
                },
            }
        }
    }
}

/// The element id of a section's accordion item.
fn section_id(section: Section) -> String {
    format!("filter-section-{section:?}")
}

/// Scrolls an opened section to the top of its panel, as the app does.
fn scroll_section_into_view(section: Section) {
    let id = section_id(section);
    document::eval(&format!(
        "setTimeout(() => {{ const el = document.getElementById('{id}'); if (el) el.scrollIntoView({{ behavior: 'smooth', block: 'start' }}); }}, 50)"
    ));
}

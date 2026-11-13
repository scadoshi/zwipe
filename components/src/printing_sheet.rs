//! The printing sheet: every printing of a card, one at a time, to pick one or only to look.
//!
//! The printings are the cover flow: the selected one large in the middle and up to ten on each
//! side stacked behind it, moved by the arrows, a sideways scroll, a drag or swipe, or a tap on a
//! side card. From the kit's modal width it fills a large modal, with Enter, a click on the middle
//! card or `Select` picking it. Below that width it sits in a centered modal down to the
//! breakpoint and a bottom sheet under it, with the prices at its bottom and Save at their right,
//! Enter saving. Either form takes focus as it opens, so the arrow keys, `h`, `l` and Enter work
//! at once, and hands focus back as it closes. It closes from its title bar's ×, a backdrop tap, a drag down or Esc.
//!
//! Components never call the API. The host provides a [`PrintingsSource`]: the printings it has
//! fetched, by the card's Oracle id, and a callback that fetches them. Without a source the sheet
//! shows nothing.

use std::collections::HashMap;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_ui::card::info;
use zwipe_ui::screens::printing_sheet as model;
use zwipe_ui::shortcuts::{self, FlowKey};

use crate::icons::{CHECK, DockIcon};
use crate::keys::use_sheet_focus;
use crate::media::{KIT_MODAL_PX, use_min_width};
use crate::{BottomSheet, CoverFlow, LoadFailed, SheetFrame, SheetWide, use_overlay_back_action};

/// How far a card's printings have come.
#[derive(Debug, Clone, PartialEq)]
pub enum PrintingsFetch {
    /// On their way.
    Loading,
    /// Fetched, in the order the sheet shows them.
    Ready(Vec<Card>),
    /// The fetch failed; the sheet offers Retry.
    Failed,
}

/// The host's printings: each card's fetch by its Oracle id, and the callback that starts one,
/// which writes [`PrintingsFetch::Loading`] and then its outcome into `results`. The sheet asks
/// each time it opens and on Retry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrintingsSource {
    /// Each card's printings asked for, by the card's Oracle id.
    pub results: Signal<HashMap<Uuid, PrintingsFetch>>,
    /// Fetches the printings of the card with this Oracle id.
    pub request: Callback<Uuid>,
}

/// The open bottom sheet's body, which takes the keys below the kit's modal width.
const NARROW_BODY: &str = ".web-printing-sheet .bottom-sheet.show .web-printing-body";

/// The open modal's body, which takes the keys from the kit's modal width on.
const WIDE_BODY: &str = ".web-flow-sheet.show .web-flow-body";

/// Which way a step through the printings goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// To the printing before.
    Back,
    /// To the printing after.
    Forward,
}

/// The printings of `card` from the [`PrintingsSource`] in context, opening on the one shown. With
/// an `on_save`, the sheet picks: Select or Enter hands it the showing printing and closes, and
/// leaving with a different printing showing calls `on_unchanged`. Without one it only browses: no
/// Select or Save, and Enter does nothing.
#[component]
pub fn PrintingSheet(
    card: Card,
    mut open: Signal<bool>,
    #[props(default)] on_save: Option<EventHandler<Card>>,
    #[props(default)] on_unchanged: Option<EventHandler<()>>,
) -> Element {
    let read_only = on_save.is_none();
    let source = try_use_context::<PrintingsSource>();
    // From the kit's modal width on, the picker is the cover flow.
    let flow = use_min_width(KIT_MODAL_PX);
    // Retry bumps `attempt`, which asks again.
    let mut attempt = use_signal(|| 0u32);
    let mut index = use_signal(|| 0usize);
    let mut initial = use_signal(|| 0usize);
    // Whether `index` and `initial` sit on the shown printing of the latest fetch.
    let mut seeded = use_signal(|| false);
    let shown_id = card.scryfall_data.id;
    let oracle_id = card.scryfall_data.oracle_id;

    use_effect(move || {
        let _ = attempt();
        if !open() {
            return;
        }
        let (Some(source), Some(oracle_id)) = (source, oracle_id) else {
            return;
        };
        seeded.set(false);
        source.request.call(oracle_id);
    });

    // The fetched printings open on the one shown.
    use_effect(move || {
        let (Some(source), Some(oracle_id)) = (source, oracle_id) else {
            return;
        };
        let results = source.results.read();
        let Some(PrintingsFetch::Ready(cards)) = results.get(&oracle_id) else {
            return;
        };
        if *seeded.peek() {
            return;
        }
        let at = cards
            .iter()
            .position(|p| p.scryfall_data.id == shown_id)
            .unwrap_or(0);
        index.set(at);
        initial.set(at);
        seeded.set(true);
    });

    let fetch = source
        .zip(oracle_id)
        .and_then(|(source, oracle_id)| source.results.read().get(&oracle_id).cloned());
    // Until the first fetch starts, the sheet ghosts as it does while one runs.
    let loading = source.is_some()
        && oracle_id.is_some()
        && matches!(fetch, None | Some(PrintingsFetch::Loading));
    let failed = matches!(fetch, Some(PrintingsFetch::Failed));
    let all = match fetch {
        Some(PrintingsFetch::Ready(cards)) => cards,
        _ => Vec::new(),
    };
    let count = all.len();
    let at = index();
    let changed = count > 0 && at != initial();
    let visible = all.get(at).cloned();

    let leave = use_callback(move |(): ()| {
        if !read_only
            && changed
            && let Some(on_unchanged) = on_unchanged
        {
            on_unchanged.call(());
        }
        open.set(false);
    });

    let mut step = move |way: Step| {
        if count < 2 {
            return;
        }
        let next = if way == Step::Forward {
            at.saturating_add(1).min(count.saturating_sub(1))
        } else {
            at.saturating_sub(1)
        };
        index.set(next);
    };
    let visible_for_save = visible.clone();
    let save = use_callback(move |(): ()| {
        let Some(on_save) = on_save else {
            return;
        };
        if !changed {
            return;
        }
        if let Some(picked) = visible_for_save.clone() {
            on_save.call(picked);
            open.set(false);
        }
    });

    // The bottom sheet's body takes the keys as soon as it opens; the modal form focuses its own.
    let narrow_open = use_memo(move || open() && !flow());
    use_sheet_focus(narrow_open.into(), NARROW_BODY);

    let retry = use_callback(move |()| {
        let next = attempt.peek().wrapping_add(1);
        attempt.set(next);
    });

    if flow() {
        return rsx! {
            CoverFlowSheet {
                open,
                printings: all,
                index,
                loading,
                failed,
                on_retry: retry,
                changed,
                read_only,
                on_leave: leave,
                on_select: save,
            }
        };
    }

    rsx! {
        // `sheet-over` lifts the sheet above a details sheet already open
        // under it. The body takes the arrow keys and Enter.
        div { class: "web-sheet web-sheet-modal web-printing-sheet sheet-over",
            BottomSheet {
                open,
                title: "Printings".to_string(),
                closable: true,
                on_dismiss: move |()| leave.call(()),
                div {
                    class: "web-printing-body",
                    tabindex: "-1",
                    onkeydown: move |e: Event<KeyboardData>| match flow_press(&e) {
                        Some(FlowKey::Back) => {
                            e.prevent_default();
                            step(Step::Back);
                        }
                        Some(FlowKey::Forward) => {
                            e.prevent_default();
                            step(Step::Forward);
                        }
                        Some(FlowKey::Select) => save.call(()),
                        None => {}
                    },
                    if failed {
                        LoadFailed { message: model::LOAD_FAILED, on_retry: retry }
                    } else if loading || visible.is_some() {
                        CoverFlow { printings: all, index, loading }
                        // The prices with Save at their right, at the
                        // sheet's bottom. Save is grayed until a different
                        // printing shows.
                        div { class: "web-printing-foot",
                            div { class: "web-printing-foot-row",
                                if let Some(card) = visible.filter(|_| !loading) {
                                    CardPrices { card }
                                } else {
                                    CardPricesGhost {}
                                }
                                if loading && !read_only {
                                    div { class: "action-dock web-printing-save",
                                        span { class: "skeleton-bar skeleton-chip web-printing-save-ghost" }
                                    }
                                } else if !read_only {
                                    div { class: "action-dock web-printing-save",
                                        button {
                                            class: "action-dock-btn",
                                            "aria-label": "Save",
                                            disabled: !changed,
                                            onclick: move |_| save.call(()),
                                            // Enter presses the button and stays off the body's pick.
                                            onkeydown: move |e: Event<KeyboardData>| {
                                                if e.key() == Key::Enter {
                                                    e.stop_propagation();
                                                }
                                            },
                                            DockIcon { path: CHECK }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The printings as a cover flow in the kit's modal: the one at `index`
/// large and centered, up to ten on each side behind it, every card gliding
/// to its new place as the selection moves, and an arrow button at each
/// side steps one printing. `on_select` picks the centered
/// printing; it does nothing while it is the one the sheet opened on, as
/// the `Select` button shows. `read_only` leaves out `Select`.
#[component]
fn CoverFlowSheet(
    open: Signal<bool>,
    printings: Vec<Card>,
    mut index: Signal<usize>,
    loading: bool,
    failed: bool,
    on_retry: Callback<()>,
    changed: bool,
    read_only: bool,
    on_leave: Callback<()>,
    on_select: Callback<()>,
) -> Element {
    let layer = use_overlay_back_action(open.into(), on_leave);
    let count = printings.len();
    let at = index().min(count.saturating_sub(1));
    let selected = printings.get(at).cloned();

    // The body takes the keys as soon as the modal opens.
    use_sheet_focus(open.into(), WIDE_BODY);

    // The arrow keys step as the flow's buttons do.
    let mut step = move |way: Step| {
        let current = *index.peek();
        let next = if way == Step::Forward {
            current.saturating_add(1)
        } else {
            current.saturating_sub(1)
        };
        index.set(next.min(count.saturating_sub(1)));
    };
    let prices = selected
        .as_ref()
        .map_or_default(|c| info::prices(&c.scryfall_data));

    rsx! {
        div { class: "web-flow",
            SheetFrame { wide: SheetWide::Modal,
                div {
                    class: if open() { "sheet-backdrop show" } else { "sheet-backdrop" },
                    style: layer().map(|z| format!("--overlay-z: {z};")),
                    onclick: move |_| on_leave.call(()),
                }
                div {
                    class: if open() { "bottom-sheet show web-flow-sheet" } else { "bottom-sheet web-flow-sheet" },
                    style: layer().map(|z| format!("--overlay-z: {z};")),
                    role: "dialog",
                    "aria-label": "Printings",
                    "aria-modal": "true",
                    div { class: "sheet-header", style: "position: relative;",
                        button {
                            class: "sheet-close",
                            "aria-label": "Close",
                            onclick: move |_| on_leave.call(()),
                            "\u{00d7}"
                        }
                        span { class: "sheet-title", "Printings" }
                    }
                    div {
                        class: "sheet-content web-flow-body",
                        tabindex: "-1",
                        onkeydown: move |e: Event<KeyboardData>| match flow_press(&e) {
                            Some(FlowKey::Back) => {
                                e.prevent_default();
                                step(Step::Back);
                            }
                            Some(FlowKey::Forward) => {
                                e.prevent_default();
                                step(Step::Forward);
                            }
                            Some(FlowKey::Select) if !read_only => on_select.call(()),
                            _ => {}
                        },
                        if failed {
                            LoadFailed { message: model::LOAD_FAILED, on_retry: move |()| on_retry.call(()) }
                        } else {
                            CoverFlow {
                                printings,
                                index,
                                loading,
                                on_pick: (!read_only).then(|| EventHandler::new(move |()| on_select.call(()))),
                            }
                            if loading {
                                div { class: "web-flow-info",
                                    span { class: "skeleton-bar skeleton-chip skeleton-tinted stat-chip-p3 web-flow-ghost-set" }
                                    span { class: "skeleton-bar skeleton-chip skeleton-tinted stat-chip-muted web-flow-ghost-number" }
                                    for i in 0..GHOST_PRICES {
                                        span { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-tinted stat-chip-money web-flow-ghost-price" }
                                    }
                                    span { class: "skeleton-bar web-flow-ghost-count" }
                                    if !read_only {
                                        span { class: "skeleton-bar skeleton-chip web-flow-ghost-select" }
                                    }
                                }
                            } else if let Some(card) = selected {
                                div { class: "web-flow-info",
                                    span { class: "stat-chip stat-chip-p3", "{card.scryfall_data.set_name}" }
                                    span { class: "stat-chip stat-chip-muted", "#{card.scryfall_data.collector_number}" }
                                    for price in prices {
                                        span { key: "{price}", class: "stat-chip stat-chip-money", "{price}" }
                                    }
                                    span { class: "web-flow-count", "{at.saturating_add(1)} of {count}" }
                                    if !read_only {
                                        button {
                                            r#type: "button",
                                            class: "btn-action btn-icon web-flow-select",
                                            "aria-label": "Select",
                                            title: "Select",
                                            disabled: !changed,
                                            onclick: move |_| on_select.call(()),
                                            DockIcon { path: CHECK }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The card's prices as chips, under a printing in the narrow sheet.
#[component]
fn CardPrices(card: Card) -> Element {
    let prices = info::prices(&card.scryfall_data);
    rsx! {
        div { class: "card-prices",
            for price in prices {
                span { key: "{price}", class: "card-row-price card-price-chip", "{price}" }
            }
        }
    }
}

/// How many price chips a printing's ghost shows: dollars, euros and tix.
const GHOST_PRICES: usize = 3;

/// The ghost of a printing's prices, under the cover flow while the
/// printings load: a price chip's ghost in the money tone for each price.
#[component]
pub fn CardPricesGhost() -> Element {
    rsx! {
        div { class: "card-prices",
            for i in 0..GHOST_PRICES {
                span { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-tinted stat-chip-money card-price-ghost" }
            }
        }
    }
}

/// The cover-flow action a keydown stands for ([`shortcuts::flow_key`]); a combo with Ctrl, Alt
/// or Meta is the browser's.
fn flow_press(e: &Event<KeyboardData>) -> Option<FlowKey> {
    let combo = e
        .modifiers()
        .intersects(Modifiers::CONTROL | Modifiers::ALT | Modifiers::META);
    if combo {
        return None;
    }
    shortcuts::flow_key(&e.key().to_string())
}

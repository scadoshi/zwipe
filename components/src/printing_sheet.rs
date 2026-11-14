//! The printing sheet: every printing of a card, one at a time, to pick one or only to look.
//!
//! The printings are the cover flow: the selected one large in the middle and up to ten on each
//! side stacked behind it, moved by the arrows, a sideways scroll, a drag or swipe, or a tap on a
//! side card. From the kit's modal width it fills a large modal, with Enter, a click on the middle
//! card or `Select` picking it. Below that width it sits in a centered modal down to the
//! breakpoint and a bottom sheet under it, with the printing's chips at its bottom and Save
//! under them at the right, Enter saving. Both forms show which printing it is as one row of
//! chips ([`PrintingChips`]). Either form takes focus as it opens, so the arrow keys, `h`, `l` and Enter work
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
use crate::keys::use_sheet_focus_handing_back;
use crate::media::{KIT_MODAL_PX, use_min_width};
use crate::{
    BottomSheet, CoverFlow, IMAGE_ICON, LoadFailed, SheetFrame, SheetWide, use_overlay_back_action,
};

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

/// Where focus goes after a save draws the card with Scryfall id `id` anew, in order: its
/// Printings button in its row or in a details drawer showing it, else the card itself, a List
/// row's line or a stack card, where its row is shut or it is in Stacks.
fn handback_selectors(id: Uuid) -> Vec<String> {
    let action = crate::data_action::PRINTINGS;
    vec![
        format!(
            "[data-row=\"{id}\"] [data-action=\"{action}\"], [data-card=\"{id}\"] [data-action=\"{action}\"]"
        ),
        format!(
            ".web-deck-row[data-row=\"{id}\"] .card-row-compact, .card-stack-slot[data-row=\"{id}\"] .card-stack-card"
        ),
    ]
}

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
    // Each form's body takes the keys as soon as it opens.
    let narrow_open = use_memo(move || open() && !flow());
    let narrow_focus = use_sheet_focus_handing_back(narrow_open.into(), NARROW_BODY);
    let wide_open = use_memo(move || open() && flow());
    let wide_focus = use_sheet_focus_handing_back(wide_open.into(), WIDE_BODY);
    let visible_for_save = visible.clone();
    let save = use_callback(move |(): ()| {
        let Some(on_save) = on_save else {
            return;
        };
        if !changed {
            return;
        }
        if let Some(picked) = visible_for_save.clone() {
            // The save draws the card again as the new printing, with a Printings button of its
            // own, which takes the focus.
            let reopen = handback_selectors(picked.scryfall_data.id);
            narrow_focus.hand_back_to(reopen.clone());
            wide_focus.hand_back_to(reopen);
            on_save.call(picked);
            open.set(false);
        }
    });

    let retry = use_callback(move |()| {
        let next = attempt.peek().wrapping_add(1);
        attempt.set(next);
    });

    let name = card.scryfall_data.name;
    if flow() {
        return rsx! {
            CoverFlowSheet {
                name,
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
                title: name,
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
                        // Enter's own press would land on the Printings button once focus
                        // goes back to it, opening the sheet again.
                        Some(FlowKey::Select) => {
                            e.prevent_default();
                            save.call(());
                        }
                        None => {}
                    },
                    if failed {
                        LoadFailed { message: model::LOAD_FAILED, on_retry: retry }
                    } else if loading || visible.is_some() {
                        CoverFlow { printings: all, index, loading }
                        // The printing's chips, then Save at the right, at
                        // the sheet's bottom. Save is grayed until a
                        // different printing shows.
                        div { class: "web-printing-foot",
                            if let Some(card) = visible.filter(|_| !loading) {
                                PrintingChips { card, at, count }
                            } else {
                                PrintingChipsGhost {}
                            }
                            div { class: "web-printing-foot-row",
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
    // The card's name, the sheet's title.
    name: String,
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
                    "aria-label": model::label(&name),
                    "aria-modal": "true",
                    div { class: "sheet-header", style: "position: relative;",
                        button {
                            class: "sheet-close",
                            "aria-label": "Close",
                            onclick: move |_| on_leave.call(()),
                            "\u{00d7}"
                        }
                        span { class: "sheet-title", "{name}" }
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
                            Some(FlowKey::Select) => {
                                e.prevent_default();
                                if !read_only {
                                    on_select.call(());
                                }
                            }
                            None => {}
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
                                    PrintingChipsGhost {}
                                    if !read_only {
                                        span { class: "skeleton-bar skeleton-chip web-flow-ghost-select" }
                                    }
                                }
                            } else if let Some(card) = selected {
                                div { class: "web-flow-info",
                                    PrintingChips { card, at, count }
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

/// Which printing `card` is and what it costs, as one centered row of chips that wraps: its set
/// (cut short first), its collector number, its place among the `count` printings (`at` from 0),
/// its artist and its prices. Every Printings sheet draws it under its cover flow.
#[component]
pub fn PrintingChips(card: Card, at: usize, count: usize) -> Element {
    let prices = info::prices(&card.scryfall_data);
    rsx! {
        div { class: "printing-chips",
            span { class: "stat-chip stat-chip-p3 printing-chips-set", "{card.scryfall_data.set_name}" }
            span { class: "stat-chip stat-chip-muted", "#{card.scryfall_data.collector_number}" }
            span { class: "stat-chip stat-chip-muted", "{at.saturating_add(1)} of {count}" }
            ArtistChip { card }
            for price in prices {
                span { key: "{price}", class: "stat-chip stat-chip-money", "{price}" }
            }
        }
    }
}

/// How many price chips a printing's ghost shows: dollars, euros and tix.
const GHOST_PRICES: usize = 3;

/// The ghost of [`PrintingChips`] while the printings load: each chip's ghost in its tone, at
/// about its size.
#[component]
pub fn PrintingChipsGhost() -> Element {
    rsx! {
        div { class: "printing-chips",
            span { class: "skeleton-bar skeleton-chip skeleton-tinted stat-chip-p3 printing-chips-ghost-set" }
            span { class: "skeleton-bar skeleton-chip skeleton-tinted stat-chip-muted printing-chips-ghost-number" }
            span { class: "skeleton-bar skeleton-chip skeleton-tinted stat-chip-muted printing-chips-ghost-number" }
            span { class: "skeleton-bar skeleton-chip skeleton-tinted stat-chip-p6 printing-chips-ghost-artist" }
            for i in 0..GHOST_PRICES {
                span { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-tinted stat-chip-money printing-chips-ghost-price" }
            }
        }
    }
}

/// The artist of `card`, a printing, as a read-only stat chip in the artist's tone, like the
/// chips beside it: the Image icon, then the name. Nothing for a printing with no artist.
#[component]
pub fn ArtistChip(card: Card) -> Element {
    let Some(artist) = info::artist(&card.scryfall_data) else {
        return rsx! {};
    };
    rsx! {
        span { class: "stat-chip stat-chip-p6 printing-artist",
            DockIcon { path: IMAGE_ICON }
            "{artist}"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_goes_back_to_the_saved_printings_button_else_the_card_itself() {
        let id = Uuid::nil();
        let selectors = handback_selectors(id);
        assert_eq!(selectors.len(), 2, "{selectors:?}");
        assert_eq!(
            selectors.first().map(String::as_str),
            Some(format!("[data-row=\"{id}\"] [data-action=\"printings\"], [data-card=\"{id}\"] [data-action=\"printings\"]").as_str())
        );
        assert!(
            selectors.get(1).is_some_and(|s| s.contains(&format!(
                ".card-stack-slot[data-row=\"{id}\"] .card-stack-card"
            ))),
            "{selectors:?}"
        );
    }
}

//! The printing sheet: every printing of a card, one at a time, to pick one
//! or only to look.
//!
//! The printings are components' cover flow, as in the app: the selected
//! one large in the middle and up to ten on each side stacked behind it,
//! moved by the arrows, a sideways scroll, a drag or swipe, or a tap on a
//! side card. From the kit's modal width it fills a large modal, with Enter,
//! a click on the middle card or `Select` picking it. Below that width it
//! sits in a centered modal down to the breakpoint and the app's bottom
//! sheet under it, with the prices at its bottom and Save at their right.
//! It closes from its title bar's ×, a backdrop tap, a drag down or Esc.

use components::icons::{CHECK, DockIcon};
use components::{
    BottomSheet, Button, ButtonVariant, CoverFlow, SheetFrame, SheetWide, TOAST_QUICK,
    use_overlay_back_action,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::card::Card;
use zwipe_ui::card::info;

use crate::inbound::cards::card_info::{CardInfoDisplay, PrintingSheetSkeleton};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::media::{KIT_MODAL_PX, use_min_width};

/// Which way a step through the printings goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    /// To the printing before.
    Back,
    /// To the printing after.
    Forward,
}

/// The printings of `card`, opening on the one shown. `on_save` takes the
/// picked printing and toasts `saved_message`; `read_only` hides the dock.
/// Leaving with a different printing showing toasts `Printing unchanged`.
#[component]
pub fn PrintingSheet(
    card: Card,
    mut open: Signal<bool>,
    on_save: EventHandler<Card>,
    #[props(default)] read_only: bool,
    #[props(default = "Printing saved")] saved_message: &'static str,
) -> Element {
    let authed = use_authed();
    let toast = use_toast();
    // From the kit's modal width on, the picker is the cover flow.
    let flow = use_min_width(KIT_MODAL_PX);
    let mut printings: Signal<Vec<Card>> = use_signal(Vec::new);
    let mut loading = use_signal(|| false);
    let mut index = use_signal(|| 0usize);
    let mut initial = use_signal(|| 0usize);
    let shown_id = card.scryfall_data.id;
    let oracle_id = card.scryfall_data.oracle_id;

    use_effect(move || {
        if !open() {
            return;
        }
        let Some(oracle_id) = oracle_id else {
            return;
        };
        loading.set(true);
        spawn(async move {
            if let Some(cards) = authed
                .run("load_printings", |c, _| async move {
                    c.fetch_printings(oracle_id).await
                })
                .await
            {
                let at = cards
                    .iter()
                    .position(|p| p.scryfall_data.id == shown_id)
                    .unwrap_or(0);
                printings.set(cards);
                index.set(at);
                initial.set(at);
            }
            loading.set(false);
        });
    });

    let all = printings();
    let count = all.len();
    let at = index();
    let changed = count > 0 && at != initial();
    let visible = all.get(at).cloned();

    let leave = use_callback(move |(): ()| {
        if !read_only && changed {
            toast.info(
                "Printing unchanged".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
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
        if read_only || !changed {
            return;
        }
        if let Some(picked) = visible_for_save.clone() {
            on_save.call(picked);
            toast.info(
                saved_message.to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
            open.set(false);
        }
    });

    if flow() {
        return rsx! {
            CoverFlowSheet {
                open,
                printings: all,
                index,
                loading: loading(),
                changed,
                read_only,
                on_leave: leave,
                on_select: save,
            }
        };
    }

    rsx! {
        // `sheet-over` lifts the sheet above a details sheet already open
        // under it. The body takes the arrow keys and Enter once clicked.
        div { class: "web-sheet web-sheet-modal web-printing-sheet sheet-over",
            BottomSheet {
                open,
                title: "Printings".to_string(),
                closable: true,
                on_dismiss: move |()| leave.call(()),
                div {
                    class: "web-printing-body",
                    tabindex: "-1",
                    onkeydown: move |e: Event<KeyboardData>| match e.key() {
                        Key::ArrowLeft => step(Step::Back),
                        Key::ArrowRight => step(Step::Forward),
                        Key::Enter => save.call(()),
                        _ => {}
                    },
                    if loading() {
                        PrintingSheetSkeleton {}
                    } else if let Some(card) = visible {
                        CoverFlow { printings: all, index }
                        // The prices with Save at their right, at the
                        // sheet's bottom. Save is grayed until a different
                        // printing shows.
                        div { class: "web-printing-foot",
                            div { class: "web-printing-foot-row",
                                CardInfoDisplay { card }
                                if !read_only {
                                    div { class: "action-dock web-printing-save",
                                        button {
                                            class: "action-dock-btn",
                                            "aria-label": "Save",
                                            disabled: !changed,
                                            onclick: move |_| save.call(()),
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
    changed: bool,
    read_only: bool,
    on_leave: Callback<()>,
    on_select: Callback<()>,
) -> Element {
    use_overlay_back_action(open.into(), on_leave);
    let count = printings.len();
    let at = index().min(count.saturating_sub(1));
    let selected = printings.get(at).cloned();

    // The body takes the keys as soon as the modal opens.
    use_effect(move || {
        if open() {
            document::eval(
                "setTimeout(() => { const b = document.querySelector('.web-flow-body'); if (b) b.focus(); }, 60)",
            );
        }
    });

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
        .map(|c| info::prices(&c.scryfall_data))
        .unwrap_or_default();

    rsx! {
        div { class: "web-flow",
            SheetFrame { wide: SheetWide::Modal,
                div {
                    class: if open() { "sheet-backdrop show" } else { "sheet-backdrop" },
                    onclick: move |_| on_leave.call(()),
                }
                div { class: if open() { "bottom-sheet show web-flow-sheet" } else { "bottom-sheet web-flow-sheet" },
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
                        onkeydown: move |e: Event<KeyboardData>| match e.key() {
                            Key::ArrowLeft => {
                                e.prevent_default();
                                step(Step::Back);
                            }
                            Key::ArrowRight => {
                                e.prevent_default();
                                step(Step::Forward);
                            }
                            Key::Enter if !read_only => on_select.call(()),
                            _ => {}
                        },
                        if loading {
                            PrintingSheetSkeleton {}
                        } else {
                            CoverFlow {
                                printings,
                                index,
                                on_pick: (!read_only).then(|| EventHandler::new(move |()| on_select.call(()))),
                            }
                            if let Some(card) = selected {
                                div { class: "web-flow-info",
                                    span { class: "stat-chip stat-chip-p3", "{card.scryfall_data.set_name}" }
                                    span { class: "stat-chip stat-chip-muted", "#{card.scryfall_data.collector_number}" }
                                    for price in prices {
                                        span { key: "{price}", class: "stat-chip stat-chip-money", "{price}" }
                                    }
                                    span { class: "web-flow-count", "{at.saturating_add(1)} of {count}" }
                                    if !read_only {
                                        Button {
                                            variant: ButtonVariant::Action,
                                            class: "web-flow-select",
                                            disabled: !changed,
                                            onclick: move |_| on_select.call(()),
                                            "Select"
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

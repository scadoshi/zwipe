//! The printing sheet: every printing of a card, one at a time, to pick one
//! or only to look.
//!
//! The app pages through printings with a swipe carousel. From the kit's
//! modal width the printings are a cover flow in a large modal: the
//! selected one large in the middle and up to ten on each side stacked
//! behind it, moved by the wheel or a trackpad's sideways scroll, the
//! arrows, a drag or a click on a side card, with Enter, a click on the
//! middle card or `Select` picking it. Below that width `Previous` and
//! `Next` (and the arrows) step through them one at a time, in a centered
//! modal down to the breakpoint and the app's bottom sheet under it, with
//! the dots and the prices at its bottom and Save at the prices' right. It
//! closes from its title bar's ×, a backdrop tap, a drag down or Esc.

use components::icons::{ARROW_LEFT, ARROW_RIGHT, CHECK, DockIcon};
use components::{
    BottomSheet, Button, ButtonVariant, SheetFrame, SheetWide, TOAST_QUICK, use_overlay_back_action,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;
use zwipe_ui::card::info;

use crate::inbound::cards::card_info::{CardInfoDisplay, PrintingSheetSkeleton};
use crate::inbound::cards::cover_flow::{flow_slot, flow_window};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::media::use_min_width;

/// The width, 60rem, from which the kit draws a sheet form as a modal, and
/// the picker is the cover flow.
const KIT_MODAL_PX: u32 = 960;

/// How far a wheel or trackpad scrolls, in pixels, to move one printing.
const WHEEL_STEP_PX: f64 = 60.0;

/// How far a drag runs, in pixels, to move one printing.
const DRAG_STEP_PX: f64 = 70.0;

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

    let mut step = move |forward: bool| {
        if count < 2 {
            return;
        }
        let next = if forward {
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
                        Key::ArrowLeft => step(false),
                        Key::ArrowRight => step(true),
                        Key::Enter => save.call(()),
                        _ => {}
                    },
                    if loading() {
                        PrintingSheetSkeleton {}
                    } else if let Some(card) = visible {
                        div { class: "web-printing-stage",
                            if count > 1 {
                                button {
                                    class: "card-action-btn web-printing-step",
                                    disabled: at == 0,
                                    "aria-label": "Previous printing",
                                    onclick: move |_| step(false),
                                    DockIcon { path: ARROW_LEFT }
                                }
                            }
                            components::FlippableCardImage {
                                key: "{card.scryfall_data.id}",
                                sd: card.scryfall_data.clone(),
                                size: ImageSize::Large,
                                class: "carousel-card-image".to_string(),
                            }
                            if count > 1 {
                                button {
                                    class: "card-action-btn web-printing-step",
                                    disabled: at.saturating_add(1) >= count,
                                    "aria-label": "Next printing",
                                    onclick: move |_| step(true),
                                    DockIcon { path: ARROW_RIGHT }
                                }
                            }
                        }
                        // The dots, then the prices with Save at their
                        // right, at the sheet's bottom. Save is grayed
                        // until a different printing shows.
                        div { class: "web-printing-foot",
                            if count > 1 {
                                CarouselDots { current: at, total: count }
                            }
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

/// Distance between dot centers in rem: a 0.5rem dot plus a 0.375rem gap.
const DOT_PITCH_REM: f64 = 0.875;
/// Center of the first dot from the strip's left edge in rem: half a dot.
const FIRST_DOT_CENTER_REM: f64 = 0.25;

/// The app's page dots under the printing, the strip sliding so the current
/// one stays centered.
#[component]
fn CarouselDots(current: usize, total: usize) -> Element {
    let index = f64::from(u32::try_from(current).unwrap_or(u32::MAX));
    let offset_rem = index * DOT_PITCH_REM + FIRST_DOT_CENTER_REM;
    rsx! {
        div { class: "carousel-dots",
            div {
                class: "carousel-dots-track",
                style: "margin-left: 50%; transform: translateX(-{offset_rem}rem);",
                for i in 0..total {
                    div { key: "{i}", class: if i == current { "carousel-dot active" } else { "carousel-dot" } }
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
    let mut wheel = use_signal(|| 0.0_f64);
    let mut drag_from: Signal<Option<f64>> = use_signal(|| None);
    let mut dragged = use_signal(|| false);
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

    let mut go = move |next: usize| {
        if count > 0 {
            index.set(next.min(count.saturating_sub(1)));
        }
    };
    let mut step = move |forward: bool| {
        let current = *index.peek();
        go(if forward {
            current.saturating_add(1)
        } else {
            current.saturating_sub(1)
        });
    };

    let cards = flow_window(at, count)
        .into_iter()
        .filter_map(|(i, offset)| printings.get(i).map(|card| (i, offset, card.clone())));
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
                                step(false);
                            }
                            Key::ArrowRight => {
                                e.prevent_default();
                                step(true);
                            }
                            Key::Enter if !read_only => on_select.call(()),
                            _ => {}
                        },
                        if loading {
                            PrintingSheetSkeleton {}
                        } else {
                            div { class: "web-flow-row",
                                button {
                                    class: "btn-action btn-icon web-flow-step",
                                    disabled: at == 0,
                                    "aria-label": "Previous printing",
                                    onclick: move |_| step(false),
                                    // Enter presses the button and stays off the body's Select.
                                    onkeydown: move |e: Event<KeyboardData>| {
                                        if e.key() == Key::Enter {
                                            e.stop_propagation();
                                        }
                                    },
                                    DockIcon { path: ARROW_LEFT }
                                }
                                div {
                                    class: "web-flow-stage",
                                    onwheel: move |e: Event<WheelData>| {
                                        let delta = e.delta().strip_units();
                                        let along = if delta.x.abs() > delta.y.abs() { delta.x } else { delta.y };
                                        let mut sum = *wheel.peek() + along;
                                        while sum >= WHEEL_STEP_PX {
                                            step(true);
                                            sum -= WHEEL_STEP_PX;
                                        }
                                        while sum <= -WHEEL_STEP_PX {
                                            step(false);
                                            sum += WHEEL_STEP_PX;
                                        }
                                        wheel.set(sum);
                                    },
                                    onmousedown: move |e: Event<MouseData>| {
                                        e.prevent_default();
                                        drag_from.set(Some(e.client_coordinates().x));
                                        dragged.set(false);
                                    },
                                    onmousemove: move |e: Event<MouseData>| {
                                        let Some(from) = *drag_from.peek() else {
                                            return;
                                        };
                                        let x = e.client_coordinates().x;
                                        let run = x - from;
                                        if run.abs() >= DRAG_STEP_PX {
                                            // Dragging left brings the next printing in.
                                            step(run < 0.0);
                                            drag_from.set(Some(x));
                                            dragged.set(true);
                                        }
                                    },
                                    onmouseup: move |_| drag_from.set(None),
                                    onmouseleave: move |_| drag_from.set(None),
                                    for (i, offset, card) in cards {
                                        div {
                                            key: "{card.scryfall_data.id}",
                                            class: if offset == 0 { "web-flow-card selected" } else { "web-flow-card" },
                                            style: flow_slot(offset).style(),
                                            onclick: move |_| {
                                                if *dragged.peek() {
                                                    return;
                                                }
                                                if offset == 0 {
                                                    if !read_only {
                                                        on_select.call(());
                                                    }
                                                } else {
                                                    go(i);
                                                }
                                            },
                                            if let Some(url) = card.scryfall_data.primary_image_url(ImageSize::Large) {
                                                img {
                                                    src: "{url}",
                                                    alt: "{card.scryfall_data.name}, {card.scryfall_data.set_name}",
                                                    loading: "lazy",
                                                    draggable: false,
                                                }
                                            } else {
                                                div { class: "web-flow-missing", "{card.scryfall_data.set_name}" }
                                            }
                                        }
                                    }
                                }
                                button {
                                    class: "btn-action btn-icon web-flow-step",
                                    disabled: at.saturating_add(1) >= count,
                                    "aria-label": "Next printing",
                                    onclick: move |_| step(true),
                                    // Enter presses the button and stays off the body's Select.
                                    onkeydown: move |e: Event<KeyboardData>| {
                                        if e.key() == Key::Enter {
                                            e.stop_propagation();
                                        }
                                    },
                                    DockIcon { path: ARROW_RIGHT }
                                }
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

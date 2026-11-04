//! The printing sheet: every printing of a card, one at a time, to pick one
//! or only to look.
//!
//! The app pages through printings with a swipe carousel; here `Previous`
//! and `Next` (and the left and right arrows) step through them. On a wide
//! window the sheet is a centered modal; below the breakpoint it is the
//! app's bottom sheet. It closes from its title bar's ×, a backdrop tap, a
//! drag down or Esc; Save is the checkmark in its dock.

use components::icons::CHECK;
use components::{BottomSheet, DockIcon, TOAST_QUICK};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use crate::inbound::cards::card_info::{CardInfoDisplay, PrintingSheetSkeleton};
use crate::inbound::chrome::authed::use_authed;

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
                    } else if let Some(card) = visible.clone() {
                        div { class: "web-printing-stage",
                            if count > 1 {
                                button {
                                    class: "card-action-btn web-printing-step",
                                    disabled: at == 0,
                                    "aria-label": "Previous printing",
                                    onclick: move |_| step(false),
                                    "\u{2190}"
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
                                    "\u{2192}"
                                }
                            }
                        }
                        if count > 1 {
                            CarouselDots { current: at, total: count }
                        }
                        CardInfoDisplay { card }
                    }
                }
                // Save floats at the sheet's bottom right, grayed until a
                // different printing shows.
                if !read_only {
                    div { class: "action-dock action-dock-right sheet-dock",
                        button {
                            class: "action-dock-btn",
                            "aria-label": "Save",
                            disabled: !changed || visible.is_none(),
                            onclick: move |_| save.call(()),
                            DockIcon { path: CHECK }
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

//! A card's prices as chips, for the printing sheet, and a swipe card's
//! full details in the drawer behind the dock's eye.

use components::icons::{FLIP, LAYERS};
use components::{CardDetailsPanel, DockAction, DrawerSheet, use_overlay_back};
use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_ui::card::info;

use crate::inbound::cards::otag::use_describe_tag;

/// The card's prices as chips, under a printing in the printing sheet.
#[component]
pub fn CardInfoDisplay(card: Card) -> Element {
    let prices = info::prices(&card.scryfall_data);
    rsx! {
        div { class: "card-prices",
            for price in prices {
                span { key: "{price}", class: "card-row-price card-price-chip", "{price}" }
            }
        }
    }
}

/// The card's details as a drawer from the right edge, at every width,
/// behind the eye button: the card as a [`CardDetailsPanel`] with the host's
/// `footer` in its foot, the panel's title the drawer's only title. It closes from its title bar's ×, a backdrop tap,
/// Escape or the back gesture. Its dock, there only when it has something to
/// hold, has `Flip` for a multi-faced card and `Printings` when the host
/// passes `on_printings`. Mount it at the screen root beside the printing
/// sheet.
#[component]
pub fn CardDetailsSheet(
    open: Signal<bool>,
    card: Card,
    #[props(default)] on_printings: Option<EventHandler<()>>,
    #[props(default)] footer: Option<Element>,
) -> Element {
    let face_count = components::card_face_count(&card);
    let mut face = use_signal(|| 0usize);
    let describe_tag = use_describe_tag();
    use_overlay_back(open);
    // Flip and Printings as the drawer's dock, pinned at its bottom right.
    let dock = (face_count > 1 || on_printings.is_some()).then(|| {
        rsx! {
            if face_count > 1 {
                DockAction {
                    label: "Flip",
                    icon: FLIP,
                    onclick: move |()| face.set(face().saturating_add(1).checked_rem(face_count).unwrap_or(0)),
                }
            }
            if let Some(handler) = on_printings {
                DockAction { label: "Printings", icon: LAYERS, onclick: move |()| handler.call(()) }
            }
        }
    });
    rsx! {
        DrawerSheet {
            open,
            on_close: move |()| {
                let mut open = open;
                open.set(false);
            },
            every_width: true,
            class: "card-details-drawer",
            dock,
            CardDetailsPanel {
                card,
                show_flip: false,
                face: Some(face),
                describe_tag: Some(describe_tag),
                footer,
            }
        }
    }
}

/// A swipe card's ghost while its cards load: the image block, with "No
/// cards" in it once a finished search came back empty.
#[component]
pub fn CardSkeleton(#[props(default)] is_loading: bool) -> Element {
    rsx! {
        div { class: "skeleton-card",
            // The stack's slot, the card's ghost centered in it as the real
            // card is.
            div { class: "skeleton-stack",
                div { class: "skeleton-image",
                    if !is_loading {
                        "No cards"
                    }
                }
            }
        }
    }
}

/// The printing sheet's ghost: the image, the dots and the info lines.
#[component]
pub fn PrintingSheetSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-printing",
            div { class: "skeleton-printing-image" }
            div { class: "skeleton-printing-dots",
                for i in 0..5 {
                    div { key: "{i}", class: "skeleton-printing-dot" }
                }
            }
            div { class: "skeleton-printing-info",
                div { class: "skeleton-bar skeleton-printing-info-price" }
                div { class: "skeleton-bar skeleton-printing-info-set" }
                div { class: "skeleton-bar skeleton-printing-info-released" }
                div { class: "skeleton-bar skeleton-printing-info-artist" }
            }
        }
    }
}

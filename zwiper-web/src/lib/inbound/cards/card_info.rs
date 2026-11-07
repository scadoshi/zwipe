//! A card's prices as chips, for the printing sheet, and a swipe card's
//! full details in the drawer behind the dock's eye.

use components::{CardDetailsPanel, DrawerSheet, use_overlay_back};
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
/// `footer` in its foot and `body_end` scrolling at its body's end, the panel's title the drawer's only title. It closes from its title bar's ×, a backdrop tap,
/// Escape or the back gesture. The panel's foot has `Flip` for a
/// multi-faced card and a `Printings` pill when the host passes
/// `on_printings`, as the wide pane does. Mount it at the screen root
/// beside the printing sheet.
#[component]
pub fn CardDetailsSheet(
    open: Signal<bool>,
    card: Card,
    #[props(default)] on_printings: Option<EventHandler<()>>,
    #[props(default)] footer: Option<Element>,
    #[props(default)] body_end: Option<Element>,
) -> Element {
    let describe_tag = use_describe_tag();
    use_overlay_back(open);
    rsx! {
        DrawerSheet {
            open,
            on_close: move |()| {
                let mut open = open;
                open.set(false);
            },
            every_width: true,
            class: "card-details-drawer",
            CardDetailsPanel {
                card,
                describe_tag: Some(describe_tag),
                on_printings,
                footer,
                body_end,
            }
        }
    }
}

/// A swipe card's ghost: the image block, while cards load and where the
/// stack has run out.
#[component]
pub fn CardSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-card",
            // The stack's slot, the card's ghost centered in it as the real
            // card is.
            div { class: "skeleton-stack",
                div { class: "skeleton-image" }
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

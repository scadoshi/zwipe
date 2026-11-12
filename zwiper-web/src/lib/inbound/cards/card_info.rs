//! A card's prices as chips, for the printing sheet, and a swipe card's
//! full details in the drawer behind the dock's eye.

use components::CardDetailsSheet as SharedDetailsSheet;
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
/// behind the eye button: the card's name and mana cost in the drawer's
/// title bar beside the ×, then the card as a headless [`CardDetailsPanel`]
/// with the host's `footer` in its foot and `body_end` scrolling at its
/// body's end, under the title bar. It closes from its title bar's ×, a backdrop tap,
/// Escape or the back gesture. The panel's foot has `Flip` for a
/// multi-faced card, `Image` when the host passes `on_image` (called with
/// the shown face) and the card has art, and a `Printings` pill when the
/// host passes `on_printings`, as the wide pane does. `class` names the drawer, and a
/// host that adds its own class keeps `card-details-drawer` in it. Swapping
/// to another card while open crossfades. Mount it at
/// the screen root beside the printing sheet.
#[component]
pub fn CardDetailsSheet(
    open: Signal<bool>,
    card: Card,
    #[props(default)] on_image: Option<EventHandler<usize>>,
    #[props(default)] on_printings: Option<EventHandler<()>>,
    #[props(default)] footer: Option<Element>,
    #[props(default)] body_end: Option<Element>,
    #[props(default = "card-details-drawer")] class: &'static str,
    #[props(default)] face: Option<usize>,
    #[props(default)] on_face_change: Option<EventHandler<usize>>,
) -> Element {
    let describe_tag = use_describe_tag();
    rsx! {
        SharedDetailsSheet {
            open,
            card,
            on_image,
            on_printings,
            footer,
            body_end,
            class,
            face,
            on_face_change,
            describe_tag: Some(describe_tag),
        }
    }
}

/// A swipe card's ghost: the image block, while cards load and where the
/// stack has run out. `note` says why a filter left nothing to deal, and
/// `detail`, under it, whether Synergy may be the reason.
#[component]
pub fn CardSkeleton(
    #[props(default)] note: Option<&'static str>,
    #[props(default)] detail: Option<&'static str>,
) -> Element {
    rsx! {
        div { class: "skeleton-card",
            // The stack's slot, the card's ghost centered in it as the real
            // card is.
            div { class: "skeleton-stack",
                if let Some(note) = note {
                    div { class: "skeleton-image skeleton-note",
                        span { {note} }
                        if let Some(detail) = detail {
                            span { class: "skeleton-note-detail", {detail} }
                        }
                    }
                } else {
                    div { class: "skeleton-image" }
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

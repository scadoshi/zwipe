//! A swipe card's text: its prices under the card, and its full details in
//! a sheet behind the dock's eye.

use components::{BottomSheet, Button, ButtonVariant, CardDetails};
use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_ui::card::info;

use crate::inbound::cards::otag::use_describe_tag;

/// The card's prices as chips under a swipe card, as the app shows them.
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

/// The card's details (cost, type, oracle text, keywords, roles) as the
/// sheet behind the eye button. It closes from its title bar's × or a drag
/// down; its own footer, there only when it has something to hold, has
/// `Flip` for a multi-faced card and `Printings` when the host passes
/// `on_printings`, as the app's does. Mount it at the screen root beside
/// the printing sheet.
#[component]
pub fn CardDetailsSheet(
    open: Signal<bool>,
    card: Card,
    #[props(default)] on_printings: Option<EventHandler<()>>,
    #[props(default)] footer: Option<Element>,
) -> Element {
    let name = card.scryfall_data.name.clone();
    let face_count = components::card_face_count(&card);
    let mut face = use_signal(|| 0usize);
    let describe_tag = use_describe_tag();
    let actions = (face_count > 1 || on_printings.is_some()).then(|| {
        rsx! {
            if face_count > 1 {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| face.set(face().saturating_add(1).checked_rem(face_count).unwrap_or(0)),
                    "Flip"
                }
            }
            if let Some(handler) = on_printings {
                Button { variant: ButtonVariant::Action, onclick: move |_| handler.call(()), "Printings" }
            }
        }
    });
    rsx! {
        BottomSheet {
            open,
            title: name,
            closable: true,
            footer: actions,
            div { class: "card-rules",
                CardDetails {
                    card,
                    show_name: false,
                    show_cost: true,
                    show_classification: true,
                    show_flip: false,
                    face: Some(face),
                    describe_tag: Some(describe_tag),
                }
                if let Some(footer) = footer {
                    {footer}
                }
            }
        }
    }
}

/// A swipe card's ghost while its cards load: the image block and the
/// price line, with "No cards" in the block once a finished search came
/// back empty.
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
            // The price line's ghost: two chip-sized pills, centered.
            div { class: "card-prices",
                div { class: "skeleton-bar skeleton-price-chip" }
                div { class: "skeleton-bar skeleton-price-chip" }
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

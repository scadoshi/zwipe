//! A swipe card's text: its prices, set and artist under the card, and its
//! full details in a sheet behind the eye button.

use components::{BottomSheet, Button, ButtonVariant, CardDetails};
use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_ui::card::info;

use crate::inbound::cards::otag::use_describe_tag;

/// The card's prices, set, release date and artist, as the app shows them
/// under a swipe card.
#[component]
pub fn CardInfoDisplay(card: Card) -> Element {
    let data = &card.scryfall_data;
    let blank = || "\u{a0}".to_string();
    let price_text = info::price_line(data).unwrap_or_else(blank);
    let set_text = info::set_line(data);
    let released_text = info::released_line(data);
    let artist_text = info::artist_line(data).unwrap_or_else(blank);
    rsx! {
        div { class: "card-info",
            span { class: "card-info-name", "{card.scryfall_data.name}" }
            span { "{price_text}" }
            span { "{set_text}" }
            span { "{released_text}" }
            span { "{artist_text}" }
        }
    }
}

/// The action bar's eye button, which opens and closes the card's
/// [`CardDetailsSheet`].
#[component]
pub fn RulesButton(open: Signal<bool>) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Action,
            class: "btn-action-eye",
            onclick: move |_| open.toggle(),
            svg {
                class: "eye-icon",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                path { d: "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" }
                circle { cx: "12", cy: "12", r: "3" }
            }
        }
    }
}

/// The card's details (cost, type, oracle text, keywords, roles) as the
/// sheet behind the eye button: `Back`, `Flip` for a multi-faced card, and
/// `Printings` when the host passes `on_printings`. Mount it at the screen
/// root beside the printing sheet.
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
    let mut open_for_back = open;
    rsx! {
        BottomSheet {
            open,
            title: name,
            footer: rsx! {
                Button { variant: ButtonVariant::Action, onclick: move |_| open_for_back.set(false), "Back" }
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
            },
            div { class: "card-rules",
                div { class: "sheet-panel",
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
}

/// A swipe card's ghost while its cards load: the image block and the info
/// lines, with "No cards" in the block once a finished search came back empty.
#[component]
pub fn CardSkeleton(#[props(default)] is_loading: bool) -> Element {
    rsx! {
        div { class: "skeleton-card",
            div { class: "skeleton-image",
                if !is_loading {
                    "No cards"
                }
            }
            div { class: "skeleton-info",
                div { class: "skeleton-bar skeleton-bar-name" }
                div { class: "skeleton-bar skeleton-bar-price" }
                div { class: "skeleton-bar skeleton-bar-set" }
                div { class: "skeleton-bar skeleton-bar-date" }
                div { class: "skeleton-bar skeleton-bar-artist" }
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

//! A swipe card's full details in the drawer behind the dock's eye.

use components::CardDetailsSheet as SharedDetailsSheet;
use dioxus::prelude::*;
use zwipe_core::domain::card::Card;

use crate::inbound::cards::otag::use_describe_tag;

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

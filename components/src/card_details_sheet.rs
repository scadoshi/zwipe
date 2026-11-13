//! A card's details in a drawer: the card details panel under a title bar with the card's name
//! and cost, shared by the deck page's stacks and the web's card screens.

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;

use crate::drawer_swap::use_drawer_swap;
use crate::{
    CardDetailsCost, CardDetailsPanel, DrawerEdge, DrawerFocus, DrawerSheet, use_meld,
    use_overlay_back,
};

/// `card`'s details as a drawer, while `open`. Its foot carries `Flip` for a multi-faced card,
/// `Image` when the host passes `on_image` (called with the shown face) and the card has art, and
/// a `Printings` pill when the host passes `on_printings`. `describe_tag` gives an Oracle tag's
/// description, where the host has a catalog, and `on_examples` opens a tag's example cards,
/// where the host shows them. `class` names the drawer, and a host that adds its
/// own class keeps `card-details-drawer` in it. The shown face follows `face` when the host
/// passes one. The drawer carries the card's id as its `data-card`, so the keys that act on a
/// card find the one it shows. `edge` is the edge it runs along, the right unless given. Swapping to another card while it is open crossfades (`use_drawer_swap`).
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
    #[props(default)] describe_tag: Option<Callback<String, Option<String>>>,
    #[props(default)] on_examples: Option<Callback<String>>,
    #[props(default)] edge: DrawerEdge,
) -> Element {
    let layer = use_overlay_back(open);
    use_drawer_swap();
    // The shown face, shared by the title bar's cost and the panel's Flip,
    // and kept on the host's `face` when it passes one.
    let mut shown_face = use_signal(|| face.unwrap_or(0));
    use_effect(use_reactive!(|face| {
        if let Some(face) = face {
            shown_face.set(face);
        }
    }));
    let face = shown_face;
    // The meld toggle, shared by the title bar and the panel, so the title
    // shows the melded card's name and cost (none) while it is on.
    let meld = use_meld(&card.scryfall_data);
    let title_card = meld.apply(&card);
    let name = title_card.scryfall_data.name.clone();
    let on_image = on_image.map(|handler| EventHandler::new(move |()| handler.call(face())));
    let shown_id = card.scryfall_data.id.to_string();
    rsx! {
        DrawerSheet {
            layer: layer(),
            open,
            title: name,
            title_trailing: rsx! { CardDetailsCost { card: title_card, face } },
            on_close: move |()| {
                let mut open = open;
                open.set(false);
            },
            every_width: true,
            edge,
            focus: DrawerFocus::Kept,
            class,
            data_card: shown_id,
            CardDetailsPanel {
                card,
                headless: true,
                face,
                meld: Some(meld),
                describe_tag,
                on_examples,
                on_image,
                on_face_change,
                on_printings,
                footer,
                body_end,
            }
        }
    }
}

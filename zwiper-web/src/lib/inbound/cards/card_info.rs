//! A card's prices as chips, for the printing sheet, and a swipe card's
//! full details in the drawer behind the dock's eye.

use std::time::Duration;

use components::{CardDetailsCost, CardDetailsPanel, DrawerSheet, use_meld, use_overlay_back};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_ui::card::info;

use crate::inbound::cards::otag::use_describe_tag;
use crate::outbound::timer;

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
/// host that adds its own class keeps `card-details-drawer` in it. Mount it
/// at the screen root beside the printing sheet.
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
    use_overlay_back(open);
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
    // A swap to another card while open crossfades: the card drawn last
    // stays as a layer fading out over the new one for one entrance.
    let card_id = card.scryfall_data.id;
    let mut drawn = use_hook(|| CopyValue::new(None::<Outgoing>));
    let mut outgoing = use_hook(|| CopyValue::new(None::<(Outgoing, u32)>));
    let mut swaps = use_signal(|| 0u32);
    let swap_turn = swaps();
    let before = drawn.take();
    let open_now = *open.peek();
    let was_open = before.as_ref().is_some_and(|o| o.open);
    if crossfades(
        before.as_ref().map(|o| o.card.scryfall_data.id),
        card_id,
        was_open && open_now,
    ) {
        let turn = swap_turn.wrapping_add(1);
        outgoing.set(before.map(|o| (o, turn)));
        spawn(async move {
            timer::sleep(SWAP).await;
            if outgoing.read().as_ref().is_some_and(|(_, t)| *t == turn) {
                outgoing.set(None);
                swaps.set(turn);
            }
        });
    }
    drawn.set(Some(Outgoing {
        card: card.clone(),
        footer: footer.clone(),
        body_end: body_end.clone(),
        open: open_now,
    }));
    let fading = outgoing.read().as_ref().map(|(o, _)| o.clone());
    rsx! {
        DrawerSheet {
            open,
            title: name,
            title_trailing: rsx! { CardDetailsCost { card: title_card, face } },
            on_close: move |()| {
                let mut open = open;
                open.set(false);
            },
            every_width: true,
            class,
            div { class: "card-details-crossfade",
                if let Some(old) = fading {
                    div {
                        class: "card-details-out",
                        "aria-hidden": "true",
                        inert: true,
                        CardDetailsPanel {
                            card: old.card,
                            headless: true,
                            describe_tag: Some(describe_tag),
                            footer: old.footer,
                            body_end: old.body_end,
                        }
                    }
                }
                // A one-card list, so its key remounts the layer, and its
                // entrance runs, only when the card changes.
                for shown_id in [card_id] {
                    div { key: "{shown_id}", class: "card-details-in",
                        CardDetailsPanel {
                            card: card.clone(),
                            headless: true,
                            face,
                            meld: Some(meld.clone()),
                            describe_tag: Some(describe_tag),
                            on_image,
                            on_face_change,
                            on_printings,
                            footer: footer.clone(),
                            body_end: body_end.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// How long a swapped-out card takes to fade under the new one: the kit's `--enter-duration`.
const SWAP: Duration = Duration::from_millis(280);

/// What the drawer drew for one card, kept to fade out when it swaps to another.
#[derive(Clone)]
struct Outgoing {
    /// The card.
    card: Card,
    /// The host's footer for it.
    footer: Option<Element>,
    /// The host's body end for it.
    body_end: Option<Element>,
    /// Whether the drawer was open when it drew the card.
    open: bool,
}

/// Whether the drawer crossfades on drawing card `now` after `before`: only a swap to another
/// card while it stays `open`. A face or meld change, or a card a drawer opens on, swaps in place.
fn crossfades(before: Option<Uuid>, now: Uuid, open: bool) -> bool {
    open && before.is_some_and(|was| was != now)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_swap_in_an_open_drawer_crossfades() {
        let (a, b) = (Uuid::from_u128(1), Uuid::from_u128(2));
        assert!(crossfades(Some(a), b, true), "another card while open");
        assert!(
            !crossfades(Some(a), a, true),
            "the same card: a face or meld change"
        );
        assert!(
            !crossfades(Some(a), b, false),
            "a closed drawer just shows the new card"
        );
        assert!(!crossfades(None, b, true), "nothing shown before");
    }
}

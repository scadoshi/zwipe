//! One group of a deck's cards as a stack: the cards overlap, each tucked
//! under the one above it so only its title strip shows, the last in full.
//! Which card tucks, which move down to reveal one, and which show in full
//! come from zwipe-ui's [`stacks::tucked`], [`stacks::shifted`] and
//! [`stacks::exposed`], with [`stacks::tucked_pushed`] in place of the first
//! two on phones.

use std::collections::HashSet;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;
use zwipe_ui::deck_cards::stacks;

use crate::CardProxy;
use crate::meld::{MeldSource, MeldToggle};

/// The class that draws a covered card's [`CardProxy`] as its top strip only.
const STRIP_CLASS: &str = "card-proxy-strip";

/// One card in a [`CardStack`], with its copies in the deck.
#[derive(Debug, Clone, PartialEq)]
pub struct StackCard {
    /// The card.
    pub card: Card,
    /// Its copies; more than one shows a badge.
    pub qty: i32,
    /// The face it shows, `0` the front: a two-faced card shows its back after a flip.
    pub face: usize,
}

/// The stack of `cards`, top to bottom. `revealed` shows in full, the cards
/// after it moving down to make room, and floats once it has settled: a small
/// lift and a slow sway, which mark it in place of an outline.
/// A pointer over a card or focus on it calls `on_hover` and leaving calls
/// `on_leave`; a click or Enter calls `on_select`.
///
/// With no `on_hover` the stack is in tap mode, for phones, and carries the
/// `tap` class: a card's mouseenter, mouseleave, focus and blur call
/// nothing, since a webview fires them before a tap's click, and the click
/// stops at the card, so a host's tap-off handler above the stack never
/// sees it. A revealed card pushes the cards after it down in the layout
/// ([`stacks::tucked_pushed`]), so the stack grows over the page below it
/// instead of laying its cards over it. While `dealing` the cards
/// rise into their slots one after another, the top card first, each slot
/// carrying its place in the stack for the stagger.
#[component]
pub fn CardStack(
    cards: Vec<StackCard>,
    #[props(default)] revealed: Option<Uuid>,
    #[props(default)] dealing: bool,
    #[props(default)] on_hover: Option<EventHandler<Uuid>>,
    #[props(default)] on_leave: Option<EventHandler<Uuid>>,
    on_select: EventHandler<Uuid>,
) -> Element {
    let tap = on_hover.is_none();
    let at = revealed.and_then(|r| cards.iter().position(|c| c.card.scryfall_data.id == r));
    let (tucks, shifts) = if tap {
        (
            stacks::tucked_pushed(cards.len(), at),
            vec![false; cards.len()],
        )
    } else {
        (
            stacks::tucked(cards.len()),
            stacks::shifted(cards.len(), at),
        )
    };
    let full = stacks::exposed(cards.len(), at);
    // A meld half with its Meld on shows its melded card.
    let melds = try_use_context::<MeldSource>();
    // The cards whose image failed to load, which draw their text instead.
    let mut failed: Signal<HashSet<Uuid>> = use_signal(HashSet::new);
    let hover = move |id: Uuid| {
        if let Some(handler) = on_hover {
            handler.call(id);
        }
    };
    let leave = move |id: Uuid| {
        if let Some(handler) = on_leave {
            handler.call(id);
        }
    };
    let mut stack_class = String::from("card-stack");
    if tap {
        stack_class.push_str(" tap");
    }
    if dealing {
        stack_class.push_str(" dealing");
    }
    rsx! {
        div { class: "{stack_class}", role: "list",
            for (index, (((StackCard { card, qty, face }, tucked), shifted), exposed)) in cards.into_iter().zip(tucks).zip(shifts).zip(full).enumerate() {
                {
                    let id = card.scryfall_data.id;
                    let name = card.scryfall_data.name.clone();
                    let meld = MeldToggle::of(&card.scryfall_data, melds);
                    // The card a proxy draws: the melded card while Meld is on.
                    let proxy_card = meld.shown().map_or_else(|| card.clone(), |melded| Card { scryfall_data: melded.clone(), ..card.clone() });
                    let image = match meld.shown() {
                        Some(melded) => melded.shown_face_image_url(0, ImageSize::Normal),
                        None => card.scryfall_data.shown_face_image_url(face, ImageSize::Normal),
                    }
                    .map(str::to_owned);
                    let mut class = String::from("card-stack-slot");
                    if tucked {
                        class.push_str(" tucked");
                    }
                    if shifted {
                        class.push_str(" shifted");
                    }
                    if revealed == Some(id) {
                        class.push_str(" revealed");
                    }
                    let label = if qty > 1 { format!("{name}, {qty} copies") } else { name };
                    rsx! {
                        // `data-row` names the card, as a list row does, so a
                        // host can find the card under the cursor.
                        div {
                            key: "{id}",
                            role: "listitem",
                            class: "{class}",
                            "data-row": "{id}",
                            style: dealing.then(|| format!("--deal-i: {index}")),
                            button {
                                r#type: "button",
                                class: if revealed == Some(id) { "card-stack-card floatable floating" } else { "card-stack-card floatable" },
                                "aria-label": "{label}",
                                onmouseenter: move |_| hover(id),
                                onmouseleave: move |_| leave(id),
                                onfocus: move |_| hover(id),
                                onblur: move |_| leave(id),
                                onclick: move |event: MouseEvent| {
                                    if tap {
                                        event.stop_propagation();
                                    }
                                    on_select.call(id);
                                },
                                // An image that is missing or failed leaves the
                                // card's text in the card's box.
                                if let Some(src) = image.filter(|_| !failed.read().contains(&id)) {
                                    img {
                                        src: "{src}",
                                        alt: "",
                                        loading: "lazy",
                                        draggable: "false",
                                        onerror: move |_| {
                                            failed.write().insert(id);
                                        },
                                    }
                                } else {
                                    // A covered card shows only its strip: the
                                    // name and cost on one line.
                                    CardProxy {
                                        sd: proxy_card.scryfall_data,
                                        face,
                                        profile: Some(proxy_card.card_profile),
                                        class: if exposed { String::new() } else { STRIP_CLASS.to_owned() },
                                    }
                                }
                                if qty > 1 {
                                    span { class: "card-stack-qty", "\u{d7}{qty}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_ui::deck_cards::stacks::{DEAL_DURATION, DEAL_STAGGER, DEAL_STAGGER_CAP};

    /// The kit's Magic sheet, where the stacks' rules live.
    const MAGIC_CSS: &str = include_str!("../assets/magic.css");

    /// The body of the first rule whose selector reads `selector`.
    fn rule(selector: &str) -> &'static str {
        let at = MAGIC_CSS.find(&format!("{selector} {{")).unwrap();
        let body = MAGIC_CSS.get(at..).unwrap();
        body.get(body.find('{').unwrap().saturating_add(1)..body.find('}').unwrap())
            .unwrap()
    }

    #[test]
    fn a_dealing_stack_rises_in_card_by_card_on_the_shared_timing() {
        let deal = rule(".card-stack.dealing .card-stack-card");
        for part in [
            "card-stack-deal".to_owned(),
            format!("{}ms", DEAL_DURATION.as_millis()),
            "var(--ease-enter)".to_owned(),
            format!(
                "calc(min(var(--deal-i), {DEAL_STAGGER_CAP}) * {}ms)",
                DEAL_STAGGER.as_millis()
            ),
            "backwards".to_owned(),
        ] {
            assert!(deal.contains(&part), "{part} in {deal}");
        }
        let skipped = MAGIC_CSS
            .split("@media (prefers-reduced-motion: reduce)")
            .skip(1)
            .filter_map(|block| block.get(..block.find("\n}\n")?))
            .any(|block| block.contains(".card-stack.dealing .card-stack-card"));
        assert!(skipped, "reduced motion skips the deal");
    }

    #[test]
    fn nothing_about_a_stack_promotes_its_cards_to_layers() {
        // A rule's selector and its body, split at each `}`.
        for rule in MAGIC_CSS.split('}') {
            let Some((selector, body)) = rule.split_once('{') else {
                continue;
            };
            if selector.contains(".card-stack") {
                assert!(
                    !body.contains("will-change"),
                    "a stack at a fractional x is drawn a pixel off while promoted: {}",
                    selector.trim()
                );
            }
        }
    }
}

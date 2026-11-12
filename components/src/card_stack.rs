//! One group of a deck's cards as a stack: the cards overlap, each tucked
//! under the one above it so only its title strip shows, the last in full.
//! Which card tucks, which move down to reveal one, and which show in full
//! come from zwipe-ui's [`stacks::tucked`], [`stacks::shifted`] and
//! [`stacks::exposed`], so the phones' stacks do the same.

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
/// after it moving down to make room; `open` outlines the card whose details
/// are up while it shows in full.
/// A pointer over a card or focus on it calls `on_hover` and leaving calls
/// `on_leave`; a click or Enter calls `on_select`. While `dealing` the cards
/// rise into their slots one after another, the top card first, each slot
/// carrying its place in the stack for the stagger.
#[component]
pub fn CardStack(
    cards: Vec<StackCard>,
    #[props(default)] revealed: Option<Uuid>,
    #[props(default)] open: Option<Uuid>,
    #[props(default)] dealing: bool,
    on_hover: EventHandler<Uuid>,
    on_leave: EventHandler<Uuid>,
    on_select: EventHandler<Uuid>,
) -> Element {
    let at = revealed.and_then(|r| cards.iter().position(|c| c.card.scryfall_data.id == r));
    let tucks = stacks::tucked(cards.len());
    let shifts = stacks::shifted(cards.len(), at);
    let full = stacks::exposed(cards.len(), at);
    // A meld half with its Meld on shows its melded card.
    let melds = try_use_context::<MeldSource>();
    // The cards whose image failed to load, which draw their text instead.
    let mut failed: Signal<HashSet<Uuid>> = use_signal(HashSet::new);
    rsx! {
        div { class: if dealing { "card-stack dealing" } else { "card-stack" }, role: "list",
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
                    if exposed && open == Some(id) {
                        class.push_str(" open");
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
                                class: "card-stack-card",
                                "aria-label": "{label}",
                                onmouseenter: move |_| on_hover.call(id),
                                onmouseleave: move |_| on_leave.call(id),
                                onfocus: move |_| on_hover.call(id),
                                onblur: move |_| on_leave.call(id),
                                onclick: move |_| on_select.call(id),
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

//! A folding group of cards in a deck's List on a phone and narrow web: a header that folds it
//! over its rows, with the group's price and cart under the title where it has them.

use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_ui::buy::{group_cards, group_price};
use zwipe_ui::deck_cards::keys::RowInfo;

use crate::{CartButton, GroupBuy, IdentityPips, Pressable};

/// A group's header with the card row's disclosure arrow (down while expanded, sideways while
/// shut, on the same eased turn) over a body that opens and closes with the `.collapsible`
/// grid-rows ease. The rows stay mounted while shut, and inert, so an open row survives the
/// round trip.
///
/// `pips` lead the title for a color group. `trailing` (the group's [`group_tail`]) sits on the
/// line under the title; with it, only the arrow and the title fold the group, in a
/// `.card-group-toggle`, so a press on the cart never folds it.
#[component]
pub fn CardGroup(
    title: String,
    #[props(default)] pips: Option<Vec<Color>>,
    expanded: bool,
    ontoggle: EventHandler,
    #[props(default)] trailing: Option<Element>,
    children: Element,
) -> Element {
    let header_class = if expanded {
        "card-group-header group-collapsible expanded"
    } else {
        "card-group-header group-collapsible"
    };
    let heading = rsx! {
        span { class: "card-row-arrow", "\u{25b8}" }
        if let Some(pips) = pips.as_ref().filter(|p| !p.is_empty()) {
            IdentityPips { colors: pips.clone() }
        }
        "{title}"
    };
    rsx! {
        div {
            class: if expanded { "card-group row-enter" } else { "card-group row-enter collapsed" },
            if let Some(trailing) = trailing {
                div { class: "{header_class} card-group-header-tailed",
                    Pressable { class: "card-group-toggle", expanded, onpress: move |()| ontoggle(()), {heading} }
                    {trailing}
                }
            } else {
                Pressable { class: header_class, expanded, onpress: move |()| ontoggle(()), {heading} }
            }
            div {
                class: if expanded { "collapsible open" } else { "collapsible" },
                inert: (!expanded).then_some(true),
                div { class: "collapsible-inner", {children} }
            }
        }
    }
}

/// Where a [`group_tail`] sits, which sets its class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TailPlace {
    /// Under a List group's title ([`CardGroup`]), as `.card-group-tail`.
    List,
    /// At the end of a stack's header chip, as `.deck-stack-tail`.
    Stack,
}

impl TailPlace {
    /// The tail's class here.
    #[must_use]
    pub const fn class(self) -> &'static str {
        match self {
            Self::List => "card-group-tail",
            Self::Stack => "deck-stack-tail",
        }
    }
}

/// A group's total price in `currency` and its cart, which opens `group_buy` on the group's
/// cards under `title`. `None` for a group with neither, such as the tokens, so the host draws
/// its header without a tail.
#[must_use]
pub fn group_tail(
    place: TailPlace,
    title: String,
    rows: Vec<RowInfo>,
    currency: PriceCurrency,
    group_buy: GroupBuy,
) -> Option<Element> {
    let price = group_price(&rows, currency);
    let buyable = !group_cards(&rows).is_empty();
    (price.is_some() || buyable).then(|| {
        rsx! {
            span { class: place.class(),
                if let Some(p) = price {
                    span { class: "stat-chip stat-chip-money", "{p}" }
                }
                if buyable {
                    CartButton { onclick: move |_| group_buy.offer(&title, &rows) }
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A group header rendered once, with a cart in its tail or none.
    fn group_html(with_tail: bool) -> String {
        fn tailed() -> Element {
            rsx! {
                CardGroup {
                    title: "Creatures (2)".to_owned(),
                    expanded: true,
                    ontoggle: move |()| {},
                    trailing: rsx! {
                        span { class: "card-group-tail",
                            CartButton { onclick: move |_| {} }
                        }
                    },
                    "rows"
                }
            }
        }
        fn plain() -> Element {
            rsx! {
                CardGroup { title: "Creatures (2)".to_owned(), expanded: true, ontoggle: move |()| {}, "rows" }
            }
        }
        let mut dom = VirtualDom::new(if with_tail { tailed } else { plain });
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn a_group_with_a_cart_folds_only_from_its_title() {
        let html = group_html(true);
        let (header, _) = html.split_once("collapsible open").unwrap();
        let (_, toggle) = header.split_once(r#"class="card-group-toggle""#).unwrap();
        let (toggle, rest) = toggle.split_once("</div>").unwrap();
        assert!(
            toggle.contains("Creatures (2)"),
            "the title folds: {toggle}"
        );
        assert!(
            !toggle.contains("cart-button"),
            "the cart never folds: {toggle}"
        );
        assert!(
            rest.contains("cart-button"),
            "the cart follows the title: {rest}"
        );
        assert!(
            header.matches(r#"role="button""#).count() == 1,
            "only the title is the fold's button: {header}"
        );
    }

    #[test]
    fn a_group_without_a_cart_folds_from_its_whole_header() {
        let html = group_html(false);
        assert!(!html.contains("card-group-toggle"), "{html}");
        assert!(
            html.contains(r#"class="card-group-header group-collapsible expanded" role="button""#),
            "{html}"
        );
    }
}

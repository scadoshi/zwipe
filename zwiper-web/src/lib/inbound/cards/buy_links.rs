//! Buy links: one link per retailer that opens in a new tab, as zite's
//! featured flavor does, where the app opens a `Buy` dialog to pick one.
//! The URLs are zwipe-ui's.

use dioxus::prelude::*;
use zwipe_ui::buy;

/// One retailer link in a card's or deck's action row, its name and the
/// `↗` that marks it as leaving the site.
#[component]
pub fn BuyLink(href: String, label: &'static str) -> Element {
    rsx! {
        a {
            class: "card-action-btn web-buy-link",
            href,
            target: "_blank",
            rel: "noopener",
            onclick: move |evt| evt.stop_propagation(),
            "{label}"
            span { class: "web-buy-ext", "\u{2197}" }
        }
    }
}

/// `TCGplayer` and `Card Kingdom` for one card: its `TCGplayer` product page
/// when the id is known, else a one-copy entry by name.
#[component]
pub fn CardBuyLinks(name: String, tcgplayer_id: Option<i32>) -> Element {
    rsx! {
        BuyLink { href: buy::tcgplayer_card_url(&name, tcgplayer_id), label: "TCGplayer" }
        BuyLink { href: buy::cardkingdom_card_url(&name), label: "Card Kingdom" }
    }
}

/// One retailer as an item of a `More` menu: a link styled as the menu's
/// buttons, or a disabled button while there is nothing to buy. A click
/// closes the menu through `on_open`.
#[component]
pub fn MenuBuyLink(
    href: Option<String>,
    label: &'static str,
    on_open: EventHandler<()>,
) -> Element {
    match href {
        Some(href) => rsx! {
            a {
                class: "btn web-buy-link web-menu-link",
                href,
                target: "_blank",
                rel: "noopener",
                onclick: move |_| on_open.call(()),
                "{label}"
                span { class: "web-buy-ext", "\u{2197}" }
            }
        },
        None => rsx! {
            components::Button { disabled: true, onclick: move |_| {},
                "{label}"
                span { class: "web-buy-ext", "\u{2197}" }
            }
        },
    }
}

//! The hint a shared deck's page opens on `?`: what the page is and what a visitor can do there.

use super::{HintContent, KeyIcon, icon_key, line, text};

/// The shared deck page's title in its hint.
pub const TITLE: &str = "Shared deck";

/// The shared deck page's hint: someone shared the deck, a card opens its details, and the cart
/// buys the cards.
#[must_use]
pub fn content() -> HintContent {
    HintContent {
        title: TITLE.to_owned(),
        blocks: vec![
            line(vec![text("Someone shared this deck with you.")]),
            line(vec![
                text("Tap a card for its details, or "),
                icon_key(KeyIcon::Cart),
                text(" to buy the cards you want."),
            ]),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hint_names_the_cart_by_its_icon() {
        let hint = content();
        assert_eq!(hint.title, "Shared deck");
        assert_eq!(
            hint.plain(),
            "Someone shared this deck with you.\nTap a card for its details, or cart to buy the cards you want."
        );
    }
}

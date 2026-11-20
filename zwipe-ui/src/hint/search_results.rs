//! The hint the card search's results page opens on `?`: what the page lists and where a card
//! leads.

use super::{HintContent, line, text};

/// The results page's title in its hint.
pub const TITLE: &str = "Search results";

/// The results page's hint: every card the search matches, each opening its own page.
#[must_use]
pub fn content() -> HintContent {
    HintContent {
        title: TITLE.to_owned(),
        blocks: vec![
            line(vec![text("Every card that matches your search.")]),
            line(vec![text(
                "Tap one for its page, with its printings and stats.",
            )]),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hint_says_what_the_page_lists_and_where_a_card_leads() {
        let hint = content();
        assert_eq!(hint.title, "Search results");
        assert_eq!(
            hint.plain(),
            "Every card that matches your search.\nTap one for its page, with its printings and stats."
        );
    }
}

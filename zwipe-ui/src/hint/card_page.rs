//! The hint a card's page opens on `?`: the eye shows the card's details, and what the dock's
//! other buttons do.

use super::{HintContent, KeyIcon, icon_key, line, text};

/// The card page's title in its hint.
pub const TITLE: &str = "Card";

/// The card page's hint: the eye shows the card's details with its printings and stats, Send to
/// puts it in a deck where a session can (`signed_in`), and Share copies its link.
#[must_use]
pub fn content(signed_in: bool) -> HintContent {
    let send = signed_in.then(|| {
        line(vec![
            icon_key(KeyIcon::Send),
            text(" puts it in one of your decks."),
        ])
    });
    let blocks = [
        Some(line(vec![
            icon_key(KeyIcon::Eye),
            text(" shows its details, with its printings and stats."),
        ])),
        send,
        Some(line(vec![
            icon_key(KeyIcon::Share),
            text(" copies its link."),
        ])),
    ]
    .into_iter()
    .flatten()
    .collect();
    HintContent {
        title: TITLE.to_owned(),
        blocks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hint_names_the_eye_and_the_dock_by_their_icons() {
        let hint = content(true);
        assert_eq!(hint.title, "Card");
        assert_eq!(
            hint.plain(),
            "eye shows its details, with its printings and stats.\nsend puts it in one of your decks.\nshare copies its link."
        );
    }

    #[test]
    fn signed_out_there_is_no_send_to() {
        let hint = content(false);
        assert_eq!(
            hint.plain(),
            "eye shows its details, with its printings and stats.\nshare copies its link."
        );
    }
}

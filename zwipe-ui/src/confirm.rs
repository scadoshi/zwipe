//! The short word for a swipe's confirmation. The swipe screens name each
//! confirmation in full ("Added to deck"); the quiet chip and the toast both
//! say the short form ("Added"), so a stack of them stays close in width.

use crate::screens::{add_cards, deck_cards, remove_cards, swipe_select};

/// The chip's word for a confirmation that says `text` in full. A text this
/// table does not know shows as it is.
#[must_use]
pub fn chip_word(text: &str) -> &str {
    match text {
        add_cards::SKIPPED => "Skipped",
        add_cards::ADDED_TO_DECK => "Added",
        add_cards::ADDED_TO_MAYBEBOARD
        | add_cards::ALREADY_ON_MAYBEBOARD
        | remove_cards::MOVED_TO_MAYBEBOARD
        | swipe_select::SAVED => "Maybe",
        remove_cards::KEPT => "Kept",
        remove_cards::REMOVED => "Removed",
        deck_cards::MOVED_TO_MAINBOARD => "Main",
        add_cards::UNDID_SKIP
        | add_cards::UNDID_ADD
        | add_cards::UNDID_MAYBEBOARD
        | add_cards::UNDID_MOVE_TO_MAINBOARD
        | remove_cards::UNDID_KEEP
        | remove_cards::UNDID_REMOVE
        | remove_cards::UNDID_MOVE
        | swipe_select::UNSAVED => "Undid",
        picked if picked.ends_with(" selected") => "Picked",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toast::{Toast, ToastKind};

    #[test]
    fn each_confirmation_has_its_short_word() {
        for (text, word) in [
            (add_cards::SKIPPED, "Skipped"),
            (add_cards::ADDED_TO_DECK, "Added"),
            (add_cards::ADDED_TO_MAYBEBOARD, "Maybe"),
            (add_cards::ALREADY_ON_MAYBEBOARD, "Maybe"),
            (add_cards::UNDID_SKIP, "Undid"),
            (add_cards::UNDID_ADD, "Undid"),
            (add_cards::UNDID_MAYBEBOARD, "Undid"),
            (add_cards::UNDID_MOVE_TO_MAINBOARD, "Undid"),
            (remove_cards::KEPT, "Kept"),
            (remove_cards::REMOVED, "Removed"),
            (remove_cards::MOVED_TO_MAYBEBOARD, "Maybe"),
            (deck_cards::MOVED_TO_MAINBOARD, "Main"),
            (remove_cards::UNDID_KEEP, "Undid"),
            (remove_cards::UNDID_REMOVE, "Undid"),
            (remove_cards::UNDID_MOVE, "Undid"),
            (swipe_select::SAVED, "Maybe"),
            (swipe_select::UNSAVED, "Undid"),
            ("Commander selected", "Picked"),
        ] {
            assert_eq!(chip_word(text), word, "{text}");
        }
    }

    #[test]
    fn add_keeps_its_own_word_for_a_move_to_mainboard() {
        let toast = Toast::quick(ToastKind::Success, add_cards::MOVED_TO_MAINBOARD)
            .with_chip(add_cards::MOVED_TO_MAINBOARD_CHIP);
        assert_eq!(toast.chip_word(), "Added");
        let remove = Toast::quick(ToastKind::Success, remove_cards::MOVED_TO_MAINBOARD);
        assert_eq!(remove.chip_word(), "Main");
    }

    #[test]
    fn an_unknown_confirmation_shows_as_it_is() {
        assert_eq!(chip_word("Something new"), "Something new");
    }
}

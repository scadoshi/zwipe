//! The quiet chip's short word for a swipe's confirmation. The swipe screens
//! confirm in a toast's full words ("Added to deck"); the chip says the
//! short form ("Added"), so it stays quiet.

use crate::screens::{add_cards, remove_cards, swipe_select};

/// The chip's word for a confirmation that says `text` in full. A text this
/// table does not know shows as it is.
#[must_use]
pub fn chip_word(text: &str) -> &str {
    match text {
        add_cards::SKIPPED => "Skipped",
        add_cards::ADDED_TO_DECK | add_cards::MOVED_TO_DECK => "Added",
        add_cards::ADDED_TO_MAYBEBOARD
        | add_cards::ALREADY_ON_MAYBEBOARD
        | remove_cards::MOVED_TO_MAYBEBOARD
        | swipe_select::SAVED => "Maybe",
        remove_cards::KEPT => "Kept",
        remove_cards::REMOVED => "Removed",
        remove_cards::MOVED_TO_MAIN => "Main",
        add_cards::UNDID_SKIP
        | add_cards::UNDID_ADD
        | add_cards::UNDID_MAYBEBOARD
        | add_cards::UNDID_MOVE_TO_DECK
        | remove_cards::UNDID_KEEP
        | remove_cards::UNDID_REMOVE
        | remove_cards::UNDID_MOVE
        | swipe_select::UNSAVED => "Undone",
        picked if picked.ends_with(" selected") => "Picked",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_confirmation_has_its_short_word() {
        for (text, word) in [
            (add_cards::SKIPPED, "Skipped"),
            (add_cards::ADDED_TO_DECK, "Added"),
            (add_cards::ADDED_TO_MAYBEBOARD, "Maybe"),
            (add_cards::MOVED_TO_DECK, "Added"),
            (add_cards::ALREADY_ON_MAYBEBOARD, "Maybe"),
            (add_cards::UNDID_SKIP, "Undone"),
            (add_cards::UNDID_ADD, "Undone"),
            (add_cards::UNDID_MAYBEBOARD, "Undone"),
            (add_cards::UNDID_MOVE_TO_DECK, "Undone"),
            (remove_cards::KEPT, "Kept"),
            (remove_cards::REMOVED, "Removed"),
            (remove_cards::MOVED_TO_MAYBEBOARD, "Maybe"),
            (remove_cards::MOVED_TO_MAIN, "Main"),
            (remove_cards::UNDID_KEEP, "Undone"),
            (remove_cards::UNDID_REMOVE, "Undone"),
            (remove_cards::UNDID_MOVE, "Undone"),
            (swipe_select::SAVED, "Maybe"),
            (swipe_select::UNSAVED, "Undone"),
            ("Commander selected", "Picked"),
        ] {
            assert_eq!(chip_word(text), word, "{text}");
        }
    }

    #[test]
    fn an_unknown_confirmation_shows_as_it_is() {
        assert_eq!(chip_word("Something new"), "Something new");
    }
}

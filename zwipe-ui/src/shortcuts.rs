//! Keyboard shortcuts every client shares (`context/plans/keyboard_shortcuts.md`): the keys that
//! mean the same on every screen, the keys every page answers, undo, Printings' cover flow, and
//! the rows the shortcuts sheet lists for the deck cards page. A deck page that cannot be edited
//! (a shared deck) answers only the keys that read ([`Editable::No`]).

use crate::hint::KeyIcon;

/// Whether the deck page on show can be edited: the owner's own deck, or a shared deck read by
/// anyone. A read-only page answers no key that changes the deck.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Editable {
    /// The owner's deck: every key.
    Yes,
    /// A shared deck: only the keys that read.
    No,
}

/// A key that means the same on every screen: `f` opens the screen's filter where it has one, `v`
/// flips the card on show or, a card with one face, shows its meld, and `d` opens or closes the
/// card's details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardKey {
    /// Opens the screen's filter.
    Filter,
    /// Flips the card on show, or toggles its meld.
    Flip,
    /// Opens or closes the card's details.
    Details,
}

/// The [`CardKey`] `key` (a `KeyboardEvent.key`) stands for. Every screen's own key map defers to
/// this one for `f`, `v` and `d`.
#[must_use]
pub fn card_key(key: &str) -> Option<CardKey> {
    match key {
        "f" | "F" => Some(CardKey::Filter),
        "v" | "V" => Some(CardKey::Flip),
        "d" | "D" => Some(CardKey::Details),
        _ => None,
    }
}

/// How the sheets write Ctrl+Z (Cmd+Z on a Mac), which undoes wherever `u` does except while
/// typing in a field, where the field keeps its own undo.
pub const UNDO_COMBO: &str = "Ctrl+Z";

/// Whether `key` is `u`, the deck cards page's Undo, the same as its dock's Undo button.
#[must_use]
pub fn undo_letter(key: &str) -> bool {
    matches!(key, "u" | "U")
}

/// What a key does in Printings' cover flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlowKey {
    /// To the printing before.
    Back,
    /// To the printing after.
    Forward,
    /// Picks the printing in the center.
    Select,
}

/// The cover-flow action `key` stands for.
#[must_use]
pub fn flow_key(key: &str) -> Option<FlowKey> {
    match key {
        "ArrowLeft" | "h" | "H" => Some(FlowKey::Back),
        "ArrowRight" | "l" | "L" => Some(FlowKey::Forward),
        "Enter" => Some(FlowKey::Select),
        _ => None,
    }
}

/// What a key does on any page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalKey {
    /// Opens the page's own hint, as its `?` button does.
    Hint,
    /// Opens the shortcuts sheet.
    Shortcuts,
}

/// The key that opens a hint: the topmost open modal's, else the page's.
pub const HINT_KEY: &str = "?";

/// The keys that work everywhere ([`global_key`]).
pub const GLOBAL_KEYS: [&str; 2] = [HINT_KEY, "H"];

/// The global action `key` stands for: `?` the page's hint, Shift+H (`H`) the shortcuts sheet.
#[must_use]
pub fn global_key(key: &str) -> Option<GlobalKey> {
    match key {
        "?" => Some(GlobalKey::Hint),
        "H" => Some(GlobalKey::Shortcuts),
        _ => None,
    }
}

/// One row of the shortcuts sheet: its keys as the sheet writes them, and what they do.
pub type SheetRow = (&'static [&'static str], &'static str);

/// The rows every page's sheet ends with: the keys every page answers.
pub const EVERYWHERE_ROWS: [SheetRow; 3] = [
    (&["Esc"], "Close the open sheet"),
    (&["?"], "This page's hint"),
    (&["Shift+H"], "These shortcuts"),
];

/// The deck cards page's own rows, the read-only page's being only the keys that read.
#[must_use]
pub fn deck_cards_rows(editable: Editable) -> Vec<SheetRow> {
    let reads = card_read_rows();
    let edits: [SheetRow; 6] = [
        (&["m"], "Maybeboard"),
        (&["s"], "Sideboard"),
        (&["Shift+S"], STAR_ROW),
        (&["-"], "One fewer"),
        (&["+"], "One more"),
        (&["u", UNDO_COMBO], "Undo"),
    ];
    let filter: SheetRow = (&["f"], "Filter");
    let edits = match editable {
        Editable::Yes => &edits[..],
        Editable::No => &[],
    };
    reads
        .into_iter()
        .chain(edits.iter().copied())
        .chain([filter])
        .collect()
}

/// The keys that read a card under the cursor or open: Printings, Image, Flip and details. A deck
/// page lists them first; a screen of card rows with no editing (Playstyle, Activity) lists only
/// them.
#[must_use]
pub fn card_read_rows() -> [SheetRow; 4] {
    [
        (&["p"], "Printings"),
        (&["i"], IMAGE_ROW),
        (&["v"], FLIP_ROW),
        (&["d"], "Card details"),
    ]
}

/// What the `i` row does: the Image button, drawn as its icon, never the word.
const IMAGE_ROW: &str = "[[card-art]]";

/// What the `v` row does: the Flip button, drawn as its icon.
const FLIP_ROW: &str = "[[flip]]";

/// What the Shift+S row does: the Star toggle, drawn as its icon.
const STAR_ROW: &str = "[[star]]";

/// The icon a sheet row's text names as a whole, `[[name]]`, drawn as that icon's key chip in
/// place of words; `None` for a row of words.
#[must_use]
pub fn row_icon(does: &str) -> Option<KeyIcon> {
    does.strip_prefix("[[")
        .and_then(|rest| rest.strip_suffix("]]"))
        .and_then(KeyIcon::from_name)
}

/// A sheet row's text as a screen reader hears it: the button's name for an icon row, the
/// words as they are otherwise.
#[must_use]
pub fn row_label(does: &str) -> &str {
    match row_icon(does) {
        Some(KeyIcon::CardArt) => "Image",
        Some(KeyIcon::Flip) => "Flip card",
        Some(KeyIcon::Meld) => "Meld",
        Some(KeyIcon::Star) => "Star",
        _ => does,
    }
}

/// The Oracle tag examples' own rows: `←` steps to the next card, `↓` back one, and the card keys
/// that read.
#[must_use]
pub fn examples_rows() -> [SheetRow; 4] {
    [
        (&["←", "h"], "Next"),
        (&["↓", "j"], "Back one"),
        (&["d"], "Card details"),
        (&["v"], FLIP_ROW),
    ]
}

/// The line the deck cards page's sheet shows above its keys.
pub const DECK_CARDS_NOTE: &str = "Hover or open a card, then press its key";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_image_key_shows_the_image_button_s_icon_not_the_word() {
        let rows = card_read_rows();
        let image = rows
            .iter()
            .find(|(keys, _)| keys == &["i"])
            .map(|(_, does)| *does);
        assert_eq!(image, Some("[[card-art]]"));
        assert_eq!(row_icon("[[card-art]]"), Some(KeyIcon::CardArt));
        assert_eq!(row_icon("Printings"), None, "words stay words");
        assert_eq!(row_icon("[[no-such-icon]]"), None);
        assert_eq!(
            row_label("[[card-art]]"),
            "Image",
            "its name for a screen reader"
        );
        assert_eq!(row_label("Printings"), "Printings");
    }

    #[test]
    fn the_star_key_shows_the_star_toggle_s_icon() {
        let star = deck_cards_rows(Editable::Yes)
            .into_iter()
            .find(|(keys, _)| keys == &["Shift+S"])
            .map(|(_, does)| does);
        assert_eq!(star, Some("[[star]]"));
        assert_eq!(row_icon("[[star]]"), Some(KeyIcon::Star));
        assert_eq!(row_label("[[star]]"), "Star");
    }

    #[test]
    fn the_flip_key_shows_the_flip_button_s_icon() {
        let rows = card_read_rows();
        let flip = rows
            .iter()
            .find(|(keys, _)| keys == &["v"])
            .map(|(_, does)| *does);
        assert_eq!(flip, Some("[[flip]]"));
        assert_eq!(row_icon("[[flip]]"), Some(KeyIcon::Flip));
        assert_eq!(row_label("[[flip]]"), "Flip card");
        assert_eq!(row_label("[[meld]]"), "Meld");
    }

    #[test]
    fn a_read_only_deck_lists_only_the_keys_that_read() {
        let read_only: Vec<&str> = deck_cards_rows(Editable::No)
            .into_iter()
            .flat_map(|(keys, _)| keys.iter().copied())
            .collect();
        assert_eq!(read_only, ["p", "i", "v", "d", "f"]);
        let editable = deck_cards_rows(Editable::Yes);
        for key in ["m", "s", "Shift+S", "-", "+", "u", UNDO_COMBO] {
            assert!(
                editable.iter().any(|(keys, _)| keys.contains(&key)),
                "{key} on the owner's deck"
            );
            assert!(!read_only.contains(&key), "{key} on a shared deck");
        }
    }

    #[test]
    fn the_global_keys_are_exactly_the_ones_global_key_acts_on() {
        for byte in b' '..=b'~' {
            let key = char::from(byte).to_string();
            assert_eq!(
                global_key(&key).is_some(),
                GLOBAL_KEYS.contains(&key.as_str()),
                "{key}"
            );
        }
    }

    #[test]
    fn f_v_and_d_mean_the_same_everywhere() {
        assert_eq!(card_key("f"), Some(CardKey::Filter));
        assert_eq!(card_key("V"), Some(CardKey::Flip));
        assert_eq!(card_key("d"), Some(CardKey::Details));
        assert_eq!(card_key("x"), None);
    }
}

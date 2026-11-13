//! Keyboard shortcuts every client shares (`context/plans/keyboard_shortcuts.md`): the keys that
//! mean the same on every screen, the keys every page answers, undo, Printings' cover flow, and
//! the rows the shortcuts sheet lists for the deck cards page. A deck page that cannot be edited
//! (a shared deck) answers only the keys that read ([`Editable::No`]).

use orangui_core::page_keys;

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
    /// Opens the theme sheet, as the menu's palette button does.
    Themes,
}

/// The key that opens a hint: the topmost open modal's, else the page's.
pub const HINT_KEY: &str = "?";

/// The keys that work everywhere ([`global_key`]).
pub const GLOBAL_KEYS: [&str; 3] = [HINT_KEY, "H", "t"];

/// The global action `key` stands for: `?` the page's hint, Shift+H (`H`) the shortcuts sheet,
/// `t` the theme sheet (lowercase only, as Shift+T is no key).
#[must_use]
pub fn global_key(key: &str) -> Option<GlobalKey> {
    match key {
        "?" => Some(GlobalKey::Hint),
        "H" => Some(GlobalKey::Shortcuts),
        "t" => Some(GlobalKey::Themes),
        _ => None,
    }
}

/// One row of the shortcuts sheet: its keys as the sheet writes them, and what they do.
pub type SheetRow = (&'static [&'static str], &'static str);

/// A group of the shortcuts sheet: a small heading over its rows.
pub type SheetGroup = page_keys::KeyGroup;

/// The hint key's row, on a screen with a hint.
pub const HINT_ROW: SheetRow = (&["?"], "This page's hint");

/// The theme sheet's own rows, under [`page_keys::IN_THEMES`].
pub const THEME_SHEET_ROWS: [SheetRow; 4] = page_keys::THEME_SHEET_ROWS;

/// The step rows, on a screen whose own keys leave `j` and `k` to scroll the page.
pub const STEP_ROWS: [SheetRow; 2] = page_keys::STEP_ROWS;

/// One screen's keys by group, before the keys every page has join them ([`sheet_groups`]).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScreenKeys {
    /// The screen's moves, first under Navigation.
    pub moves: Vec<SheetRow>,
    /// The move that opens what the cursor is on, last under Navigation.
    pub open: Option<SheetRow>,
    /// The keys that act on a card.
    pub card: Vec<SheetRow>,
    /// A swipe screen's four directions.
    pub swipe: Vec<SheetRow>,
    /// The screen's own tools, first under Page.
    pub page: Vec<SheetRow>,
    /// Whether `j` and `k` scroll the page here (the screen's own keys don't take them).
    pub steps: bool,
}

/// The sheet's groups for a screen's `keys`, with the keys every page has: Swipe (a swipe screen's
/// directions lead), Navigation (its moves, the ends, the steps where on, then its open), Card, Page (its tools, then Themes, the
/// hint, these shortcuts and Esc) and In Themes. A group with no rows is left out.
#[must_use]
pub fn sheet_groups(keys: ScreenKeys) -> Vec<SheetGroup> {
    let steps: &[SheetRow] = if keys.steps { &STEP_ROWS } else { &[] };
    let navigation = keys
        .moves
        .into_iter()
        .chain([page_keys::TOP_ROW, page_keys::BOTTOM_ROW])
        .chain(steps.iter().copied())
        .chain(keys.open);
    let page = keys.page.into_iter().chain([
        page_keys::THEMES_ROW,
        HINT_ROW,
        page_keys::SHORTCUTS_ROW,
        page_keys::CLOSE_ROW,
    ]);
    [
        SheetGroup::new(page_keys::SWIPE, keys.swipe),
        SheetGroup::new(page_keys::NAVIGATION, navigation),
        SheetGroup::new(page_keys::CARD, keys.card),
        SheetGroup::new(page_keys::PAGE, page),
        SheetGroup::new(page_keys::IN_THEMES, THEME_SHEET_ROWS),
    ]
    .into_iter()
    .filter(|group| !group.rows.is_empty())
    .collect()
}

/// The groups of a screen with no keys of its own: the keys every page has, the steps on.
#[must_use]
pub fn everywhere_groups() -> Vec<SheetGroup> {
    sheet_groups(ScreenKeys {
        steps: true,
        ..ScreenKeys::default()
    })
}

/// `groups` with the hint key's row ([`HINT_ROW`]) taken out, for a page with no hint, where `?`
/// does nothing. A group left with no rows goes too.
#[must_use]
pub fn without_hint(groups: Vec<SheetGroup>) -> Vec<SheetGroup> {
    groups
        .into_iter()
        .map(|group| {
            let rows = group.rows.into_iter().filter(|row| *row != HINT_ROW);
            SheetGroup::new(group.title, rows)
        })
        .filter(|group| !group.rows.is_empty())
        .collect()
}

/// Every row of `groups` in order, for checks that read the sheet as one list.
#[must_use]
pub fn rows_of(groups: &[SheetGroup]) -> Vec<SheetRow> {
    groups
        .iter()
        .flat_map(|group| group.rows.iter().copied())
        .collect()
}

/// The key that puts the cursor in a page's Quick add.
pub const QUICK_ADD_KEY: &str = "/";

/// The deck cards page's own rows, the read-only page's being only the keys that read, with no
/// Quick add.
#[must_use]
pub fn deck_cards_rows(editable: Editable) -> Vec<SheetRow> {
    let reads = card_read_rows();
    let moves: [SheetRow; 3] = [
        (&["↑", "↓", "←", "→"], "Move between cards"),
        (&["h", "j", "k", "l"], "Move between cards"),
        (&["Home", "End"], "First or last card"),
    ];
    let open: SheetRow = (&["Enter"], "Open the card");
    let edits: [SheetRow; 6] = [
        (&["m"], "Maybeboard"),
        (&["s"], "Sideboard"),
        (&["Shift+S"], STAR_ROW),
        (&["-"], "One fewer"),
        (&["+"], "One more"),
        (&["u", UNDO_COMBO], "Undo"),
    ];
    let filter: SheetRow = (&["f"], "Filter");
    let quick_add: SheetRow = (&[QUICK_ADD_KEY], "Quick add");
    let (edits, quick_add) = match editable {
        Editable::Yes => (&edits[..], Some(quick_add)),
        Editable::No => (&[][..], None),
    };
    moves
        .into_iter()
        .chain([open])
        .chain(reads)
        .chain(edits.iter().copied())
        .chain([filter])
        .chain(quick_add)
        .collect()
}

/// The rows of a screen of card rows with no editing (the Commander maybeboard, Activity,
/// Playstyle): the cursor's keys, one column so only up and down, then the keys that read a card.
#[must_use]
pub fn card_rows_rows() -> Vec<SheetRow> {
    let moves: [SheetRow; 3] = [
        (&["↑", "↓"], "Move between cards"),
        (&["j", "k"], "Move between cards"),
        (&["Home", "End"], "First or last card"),
    ];
    let open: SheetRow = (&["Enter"], "Open the card");
    moves
        .into_iter()
        .chain([open])
        .chain(card_read_rows())
        .collect()
}

/// The deck cards page's groups: the cursor's moves and Enter under Navigation; the keys that
/// read a card, then (owner's page) the ones that change it, under Card; Undo, the filter and
/// Quick add (owner's page) under Page. The cursor keeps `j` and `k`.
#[must_use]
pub fn deck_cards_groups(editable: Editable) -> Vec<SheetGroup> {
    let rows = deck_cards_rows(editable);
    let find = |key: &str| rows.iter().copied().find(|row| row.0.contains(&key));
    let pick =
        |keys: &[&str]| -> Vec<SheetRow> { keys.iter().filter_map(|key| find(key)).collect() };
    sheet_groups(ScreenKeys {
        moves: pick(&["↑", "h", "Home"]),
        open: find("Enter"),
        card: pick(&["p", "i", "v", "d", "m", "s", "Shift+S", "-", "+"]),
        page: pick(&["u", "f", QUICK_ADD_KEY]),
        ..ScreenKeys::default()
    })
}

/// The Oracle tag examples' groups: its two swipes, then the keys that read a card.
#[must_use]
pub fn examples_groups() -> Vec<SheetGroup> {
    let [next, back, details, flip] = examples_rows();
    sheet_groups(ScreenKeys {
        swipe: vec![next, back],
        card: vec![details, flip],
        ..ScreenKeys::default()
    })
}

/// A screen of card rows with no editing (the Commander maybeboard, Activity, Playstyle): the
/// cursor's moves and Enter under Navigation, the keys that read a card under Card. The cursor
/// keeps `j` and `k`.
#[must_use]
pub fn card_rows_groups() -> Vec<SheetGroup> {
    let rows = card_rows_rows();
    let find = |key: &str| rows.iter().copied().find(|row| row.0.contains(&key));
    sheet_groups(ScreenKeys {
        moves: ["↑", "j", "Home"]
            .iter()
            .filter_map(|key| find(key))
            .collect(),
        open: find("Enter"),
        card: card_read_rows().to_vec(),
        ..ScreenKeys::default()
    })
}

/// The keys that read a card under the cursor or open: Printings, Image, Flip and details. A deck
/// page lists them after its cursor's keys, as a screen of card rows does
/// ([`card_rows_rows`]).
#[must_use]
pub fn card_read_rows() -> [SheetRow; 4] {
    [
        (&["p"], "Printings"),
        (&["i"], IMAGE_ROW),
        (&["v"], FLIP_ROW),
        (&["d"], "Card details"),
    ]
}

/// What the `i` row does: the Image button's icon with its name, so the sheet teaches the icon.
const IMAGE_ROW: &str = "[[image]] Image";

/// What the `v` row does: the Flip button's icon and name, and Meld's, since `v` toggles a meld
/// half's Meld where a card has one face.
pub const FLIP_ROW: &str = "[[flip]] Flip or [[meld]] meld";

/// What the Shift+S row does: the Star toggle's icon with its name.
const STAR_ROW: &str = "[[star]] Star";

/// One run of a sheet row's text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RowPart<'a> {
    /// Words, drawn as they are.
    Text(&'a str),
    /// A `[[name]]` token naming an icon button, drawn as its icon in a key chip.
    Icon(KeyIcon),
}

/// A sheet row's text in runs: each `[[name]]` an icon button's icon, the rest words. A name no
/// icon has, or an unclosed `[[`, stays words.
#[must_use]
pub fn row_parts(does: &str) -> Vec<RowPart<'_>> {
    let mut parts = Vec::new();
    let mut rest = does;
    while let Some((before, after)) = rest.split_once("[[") {
        let Some((name, tail)) = after.split_once("]]") else {
            break;
        };
        if !before.is_empty() {
            parts.push(RowPart::Text(before));
        }
        let token_len = name.len().saturating_add(4);
        let token = rest
            .get(before.len()..before.len().saturating_add(token_len))
            .unwrap_or(name);
        parts.push(KeyIcon::from_name(name).map_or(RowPart::Text(token), RowPart::Icon));
        rest = tail;
    }
    if !rest.is_empty() || parts.is_empty() {
        parts.push(RowPart::Text(rest));
    }
    parts
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_reads_its_icons_as_icons_and_its_words_as_words() {
        assert_eq!(
            row_parts("[[flip]] Flip or [[meld]] meld"),
            [
                RowPart::Icon(KeyIcon::Flip),
                RowPart::Text(" Flip or "),
                RowPart::Icon(KeyIcon::Meld),
                RowPart::Text(" meld"),
            ]
        );
        assert_eq!(row_parts("Printings"), [RowPart::Text("Printings")]);
        assert_eq!(
            row_parts("[[nope]] stays"),
            [RowPart::Text("[[nope]]"), RowPart::Text(" stays")],
            "a name no icon has stays words"
        );
        assert_eq!(
            row_parts("open [[ never closes"),
            [RowPart::Text("open [[ never closes")]
        );
    }

    #[test]
    fn an_icon_row_names_its_button_beside_the_icon() {
        let rows = card_read_rows();
        let does = |key: &str| {
            rows.iter()
                .find(|(keys, _)| keys == &[key])
                .map(|(_, does)| *does)
        };
        assert_eq!(does("i"), Some("[[image]] Image"));
        assert_eq!(does("v"), Some("[[flip]] Flip or [[meld]] meld"));
        let star = deck_cards_rows(Editable::Yes)
            .into_iter()
            .find(|(keys, _)| keys == &["Shift+S"])
            .map(|(_, does)| does);
        assert_eq!(star, Some("[[star]] Star"));
    }

    /// Each group's title and its rows' first keys.
    fn outline(groups: &[SheetGroup]) -> Vec<(&'static str, Vec<&'static str>)> {
        groups
            .iter()
            .map(|group| {
                let firsts = group
                    .rows
                    .iter()
                    .filter_map(|(keys, _)| keys.first().copied());
                (group.title, firsts.collect())
            })
            .collect()
    }

    #[test]
    fn the_owner_s_deck_page_groups_its_keys_as_approved() {
        assert_eq!(
            outline(&deck_cards_groups(Editable::Yes)),
            vec![
                (
                    "Navigation",
                    vec!["↑", "h", "Home", "gg", "Shift+G", "Enter"]
                ),
                (
                    "Card",
                    vec!["p", "i", "v", "d", "m", "s", "Shift+S", "-", "+"]
                ),
                ("Page", vec!["u", "f", "/", "t", "?", "Shift+H", "Esc"]),
                ("In Themes", vec!["↑", "h", "gg", "Enter"]),
            ]
        );
    }

    #[test]
    fn a_shared_deck_groups_only_the_keys_that_read() {
        assert_eq!(
            outline(&deck_cards_groups(Editable::No)),
            vec![
                (
                    "Navigation",
                    vec!["↑", "h", "Home", "gg", "Shift+G", "Enter"]
                ),
                ("Card", vec!["p", "i", "v", "d"]),
                ("Page", vec!["f", "t", "?", "Shift+H", "Esc"]),
                ("In Themes", vec!["↑", "h", "gg", "Enter"]),
            ]
        );
    }

    #[test]
    fn a_screen_with_no_keys_lists_the_steps_and_the_keys_every_page_has() {
        assert_eq!(
            outline(&everywhere_groups()),
            vec![
                ("Navigation", vec!["gg", "Shift+G", "j", "k"]),
                ("Page", vec!["t", "?", "Shift+H", "Esc"]),
                ("In Themes", vec!["↑", "h", "gg", "Enter"]),
            ]
        );
    }

    #[test]
    fn a_page_with_no_hint_loses_only_the_hint_key() {
        assert_eq!(
            outline(&without_hint(everywhere_groups())),
            vec![
                ("Navigation", vec!["gg", "Shift+G", "j", "k"]),
                ("Page", vec!["t", "Shift+H", "Esc"]),
                ("In Themes", vec!["↑", "h", "gg", "Enter"]),
            ]
        );
        let only_hint = vec![SheetGroup::new("Page", [HINT_ROW])];
        assert!(without_hint(only_hint).is_empty(), "an emptied group goes");
    }

    #[test]
    fn the_examples_and_card_rows_group_their_keys() {
        assert_eq!(
            outline(&examples_groups()).get(..3).map(<[_]>::to_vec),
            Some(vec![
                ("Swipe", vec!["←", "↓"]),
                ("Navigation", vec!["gg", "Shift+G"]),
                ("Card", vec!["d", "v"]),
            ])
        );
        assert_eq!(
            outline(&card_rows_groups()).get(..2).map(<[_]>::to_vec),
            Some(vec![
                (
                    "Navigation",
                    vec!["↑", "j", "Home", "gg", "Shift+G", "Enter"]
                ),
                ("Card", vec!["p", "i", "v", "d"]),
            ])
        );
    }

    #[test]
    fn every_key_the_deck_page_answers_is_listed() {
        let listed = |editable| -> Vec<&str> {
            deck_cards_rows(editable)
                .into_iter()
                .flat_map(|(keys, _)| keys.iter().copied())
                .collect()
        };
        let owner = listed(Editable::Yes);
        let shared = listed(Editable::No);
        // The cursor's moves, Home and End among them.
        for key in ["Home", "End"] {
            assert!(owner.contains(&key), "{key} on the owner's deck");
            assert!(shared.contains(&key), "{key} on a shared deck");
        }
        // `/` focuses Quick add, which only the owner's page has.
        assert!(owner.contains(&"/"), "/ on the owner's deck: {owner:?}");
        assert!(!shared.contains(&"/"), "a shared deck has no Quick add");
    }

    #[test]
    fn a_read_only_deck_lists_only_the_keys_that_read() {
        let read_only: Vec<&str> = deck_cards_rows(Editable::No)
            .into_iter()
            .flat_map(|(keys, _)| keys.iter().copied())
            .collect();
        assert_eq!(
            read_only,
            [
                "↑", "↓", "←", "→", "h", "j", "k", "l", "Home", "End", "Enter", "p", "i", "v", "d",
                "f"
            ]
        );
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

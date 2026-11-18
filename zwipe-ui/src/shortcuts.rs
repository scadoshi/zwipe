//! Keyboard shortcuts every client shares: the keys that
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

/// A key that means the same on every screen: `f` opens the screen's filter where it has one and
/// `/` opens it at its Name section with the name field focused, `v` flips the card on show or, a
/// card with one face, shows its meld, and `d` opens or closes the card's details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardKey {
    /// Opens the screen's filter.
    Filter,
    /// Opens the screen's filter at its Name section, the name field focused.
    FilterByName,
    /// Flips the card on show, or toggles its meld.
    Flip,
    /// Opens or closes the card's details.
    Details,
}

/// The [`CardKey`] `key` (a `KeyboardEvent.key`) stands for. Every screen's own key map defers to
/// this one for `f`, `/`, `v` and `d`.
#[must_use]
pub fn card_key(key: &str) -> Option<CardKey> {
    match key {
        "f" | "F" => Some(CardKey::Filter),
        FILTER_NAME_KEY => Some(CardKey::FilterByName),
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
    /// Flips or melds every printing together, as `v` does a card ([`CardKey::Flip`]).
    Flip,
}

/// The cover-flow action `key` stands for.
#[must_use]
pub fn flow_key(key: &str) -> Option<FlowKey> {
    match key {
        "ArrowLeft" | "h" | "H" => Some(FlowKey::Back),
        "ArrowRight" | "l" | "L" => Some(FlowKey::Forward),
        "Enter" => Some(FlowKey::Select),
        "v" | "V" => Some(FlowKey::Flip),
        _ => None,
    }
}

/// The heading over the Printings sheet's own keys.
pub const IN_PRINTINGS: &str = "In printings";

/// The Printings sheet's own rows ([`flow_key`]), listed under [`IN_PRINTINGS`] on every screen
/// with keys that act on a card.
pub const PRINTINGS_SHEET_ROWS: [SheetRow; 4] = [
    (&["←", "→"], "Previous or next printing"),
    (&["h", "l"], "Previous or next printing"),
    (&["v"], FLIP_ROW),
    (&["Enter"], "Pick the printing"),
];

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
/// hint, these shortcuts and Esc), In printings where the screen has keys that act on a card, and
/// In themes. A group with no rows is left out.
#[must_use]
pub fn sheet_groups(keys: ScreenKeys) -> Vec<SheetGroup> {
    let steps: &[SheetRow] = if keys.steps { &STEP_ROWS } else { &[] };
    let navigation = keys
        .moves
        .into_iter()
        .chain([page_keys::TOP_ROW, page_keys::BOTTOM_ROW])
        .chain(steps.iter().copied())
        .chain(keys.open);
    let printings: &[SheetRow] = if keys.card.is_empty() {
        &[]
    } else {
        &PRINTINGS_SHEET_ROWS
    };
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
        SheetGroup::new(IN_PRINTINGS, printings.iter().copied()),
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

/// The key that puts the cursor in a page's Quick add. Lowercase only, so Shift+Q stays free.
pub const QUICK_ADD_KEY: &str = "q";

/// The key that opens a screen's filter at its Name section with the name field focused
/// ([`CardKey::FilterByName`]).
pub const FILTER_NAME_KEY: &str = "/";

/// The sheet's row for [`FILTER_NAME_KEY`], on every screen with a filter.
pub const FILTER_NAME_ROW: SheetRow = (&[FILTER_NAME_KEY], "Filter by name");

/// A deck action the owner's deck page opens from the keyboard, as its dock button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeckAction {
    /// `a`: Add cards.
    AddCards,
    /// `r`: Remove cards.
    RemoveCards,
    /// `e`: Edit deck.
    EditDeck,
}

/// The deck action `key` (`KeyboardEvent.key`) opens on the owner's deck page: `a` Add cards,
/// `r` Remove cards, `e` Edit deck. Lowercase only, so Shift+A and the rest stay free.
#[must_use]
pub fn deck_action_key(key: &str) -> Option<DeckAction> {
    match key {
        "a" => Some(DeckAction::AddCards),
        "r" => Some(DeckAction::RemoveCards),
        "e" => Some(DeckAction::EditDeck),
        _ => None,
    }
}

/// The sheet's rows for [`deck_action_key`]'s keys, in its order.
const DECK_ACTION_ROWS: [SheetRow; 3] = [
    (&["a"], "Add cards"),
    (&["r"], "Remove cards"),
    (&["e"], "Edit deck"),
];

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
    let filter: [SheetRow; 2] = [(&["f"], "Filter"), FILTER_NAME_ROW];
    let quick_add: SheetRow = (&[QUICK_ADD_KEY], "Quick add");
    let (edits, quick_add, actions) = match editable {
        Editable::Yes => (&edits[..], Some(quick_add), &DECK_ACTION_ROWS[..]),
        Editable::No => (&[][..], None, &[][..]),
    };
    moves
        .into_iter()
        .chain([open])
        .chain(reads)
        .chain(edits.iter().copied())
        .chain(filter)
        .chain(quick_add)
        .chain(actions.iter().copied())
        .collect()
}

/// How a screen of card rows lays out its lists, which decides the cursor keys its sheet lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListLayout {
    /// One list (the Commander maybeboard, Activity): the cursor moves up and down.
    One,
    /// Two lists side by side (Playstyle's Suggested commanders and Staples): left and right
    /// cross between them, as the deck page's columns do.
    SideBySide,
}

/// The rows of a screen of card rows with no editing (the Commander maybeboard, Activity,
/// Playstyle): the cursor's keys, up and down for one list and the deck page's four ways for
/// lists side by side, then the keys that read a card.
#[must_use]
pub fn card_rows_rows(layout: ListLayout) -> Vec<SheetRow> {
    let moves: [SheetRow; 3] = match layout {
        ListLayout::One => [
            (&["↑", "↓"], "Move between cards"),
            (&["j", "k"], "Move between cards"),
            (&["Home", "End"], "First or last card"),
        ],
        ListLayout::SideBySide => [
            (&["↑", "↓", "←", "→"], "Move between cards"),
            (&["h", "j", "k", "l"], "Move between cards"),
            (&["Home", "End"], "First or last card"),
        ],
    };
    let open: SheetRow = (&["Enter"], "Open the card");
    moves
        .into_iter()
        .chain([open])
        .chain(card_read_rows())
        .collect()
}

/// The deck page's way up from a column's top card into the featured cards in its head, and back
/// down into the cards ([`deck_cards::cursor::step_with_head`](crate::deck_cards::cursor::step_with_head)).
pub const FEATURED_ROW: SheetRow = (&["↑", "k"], "Up into the featured cards");

/// The deck cards page's groups: the cursor's moves, the way up into the featured cards, and Enter under Navigation; the keys that
/// read a card, then (owner's page) the ones that change it, under Card; Undo, the filter, then
/// (owner's page) Quick add and Add cards, Remove cards and Edit deck under Page. The cursor keeps
/// `j` and `k`.
#[must_use]
pub fn deck_cards_groups(editable: Editable) -> Vec<SheetGroup> {
    let rows = deck_cards_rows(editable);
    let find = |key: &str| rows.iter().copied().find(|row| row.0.contains(&key));
    let pick =
        |keys: &[&str]| -> Vec<SheetRow> { keys.iter().filter_map(|key| find(key)).collect() };
    sheet_groups(ScreenKeys {
        moves: pick(&["↑", "h", "Home"])
            .into_iter()
            .chain([FEATURED_ROW])
            .collect(),
        open: find("Enter"),
        card: pick(&["p", "i", "v", "d", "m", "s", "Shift+S", "-", "+"]),
        page: pick(&["u", "f", FILTER_NAME_KEY, QUICK_ADD_KEY, "a", "r", "e"]),
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

/// A screen of card rows with no editing (the Commander maybeboard, Activity, Playstyle), its
/// lists laid out as `layout`: the cursor's moves and Enter under Navigation, the keys that read
/// a card under Card. The cursor keeps `j` and `k`.
#[must_use]
pub fn card_rows_groups(layout: ListLayout) -> Vec<SheetGroup> {
    let rows = card_rows_rows(layout);
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
                    vec!["↑", "h", "Home", "↑", "gg", "Shift+G", "Enter"]
                ),
                (
                    "Card",
                    vec!["p", "i", "v", "d", "m", "s", "Shift+S", "-", "+"]
                ),
                (
                    "Page",
                    vec![
                        "u", "f", "/", "q", "a", "r", "e", "t", "?", "Shift+H", "Esc"
                    ]
                ),
                ("In printings", vec!["←", "h", "v", "Enter"]),
                ("In themes", vec!["↑", "h", "gg", "Enter"]),
            ]
        );
    }

    #[test]
    fn the_owner_s_deck_page_opens_add_remove_and_edit_with_a_r_and_e() {
        assert_eq!(deck_action_key("a"), Some(DeckAction::AddCards));
        assert_eq!(deck_action_key("r"), Some(DeckAction::RemoveCards));
        assert_eq!(deck_action_key("e"), Some(DeckAction::EditDeck));
        for key in ["A", "R", "E", "b", "c", "x", "/", "Enter"] {
            assert_eq!(deck_action_key(key), None, "{key}");
        }
        let rows = rows_of(&deck_cards_groups(Editable::Yes));
        for (key, does) in [
            ("a", "Add cards"),
            ("r", "Remove cards"),
            ("e", "Edit deck"),
        ] {
            assert!(rows.contains(&(&[key][..], does)), "{key}: {does}");
        }
    }

    #[test]
    fn a_shared_deck_groups_only_the_keys_that_read() {
        assert_eq!(
            outline(&deck_cards_groups(Editable::No)),
            vec![
                (
                    "Navigation",
                    vec!["↑", "h", "Home", "↑", "gg", "Shift+G", "Enter"]
                ),
                ("Card", vec!["p", "i", "v", "d"]),
                ("Page", vec!["f", "/", "t", "?", "Shift+H", "Esc"]),
                ("In printings", vec!["←", "h", "v", "Enter"]),
                ("In themes", vec!["↑", "h", "gg", "Enter"]),
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
                ("In themes", vec!["↑", "h", "gg", "Enter"]),
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
                ("In themes", vec!["↑", "h", "gg", "Enter"]),
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
            outline(&card_rows_groups(ListLayout::One))
                .get(..2)
                .map(<[_]>::to_vec),
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
    fn lists_side_by_side_list_left_and_right_as_the_deck_page_does() {
        let moves =
            |layout| -> Vec<SheetRow> { card_rows_rows(layout).into_iter().take(3).collect() };
        assert_eq!(
            moves(ListLayout::SideBySide),
            deck_cards_rows(Editable::No)
                .into_iter()
                .take(3)
                .collect::<Vec<_>>(),
            "Playstyle's two lists move as the deck page's columns do"
        );
        assert_eq!(
            moves(ListLayout::One).first().map(|row| row.0),
            Some(&["↑", "↓"][..]),
            "one list moves only up and down"
        );
        assert_eq!(
            outline(&card_rows_groups(ListLayout::SideBySide))
                .first()
                .cloned(),
            Some((
                "Navigation",
                vec!["↑", "h", "Home", "gg", "Shift+G", "Enter"]
            ))
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
        // `q` focuses Quick add, which only the owner's page has; `/` filters on both.
        assert!(owner.contains(&"q"), "q on the owner's deck: {owner:?}");
        assert!(!shared.contains(&"q"), "a shared deck has no Quick add");
        assert!(owner.contains(&"/"), "/ on the owner's deck: {owner:?}");
        assert!(shared.contains(&"/"), "/ on a shared deck: {shared:?}");
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
                "f", "/"
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
    fn every_screen_with_card_keys_lists_the_printings_sheet_s_keys_and_v_among_them() {
        let screens = [
            deck_cards_groups(Editable::Yes),
            deck_cards_groups(Editable::No),
            card_rows_groups(ListLayout::One),
            card_rows_groups(ListLayout::SideBySide),
            examples_groups(),
        ];
        for groups in screens {
            let printings = groups
                .iter()
                .find(|group| group.title == IN_PRINTINGS)
                .map(|group| group.rows.clone());
            assert_eq!(printings, Some(PRINTINGS_SHEET_ROWS.to_vec()), "{groups:?}");
        }
        assert!(
            PRINTINGS_SHEET_ROWS.contains(&(&["v"][..], FLIP_ROW)),
            "v flips or melds the printings"
        );
        assert!(
            !everywhere_groups().iter().any(|g| g.title == IN_PRINTINGS),
            "a screen with no cards has no Printings sheet"
        );
    }

    #[test]
    fn v_flips_or_melds_the_printings_as_it_does_a_card() {
        for key in ["v", "V"] {
            assert_eq!(flow_key(key), Some(FlowKey::Flip), "{key}");
            assert_eq!(card_key(key), Some(CardKey::Flip), "{key}");
        }
        assert_eq!(flow_key("ArrowLeft"), Some(FlowKey::Back));
        assert_eq!(flow_key("Enter"), Some(FlowKey::Select));
    }

    #[test]
    fn the_deck_page_lists_the_way_up_into_its_featured_cards() {
        for editable in [Editable::Yes, Editable::No] {
            let groups = deck_cards_groups(editable);
            let navigation = groups
                .iter()
                .find(|group| group.title == page_keys::NAVIGATION)
                .map_or_else(Vec::new, |group| group.rows.clone());
            assert!(navigation.contains(&FEATURED_ROW), "{navigation:?}");
        }
        assert!(
            !rows_of(&card_rows_groups(ListLayout::SideBySide)).contains(&FEATURED_ROW),
            "screens of card rows have no featured cards"
        );
    }

    #[test]
    fn f_v_and_d_mean_the_same_everywhere() {
        assert_eq!(card_key("f"), Some(CardKey::Filter));
        assert_eq!(card_key("V"), Some(CardKey::Flip));
        assert_eq!(card_key("d"), Some(CardKey::Details));
        assert_eq!(card_key("x"), None);
    }

    #[test]
    fn slash_opens_the_filter_at_its_name_and_q_opens_quick_add() {
        assert_eq!(card_key("/"), Some(CardKey::FilterByName));
        for key in ["q", "Q"] {
            assert_eq!(
                card_key(key),
                None,
                "{key} is Quick add's, not the filter's"
            );
        }
        let owner = rows_of(&deck_cards_groups(Editable::Yes));
        assert!(owner.contains(&(&["q"][..], "Quick add")), "{owner:?}");
        assert!(owner.contains(&(&["/"][..], "Filter by name")), "{owner:?}");
        let shared = rows_of(&deck_cards_groups(Editable::No));
        assert!(
            shared.contains(&(&["/"][..], "Filter by name")),
            "{shared:?}"
        );
        assert!(
            !shared.iter().any(|(_, does)| *does == "Quick add"),
            "a shared deck has no Quick add: {shared:?}"
        );
    }
}

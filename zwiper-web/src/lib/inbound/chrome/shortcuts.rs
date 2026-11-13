//! The web's keyboard shortcuts (`context/plans/keyboard_shortcuts.md`): which key does what on
//! the swipe screens, in Printings and everywhere, and the rows the shortcuts sheet lists on each
//! screen that offers it. When a key reaches a screen at all is `components::keys`' guard,
//! re-exported here for `swipe_stack` and the deck screens.

pub use components::keys::{
    KEYED_MODAL, MODAL_HINT_BUTTON, MODAL_SELECTORS, OVERLAY_SELECTORS, STACK_DETAILS_CLASS,
    TYPING_TAGS, guard_js, guard_js_through, guard_js_with_modifiers, hint_guard_js, modal_hint_js,
    press_first_js,
};
pub use components::swipe_stack::SWIPE_LETTERS;
use components::swipe_stack::{Swipe, swipe_key};
pub use zwipe_ui::shortcuts::{
    CardKey, FlowKey, GLOBAL_KEYS, GlobalKey, HINT_KEY, QUICK_ADD_KEY, UNDO_COMBO, card_key,
    flow_key, global_key, undo_letter,
};
use zwipe_ui::shortcuts::{
    DECK_CARDS_NOTE, EVERYWHERE_ROWS, Editable, FLIP_ROW, SheetRow, card_rows_rows,
    deck_cards_rows, examples_rows,
};

/// What a key does on a swipe screen (Add, Remove, Commander swipe): the same action as its
/// swipe or dock button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortcut {
    /// A swipe in this direction: right adds, left skips, up maybeboards, down undoes.
    Swipe(Swipe),
    /// Opens Printings for the card on top.
    Printings,
    /// Shows or hides the card details.
    Details,
    /// Flips the card on top to its other face, for a card with two.
    Flip,
    /// Opens the filter.
    Filter,
    /// Deals the pile again.
    Refresh,
    /// Turns Synergy on or off.
    Synergy,
}

/// The shortcut `key` (a `KeyboardEvent.key`) stands for on a swipe screen. Letters answer in
/// either case, so Caps Lock doesn't turn them off, except `H`, which opens the shortcuts sheet.
#[must_use]
pub fn swipe_screen_shortcut(key: &str) -> Option<Shortcut> {
    match card_key(key) {
        Some(CardKey::Filter) => return Some(Shortcut::Filter),
        Some(CardKey::Flip) => return Some(Shortcut::Flip),
        Some(CardKey::Details) => return Some(Shortcut::Details),
        None => {}
    }
    if let Some(swipe) = swipe_key(key) {
        return Some(Shortcut::Swipe(swipe));
    }
    let shortcut = match key {
        "p" | "P" => Shortcut::Printings,
        "r" | "R" => Shortcut::Refresh,
        "s" | "S" => Shortcut::Synergy,
        _ => return None,
    };
    Some(shortcut)
}

/// What one swipe screen calls its four swipes, as its dock buttons say them, and whether it has
/// Synergy. `up` is `None` on a screen with no up swipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwipeLabels {
    /// The right swipe.
    pub right: &'static str,
    /// The left swipe.
    pub left: &'static str,
    /// The up swipe, if the screen has one.
    pub up: Option<&'static str>,
    /// The down swipe.
    pub down: &'static str,
    /// Whether the screen has Synergy, which `s` turns on or off.
    pub synergy: bool,
}

impl SwipeLabels {
    /// The labels without the up swipe, for a screen that offers none.
    #[must_use]
    pub const fn without_up(self) -> Self {
        Self { up: None, ..self }
    }

    /// The screen's swipe dock: left, up, down, right, each with its label.
    #[must_use]
    pub fn buttons(self) -> Vec<(Swipe, &'static str)> {
        [
            Some((Swipe::Left, self.left)),
            self.up.map(|up| (Swipe::Up, up)),
            Some((Swipe::Down, self.down)),
            Some((Swipe::Right, self.right)),
        ]
        .into_iter()
        .flatten()
        .collect()
    }
}

/// Add's swipes.
pub const ADD_LABELS: SwipeLabels = SwipeLabels {
    right: "Add",
    left: "Skip",
    up: Some("Maybeboard"),
    down: "Undo",
    synergy: true,
};

/// Remove's swipes.
pub const REMOVE_LABELS: SwipeLabels = SwipeLabels {
    right: "Remove",
    left: "Keep",
    up: Some("Move"),
    down: "Undo",
    synergy: false,
};

/// Commander swipe's swipes; a mode that saves nothing for later has no up swipe.
pub const SELECT_LABELS: SwipeLabels = SwipeLabels {
    right: "Pick",
    left: "Skip",
    up: Some("Save"),
    down: "Undo",
    synergy: false,
};

/// A screen that offers the shortcuts sheet, which decides the keys the sheet lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutsScreen {
    /// A swipe screen, its swipes named as its dock names them.
    Swipe(SwipeLabels),
    /// The deck cards screen, whose keys act on the card under the cursor, else the open one.
    DeckCards,
    /// An Oracle tag's example cards, swiped through in their modal.
    Examples,
    /// A screen of card rows with no editing (Playstyle, Activity), whose keys act on the card
    /// under the cursor, else the open one.
    CardRows,
}

/// The rows the shortcuts sheet lists on `screen`: each key, with its vim letter if it has one,
/// and what it does, the screen's own keys first and then the keys every screen answers.
#[must_use]
pub fn sheet_rows(screen: &ShortcutsScreen) -> Vec<SheetRow> {
    let own: Vec<SheetRow> = match screen {
        ShortcutsScreen::Swipe(labels) => {
            let swipes = [
                Some((&["→", "l"][..], labels.right)),
                Some((&["←", "h"][..], labels.left)),
                labels.up.map(|up| (&["↑", "k"][..], up)),
                Some((&["↓", "j", "u", UNDO_COMBO][..], labels.down)),
            ];
            let rest: [(&'static [&'static str], &'static str); 5] = [
                (&["p"], "Printings"),
                (&["d"], "Card details"),
                (&["v"], FLIP_ROW),
                (&["f"], "Filter"),
                (&["r"], "Refresh"),
            ];
            let synergy = labels.synergy.then_some((&["s"][..], "Synergy on or off"));
            swipes
                .into_iter()
                .flatten()
                .chain(rest)
                .chain(synergy)
                .collect()
        }
        ShortcutsScreen::DeckCards => deck_cards_rows(Editable::Yes),
        ShortcutsScreen::Examples => examples_rows().to_vec(),
        ShortcutsScreen::CardRows => card_rows_rows(),
    };
    own.into_iter().chain(EVERYWHERE_ROWS).collect()
}

/// Whether `screen`'s own keys use `j` and `k` (the swipes, the deck page's and the card rows'
/// cursor, the examples' down swipe), so they don't scroll the page there.
#[must_use]
pub fn claims_steps(screen: &ShortcutsScreen) -> bool {
    match screen {
        ShortcutsScreen::Swipe(_)
        | ShortcutsScreen::DeckCards
        | ShortcutsScreen::Examples
        | ShortcutsScreen::CardRows => true,
    }
}

/// The line the shortcuts sheet shows above the keys on `screen`, if it needs one.
#[must_use]
pub fn sheet_note(screen: &ShortcutsScreen) -> Option<&'static str> {
    match screen {
        ShortcutsScreen::Swipe(_) | ShortcutsScreen::Examples => None,
        ShortcutsScreen::DeckCards | ShortcutsScreen::CardRows => Some(DECK_CARDS_NOTE),
    }
}

#[cfg(test)]
mod tests {
    use zwipe_ui::deck_cards::{cursor, keys as hover_keys};
    use zwipe_ui::shortcuts::THEME_ROWS;

    use super::*;

    #[test]
    fn arrows_and_vim_letters_swipe() {
        for (keys, swipe) in [
            (&["ArrowRight", "l", "L"][..], Swipe::Right),
            (&["ArrowLeft", "h"][..], Swipe::Left),
            (&["ArrowUp", "k", "K"][..], Swipe::Up),
            (&["ArrowDown", "j", "J"][..], Swipe::Down),
        ] {
            for key in keys {
                assert_eq!(
                    swipe_screen_shortcut(key),
                    Some(Shortcut::Swipe(swipe)),
                    "{key}"
                );
            }
        }
    }

    #[test]
    fn letters_open_the_dock_actions() {
        for (key, shortcut) in [
            ("p", Shortcut::Printings),
            ("d", Shortcut::Details),
            ("v", Shortcut::Flip),
            ("V", Shortcut::Flip),
            ("f", Shortcut::Filter),
            ("r", Shortcut::Refresh),
            ("s", Shortcut::Synergy),
            ("P", Shortcut::Printings),
            ("S", Shortcut::Synergy),
        ] {
            assert_eq!(swipe_screen_shortcut(key), Some(shortcut), "{key}");
        }
    }

    #[test]
    fn other_keys_do_nothing_on_a_swipe_screen() {
        for key in ["a", "x", "Enter", " ", "Escape", "?", "/", "Tab", "1"] {
            assert_eq!(swipe_screen_shortcut(key), None, "{key}");
        }
    }

    #[test]
    fn the_cover_flow_moves_and_selects() {
        assert_eq!(flow_key("ArrowLeft"), Some(FlowKey::Back));
        assert_eq!(flow_key("h"), Some(FlowKey::Back));
        assert_eq!(flow_key("ArrowRight"), Some(FlowKey::Forward));
        assert_eq!(flow_key("l"), Some(FlowKey::Forward));
        assert_eq!(flow_key("Enter"), Some(FlowKey::Select));
        assert_eq!(flow_key("j"), None);
        assert_eq!(
            flow_key("Escape"),
            None,
            "Esc closes through the sheet itself"
        );
    }

    #[test]
    fn shift_h_opens_the_sheet_and_question_mark_the_hint() {
        assert_eq!(global_key("?"), Some(GlobalKey::Hint));
        assert_eq!(global_key("H"), Some(GlobalKey::Shortcuts));
        assert_eq!(global_key("t"), Some(GlobalKey::Themes));
        assert_eq!(global_key("T"), None, "lowercase t only");
        assert_eq!(global_key("h"), None, "lowercase h stays Skip");
        assert_eq!(
            swipe_screen_shortcut("H"),
            None,
            "Shift+H is the sheet's, not Skip"
        );
        assert_eq!(
            swipe_screen_shortcut("h"),
            Some(Shortcut::Swipe(Swipe::Left))
        );
        for key in ["a", "Escape", "/", "p"] {
            assert_eq!(global_key(key), None, "{key}");
        }
    }

    #[test]
    fn the_sheet_names_each_swipe_as_its_screen_s_dock_does() {
        let names = |labels: &SwipeLabels| -> Vec<&'static str> {
            sheet_rows(&ShortcutsScreen::Swipe(*labels))
                .iter()
                .take_while(|(keys, _)| keys.first().is_some_and(|k| "→←↑↓".contains(k)))
                .map(|(_, does)| *does)
                .collect()
        };
        assert_eq!(names(&ADD_LABELS), ["Add", "Skip", "Maybeboard", "Undo"]);
        assert_eq!(names(&REMOVE_LABELS), ["Remove", "Keep", "Move", "Undo"]);
        assert_eq!(names(&SELECT_LABELS), ["Pick", "Skip", "Save", "Undo"]);
        assert_eq!(
            names(&SELECT_LABELS.without_up()),
            ["Pick", "Skip", "Undo"],
            "a screen with no up swipe lists no up key"
        );
    }

    #[test]
    fn a_screen_s_dock_buttons_come_from_its_labels() {
        assert_eq!(
            REMOVE_LABELS.buttons(),
            [
                (Swipe::Left, "Keep"),
                (Swipe::Up, "Move"),
                (Swipe::Down, "Undo"),
                (Swipe::Right, "Remove"),
            ]
        );
        assert_eq!(
            SELECT_LABELS.without_up().buttons(),
            [
                (Swipe::Left, "Skip"),
                (Swipe::Down, "Undo"),
                (Swipe::Right, "Pick"),
            ]
        );
    }

    #[test]
    fn only_a_screen_with_synergy_lists_s() {
        let lists_synergy = |labels: SwipeLabels| {
            sheet_rows(&ShortcutsScreen::Swipe(labels))
                .iter()
                .any(|(keys, _)| *keys == ["s"])
        };
        assert!(lists_synergy(ADD_LABELS));
        assert!(!lists_synergy(REMOVE_LABELS), "Remove has no Synergy");
        assert!(
            !lists_synergy(SELECT_LABELS),
            "Commander swipe has no Synergy"
        );
        assert!(!lists_synergy(SELECT_LABELS.without_up()));
    }

    #[test]
    fn u_undoes_on_every_screen_that_has_undo() {
        for key in ["u", "U"] {
            assert_eq!(
                swipe_screen_shortcut(key),
                Some(Shortcut::Swipe(Swipe::Down)),
                "{key} on a swipe screen is the down swipe, Undo"
            );
            assert!(undo_letter(key), "{key} on the deck cards screen");
        }
        for key in ["j", "z", "Z", "y"] {
            assert!(!undo_letter(key), "{key}");
        }
    }

    #[test]
    fn the_deck_cards_sheet_lists_only_that_screen_s_keys() {
        let rows = sheet_rows(&ShortcutsScreen::DeckCards);
        assert_eq!(
            sheet_note(&ShortcutsScreen::DeckCards),
            Some("Move to a card, or hover it, then press its key")
        );
        assert_eq!(sheet_note(&ShortcutsScreen::Swipe(ADD_LABELS)), None);
        // Every listed key really does what its row says on the deck cards screen.
        for (keys, does) in &rows {
            for label in *keys {
                let key = match *label {
                    "Shift+H" => "H",
                    "Shift+S" => "S",
                    "↑" => "ArrowUp",
                    "↓" => "ArrowDown",
                    "←" => "ArrowLeft",
                    "→" => "ArrowRight",
                    other => other,
                };
                let known = cursor::move_for(key).is_some()
                    || page_key_label(label)
                    || key == "Enter"
                    || hover_keys::hover_key(key, Editable::Yes).is_some()
                    || card_key(key).is_some()
                    || undo_letter(key)
                    || global_key(key).is_some()
                    || key == "Esc"
                    || key == UNDO_COMBO
                    || key == QUICK_ADD_KEY;
                assert!(known, "{label} for {does}");
            }
        }
    }

    #[test]
    fn the_sheet_lists_every_key() {
        let rows = sheet_rows(&ShortcutsScreen::Swipe(ADD_LABELS));
        let actions: Vec<&str> = rows.iter().map(|(_, does)| *does).collect();
        assert_eq!(
            actions,
            [
                "Add",
                "Skip",
                "Maybeboard",
                "Undo",
                "Printings",
                "Card details",
                FLIP_ROW,
                "Filter",
                "Refresh",
                "Synergy on or off",
                "Close the open sheet",
                "This page's hint",
                "These shortcuts",
                "To the top",
                "To the bottom",
                "Themes",
                "Move between themes in Themes",
                "Move between themes in Themes",
                "Pick the theme in Themes",
            ]
        );
        assert_eq!(rows.first().map(|(keys, _)| *keys), Some(&["→", "l"][..]));
        assert_eq!(
            rows.iter()
                .find(|(_, does)| *does == "Undo")
                .map(|(keys, _)| *keys),
            Some(&["↓", "j", "u", UNDO_COMBO][..]),
            "u and Ctrl+Z undo beside the down swipe"
        );
        // Every listed key really does what its row says; the theme sheet's own rows are checked
        // in `every_theme_sheet_key_works_there`.
        let typed = |label: &str| match label {
            "→" => "ArrowRight".to_owned(),
            "←" => "ArrowLeft".to_owned(),
            "↑" => "ArrowUp".to_owned(),
            "↓" => "ArrowDown".to_owned(),
            "Shift+H" => "H".to_owned(),
            other => other.to_owned(),
        };
        for (keys, does) in rows.iter().filter(|row| !in_theme_sheet(row)) {
            for label in *keys {
                let key = typed(label);
                let known = swipe_screen_shortcut(&key).is_some()
                    || page_key_label(label)
                    || global_key(&key).is_some()
                    || key == "Esc"
                    || key == UNDO_COMBO;
                assert!(known, "{label} for {does}");
            }
        }
    }

    /// Whether `label` is one of orangui's page keys the root answers on every screen, as a sheet
    /// writes them: `gg` and Shift+G (orangui's tests hold what they do).
    fn page_key_label(label: &str) -> bool {
        matches!(label, "gg" | "Shift+G")
    }

    /// Whether `row` is one of the theme sheet's own keys, which work in that sheet rather than on
    /// the screen; its `t` row opens it from the screen.
    fn in_theme_sheet(row: &SheetRow) -> bool {
        THEME_ROWS.contains(row) && global_key(row.0.first().copied().unwrap_or_default()).is_none()
    }

    #[test]
    fn a_screen_whose_keys_use_j_and_k_keeps_them_and_the_rest_scroll() {
        assert!(claims_steps(&ShortcutsScreen::Swipe(ADD_LABELS)));
        assert!(claims_steps(&ShortcutsScreen::DeckCards));
        assert!(claims_steps(&ShortcutsScreen::Examples));
        assert!(
            claims_steps(&ShortcutsScreen::CardRows),
            "the card-row screens' cursor moves on j and k"
        );
    }

    #[test]
    fn the_card_row_screens_list_their_cursor_keys_then_the_card_keys() {
        let rows = sheet_rows(&ShortcutsScreen::CardRows);
        let actions: Vec<&str> = rows.iter().take(7).map(|(_, does)| *does).collect();
        assert_eq!(
            actions,
            [
                "Move between cards",
                "Move between cards",
                "First or last card",
                "Open the card",
                "Printings",
                zwipe_ui::shortcuts::card_read_rows()[1].1,
                FLIP_ROW,
            ]
        );
        assert_eq!(rows.first().map(|(keys, _)| *keys), Some(&["↑", "↓"][..]));
    }

    #[test]
    fn every_theme_sheet_key_works_there() {
        for (keys, does) in THEME_ROWS {
            for label in keys {
                let key = match *label {
                    "↑" => "ArrowUp",
                    "↓" => "ArrowDown",
                    "←" => "ArrowLeft",
                    "→" => "ArrowRight",
                    other => other,
                };
                let works = cursor::move_for(key).is_some()
                    || key == "Enter"
                    || global_key(key) == Some(GlobalKey::Themes);
                assert!(works, "{label} for {does}");
            }
        }
        for screen in [
            ShortcutsScreen::Swipe(ADD_LABELS),
            ShortcutsScreen::DeckCards,
            ShortcutsScreen::Examples,
            ShortcutsScreen::CardRows,
        ] {
            let rows = sheet_rows(&screen);
            assert!(
                THEME_ROWS.iter().all(|row| rows.contains(row)),
                "{screen:?} lists the theme sheet's keys"
            );
        }
    }

    /// Every key a keyboard types without a modifier, as `KeyboardEvent.key` names it.
    fn printable_keys() -> impl Iterator<Item = String> {
        (b' '..=b'~').map(|byte| char::from(byte).to_string())
    }

    #[test]
    fn the_global_keys_are_exactly_the_ones_global_key_acts_on() {
        for key in printable_keys() {
            assert_eq!(
                global_key(&key).is_some(),
                GLOBAL_KEYS.contains(&key.as_str()),
                "{key}"
            );
        }
    }

    #[test]
    fn f_filters_v_flips_and_d_shows_details_on_every_swipe_screen() {
        for key in printable_keys() {
            let shortcut = swipe_screen_shortcut(&key);
            assert_eq!(
                shortcut == Some(Shortcut::Filter),
                card_key(&key) == Some(CardKey::Filter),
                "{key}"
            );
            assert_eq!(
                shortcut == Some(Shortcut::Flip),
                card_key(&key) == Some(CardKey::Flip),
                "{key}"
            );
            assert_eq!(
                shortcut == Some(Shortcut::Details),
                card_key(&key) == Some(CardKey::Details),
                "{key}"
            );
        }
        assert_eq!(card_key("f"), Some(CardKey::Filter));
        assert_eq!(card_key("v"), Some(CardKey::Flip));
        assert_eq!(card_key("d"), Some(CardKey::Details));
    }

    #[test]
    fn every_key_the_examples_sheet_lists_works_there() {
        let rows = sheet_rows(&ShortcutsScreen::Examples);
        for (keys, does) in rows.iter().filter(|row| !in_theme_sheet(row)) {
            for label in *keys {
                let key = match *label {
                    "←" => "ArrowLeft",
                    "↓" => "ArrowDown",
                    "Shift+H" => "H",
                    other => other,
                };
                let works = match swipe_screen_shortcut(key) {
                    Some(Shortcut::Swipe(swipe)) => [Swipe::Left, Swipe::Down].contains(&swipe),
                    _ => card_key(key).is_some_and(|k| k != CardKey::Filter),
                } || global_key(key).is_some()
                    || page_key_label(label)
                    || key == "Esc";
                assert!(works, "{label} for {does}");
            }
        }
    }

    #[test]
    fn the_swipe_letters_are_exactly_the_letters_that_swipe() {
        for key in printable_keys() {
            let swipes = matches!(swipe_screen_shortcut(&key), Some(Shortcut::Swipe(_)));
            assert_eq!(swipes, SWIPE_LETTERS.contains(&key.as_str()), "{key}");
        }
    }
}

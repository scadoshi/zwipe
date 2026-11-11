//! The web's keyboard shortcuts (`context/plans/keyboard_shortcuts.md`): which key does what on
//! the swipe screens, in Printings and everywhere, the rows the shortcuts sheet lists on each
//! screen that offers it, and when a key reaches a screen at all. The listeners in `page_keys`
//! and `swipe_stack` run the guard as browser script built from the same constants, so the two
//! cannot drift.

use crate::inbound::cards::swipe_stack::Swipe;

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
    let shortcut = match key {
        "ArrowRight" | "l" | "L" => Shortcut::Swipe(Swipe::Right),
        "ArrowLeft" | "h" => Shortcut::Swipe(Swipe::Left),
        "ArrowUp" | "k" | "K" => Shortcut::Swipe(Swipe::Up),
        "ArrowDown" | "j" | "J" => Shortcut::Swipe(Swipe::Down),
        "p" | "P" => Shortcut::Printings,
        "d" | "D" => Shortcut::Details,
        "v" | "V" => Shortcut::Flip,
        "f" | "F" => Shortcut::Filter,
        "r" | "R" => Shortcut::Refresh,
        "s" | "S" => Shortcut::Synergy,
        _ => return None,
    };
    Some(shortcut)
}

/// What a key does in Printings' cover flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// What a key does on any screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalKey {
    /// Opens the page's own hint, as its `?` button does.
    Hint,
    /// Opens the shortcuts sheet.
    Shortcuts,
}

/// The global action `key` stands for: `?` the page's hint, Shift+H (`H`) the shortcuts sheet.
#[must_use]
pub fn global_key(key: &str) -> Option<GlobalKey> {
    match key {
        "?" => Some(GlobalKey::Hint),
        "H" => Some(GlobalKey::Shortcuts),
        _ => None,
    }
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
    /// The deck cards screen, whose keys act on the card under the cursor.
    DeckCards,
}

/// The rows the shortcuts sheet lists on `screen`: each key, with its vim letter if it has one,
/// and what it does, the screen's own keys first and then the keys every screen answers.
#[must_use]
pub fn sheet_rows(screen: &ShortcutsScreen) -> Vec<(&'static [&'static str], &'static str)> {
    let everywhere: [(&'static [&'static str], &'static str); 3] = [
        (&["Esc"], "Close the open sheet"),
        (&["?"], "This page's hint"),
        (&["Shift+H"], "These shortcuts"),
    ];
    let own: Vec<(&'static [&'static str], &'static str)> = match screen {
        ShortcutsScreen::Swipe(labels) => {
            let swipes = [
                Some((&["→", "l"][..], labels.right)),
                Some((&["←", "h"][..], labels.left)),
                labels.up.map(|up| (&["↑", "k"][..], up)),
                Some((&["↓", "j"][..], labels.down)),
            ];
            let rest: [(&'static [&'static str], &'static str); 5] = [
                (&["p"], "Printings"),
                (&["d"], "Card details"),
                (&["v"], "Flip card"),
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
        ShortcutsScreen::DeckCards => vec![
            (&["p"], "Printings"),
            (&["m"], "Maybeboard"),
            (&["s"], "Sideboard"),
            (&["Shift+S"], "Star"),
            (&["-"], "One fewer"),
            (&["+"], "One more"),
        ],
    };
    own.into_iter().chain(everywhere).collect()
}

/// The line the shortcuts sheet shows above the keys on `screen`, if it needs one.
#[must_use]
pub fn sheet_note(screen: &ShortcutsScreen) -> Option<&'static str> {
    match screen {
        ShortcutsScreen::Swipe(_) => None,
        ShortcutsScreen::DeckCards => Some("Hover a card, then press its key"),
    }
}

/// Element tags that take typing: a key pressed while one has focus never reaches a screen.
pub const TYPING_TAGS: [&str; 3] = ["INPUT", "TEXTAREA", "SELECT"];

/// Open overlays that keep the screen's keys to themselves: a dialog and a sheet or drawer.
pub const OVERLAY_SELECTORS: [&str; 2] =
    [".alert-dialog[data-state=\"open\"]", ".bottom-sheet.show"];

/// One keydown, as the browser saw it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeyPress {
    /// The focused element's tag, upper case as the DOM reports it.
    pub focus_tag: String,
    /// Whether the focused element is contenteditable.
    pub editable: bool,
    /// Whether Ctrl, Alt or Meta was held.
    pub modified: bool,
    /// Whether a dialog, sheet or drawer was open.
    pub overlay_open: bool,
}

/// Whether a keydown reaches the screen: not while typing in a field, not with Ctrl, Alt or Meta
/// held (those combos are the browser's), and not while a dialog, sheet or drawer is open (it
/// takes its own keys).
#[must_use]
pub fn reaches_screen(press: &KeyPress) -> bool {
    let typing = press.editable || TYPING_TAGS.contains(&press.focus_tag.as_str());
    !typing && !press.modified && !press.overlay_open
}

/// The browser-side guard, as a script expression that is true when `e` (a `KeyboardEvent`)
/// should reach the screen: [`reaches_screen`] over [`TYPING_TAGS`] and [`OVERLAY_SELECTORS`].
#[must_use]
pub fn guard_js() -> String {
    browser_guard(Modifiers::Refused)
}

/// [`guard_js`] without the modifier check, for a listener that wants a combo such as Ctrl+Z
/// and checks the modifiers itself.
#[must_use]
pub fn guard_js_with_modifiers() -> String {
    browser_guard(Modifiers::Allowed)
}

/// Whether the browser guard turns a key with Ctrl, Alt or Meta away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modifiers {
    /// A combo never reaches the screen.
    Refused,
    /// The listener checks the combo itself.
    Allowed,
}

/// The browser guard's expression under `modifiers`.
fn browser_guard(modifiers: Modifiers) -> String {
    let tags = TYPING_TAGS
        .iter()
        .map(|tag| format!("'{tag}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let overlays = OVERLAY_SELECTORS
        .iter()
        .map(|selector| format!("!document.querySelector('{selector}')"))
        .collect::<Vec<_>>()
        .join(" && ");
    let combo = match modifiers {
        Modifiers::Refused => " && !(e.ctrlKey || e.altKey || e.metaKey)",
        Modifiers::Allowed => "",
    };
    format!(
        "(() => {{ const f = document.activeElement; const typing = !!f && ([{tags}].includes(f.tagName) || f.isContentEditable); return !typing{combo} && !e.isComposing && {overlays}; }})()"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbound::screens::deck::cards::hover_keys;

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
                .take_while(|(keys, _)| keys.len() == 2)
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
    fn the_deck_cards_sheet_lists_only_that_screen_s_keys() {
        let rows = sheet_rows(&ShortcutsScreen::DeckCards);
        let listed: Vec<(&[&str], &str)> = rows.clone();
        assert_eq!(
            listed,
            [
                (&["p"][..], "Printings"),
                (&["m"][..], "Maybeboard"),
                (&["s"][..], "Sideboard"),
                (&["Shift+S"][..], "Star"),
                (&["-"][..], "One fewer"),
                (&["+"][..], "One more"),
                (&["Esc"][..], "Close the open sheet"),
                (&["?"][..], "This page's hint"),
                (&["Shift+H"][..], "These shortcuts"),
            ]
        );
        assert_eq!(
            sheet_note(&ShortcutsScreen::DeckCards),
            Some("Hover a card, then press its key")
        );
        assert_eq!(sheet_note(&ShortcutsScreen::Swipe(ADD_LABELS)), None);
        // Every listed key really does what its row says on the deck cards screen.
        for (keys, does) in &rows {
            for label in *keys {
                let key = if *label == "Shift+H" {
                    "H"
                } else if *label == "Shift+S" {
                    "S"
                } else {
                    label
                };
                let known = hover_keys::hover_key(key).is_some()
                    || global_key(key).is_some()
                    || key == "Esc";
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
                "Flip card",
                "Filter",
                "Refresh",
                "Synergy on or off",
                "Close the open sheet",
                "This page's hint",
                "These shortcuts",
            ]
        );
        assert_eq!(rows.first().map(|(keys, _)| *keys), Some(&["→", "l"][..]));
        // Every listed key really does what its row says.
        let typed = |label: &str| match label {
            "→" => "ArrowRight".to_owned(),
            "←" => "ArrowLeft".to_owned(),
            "↑" => "ArrowUp".to_owned(),
            "↓" => "ArrowDown".to_owned(),
            "Shift+H" => "H".to_owned(),
            other => other.to_owned(),
        };
        for (keys, does) in &rows {
            for label in *keys {
                let key = typed(label);
                let known = swipe_screen_shortcut(&key).is_some()
                    || global_key(&key).is_some()
                    || key == "Esc";
                assert!(known, "{label} for {does}");
            }
        }
    }

    #[test]
    fn typing_in_a_field_never_reaches_the_screen() {
        for tag in ["INPUT", "TEXTAREA", "SELECT"] {
            let press = KeyPress {
                focus_tag: tag.to_owned(),
                ..KeyPress::default()
            };
            assert!(!reaches_screen(&press), "{tag}");
        }
        let editable = KeyPress {
            focus_tag: "DIV".to_owned(),
            editable: true,
            ..KeyPress::default()
        };
        assert!(!reaches_screen(&editable), "contenteditable");
        for tag in ["BODY", "BUTTON", "A", "DIV"] {
            let press = KeyPress {
                focus_tag: tag.to_owned(),
                ..KeyPress::default()
            };
            assert!(reaches_screen(&press), "{tag}");
        }
    }

    #[test]
    fn modifier_combos_and_open_overlays_keep_the_key_from_the_screen() {
        let modified = KeyPress {
            focus_tag: "BODY".to_owned(),
            modified: true,
            ..KeyPress::default()
        };
        assert!(!reaches_screen(&modified), "Ctrl, Alt or Meta");
        let behind = KeyPress {
            focus_tag: "BODY".to_owned(),
            overlay_open: true,
            ..KeyPress::default()
        };
        assert!(!reaches_screen(&behind), "a sheet is open");
    }

    #[test]
    fn the_browser_guard_checks_every_typing_tag_and_overlay() {
        let js = guard_js();
        for tag in TYPING_TAGS {
            assert!(js.contains(&format!("'{tag}'")), "{tag} in {js}");
        }
        for selector in OVERLAY_SELECTORS {
            assert!(js.contains(selector), "{selector} in {js}");
        }
        for check in ["isContentEditable", "e.ctrlKey", "e.altKey", "e.metaKey"] {
            assert!(js.contains(check), "{check} in {js}");
        }
    }
}

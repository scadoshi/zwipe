//! The web's keyboard shortcuts (`context/plans/keyboard_shortcuts.md`): which key does what on
//! the swipe screens, in Printings and everywhere, the rows the shortcuts sheet lists on each
//! screen that offers it, and when a key reaches a screen at all. The listeners in `page_keys`
//! and `swipe_stack` run that guard as browser script built from the constants here.

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
    match card_key(key) {
        Some(CardKey::Filter) => return Some(Shortcut::Filter),
        Some(CardKey::Flip) => return Some(Shortcut::Flip),
        Some(CardKey::Details) => return Some(Shortcut::Details),
        None => {}
    }
    let shortcut = match key {
        "ArrowRight" | "l" | "L" => Shortcut::Swipe(Swipe::Right),
        "ArrowLeft" | "h" => Shortcut::Swipe(Swipe::Left),
        "ArrowUp" | "k" | "K" => Shortcut::Swipe(Swipe::Up),
        "ArrowDown" | "j" | "J" | "u" | "U" => Shortcut::Swipe(Swipe::Down),
        "p" | "P" => Shortcut::Printings,
        "r" | "R" => Shortcut::Refresh,
        "s" | "S" => Shortcut::Synergy,
        _ => return None,
    };
    Some(shortcut)
}

/// A key that means the same on every screen: `f` opens the screen's filter where it has one, `v`
/// flips the card on show or, a card with one face, shows its meld, and `d` opens or closes the
/// card's details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardKey {
    /// Opens the screen's filter.
    Filter,
    /// Flips the card on show, or toggles its meld.
    Flip,
    /// Opens or closes the card's details.
    Details,
}

/// The [`CardKey`] `key` stands for. Every screen's own key map defers to this one for `f` and `v`.
#[must_use]
pub fn card_key(key: &str) -> Option<CardKey> {
    match key {
        "f" | "F" => Some(CardKey::Filter),
        "v" | "V" => Some(CardKey::Flip),
        "d" | "D" => Some(CardKey::Details),
        _ => None,
    }
}

/// A script that clicks the first element inside `scope` (a selector) marked with one of `actions`
/// ([`components::data_action`]), in their order; nothing when it shows none.
#[must_use]
pub fn press_first_js(scope: &str, actions: &[&str]) -> String {
    format!(
        "{{ const found = {actions:?}.map((a) => document.querySelector('{scope} [data-action=\"' + a + '\"]')).find(Boolean); if (found) found.click(); }}"
    )
}

/// The letter keys that swipe on a swipe screen, beside the arrows ([`swipe_screen_shortcut`]).
pub const SWIPE_LETTERS: [&str; 9] = ["h", "j", "k", "l", "u", "J", "K", "L", "U"];

/// How the sheets write Ctrl+Z (Cmd+Z on a Mac), which undoes wherever `u` does except while
/// typing in a field, where the field keeps its own undo ([`super::page_keys::use_undo_key`]).
pub const UNDO_COMBO: &str = "Ctrl+Z";

/// Whether `key` is `u`, the deck cards screen's Undo, the same as its dock's Undo button.
#[must_use]
pub fn undo_letter(key: &str) -> bool {
    matches!(key, "u" | "U")
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
                Some((&["↓", "j", "u", UNDO_COMBO][..], labels.down)),
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
            (&["i"], "Image"),
            (&["v"], "Flip card"),
            (&["d"], "Card details"),
            (&["m"], "Maybeboard"),
            (&["s"], "Sideboard"),
            (&["Shift+S"], "Star"),
            (&["-"], "One fewer"),
            (&["+"], "One more"),
            (&["u", UNDO_COMBO], "Undo"),
            (&["f"], "Filter"),
        ],
        ShortcutsScreen::Examples => vec![
            (&["←", "h"], "Next"),
            (&["↓", "j"], "Back one"),
            (&["d"], "Card details"),
            (&["v"], "Flip card"),
        ],
    };
    own.into_iter().chain(everywhere).collect()
}

/// The line the shortcuts sheet shows above the keys on `screen`, if it needs one.
#[must_use]
pub fn sheet_note(screen: &ShortcutsScreen) -> Option<&'static str> {
    match screen {
        ShortcutsScreen::Swipe(_) | ShortcutsScreen::Examples => None,
        ShortcutsScreen::DeckCards => Some("Hover or open a card, then press its key"),
    }
}

/// Element tags that take typing: a key pressed while one has focus never reaches a screen.
pub const TYPING_TAGS: [&str; 3] = ["INPUT", "TEXTAREA", "SELECT"];

/// An open sheet or drawer.
const OPEN_SHEET: &str = ".bottom-sheet.show";

/// Open overlays that keep the screen's keys to themselves: a dialog and a sheet or drawer.
pub const OVERLAY_SELECTORS: [&str; 2] = [".alert-dialog[data-state=\"open\"]", OPEN_SHEET];

/// The class of the Stacks view's card details drawer, the one sheet the deck cards screen's
/// hover keys pass through ([`guard_js_through`]).
pub const STACK_DETAILS_CLASS: &str = "deck-stack-details";

/// The browser-side guard, as a script expression that is true when `e` (a `KeyboardEvent`)
/// should reach the screen: not while typing in one of [`TYPING_TAGS`] or a contenteditable, not
/// with Ctrl, Alt or Meta held (those combos are the browser's), and not while one of
/// [`OVERLAY_SELECTORS`] is open (it takes its own keys).
#[must_use]
pub fn guard_js() -> String {
    browser_guard(Modifiers::Refused, &overlays_closed(None))
}

/// [`guard_js`] without the modifier check, for a listener that wants a combo such as Ctrl+Z
/// and checks the modifiers itself.
#[must_use]
pub fn guard_js_with_modifiers() -> String {
    browser_guard(Modifiers::Allowed, &overlays_closed(None))
}

/// Whether the browser guard turns a key with Ctrl, Alt or Meta away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modifiers {
    /// A combo never reaches the screen.
    Refused,
    /// The listener checks the combo itself.
    Allowed,
}

/// [`guard_js`], except that while the open sheet is the one with `sheet_class`, `keys` still
/// reach the screen. Any other sheet or a dialog keeps every key, as [`guard_js`] does.
#[must_use]
pub fn guard_js_through(sheet_class: &str, keys: &[&str]) -> String {
    let keys = keys
        .iter()
        .map(|key| format!("'{key}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let passing = format!(
        "(!document.querySelector('{OPEN_SHEET}.{sheet_class}') || [{keys}].includes(e.key))"
    );
    let overlays = format!("{} && {passing}", overlays_closed(Some(sheet_class)));
    browser_guard(Modifiers::Refused, &overlays)
}

/// The expression that holds while no overlay in [`OVERLAY_SELECTORS`] is open, a sheet with
/// `except_class` not counting.
fn overlays_closed(except_class: Option<&str>) -> String {
    OVERLAY_SELECTORS
        .iter()
        .map(|selector| match except_class {
            Some(class) if *selector == OPEN_SHEET => {
                format!("!document.querySelector('{selector}:not(.{class})')")
            }
            _ => format!("!document.querySelector('{selector}')"),
        })
        .collect::<Vec<_>>()
        .join(" && ")
}

/// The open modals and sheets, outermost first: a dialog, a sheet or drawer, a large modal
/// (the Oracle tag dictionary and its examples) and Commander swipe.
pub const MODAL_SELECTORS: [&str; 4] = [
    ".alert-dialog[data-state=\"open\"]",
    OPEN_SHEET,
    ".screen.web-large-modal",
    ".swipe-select-screen.show",
];

/// A modal's or sheet's own `?`: the corner button of a modal's page header, or a sheet's
/// header hint.
pub const MODAL_HINT_BUTTON: &str =
    ".page-header > .page-header-corner, .sheet-header > .drawer-sheet-hint";

/// The guard for `?` while a modal may be open: not while typing in a field and not with Ctrl,
/// Alt or Meta held; an open modal answers it instead of keeping it.
#[must_use]
pub fn hint_guard_js() -> String {
    browser_guard(Modifiers::Refused, "true")
}

/// A script that presses the `?` of the topmost open modal or sheet ([`MODAL_SELECTORS`], the
/// last open one in the page) and sends whether one was open: `true` when a modal took the key,
/// even one with no hint, so the screen's hint opens only with no modal open.
#[must_use]
pub fn modal_hint_js() -> String {
    let modals = MODAL_SELECTORS.join(", ");
    format!(
        "(() => {{ const open = [...document.querySelectorAll('{modals}')]; const top = open[open.length - 1]; if (!top) return false; const hint = top.querySelector('{MODAL_HINT_BUTTON}'); if (hint) hint.click(); return true; }})()"
    )
}

/// The browser guard's expression under `modifiers`, with `overlays` the check that no overlay
/// keeps the key.
fn browser_guard(modifiers: Modifiers, overlays: &str) -> String {
    let tags = TYPING_TAGS
        .iter()
        .map(|tag| format!("'{tag}'"))
        .collect::<Vec<_>>()
        .join(", ");
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
            Some("Hover or open a card, then press its key")
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
                    || card_key(key).is_some()
                    || undo_letter(key)
                    || global_key(key).is_some()
                    || key == "Esc"
                    || key == UNDO_COMBO;
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
        assert_eq!(
            rows.iter()
                .find(|(_, does)| *does == "Undo")
                .map(|(keys, _)| *keys),
            Some(&["↓", "j", "u", UNDO_COMBO][..]),
            "u and Ctrl+Z undo beside the down swipe"
        );
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
                    || key == "Esc"
                    || key == UNDO_COMBO;
                assert!(known, "{label} for {does}");
            }
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
        for (keys, does) in sheet_rows(&ShortcutsScreen::Examples) {
            for label in keys {
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
                    || key == "Esc";
                assert!(works, "{label} for {does}");
            }
        }
    }

    #[test]
    fn the_flip_key_presses_flip_else_meld() {
        let js = press_first_js(
            ".swipe-stack-card.top",
            &components::data_action::FLIP_OR_MELD,
        );
        assert!(js.contains(r#"["flip", "meld"]"#), "{js}");
        assert!(js.contains(".swipe-stack-card.top [data-action="), "{js}");
    }

    #[test]
    fn the_swipe_letters_are_exactly_the_letters_that_swipe() {
        for key in printable_keys() {
            let swipes = matches!(swipe_screen_shortcut(&key), Some(Shortcut::Swipe(_)));
            assert_eq!(swipes, SWIPE_LETTERS.contains(&key.as_str()), "{key}");
        }
    }

    #[test]
    fn the_stack_details_drawer_lets_only_the_hover_keys_through() {
        let through = guard_js_through(STACK_DETAILS_CLASS, &hover_keys::HOVER_KEYS);
        assert!(
            through.contains(&format!(".bottom-sheet.show:not(.{STACK_DETAILS_CLASS})")),
            "every other sheet still blocks: {through}"
        );
        assert!(
            through.contains(".alert-dialog[data-state=\"open\"]"),
            "a dialog still blocks: {through}"
        );
        for key in hover_keys::HOVER_KEYS {
            assert!(through.contains(&format!("'{key}'")), "{key} in {through}");
        }
        assert!(!through.contains("'/'"), "/ stays blocked: {through}");
        assert!(
            !guard_js().contains(STACK_DETAILS_CLASS),
            "the other screens' guard lets nothing through"
        );
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

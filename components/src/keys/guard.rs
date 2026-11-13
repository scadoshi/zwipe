//! When a key reaches a screen at all, as browser script: not while typing in a field, not with
//! Ctrl, Alt or Meta held, and not while a dialog or sheet keeps its own keys. The listeners in
//! [`page_keys`](super::page_keys) and the shortcuts sheet run these guards, and the `?` key goes
//! to the topmost open modal's hint before the page's.

/// A script that clicks the first element inside `scope` (a selector) marked with one of `actions`
/// ([`crate::data_action`]), in their order; nothing when it shows none.
#[must_use]
pub fn press_first_js(scope: &str, actions: &[&str]) -> String {
    format!(
        "{{ const found = {actions:?}.map((a) => document.querySelector('{scope} [data-action=\"' + a + '\"]')).find(Boolean); if (found) found.click(); }}"
    )
}

/// The key a keydown `e` stands for, as a script expression: `KeyboardEvent.key`, with a letter's
/// case taken from Shift alone, so Caps Lock never turns `s` into Shift+S. The letter itself comes
/// from the layout, so a key keeps the letter printed on it.
pub const PRESSED_KEY: &str = "((k, shift) => k.length === 1 && k.toLowerCase() !== k.toUpperCase() ? (shift ? k.toUpperCase() : k.toLowerCase()) : k)(e.key, e.shiftKey)";

/// Element tags that take typing: a key pressed while one has focus never reaches a screen.
pub const TYPING_TAGS: [&str; 3] = ["INPUT", "TEXTAREA", "SELECT"];

/// An open sheet or drawer.
const OPEN_SHEET: &str = ".bottom-sheet.show";

/// The image preview, an overlay of its own that is neither a sheet nor a dialog.
const IMAGE_PREVIEW: &str = ".image-preview-container";

/// The guard for the deck page's cursor keys ([`guard_js`]'s typing and combo checks): they reach
/// the page while no modal is open but the sheet with `sheet_class`, the deck page's details
/// drawer, which they pass through, and no image preview shows.
#[must_use]
pub fn cursor_guard_js(sheet_class: &str) -> String {
    let modals = MODAL_SELECTORS.join(", ");
    let overlays = format!(
        "[...document.querySelectorAll('{modals}')].every((m) => m.matches('{OPEN_SHEET}.{sheet_class}')) && !document.querySelector('{IMAGE_PREVIEW}')"
    );
    browser_guard(Modifiers::Refused, &overlays)
}

/// The guard for an open sheet's own keys ([`guard_js`]'s typing and combo checks): they reach
/// the sheet while it is the topmost open modal, a shown sheet holding an element that matches
/// `holding` (a selector), so a dialog or another sheet over it keeps them.
#[must_use]
pub fn top_sheet_guard_js(holding: &str) -> String {
    let modals = MODAL_SELECTORS.join(", ");
    let on_top = format!(
        "(() => {{ const open = [...document.querySelectorAll('{modals}')]; const top = open[open.length - 1]; return !!top && top.matches('{OPEN_SHEET}') && !!top.querySelector('{holding}'); }})()"
    );
    browser_guard(Modifiers::Refused, &on_top)
}

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
    browser_guard(Modifiers::Refused, &overlays_closed_or_keyed_on_top())
}

/// [`guard_js`] without the modifier check, for a listener that wants a combo such as Ctrl+Z
/// and checks the modifiers itself.
#[must_use]
pub fn guard_js_with_modifiers() -> String {
    browser_guard(Modifiers::Allowed, &overlays_closed_or_keyed_on_top())
}

/// The modal that takes the screen's keys while it is the topmost one open, wherever it opens: the
/// Oracle tag examples, whose listeners register last, so they are the ones that answer, even
/// opened from a sheet (the filter's dictionary).
pub const KEYED_MODAL: &str = ".web-examples";

/// The expression that holds while no overlay is open, or while the topmost open modal
/// ([`MODAL_SELECTORS`]) is the [`KEYED_MODAL`].
fn overlays_closed_or_keyed_on_top() -> String {
    let modals = MODAL_SELECTORS.join(", ");
    format!(
        "({} || (() => {{ const open = [...document.querySelectorAll('{modals}')]; const top = open[open.length - 1]; return !!top && top.matches('{KEYED_MODAL}'); }})())",
        overlays_closed(None)
    )
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
        "(!document.querySelector('{OPEN_SHEET}.{sheet_class}') || [{keys}].includes({PRESSED_KEY}))"
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
    use zwipe_ui::deck_cards::keys as hover_keys;

    use super::*;

    #[test]
    fn a_letters_case_comes_from_shift_alone_so_caps_lock_changes_nothing() {
        assert!(PRESSED_KEY.contains("(e.key, e.shiftKey)"), "{PRESSED_KEY}");
        assert!(
            PRESSED_KEY.contains("shift ? k.toUpperCase() : k.toLowerCase()"),
            "{PRESSED_KEY}"
        );
        assert!(
            PRESSED_KEY.contains("k.toLowerCase() !== k.toUpperCase()"),
            "only a letter changes case: {PRESSED_KEY}"
        );
        let through = guard_js_through(STACK_DETAILS_CLASS, &["S"]);
        assert!(
            through.contains(&format!("['S'].includes({PRESSED_KEY})")),
            "{through}"
        );
    }

    #[test]
    fn the_cursor_keys_pass_only_the_details_drawer_and_never_a_modal_or_the_image() {
        let js = cursor_guard_js(STACK_DETAILS_CLASS);
        for modal in MODAL_SELECTORS {
            assert!(js.contains(modal), "{modal} keeps the keys: {js}");
        }
        assert!(
            js.contains(&format!(
                "m.matches('.bottom-sheet.show.{STACK_DETAILS_CLASS}')"
            )),
            "{js}"
        );
        assert!(
            js.contains("!document.querySelector('.image-preview-container')"),
            "{js}"
        );
        assert!(js.contains("e.ctrlKey || e.altKey || e.metaKey"), "{js}");
        assert!(js.contains("'INPUT'"), "never while typing: {js}");
    }

    #[test]
    fn the_flip_key_presses_flip_else_meld() {
        let js = press_first_js(".swipe-stack-card.top", &crate::data_action::FLIP_OR_MELD);
        assert!(js.contains(r#"["flip", "meld"]"#), "{js}");
        assert!(js.contains(".swipe-stack-card.top [data-action="), "{js}");
    }

    #[test]
    fn the_examples_take_their_keys_when_they_are_the_topmost_modal_even_over_a_sheet() {
        for js in [guard_js(), guard_js_with_modifiers()] {
            assert!(
                js.contains(&overlays_closed(None)),
                "no overlay still passes: {js}"
            );
            assert!(
                js.contains(&format!("top.matches('{KEYED_MODAL}')")),
                "the examples on top pass: {js}"
            );
            for modal in MODAL_SELECTORS {
                assert!(js.contains(modal), "{modal} counts toward the top: {js}");
            }
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

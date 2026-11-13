//! Enter runs an open sheet's main action. A sheet marks that button with [`SHEET_ENTER`], and
//! with [`SHEET_ENTER_OR_CLOSE`] when the button is off only because nothing changed, so Enter
//! then closes the sheet as its × does. [`SHEET_ENTER_JS`], loaded once in the page's head,
//! listens for the key.
//!
//! Enter reaches the sheet only while that sheet is the top open modal, no dialog is open, no
//! modifier is held, and focus is on nothing that answers Enter itself (a field, a button, a link,
//! a chip or another control). A sheet with no marked button, such as Printings, which keys its
//! own body, or the theme sheet, where Enter picks, never sees it from here.

/// The class of a sheet's main action, which Enter clicks while it is on.
pub const SHEET_ENTER: &str = "sheet-enter";

/// The classes of a sheet's main action that is off only while nothing changed (a filter's
/// Apply, a picker's Save): Enter clicks it while it is on and closes the sheet while it is off.
pub const SHEET_ENTER_OR_CLOSE: &str = "sheet-enter sheet-enter-closes";

/// The page-wide listener for [`SHEET_ENTER`]; see the module's doc. The top open modal is the
/// one with the highest z-index, the later one in the page on a tie, as a sheet opened last sits
/// on top. It runs after the focused element's own handlers and leaves a key they took.
pub const SHEET_ENTER_JS: &str = r#"
if (!window.__zwipeSheetEnter) {
    window.__zwipeSheetEnter = true;
    const MODALS = '.alert-dialog[data-state="open"], .bottom-sheet.show, .screen.web-large-modal, .swipe-select-screen.show';
    const CONTROLS = 'input, textarea, select, button, a[href], [contenteditable], [role="button"], [role="link"], [role="checkbox"], [role="radio"], [role="switch"], [role="tab"], [role="option"], [role="menuitem"], [role="combobox"], [role="slider"], .chip';
    const onControl = (el) => !!el && el !== document.body && !!el.closest(CONTROLS);
    const topModal = () => {
        let top = null;
        let topZ = -Infinity;
        for (const m of document.querySelectorAll(MODALS)) {
            const z = parseInt(getComputedStyle(m).zIndex, 10);
            const at = Number.isNaN(z) ? 0 : z;
            if (at >= topZ) { top = m; topZ = at; }
        }
        return top;
    };
    document.addEventListener('keydown', (e) => {
        if (e.key !== 'Enter' || e.isComposing || e.repeat || e.defaultPrevented) return;
        if (e.ctrlKey || e.altKey || e.metaKey || e.shiftKey) return;
        if (document.querySelector('.alert-dialog[data-state="open"]')) return;
        if (onControl(document.activeElement)) return;
        const sheet = topModal();
        if (!sheet || !sheet.matches('.bottom-sheet.show')) return;
        const action = sheet.querySelector('.sheet-enter');
        if (!action) return;
        if (!action.disabled) {
            e.preventDefault();
            action.click();
        } else if (action.classList.contains('sheet-enter-closes')) {
            const close = sheet.querySelector('.sheet-header .sheet-close, .sheet-header .sheet-back');
            if (close) { e.preventDefault(); close.click(); }
        }
    });
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_listener_clicks_the_classes_a_sheet_marks() {
        assert!(SHEET_ENTER_JS.contains(&format!("'.{SHEET_ENTER}'")));
        let closes = SHEET_ENTER_OR_CLOSE
            .strip_prefix(SHEET_ENTER)
            .map_or("", str::trim);
        assert!(
            SHEET_ENTER_JS.contains(&format!("classList.contains('{closes}')")),
            "{closes}"
        );
    }

    #[test]
    fn enter_on_a_control_or_with_a_modifier_is_left_alone() {
        for guard in [
            "if (onControl(document.activeElement)) return;",
            "if (e.ctrlKey || e.altKey || e.metaKey || e.shiftKey) return;",
            "e.defaultPrevented",
            "e.isComposing",
        ] {
            assert!(SHEET_ENTER_JS.contains(guard), "{guard}");
        }
        for control in ["input", "textarea", "button", ".chip", "[role=\"button\"]"] {
            assert!(SHEET_ENTER_JS.contains(control), "{control}");
        }
    }
}

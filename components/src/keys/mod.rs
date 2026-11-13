//! The keys a screen answers in the browser: the guard that decides whether a key reaches a
//! screen at all, the listeners screens register for their own keys and undo, the focus a keyed sheet takes as it opens, and the shortcuts
//! sheet with the keys that work everywhere (`?` and Shift+H). The key maps themselves are pure
//! and live in `zwipe_ui::shortcuts`.

mod guard;
mod page_keys;
mod sheet_focus;
mod shortcuts_sheet;

pub use guard::{
    KEYED_MODAL, MODAL_HINT_BUTTON, MODAL_SELECTORS, OVERLAY_SELECTORS, PRESSED_KEY,
    STACK_DETAILS_CLASS, TYPING_TAGS, cursor_guard_js, guard_js, guard_js_through,
    guard_js_with_modifiers, hint_guard_js, modal_hint_js, press_first_js, top_sheet_guard_js,
};
pub use page_keys::{
    focus_by_id, use_page_keys, use_page_keys_through, use_scoped_keys, use_undo_key,
};
pub use sheet_focus::{SheetFocus, use_sheet_focus, use_sheet_focus_handing_back};
pub use shortcuts_sheet::{
    SHORTCUTS_LABEL, ShortcutsList, ShortcutsOffered, ShortcutsOpen, ShortcutsSheet,
    shortcuts_action, use_global_keys, use_offer_shortcuts, use_shortcuts_action,
    use_shortcuts_root,
};

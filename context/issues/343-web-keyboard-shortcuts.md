# 343. Keyboard shortcuts on the web

State: fixing | Owner: thinker | PR: - | Build seen: release 1.18.17

## Report
Scotty, 2026-11-11 (feature, approved): keyboard shortcuts for the web, v1. The spec is `context/plans/keyboard_shortcuts.md`.

## Fix
thinker, 2026-11-11, on `nits/thinker-keyboard`. Web only (zwiper-web).
- **Swipe screens** (Add, Remove, Commander swipe): the arrows and `hjkl` swipe; `p` Printings, `d` card details, `f` Filter, `r` Refresh, `s` Synergy.
  - Each key calls the same callback as its dock button or swipe. Synergy's toggle became one callback for the bolt and `s`.
  - A key whose action isn't offered there does nothing: `s` on Remove or Commander swipe, `f` where the dock has no Filter.
  - `f` was Flip on Add and Remove; per the spec it is Filter now, and the flip key is gone.
- **Printings:** ← → and `h`/`l` move, Enter selects, Esc closes.
- **Everywhere:** Esc closes the open sheet, as it already did. `?` opens the page's own hint, the same callback as its `?` button: `ScreenFrame`'s `on_hint`, and Commander swipe's own. Shift+H opens the shortcuts sheet, which lists every key in the `HintKey` style inside the existing `BottomSheet` (a modal on a wide window).
- **Dock button:** the swipe screens' docks get a keyboard button that opens the same sheet. It shows only for `(hover: hover) and (pointer: fine)`.
- **Rules:**
  - Keys are off while a text field, select or contenteditable has focus.
  - Ctrl, Alt and Meta combos pass through to the browser.
  - An open dialog or sheet keeps the keys from the screen behind it.
- **Where it lives:** `inbound/chrome/shortcuts.rs` holds the key table, the cover-flow keys, the sheet's rows and the typing guard. The browser-side guard is built from the same constants, and the page, Ctrl+Z and swipe listeners all use it, which adds `<select>` to what counts as typing. Tests come first in that module.
- **Not checked live:** Commander swipe (inside the deck form flow) shares the hooks but wasn't driven headless.

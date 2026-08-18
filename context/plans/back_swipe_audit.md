# Back-swipe audit — overlays that exit the screen instead of closing

**Status: OPEN.** Confirmed broken in at least one place, with an inventory
below showing it is not a one-off. Needs one focused session, not a drive-by.

## Symptom

The OS back intent (iOS edge-swipe, Android gesture/button) sometimes blows
past an open overlay and navigates the router instead, throwing the user out of
the screen they were working in.

Owner's two examples, which are the whole bug in miniature:

- **Works:** Deck edit/create → Oracle tags → Dictionary → back-swipe → returns
  to the oracle-tag selection, as expected.
- **Broken:** Deck edit/create → Format selection → back-swipe → leaves the
  edit/create screen entirely. The format picker is never closed; the router
  goes back a route.

## The mechanism (why one works and the other doesn't)

`BackHandlerLayout` (`components/navigation/back_handler.rs`) receives the OS
back intent and does exactly this:

```rust
if !overlays.close_top() {
    if nav.can_go_back() { nav.go_back() } else { finish_activity() }
}
```

Overlays are not routes, so they can only participate by **registering a close
action** with `OverlayBackStack` while open — `use_overlay_back` for
signal-toggled overlays, `use_overlay_back_action` for callback-driven ones
(the shared `AlertDialogRoot` wrapper uses the latter, which is why every
AlertDialog-based dialog is already safe).

**Registration is opt-in and manual.** An overlay that forgets the hook is
invisible to `close_top()`, so back falls straight through to `go_back()`. That
is the bug, and it is a whole class rather than a single defect.

## Inventory (2026-08-17)

**Registered — these behave correctly:**

| Overlay | File |
|---|---|
| All AlertDialog-based dialogs | `components/alert_dialog/component.rs` |
| Card filter sheet | `screens/deck/card/filter/card_filter_sheet.rs` |
| Oracle tag select | `screens/deck/components/oracle_tag_select.rs` |
| Swipe select | `screens/deck/components/swipe_select.rs` |
| Oracle tag dictionary | `screens/oracle_tag_dictionary.rs` |
| Oracle tag examples | `screens/oracle_tag_examples.rs` |

**Confirmed gaps:**

- **`components/bottom_sheet.rs` — the shared `BottomSheet` does not register.**
  This is the big one: it is used by the deck view's More menu
  (`more_buttons.rs`), the deck list, the commander maybeboard, profile, and
  preferences. One unregistered primitive, at least six screens affected.
- **`screens/deck/components/format_select.rs`** — the reported bug. It renders
  a full-screen in-place overlay (`class: "screen-content … tag-screen"`),
  structurally identical to `oracle_tag_select`, which *does* register. A
  straight copy-paste omission.

**To verify during the pass** (all own an `open: Signal<bool>`; some may be
AlertDialog-based and therefore already covered):

`screens/deck/components/tag_select.rs`,
`screens/deck/card/components/card_info.rs`,
`screens/deck/card/components/printing_sheet.rs`,
`screens/deck/card/filter/oracle_tags.rs`,
`screens/deck/components/clone_deck_dialog.rs`,
`components/logout_dialog.rs`,
`screens/profile/change_email.rs`, `change_password.rs`, `change_username.rs`,
`screens/profile/components/delete_account_dialog.rs`,
`screens/profile/preferences.rs`.

(`components/hint_dialog.rs` is built on `AlertDialogRoot` — already covered.)

## Fix strategy — make it structural, not a checklist

Registering each missing overlay one by one fixes today's bugs and guarantees
tomorrow's: the next overlay anyone adds will forget the hook exactly the same
way. Two layers:

1. **Register inside the shared primitives.** `BottomSheet` should register on
   behalf of every sheet, the way `AlertDialogRoot` already does for dialogs.
   That converts "six screens to fix" into one change, and makes future sheets
   correct by construction.
2. **Then the one-offs.** Full-screen in-place overlays like `format_select`
   have no shared primitive to hang it on. Either give them one (a
   `ScreenOverlay` wrapper that owns the `.screen-content` markup *and* the
   registration), or register individually and accept the recurrence risk.
   The wrapper is the better answer for the same reason the skeleton now
   borrows the live list's classes: one definition, no drift.

## Test procedure

Per overlay, on a device (back-swipe is not reproducible in a browser):

1. Open the screen, open the overlay.
2. Back-swipe once → the overlay should close and the screen stay put.
3. Back-swipe again → now the screen should exit.
4. Nested case: open an overlay from inside an overlay (the dictionary case
   above) and confirm back unwinds one level at a time, innermost first.
5. Root case: back-swipe at a root screen with nothing open → iOS no-ops,
   Android exits the app. Neither should crash.

Worth logging results in a table as the pass goes, since the point is coverage
rather than fixing the one reported case.

## Notes

- iOS and Android share `close_top()` but reach it through different bridges
  (a `UIScreenEdgePanGestureRecognizer` vs the patched `MainActivity`
  dispatching a `zwipe:back` DOM event). Test **both** — a registration fix is
  shared, but a bridge-level bug would not be.
- Android's bridge depends on `zcripts/android/back_handler.sh` having been run
  post-bundle; if back does nothing at all on Android, suspect the build step
  before suspecting this code. See
  [`android_resume_crash.md`](android_resume_crash.md), where the same manual
  step is hypothesis 1.

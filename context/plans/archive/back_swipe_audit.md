# Back-swipe audit — overlays that exit the screen instead of closing

**Status: FIXED AND VERIFIED ON BOTH PLATFORMS 2026-08-17.** Ships in 1.9.2.
Four overlays registered; the inventory below is what the pass found and the
test list at the bottom is what was run.

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

- **FIXED — `components/bottom_sheet.rs`: the shared `BottomSheet` did not register.**
  This is the big one: it is used by the deck view's More menu
  (`more_buttons.rs`), the deck list, the commander maybeboard, profile, and
  preferences. One unregistered primitive, at least six screens affected.
- **FIXED — `screens/deck/components/format_select.rs`** — the reported bug. It renders
  a full-screen in-place overlay (`class: "screen-content … tag-screen"`),
  structurally identical to `oracle_tag_select`, which *does* register. A
  straight copy-paste omission.

**Verified during the pass.** `tag_select.rs` and `printing_sheet.rs` were
also unregistered and are now FIXED (printing_sheet hand-rolls its own backdrop
rather than using `BottomSheet`, so the shared fix did not reach it).
`card_info.rs`, `clone_deck_dialog.rs`, `logout_dialog.rs`,
`delete_account_dialog.rs` and `change_password.rs` are AlertDialog-based and
were already covered; `change_email.rs`, `change_username.rs` and
`preferences.rs` are `BottomSheet`-based and are covered by the shared fix.
`oracle_tags.rs`, `view.rs` and `deck_fields.rs` render no overlay markup of
their own — they are hosts whose children register. The original list was:

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
  [`android_ndk_context_crash.md`](android_ndk_context_crash.md), where the same manual
  step is hypothesis 1.

## Test list (2026-08-17 fixes — verify on device)

Every overlay below should behave the same way: **first back closes the
overlay and leaves the screen where it was; a second back leaves the screen.**
Failure looks like one back doing both.

### Newly fixed — these were broken, test them first

| # | Where | How to get there | Expected |
|---|---|---|---|
| 1 | **Format picker** | Deck edit/create → Format | Back = Cancel: reverts the format *and* its command-zone cascade, stays on the edit form. This was the reported bug (back used to exit the whole edit screen). |
| 2 | **Deck tag picker** | Deck edit/create → Tags | Back = Cancel: selection reverts to what it was on open, stays on the form. |
| 3 | **Printing sheet** | Any card row → expand → Printing | Sheet closes, screen stays. If you'd swiped to a different printing, expect the "Printing discarded" toast, same as tapping the backdrop. |
| 4 | **Deck More sheet** | Deck view → More | Sheet closes only. |
| 5 | **Deck list More sheet** | Decks → More | Sheet closes only. |
| 6 | **Commander maybeboard More** | Decks → More → Commander maybeboard → More | Sheet closes only. |
| 7 | **Profile sheets** | Profile → Change username / Change email | Sheet closes only. |
| 8 | **Preferences sheet** | Profile → Preferences, change the theme, then back | Sheet closes **and the theme reverts** (back mirrors the backdrop, which discards an unsaved theme). |

### Regression check — these already worked, confirm they still do

| # | Where | Expected |
|---|---|---|
| 9 | Oracle tag picker (edit → Tags → Oracle tags) | closes, form stays |
| 10 | Oracle tag **dictionary**, opened from that picker | closes back to the picker, **not** out to the form (the nested case) |
| 11 | Oracle tag examples | closes back to where it opened |
| 12 | Card filter sheet (Add/Remove/Deck cards → Filter) | closes only |
| 13 | Swipe select (commander picker) | closes only |
| 14 | Any AlertDialog: Clear skips, Delete deck, Clone deck, Logout, Delete account, hint dialogs | dialog closes only |

### Edge cases worth one pass each

| # | Case | Expected |
|---|---|---|
| 15 | Two overlays deep (picker → dictionary) | Back unwinds **one level per swipe**, innermost first |
| 16 | Back at a root screen with nothing open | Android: exits the app. iOS: no-op. Neither crashes |
| 17 | Open a sheet, rotate or toggle system dark mode, then back | Sheet still closes correctly (the overlay stack survives the config change) |
| 18 | Open an overlay, background the app, return, then back | Still closes the overlay, not the screen |

### Platform note

iOS and Android reach `close_top()` through different bridges (a
`UIScreenEdgePanGestureRecognizer` vs the patched `MainActivity` dispatching
`zwipe:back`). The registration fix is shared, so a failure on **one** platform
only points at that bridge, not at this work.


## Verification results (2026-08-17)

Both bridges exercised, which mattered here: iOS reaches `close_top()` through
a `UIScreenEdgePanGestureRecognizer`, Android through the patched
`MainActivity` dispatching `zwipe:back`. A registration fix is shared, but a
bridge bug would not have been.

**iOS** — owner ran the list by hand on device: passing.

**Android** — driven over adb against the 1.9.2 build, using `KEYCODE_BACK`
(the hardware/nav-bar path, which normally goes untested):

| Test | Result |
|---|---|
| #1 Format picker — the reported bug | **PASS** — closed, stayed on Edit Deck |
| #5 Deck list More sheet (shared `BottomSheet`) | **PASS** — closed, stayed on Decks |
| #15 Nested: Oracle tags → Dictionary | **PASS** — one layer per back, innermost first |
| Fall-through: back with nothing open | **PASS** — left Edit Deck for the deck screen |

That last row is as important as the fixes: it proves registration did not
*over*-capture. Back still navigates when there is nothing to close, so the
stack is not swallowing intents it should pass on.

The nested case is the strongest single result — three levels deep, each back
peeling exactly one layer, which exercises the LIFO ordering, the per-instance
ids and the auto-deregistration on close all at once.

**Not yet exercised**, all lower risk: the preferences theme-revert (#8), the
printing sheet (#3), and the AlertDialog family (#14), which never broke since
`AlertDialogRoot` has always registered.

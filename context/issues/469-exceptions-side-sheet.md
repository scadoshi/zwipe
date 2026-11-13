# 469. On a wide window, Exceptions opens as a centered modal, and its Change is off with no reason

State: fixing | Owner: laptop | Type: issue | PR: - (pushed branch, hash in the composer's log) | Build seen: release ed91ed2d6

## Report
Scotty, via the composer, on the wide web: Settings > Exceptions opens as a centered modal; make it a side sheet like the other pickers. Phone web (bottom sheet) and the app are fine. Also make the Change button's disabled state explain itself while Universes Beyond is set to Show, with a short reason under it.

## Before
surfer (`before-469`, ed91ed2d6, 1440), 3 of 3:
- with Universes Beyond on Show, Change is disabled (opacity 0.5) with no title, no aria and no visible reason; only the row's `?` explains it;
- on Hide, Exceptions opens centered, 464 to 976px of 1440, unlike the pickers' side sheets (1040 to 1424).

## Cause
- `zwiper-web/src/lib/inbound/screens/config/exceptions.rs` wraps its sheet in `SheetFrame { wide: SheetWide::Modal }`.
- The Exceptions row in `config/mod.rs` disables Change while Universes Beyond shows and fades the whole row (`ub-chips-disabled`), with no reason drawn.

## Fix
laptop:
- **Side sheet:** the sheet's frame is `SheetWide::Drawer`, the pickers' side sheet. A bottom sheet below the breakpoint is unchanged.
- **Reason:** a line, "Set Universes Beyond to Hide to choose exceptions", sits under the row at full strength while the setting has loaded and Universes Beyond shows. It comes from `zwipe_ui::screens::config::exceptions_locked` and `EXCEPTIONS_LOCKED`, and the row's rule gives way to it.
- **Test:** `change_says_why_it_is_off_only_while_universes_beyond_shows` (red first).

## Checked
Local web, signed in:
- at 1440, Show leaves Change off with the reason at opacity 1;
- Hide turns Change on with no reason, and Exceptions opens at 1040 to 1424 (the drawer);
- at 390 the reason wraps under the row with no rule between them.

## Verification
-
